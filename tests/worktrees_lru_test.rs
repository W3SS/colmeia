//! F02 — LRU de worktrees: teto de cache respeitado, reutilização de
//! worktree existente e despejo do menos recente.

use colmeia::git::{Git, GitBackend};
use colmeia::store::Store;
use colmeia::worktrees;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&out.stderr));
}

fn repo_com_commit(tmp: &Path, nome: &str) -> std::path::PathBuf {
    let repo = tmp.join(nome);
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "user.email", "t@t.local"]);
    git(&repo, &["config", "user.name", "T"]);
    std::fs::write(repo.join("a.txt"), "base\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);
    repo
}

#[test]
fn lru_evita_oldest_quando_estoura_teto() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("wt-cache");
    std::fs::create_dir(&cache).unwrap();
    std::env::set_var("COLMEIA_WORKTREE_ROOT", &cache);

    let repo = repo_com_commit(tmp.path(), "repo");
    let store = Store::open_in_memory().unwrap();
    let repo_id = store.add_repo(repo.to_str().unwrap(), None, "managed").unwrap();
    let gitb = Git::new(256);

    // Teto de 2; adquire 3 worktrees com propósitos distintos
    let w1 = worktrees::acquire(&store, &gitb, repo_id, "c1", "HEAD", "t1", 2).unwrap();
    let w2 = worktrees::acquire(&store, &gitb, repo_id, "c2", "HEAD", "t2", 2).unwrap();
    // usa w1 de novo para renovar seu LRU
    let _ = worktrees::acquire(&store, &gitb, repo_id, "c1", "HEAD", "t1", 2).unwrap();
    let w3 = worktrees::acquire(&store, &gitb, repo_id, "c3", "HEAD", "t3", 2).unwrap();

    // w2 era o menos recentemente usado → foi despejado
    assert!(!w2.exists(), "worktree LRU (t2) deveria ter sido removido");
    assert!(w1.exists(), "worktree renovado (t1) deve sobreviver");
    assert!(w3.exists(), "worktree novo (t3) deve existir");
    assert!(
        store.worktree_count().unwrap() <= 2,
        "teto de worktrees violado"
    );

    std::env::remove_var("COLMEIA_WORKTREE_ROOT");
}
