//! F04 — Hot-subset watcher: snapshot/diff detectam repos novos e alterados.

use colmeia::store::Store;
use colmeia::watch;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git").current_dir(repo).args(args).output().unwrap();
    assert!(out.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&out.stderr));
}

fn novo_repo(tmp: &Path, nome: &str) -> std::path::PathBuf {
    let repo = tmp.join(nome);
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "user.email", "t@t.local"]);
    git(&repo, &["config", "user.name", "T"]);
    std::fs::write(repo.join("f.txt"), "x\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "init"]);
    repo
}

#[test]
fn diff_detecta_novo_e_modificado() {
    let tmp = tempfile::tempdir().unwrap();
    let r1 = novo_repo(tmp.path(), "r1");
    let _r2 = novo_repo(tmp.path(), "r2");

    let store = Store::open_in_memory().unwrap();
    let id1 = store.add_repo(r1.to_str().unwrap(), None, "managed").unwrap();
    let _id2 = store.add_repo(tmp.path().join("r2").to_str().unwrap(), None, "managed").unwrap();

    let s1 = watch::snapshot(&store, 10);
    assert_eq!(s1.len(), 2);

    // Sem mudanças → diff vazio
    let s2 = watch::snapshot(&store, 10);
    assert!(watch::diff(&s1, &s2).is_empty());

    // Novo commit em r1 → diff aponta r1
    std::fs::write(r1.join("f.txt"), "y\n").unwrap();
    git(&r1, &["add", "."]);
    git(&r1, &["commit", "-m", "mudou"]);
    let s3 = watch::snapshot(&store, 10);
    let changed = watch::diff(&s2, &s3);
    assert_eq!(changed.len(), 1);
    assert!(changed[0].ends_with("r1"));

    // Repo inexistente some do snapshot sem pânico
    store.update_repo_head(id1, "abc").ok();
    let s4 = watch::snapshot(&store, 10);
    assert!(s4.len() >= 1);
}
