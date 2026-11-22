//! F32/F33/F34/F35 — Adaptadores de forge (GitHub, GitLab, Forgejo).
//!
//! `ForgeAdapter` abstrai criação de PR e listagem. Implementações usam
//! `ureq` (já na árvore) com `api_base` configurável — os testes apontam
//! para um servidor `tiny_http` local. Rate limiting via tabela
//! `rate_budget` com orçamento por hora e sinalização de backoff quando a
//! API responde 403/429.

use crate::store::Store;

#[derive(Debug, PartialEq)]
pub enum ForgeError {
    RateLimited { retry_after_secs: u64 },
    Http(u16, String),
    Transport(String),
}

impl std::fmt::Display for ForgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForgeError::RateLimited { retry_after_secs } => {
                write!(f, "rate limited; tente novamente em {retry_after_secs}s")
            }
            ForgeError::Http(code, body) => write!(f, "HTTP {code}: {body}"),
            ForgeError::Transport(e) => write!(f, "transporte: {e}"),
        }
    }
}
impl std::error::Error for ForgeError {}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct PrInfo {
    pub number: u64,
    pub title: String,
    pub state: String,
    pub url: String,
}

pub trait ForgeAdapter: Send + Sync {
    fn forge_name(&self) -> &'static str;
    fn create_pr(
        &self,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<PrInfo, ForgeError>;
    fn list_open_prs(&self, repo: &str) -> Result<Vec<PrInfo>, ForgeError>;
}

/// Orçamento por hora aplicado antes de cada chamada.
pub fn rate_gate(store: &Store, forge: &str, cap_per_hour: i64) -> Result<(), ForgeError> {
    match store.rate_consume(forge, cap_per_hour) {
        Ok(true) => Ok(()),
        Ok(false) => Err(ForgeError::RateLimited {
            retry_after_secs: 3600,
        }),
        Err(e) => Err(ForgeError::Transport(e.to_string())),
    }
}

fn map_ureq(err: ureq::Error) -> ForgeError {
    match err {
        ureq::Error::Status(code, resp) => {
            let retry = resp
                .header("retry-after")
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            let body = resp.into_string().unwrap_or_default();
            if code == 403 || code == 429 {
                ForgeError::RateLimited {
                    retry_after_secs: retry,
                }
            } else {
                ForgeError::Http(code, body.chars().take(200).collect())
            }
        }
        ureq::Error::Transport(t) => {
            let msg = t.to_string();
            // alguns caminhos do ureq reportam status de erro como Transport
            for code in [403u16, 429] {
                if msg.contains(&format!("status code {code}")) {
                    return ForgeError::RateLimited {
                        retry_after_secs: 60,
                    };
                }
            }
            ForgeError::Transport(msg)
        }
    }
}

// ── GitHub ─────────────────────────────────────────────────────────────────

pub struct GitHubAdapter {
    token: String,
    pub api_base: String,
    client: ureq::Agent,
}

impl GitHubAdapter {
    pub fn new(token: impl Into<String>) -> Self {
        Self::with_api_base(token, "https://api.github.com")
    }

    pub fn with_api_base(token: impl Into<String>, api_base: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            api_base: api_base.into().trim_end_matches('/').to_string(),
            client: ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_secs(5))
                .timeout_read(std::time::Duration::from_secs(15))
                .build(),
        }
    }

    fn auth(&self, req: ureq::Request) -> ureq::Request {
        req.set("Authorization", &format!("Bearer {}", self.token))
            .set("Accept", "application/vnd.github+json")
            .set("User-Agent", "colmeia/0.1")
    }

    fn parse_pr(v: &serde_json::Value) -> PrInfo {
        PrInfo {
            number: v["number"].as_u64().unwrap_or(0),
            title: v["title"].as_str().unwrap_or("").to_string(),
            state: v["state"].as_str().unwrap_or("").to_string(),
            url: v["html_url"]
                .as_str()
                .or_else(|| v["url"].as_str())
                .unwrap_or("")
                .to_string(),
        }
    }
}

impl ForgeAdapter for GitHubAdapter {
    fn forge_name(&self) -> &'static str {
        "github"
    }

    fn create_pr(
        &self,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<PrInfo, ForgeError> {
        let payload = serde_json::json!({
            "title": title, "body": body, "head": head, "base": base
        });
        let resp = self
            .auth(self.client.post(&format!("{}/repos/{}/pulls", self.api_base, repo)))
            .send_json(payload)
            .map_err(|e| ForgeError::Transport(e.to_string()))?;
        Ok(Self::parse_pr(&resp.into_json::<serde_json::Value>().map_err(|e| ForgeError::Transport(e.to_string()))?))
    }

    fn list_open_prs(&self, repo: &str) -> Result<Vec<PrInfo>, ForgeError> {
        let resp = self
            .auth(
                self.client
                    .get(&format!("{}/repos/{}/pulls", self.api_base, repo))
                    .query("state", "open"),
            )
            .call()
            .map_err(map_ureq)?;
        let arr = resp
            .into_json::<serde_json::Value>()
            .map_err(|e| ForgeError::Transport(e.to_string()))?;
        Ok(arr
            .as_array()
            .map(|a| a.iter().map(Self::parse_pr).collect())
            .unwrap_or_default())
    }
}

// ── GitLab / Forgejo (REST mínimo, mesmo formato de PR = "merge request") ──

pub struct GitLabLikeAdapter {
    token: String,
    pub api_base: String,
    forge: &'static str,
    client: ureq::Agent,
}

impl GitLabLikeAdapter {
    pub fn gitlab(token: impl Into<String>, api_base: impl Into<String>) -> Self {
        Self::new(token, api_base, "gitlab")
    }

    pub fn forgejo(token: impl Into<String>, api_base: impl Into<String>) -> Self {
        Self::new(token, api_base, "forgejo")
    }

    fn new(token: impl Into<String>, api_base: impl Into<String>, forge: &'static str) -> Self {
        Self {
            token: token.into(),
            api_base: api_base.into().trim_end_matches('/').to_string(),
            forge,
            client: ureq::AgentBuilder::new()
                .timeout_connect(std::time::Duration::from_secs(5))
                .timeout_read(std::time::Duration::from_secs(15))
                .build(),
        }
    }
}

impl ForgeAdapter for GitLabLikeAdapter {
    fn forge_name(&self) -> &'static str {
        self.forge
    }

    fn create_pr(
        &self,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<PrInfo, ForgeError> {
        // GitLab: POST /projects/:id/merge_requests; Forgejo: POST /repos/{owner}/{repo}/pulls
        let (url, project) = if self.forge == "gitlab" {
            (
                format!("{}/projects/{}/merge_requests", self.api_base, repo.replace('/', "%2F")),
                true,
            )
        } else {
            (
                format!("{}/repos/{}/pulls", self.api_base, repo),
                false,
            )
        };
        let mut payload = serde_json::json!({ "title": title, "body": body });
        if project {
            payload["source_branch"] = serde_json::json!(head);
            payload["target_branch"] = serde_json::json!(base);
        } else {
            payload["head"] = serde_json::json!(head);
            payload["base"] = serde_json::json!(base);
        }
        let resp = self
            .client
            .post(&url)
            .set("PRIVATE-TOKEN", &self.token)
            .set("User-Agent", "colmeia/0.1")
            .send_json(payload)
            .map_err(|e| ForgeError::Transport(e.to_string()))?;
        let v = resp.into_json::<serde_json::Value>().map_err(|e| ForgeError::Transport(e.to_string()))?;
        Ok(PrInfo {
            number: v["number"]
                .as_u64()
                .or_else(|| v["iid"].as_u64())
                .unwrap_or(0),
            title: v["title"].as_str().unwrap_or("").to_string(),
            state: v["state"].as_str().unwrap_or("").to_string(),
            url: v["web_url"]
                .as_str()
                .or_else(|| v["html_url"].as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    fn list_open_prs(&self, repo: &str) -> Result<Vec<PrInfo>, ForgeError> {
        let url = if self.forge == "gitlab" {
            format!(
                "{}/projects/{}/merge_requests?state=opened",
                self.api_base,
                repo.replace('/', "%2F")
            )
        } else {
            format!("{}/repos/{}/pulls?state=open", self.api_base, repo)
        };
        let resp = self
            .client
            .get(&url)
            .set("PRIVATE-TOKEN", &self.token)
            .set("User-Agent", "colmeia/0.1")
            .call()
            .map_err(map_ureq)?;
        let arr = resp.into_json::<serde_json::Value>().map_err(|e| ForgeError::Transport(e.to_string()))?;
        Ok(arr
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|v| PrInfo {
                        number: v["number"]
                            .as_u64()
                            .or_else(|| v["iid"].as_u64())
                            .unwrap_or(0),
                        title: v["title"].as_str().unwrap_or("").to_string(),
                        state: v["state"].as_str().unwrap_or("").to_string(),
                        url: v["web_url"]
                            .as_str()
                            .or_else(|| v["html_url"].as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
}
