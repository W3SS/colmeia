//! F25 — Hooks git locais com gate de segurança.
//!
//! Instala `pre-commit`/`pre-push` que invocam `colmeia gate`. Se o binário
//! não estiver no PATH o hook deixa passar com aviso (nunca bloqueia por
//! ausência da ferramenta). O runner executa o script com timeout e devolve
//! o status de saída.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum HookStatus {
    Ok,
    Failed(i32),
    TimedOut,
    Missing,
}

#[derive(Debug)]
pub struct HookResult {
    pub status: HookStatus,
    pub stdout: String,
    pub stderr: String,
}

const HOOK_BODY: &str = r#"#!/bin/sh
# gerado pelo colmeia (F25) — gate de segurança no histórico
if command -v colmeia >/dev/null 2>&1; then
  if ! colmeia gate --range HEAD~1..HEAD --format json >/dev/null 2>&1; then
    echo "colmeia: gate bloqueou (findings de segurança no último commit)" >&2
    exit 1
  fi
else
  echo "colmeia: binário não encontrado no PATH; gate pulado" >&2
fi
exit 0
"#;

/// Caminho do diretório de hooks de um repositório. Resolve em Rust puro
/// para repositórios comuns (`<repo>/.git/hooks`) e recorre a
/// `git rev-parse --git-path` para worktrees/gitfiles.
pub fn hooks_dir(repo_path: &Path) -> anyhow::Result<PathBuf> {
    let dot_git = repo_path.join(".git");
    if dot_git.is_dir() {
        return Ok(dot_git.join("hooks"));
    }
    let out = Command::new("git")
        .args(["rev-parse", "--git-path", "hooks"])
        .current_dir(repo_path)
        .output()?;
    if !out.status.success() {
        anyhow::bail!("não foi possível resolver o diretório de hooks");
    }
    let rel = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(if Path::new(&rel).is_absolute() {
        PathBuf::from(rel)
    } else {
        repo_path.join(rel)
    })
}

/// Instala os hooks `pre-commit` e `pre-push`. Sobrescreve instalações
/// anteriores do colmeia (o script é idempotente por natureza).
pub fn install(repo_path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let dir = hooks_dir(repo_path)?;
    std::fs::create_dir_all(&dir)?;
    let mut installed = Vec::new();
    for name in ["pre-commit", "pre-push"] {
        let p = dir.join(name);
        let mut f = std::fs::File::create(&p)?;
        f.write_all(HOOK_BODY.as_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755))?;
        }
        installed.push(p);
    }
    Ok(installed)
}

/// Executa o hook de um evento com timeout. Devolve Missing se o script
/// não existir.
pub fn run_hook(repo_path: &Path, event: &str, timeout: Duration) -> anyhow::Result<HookResult> {
    let script = hooks_dir(repo_path)?.join(event);
    if !script.exists() {
        return Ok(HookResult {
            status: HookStatus::Missing,
            stdout: String::new(),
            stderr: String::new(),
        });
    }
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&script)
        .current_dir(repo_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            let out = child.wait_with_output()?;
            let code = status.code().unwrap_or(-1);
            return Ok(HookResult {
                status: if code == 0 {
                    HookStatus::Ok
                } else {
                    HookStatus::Failed(code)
                },
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(HookResult {
                status: HookStatus::TimedOut,
                stdout: String::new(),
                stderr: String::new(),
            });
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
