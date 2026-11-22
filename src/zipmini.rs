//! F21 — Leitor mínimo de ZIP (central directory) + descompressão deflate.
//! Sem crates pesados: só miniz_oxide (puro, já na árvore de deps).

/// Entrada descompactada de um ZIP.
pub struct ZipEntry {
    pub name: String,
    pub data: Vec<u8>,
}

fn u16le(b: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*b.get(o)?, *b.get(o + 1)?]))
}
fn u32le(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *b.get(o)?,
        *b.get(o + 1)?,
        *b.get(o + 2)?,
        *b.get(o + 3)?,
    ]))
}

/// Extrai as entradas (method 0 stored ou 8 deflate). Ignora entradas de
/// diretório. Falha somente se o container estiver malformado.
pub fn extract(bytes: &[u8]) -> anyhow::Result<Vec<ZipEntry>> {
    // Localiza o End of Central Directory por varredura reversa da assinatura
    let mut eocd = None;
    let mut pos = bytes.len().saturating_sub(22);
    while pos > 0 {
        if u32le(bytes, pos) == Some(0x0605_4b50) {
            eocd = Some(pos);
            break;
        }
        pos -= 1;
    }
    let eocd = eocd.ok_or_else(|| anyhow::anyhow!("EOCD não encontrado: não é um ZIP"))?;
    let count = u16le(bytes, eocd + 10).unwrap_or(0) as usize;
    let cd_offset = u32le(bytes, eocd + 16).unwrap_or(0) as usize;

    let mut entries = Vec::with_capacity(count.min(4096));
    let mut p = cd_offset;
    for _ in 0..count {
        if u32le(bytes, p) != Some(0x0201_4b50) {
            break; // fim do central directory
        }
        let method = u16le(bytes, p + 10).unwrap_or(0);
        let comp_size = u32le(bytes, p + 20).unwrap_or(0) as usize;
        let name_len = u16le(bytes, p + 28).unwrap_or(0) as usize;
        let extra_len = u16le(bytes, p + 30).unwrap_or(0) as usize;
        let comment_len = u16le(bytes, p + 32).unwrap_or(0) as usize;
        let local_off = u32le(bytes, p + 42).unwrap_or(0) as usize;
        let name = String::from_utf8_lossy(
            bytes
                .get(p + 46..p + 46 + name_len)
                .unwrap_or(&[]),
        )
        .to_string();

        // Local header: pula para os dados compactados
        let lh_name = u16le(bytes, local_off + 26).unwrap_or(0) as usize;
        let lh_extra = u16le(bytes, local_off + 28).unwrap_or(0) as usize;
        let data_start = local_off + 30 + lh_name + lh_extra;
        let raw = bytes
            .get(data_start..data_start + comp_size)
            .unwrap_or(&[])
            .to_vec();

        let data = match method {
            0 => raw,
            8 => miniz_oxide::inflate::decompress_to_vec(&raw)
                .map_err(|e| anyhow::anyhow!("deflate falhou em {name}: {e:?}"))?,
            other => anyhow::bail!("método ZIP {other} não suportado em {name}"),
        };

        if !name.ends_with('/') && !name.is_empty() {
            entries.push(ZipEntry { name, data });
        }
        p += 46 + name_len + extra_len + comment_len;
    }
    Ok(entries)
}
