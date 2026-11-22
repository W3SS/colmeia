//! F43 — Contrato do protocolo de hardware T0 (firmware dos nós Colmeia).
//!
//! Emite o contrato JSON que um nó físico deve implementar para participar
//! da frota: capacidades por tier de hardware, canal de eventos e comandos.

/// Contrato T0 — versão semântica estável por release.
pub const CONTRACT_VERSION: &str = "t0.1.0";

/// Emite o contrato JSON do protocolo T0.
pub fn contract_t0() -> serde_json::Value {
    serde_json::json!({
        "protocol": "colmeia-t0",
        "version": CONTRACT_VERSION,
        "eventChannel": {
            "transport": "sse",
            "path": "/api/events",
            "replay": "Last-Event-ID"
        },
        "tiers": {
            "minimum": {
                "workers": 1,
                "memory_soft_mb": 64,
                "memory_hard_mb": 96,
                "watch": "polling-2s",
                "tui": false
            },
            "recommended": {
                "workers": 2,
                "memory_soft_mb": 192,
                "memory_hard_mb": 256,
                "watch": "polling-2s",
                "tui": true
            },
            "maximum": {
                "workers": 4,
                "memory_soft_mb": 448,
                "memory_hard_mb": 512,
                "watch": "polling-2s",
                "tui": true
            }
        },
        "commands": [
            { "name": "repos.list", "direction": "node->hub" },
            { "name": "job.lease", "direction": "node->hub" },
            { "name": "job.finish", "direction": "node->hub" },
            { "name": "repo.scan", "direction": "hub->node" },
            { "name": "gate.run", "direction": "hub->node" },
            { "name": "sync.execute", "direction": "hub->node" }
        ],
        "invariants": [
            "sem dependência de kernel watches (polling determinístico)",
            "zero deps novas além das declaradas no contrato de leveza",
            "eventos ordenados por id crescente",
            "clock unix de 64 bits em todos os timestamps"
        ]
    })
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn contrato_tem_campos_obrigatorios() {
        let c = contract_t0();
        assert_eq!(c["protocol"], "colmeia-t0");
        assert!(c["version"].as_str().unwrap().starts_with("t0."));
        assert_eq!(c["eventChannel"]["transport"], "sse");
        for tier in ["minimum", "recommended", "maximum"] {
            assert!(c["tiers"][tier]["workers"].is_number(), "tier {tier}");
            assert!(c["tiers"][tier]["memory_hard_mb"].is_number());
        }
        let cmds = c["commands"].as_array().unwrap();
        assert!(cmds.iter().any(|c| c["name"] == "repos.list"));
        assert!(cmds.iter().any(|c| c["name"] == "gate.run"));
        // invariants citam polling (F04)
        let inv = c["invariants"].as_array().unwrap();
        assert!(inv.iter().any(|i| i.as_str().unwrap().contains("polling")));
    }

    #[test]
    fn contrato_e_json_serializavel() {
        let s = serde_json::to_string_pretty(&contract_t0()).unwrap();
        assert!(s.len() > 200);
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["protocol"], "colmeia-t0");
    }
}
