//! F02 — Cache de worktrees com despejo LRU.
//!
//! Critério de aceite MVP: worktrees ≤ 30 em T2. O cache vive em
//! `~/.colmeia/worktrees/<repo_id>/<rótulo>` e a política de despejo usa a
//! coluna `last_used` (tabela `worktrees`). Cada aquisição atualiza o
//! timestamp; ao estourar o teto, os menos recentes são removidos com
//! `git worktree remove --force`.

use crate::git::Git;
use crate::store::Store;
use std::path::PathBuf;

fn cache_root() -> PathBuf {
    if let Ok(custom) = std::env::var("COLMEIA_WORKTREE_ROOT") {
        return PathBuf::from(custom);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".colmeia/worktrees")
}

/// Remove worktrees excedentes (LRU) até caber em `max`.
pub fn prune(store: &Store, git: &Git, max: usize) -> anyhow::Result<usize> {
    let count = store.worktree_count()?;
    let excess = count - (max as i64).min(count);
    if excess <= 0 {
        return Ok(0);
    }
    let victims = store.worktree_lru_victims(excess)?;
    let mut removed = 0;
    for (path, repo_id) in victims {
        // `git worktree remove` precisa rodar dentro do repositório de origem
        if let Ok(repo_path) = store.get_repo_path(repo_id) {
            let _ = git.run(
                std::path::Path::new(&repo_path),
                &["worktree", "remove", "--force", &path],
            );
        }
        if PathBuf::from(&path).exists() {
            std::fs::remove_dir_all(&path).ok();
        }
        store.worktree_remove(&path)?;
        removed += 1;
    }
    Ok(removed)
}

/// Adquire um worktree para (repo, rótulo, propósito), criando se necessário.
/// Reutiliza entrada existente e atualiza o LRU. Devolve o caminho.
pub fn acquire(
    store: &Store,
    git: &Git,
    repo_id: i64,
    label: &str,
    target: &str,
    purpose: &str,
    max: usize,
) -> anyhow::Result<PathBuf> {
    if let Some(existing) = store.worktree_get(repo_id, purpose)? {
        let path = PathBuf::from(&existing);
        if path.is_dir() {
            store.worktree_touch(&existing)?;
            return Ok(path);
        }
        store.worktree_remove(&existing)?;
    }

    prune(store, git, max.saturating_sub(1) as usize)?;

    let repo_path = store.get_repo_path(repo_id)?;
    let safe_label: String = label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    // Hash do caminho do repo: ids de bancos distintos não colidem no cache
    let repo_tag = &crate::gate::blob_sha(&repo_path)[..8];
    let dir = cache_root().join(format!(
        "{}-{}/{}-{}",
        repo_id,
        repo_tag,
        purpose,
        &safe_label.chars().take(48).collect::<String>()
    ));
    if dir.exists() {
        // Entrada órfã (sem registro no banco): limpa antes de recriar
        let _ = git.run(
            std::path::Path::new(&repo_path),
            &["worktree", "remove", "--force", &dir.to_string_lossy()],
        );
        std::fs::remove_dir_all(&dir).ok();
    }
    std::fs::create_dir_all(&dir)?;
    let dir_str = dir.to_string_lossy().to_string();

    // --detach evita o erro "branch already checked out" quando o branch já
    // está em uso no checkout principal.
    let out = git.run(
        std::path::Path::new(&repo_path),
        &["worktree", "add", "--detach", &dir_str, target],
    )?;
    anyhow::ensure!(
        out.status.success(),
        "git worktree add falhou: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    store.worktree_insert(repo_id, label, &dir_str, purpose)?;
    Ok(dir)
}

/// Libera um worktree sem removê-lo do disco (apenas atualiza o LRU).
pub fn release(store: &Store, path: &std::path::Path) -> anyhow::Result<()> {
    store.worktree_touch(&path.to_string_lossy())
}
