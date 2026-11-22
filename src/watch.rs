//! F04 — Hot-subset watcher.
//!
//! Sem crates de FS events para manter o binário leve (contrato de
//! leveza), o hot-subset usa polling com estado de mtime sobre os
//! diretórios `.git` dos repositórios ativos. O conjunto observado é
//! limitado por `hot_watch_count` (prioridade: repos `managed` mais
//! ativos). Mudanças disparam enqueue de `fetch`.

use crate::store::Store;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub type Snapshot = HashMap<PathBuf, u64>;

/// Fingerprint barata de um repo: mtime(ns) do HEAD, tamanho do packed-refs
/// e tamanho do reflog. O reflog cresce a cada commit, então a detecção
/// funciona mesmo em filesystems com granularidade de 1 s.
fn fingerprint(repo: &Path) -> Option<u64> {
    let git_dir = if repo.join(".git").is_dir() {
        repo.join(".git")
    } else {
        repo.to_path_buf()
    };
    let head_meta = std::fs::metadata(git_dir.join("HEAD")).ok()?;
    let mtime = head_meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let packed_len = std::fs::metadata(git_dir.join("packed-refs"))
        .map(|m| m.len())
        .unwrap_or(0);
    let reflog_len = std::fs::metadata(git_dir.join("logs/HEAD"))
        .map(|m| m.len())
        .unwrap_or(0);
    Some(mtime ^ packed_len.rotate_left(17) ^ reflog_len.rotate_left(33))
}

/// Monta o snapshot do hot-subset (primeiros `hot_count` repos ativos).
pub fn snapshot(store: &Store, hot_count: usize) -> Snapshot {
    let mut snap = Snapshot::new();
    let repos = store.list_repo_views().unwrap_or_default();
    for view in repos.iter().take(hot_count) {
        let path = PathBuf::from(&view.path);
        if let Some(fp) = fingerprint(&path) {
            snap.insert(path, fp);
        }
    }
    snap
}

/// Diferença entre snapshots: repos novos ou modificados.
pub fn diff(prev: &Snapshot, next: &Snapshot) -> Vec<PathBuf> {
    next.iter()
        .filter(|(path, fp)| prev.get(*path) != Some(*fp))
        .map(|(path, _)| path.clone())
        .collect()
}

/// Laço do watcher para o daemon. A cada intervalo compara snapshots e
/// enfileira `fetch` nos repos alterados.
pub fn run_daemon(store: Arc<Store>, hot_count: usize, interval: Duration) {
    std::thread::spawn(move || {
        let mut prev = snapshot(&store, hot_count);
        loop {
            std::thread::sleep(interval);
            let next = snapshot(&store, hot_count);
            for path in diff(&prev, &next) {
                // resolve id pelo caminho
                if let Some((id, _, _, _)) = store
                    .list_repos()
                    .unwrap_or_default()
                    .into_iter()
                    .find(|(_, p, _, _)| p == &path.to_string_lossy())
                {
                    let _ = store.enqueue(
                        "fetch",
                        id,
                        &serde_json::json!({ "remote": "origin", "trigger": "watch" }),
                        4,
                    );
                    let _ = store.event(
                        id,
                        "watch.hot",
                        &serde_json::json!({ "path": path.to_string_lossy() }),
                    );
                }
            }
            prev = next;
        }
    });
}
