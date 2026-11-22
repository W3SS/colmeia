//! F51 — Testes de integração do vault de segredos.

use colmeia::{secretbox, store::Store};

#[test]
fn vault_roundtrip() {
    let store = Store::open_in_memory().unwrap();
    secretbox::vault_set(&store, "github-token", "ghp_valor-super-secreto", "senha-correta")
        .unwrap();

    let got = secretbox::vault_get(&store, "github-token", "senha-correta").unwrap();
    assert_eq!(got.as_deref(), Some("ghp_valor-super-secreto"));
}

#[test]
fn vault_passphrase_errada_falha() {
    let store = Store::open_in_memory().unwrap();
    secretbox::vault_set(&store, "k", "v", "certa").unwrap();

    let err = secretbox::vault_get(&store, "k", "errada").unwrap_err();
    assert!(err.to_string().contains("autenticação"), "erro: {err}");
}

#[test]
fn vault_nonce_aleatorio_cada_gravacao() {
    let store = Store::open_in_memory().unwrap();
    secretbox::vault_set(&store, "k", "mesmo-valor", "p").unwrap();
    let (ct1, n1) = store.secret_get("k").unwrap().unwrap();
    secretbox::vault_set(&store, "k", "mesmo-valor", "p").unwrap();
    let (ct2, n2) = store.secret_get("k").unwrap().unwrap();

    assert_ne!(ct1, ct2, "mesmo valor cifrado deve produzir ciphertexts distintos");
    assert_ne!(n1, n2, "nonces devem ser únicos por gravação");
}

#[test]
fn vault_segredo_inexistente_retorna_none() {
    let store = Store::open_in_memory().unwrap();
    assert!(secretbox::vault_get(&store, "nao-existe", "p")
        .unwrap()
        .is_none());
}

#[test]
fn vault_overwrite() {
    let store = Store::open_in_memory().unwrap();
    secretbox::vault_set(&store, "k", "v1", "p").unwrap();
    secretbox::vault_set(&store, "k", "v2", "p").unwrap();
    assert_eq!(
        secretbox::vault_get(&store, "k", "p").unwrap().as_deref(),
        Some("v2")
    );
}

#[test]
fn vault_nao_persiste_plaintext() {
    let store = Store::open_in_memory().unwrap();
    secretbox::vault_set(&store, "api", "valor-em-claro-123", "p").unwrap();
    let (ct, _) = store.secret_get("api").unwrap().unwrap();
    assert!(
        !ct.contains("valor-em-claro"),
        "ciphertext em hex não pode conter o valor em claro"
    );
}
