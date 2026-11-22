use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub sev: &'static str,
    pub line: usize,
    pub msg: String,
    pub owasp: &'static str,
}

pub fn is_skill(path: &str) -> bool {
    path.ends_with("SKILL.md")
        || path.ends_with(".prompt")
        || path.ends_with(".skill")
        || path.contains("skills/")
}

pub fn scan(path: &str, text: &str) -> Vec<Finding> {
    let mut out = Vec::new();

    // Regex de segredos hardcoded
    let secret =
        Regex::new(r"(ghp_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16})").unwrap();

    // Regex de APIs perigosas (código não confiável)
    let danger =
        Regex::new(r"(eval\(|exec\(|shell\s*=\s*True|pickle\.loads|yaml\.load\()").unwrap();

    // Regex de prompt injection em skills
    let inject = Regex::new(
        r"(?i)(ignore previous instructions|disregard your instructions|you are now a|jailbreak)",
    )
    .unwrap();

    for (i, line) in text.lines().enumerate() {
        let line_num = i + 1;

        if secret.is_match(line) {
            out.push(Finding {
                rule: "secret.token",
                sev: "critical",
                line: line_num,
                msg: "Token ou credencial sensível detectada no código".into(),
                owasp: "ASVS",
            });
        }

        if danger.is_match(line) {
            out.push(Finding {
                rule: "code.dangerous",
                sev: "high",
                line: line_num,
                msg: "Invocação de API de execução dinâmica perigosa".into(),
                owasp: "Top10-A03",
            });
        }

        if is_skill(path) {
            if inject.is_match(line) {
                out.push(Finding {
                    rule: "skill.injection.override",
                    sev: "high",
                    line: line_num,
                    msg: "Possível Prompt Injection detectado em arquivo de Skill/Prompt".into(),
                    owasp: "LLM01",
                });
            }

            if line.chars().any(|c| {
                matches!(
                    c as u32,
                    0x200B..=0x200F | 0x202A..=0x202E | 0xFEFF
                )
            }) {
                out.push(Finding {
                    rule: "skill.hidden_unicode",
                    sev: "critical",
                    line: line_num,
                    msg: "Caracteres Unicode invisíveis ou bidi override detectados em Skill"
                        .into(),
                    owasp: "LLM01",
                });
            }
        }
    }

    out
}

/// Calcula a Entropia de Shannon de uma string para detectar payloads cifrados ou chaves
pub fn entropy(s: &str) -> f64 {
    let mut freqs = [0u32; 256];
    for b in s.bytes() {
        freqs[b as usize] += 1;
    }
    let n = s.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    -freqs
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / n;
            p * p.log2()
        })
        .sum::<f64>()
}
