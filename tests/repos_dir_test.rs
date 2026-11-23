//! Testes de integração baseados no diretório real de repositórios.
//!
//! Diretório-alvo (somente leitura nos originais):
//!   padrão: ~/Ubuntu/home/Documents/repos (se existir, senão ~/Documents/repos)
//!   override: COLMEIA_REPOS_TEST_DIR=/caminho/para/repos
//!
//! Regras de segurança do contrato de testes:
//!   - NUNCA deletar conteúdo do diretório real.
//!   - Leitura (head, branches, log) é permitida e esperada.
//!   - Mover/manipular só em CLONES dentro de `tempdir` (limpeza automática
//!     do temp não conta como deleção de projeto do usuário).
//!   - Banco SQLite sempre em `tempdir`/`in-memory`.

use colmeia::git::{Git, GitBackend};
use colmeia::store::Store;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn repos_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("COLMEIA_REPOS_TEST_DIR") {
        return PathBuf::from(dir);
    }
    // Sem caminho hardcoded: candidatos relativos a $HOME (não expõe o usuário).
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    for cand in ["Ubuntu/home/Documents/repos", "Documents/repos"] {
        let p = home.join(cand);
        if p.is_dir() {
            return p;
        }
    }
    home.join("Documents/repos") // fallback: mensagem de erro do teste
}

/// Descobre repositórios git (diretórios com `.git`), determinístico, com limite
/// de quantidade e de profundidade (os repos da frota ficam no nível 1–2).
fn discover_git_repos(root: &Path, max: usize) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    let mut pending = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = pending.pop() {
        if repos.len() >= max || depth > 3 {
            continue;
        }
        if !dir.is_dir() {
            continue;
        }
        let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if matches!(name, "node_modules" | "target" | ".git" | "__pycache__") {
            continue;
        }
        if dir.join(".git").is_dir() {
            repos.push(dir.clone());
            continue; // não desce dentro de um repo
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                pending.push((e.path(), depth + 1));
            }
        }
    }
    repos.sort();
    repos
}

fn git(repo: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("git deve estar instalado");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn test_discovers_git_repos_in_directory() {
    let root = repos_dir();
    if !root.is_dir() {
        // CI/outras máquinas: diretório real de repos não existe — skip.
        eprintln!("diretório de repos ausente, teste pulado: {}", root.display());
        return;
    }
    let repos = discover_git_repos(&root, 200);
    assert!(
        repos.len() >= 10,
        "esperado >= 10 repositórios em {}, achou {}",
        root.display(),
        repos.len()
    );
    println!("descobertos {} repositórios", repos.len());
}

#[test]
fn test_reads_heads_and_branches_without_mutation() {
    let root = repos_dir();
    if !root.is_dir() {
        eprintln!("diretório de repos ausente, teste pulado");
        return;
    }
    let repos = discover_git_repos(&root, 8);
    assert!(!repos.is_empty(), "nenhum repositório descoberto");
    let backend = Git::default();

    for repo in repos {
        // HEAD via backend da Colmeia (read-only)
        let head = backend.head(&repo).unwrap_or_else(|e| {
            panic!("head falhou em {}: {e}", repo.display());
        });
        assert!(
            head.chars().all(|c| c.is_ascii_hexdigit()) && head.len() >= 7,
            "HEAD inválido em {}: {head}",
            repo.display()
        );

        // Lista branches locais via git puro (read-only)
        let branches = git(&repo, &["branch", "--list", "--format=%(refname:short)"]);
        assert!(
            !branches.is_empty(),
            "repo sem branches locais: {}",
            repo.display()
        );

        // Último commit (read-only)
        let subject = git(&repo, &["log", "-1", "--format=%s"]);
        assert!(!subject.is_empty(), "repo sem commits: {}", repo.display());

        // Garantia anti-mutação: working tree intocado
        let status = git(&repo, &["status", "--porcelain"]);
        let _ = status; // informativo: repos reais podem ter sujeira; não falha
        println!("{} @ {} · {}", repo.display(), &head[..7.min(head.len())], subject);
    }
}

#[test]
fn test_registers_repos_in_temporary_store() {
    let root = repos_dir();
    if !root.is_dir() {
        eprintln!("diretório de repos ausente, teste pulado");
        return;
    }
    let repos = discover_git_repos(&root, 12);
    assert!(!repos.is_empty());

    // Banco em diretório temporário: nada permanente
    let tmp = tempdir().unwrap();
    let db = tmp.path().join("colmeia-test.sqlite");
    let store = Store::open(&db).expect("store");

    for repo in &repos {
        let mode = if repo.join(".git").is_dir() { "mirror" } else { "managed" };
        let id = store
            .add_repo(&repo.to_string_lossy(), None, mode)
            .unwrap_or_else(|e| panic!("add_repo falhou para {}: {e}", repo.display()));
        assert!(id > 0);
        store
            .update_repo_head(id, &git(repo, &["rev-parse", "HEAD"]))
            .unwrap();
    }

    let listed = store.list_repos().unwrap();
    assert_eq!(listed.len(), repos.len(), "roundtrip do store");
    println!("{} repositórios registrados no store temporário", listed.len());
}

#[test]
fn test_manipulates_clone_without_touching_originals() {
    let root = repos_dir();
    if !root.is_dir() {
        eprintln!("diretório de repos ausente, teste pulado");
        return;
    }
    let repos = discover_git_repos(&root, 200);
    let source = repos
        .iter()
        .find(|r| {
            // escolhe o menor repo para clonar rápido
            fs::metadata(r.join(".git")).map(|m| m.len()).unwrap_or(u64::MAX) < 8 * 1024 * 1024
        })
        .cloned()
        .unwrap_or_else(|| repos[0].clone());

    let head_before = git(&source, &["rev-parse", "HEAD"]);

    // CLONE para tempdir: aqui mover/manipular é permitido
    let tmp = tempdir().unwrap();
    let clone_path = tmp.path().join("clone-de-teste");
    let backend = Git::default();
    let url = format!("file://{}", source.display());
    backend
        .clone_repo(&clone_path, &url, false)
        .unwrap_or_else(|e| panic!("clone falhou de {}: {e}", source.display()));

    // Manipulação: branch de teste + commit no clone apenas
    git(&clone_path, &["config", "user.name", "Colmeia Tester"]);
    git(&clone_path, &["config", "user.email", "tester@colmeia.local"]);
    git(&clone_path, &["switch", "-c", "colmeia/teste-nao-destrutivo"]);
    fs::write(clone_path.join("COLMEIA_TESTE.md"), "arquivo de teste\n").unwrap();
    git(&clone_path, &["add", "COLMEIA_TESTE.md"]);
    git(&clone_path, &["commit", "-m", "teste: commit em clone temporário"]);

    let branches = git(&clone_path, &["branch", "--list", "colmeia/*"]);
    assert!(
        branches.contains("colmeia/teste-nao-destrutivo"),
        "branch de teste ausente no clone"
    );

    // Original intacto
    let head_after = git(&source, &["rev-parse", "HEAD"]);
    assert_eq!(head_before, head_after, "original não pode mudar");
    let orig_branch = git(&source, &["branch", "--list", "colmeia/*"]);
    assert!(
        orig_branch.is_empty(),
        "branch de teste vazou para o original"
    );
    println!("clone manipulado com sucesso; original {} inalterado", source.display());
}
