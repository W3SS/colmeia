//! F10 — path_sync de skill-packs.
//!
//! Copia arquivos de um repo de origem para um destino conforme um glob,
//! com hash de conteúdo (só copia o que mudou) e commit determinístico no
//! destino. Sem conflito: o sync sempre produz o estado exato da origem
//! para o conjunto coberto pelo glob.

use crate::git::Git;
use crate::store::Store;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Glob mínimo: `*` (sem `/`), `?`, `**` (qualquer profundidade) e literais.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    fn seg_match(p: &[char], s: &[char]) -> bool {
        // casamento de um segmento: * e ?
        if p.is_empty() {
            return s.is_empty();
        }
        match p[0] {
            '*' => seg_match(&p[1..], s) || (!s.is_empty() && seg_match(p, &s[1..])),
            '?' => !s.is_empty() && seg_match(&p[1..], &s[1..]),
            c => !s.is_empty() && s[0] == c && seg_match(&p[1..], &s[1..]),
        }
    }
    let pats: Vec<&str> = pattern.split('/').collect();
    let segs: Vec<&str> = path.split('/').collect();
    fn rec(pats: &[&str], segs: &[&str]) -> bool {
        if pats.is_empty() {
            return segs.is_empty();
        }
        if pats[0] == "**" {
            for skip in 0..=segs.len() {
                if rec(&pats[1..], &segs[skip..]) {
                    return true;
                }
            }
            return false;
        }
        !segs.is_empty() && seg_match(&pats[0].chars().collect::<Vec<_>>(), &segs[0].chars().collect::<Vec<_>>()) && rec(&pats[1..], &segs[1..])
    }
    rec(&pats, &segs)
}

fn collect_matches(root: &Path, pattern: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name == ".git" {
                    continue;
                }
                let Ok(meta) = entry.metadata() else { continue };
                if meta.is_dir() {
                    stack.push(path);
                } else {
                    let rel = path
                        .strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/");
                    if glob_match(pattern, &rel) {
                        out.push(path);
                    }
                }
            }
        }
    }
    out.sort();
    out
}

fn sha256_file(path: &Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path)?;
    let mut h = Sha256::new();
    h.update(&bytes);
    Ok(hex::encode(h.finalize()))
}

pub fn path_sync(store: &Store, git: &Git, repo_id: i64, payload: &Value) -> anyhow::Result<()> {
    let src = store.get_repo_path(repo_id)?;
    let dst_id = payload["dst"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("path_sync exige 'dst' (id do repo destino)"))?;
    let pattern = payload["glob"].as_str().unwrap_or("**");
    let dst = store.get_repo_path(dst_id)?;

    let src_root = Path::new(&src);
    let dst_root = Path::new(&dst);
    let files = collect_matches(src_root, pattern);

    let mut copied = 0usize;
    let mut content_hashes = Vec::new();
    for file in &files {
        let rel = file.strip_prefix(src_root).unwrap().to_path_buf();
        let target = dst_root.join(&rel);
        let same = match sha256_file(&target) {
            Ok(dst_hash) => dst_hash == sha256_file(file)?,
            Err(_) => false,
        };
        if same {
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(file, &target)?;
        copied += 1;
        content_hashes.push(format!("{}:{}", rel.to_string_lossy(), sha256_file(file)?));
    }

    if copied > 0 {
        // Commit determinístico no destino
        git.run(dst_root, &["add", "-A"])?;
        let out = git.run_with_env(
            dst_root,
            &[
                "commit",
                "--no-edit",
                "-m",
                &format!(
                    "colmeia: path_sync {} ({} arquivos, hash {})",
                    pattern,
                    copied,
                    crate::gate::blob_sha(&content_hashes.join("\n"))[..12].to_string()
                ),
            ],
            &[("GIT_MERGE_AUTOEDIT", "no")],
        )?;
        if !out.status.success() {
            // Sem mudanças staged (conteúdo idêntico) — ok
            let stderr = String::from_utf8_lossy(&out.stderr);
            anyhow::ensure!(
                stderr.contains("nothing to commit") || stderr.contains("nothing added"),
                "commit path_sync falhou: {stderr}"
            );
        }
    }

    store.event(
        repo_id,
        "path_sync.done",
        &serde_json::json!({
            "dst": dst_id,
            "glob": pattern,
            "files": files.len(),
            "copied": copied,
            "content_hash": crate::gate::blob_sha(&content_hashes.join("\n")),
        }),
    )?;
    Ok(())
}
