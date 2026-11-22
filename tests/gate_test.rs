//! Teste do gate SAST (src/gate.rs): repo temporário com segredo plantado
//! no último commit → `run_repo` deve registrar um finding `secret.token`.
//! Tudo em tempdir; nenhum repo real é tocado.

use colmeia::gate;
use colmeia::git::{Git, GitBackend};
use colmeia::store::Store;
use std::process::Command;

fn git(repo: &std::path::Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("git deve estar instalado");
    assert!(out.status.success(), "git {:?} falhou: {}", args, String::from_utf8_lossy(&out.stderr));
}

#[test]
fn gate_flags_planted_secret() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("alvo");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-b", "main"]);
    git(&repo, &["config", "user.name", "Colmeia Test"]);
    git(&repo, &["config", "user.email", "t@colmeia.local"]);

    std::fs::write(repo.join("ok.txt"), "limpo\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);

    // Commit suspeito: token com formato ghp_ plantado
    std::fs::write(repo.join("config.py"), "TOKEN = \"ghp_ABCDEFGHIJKLMNOPQRSTUVWX\"\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "chave acidental"]);

    let store = Store::open_in_memory().unwrap();
    let id = store.add_repo(&repo.to_string_lossy(), None, "managed").unwrap();
    let cfg = colmeia::config::Config::default();
    let git_backend = Git::default();

    gate::run_repo(
        &store,
        &git_backend,
        &cfg,
        id,
        &serde_json::json!({"range": "HEAD~1..HEAD"}),
    )
    .expect("gate deve rodar");

    let findings = store.list_findings(id).unwrap();
    assert!(
        findings.iter().any(|(rule, ..)| rule == "secret.token"),
        "secret.token não detectado; findings={findings:?}"
    );

    // SARIF sai com o finding mapeado
    let sarif = gate::sarif(&findings);
    let results = sarif["runs"][0]["results"].as_array().unwrap();
    assert!(results.iter().any(|r| r["ruleId"] == "secret.token"));
}
