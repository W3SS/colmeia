use crate::{
    config::Config,
    git::{Git, GitBackend},
    rules,
    store::Store,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn blob_sha(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn run_repo(
    store: &Store,
    git: &Git,
    _cfg: &Config,
    repo_id: i64,
    payload: &Value,
) -> anyhow::Result<()> {
    let p = store.get_repo_path(repo_id)?;
    let range = payload["range"].as_str().unwrap_or("HEAD~1..HEAD");
    let path = Path::new(&p);

    let files = git.changed_files(path, range)?;
    let mut total_findings = 0;

    for rel_path in files {
        let full_path = path.join(&rel_path);
        if !full_path.is_file() {
            continue;
        }

        let text = match std::fs::read_to_string(&full_path) {
            Ok(t) => t,
            Err(_) => continue, // Arquivo binário ou ilegível
        };

        // Pula arquivos gigantes (> 1 MB) para preservar orçamento de memória no tier T2
        if text.len() > 1_000_000 {
            continue;
        }

        let blob = blob_sha(&text);
        let findings = rules::scan(&rel_path, &text);

        for fd in findings {
            store.upsert_finding(
                repo_id,
                &blob,
                fd.rule,
                fd.sev,
                &rel_path,
                fd.line as i64,
                &fd.msg,
                fd.owasp,
            )?;
            total_findings += 1;
        }
    }

    store.event(
        repo_id,
        "gate.done",
        &serde_json::json!({ "findings": total_findings }),
    )?;
    Ok(())
}

pub fn sarif(findings: &[(String, String, String, i64, String)]) -> Value {
    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "colmeia-gate",
                    "version": "0.1.0",
                    "informationUri": "https://github.com/colmeia-app/colmeia"
                }
            },
            "results": findings.iter().map(|(rule, sev, path, line, msg)| serde_json::json!({
                "ruleId": rule,
                "level": match sev.as_str() {
                    "critical" | "high" => "error",
                    "medium" => "warning",
                    _ => "note",
                },
                "message": { "text": msg },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": path },
                        "region": { "startLine": line }
                    }
                }]
            })).collect::<Vec<_>>()
        }]
    })
}
