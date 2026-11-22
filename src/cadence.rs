//! F03 — Scheduler adaptativo: cron sobre as políticas de sync.
//!
//! Lê `sync_policies.cron` (expressão cron de 6 campos com segundos),
//! enfileira o job correspondente quando a expressão dispara e registra
//! `last_run` para não repetir dentro do mesmo minuto.

use crate::store::{PolicyRecord, Store};
use cron::Schedule;
use std::str::FromStr;
use std::sync::Arc;

/// Uma política cuja expressão cron disparou na janela recente e ainda não
/// foi executada. Usa janela de 90 s para o tick do daemon não perder um
/// disparo que aconteceu logo antes dele acordar; `last_run` evita duplicar.
pub fn is_due(cron_expr: &str, last_run: Option<i64>) -> bool {
    let Ok(schedule) = Schedule::from_str(cron_expr) else {
        return false;
    };
    let now = chrono::Utc::now();
    let now_ts = now.timestamp();
    // Primeira ocorrência depois de (agora − 90 s)
    let from = now - chrono::Duration::seconds(90);
    let Some(next) = schedule.after(&from).next() else {
        return false;
    };
    let t = next.timestamp();
    if t > now_ts {
        return false; // ainda não chegou na janela
    }
    // Disparou em `t`; já executamos esse disparo?
    !last_run.map_or(false, |last| t <= last)
}

/// Executa um tick do cron: enfileira jobs das políticas vencidas.
/// Retorna o número de políticas disparadas.
pub fn tick(store: &Store) -> anyhow::Result<usize> {
    let policies: Vec<PolicyRecord> = store.list_policies()?;
    let mut fired = 0;
    for (id, src, dst, kind, refspec, glob, cron_expr) in policies {
        let Some(expr) = cron_expr else { continue };
        // last_run é lido de novo por política para ficar correto sob concorrência leve
        let last = policy_last_run(store, id)?;
        if !is_due(&expr, last) {
            continue;
        }
        let (job_kind, repo_id, payload) = match kind.as_str() {
            "fetch_upstream" => (
                "fetch",
                src.unwrap_or(0),
                serde_json::json!({ "remote": refspec.clone().unwrap_or_else(|| "origin".into()) }),
            ),
            "branch_sync" => (
                "branch_sync",
                src.unwrap_or(0),
                serde_json::json!({ "refspec": refspec.clone().unwrap_or_else(|| "origin/main".into()) }),
            ),
            "push_mirror" => (
                "push_mirror",
                src.unwrap_or(0),
                serde_json::json!({ "dst": refspec.clone().unwrap_or_else(|| "mirror".into()) }),
            ),
            "path_sync" => (
                "path_sync",
                src.unwrap_or(0),
                serde_json::json!({
                    "dst": dst,
                    "glob": glob.clone().unwrap_or_else(|| "**".into()),
                    "src_policy": id,
                }),
            ),
            _ => continue,
        };
        store.enqueue(&job_kind, repo_id, &payload, 5)?;
        store.mark_policy_run(id)?;
        fired += 1;
    }
    Ok(fired)
}

fn policy_last_run(store: &Store, id: i64) -> anyhow::Result<Option<i64>> {
    let c = store.conn.lock().unwrap();
    Ok(c.query_row(
        "SELECT last_run FROM sync_policies WHERE id=?1",
        [id],
        |r| r.get(0),
    )
    .ok()
    .flatten())
}

/// Laço de cron para o daemon: dispara a cada `interval` e é resiliente a
/// falhas (loga no evento e segue).
pub fn run_daemon(store: Arc<Store>, interval: std::time::Duration) {
    std::thread::spawn(move || loop {
        if let Err(e) = tick(&store) {
            let _ = store.event(0, "cron.error", &serde_json::json!({ "error": e.to_string() }));
        }
        std::thread::sleep(interval);
    });
}
