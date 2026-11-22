//! F52 — Redação de segredos em logs/audit.
//!
//! Aplica os mesmos padrões do SAST (rules.rs) mais tokens comuns de
//! provedores, substituindo o valor por `***REDACTED***`. Regex compiladas
//! sob demanda via OnceLock — sem crates novas (regex já é dependência).

use regex::Regex;
use std::sync::OnceLock;

/// (regex, grupo do valor sensível) — grupo 0 = redige o match inteiro.
fn patterns() -> &'static Vec<(Regex, usize)> {
    static P: OnceLock<Vec<(Regex, usize)>> = OnceLock::new();
    P.get_or_init(|| {
        let specs: Vec<(&str, usize)> = vec![
            (r"gh[pousr]_[A-Za-z0-9]{20,}", 0),
            (r"github_pat_[A-Za-z0-9_]{20,}", 0),
            (r"sk-[A-Za-z0-9]{20,}", 0),
            (r"AKIA[0-9A-Z]{16}", 0),
            (r"ASIA[0-9A-Z]{16}", 0),
            (r"xox[baprs]-[A-Za-z0-9-]{10,}", 0),
            (r"gh[lr]_[A-Za-z0-9]{20,}", 0),
            (r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z0-9 ]*PRIVATE KEY-----", 0),
            // pares chave=valor: preserva nome da chave e separador
            (r"(?i)\b(api[_-]?key|secret|password|passwd|token|client[_-]?secret)(\s*[:=]\s*)(\S+)", 3),
            (r"(?i)\b(authorization\s*[:=]?\s*bearer\s+)(\S+)", 2),
            (r"(?i)\b(AWS_SECRET_ACCESS_KEY\s*[:=]\s*)(\S+)", 2),
        ];
        specs
            .into_iter()
            .map(|(pat, g)| (Regex::new(pat).unwrap(), g))
            .collect()
    })
}

/// Redige apenas o grupo `value_group`, preservando o resto do match.
fn group_redact(re: &Regex, text: &str, value_group: usize) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for caps in re.captures_iter(text) {
        let m = caps.get(0).unwrap();
        out.push_str(&text[last..m.start()]);
        match caps.get(value_group) {
            Some(v) => {
                out.push_str(&text[m.start()..v.start()]);
                out.push_str("***REDACTED***");
                out.push_str(&text[v.end()..m.end()]);
            }
            None => out.push_str("***REDACTED***"),
        }
        last = m.end();
    }
    out.push_str(&text[last..]);
    out
}

/// Retorna `text` com segredos redigidos. Idempotente.
pub fn redact(text: &str) -> String {
    let mut out = text.to_string();
    for (re, value_group) in patterns().iter() {
        if !re.is_match(&out) {
            continue;
        }
        out = if *value_group == 0 {
            re.replace_all(&out, "***REDACTED***").into_owned()
        } else {
            group_redact(re, &out, *value_group)
        };
    }
    out
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn redige_tokens_e_pares() {
        let s = "token=ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZ1234 fim";
        assert_eq!(redact(s), "token=***REDACTED*** fim");

        let s2 = "Authorization: Bearer abc.def.ghi";
        assert!(redact(s2).contains("***REDACTED***"));
        assert!(!redact(s2).contains("abc.def.ghi"));

        let s3 = "aws AKIAIOSFODNN7EXAMPLE ok";
        assert!(redact(s3).contains("***REDACTED***"));
        assert!(!redact(s3).contains("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn idempotente() {
        let s = "api_key: supersecreta123\n";
        let once = redact(s);
        assert_eq!(redact(&once), once);
    }

    #[test]
    fn nao_redige_texto_comum() {
        let s = "merge concluído em 42s, 3 findings";
        assert_eq!(redact(s), s);
    }
}
