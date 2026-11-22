use crate::{
    git::{Git, GitBackend},
    store::Store,
};
use serde_json::Value;
use std::path::Path;

pub fn run(store: &Store, git: &Git, repo_id: i64, payload: &Value) -> anyhow::Result<()> {
    let p = store.get_repo_path(repo_id)?;
    let refspec = payload["ref"].as_str().unwrap_or("origin/main");
    let policy = payload["policy"].as_str().unwrap_or("escalate");
    let path = Path::new(&p);

    // 1. Tenta fast-forward limpo primeiro
    if git.ff_merge(path, refspec)? {
        let sha = git.head(path)?;
        store.update_repo_head(repo_id, &sha)?;
        store.event(repo_id, "merge.ff", &serde_json::json!({ "ref": refspec }))?;
        return Ok(());
    }

    // 2. Aplica política determinística de reconciliação
    match policy {
        "ours" | "theirs" => {
            // A1: --no-edit + GIT_MERGE_AUTOEDIT=no garantem execução headless —
            // sem isso o git abre um editor para a mensagem e o job trava em daemon.
            let out = git.run_with_env(
                path,
                &["merge", "--no-edit", "-X", policy, refspec],
                &[("GIT_MERGE_AUTOEDIT", "no")],
            )?;
            anyhow::ensure!(
                out.status.success(),
                "Git merge -X {} falhou: {}",
                policy,
                String::from_utf8_lossy(&out.stderr)
            );
            let sha = git.head(path)?;
            store.update_repo_head(repo_id, &sha)?;
            store.event(
                repo_id,
                "merge.driver",
                &serde_json::json!({ "driver": policy, "ref": refspec }),
            )?;
        }
        _ => {
            // Em caso de política 'escalate' ou conflito que requer decisão humana: aborta
            let _ = git.run(path, &["merge", "--abort"]);
            store.event(
                repo_id,
                "merge.conflict",
                &serde_json::json!({ "ref": refspec, "needs": "human" }),
            )?;
        }
    }

    Ok(())
}
