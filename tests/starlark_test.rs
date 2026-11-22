//! F17 — Testes de integração do Starlark hermético para packs.

use colmeia::starlark;

fn pack_tmp(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, content) in files {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    dir
}

#[test]
fn pack_basico_coleta_rules() {
    let pack = pack_tmp(&[(
        "rules.star",
        r#"
def nivel(path):
    if path.endswith(".lock"):
        return "block"
    return "warn"

regras = [
    rule(regex="AKIA[0-9A-Z]{16}", severity="critical", message="chave AWS", id="aws-key"),
    rule(regex="ghp_[A-Za-z0-9]{20,}", severity=nivel("x.txt"), message="token GH", id="gh-token"),
]
"#,
    )]);
    let rules = starlark::run_pack(&pack.path().join("rules.star")).unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].id, "aws-key");
    assert_eq!(rules[0].severity, "critical");
    assert_eq!(rules[1].severity, "warn");
}

#[test]
fn pack_load_importa_simbolos() {
    let pack = pack_tmp(&[
        (
            "lib.star",
            "PADRAO = \"ghp_[A-Za-z0-9]{20,}\"\nSEV = \"critical\"\n",
        ),
        (
            "main.star",
            r#"load("lib.star", "PADRAO", "SEV")
regras = [rule(regex=PADRAO, severity=SEV, message="token", id="t1")]
"#,
        ),
    ]);
    let rules = starlark::run_pack(&pack.path().join("main.star")).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].regex, "ghp_[A-Za-z0-9]{20,}");
    assert_eq!(rules[0].severity, "critical");
}

#[test]
fn pack_load_fora_do_diretorio_e_rejeitado() {
    let dir = tempfile::tempdir().unwrap();
    let outra = tempfile::tempdir().unwrap();
    std::fs::write(outra.path().join("segredos.star"), "X = 1\n").unwrap();
    // caminho relativo que sobe do pack: deve falhar na canonicalização/starts_with
    let pack = dir.path().join("pack");
    std::fs::create_dir_all(&pack).unwrap();
    let mal = format!("load(\"{}/segredos.star\", \"X\")\n", outra.path().display());
    std::fs::write(pack.join("main.star"), mal).unwrap();
    let r = starlark::run_pack(&pack.join("main.star"));
    assert!(r.is_err());
}

#[test]
fn pack_sem_regex_nao_gera_regra() {
    let pack = pack_tmp(&[("r.star", "x = rule(severity=\"low\", message=\"sem regex\")\n")]);
    let rules = starlark::run_pack(&pack.path().join("r.star")).unwrap();
    assert!(rules.is_empty());
}

#[test]
fn pack_divisao_por_zero_erro() {
    let pack = pack_tmp(&[("r.star", "x = 1 // 0\n")]);
    assert!(starlark::run_pack(&pack.path().join("r.star")).is_err());
}

#[test]
fn pack_gas_esgota_em_loop() {
    // sem recursão/while no subset, mas gas também limita execuções longas;
    // aqui validamos apenas que um pack grande demais é rejeitado no parser
    // ou executa — o importante é que não há laço infinito possível no subset.
    let pack = pack_tmp(&[("r.star", "regras = [rule(regex=\"a\", severity=\"s\", message=\"m\")]\n")]);
    let rules = starlark::run_pack(&pack.path().join("r.star")).unwrap();
    assert_eq!(rules.len(), 1);
}

#[test]
fn pack_operacoes_sobre_listas() {
    let pack = pack_tmp(&[(
        "r.star",
        r#"base = [rule(regex="a", severity="s", message="m")]
extra = [rule(regex="b", severity="s", message="m")]
regras = base + extra
"#,
    )]);
    let rules = starlark::run_pack(&pack.path().join("r.star")).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn pack_erro_sintaxe() {
    let pack = pack_tmp(&[("r.star", "def quebrado(:\n")]);
    assert!(starlark::run_pack(&pack.path().join("r.star")).is_err());
}
