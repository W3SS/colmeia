use colmeia::git::{Git, GitBackend};
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_git_backend_operations() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path();

    // 1. Inicializa um repositório git real para teste
    let init_out = Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("git init falhou");
    assert!(init_out.status.success());

    // Configura credenciais para permitir commit
    let _ = Command::new("git")
        .args(["config", "user.name", "Colmeia Tester"])
        .current_dir(repo_path)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "tester@colmeia.local"])
        .current_dir(repo_path)
        .output();

    // Cria primeiro commit
    let file1 = repo_path.join("file1.txt");
    std::fs::write(&file1, "Linha 1\nLinha 2\n").unwrap();

    let _ = Command::new("git")
        .args(["add", "file1.txt"])
        .current_dir(repo_path)
        .output();
    let commit_out = Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("git commit falhou");
    assert!(commit_out.status.success());

    let git = Git::default();

    // 2. Testa head()
    let head_sha = git.head(repo_path).expect("Falha ao obter HEAD");
    assert_eq!(head_sha.len(), 40);

    // 3. Testa numstat()
    let stats = git
        .numstat(repo_path, "HEAD")
        .expect("Falha ao rodar numstat");
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].0, "file1.txt");
    assert_eq!(stats[0].1, 2);

    // 4. Cria branch secundária e testa ff_merge
    let _ = Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo_path)
        .output();
    let file2 = repo_path.join("file2.txt");
    std::fs::write(&file2, "Feature content\n").unwrap();
    let _ = Command::new("git")
        .args(["add", "file2.txt"])
        .current_dir(repo_path)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "Feature commit"])
        .current_dir(repo_path)
        .output();

    // Volta para o branch padrão e executa ff_merge
    let on_main = Command::new("git")
        .args(["checkout", "main"])
        .current_dir(repo_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !on_main {
        let status = Command::new("git")
            .args(["checkout", "master"])
            .current_dir(repo_path)
            .status()
            .unwrap();
        assert!(status.success());
    }

    let ff_result = git
        .ff_merge(repo_path, "feature")
        .expect("Falha no ff_merge");
    assert!(ff_result);

    // Verifica que o arquivo da branch incorporada existe
    assert!(file2.exists());
}
