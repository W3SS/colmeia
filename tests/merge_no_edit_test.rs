//! Teste de integração A1: `git merge -X` nunca deve abrir editor interativo
//! (headless-safe). Reproduz o cenário de divergência entre branches e exige
//! que o driver `theirs` complete sozinho, sem TTY e sem GIT_EDITOR configurado.

use colmeia::git::{Git, GitBackend};
use colmeia::merge;
use colmeia::store::Store;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("falha ao executar git");
    assert!(
        out.status.success(),
        "git {:?} falhou: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(repo: &Path, name: &str, content: &str) {
    std::fs::write(repo.join(name), content).unwrap();
}

#[test]
fn merge_driver_theirs_roda_headless_sem_editor() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir(&repo).unwrap();

    // Garante ambiente hostil a editores: nenhum TTY, nenhum GIT_EDITOR.
    // (O processo de teste já é sem TTY; aqui reforçamos removendo a var.)
    std::env::remove_var("GIT_EDITOR");
    std::env::remove_var("EDITOR");
    std::env::remove_var("VISUAL");

    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "user.email", "teste@colmeia.local"]);
    git(&repo, &["config", "user.name", "Colmeia Teste"]);

    // Commit base
    write(&repo, "app.txt", "base\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);

    // Branch `feature` diverge com conteúdo próprio
    git(&repo, &["checkout", "-b", "feature"]);
    write(&repo, "app.txt", "versao feature\n");
    write(&repo, "novo.txt", "arquivo da feature\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "feature"]);

    // main também diverge (conflito real em app.txt)
    git(&repo, &["checkout", "main"]);
    write(&repo, "app.txt", "versao main\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "main avanca"]);

    // Store + registro do repo
    let store = Store::open_in_memory().unwrap();
    let repo_id = store.add_repo(repo.to_str().unwrap(), None, "managed").unwrap();

    let git_backend = Git::new(256);
    let payload = serde_json::json!({ "ref": "feature", "policy": "theirs" });

    // O merge com driver theirs deve resolver o conflito sozinho e concluir
    // sem jamais invocar um editor. Com o bug A1 (sem --no-edit), este teste
    // falharia/ travaria dependendo do ambiente.
    merge::run(&store, &git_backend, repo_id, &payload).expect("merge deve concluir headless");

    // Driver theirs: conteúdo da feature vence o conflito
    let conteudo = std::fs::read_to_string(repo.join("app.txt")).unwrap();
    assert!(
        conteudo.contains("versao feature"),
        "driver theirs deve preferir o lado de 'ref', obteve: {conteudo:?}"
    );
    // Arquivo novo da feature presente
    assert!(repo.join("novo.txt").exists());

    // HEAD avançou para o commit de merge
    let head = git_backend.head(&repo).unwrap();
    let main_ref = Command::new("git")
        .current_dir(&repo)
        .args(["rev-parse", "refs/heads/main"])
        .output()
        .unwrap();
    assert_eq!(
        head.trim(),
        String::from_utf8_lossy(&main_ref.stdout).trim(),
        "main deve apontar para o merge commit"
    );
}

#[test]
fn merge_driver_ours_preferencia_local() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir(&repo).unwrap();

    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "user.email", "teste@colmeia.local"]);
    git(&repo, &["config", "user.name", "Colmeia Teste"]);

    write(&repo, "app.txt", "base\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);

    git(&repo, &["checkout", "-b", "feature"]);
    write(&repo, "app.txt", "versao feature\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "feature"]);

    git(&repo, &["checkout", "main"]);
    write(&repo, "app.txt", "versao main\n");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "main avanca"]);

    let store = Store::open_in_memory().unwrap();
    let repo_id = store.add_repo(repo.to_str().unwrap(), None, "managed").unwrap();
    let git_backend = Git::new(256);

    merge::run(
        &store,
        &git_backend,
        repo_id,
        &serde_json::json!({ "ref": "feature", "policy": "ours" }),
    )
    .expect("merge ours deve concluir headless");

    let conteudo = std::fs::read_to_string(repo.join("app.txt")).unwrap();
    assert!(
        conteudo.contains("versao main"),
        "driver ours deve preferir o lado local, obteve: {conteudo:?}"
    );
}
