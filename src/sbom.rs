use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDep {
    pub ecosystem: String,
    pub name: String,
    pub version: String,
}

pub fn parse(repo_path: &str) -> Vec<PackageDep> {
    let mut out = Vec::new();
    let p = Path::new(repo_path);

    // 1. Parser Rust Cargo.lock
    if let Ok(content) = std::fs::read_to_string(p.join("Cargo.lock")) {
        for block in content.split("[[package]]").skip(1) {
            let name = extract_toml_field(block, "name");
            let version = extract_toml_field(block, "version");
            if !name.is_empty() && !version.is_empty() {
                out.push(PackageDep {
                    ecosystem: "crates.io".into(),
                    name,
                    version,
                });
            }
        }
    }

    // 2. Parser Python requirements.txt
    if let Ok(content) = std::fs::read_to_string(p.join("requirements.txt")) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((name, ver)) = trimmed.split_once("==") {
                out.push(PackageDep {
                    ecosystem: "PyPI".into(),
                    name: name.trim().into(),
                    version: ver.trim().into(),
                });
            }
        }
    }

    // 3. Parser JavaScript/Node package-lock.json
    if let Ok(content) = std::fs::read_to_string(p.join("package-lock.json")) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(packages) = val.get("packages").and_then(|v| v.as_object()) {
                for (key, meta) in packages {
                    let clean_name = key.trim_start_matches("node_modules/");
                    if clean_name.is_empty() {
                        continue;
                    }
                    if let Some(ver) = meta.get("version").and_then(|v| v.as_str()) {
                        out.push(PackageDep {
                            ecosystem: "npm".into(),
                            name: clean_name.into(),
                            version: ver.into(),
                        });
                    }
                }
            }
        }
    }

    out
}

fn extract_toml_field(block: &str, field_name: &str) -> String {
    let prefix = format!("{} =", field_name);
    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&prefix) {
            if let Some(val) = trimmed.split('"').nth(1) {
                return val.to_string();
            }
        }
    }
    String::new()
}
