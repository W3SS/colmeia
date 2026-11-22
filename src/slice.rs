use crate::{
    config::Config,
    gate::blob_sha,
    git::{Git, GitBackend},
    rules, store::Store, worktrees,
};
use serde_json::Value;
use std::path::Path;

pub fn partition_units(units: &[(String, usize)], max_lines: usize) -> Vec<Vec<String>> {
    let mut sorted_units = units.to_vec();

    // Ordenação estável + desempate criptográfico determinístico (RFC 0001)
    sorted_units.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| blob_sha(&a.0).cmp(&blob_sha(&b.0)))
    });

    let mut slices: Vec<Vec<String>> = vec![vec![]];
    let mut current_accum = 0;

    for (path, lines) in sorted_units {
        if current_accum + lines > max_lines && !slices.last().unwrap().is_empty() {
            slices.push(vec![]);
            current_accum = 0;
        }
        slices.last_mut().unwrap().push(path);
        current_accum += lines;
    }

    slices
}

pub fn plan(store: &Store, git: &Git, repo_id: i64, payload: &Value) -> anyhow::Result<i64> {
    let p = store.get_repo_path(repo_id)?;
    let commit_arg = payload["commit"].as_str().unwrap_or("HEAD");
    let max_lines = payload["max_lines"].as_u64().unwrap_or(400) as usize;

    // Resolve o alvo para SHA completo na criação do plano: o plano fica
    // pinado (determinismo) e não depende de qual HEAD o worktree tem.
    let rev = git.run(
        Path::new(&p),
        &["rev-parse", "--verify", &format!("{commit_arg}^{{commit}}")],
    )?;
    anyhow::ensure!(
        rev.status.success(),
        "commit {commit_arg} não encontrado: {}",
        String::from_utf8_lossy(&rev.stderr)
    );
    let commit = String::from_utf8_lossy(&rev.stdout).trim().to_string();

    let units = git.numstat(Path::new(&p), &commit)?;
    let slices = partition_units(&units, max_lines);

    let plan_val = serde_json::json!({
        "commit": commit,
        "slices": slices,
        "max_lines": max_lines,
    });
    let plan_str = plan_val.to_string();
    let plan_hash = blob_sha(&plan_str);

    let id = store.add_recipe(repo_id, "slice", &commit, &plan_hash, &plan_hash, &plan_str)?;

    store.event(repo_id, "slice.plan", &plan_val)?;
    Ok(id)
}

// ── F12: exec · F16: replay ────────────────────────────────────────────────

/// Executa um plano de fatias: recria cada fatia como commit atômico sobre o
/// pai do commit original, com gate por fatia, backup ref e CAS no avanço do
/// branch. Determinístico (autor/datas herdados do commit original), o que
/// torna o replay bit-idêntico (F16).
pub fn exec(
    store: &Store,
    git: &Git,
    cfg: &Config,
    repo_id: i64,
    payload: &Value,
) -> anyhow::Result<()> {
    let recipe_id = payload["plan_id"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("slice_exec exige 'plan_id' (id da recipe)"))?;
    let (_, kind, plan_str, _, _) = store
        .get_recipe(recipe_id)?
        .ok_or_else(|| anyhow::anyhow!("recipe {recipe_id} não encontrada"))?;
    anyhow::ensure!(kind == "slice", "recipe {recipe_id} não é um plano slice");
    let plan: Value = serde_json::from_str(&plan_str)?;
    run_plan(store, git, cfg, repo_id, &plan, recipe_id, payload)
}

/// Re-executa um plano já gravado e exige resultado bit-idêntico.
pub fn replay(
    store: &Store,
    git: &Git,
    cfg: &Config,
    repo_id: i64,
    recipe_id: i64,
) -> anyhow::Result<()> {
    let (_, kind, plan_str, _, expected_output) = store
        .get_recipe(recipe_id)?
        .ok_or_else(|| anyhow::anyhow!("recipe {recipe_id} não encontrada"))?;
    anyhow::ensure!(kind == "slice", "recipe {recipe_id} não é um plano slice");
    let plan: Value = serde_json::from_str(&plan_str)?;
    let payload = serde_json::json!({ "plan_id": recipe_id, "dry_run": true });
    run_plan(store, git, cfg, repo_id, &plan, recipe_id, &payload)?;
    let (_, _, _, _, new_output) = store.get_recipe(recipe_id)?.unwrap();
    anyhow::ensure!(
        new_output == expected_output,
        "replay divergiu: esperado {expected_output}, obteve {new_output}"
    );
    store.event(
        repo_id,
        "slice.replay_ok",
        &serde_json::json!({ "recipe": recipe_id, "output_hash": new_output }),
    )?;
    Ok(())
}

fn run_plan(
    store: &Store,
    git: &Git,
    cfg: &Config,
    repo_id: i64,
    plan: &Value,
    recipe_id: i64,
    payload: &Value,
) -> anyhow::Result<()> {
    let commit = plan["commit"].as_str().unwrap_or("HEAD");
    let slices = plan["slices"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("plano sem 'slices'"))?;
    let dry_run = payload["dry_run"].as_bool().unwrap_or(false);
    let repo_path = store.get_repo_path(repo_id)?;
    let repo = Path::new(&repo_path);

    // Branch alvo e CAS base
    let branch = {
        let c = store.conn.lock().unwrap();
        c.query_row(
            "SELECT COALESCE(default_branch, 'main') FROM repos WHERE id=?1",
            [repo_id],
            |r| r.get::<_, String>(0),
        )?
    };
    let old_head_out = git.run(repo, &["rev-parse", &format!("refs/heads/{branch}")])?;
    anyhow::ensure!(
        old_head_out.status.success(),
        "branch {branch} não existe em {repo_path}"
    );
    let old_head = String::from_utf8_lossy(&old_head_out.stdout).trim().to_string();

    // Metadados do commit original (determinismo do replay)
    let meta_out = git.run(repo, &["show", "-s", "--format=%an%x00%ae%x00%ad%x00%cn%x00%ce", commit])?;
    anyhow::ensure!(meta_out.status.success(), "commit {commit} não encontrado");
    let meta = String::from_utf8_lossy(&meta_out.stdout);
    let parts: Vec<&str> = meta.trim().split('\x00').collect();
    anyhow::ensure!(parts.len() >= 5, "metadados do commit ilegíveis");
    let (an, ae, ad, cn, ce) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

    let base = format!("{commit}^");
    let wt = worktrees::acquire(
        store,
        git,
        repo_id,
        commit,
        &base,
        "slice",
        cfg.worktree_max.max(1),
    )?;

    // Reuso do worktree: volta ao estado base antes de aplicar as fatias
    // (sem isso, um re-exec/replay encontra o conteúdo já aplicado e o
    // commit da fatia falha com "nothing to commit").
    let out = git.run(&wt, &["reset", "--hard", &base])?;
    anyhow::ensure!(
        out.status.success(),
        "reset do worktree falhou: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = git.run(&wt, &["clean", "-fd"]);

    let n = slices.len();
    let mut slice_shas: Vec<String> = Vec::new();
    let mut total_findings = 0usize;

    for (i, slice) in slices.iter().enumerate() {
        let files: Vec<String> = slice
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|f| f.as_str().map(String::from))
            .collect();
        if files.is_empty() {
            continue;
        }
        // Aplica o conteúdo do commit original apenas para os arquivos da fatia
        let mut args = vec!["checkout", commit, "--"];
        args.extend(files.iter().map(|s| s.as_str()));
        let out = git.run(&wt, &args)?;
        anyhow::ensure!(
            out.status.success(),
            "checkout da fatia {i} falhou: {}",
            String::from_utf8_lossy(&out.stderr)
        );

        // Gate por fatia (SAST in-core sobre os arquivos da fatia)
        for rel in &files {
            let full = wt.join(rel);
            if !full.is_file() {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&full) {
                if text.len() > 1_000_000 {
                    continue;
                }
                let blob = blob_sha(&text);
                for fd in rules::scan(rel, &text) {
                    store.upsert_finding(repo_id, &blob, fd.rule, fd.sev, rel, fd.line as i64, &fd.msg, fd.owasp)?;
                    total_findings += 1;
                }
            }
        }

        let short = &blob_sha(&files.join("\n"))[..12];
        let message = format!("colmeia slice {}/{n} of {commit} ({short})", i + 1);
        let out = git.run_with_env(
            &wt,
            &["commit", "--no-edit", "-m", &message],
            &[
                ("GIT_AUTHOR_NAME", an),
                ("GIT_AUTHOR_EMAIL", ae),
                ("GIT_AUTHOR_DATE", ad),
                ("GIT_COMMITTER_NAME", cn),
                ("GIT_COMMITTER_EMAIL", ce),
                ("GIT_COMMITTER_DATE", ad),
                ("GIT_MERGE_AUTOEDIT", "no"),
            ],
        )?;
        anyhow::ensure!(
            out.status.success(),
            "commit da fatia {} falhou: {} | stdout: {}",
            i + 1,
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        );
        slice_shas.push(git.head(&wt)?);
    }

    anyhow::ensure!(!slice_shas.is_empty(), "plano não produziu fatias");
    let new_tip = slice_shas.last().unwrap().clone();
    let output_hash = blob_sha(&slice_shas.join("\n"));
    store.update_recipe_output(recipe_id, &output_hash)?;

    if !dry_run {
        // Backup ref antes do CAS
        let _ = git.run(
            repo,
            &[
                "update-ref",
                &format!("refs/colmeia/backup/{branch}"),
                &old_head,
            ],
        );
        // CAS: só avança se o branch ainda estiver onde esperávamos
        let out = git.run(
            repo,
            &["update-ref", &format!("refs/heads/{branch}"), &new_tip, &old_head],
        )?;
        anyhow::ensure!(
            out.status.success(),
            "CAS falhou: branch {branch} moveu durante o exec (backup em refs/colmeia/backup/{branch})"
        );
        store.update_repo_head(repo_id, &new_tip)?;
    }

    worktrees::release(store, &wt)?;
    store.event(
        repo_id,
        if dry_run { "slice.dry_run" } else { "slice.exec" },
        &serde_json::json!({
            "recipe": recipe_id,
            "slices": n,
            "findings": total_findings,
            "output_hash": output_hash,
            "tip": new_tip,
        }),
    )?;
    Ok(())
}
