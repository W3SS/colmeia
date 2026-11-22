use crate::git::{Git, GitBackend};
use crate::store::Store;
use serde_json::Value;
use std::path::Path;

fn repo_path(store: &Store, repo_id: i64) -> anyhow::Result<String> {
    store.get_repo_path(repo_id)
}

pub fn fetch(store: &Store, git: &Git, repo_id: i64, payload: &Value) -> anyhow::Result<()> {
    let p = repo_path(store, repo_id)?;
    let remote = payload["remote"].as_str().unwrap_or("origin");
    let path = Path::new(&p);

    git.fetch(path, remote)?;
    let sha = git.head(path)?;

    store.update_repo_head(repo_id, &sha)?;
    store.event(repo_id, "sync.ok", &serde_json::json!({ "sha": sha }))?;
    Ok(())
}

pub fn branch_sync(store: &Store, git: &Git, repo_id: i64, payload: &Value) -> anyhow::Result<()> {
    let p = repo_path(store, repo_id)?;
    let refspec = payload["refspec"].as_str().unwrap_or("origin/main");
    let path = Path::new(&p);

    if git.ff_merge(path, refspec)? {
        let sha = git.head(path)?;
        store.update_repo_head(repo_id, &sha)?;
        store.event(repo_id, "sync.ff", &serde_json::json!({ "ref": refspec }))?;
    } else {
        // Divergência detectada: escala para merge determinístico, nunca força push ou merge cego
        store.enqueue(
            "merge",
            repo_id,
            &serde_json::json!({ "ref": refspec, "policy": "escalate" }),
            3,
        )?;
        store.event(
            repo_id,
            "sync.diverged",
            &serde_json::json!({ "ref": refspec }),
        )?;
    }
    Ok(())
}

pub fn push_mirror(store: &Store, git: &Git, repo_id: i64, payload: &Value) -> anyhow::Result<()> {
    let p = repo_path(store, repo_id)?;
    let dst = payload["dst"].as_str().unwrap_or("mirror");
    let path = Path::new(&p);

    let out = git.run(path, &["push", "--mirror", dst])?;
    anyhow::ensure!(
        out.status.success(),
        "Push mirror falhou: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    store.event(repo_id, "sync.mirror", &serde_json::json!({ "dst": dst }))?;
    Ok(())
}
