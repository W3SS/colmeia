use std::path::Path;
use std::process::{Command, Output};

pub trait GitBackend: Send + Sync {
    fn clone_repo(&self, dest: &Path, url: &str, bare: bool) -> anyhow::Result<()>;
    fn head(&self, repo: &Path) -> anyhow::Result<String>;
    fn fetch(&self, repo: &Path, remote: &str) -> anyhow::Result<()>;
    fn ff_merge(&self, repo: &Path, refspec: &str) -> anyhow::Result<bool>;
    fn changed_files(&self, repo: &Path, range: &str) -> anyhow::Result<Vec<String>>;
    fn numstat(&self, repo: &Path, commit: &str) -> anyhow::Result<Vec<(String, usize)>>;
}

#[derive(Clone, Debug)]
pub struct Git {
    pub cap_mb: u64,
}

impl Default for Git {
    fn default() -> Self {
        Self { cap_mb: 256 }
    }
}

impl Git {
    pub fn new(cap_mb: u64) -> Self {
        Self { cap_mb }
    }

    pub fn run(&self, repo: &Path, args: &[&str]) -> anyhow::Result<Output> {
        self.run_with_env(repo, args, &[])
    }

    /// Igual a `run`, mas injeta variáveis de ambiente no processo filho.
    /// Usado para desativar editores interativos (ex.: GIT_MERGE_AUTOEDIT=no).
    pub fn run_with_env(
        &self,
        repo: &Path,
        args: &[&str],
        envs: &[(&str, &str)],
    ) -> anyhow::Result<Output> {
        let mut cmd = Command::new("git");
        cmd.current_dir(repo).args(args);
        for (key, value) in envs {
            cmd.env(key, value);
        }

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let bytes = self.cap_mb * 1024 * 1024;
            unsafe {
                cmd.pre_exec(move || {
                    let rlim = libc::rlimit {
                        rlim_cur: bytes,
                        rlim_max: bytes,
                    };
                    // RLIMIT_AS limita o espaço total de endereçamento virtual
                    libc::setrlimit(libc::RLIMIT_AS, &rlim);
                    Ok(())
                });
            }
        }

        let out = cmd.output()?;
        Ok(out)
    }
}

impl GitBackend for Git {
    fn clone_repo(&self, dest: &Path, url: &str, bare: bool) -> anyhow::Result<()> {
        let parent = dest.parent().unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;
        let mut args = vec!["clone"];
        if bare {
            args.push("--bare");
            args.push("--filter=blob:none");
        }
        let dest_str = dest.to_string_lossy();
        args.push(url);
        args.push(&dest_str);

        let out = self.run(parent, &args)?;
        anyhow::ensure!(
            out.status.success(),
            "Git clone falhou: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(())
    }

    fn head(&self, repo: &Path) -> anyhow::Result<String> {
        let out = self.run(repo, &["rev-parse", "HEAD"])?;
        anyhow::ensure!(
            out.status.success(),
            "git rev-parse HEAD falhou: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    fn fetch(&self, repo: &Path, remote: &str) -> anyhow::Result<()> {
        let out = self.run(repo, &["fetch", remote])?;
        anyhow::ensure!(
            out.status.success(),
            "git fetch falhou: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(())
    }

    fn ff_merge(&self, repo: &Path, refspec: &str) -> anyhow::Result<bool> {
        let out = self.run(repo, &["merge", "--ff-only", refspec])?;
        Ok(out.status.success())
    }

    fn changed_files(&self, repo: &Path, range: &str) -> anyhow::Result<Vec<String>> {
        let out = self.run(repo, &["diff", "--name-only", range])?;
        if !out.status.success() {
            return Ok(Vec::new());
        }
        let files = String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        Ok(files)
    }

    fn numstat(&self, repo: &Path, commit: &str) -> anyhow::Result<Vec<(String, usize)>> {
        let out = self.run(repo, &["show", "--numstat", "--format=", commit])?;
        if !out.status.success() {
            return Ok(Vec::new());
        }
        let mut units = Vec::new();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let mut parts = line.split_whitespace();
            if let (Some(a), Some(d), Some(p)) = (parts.next(), parts.next(), parts.next()) {
                let add: usize = a.parse().unwrap_or(0);
                let del: usize = d.parse().unwrap_or(0);
                units.push((p.to_string(), add + del));
            }
        }
        Ok(units)
    }
}
