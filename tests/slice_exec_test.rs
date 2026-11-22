//! F12/F16 — slice exec com CAS+backup e replay bit-idêntico.

use colmeia::git::{Git, GitBackend};
use colmeia::slice;
use colmeia::store::Store;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git").current_dir(repo).args(args).output().unwrap();
    assert!(out.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&out.stderr));
}

fn novo_repo(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    let repo = tmp.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "user.email", "t@t.local"]);
    git(&repo, &["config", "user.name", "T"]);
    std::fs::write(repo.join("base.txt"), "base\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);
    repo
}

#[test]
fn slice_exec_reescreve_branch_com_slices_e_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = novo_repo(&tmp);

    // Commit "grande" tocando 3 arquivos
    std::fs::write(repo.join("x.txt"), "x\n").unwrap();
    std::fs::write(repo.join("y.txt"), "y\n").unwrap();
    std::fs::write(repo.join("z.txt"), "z\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "commit grande"]);
    let head_antes = Command::new("git")
        .current_dir(&repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let head_antes = String::from_utf8_lossy(&head_antes.stdout).trim().to_string();

    let store = Store::open_in_memory().unwrap();
    let repo_id = store.add_repo(repo.to_str().unwrap(), None, "managed").unwrap();
    let gitb = Git::new(256);
    let cfg = colmeia::config::Config::default();

    // Plano com max_lines=1 → cada arquivo vira uma fatia (3 slices)
    let plan_id = slice::plan(
        &store,
        &gitb,
        repo_id,
        &serde_json::json!({ "commit": "HEAD", "max_lines": 1 }),
    )
    .unwrap();

    slice::exec(
        &store,
        &gitb,
        &cfg,
        repo_id,
        &serde_json::json!({ "plan_id": plan_id }),
    )
    .unwrap();

    // Branch avançou e tem 3 commits de fatia sobre a base
    let log = Command::new("git")
        .current_dir(&repo)
        .args(["log", "--format=%s", "-n", "4"])
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&log.stdout);
    assert!(log.contains("slice 1/3"), "log: {log}");
    assert!(log.contains("slice 2/3"), "log: {log}");
    assert!(log.contains("slice 3/3"), "log: {log}");

    // Backup ref preserva o head original
    let bak = Command::new("git")
        .current_dir(&repo)
        .args(["rev-parse", "refs/colmeia/backup/main"])
        .output()
        .unwrap();
    assert!(bak.status.success());
    assert_eq!(String::from_utf8_lossy(&bak.stdout).trim(), head_antes);

    // Reconteúdo: soma das fatias equivale ao commit grande
    let cat = std::fs::read_to_string(repo.join("x.txt")).unwrap()
        + &std::fs::read_to_string(repo.join("y.txt")).unwrap()
        + &std::fs::read_to_string(repo.join("z.txt")).unwrap();
    assert!(cat.contains('x') && cat.contains('y') && cat.contains('z'));

    // F16: replay bit-idêntico (dry-run reexecuta e compara output_hash)
    slice::replay(&store, &gitb, &cfg, repo_id, plan_id).unwrap();
}

#[test]
fn slice_exec_detecta_segredo_na_fatia() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = novo_repo(&tmp);

    std::fs::write(repo.join("ok.txt"), "ok\n").unwrap();
    std::fs::write(repo.join("seg.txt"), "token ghp_ABCDEFGHIJKLMNOPQRSTUVWX\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "com segredo"]);

    let store = Store::open_in_memory().unwrap();
    let repo_id = store.add_repo(repo.to_str().unwrap(), None, "managed").unwrap();
    let gitb = Git::new(256);
    let cfg = colmeia::config::Config::default();

    let plan_id = slice::plan(
        &store,
        &gitb,
        repo_id,
        &serde_json::json!({ "commit": "HEAD", "max_lines": 1 }),
    )
    .unwrap();
    slice::exec(
        &store,
        &gitb,
        &cfg,
        repo_id,
        &serde_json::json!({ "plan_id": plan_id }),
    )
    .unwrap();

    let findings = store.list_findings(repo_id).unwrap();
    assert!(
        findings.iter().any(|f| f.0 == "secret.token"),
        "gate por fatia deve registrar o segredo: {findings:?}"
    );
}
