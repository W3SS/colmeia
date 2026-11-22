//! F52 — Fuzz: os parsers nunca devem panificar com entrada arbitrária.

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn sbom_parse_nao_panica(cargo_lock in "[[:print:][:space:]]{0,2048}",
                             reqs in "[[:print:][:space:]]{0,1024}",
                             lock in "[[:print:][:space:]]{0,2048}") {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), cargo_lock).unwrap();
        std::fs::write(dir.path().join("requirements.txt"), reqs).unwrap();
        std::fs::write(dir.path().join("package-lock.json"), lock).unwrap();
        let _ = colmeia::sbom::parse(dir.path().to_str().unwrap());
    }

    #[test]
    fn rules_scan_nao_panica(path in "[a-zA-Z0-9_./-]{0,80}", text in "[[:print:][:space:]\u{0}-\u{ff}]{0,1024}") {
        let _ = colmeia::rules::scan(&path, &text);
    }

    #[test]
    fn redact_nao_panica_e_idempotente(text in "[[:print:][:space:]]{0,1024}") {
        let once = colmeia::redact::redact(&text);
        prop_assert_eq!(colmeia::redact::redact(&once), once);
    }

    #[test]
    fn path_sync_glob_nao_panica(pat in "[a-zA-Z0-9*/?_.-]{0,60}",
                                 path in "[a-zA-Z0-9_./-]{0,120}") {
        let _ = colmeia::path_sync::glob_match(&pat, &path);
    }
}
