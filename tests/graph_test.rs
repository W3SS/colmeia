//! F28 — Teste de integração do grafo por co-mudança.

use colmeia::git::{Git, GitBackend};
use colmeia::graph;
use colmeia::store::Store;
use std::path::Path;

fn repo_tmp(store: &Store, name: &str, files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let git = Git::new(64);
    git.run(Path::new(dir.path()), &["init", "-q", "-b", "main"])
        .unwrap();
    git.run(Path::new(dir.path()), &["config", "user.email", "t@t"])
        .unwrap();
    git.run(Path::new(dir.path()), &["config", "user.name", "t"])
        .unwrap();
    for (p, content) in files {
        let full = dir.path().join(p);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&full, content).unwrap();
    }
    git.run(Path::new(dir.path()), &["add", "."]).unwrap();
    git.run(Path::new(dir.path()), &["commit", "-q", "-m", "init"]).unwrap();
    store
        .add_repo(dir.path().to_str().unwrap(), None, "mirror")
        .map(|_| ())
        .ok();
    let _ = name;
    dir
}

#[test]
fn grafo_cochange_detecta_arquivo_em_comum() {
    let store = Store::open_in_memory().unwrap();
    let _a = repo_tmp(
        &store,
        "a",
        &[("src/shared.rs", "pub fn f() {}"), ("src/only_a.rs", "a")],
    );
    let _b = repo_tmp(
        &store,
        "b",
        &[("src/shared.rs", "pub fn f() { mod }"), ("src/only_b.rs", "b")],
    );
    let _c = repo_tmp(&store, "c", &[("README.md", "sem nada em comum")]);

    let git = Git::new(64);
    let upserted = graph::recompute(&store, &git, 30, 0.05).unwrap();
    assert!(upserted >= 1, "esperava ao menos uma aresta");

    let edges = store.graph_edges().unwrap();
    let ab = edges
        .iter()
        .find(|(s, d, _, k)| *k == "cochange" && (*s, *d) == (1, 2) || (*s, *d) == (2, 1));
    assert!(ab.is_some(), "aresta entre os repos com arquivo em comum");
    // /c não tocou arquivo algum em comum
    let c_involved = edges.iter().any(|(s, d, _, _)| *s == 3 || *d == 3);
    assert!(!c_involved);

    // view e layout exportável
    let view = graph::view(&store).unwrap();
    assert_eq!(view.nodes.len(), 3);
    let layout = colmeia::layout::export_layout(&store, 0.05, true).unwrap();
    assert_eq!(layout["nodes"].as_array().unwrap().len(), 3);
}
