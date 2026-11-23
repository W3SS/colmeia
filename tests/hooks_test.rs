//! F25 — Testes de integração dos hooks git locais.

use colmeia::git::Git;
use colmeia::hooks::{self, HookStatus};
use std::path::Path;
use std::time::Duration;

/// Serializa os testes: PATH é global por processo e testes paralelos que
/// isolam o PATH trocam o ambiente um do outro (sleep/sh somem → exit 127).
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap()
}

fn repo_tmp() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let git = Git::new(64);
    git.run(Path::new(dir.path()), &["init", "-q", "-b", "main"])
        .unwrap();
    git.run(Path::new(dir.path()), &["config", "user.email", "t@t"])
        .unwrap();
    git.run(Path::new(dir.path()), &["config", "user.name", "t"])
        .unwrap();
    std::fs::write(dir.path().join("a.txt"), "oi").unwrap();
    git.run(Path::new(dir.path()), &["add", "."]).unwrap();
    git.run(Path::new(dir.path()), &["commit", "-q", "-m", "init"]).unwrap();
    dir
}

#[test]
fn hooks_instala_dois_scripts_executaveis() {
    let _env = env_lock();
    let repo = repo_tmp();
    let installed = hooks::install(repo.path()).unwrap();
    assert_eq!(installed.len(), 2);
    for p in &installed {
        assert!(p.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(p.metadata().unwrap().permissions().mode() & 0o111, 0o111);
        }
    }
}

#[test]
fn hook_executa_sem_colmeia_no_path() {
    let _env = env_lock();
    // sem `colmeia` no PATH o hook deve passar (Ok) e apenas avisar.
    // O PATH isolado contém só `sh` (symlink) para o spawn funcionar.
    let repo = repo_tmp();
    hooks::install(repo.path()).unwrap();

    let result = {
        let bin_dir = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/bin/sh", bin_dir.path().join("sh")).unwrap();
        let original = std::env::var("PATH").unwrap();
        std::env::set_var("PATH", bin_dir.path());
        let r = hooks::run_hook(repo.path(), "pre-push", Duration::from_secs(10));
        std::env::set_var("PATH", original);
        r.unwrap()
    };
    assert_eq!(result.status, HookStatus::Ok);
    assert!(result.stderr.contains("gate pulado"));
}

#[test]
fn hook_missing_quando_script_nao_existe() {
    let _env = env_lock();
    let repo = repo_tmp();
    let r = hooks::run_hook(repo.path(), "pre-push", Duration::from_secs(5)).unwrap();
    assert_eq!(r.status, HookStatus::Missing);
}

#[test]
fn hook_timeout() {
    let _env = env_lock();
    let repo = repo_tmp();
    hooks::install(repo.path()).unwrap();
    // substitui o script por um sleep longo para exercitar o timeout
    let dir = hooks::hooks_dir(repo.path()).unwrap();
    std::fs::write(dir.join("pre-push"), "#!/bin/sh\nsleep 30\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            dir.join("pre-push"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
    }
    let r = hooks::run_hook(repo.path(), "pre-push", Duration::from_millis(300)).unwrap();
    assert_eq!(r.status, HookStatus::TimedOut);
}
