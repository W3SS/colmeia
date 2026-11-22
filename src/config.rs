use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "d_profile")]
    pub profile: String,
    #[serde(default = "d_workers")]
    pub workers: usize,
    #[serde(default = "d_soft")]
    pub memory_soft_mb: u64,
    #[serde(default = "d_hard")]
    pub memory_hard_mb: u64,
    #[serde(default = "d_gitcap")]
    pub git_child_cap_mb: u64,
    #[serde(default = "d_hot")]
    pub hot_watch_count: usize,
    #[serde(default = "d_worktree_max")]
    pub worktree_max: usize,
    #[serde(default)]
    pub security: Security,
    #[serde(default)]
    pub forge: Forge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Security {
    #[serde(default = "d_eco")]
    pub osv_ecosystems: Vec<String>,
    #[serde(default = "d_sev")]
    pub gate_block_severity: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Forge {
    pub github_token_env: Option<String>,
}

fn d_profile() -> String {
    "t2".into()
}
fn d_workers() -> usize {
    2
}
fn d_soft() -> u64 {
    600
}
fn d_hard() -> u64 {
    900
}
fn d_gitcap() -> u64 {
    256
}
fn d_hot() -> usize {
    100
}
fn d_worktree_max() -> usize {
    30
}
fn d_eco() -> Vec<String> {
    vec!["PyPI".into(), "npm".into(), "crates.io".into()]
}
fn d_sev() -> String {
    "high".into()
}

impl Default for Security {
    fn default() -> Self {
        Self {
            osv_ecosystems: d_eco(),
            gate_block_severity: d_sev(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: d_profile(),
            workers: d_workers(),
            memory_soft_mb: d_soft(),
            memory_hard_mb: d_hard(),
            git_child_cap_mb: d_gitcap(),
            hot_watch_count: d_hot(),
            worktree_max: d_worktree_max(),
            security: Security::default(),
            forge: Forge::default(),
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let p = Self::path();
        if p.exists() {
            let content = std::fs::read_to_string(&p)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".colmeia/colmeia.toml")
    }

    /// Auto-tier baseado na RAM total disponível (T1 <= 1GB, T2 <= 6GB, T3 > 6GB)
    pub fn detect_tier() -> &'static str {
        if let Ok(mem) = std::fs::read_to_string("/proc/meminfo") {
            let kb: u64 = mem
                .lines()
                .find(|l| l.starts_with("MemTotal"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok())
                .unwrap_or(4_000_000);

            match kb / 1024 {
                0..=1024 => "t1",
                1025..=6144 => "t2",
                _ => "t3",
            }
        } else {
            // Em macOS ou ambientes onde /proc/meminfo não existe, assume T2 como padrão
            "t2"
        }
    }

    /// Aplica um perfil nomeado (F47): minimum | recommended | maximum.
    /// Perfis desconhecidos são ignorados (mantém os valores atuais).
    pub fn apply_profile(cfg: &mut Config, profile: &str) {
        let (workers, soft, hard, worktree_max, hot) = match profile {
            "minimum" => (1usize, 64u64, 96u64, 5usize, 20usize),
            "maximum" => (4, 448, 512, 60, 200),
            "recommended" => (2, 192, 256, 30, 100),
            _ => return,
        };
        cfg.workers = workers;
        cfg.memory_soft_mb = soft;
        cfg.memory_hard_mb = hard;
        cfg.worktree_max = worktree_max;
        cfg.hot_watch_count = hot;
    }
}
