//! F14 — Merge 3-way estruturado por chave (JSON / TOML / YAML subset).
//!
//! Estratégia determinística: para cada chave do dict/mapeamento resultante:
//! - só `ours` mudou → usa `ours`
//! - só `theirs` mudou → usa `theirs`
//! - ambos mudaram igual → usa o valor comum
//! - ambos mudaram diferente → conflito, emitido como marcador textual
//!   (`<<<<<<< ours`, `=======`, `>>>>>>> theirs`) e listado em `conflicts`.
//!
//! YAML é um subset mínimo: mapeamentos por indentação com valores escalares,
//! suficiente para manifests simples (docker-compose leve, CI básico).

use serde_json::Value as J;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Toml,
    Yaml,
}

#[derive(Debug)]
pub struct MergeResult {
    pub content: String,
    pub conflicts: Vec<String>,
}

impl MergeResult {
    pub fn is_clean(&self) -> bool {
        self.conflicts.is_empty()
    }
}

/// Merge 3-way de conteúdo estruturado.
pub fn merge3(base: &str, ours: &str, theirs: &str, format: Format) -> anyhow::Result<MergeResult> {
    match format {
        Format::Json => merge_json(base, ours, theirs),
        Format::Toml => merge_toml(base, ours, theirs),
        Format::Yaml => merge_yaml(base, ours, theirs),
    }
}

// ── JSON ───────────────────────────────────────────────────────────────────

fn conflict_marker(path: &str, a: &J, b: &J, conflicts: &mut Vec<String>) -> J {
    conflicts.push(path.to_string());
    J::String(format!(
        "<<<<<<< ours\n{}\n=======\n{}\n>>>>>>> theirs",
        serde_json::to_string_pretty(a).unwrap_or_else(|_| a.to_string()),
        serde_json::to_string_pretty(b).unwrap_or_else(|_| b.to_string())
    ))
}

fn merge_json_value(path: &str, base: &J, ours: &J, theirs: &J, conflicts: &mut Vec<String>) -> J {
    match (base, ours, theirs) {
        // dict ×3: fusão recursiva por chave
        (J::Object(b), J::Object(o), J::Object(t)) => {
            let mut out = serde_json::Map::new();
            let mut keys: Vec<&String> = b.keys().chain(o.keys()).chain(t.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                let p = format!("{path}.{k}");
                let vb = b.get(k).unwrap_or(&J::Null);
                let vo = o.get(k).unwrap_or(&J::Null);
                let vt = t.get(k).unwrap_or(&J::Null);
                out.insert(k.clone(), merge_json_value(&p, vb, vo, vt, conflicts));
            }
            J::Object(out)
        }
        // mesmo valor ou só um lado mudou
        _ if ours == theirs => ours.clone(),
        _ if base == ours => theirs.clone(),
        _ if base == theirs => ours.clone(),
        // ambos mudaram diferente
        _ => conflict_marker(path, ours, theirs, conflicts),
    }
}

fn merge_json(base: &str, ours: &str, theirs: &str) -> anyhow::Result<MergeResult> {
    let b: J = serde_json::from_str(base)?;
    let o: J = serde_json::from_str(ours)?;
    let t: J = serde_json::from_str(theirs)?;
    let mut conflicts = Vec::new();
    let merged = merge_json_value("$", &b, &o, &t, &mut conflicts);
    Ok(MergeResult {
        content: serde_json::to_string_pretty(&merged)?,
        conflicts,
    })
}

// ── TOML ───────────────────────────────────────────────────────────────────

fn merge_toml(base: &str, ours: &str, theirs: &str) -> anyhow::Result<MergeResult> {
    let b: toml::Value = base.parse()?;
    let o: toml::Value = ours.parse()?;
    let t: toml::Value = theirs.parse()?;
    // converte para JSON (mesma representação de mapeamentos) e reaproveita
    let bj = toml_to_json(&b);
    let oj = toml_to_json(&o);
    let tj = toml_to_json(&t);
    let mut conflicts = Vec::new();
    let merged = merge_json_value("$", &bj, &oj, &tj, &mut conflicts);
    // serializa o resultado como TOML (sem os marcadores de conflito
    // estruturais — eles ficam apenas em `conflicts`)
    let content = if conflicts.is_empty() {
        merged_json_to_toml(&merged)?
    } else {
        merged.to_string()
    };
    Ok(MergeResult { content, conflicts })
}

fn toml_to_json(v: &toml::Value) -> J {
    match v {
        toml::Value::String(s) => J::String(s.clone()),
        toml::Value::Integer(n) => J::Number((*n).into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f).map_or(J::Null, J::Number),
        toml::Value::Boolean(b) => J::Bool(*b),
        toml::Value::Datetime(d) => J::String(d.to_string()),
        toml::Value::Array(a) => J::Array(a.iter().map(toml_to_json).collect()),
        toml::Value::Table(t) => {
            J::Object(t.iter().map(|(k, v)| (k.clone(), toml_to_json(v))).collect())
        }
    }
}

fn json_to_toml(v: &J) -> toml::Value {
    match v {
        J::String(s) => toml::Value::String(s.clone()),
        J::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml::Value::Integer(i)
            } else {
                toml::Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        J::Bool(b) => toml::Value::Boolean(*b),
        J::Array(a) => toml::Value::Array(a.iter().map(json_to_toml).collect()),
        J::Object(o) => toml::Value::Table(
            o.iter()
                .map(|(k, v)| {
                    // chaves com pontos precisam ser re-quotadas
                    let key = if k.contains('.') {
                        toml::Value::String(k.clone()).to_string()
                    } else {
                        k.clone()
                    };
                    (key, json_to_toml(v))
                })
                .collect(),
        ),
        J::Null => toml::Value::String(String::new()),
    }
}

fn merged_json_to_toml(v: &J) -> anyhow::Result<String> {
    Ok(toml::to_string_pretty(&json_to_toml(v))?)
}

// ── YAML subset ────────────────────────────────────────────────────────────

/// Merge 3-way de YAML subset (mapeamentos por indentação, valores escalares).
/// Linhas de lista (`- item`) são tratadas como blocos opacos: se ambos os
/// lados alterarem o bloco, vira conflito de linha.
pub fn merge_yaml(base: &str, ours: &str, theirs: &str) -> anyhow::Result<MergeResult> {
    let b = parse_yaml_subset(base);
    let o = parse_yaml_subset(ours);
    let t = parse_yaml_subset(theirs);
    let mut conflicts = Vec::new();
    let merged = merge_yaml_maps("", &b, &o, &t, &mut conflicts);
    Ok(MergeResult {
        content: render_yaml(&merged),
        conflicts,
    })
}

#[derive(Debug, Clone, PartialEq)]
enum YamlNode {
    Scalar(String),
    Map(Vec<(String, YamlNode)>),
}

/// Parser de subset: `chave: valor` / `chave:` + bloco indentado / `# comentário`.
fn parse_yaml_subset(src: &str) -> Vec<(String, YamlNode)> {
    let mut lines: Vec<(usize, String)> = Vec::new();
    for raw in src.lines() {
        let no_comment = raw.split('#').next().unwrap_or("").to_string();
        let indent = no_comment.len() - no_comment.trim_start().len();
        let text = no_comment.trim().to_string();
        if text.is_empty() {
            continue;
        }
        lines.push((indent, text));
    }
    let (nodes, _) = parse_yaml_block(&lines, 0, 0);
    nodes
}

fn parse_yaml_block(
    lines: &[(usize, String)],
    mut i: usize,
    min_indent: usize,
) -> (Vec<(String, YamlNode)>, usize) {
    let mut out = Vec::new();
    while i < lines.len() {
        let (indent, text) = &lines[i];
        if *indent < min_indent {
            break;
        }
        if let Some((key, rest)) = text.split_once(':') {
            let key = key.trim().to_string();
            let rest = rest.trim().to_string();
            if rest.is_empty() {
                // bloco aninhado ou vazio
                let (children, next) = parse_yaml_block(lines, i + 1, indent + 1);
                i = next;
                if children.is_empty() {
                    out.push((key, YamlNode::Scalar(String::new())));
                } else {
                    out.push((key, YamlNode::Map(children)));
                }
            } else {
                out.push((key, YamlNode::Scalar(rest)));
                i += 1;
            }
        } else {
            // linha opaca (lista etc.): preserva como scalar com chave sintética
            out.push((format!("__line{i}"), YamlNode::Scalar(text.clone())));
            i += 1;
        }
    }
    (out, i)
}

fn merge_yaml_maps(
    path: &str,
    base: &[(String, YamlNode)],
    ours: &[(String, YamlNode)],
    theirs: &[(String, YamlNode)],
    conflicts: &mut Vec<String>,
) -> Vec<(String, YamlNode)> {
    fn find<'a>(m: &'a [(String, YamlNode)], k: &str) -> Option<&'a YamlNode> {
        m.iter().find(|(key, _)| key == k).map(|(_, v)| v)
    }
    let mut keys: Vec<&String> = base
        .iter()
        .chain(ours.iter())
        .chain(theirs.iter())
        .map(|(k, _)| k)
        .collect();
    keys.sort();
    keys.dedup();
    let mut out = Vec::new();
    for k in keys {
        let p = if path.is_empty() { k.clone() } else { format!("{path}.{k}") };
        match (find(base, k), find(ours, k), find(theirs, k)) {
            (Some(b), Some(o), Some(t)) => match (b, o, t) {
                (YamlNode::Map(bm), YamlNode::Map(om), YamlNode::Map(tm)) => {
                    out.push((k.clone(), YamlNode::Map(merge_yaml_maps(&p, bm, om, tm, conflicts))));
                }
                _ if o == t => out.push((k.clone(), o.clone())),
                _ if b == o => out.push((k.clone(), t.clone())),
                _ if b == t => out.push((k.clone(), o.clone())),
                _ => {
                    conflicts.push(p);
                    out.push((
                        k.clone(),
                        YamlNode::Scalar(format!(
                            "<<<<<<< ours\n{}\n=======\n{}\n>>>>>>> theirs",
                            render_scalar(o),
                            render_scalar(t)
                        )),
                    ));
                }
            },
            // chave nova em ambos (None, Some, Some): conflito se divergirem
            (None, Some(o), Some(t)) => {
                if o == t {
                    out.push((k.clone(), o.clone()));
                } else {
                    conflicts.push(p);
                    out.push((k.clone(), o.clone()));
                }
            }
            // removido por um lado e modificado pelo outro: conflito
            (Some(_), None, Some(t)) => {
                conflicts.push(p);
                out.push((k.clone(), t.clone()));
            }
            (Some(_), Some(o), None) => {
                conflicts.push(p);
                out.push((k.clone(), o.clone()));
            }
            (Some(_), None, None) => {} // removido pelos dois lados
            (_, Some(o), None) => out.push((k.clone(), o.clone())),
            (None, None, Some(t)) => out.push((k.clone(), t.clone())),
            (None, None, None) => {}
        }
    }
    out
}

fn render_scalar(n: &YamlNode) -> String {
    match n {
        YamlNode::Scalar(s) => s.clone(),
        YamlNode::Map(_) => String::new(),
    }
}

fn render_yaml(nodes: &[(String, YamlNode)]) -> String {
    fn rec(nodes: &[(String, YamlNode)], indent: usize, out: &mut String) {
        for (k, v) in nodes {
            let pad = "  ".repeat(indent);
            match v {
                YamlNode::Scalar(s) => {
                    if k.starts_with("__line") {
                        out.push_str(&format!("{pad}{s}\n"));
                    } else {
                        out.push_str(&format!("{pad}{k}: {s}\n"));
                    }
                }
                YamlNode::Map(children) => {
                    out.push_str(&format!("{pad}{k}:\n"));
                    rec(children, indent + 1, out);
                }
            }
        }
    }
    let mut out = String::new();
    rec(nodes, 0, &mut out);
    out
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn json_sem_conflito() {
        let base = r#"{"a": 1, "b": {"x": 1, "y": 2}}"#;
        let ours = r#"{"a": 1, "b": {"x": 10, "y": 2}, "c": 3}"#;
        let theirs = r#"{"a": 2, "b": {"x": 1, "y": 20}}"#;
        let r = merge3(base, ours, theirs, Format::Json).unwrap();
        assert!(r.is_clean(), "conflicts: {:?}", r.conflicts);
        let v: J = serde_json::from_str(&r.content).unwrap();
        assert_eq!(v["a"], 2);
        assert_eq!(v["b"]["x"], 10);
        assert_eq!(v["b"]["y"], 20);
        assert_eq!(v["c"], 3);
    }

    #[test]
    fn json_com_conflito_marca() {
        let base = r#"{"k": 1}"#;
        let ours = r#"{"k": 2}"#;
        let theirs = r#"{"k": 3}"#;
        let r = merge3(base, ours, theirs, Format::Json).unwrap();
        assert_eq!(r.conflicts, vec!["$.k"]);
        assert!(r.content.contains("<<<<<<< ours"));
    }

    #[test]
    fn toml_roundtrip_limpo() {
        let base = "[pkg]\nname = \"a\"\nversion = \"1\"\n";
        let ours = "[pkg]\nname = \"a\"\nversion = \"2\"\n";
        let theirs = "[pkg]\nname = \"b\"\nversion = \"1\"\n";
        let r = merge3(base, ours, theirs, Format::Toml).unwrap();
        assert!(r.is_clean(), "conflicts: {:?}", r.conflicts);
        let v: toml::Value = r.content.parse().unwrap();
        assert_eq!(v["pkg"]["name"].as_str(), Some("b"));
        assert_eq!(v["pkg"]["version"].as_str(), Some("2"));
    }

    #[test]
    fn yaml_subset_merge() {
        let base = "services:\n  web:\n    image: app:1\n    ports: 80\n";
        let ours = "services:\n  web:\n    image: app:2\n    ports: 80\n";
        let theirs = "services:\n  web:\n    image: app:1\n    ports: 8080\n";
        let r = merge3(base, ours, theirs, Format::Yaml).unwrap();
        assert!(r.is_clean(), "conflicts: {:?}", r.conflicts);
        assert!(r.content.contains("image: app:2"));
        assert!(r.content.contains("ports: 8080"));
    }
}
