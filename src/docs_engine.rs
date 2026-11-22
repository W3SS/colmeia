//! F26/F27 — Motor de doc cards com reindexação incremental e FTS5.
//!
//! `index_docs` recebe os documentos atuais de um repo e aplica só o diff:
//! conteúdo cujo `content_hash` mudou é regravado (e reindexado no FTS5);
//! documentos ausentes são removidos. Custo proporcional ao diff.

use crate::store::Store;
use sha2::{Digest, Sha256};

fn content_hash(content: &str) -> String {
    hex::encode(Sha256::digest(content.as_bytes()))
}

/// Documento extraído de um repositório (README, ADR, etc.).
#[derive(Debug)]
pub struct DocInput {
    pub path: String,
    pub title: String,
    pub content: String,
}

/// Aplica o índice de docs de um repositório de forma incremental.
/// Devolve (inseridos_atualizados, removidos).
pub fn index_docs(
    store: &Store,
    repo_id: i64,
    docs: &[DocInput],
) -> anyhow::Result<(usize, usize)> {
    let existing: std::collections::HashMap<String, String> =
        store.docs_list(repo_id)?.into_iter().collect();
    let mut current: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut upserts = 0;

    for d in docs {
        current.insert(d.path.clone());
        let hash = content_hash(&d.content);
        if existing.get(&d.path) == Some(&hash) {
            continue; // sem mudança — O(diff)
        }
        store.doc_upsert(repo_id, &d.path, &d.title, &d.content, &hash)?;
        upserts += 1;
    }

    let mut removed = 0;
    for path in existing.keys() {
        if !current.contains(path) {
            store.doc_delete(repo_id, path)?;
            removed += 1;
        }
    }
    Ok((upserts, removed))
}

/// Busca full-text com snippet.
pub fn search(store: &Store, query: &str, limit: usize) -> anyhow::Result<Vec<SearchHit>> {
    let hits = store.doc_search(query, limit)?;
    Ok(hits
        .into_iter()
        .map(|(repo_id, path, title, snippet)| SearchHit {
            repo_id,
            path,
            title,
            snippet,
        })
        .collect())
}

#[derive(Debug, serde::Serialize)]
pub struct SearchHit {
    pub repo_id: i64,
    pub path: String,
    pub title: String,
    pub snippet: String,
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn indexacao_e_busca_fts() {
        let store = Store::open_in_memory().unwrap();
        let docs = vec![
            DocInput {
                path: "README.md".into(),
                title: "Visão geral".into(),
                content: "A colmeia gerencia frotas de repositórios git".into(),
            },
            DocInput {
                path: "docs/adr-1.md".into(),
                title: "ADR-1".into(),
                content: "Decisão de arquitetura sobre o banco embedded".into(),
            },
        ];
        let (up, rm) = index_docs(&store, 1, &docs).unwrap();
        assert_eq!((up, rm), (2, 0));

        // reindex sem mudança = zero escritas
        let (up2, rm2) = index_docs(&store, 1, &docs).unwrap();
        assert_eq!((up2, rm2), (0, 0));

        // remoção
        let (up3, rm3) = index_docs(&store, 1, &docs[..1]).unwrap();
        assert_eq!((up3, rm3), (0, 1));

        let hits = search(&store, "frotas", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "README.md");
        assert!(hits[0].snippet.contains("frotas"));
    }
}
