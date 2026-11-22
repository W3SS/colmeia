//! F51 — Secretbox: segredos nunca em plaintext.
//!
//! ChaCha20-Poly1305 (RFC 8439) + PBKDF2-HMAC-SHA256 implementados sobre
//! `sha2` (já na árvore) — zero deps novas, contrato de leveza preservado.
//! O cofre é aberto por passphrase lida do terminal (ou env em testes);
//! a chave mestra nunca é persistida, só o salt do PBKDF2.

use sha2::{Digest, Sha256};

// ── HMAC-SHA256 (RFC 2104) ─────────────────────────────────────────────────

/// HMAC-SHA256 (RFC 2104) — exposto para assinatura/verificação de
/// webhooks (F37) e assinatura de exportações (F53).
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let h = Sha256::digest(key);
        k[..32].copy_from_slice(&h);
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(msg);
    let inner_hash = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    let out = outer.finalize();
    let mut r = [0u8; 32];
    r.copy_from_slice(&out);
    r
}

/// PBKDF2-HMAC-SHA256 (RFC 2898), dkLen fixo em 32 bytes.
pub fn pbkdf2_hmac_sha256(passphrase: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut block = 1u32.to_be_bytes().to_vec();
    block.splice(0..0, salt.iter().copied());
    let mut u = hmac_sha256(passphrase, &block);
    out.copy_from_slice(&u);
    for _ in 1..iterations {
        u = hmac_sha256(passphrase, &u);
        for i in 0..32 {
            out[i] ^= u[i];
        }
    }
    out
}

// ── ChaCha20 (RFC 8439 §2.3) ───────────────────────────────────────────────

fn chacha20_block(key: &[u8; 32], counter: u32, nonce: &[u8; 12]) -> [u8; 64] {
    fn quarter(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = (x[d] ^ x[a]).rotate_left(16);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = (x[b] ^ x[c]).rotate_left(12);
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = (x[d] ^ x[a]).rotate_left(8);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = (x[b] ^ x[c]).rotate_left(7);
    }
    let mut state = [0u32; 16];
    state[..4].copy_from_slice(&[0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
    for i in 0..8 {
        state[4 + i] = u32::from_le_bytes([
            key[i * 4],
            key[i * 4 + 1],
            key[i * 4 + 2],
            key[i * 4 + 3],
        ]);
    }
    state[12] = counter;
    for i in 0..3 {
        state[13 + i] = u32::from_le_bytes([
            nonce[i * 4],
            nonce[i * 4 + 1],
            nonce[i * 4 + 2],
            nonce[i * 4 + 3],
        ]);
    }
    let mut working = state;
    for _ in 0..10 {
        quarter(&mut working, 0, 4, 8, 12);
        quarter(&mut working, 1, 5, 9, 13);
        quarter(&mut working, 2, 6, 10, 14);
        quarter(&mut working, 3, 7, 11, 15);
        quarter(&mut working, 0, 5, 10, 15);
        quarter(&mut working, 1, 6, 11, 12);
        quarter(&mut working, 2, 7, 8, 13);
        quarter(&mut working, 3, 4, 9, 14);
    }
    let mut out = [0u8; 64];
    for i in 0..16 {
        let v = working[i].wrapping_add(state[i]);
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    }
    out
}

fn chacha20_xor(key: &[u8; 32], nonce: &[u8; 12], data: &mut [u8]) {
    let mut counter = 1u32; // RFC 8439 AEAD: contador do keystream começa em 1
    for chunk in data.chunks_mut(64) {
        let block = chacha20_block(key, counter, nonce);
        counter = counter.wrapping_add(1);
        for (i, b) in chunk.iter_mut().enumerate() {
            *b ^= block[i];
        }
    }
}

// ── Poly1305 (RFC 8439 §2.5) ───────────────────────────────────────────────

fn poly1305_mac(msg: &[u8], key: &[u8; 32]) -> [u8; 16] {
    // Estrutura clássica radix-2^26 (referência: poly1305-donna).
    let mut r = [0u8; 16];
    r.copy_from_slice(&key[..16]);
    r[3] &= 15;
    r[7] &= 15;
    r[11] &= 15;
    r[15] &= 15;
    r[4] &= 252;
    r[8] &= 252;
    r[12] &= 252;

    let le32 = |b: &[u8], i: usize| -> u64 {
        u32::from_le_bytes([b[i * 4], b[i * 4 + 1], b[i * 4 + 2], b[i * 4 + 3]]) as u64
    };
    let t0 = le32(&r, 0);
    let t1 = le32(&r, 1);
    let t2 = le32(&r, 2);
    let t3 = le32(&r, 3);

    let r0 = t0 & 0x3ff_ffff;
    let mut t = (t0 >> 26) | (t1 << 6);
    let r1 = t & 0x3ff_ffff;
    t = (t1 >> 20) | (t2 << 12);
    let r2 = t & 0x3ff_ffff;
    t = (t2 >> 14) | (t3 << 18);
    let r3 = t & 0x3ff_ffff;
    let r4 = t3 >> 8;

    let s0 = le32(&key[16..], 0);
    let s1 = le32(&key[16..], 1);
    let s2 = le32(&key[16..], 2);
    let s3 = le32(&key[16..], 3);

    let mut h = [0u64; 5];
    let mut msg = msg;
    loop {
        // bloco de 16 bytes + bit 1 (0x01 após os dados, RFC 8439)
        let mut block = [0u8; 17];
        let take = msg.len().min(16);
        block[..take].copy_from_slice(&msg[..take]);
        block[take] = 1;
        let last = take < 16;
        msg = &msg[take..];

        let t0 = u32::from_le_bytes([block[0], block[1], block[2], block[3]]) as u64;
        let t1 = u32::from_le_bytes([block[4], block[5], block[6], block[7]]) as u64;
        let t2 = u32::from_le_bytes([block[8], block[9], block[10], block[11]]) as u64;
        let t3 = u32::from_le_bytes([block[12], block[13], block[14], block[15]]) as u64;
        let hibit: u64 = if last { 0 } else { 1 << 24 };

        h[0] += t0 & 0x3ff_ffff;
        h[1] += ((t0 >> 26) | (t1 << 6)) & 0x3ff_ffff;
        h[2] += ((t1 >> 20) | (t2 << 12)) & 0x3ff_ffff;
        h[3] += ((t2 >> 14) | (t3 << 18)) & 0x3ff_ffff;
        h[4] += (t3 >> 8) | hibit;

        // h *= r  (mod 2^130 - 5, com r*5 nas dobras)
        let m0 = h[0] * r0 + h[1] * (5 * r4) + h[2] * (5 * r3) + h[3] * (5 * r2) + h[4] * (5 * r1);
        let m1 = h[0] * r1 + h[1] * r0 + h[2] * (5 * r4) + h[3] * (5 * r3) + h[4] * (5 * r2);
        let m2 = h[0] * r2 + h[1] * r1 + h[2] * r0 + h[3] * (5 * r4) + h[4] * (5 * r3);
        let m3 = h[0] * r3 + h[1] * r2 + h[2] * r1 + h[3] * r0 + h[4] * (5 * r4);
        let m4 = h[0] * r4 + h[1] * r3 + h[2] * r2 + h[3] * r1 + h[4] * r0;

        let mut c: u64;
        h[0] = m0 & 0x3ff_ffff; c = m0 >> 26;
        h[1] = (m1 + c) & 0x3ff_ffff; c = (m1 + c) >> 26;
        h[2] = (m2 + c) & 0x3ff_ffff; c = (m2 + c) >> 26;
        h[3] = (m3 + c) & 0x3ff_ffff; c = (m3 + c) >> 26;
        h[4] = (m4 + c) & 0x3ff_ffff; c = (m4 + c) >> 26;
        h[0] += c * 5;
        c = h[0] >> 26;
        h[0] &= 0x3ff_ffff;
        h[1] += c;

        if msg.is_empty() {
            break;
        }
    }

    // fully carry
    let mut c: u64;
    c = h[1] >> 26; h[1] &= 0x3ff_ffff; h[2] += c;
    c = h[2] >> 26; h[2] &= 0x3ff_ffff; h[3] += c;
    c = h[3] >> 26; h[3] &= 0x3ff_ffff; h[4] += c;
    c = h[4] >> 26; h[4] &= 0x3ff_ffff; h[0] += c * 5;
    c = h[0] >> 26; h[0] &= 0x3ff_ffff; h[1] += c;

    // h - (2^130 - 5): computa g = h + 5 - 2^130; se g >= 0 usa g
    let mut g = [0u64; 5];
    g[0] = h[0] + 5;
    c = g[0] >> 26; g[0] &= 0x3ff_ffff;
    g[1] = h[1] + c;
    c = g[1] >> 26; g[1] &= 0x3ff_ffff;
    g[2] = h[2] + c;
    c = g[2] >> 26; g[2] &= 0x3ff_ffff;
    g[3] = h[3] + c;
    c = g[3] >> 26; g[3] &= 0x3ff_ffff;
    // donna: g4 = h4 + c - 2^130; o borrow é intencional (wrap) em Rust
    g[4] = (h[4] + c).wrapping_sub(1 << 26); // subtrai 2^130 (bit 130 => limb4 bit 26)

    // mask = 0xffffffff se h >= p (g sem borrow), senão 0
    let mask = (g[4] >> 63).wrapping_sub(1) & 0xffff_ffff;
    let nmask = !mask;
    for i in 0..5 {
        g[i] = (g[i] & mask) | (h[i] & nmask);
    }

    // h mod 2^128: empacota os 26-bit limbs em 4 words de 32 bits
    let mut w0 = ((g[0]) | (g[1] << 26)) as u32;
    let mut w1 = ((g[1] >> 6) | (g[2] << 20)) as u32;
    let mut w2 = ((g[2] >> 12) | (g[3] << 14)) as u32;
    let mut w3 = ((g[3] >> 18) | (g[4] << 8)) as u32;

    // mac = (h + s) mod 2^128, word-wise com carry (estilo donna)
    let mut f: u64;
    f = w0 as u64 + s0;
    w0 = f as u32;
    f = w1 as u64 + s1 + (f >> 32);
    w1 = f as u32;
    f = w2 as u64 + s2 + (f >> 32);
    w2 = f as u32;
    f = w3 as u64 + s3 + (f >> 32);
    w3 = f as u32;

    let mut out = [0u8; 16];
    out[0..4].copy_from_slice(&w0.to_le_bytes());
    out[4..8].copy_from_slice(&w1.to_le_bytes());
    out[8..12].copy_from_slice(&w2.to_le_bytes());
    out[12..16].copy_from_slice(&w3.to_le_bytes());
    out
}

// ── AEAD ChaCha20-Poly1305 (RFC 8439 §2.8) ─────────────────────────────────

pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

pub fn aead_seal(key: &[u8; 32], nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> Vec<u8> {
    let mut poly_key = [0u8; 32];
    poly_key.copy_from_slice(&chacha20_block(key, 0, nonce)[..32]);

    let mut ciphertext = plaintext.to_vec();
    chacha20_xor(key, nonce, &mut ciphertext);

    let mut mac_data = aad.to_vec();
    mac_data.extend_from_slice(&pad16(aad.len()));
    mac_data.extend_from_slice(&ciphertext);
    mac_data.extend_from_slice(&pad16(ciphertext.len()));
    mac_data.extend_from_slice(&(aad.len() as u64).to_le_bytes());
    mac_data.extend_from_slice(&(ciphertext.len() as u64).to_le_bytes());
    let tag = poly1305_mac(&mac_data, &poly_key);

    let mut out = ciphertext;
    out.extend_from_slice(&tag);
    out
}

pub fn aead_open(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext_and_tag: &[u8],
    aad: &[u8],
) -> Option<Vec<u8>> {
    if ciphertext_and_tag.len() < TAG_LEN {
        return None;
    }
    let (ct, tag) = ciphertext_and_tag.split_at(ciphertext_and_tag.len() - TAG_LEN);
    let mut poly_key = [0u8; 32];
    poly_key.copy_from_slice(&chacha20_block(key, 0, nonce)[..32]);

    let mut mac_data = aad.to_vec();
    mac_data.extend_from_slice(&pad16(aad.len()));
    mac_data.extend_from_slice(ct);
    mac_data.extend_from_slice(&pad16(ct.len()));
    mac_data.extend_from_slice(&(aad.len() as u64).to_le_bytes());
    mac_data.extend_from_slice(&(ct.len() as u64).to_le_bytes());
    let expected = poly1305_mac(&mac_data, &poly_key);

    if !constant_eq(&expected, tag) {
        return None;
    }
    let mut plaintext = ct.to_vec();
    chacha20_xor(key, nonce, &mut plaintext);
    Some(plaintext)
}

fn pad16(len: usize) -> Vec<u8> {
    vec![0u8; (16 - (len % 16)) % 16]
}

fn constant_eq(a: &[u8; 16], b: &[u8]) -> bool {
    b.len() == 16 && a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

// ── Vault de alto nível (F51) ──────────────────────────────────────────────

pub const PBKDF2_ITERATIONS: u32 = 100_000;

/// Deriva a chave mestra da passphrase + salt persistido.
pub fn derive_key(passphrase: &str, salt: &[u8; 16]) -> [u8; 32] {
    pbkdf2_hmac_sha256(passphrase.as_bytes(), salt, PBKDF2_ITERATIONS)
}

// ── Entropia sem crates extras (leitura de /dev/urandom + fallback) ────────

fn rand_bytes(n: usize) -> Vec<u8> {
    use std::io::Read;
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let mut buf = vec![0u8; n];
        if f.read_exact(&mut buf).is_ok() {
            return buf;
        }
    }
    // fallback: hash de relógio + pid + contador (não criptográfico, só anti-colisão)
    static CTR: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let mut out = Vec::with_capacity(n);
    let mut seed = Vec::new();
    seed.extend_from_slice(
        &std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos().to_le_bytes())
            .unwrap_or([0u8; 16]),
    );
    seed.extend_from_slice(&(std::process::id() as u64).to_le_bytes());
    seed.extend_from_slice(&CTR.fetch_add(1, std::sync::atomic::Ordering::Relaxed).to_le_bytes());
    while out.len() < n {
        let digest = Sha256::digest(&seed);
        out.extend_from_slice(&digest);
        seed = digest.to_vec();
    }
    out.truncate(n);
    out
}

// ── Vault sobre o Store (F51) ───────────────────────────────────────────────

const SALT_ROW: &str = "__colmeia_vault_salt__";

fn vault_salt(store: &crate::store::Store) -> anyhow::Result<[u8; 16]> {
    if let Some((hexsalt, _)) = store.secret_get(SALT_ROW)? {
        let bytes = hex::decode(hexsalt)?;
        if bytes.len() == 16 {
            let mut salt = [0u8; 16];
            salt.copy_from_slice(&bytes);
            return Ok(salt);
        }
    }
    let salt: [u8; 16] = rand_bytes(16).try_into().unwrap();
    store.secret_put(SALT_ROW, &hex::encode(salt), "")?;
    Ok(salt)
}

/// Cifra e persiste um segredo no cofre. Sobrescreve se já existir.
pub fn vault_set(
    store: &crate::store::Store,
    name: &str,
    value: &str,
    passphrase: &str,
) -> anyhow::Result<()> {
    let salt = vault_salt(store)?;
    let key = derive_key(passphrase, &salt);
    let nonce: [u8; NONCE_LEN] = rand_bytes(NONCE_LEN).try_into().unwrap();
    let sealed = aead_seal(&key, &nonce, value.as_bytes(), name.as_bytes());
    store.secret_put(name, &hex::encode(sealed), &hex::encode(nonce))
}

/// Abre um segredo do cofre.
/// `Ok(None)` = não existe; `Err` = passphrase incorreta ou segredo adulterado.
pub fn vault_get(
    store: &crate::store::Store,
    name: &str,
    passphrase: &str,
) -> anyhow::Result<Option<String>> {
    let (ct_hex, nonce_hex) = match store.secret_get(name)? {
        Some(v) => v,
        None => return Ok(None),
    };
    if nonce_hex.is_empty() {
        anyhow::bail!("segredo '{name}' não é um valor cifrado do vault");
    }
    let salt = vault_salt(store)?;
    let key = derive_key(passphrase, &salt);
    let ct = hex::decode(ct_hex)?;
    let nonce_bytes = hex::decode(nonce_hex)?;
    if nonce_bytes.len() != NONCE_LEN {
        anyhow::bail!("nonce inválido no segredo '{name}'");
    }
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&nonce_bytes);
    match aead_open(&key, &nonce, &ct, name.as_bytes()) {
        Some(pt) => Ok(Some(String::from_utf8_lossy(&pt).into_owned())),
        None => anyhow::bail!("falha de autenticação no segredo '{name}' (passphrase incorreta?)"),
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn quarter_round_rfc8439_vetor() {
        // RFC 8439 §2.1.1
        let mut x = [0u32; 16];
        x[0] = 0x11111111;
        x[1] = 0x01020304;
        x[2] = 0x9b8d6f43;
        x[3] = 0x01234567;
        quarter_pub(&mut x, 0, 1, 2, 3);
        assert_eq!(x[0], 0xea2a92f4);
        assert_eq!(x[1], 0xcb1cf8ce);
        assert_eq!(x[2], 0x4581472e);
        assert_eq!(x[3], 0x5881c4bb);
    }

    #[test]
    fn quarter_round_estado_rfc8439_vetor() {
        // RFC 8439 §2.2.1 — QUARTERROUND(2,7,8,13)
        let mut x: [u32; 16] = [
            0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
            0x2a5f714c, 0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0x3d631689,
            0x2098d9d6, 0x91dbd320,
        ];
        quarter_pub(&mut x, 2, 7, 8, 13);
        assert_eq!(x[2], 0xbdb886dc);
        assert_eq!(x[7], 0xcfacafd2);
        assert_eq!(x[8], 0xe46bea80);
        assert_eq!(x[13], 0xccc07c79);
    }

    fn quarter_pub(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        // mesma função interna, exposta só para o teste
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = (x[d] ^ x[a]).rotate_left(16);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = (x[b] ^ x[c]).rotate_left(12);
        x[a] = x[a].wrapping_add(x[b]);
        x[d] = (x[d] ^ x[a]).rotate_left(8);
        x[c] = x[c].wrapping_add(x[d]);
        x[b] = (x[b] ^ x[c]).rotate_left(7);
    }

    #[test]
    fn bloco_estado_intermediario_debug() {
        // Replica chacha20_block inspecionando o estado após 20 rounds (RFC §2.3.2)
        let mut key = [0u8; 32];
        for (i, b) in key.iter_mut().enumerate() {
            *b = i as u8;
        }
        let nonce: [u8; 12] = [0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00];
        let mut state = [0u32; 16];
        state[..4].copy_from_slice(&[0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]);
        for i in 0..8 {
            state[4 + i] = u32::from_le_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]]);
        }
        state[12] = 1;
        for i in 0..3 {
            state[13 + i] = u32::from_le_bytes([nonce[i * 4], nonce[i * 4 + 1], nonce[i * 4 + 2], nonce[i * 4 + 3]]);
        }
        assert_eq!(
            state,
            [
                0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
                0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000,
                0x4a000000, 0x00000000
            ],
            "setup do estado diverge da RFC"
        );
        let mut working = state;
        for _ in 0..10 {
            quarter_pub(&mut working, 0, 4, 8, 12);
            quarter_pub(&mut working, 1, 5, 9, 13);
            quarter_pub(&mut working, 2, 6, 10, 14);
            quarter_pub(&mut working, 3, 7, 11, 15);
            quarter_pub(&mut working, 0, 5, 10, 15);
            quarter_pub(&mut working, 1, 6, 11, 12);
            quarter_pub(&mut working, 2, 7, 8, 13);
            quarter_pub(&mut working, 3, 4, 9, 14);
        }
        let expected_after: [u32; 16] = [
            0x837778ab, 0xe238d763, 0xa67ae21e, 0x5950bb2f, 0xc4f2d0c7, 0xfc62bb2f, 0x8fa018fc,
            0x3f5ec7b7, 0x335271c2, 0xf29489f3, 0xeabda8fc, 0x82e46ebd, 0xd19c12b4, 0xb04e16de,
            0x9e83d0cb, 0x4e3c50a2,
        ];
        assert_eq!(working, expected_after, "estado após 20 rounds diverge");
    }

    #[test]
    fn pbkdf2_rfc7914_vetor() {
        // vetor da RFC 7914 (PBKDF2-HMAC-SHA256, 1 iteração)
        let out = pbkdf2_hmac_sha256(b"password", b"salt", 1);
        let hexed: String = out.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hexed,
            "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b"
        );
    }

    #[test]
    fn chacha20_rfc8439_vetor() {
        // RFC 8439 §2.3.2
        let mut key = [0u8; 32];
        for (i, b) in key.iter_mut().enumerate() {
            *b = i as u8;
        }
        let nonce: [u8; 12] = [0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00];
        let block = chacha20_block(&key, 1, &nonce);
        let hexed: String = block[..16].iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hexed, "10f1e7e4d13b5915500fdd1fa32071c4");
    }

    #[test]
    fn poly1305_rfc8439_vetor_isolado() {
        // RFC 8439 §2.5.2
        let key_hex = "85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b";
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = u8::from_str_radix(&key_hex[i * 2..i * 2 + 2], 16).unwrap();
        }
        let tag = poly1305_mac(b"Cryptographic Forum Research Group", &key);
        let hexed: String = tag.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hexed, "a8061dc1305136c6c22b8baf0c0127a9");
    }

    #[test]
    fn aead_rfc8439_vetor() {
        // RFC 8439 §2.8.2
        let key: [u8; 32] = [
            0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d,
            0x8e, 0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b,
            0x9c, 0x9d, 0x9e, 0x9f,
        ];
        let nonce: [u8; 12] = [
            0x07, 0x00, 0x00, 0x00, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47,
        ];
        let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
        let aad: [u8; 12] = [
            0x50, 0x51, 0x52, 0x53, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
        ];
        let sealed = aead_seal(&key, &nonce, plaintext, &aad);
        let expected_ct_prefix = "d31a8d34648e60db7b86afbc53ef7ec2";
        let hexed: String = sealed[..16].iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hexed, expected_ct_prefix);
        let expected_tag = "1ae10b594f09e26a7e902ecbd0600691";
        let tag_hex: String = sealed[sealed.len() - 16..].iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(tag_hex, expected_tag);

        let opened = aead_open(&key, &nonce, &sealed, &aad).unwrap();
        assert_eq!(opened, plaintext);

        // tag adulterado falha
        let mut bad = sealed.clone();
        let n = bad.len();
        bad[n - 1] ^= 1;
        assert!(aead_open(&key, &nonce, &bad, &aad).is_none());
    }
}
