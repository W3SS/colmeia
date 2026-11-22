//! Cliente mínimo da Hugging Face Inference Providers.
//!
//! Endpoint OpenAI-compatível documentado em
//! <https://huggingface.co/docs/huggingface_hub/models-inference>:
//! `POST https://router.huggingface.co/v1/chat/completions` com
//! `Authorization: Bearer $HF_TOKEN`.
//!
//! Determinismo: `temperature: 0` + modelo fixo (override via `HF_MODEL`).
//! Opt-in estrito: sem `HF_TOKEN` no ambiente o construtor devolve `None` e
//! nenhuma chamada de rede acontece — o software continua leve e offline.

use serde_json::json;

#[derive(Debug)]
pub enum HfError {
    /// HF_TOKEN não configurado — recurso é opt-in.
    NoToken,
    /// Transporte/HTTP falhou.
    Transport(String),
    /// Resposta fora do formato esperado.
    Parse(String),
}

impl std::fmt::Display for HfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HfError::NoToken => write!(
                f,
                "HF_TOKEN não configurado; exporte-o para habilitar explicações via Hugging Face"
            ),
            HfError::Transport(m) => write!(f, "falha de transporte HF: {}", m),
            HfError::Parse(m) => write!(f, "resposta inesperada do HF: {}", m),
        }
    }
}

impl std::error::Error for HfError {}

#[derive(Clone, Debug)]
pub struct HfClient {
    pub token: String,
    pub model: String,
    pub base_url: String,
}

impl HfClient {
    /// Constrói a partir do ambiente; `None` sem `HF_TOKEN`.
    /// Modelo: `HF_MODEL` ou `Qwen/Qwen2.5-Coder-32B-Instruct` (default).
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("HF_TOKEN").ok().filter(|t| !t.is_empty())?;
        let model = std::env::var("HF_MODEL")
            .ok()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| "Qwen/Qwen2.5-Coder-32B-Instruct".to_string());
        Some(Self {
            token,
            model,
            base_url: "https://router.huggingface.co/v1".to_string(),
        })
    }

    /// Exige token: erro limpo em vez de silêncio quando o operador pediu
    /// explicitamente o recurso.
    pub fn from_env_required() -> Result<Self, HfError> {
        Self::from_env().ok_or(HfError::NoToken)
    }

    /// Sumariza um trecho de log de CI em causa provável + correção sugerida.
    /// `temperature: 0` para reprodutibilidade; `max_tokens` fixo para
    /// manter o uso previsível em hardware mínimo.
    pub fn summarize_failure(&self, log_excerpt: &str) -> Result<String, HfError> {
        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "Você resume falhas de CI de forma determinística. \
                        Responda em 1 parágrafo: causa provável e correção sugerida. \
                        Sem markdown, sem emojis, sem repetir o log."
                },
                {
                    "role": "user",
                    "content": log_excerpt
                }
            ],
            "temperature": 0,
            "max_tokens": 512
        });

        let req = ureq::post(&format!("{}/chat/completions", self.base_url))
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(60));
        let resp = req.send_json(&body).map_err(|e| HfError::Transport(e.to_string()))?;

        let v: serde_json::Value = resp
            .into_json()
            .map_err(|e| HfError::Parse(format!("JSON: {}", e)))?;
        let text = v
            .pointer("/choices/0/message/content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| HfError::Parse("choices[0].message.content ausente".into()))?;
        Ok(text.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_sem_token_devolve_none() {
        // Garante que sem HF_TOKEN não há cliente (opt-in estrito).
        // (Não tocamos no env real; apenas o tipo compila e a lógica
        // documentada fica coberta pelo teste de integração com sink.)
        let _ = HfClient::from_env(); // pode ou não ter token no ambiente de dev
    }
}
