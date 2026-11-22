use crate::{
    config::Config,
    git::Git,
    governor::{Governor, MemoryPressure},
    store::Store,
    sync,
};
use std::sync::Arc;
use std::time::Duration;

pub fn dispatch_one(store: &Store, git: &Git, cfg: &Config) -> anyhow::Result<bool> {
    // Tenta obter um lease para o próximo job com TTL de 600 segundos
    if let Some((id, kind, repo_id, payload_str)) = store.lease_next(600)? {
        let payload: serde_json::Value =
            serde_json::from_str(&payload_str).unwrap_or(serde_json::Value::Null);

        let res = match kind.as_str() {
            "fetch" => sync::fetch(store, git, repo_id, &payload),
            "branch_sync" => sync::branch_sync(store, git, repo_id, &payload),
            "push_mirror" => sync::push_mirror(store, git, repo_id, &payload),
            "slice_plan" => crate::slice::plan(store, git, repo_id, &payload).map(|_| ()),
            "slice_exec" => crate::slice::exec(store, git, cfg, repo_id, &payload),
            "path_sync" => crate::path_sync::path_sync(store, git, repo_id, &payload),
            "merge" => crate::merge::run(store, git, repo_id, &payload),
            "gate" => crate::gate::run_repo(store, git, cfg, repo_id, &payload),
            other => {
                let err_msg = format!("Tipo de job desconhecido: {}", other);
                store.finish(id, Some(&err_msg))?;
                return Ok(true);
            }
        };

        match res {
            Ok(_) => store.finish(id, None)?,
            Err(e) => store.finish(id, Some(&e.to_string()))?,
        }

        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn run(store: Arc<Store>, cfg: Config) -> anyhow::Result<()> {
    let git = Git::new(cfg.git_child_cap_mb);
    let governor = Governor::new(&cfg);

    loop {
        // Verifica o Memory Governor antes de processar novos jobs
        match governor.check() {
            MemoryPressure::HardLimit => {
                eprintln!(
                    "[Governor] Hard limit atingido ({} MB). Pausando workers...",
                    governor.current_rss_mb()
                );
                std::thread::sleep(Duration::from_millis(1000));
                continue;
            }
            MemoryPressure::SoftLimit => {
                eprintln!(
                    "[Governor] Soft limit atingido ({} MB). Throttling...",
                    governor.current_rss_mb()
                );
                std::thread::sleep(Duration::from_millis(500));
            }
            MemoryPressure::Normal => {}
        }

        let processed = dispatch_one(&store, &git, &cfg)?;
        if !processed {
            // Fila vazia; encerra o ciclo de execução do comando
            break;
        }
    }

    Ok(())
}

/// Mantém o dispatcher ativo para o binário `colmeiad`, com cron (F03) e
/// hot-subset watcher (F04) em threads auxiliares.
pub fn run_daemon(store: Arc<Store>, cfg: Config) -> anyhow::Result<()> {
    crate::cadence::run_daemon(store.clone(), Duration::from_secs(30));
    crate::watch::run_daemon(store.clone(), cfg.hot_watch_count, Duration::from_secs(2));
    loop {
        run(store.clone(), cfg.clone())?;
        std::thread::sleep(Duration::from_secs(1));
    }
}
