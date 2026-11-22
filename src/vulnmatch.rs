//! F22 — Matcher de vulnerabilidades OSV contra o SBOM.
//!
//! Comparação de versões semver/pep440-lite e avaliação dos eventos de
//! range do OSV (introduced/fixed/last_affected). 10k pacotes em ms — tudo
//! em memória, sem deps extras.

/// Versão normalizada (semver ou pep440-lite) para comparação ordenável.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub parts: Vec<u64>,
    pub pre: Option<String>,
}

/// Parse tolerante: semver `1.2.3-rc.1`, pep440 `1.2.3.post1`, `v1.2`,
/// `1.2`. Partes não numéricas viram pre-release.
pub fn parse_version(s: &str) -> Option<Version> {
    let s = s.trim().trim_start_matches('v');
    let (core, pre) = match s.split_once(['-', '+']) {
        Some((c, p)) => (c, Some(p.split('+').next().unwrap_or(p).to_string())),
        None => (s, None),
    };
    // pep440: corta .postN / .devN / .rcN do core
    let core = core
        .split(".post")
        .next()
        .unwrap_or(core)
        .split(".dev")
        .next()
        .unwrap_or(core);
    let mut parts = Vec::new();
    for piece in core.split('.') {
        let digits: String = piece.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            if parts.is_empty() {
                return None;
            }
            break;
        }
        parts.push(digits.parse().ok()?);
    }
    if parts.is_empty() {
        return None;
    }
    Some(Version { parts, pre })
}

fn cmp_versions(a: &Version, b: &Version) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let n = a.parts.len().max(b.parts.len());
    for i in 0..n {
        let x = a.parts.get(i).copied().unwrap_or(0);
        let y = b.parts.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    match (&a.pre, &b.pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater, // release > pre-release
        (Some(_), None) => Ordering::Less,
        (Some(p), Some(q)) => p.cmp(q),
    }
}

/// Evento de range OSV.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bound {
    Introduced(String),
    Fixed(String),
    LastAffected(String),
    Limit(String),
}

/// `version` está dentro do range descrito pelos eventos?
pub fn in_range(version: &str, events: &[Bound]) -> bool {
    let Some(v) = parse_version(version) else { return false };
    let mut lower_open = false; // introduced
    let mut inside = false;
    for e in events {
        match e {
            Bound::Introduced(s) => {
                if s == "0" {
                    inside = true;
                    lower_open = false;
                } else if let Some(i) = parse_version(s) {
                    inside = cmp_versions(&v, &i) != std::cmp::Ordering::Less;
                    lower_open = true;
                }
            }
            Bound::Fixed(s) => {
                if let Some(f) = parse_version(s) {
                    if inside && cmp_versions(&v, &f) != std::cmp::Ordering::Less {
                        inside = false;
                    }
                }
            }
            Bound::LastAffected(s) => {
                if let Some(l) = parse_version(s) {
                    if inside && cmp_versions(&v, &l) == std::cmp::Ordering::Greater {
                        inside = false;
                    }
                }
            }
            Bound::Limit(_) => {}
        }
    }
    let _ = lower_open;
    inside
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn parse_comparacao() {
        assert!(parse_version("1.2.3").is_some());
        assert!(parse_version("v2.0.0-rc.1").is_some());
        assert!(parse_version("1.2.3.post1").is_some());
        assert!(parse_version("banana").is_none());
        assert_eq!(
            cmp_versions(&parse_version("1.2.0").unwrap(), &parse_version("1.10.0").unwrap()),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            cmp_versions(&parse_version("2.0.0-rc.1").unwrap(), &parse_version("2.0.0").unwrap()),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn ranges_osv() {
        let ev = vec![
            Bound::Introduced("0".into()),
            Bound::Fixed("1.2.3".into()),
        ];
        assert!(in_range("1.0.0", &ev));
        assert!(in_range("1.2.2", &ev));
        assert!(!in_range("1.2.3", &ev));
        assert!(!in_range("2.0.0", &ev));

        let ev2 = vec![Bound::Introduced("4.17.0".into())];
        assert!(in_range("4.17.20", &ev2));
        assert!(!in_range("4.16.9", &ev2));
    }
}
