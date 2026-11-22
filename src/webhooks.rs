//! F37 — Verificação de assinatura de webhooks (HMAC-SHA256).
//!
//! Espera o header no formato `sha256=<hex>` (padrão GitHub/Forgejo) e
//! compara em tempo constante. O segredo vem do vault (F51) ou de env.

use crate::secretbox::hmac_sha256;

/// Calcula a assinatura esperada para um corpo (para testes e emissão).
pub fn sign(secret: &[u8], body: &[u8]) -> String {
    let mac = hmac_sha256(secret, body);
    format!("sha256={}", hex::encode(mac))
}

/// Verifica o header de assinatura contra o corpo recebido.
pub fn verify(secret: &[u8], body: &[u8], signature_header: &str) -> bool {
    let expected = sign(secret, body);
    let a = expected.as_bytes();
    let b = signature_header.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    // comparação em tempo constante
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn assinatura_valida() {
        let body = b"{\"event\":\"push\"}";
        let sig = sign(b"segredo", body);
        assert!(verify(b"segredo", body, &sig));
    }

    #[test]
    fn segredo_errado_rejeita() {
        let body = b"payload";
        let sig = sign(b"certo", body);
        assert!(!verify(b"errado", body, &sig));
    }

    #[test]
    fn corpo_adulterado_rejeita() {
        let sig = sign(b"s", b"abc");
        assert!(!verify(b"s", b"abd", &sig));
    }

    #[test]
    fn formato_invalido_rejeita() {
        assert!(!verify(b"s", b"abc", "not-hex"));
        assert!(!verify(b"s", b"abc", ""));
    }
}
