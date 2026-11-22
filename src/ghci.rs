//! Integração com a GitHub CLI (`gh`) para automação determinística de CI.
//!
//! Baseado na documentação oficial do GitHub CLI
//! (<https://docs.github.com/pt/github-cli/github-cli>):
//! - `gh run list --status failure --json …` → detecção de falhas de CI;
//! - `gh run view <id> --log-failed` → só os passos que falharam;
//! - `gh run rerun <id> --failed` → re-executa só os jobs falhos (triagem
//!   de flaky vs. falha real, conforme padrão gh-fix-ci);
//! - `gh api` como escape hatch REST (`--paginate`).
//!
//! Sem crates novas: tudo via `std::process::Command`. Chamadas são
//! somente-leitura por padrão; a única mutação (`rerun_failed`) exige
//! chamada explícita — nunca disparada automaticamente pelo scanner.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum GhciError {
    /// `gh` não encontrado no PATH.
    NotInstalled,
    /// Processo saiu com código != 0.
    Exit { code: Option<i32>, stderr: String },
    /// Saída não era JSON válido no formato esperado.
    Parse(String),
}

impl std::fmt::Display for GhciError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GhciError::NotInstalled => {
                write!(f, "GitHub CLI (gh) não instalado ou fora do PATH")
            }
            GhciError::Exit { code, stderr } => {
                write!(f, "gh falhou (código {:?}): {}", code, stderr.trim())
            }
            GhciError::Parse(m) => write!(f, "resposta inesperada do gh: {}", m),
        }
    }
}

impl std::error::Error for GhciError {}

/// Um run de workflow falho, como devolvido por
/// `gh run list --status failure --json …`.
#[derive(Debug, Clone, PartialEq)]
pub struct CiRun {
    pub database_id: u64,
    pub workflow: String,
    pub branch: String,
    pub conclusion: String,
    pub created_at: String,
    pub url: String,
}

/// Wrapper mínimo e testável sobre a GitHub CLI.
#[derive(Clone, Debug)]
pub struct GhCli {
    pub gh_path: PathBuf,
}

impl Default for GhCli {
    fn default() -> Self {
        Self {
            gh_path: PathBuf::from("gh"),
        }
    }
}

impl GhCli {
    pub fn new(gh_path: impl Into<PathBuf>) -> Self {
        Self {
            gh_path: gh_path.into(),
        }
    }

    /// `true` se o binário `gh` existe e responde a `--version`.
    pub fn available(&self) -> bool {
        Command::new(&self.gh_path)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Lista os últimos runs com falha do repositório `owner/repo`.
    ///
    /// Determinístico: ordenação e limite são fixados pela CLI; a resposta
    /// é parseada como JSON (saída estruturada, nunca scraping de texto).
    pub fn detect_failures(&self, repo: &str, limit: usize) -> Result<Vec<CiRun>, GhciError> {
        let limit_s = limit.to_string();
        let out = self.run(&[
            "run",
            "list",
            "--repo",
            repo,
            "--status",
            "failure",
            "--limit",
            &limit_s,
            "--json",
            "databaseId,workflowName,headBranch,conclusion,createdAt,url",
        ])?;
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| {
            GhciError::Parse(format!("JSON de `gh run list`: {}", e))
        })?;
        let arr = v
            .as_array()
            .ok_or_else(|| GhciError::Parse("esperava array".into()))?;
        let mut runs = Vec::with_capacity(arr.len());
        for item in arr {
            runs.push(CiRun {
                database_id: item
                    .get("databaseId")
                    .and_then(|x| x.as_u64())
                    .ok_or_else(|| GhciError::Parse("databaseId ausente".into()))?,
                workflow: item
                    .get("workflowName")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                branch: item
                    .get("headBranch")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                conclusion: item
                    .get("conclusion")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                created_at: item
                    .get("createdAt")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
                url: item
                    .get("url")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
        Ok(runs)
    }

    /// Retorna o log **apenas dos passos que falharam** de um run,
    /// truncado a `max_chars`. Padrão gh-fix-ci: alimentar um sumarizador
    /// com o trecho relevante, não com o log inteiro.
    pub fn failure_log(
        &self,
        repo: &str,
        run_id: u64,
        max_chars: usize,
    ) -> Result<String, GhciError> {
        let id = run_id.to_string();
        let out = self.run(&["run", "view", &id, "--repo", repo, "--log-failed"])?;
        let log = String::from_utf8_lossy(&out.stdout).into_owned();
        Ok(truncate_chars(&log, max_chars))
    }

    /// Re-executa apenas os jobs falhos de um run (`gh run rerun --failed`).
    /// Triagem de flaky: se o rerun passa, a falha era transitória.
    /// NUNCA chamado pelo scanner; só por comando explícito do operador.
    pub fn rerun_failed(&self, repo: &str, run_id: u64) -> Result<(), GhciError> {
        let id = run_id.to_string();
        self.run(&["run", "rerun", &id, "--repo", repo, "--failed"])?;
        Ok(())
    }

    /// Extrai `owner/repo` da URL de um remote git (https ou ssh).
    /// Retorna `None` se o remote não aponta para github.com.
    pub fn repo_slug_from_remote(remote: &str) -> Option<String> {
        let r = remote.trim();
        let r = r.strip_suffix(".git").unwrap_or(r);
        // https://github.com/owner/repo
        if let Some(rest) = r.strip_prefix("https://github.com/") {
            return Some(rest.trim_end_matches('/').to_string());
        }
        if let Some(rest) = r.strip_prefix("http://github.com/") {
            return Some(rest.trim_end_matches('/').to_string());
        }
        // git@github.com:owner/repo
        if let Some(rest) = r.strip_prefix("git@github.com:") {
            return Some(rest.trim_end_matches('/').to_string());
        }
        // ssh://git@github.com/owner/repo
        if let Some(rest) = r.strip_prefix("ssh://git@github.com/") {
            return Some(rest.trim_end_matches('/').to_string());
        }
        None
    }

    fn run(&self, args: &[&str]) -> Result<std::process::Output, GhciError> {
        // Resolvemos o binário uma vez: se não existir, erro limpo em vez
        // de texto de shell no stderr.
        let bin = resolve_bin(&self.gh_path);
        let bin = bin.ok_or(GhciError::NotInstalled)?;
        let out = Command::new(&bin)
            .args(args)
            // Nunca herda terminal; gh fica em modo não-interativo.
            .env("GH_PROMPT_DISABLED", "1")
            .output()
            .map_err(|e| GhciError::Exit {
                code: None,
                stderr: format!("spawn: {}", e),
            })?;
        if !out.status.success() {
            return Err(GhciError::Exit {
                code: out.status.code(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            });
        }
        Ok(out)
    }
}

/// Resolve o caminho do binário respeitando um caminho absoluto ou
/// procurando no PATH (sem depender de shell).
fn resolve_bin(p: &Path) -> Option<PathBuf> {
    if p.is_absolute() || p.components().count() > 1 {
        return p.exists().then(|| p.to_path_buf());
    }
    let name = p.to_string_lossy();
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let cand = dir.join(name.as_ref());
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Trunca no limite de caracteres respeitando fronteira de linha UTF-8.
pub fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max).collect();
    format!("{}… [truncado em {} chars]", cut, max)
}

/// Achado de uma passada de `ci-scan`: run falho recém-detectado + explicação.
#[derive(Debug)]
pub struct CiFinding {
    pub repo: String,
    pub run: CiRun,
    pub explanation: Option<String>,
}

/// Resultado determinístico de uma passada de scan.
#[derive(Debug, Default)]
pub struct CiScanReport {
    pub findings: Vec<CiFinding>,
    /// Runs já vistos em passadas anteriores (dedup).
    pub deduped: u64,
    /// Slugs pulados por orçamento de requisições (rate budget).
    pub rate_limited: u64,
}

/// Orquestração idempotente do scan de falhas de CI.
///
/// Determinismo:
/// - slugs são ordenados e deduplicados;
/// - cada run é reportado no máximo uma vez (tabela `ci_runs`);
/// - orçamento por hora (`rate_consume`) impede loops de requisições;
/// - explicação só ocorre com `hf: Some(..)` e `explain: true`.
pub fn ci_scan(
    store: &crate::store::Store,
    gh: &GhCli,
    slugs: &[String],
    limit: usize,
    explain: bool,
    hf: Option<&crate::hf::HfClient>,
) -> Result<CiScanReport, GhciError> {
    let mut uniq: Vec<&str> = slugs.iter().map(|s| s.as_str()).collect();
    uniq.sort_unstable();
    uniq.dedup();

    const RATE_CAP_PER_HOUR: i64 = 60;
    const LOG_EXCERPT_CHARS: usize = 8_000;

    let mut report = CiScanReport::default();
    for slug in uniq {
        // Orçamento: sem isso, um loop de scan vira bombardeio de API.
        let ok = store
            .rate_consume("github-cli", RATE_CAP_PER_HOUR)
            .map_err(|e| GhciError::Exit {
                code: None,
                stderr: format!("rate_budget: {}", e),
            })?;
        if !ok {
            report.rate_limited += 1;
            continue;
        }

        let runs = gh.detect_failures(slug, limit)?;
        for run in runs {
            let run_id = run.database_id as i64;
            if store.ci_seen(run_id).unwrap_or(false) {
                report.deduped += 1;
                continue;
            }
            store.ci_mark(run_id, slug, &run.workflow, &run.conclusion).ok();

            let mut finding = CiFinding {
                repo: slug.to_string(),
                run: run.clone(),
                explanation: None,
            };
            if explain && hf.is_some() {
                if let Ok(log) = gh.failure_log(slug, run.database_id, LOG_EXCERPT_CHARS) {
                    finding.explanation = hf.and_then(|h| h.summarize_failure(&log).ok());
                }
            }
            // Evento no store: consumidores (hooks, TUI, web) reagem ao achado.
            // repo_id=0: o slug no payload identifica o repo (events.repo_id é
            // nullable e sem FK enforcement; ver schema.sql).
            store
                .event(
                    0,
                    "ci_failure",
                    &serde_json::json!({
                        "repo": slug,
                        "run_id": run.database_id,
                        "workflow": run.workflow,
                        "branch": run.branch,
                        "url": run.url,
                    }),
                )
                .ok();
            report.findings.push(finding);
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_https_e_ssh() {
        assert_eq!(
            GhCli::repo_slug_from_remote("https://github.com/o/r.git"),
            Some("o/r".into())
        );
        assert_eq!(
            GhCli::repo_slug_from_remote("git@github.com:o/r"),
            Some("o/r".into())
        );
        assert_eq!(
            GhCli::repo_slug_from_remote("ssh://git@github.com/o/r.git"),
            Some("o/r".into())
        );
        assert_eq!(GhCli::repo_slug_from_remote("https://gitlab.com/o/r"), None);
        assert_eq!(GhCli::repo_slug_from_remote(""), None);
    }

    #[test]
    fn truncate_respeita_limite() {
        let s = "a".repeat(100);
        let t = truncate_chars(&s, 10);
        assert!(t.starts_with(&"a".repeat(10)));
        assert!(t.contains("truncado"));
        assert_eq!(truncate_chars("curto", 10), "curto");
    }
}
