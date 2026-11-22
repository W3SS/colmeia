//! F10 — path_sync: copia por glob com hash de conteúdo e commit
//! determinístico; segunda execução é no-op.

use colmeia::git::{Git, GitBackend};
use colmeia::path_sync::{glob_match, path_sync};
use colmeia::store::Store;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git").current_dir(repo).args(args).output().unwrap();
    assert!(out.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&out.stderr));
}

fn repo(tmp: &Path, nome: &str) -> std::path::PathBuf {
    let r = tmp.join(nome);
    std::fs::create_dir_all(&r).unwrap();
    git(&r, &["init", "-b", "main"]);
    git(&r, &["config", "user.email", "t@t.local"]);
    git(&r, &["config", "user.name", "T"]);
    r
}

#[test]
fn glob_basico() {
    assert!(glob_match("skills/**", "skills/a/SKILL.md"));
    assert!(glob_match("skills/*/SKILL.md", "skills/x/SKILL.md"));
    assert!(!glob_match("skills/*/SKILL.md", "skills/x/y/SKILL.md"));
    assert!(glob_match("*.txt", "notas.txt"));
    assert!(!glob_match("*.txt", "dir/notas.txt"));
    assert!(glob_match("**/*.md", "a/b/c.md"));
}

#[test]
fn path_sync_copia_e_e_idempotente() {
    let tmp = tempfile::tempdir().unwrap();
    let src = repo(tmp.path(), "src");
    let dst = repo(tmp.path(), "dst");

    std::fs::create_dir_all(src.join("skills/pack")).unwrap();
    std::fs::write(src.join("skills/pack/SKILL.md"), "# pack\n").unwrap();
    std::fs::write(src.join("skills/pack/outro.txt"), "fora do glob\n").unwrap();
    std::fs::write(src.join("README.md"), "# readme\n").unwrap();

    let store = Store::open_in_memory().unwrap();
    let src_id = store.add_repo(src.to_str().unwrap(), None, "managed").unwrap();
    let dst_id = store.add_repo(dst.to_str().unwrap(), None, "managed").unwrap();
    let gitb = Git::new(256);

    path_sync(
        &store,
        &gitb,
        src_id,
        &serde_json::json!({ "dst": dst_id, "glob": "skills/**/*.md" }),
    )
    .unwrap();

    assert!(dst.join("skills/pack/SKILL.md").exists());
    assert!(!dst.join("skills/pack/outro.txt").exists(), "txt não casa o glob md");
    assert!(!dst.join("README.md").exists());

    // Commit criado no destino
    let head = gitb.head(&dst).unwrap();
    let msg = Command::new("git")
        .current_dir(&dst)
        .args(["log", "-1", "--format=%s"])
        .output()
        .unwrap();
    let msg = String::from_utf8_lossy(&msg.stdout);
    assert!(msg.contains("path_sync"), "commit deve ser do path_sync: {msg}");
    assert_ne!(head, "0000000000000000000000000000000000000000");

    // Idempotente: segunda execução não copia nada nem cria commit novo
    let antes = gitb.head(&dst).unwrap();
    path_sync(
        &store,
        &gitb,
        src_id,
        &serde_json::json!({ "dst": dst_id, "glob": "skills/**/*.md" }),
    )
    .unwrap();
    let depois = gitb.head(&dst).unwrap();
    assert_eq!(antes, depois, "sem mudanças de conteúdo → sem commit novo");

    // Mudança na origem → nova sincronização copia e commita
    std::fs::write(src.join("skills/pack/SKILL.md"), "# pack v2\n").unwrap();
    path_sync(
        &store,
        &gitb,
        src_id,
        &serde_json::json!({ "dst": dst_id, "glob": "skills/**/*.md" }),
    )
    .unwrap();
    let conteudo = std::fs::read_to_string(dst.join("skills/pack/SKILL.md")).unwrap();
    assert!(conteudo.contains("v2"));
}
