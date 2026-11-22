//! F28 — Grafo de repositórios por co-mudança de arquivos.
//!
//! Arestas `cochange`: dois repos que tocaram o mesmo caminho de arquivo na
//! janela observada têm peso = Jaccard(|A∩B| / |A∪B|) sobre os conjuntos de
//! arquivos commitados. Arestas abaixo do limiar são removidas.

use crate::git::Git;
use crate::store::Store;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const DEFAULT_WINDOW_DAYS: u32 = 30;
pub const MIN_WEIGHT: f64 = 0.05;

/// Recomputa as arestas `cochange` do grafo a partir do histórico recente.
pub fn recompute(
    store: &Store,
    git: &Git,
    window_days: u32,
    min_weight: f64,
) -> anyhow::Result<usize> {
    let repos = store.list_repos()?;
    // repo_id -> conjunto de arquivos tocados na janela
    let mut files_by_repo: BTreeMap<i64, BTreeSet<String>> = BTreeMap::new();
    for (id, path, _, _) in &repos {
        let files = touched_files(git, Path::new(path), window_days)?;
        files_by_repo.insert(*id, files);
    }

    let mut upserted = 0;
    let ids: Vec<i64> = files_by_repo.keys().copied().collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            let a = &files_by_repo[&ids[i]];
            let b = &files_by_repo[&ids[j]];
            if a.is_empty() || b.is_empty() {
                continue;
            }
            let inter = a.intersection(b).count() as f64;
            if inter == 0.0 {
                continue;
            }
            let union = a.union(b).count() as f64;
            let weight = inter / union;
            if weight >= min_weight {
                store.graph_upsert_edge(ids[i], ids[j], weight, "cochange")?;
                upserted += 1;
            }
        }
    }
    Ok(upserted)
}

/// Arquivos commitados na janela, via `git log --name-only`.
fn touched_files(git: &Git, repo: &Path, window_days: u32) -> anyhow::Result<BTreeSet<String>> {
    let since = format!("--since={window_days} days ago");
    let out = git.run(
        repo,
        &["log", &since, "--name-only", "--format=%H", "--no-merges"],
    )?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut files = BTreeSet::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || (line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit())) {
            continue; // separador de commit (SHA) ou linha vazia
        }
        files.insert(line.to_string());
    }
    Ok(files)
}

/// Estrutura para exportar/layout (F29) e TUI (F46).
#[derive(Debug, serde::Serialize)]
pub struct GraphView {
    pub nodes: Vec<(i64, String)>,
    pub edges: Vec<(i64, i64, f64, String)>,
}

pub fn view(store: &Store) -> anyhow::Result<GraphView> {
    let nodes = store
        .list_repos()?
        .into_iter()
        .map(|(id, path, _, _)| (id, path))
        .collect();
    Ok(GraphView {
        nodes,
        edges: store.graph_edges()?,
    })
}
