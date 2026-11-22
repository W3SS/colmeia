//! F29 — Layout do grafo: posições determinísticas + culling por peso.
//!
//! MVP documentado: posições em grade determinística (sem simulação de
//! forças — leveza do contrato), com culling de arestas fracas por limiar
//! de peso e de nós isolados abaixo do grau mínimo. Exporta JSON puro.

use crate::graph;
use crate::store::Store;
use serde_json::json;

/// Exporta o layout do grafo como JSON.
/// - `min_weight`: arestas mais fracas que isso são omitidas (culling).
/// - `include_isolated`: nós sem arestas aparecem mesmo assim.
pub fn export_layout(
    store: &Store,
    min_weight: f64,
    include_isolated: bool,
) -> anyhow::Result<serde_json::Value> {
    let view = graph::view(store)?;
    let edges: Vec<_> = view
        .edges
        .into_iter()
        .filter(|(_, _, w, _)| *w >= min_weight)
        .collect();
    let connected: std::collections::HashSet<i64> = edges
        .iter()
        .flat_map(|(a, b, _, _)| [*a, *b])
        .collect();

    let cols = (view.nodes.len() as f64).sqrt().ceil().max(1.0) as usize;
    let mut nodes = Vec::new();
    for (idx, (id, path)) in view.nodes.iter().enumerate() {
        if !include_isolated && !connected.contains(id) {
            continue;
        }
        nodes.push(json!({
            "id": id,
            "path": path,
            "x": idx % cols,
            "y": idx / cols,
        }));
    }
    Ok(json!({
        "nodes": nodes,
        "edges": edges.iter().map(|(a, b, w, k)| json!({
            "src": a, "dst": b, "weight": w, "kind": k
        })).collect::<Vec<_>>(),
    }))
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn layout_deterministico_e_culling() {
        let store = Store::open_in_memory().unwrap();
        let a = store.add_repo("/a", None, "mirror").unwrap();
        let b = store.add_repo("/b", None, "mirror").unwrap();
        let _c = store.add_repo("/c", None, "mirror").unwrap();
        store.graph_upsert_edge(a, b, 0.9, "cochange").unwrap();
        store.graph_upsert_edge(b, _c, 0.01, "cochange").unwrap();

        let v1 = export_layout(&store, 0.5, false).unwrap();
        let v2 = export_layout(&store, 0.5, false).unwrap();
        assert_eq!(v1, v2, "layout deve ser determinístico");

        // culling: aresta 0.01 some; nó isolado /c some
        let nodes = v1["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(v1["edges"].as_array().unwrap().len(), 1);
    }
}
