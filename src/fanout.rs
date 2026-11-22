//! F31 — Fan-out de dependências: divergência de versões entre repositórios.
//!
//! Cruza o SBOM de todos os repos e aponta pacotes usados em mais de um
//! repositório com versões divergentes — candidatos a drift de dependência.

use crate::store::Store;
use std::collections::BTreeMap;

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct DriftEntry {
    pub ecosystem: String,
    pub name: String,
    /// (repo_id, versão) — um por repo distinto
    pub versions: Vec<(i64, String)>,
}

/// Relatório de pacotes com versões divergentes entre repos.
pub fn drift_report(store: &Store) -> anyhow::Result<Vec<DriftEntry>> {
    // (eco, name, version) -> {repo}
    let mut seen: BTreeMap<(String, String), Vec<(i64, String)>> = BTreeMap::new();
    for (repo_id, _commit, eco, name, version) in store.list_sbom()? {
        let key = (eco, name);
        let slot = seen.entry(key).or_default();
        if !slot.iter().any(|(r, _)| *r == repo_id) {
            slot.push((repo_id, version));
        }
    }
    let mut out = Vec::new();
    for ((eco, name), mut versions) in seen {
        versions.sort();
        let distinct: std::collections::BTreeSet<&str> =
            versions.iter().map(|(_, v)| v.as_str()).collect();
        if distinct.len() > 1 {
            out.push(DriftEntry {
                ecosystem: eco,
                name,
                versions,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn drift_detecta_divergencia() {
        let store = Store::open_in_memory().unwrap();
        store.add_repo("/a", None, "mirror").unwrap();
        store.add_repo("/b", None, "mirror").unwrap();
        store.insert_sbom_entry(1, "c1", "npm", "leftpad", "1.0.0").unwrap();
        store.insert_sbom_entry(2, "c2", "npm", "leftpad", "1.1.0").unwrap();
        store.insert_sbom_entry(1, "c1", "npm", "same", "2.0.0").unwrap();
        store.insert_sbom_entry(2, "c2", "npm", "same", "2.0.0").unwrap();

        let report = drift_report(&store).unwrap();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].name, "leftpad");
        assert_eq!(report[0].versions.len(), 2);
    }
}
