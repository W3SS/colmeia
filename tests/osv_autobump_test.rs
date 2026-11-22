//! F21/F22/F23 — OSV: ingest com delta, matcher contra SBOM e auto-bump.

use colmeia::autobump;
use colmeia::git::{Git, GitBackend};
use colmeia::osv;
use colmeia::store::Store;
use std::path::Path;
use std::process::Command;

/// Monta um ZIP "stored" (method 0) mínimo, sem deps externas.
fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    let mut offsets = Vec::new();

    for (name, data) in entries {
        offsets.push(out.len() as u32);
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes()); // local sig
        out.extend_from_slice(&20u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0u16.to_le_bytes()); // flags
        out.extend_from_slice(&0u16.to_le_bytes()); // method: stored
        out.extend_from_slice(&0u16.to_le_bytes()); // time
        out.extend_from_slice(&0u16.to_le_bytes()); // date
        out.extend_from_slice(&0u32.to_le_bytes()); // crc (não verificado)
        out.extend_from_slice(&(data.len() as u32).to_le_bytes()); // comp size
        out.extend_from_slice(&(data.len() as u32).to_le_bytes()); // uncomp size
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // extra len
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
    }

    let cd_start = out.len() as u32;
    for ((name, data), off) in entries.iter().zip(offsets.iter()) {
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central.extend_from_slice(&0u16.to_le_bytes()); // flags
        central.extend_from_slice(&0u16.to_le_bytes()); // method
        central.extend_from_slice(&0u16.to_le_bytes()); // time
        central.extend_from_slice(&0u16.to_le_bytes()); // date
        central.extend_from_slice(&0u32.to_le_bytes()); // crc
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(name.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // extra
        central.extend_from_slice(&0u16.to_le_bytes()); // comment
        central.extend_from_slice(&0u16.to_le_bytes()); // disk
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&off.to_le_bytes()); // local header offset
        central.extend_from_slice(name.as_bytes());
    }
    let cd_size = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes()); // EOCD
    out.extend_from_slice(&0u16.to_le_bytes()); // disk
    out.extend_from_slice(&0u16.to_le_bytes()); // cd disk
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_start.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment len
    out
}

fn vuln_json(id: &str, modified: &str, name: &str, fixed: &str) -> String {
    format!(
        r#"{{"id":"{id}","modified":"{modified}","aliases":["CVE-2026-0001"],
        "database_specific":{{"severity":"high"}},
        "affected":[{{"package":{{"name":"{name}","ecosystem":"npm"}},
          "ranges":[{{"type":"ECOSYSTEM","events":[
            {{"introduced":"0"}},{{"fixed":"{fixed}"}}]}}]}}]}}"#
    )
}

#[test]
fn ingest_zip_delta() {
    let store = Store::open_in_memory().unwrap();
    let zip = build_zip(&[(
        "GHSA-aaaa.json",
        vuln_json("GHSA-aaaa", "2026-01-02T00:00:00Z", "lodash", "4.17.21").as_bytes(),
    )]);

    let (added, updated, skipped) = osv::ingest_zip(&store, "npm", &zip).unwrap();
    assert_eq!(added, 1);
    assert_eq!(updated, 0);
    assert_eq!(store.list_vulns().unwrap().len(), 1);

    // Re-ingest do mesmo dump: tudo pulado (delta por modified)
    let (added2, updated2, skipped2) = osv::ingest_zip(&store, "npm", &zip).unwrap();
    assert_eq!((added2, updated2), (0, 0));
    assert!(skipped2 >= 1);

    // Dump mais novo: atualiza
    let zip2 = build_zip(&[(
        "GHSA-aaaa.json",
        vuln_json("GHSA-aaaa", "2026-06-01T00:00:00Z", "lodash", "4.17.22").as_bytes(),
    )]);
    let (_, upd, _) = osv::ingest_zip(&store, "npm", &zip2).unwrap();
    assert_eq!(upd, 1);
}

#[test]
fn matcher_cria_finding_para_versao_afetada() {
    let store = Store::open_in_memory().unwrap();
    let repo = store.add_repo("/tmp/x-osv-match", None, "managed").unwrap();
    store.insert_sbom_entry(repo, "abc", "npm", "lodash", "4.17.20").unwrap();

    let zip = build_zip(&[(
        "GHSA-bbbb.json",
        vuln_json("GHSA-bbbb", "2026-01-02T00:00:00Z", "lodash", "4.17.21").as_bytes(),
    )]);
    osv::ingest_zip(&store, "npm", &zip).unwrap();

    let matches = osv::match_all(&store).unwrap();
    assert_eq!(matches, 1);
    let findings = store.list_findings(repo).unwrap();
    assert!(findings.iter().any(|f| f.0 == "osv.vuln" && f.1 == "high"));

    // Versão corrigida não casa
    store.insert_sbom_entry(repo, "abc", "npm", "lodash", "4.17.21").unwrap();
    let matches2 = osv::match_all(&store).unwrap();
    assert_eq!(matches2, 1, "apenas a versão antiga casa");
}

#[test]
fn autobump_plano_e_aplicacao() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    let run = |args: &[&str]| {
        let out = Command::new("git").current_dir(&repo).args(args).output().unwrap();
        assert!(out.status.success());
    };
    run(&["init", "-b", "main"]);
    run(&["config", "user.email", "t@t.local"]);
    run(&["config", "user.name", "T"]);
    std::fs::write(repo.join("requirements.txt"), "flask==2.0.0\n").unwrap();
    run(&["add", "."]);
    run(&["commit", "-m", "init"]);

    let store = Store::open_in_memory().unwrap();
    let repo_id = store.add_repo(repo.to_str().unwrap(), None, "managed").unwrap();
    store.insert_sbom_entry(repo_id, "x", "PyPI", "flask", "2.0.0").unwrap();
    store.upsert_vuln(
        "GHSA-flask",
        None,
        "PyPI",
        Some("flask"),
        r#"[[{"introduced":"0"},{"fixed":"2.2.5"}]]"#,
        "high",
        "[]",
        1_700_000_000,
        0,
    ).unwrap();

    let actions = autobump::plan(&store, repo_id).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].to, "2.2.5");

    let gitb = Git::new(256);
    // Dry-run não altera o arquivo
    let r = autobump::apply(&store, &gitb, repo_id, &actions[0], true).unwrap();
    assert!(r.contains("dry-run"));
    assert!(std::fs::read_to_string(repo.join("requirements.txt")).unwrap().contains("2.0.0"));

    // Aplicação real: arquivo muda e branch é criada
    autobump::apply(&store, &gitb, repo_id, &actions[0], false).unwrap();
    assert!(std::fs::read_to_string(repo.join("requirements.txt")).unwrap().contains("flask==2.2.5"));
    let branch = Command::new("git")
        .current_dir(&repo)
        .args(["branch", "--show-current"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&branch.stdout).contains("colmeia/bump-flask-2.2.5"));
}

// mantém o import de Path usado
#[allow(unused)]
fn _p(x: &Path) {}
