//! F21 — OSV sync com delta: baixa all.zip do dump do ecossistema,
//! descompacta (zipmini), e faz upsert apenas de entradas modificadas.
//! F22 — matcher: cruza o SBOM dos repos contra os ranges de versões.

use crate::{store::Store, vulnmatch, zipmini};
use crate::config::Config;
use std::io::Read;

/// Ingesta bytes de um all.zip: retorna (novas, atualizadas, puladas).
/// Delta por `modified` — critério MVP: sync nightly ≤ 60 s.
pub fn ingest_zip(store: &Store, ecosystem: &str, bytes: &[u8]) -> anyhow::Result<(usize, usize, usize)> {
    let entries = zipmini::extract(bytes)?;
    let mut added = 0;
    let mut updated = 0;
    let mut skipped = 0;

    for entry in entries {
        if !entry.name.ends_with(".json") {
            skipped += 1;
            continue;
        }
        let Ok(v) = serde_json::from_slice::<serde_json::Value>(&entry.data) else {
            skipped += 1;
            continue;
        };
        let id = match v["id"].as_str() {
            Some(s) => s.to_string(),
            None => {
                skipped += 1;
                continue;
            }
        };
        let modified = v["modified"].as_str().and_then(parse_osv_ts).unwrap_or(0);
        let withdrawn = v["withdrawn"].as_str().and_then(parse_osv_ts).unwrap_or(0);

        // Delta: já temos essa versão?
        if let Some(stored) = store.vuln_modified(&id)? {
            if stored >= modified {
                skipped += 1;
                continue;
            }
            updated += 1;
        } else {
            added += 1;
        }

        let aliases = v["aliases"].as_array().map(|a| {
            serde_json::to_string(&a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
                .unwrap_or_default()
        });
        let severity = v["database_specific"]["severity"]
            .as_str()
            .or_else(|| {
                v["severity"]
                    .as_array()
                    .and_then(|s| s.first())
                    .and_then(|s| s["score"].as_str())
            })
            .unwrap_or("medium")
            .to_string();
        let (name, ranges, fixed) = extract_affected(&v);

        store.upsert_vuln(
            &id,
            aliases.as_deref(),
            ecosystem,
            name.as_deref(),
            &ranges,
            &severity,
            &fixed,
            modified,
            withdrawn,
        )?;
    }

    Ok((added, updated, skipped))
}

/// Extrai nome do pacote, ranges serializados e versões corrigidas.
fn extract_affected(v: &serde_json::Value) -> (Option<String>, String, String) {
    let mut name = None;
    let mut ranges_out: Vec<serde_json::Value> = Vec::new();
    let mut fixed: Vec<String> = Vec::new();

    if let Some(affected) = v["affected"].as_array() {
        for a in affected {
            if name.is_none() {
                name = a["package"]["name"].as_str().map(String::from);
            }
            if let Some(rs) = a["ranges"].as_array() {
                for r in rs {
                    if r["type"].as_str() == Some("ECOSYSTEM")
                        || r["type"].as_str() == Some("SEMVER")
                    {
                        ranges_out.push(r["events"].clone());
                    }
                }
            }
            if let Some(versions) = a["versions"].as_array() {
                for ver in versions.iter().filter_map(|x| x.as_str()) {
                    fixed.push(ver.to_string());
                }
            }
        }
    }
    (
        name,
        serde_json::to_string(&ranges_out).unwrap_or_default(),
        serde_json::to_string(&fixed).unwrap_or_default(),
    )
}

/// "2024-05-01T14:30:00Z" → epoch secs (aproximação por dia é suficiente pro delta).
fn parse_osv_ts(s: &str) -> Option<i64> {
    let date = s.get(..10)?;
    let mut it = date.split('-');
    let y: i64 = it.next()?.parse().ok()?;
    let m: i64 = it.next()?.parse().ok()?;
    let d: i64 = it.next()?.parse().ok()?;
    // Dias desde epoch (fórmula civil)
    let days = (y - 1970) * 365 + (y - 1969) / 4 - (y - 1901) / 100 + (y - 1601) / 400
        + [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334][(m - 1).clamp(0, 11) as usize]
        + d
        - 1
        + if m > 2 && (y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)) { 1 } else { 0 };
    Some(days * 86_400)
}

/// F21 — sync completo: baixa os dumps configurados e ingere com delta.
pub fn sync(store: &Store, cfg: &Config) -> anyhow::Result<()> {
    for eco in &cfg.security.osv_ecosystems {
        let url = format!(
            "https://storage.googleapis.com/osv-vulnerabilities/{}/all.zip",
            eco
        );

        match ureq::get(&url).call() {
            Ok(resp) => {
                let mut bytes = Vec::new();
                resp.into_reader()
                    .take(200_000_000)
                    .read_to_end(&mut bytes)?;

                let (added, updated, skipped) = match ingest_zip(store, eco, &bytes) {
                    Ok(r) => r,
                    Err(e) => {
                        store.event(
                            0,
                            "osv.sync.error",
                            &serde_json::json!({ "ecosystem": eco, "error": e.to_string() }),
                        )?;
                        continue;
                    }
                };

                store.event(
                    0,
                    "osv.sync",
                    &serde_json::json!({
                        "ecosystem": eco,
                        "bytes": bytes.len(),
                        "added": added,
                        "updated": updated,
                        "skipped": skipped,
                        "status": "ingested"
                    }),
                )?;
            }
            Err(e) => {
                store.event(
                    0,
                    "osv.sync.error",
                    &serde_json::json!({ "ecosystem": eco, "error": e.to_string() }),
                )?;
            }
        }
    }
    Ok(())
}

/// F22 — cruza o SBOM de todos os repos contra as vulns do ecossistema.
/// Cada casamento vira um finding `osv.vuln` (integra com o gate/SARIF).
pub fn match_all(store: &Store) -> anyhow::Result<usize> {
    let vulns = store.list_vulns()?;
    if vulns.is_empty() {
        return Ok(0);
    }
    let sbom = store.list_sbom()?;

    // Índice: ecosystem+name → ranges/fixed
    let mut matches = 0;
    for (repo_id, _commit, eco, name, version) in &sbom {
        for (vid, _aliases, veco, ranges_json, severity, fixed_json, _modified, withdrawn) in &vulns
        {
            if *withdrawn > 0 || veco != eco {
                continue;
            }
            // nome bate? (OSV guarda o nome no campo name extraído)
            let vuln_name = {
                let c = store.conn.lock().unwrap();
                c.query_row(
                    "SELECT name FROM vulns WHERE id=?1",
                    [vid],
                    |r| r.get::<_, Option<String>>(0),
                )
                .ok()
                .flatten()
            };
            let Some(vname) = vuln_name else { continue };
            if !name_eq(vname, name) {
                continue;
            }

            // Versão afetada?
            let affected = {
                let Ok(ranges) = serde_json::from_str::<serde_json::Value>(ranges_json) else {
                    continue;
                };
                let mut hit = false;
                if let Some(arr) = ranges.as_array() {
                    for evs in arr {
                        let events: Vec<vulnmatch::Bound> = evs
                            .as_array()
                            .unwrap_or(&Vec::new())
                            .iter()
                            .filter_map(|e| {
                                if let Some(v) = e["introduced"].as_str() {
                                    Some(vulnmatch::Bound::Introduced(v.into()))
                                } else if let Some(v) = e["fixed"].as_str() {
                                    Some(vulnmatch::Bound::Fixed(v.into()))
                                } else if let Some(v) = e["last_affected"].as_str() {
                                    Some(vulnmatch::Bound::LastAffected(v.into()))
                                } else {
                                    e["limit"].as_str().map(|v| vulnmatch::Bound::Limit(v.into()))
                                }
                            })
                            .collect();
                        if vulnmatch::in_range(version, &events) {
                            hit = true;
                            break;
                        }
                    }
                }
                if !hit {
                    // fallback: lista explícita de versões
                    if let Ok(vers) = serde_json::from_str::<Vec<String>>(fixed_json) {
                        hit = vers.iter().any(|v| v == version);
                    }
                }
                hit
            };

            if affected {
                let blob = crate::gate::blob_sha(&format!("{vid}:{name}:{version}"));
                store.upsert_finding(
                    *repo_id,
                    &blob,
                    "osv.vuln",
                    severity,
                    &format!("{eco}:{name}"),
                    0,
                    &format!("{vid} afeta {name}@{version}"),
                    "OSV",
                )?;
                matches += 1;
            }
        }
    }
    Ok(matches)
}

fn name_eq(osv_name: String, sbom_name: &str) -> bool {
    let a = osv_name.trim().to_lowercase();
    let b = sbom_name.trim().to_lowercase();
    a == b
}
