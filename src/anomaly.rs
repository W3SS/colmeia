//! F30 — Detecção de anomalias em métricas de repositório (EWMA + MAD).
//!
//! Mantém uma baseline por (repo, métrica): EWMA do valor e desvio médio
//! absoluto (MAD) como escala robusta. Uma observação é anômala quando
//! desvia da EWMA por mais de `k` vezes o MAD (com piso mínimo de escala
//! para os primeiros valores).

use crate::store::Store;

const ALPHA: f64 = 0.3;
const MIN_SAMPLES: i64 = 5;
const MIN_SCALE: f64 = 1e-9;

#[derive(Debug, PartialEq)]
pub enum Verdict {
    /// baseline ainda em aquecimento (poucas amostras)
    Warmup,
    Normal,
    Anomaly { z: f64 },
}

/// Observa um valor, devolve o veredito contra a baseline *anterior* e só
/// então atualiza a baseline (senão o próprio pico dilui a detecção).
pub fn observe(store: &Store, repo_id: i64, metric: &str, value: f64) -> anyhow::Result<Verdict> {
    let (ewma, mad) = store.baseline_get(repo_id, metric)?.unwrap_or((0.0, 0.0));
    let n = store
        .conn
        .lock()
        .unwrap()
        .query_row(
            "SELECT COALESCE(n,0) FROM baselines WHERE repo_id=?1 AND metric=?2",
            rusqlite::params![repo_id, metric],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0);

    // veredito contra a baseline anterior
    let verdict = if n < MIN_SAMPLES {
        Verdict::Warmup
    } else {
        // piso relativo: série constante tem MAD=0, mas ruído pequeno não
        // pode disparar anomalia nem um desvio mínimo deixar de disparar
        let scale = mad.max(ewma.abs() * 0.01).max(MIN_SCALE);
        let z = (value - ewma).abs() / scale;
        if z > 5.0 {
            Verdict::Anomaly { z }
        } else {
            Verdict::Normal
        }
    };

    // atualização EWMA/MAD
    let n_new = n + 1;
    let new_ewma = if n == 0 { value } else { ALPHA * value + (1.0 - ALPHA) * ewma };
    let new_mad = if n == 0 {
        0.0
    } else {
        ALPHA * (value - new_ewma).abs() + (1.0 - ALPHA) * mad
    };
    store.baseline_update(repo_id, metric, new_ewma, new_mad, n_new)?;
    Ok(verdict)
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn warmup_antes_de_detectar() {
        let store = Store::open_in_memory().unwrap();
        for i in 1..=5 {
            let v = observe(&store, 1, "commits_dia", i as f64).unwrap();
            assert_eq!(v, Verdict::Warmup);
        }
        let v = observe(&store, 1, "commits_dia", 6.0).unwrap();
        assert_eq!(v, Verdict::Normal);
    }

    #[test]
    fn detecta_salto_grande() {
        let store = Store::open_in_memory().unwrap();
        for _ in 0..20 {
            observe(&store, 1, "tamanho_diff", 100.0).unwrap();
        }
        let v = observe(&store, 1, "tamanho_diff", 900.0).unwrap();
        match v {
            Verdict::Anomaly { z } => assert!(z > 5.0, "z={z}"),
            other => panic!("esperava anomalia, veio {other:?}"),
        }
    }

    #[test]
    fn baseline_por_repo_e_metrica() {
        let store = Store::open_in_memory().unwrap();
        for _ in 0..6 {
            observe(&store, 1, "m", 10.0).unwrap();
            observe(&store, 2, "m", 1000.0).unwrap();
        }
        // repo 2 tem baseline própria: 1000 não é anomalia para ele
        assert_eq!(observe(&store, 2, "m", 1000.0).unwrap(), Verdict::Normal);
    }
}
