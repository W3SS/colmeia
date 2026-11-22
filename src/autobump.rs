//! F23 — Auto-bump de dependências com correção disponível.
//!
//! `plan` cruza o SBOM do repo contra as vulns com evento `fixed` e devolve
//! ações de bump (eco, nome, de, para). `apply` edita requirements.txt
//! (PyPI) ou package-lock.json (npm), cria branch `colmeia/bump-<pkg>` e
//! commita de forma determinística. Dry-run por padrão: nada é escrito
//! sem `dry_run: false`.

use crate::git::{Git, GitBackend};
use crate::store::Store;
use crate::vulnmatch;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BumpAction {
    pub ecosystem: String,
    pub name: String,
    pub from: String,
    pub to: String,
    pub vuln_id: String,
}

/// Plano de bumps para um repo a partir das vulns ingeridas.
pub fn plan(store: &Store, repo_id: i64) -> anyhow::Result<Vec<BumpAction>> {
    let sbom: Vec<_> = store
        .list_sbom()?
        .into_iter()
        .filter(|(r, _, _, _, _)| *r == repo_id)
        .collect();
    let vulns = store.list_vulns()?;
    let mut actions = Vec::new();

    for (vid, _aliases, veco, ranges_json, _sev, _fixed, _modified, withdrawn) in &vulns {
        if *withdrawn > 0 {
            continue;
        }
        let Ok(ranges) = serde_json::from_str::<serde_json::Value>(ranges_json) else {
            continue;
        };
        for (_r, _c, eco, name, version) in &sbom {
            if eco != veco {
                continue;
            }
            let Some(arr) = ranges.as_array() else { continue };
            for evs in arr {
                let events: Vec<(String, String)> = evs
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .filter_map(|e| {
                        e.as_object().and_then(|o| {
                            o.iter()
                                .next()
                                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                        })
                    })
                    .collect();
                let mut fixed_at: Option<String> = None;
                let mut introduced_at = "0".to_string();
                for (k, v) in &events {
                    match k.as_str() {
                        "fixed" => fixed_at = Some(v.clone()),
                        "introduced" => introduced_at = v.clone(),
                        _ => {}
                    }
                }
                let Some(fix) = fixed_at else { continue };
                // versão instalada está no range afetado?
                let bounds = vec![
                    vulnmatch::Bound::Introduced(introduced_at),
                    vulnmatch::Bound::Fixed(fix.clone()),
                ];
                if vulnmatch::in_range(version, &bounds) {
                    actions.push(BumpAction {
                        ecosystem: eco.clone(),
                        name: name.clone(),
                        from: version.clone(),
                        to: fix,
                        vuln_id: vid.clone(),
                    });
                }
            }
        }
    }
    actions.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.to.cmp(&b.to)));
    actions.dedup_by(|a, b| {
        if a.name == b.name && a.from == b.from {
            // mesma ação por múltiplas vulns: une ids
            b.vuln_id.push_str(&format!(",{}", a.vuln_id));
            true
        } else {
            false
        }
    });
    Ok(actions)
}

/// Aplica um bump no repo. Dry-run só reporta (e valida que a versão existe).
pub fn apply(
    store: &Store,
    git: &Git,
    repo_id: i64,
    action: &BumpAction,
    dry_run: bool,
) -> anyhow::Result<String> {
    let repo_path = store.get_repo_path(repo_id)?;
    let root = Path::new(&repo_path);

    let changed_file = match action.ecosystem.as_str() {
        "PyPI" => {
            let f = root.join("requirements.txt");
            if !f.is_file() {
                anyhow::bail!("requirements.txt não existe em {repo_path}");
            }
            let content = std::fs::read_to_string(&f)?;
            let needle = format!("{}=={}", action.name, action.from);
            anyhow::ensure!(
                content.contains(&needle),
                "versão esperada {needle} não encontrada em requirements.txt"
            );
            if !dry_run {
                std::fs::write(&f, content.replace(&needle, &format!("{}=={}", action.name, action.to)))?;
            }
            "requirements.txt"
        }
        "npm" => {
            let f = root.join("package-lock.json");
            if !f.is_file() {
                anyhow::bail!("package-lock.json não existe em {repo_path}");
            }
            let content = std::fs::read_to_string(&f)?;
            let marker = format!("\"node_modules/{}\"", action.name);
            anyhow::ensure!(
                content.contains(&marker),
                "pacote {marker} não encontrado em package-lock.json"
            );
            // Substitui a primeira "version" após o marcador do pacote
            let mut lines: Vec<String> = content.lines().map(String::from).collect();
            let mut in_pkg = false;
            let mut swapped = false;
            for line in lines.iter_mut() {
                if line.contains(&marker) {
                    in_pkg = true;
                } else if in_pkg && line.contains("\"version\"") {
                    *line = format!("      \"version\": \"{}\",", action.to);
                    in_pkg = false;
                    swapped = true;
                    break;
                }
            }
            anyhow::ensure!(swapped, "campo version de {marker} não localizado");
            let _ = in_pkg; // pode ter sido setado na última linha do arquivo
            if !dry_run {
                std::fs::write(&f, lines.join("\n"))?;
            }
            "package-lock.json"
        }
        other => anyhow::bail!("ecossistema {other} sem bump automático ainda"),
    };

    if dry_run {
        store.event(
            repo_id,
            "autobump.dry_run",
            &serde_json::json!({
                "pkg": action.name, "from": action.from, "to": action.to,
                "vuln": action.vuln_id, "file": changed_file
            }),
        )?;
        return Ok(format!("dry-run: {} {} → {}", action.name, action.from, action.to));
    }

    // Branch determinístico + commit
    let branch = format!("colmeia/bump-{}-{}", action.name, action.to);
    let _ = git.run(root, &["checkout", "-B", &branch]);
    git.run(root, &["add", changed_file])?;
    let out = git.run_with_env(
        root,
        &[
            "commit",
            "--no-edit",
            "-m",
            &format!(
                "colmeia: bump {} {} → {} ({})",
                action.name, action.from, action.to, action.vuln_id
            ),
        ],
        &[("GIT_MERGE_AUTOEDIT", "no")],
    )?;
    anyhow::ensure!(
        out.status.success(),
        "commit do bump falhou: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let sha = git.head(root)?;
    store.event(
        repo_id,
        "autobump.applied",
        &serde_json::json!({
            "pkg": action.name, "from": action.from, "to": action.to,
            "branch": branch, "sha": sha
        }),
    )?;
    Ok(format!("{} {} → {} em {branch} ({sha:.7})", action.name, action.from, action.to))
}
