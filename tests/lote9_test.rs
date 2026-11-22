//! Lote 9 — Testes: wizard (F47), self-heal (F48), packaging (F49) e guias (F50).

use colmeia::config::Config;
use colmeia::store::Store;

#[test]
fn wizard_parse_profile_escolhas_validas() {
    use colmeia::doctor::parse_profile_choice;
    assert_eq!(parse_profile_choice("1"), Some("minimum"));
    assert_eq!(parse_profile_choice("rec"), Some("recommended"));
    assert_eq!(parse_profile_choice("MAX"), Some("maximum"));
    assert_eq!(parse_profile_choice(""), None);
    assert_eq!(parse_profile_choice("9"), None);
}

#[test]
fn apply_profile_ajusta_limites() {
    let mut cfg = Config::default();
    Config::apply_profile(&mut cfg, "minimum");
    assert_eq!(cfg.workers, 1);
    assert_eq!(cfg.memory_hard_mb, 96);
    Config::apply_profile(&mut cfg, "maximum");
    assert_eq!(cfg.workers, 4);
    assert_eq!(cfg.memory_hard_mb, 512);
    // perfil desconhecido não altera
    let before = cfg.clone();
    Config::apply_profile(&mut cfg, "nope");
    assert_eq!(cfg.workers, before.workers);
}

#[test]
fn self_heal_repara_diretorio_e_schema() {
    // Nota: data-dir/config-file apontam para ~/.colmeia; em ambiente de
    // teste eles já existem ou são criáveis — o importante é o relatório
    // coerente e o banco íntegro.
    let store = Store::open_in_memory().unwrap();
    let cfg = Config::default();
    let actions = colmeia::doctor::self_heal(&store, &cfg);
    assert!(actions.iter().any(|a| a.check == "data-dir" && a.healed));
    assert!(actions.iter().any(|a| a.check == "db-schema" && a.healed));
    assert!(actions.iter().any(|a| a.check == "git-binary" && a.healed));
    assert!(actions.iter().all(|a| !a.action.is_empty()));
}

#[test]
fn firmware_contract_valido() {
    let c = colmeia::firmware::contract_t0();
    assert_eq!(c["protocol"], "colmeia-t0");
    assert!(c["tiers"]["minimum"]["memory_hard_mb"].is_number());
}

#[test]
fn packaging_homebrew_referencia_binario() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let formula = std::fs::read_to_string(root.join("packaging/homebrew/colmeia.rb")).unwrap();
    assert!(formula.contains("class Colmeia"));
    assert!(formula.contains("std_cargo_args"));
    assert!(formula.contains("test do"));
}

#[test]
fn packaging_debian_script_sintaxe_valida() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = root.join("packaging/debian/build.sh");
    let out = std::process::Command::new("sh")
        .args(["-n", script.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "sh -n falhou: {:?}", out);
    let content = std::fs::read_to_string(&script).unwrap();
    assert!(content.contains("dpkg-deb"));
    assert!(content.contains("Package: colmeia"));
}

#[test]
fn getting_started_pt_e_en_existem_e_alinham() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let pt = std::fs::read_to_string(root.join("docs/getting-started.pt.md")).unwrap();
    let en = std::fs::read_to_string(root.join("docs/getting-started.en.md")).unwrap();
    for needle in ["colmeia init", "colmeia tui", "colmeia doctor --heal", "COLMEIA_OTLP_ENDPOINT"] {
        assert!(pt.contains(needle), "pt sem '{needle}'");
        assert!(en.contains(needle), "en sem '{needle}'");
    }
}
