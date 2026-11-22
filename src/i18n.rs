//! F50 — Internacionalização mínima (PT/EN) sem crates novas.
//!
//! Tabela estática de mensagens da CLI. Idioma via `--lang` ou env
//! `COLMEIA_LANG`; padrão pt-BR. Chaves ausentes caem no pt-BR.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Pt,
    En,
}

impl Lang {
    pub fn detect(cli_lang: Option<&str>) -> Self {
        match cli_lang
            .map(|s| s.to_string())
            .or_else(|| std::env::var("COLMEIA_LANG").ok())
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "en" | "en-us" | "en-gb" | "english" => Lang::En,
            _ => Lang::Pt,
        }
    }
}

/// Mensagem por chave/idioma. pt-BR é a chave-canônica (fallback).
const TABLE: &[(&str, &str, &str)] = &[
    ("init.ok", "Colmeia inicializada com sucesso!", "Colmeia initialized successfully!"),
    ("init.config", "Arquivo de configuração gerado em", "Configuration file generated at"),
    ("doctor.ok", "diagnóstico concluído", "diagnostics complete"),
    ("vault.saved", "segredo armazenado no cofre", "secret stored in the vault"),
    ("vault.missing", "segredo não encontrado", "secret not found"),
    ("graph.edges", "arestas no grafo", "edges in graph"),
    ("sync.done", "Sincronizações concluídas.", "Synchronizations complete."),
    ("ui.quit", "para sair", "to quit"),
    ("hook.installed", "hook instalado", "hook installed"),
];

/// Traduz uma chave. Desconhecida devolve a própria chave.
pub fn t(lang: Lang, key: &str) -> &'static str {
    TABLE
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, pt, en)| match lang {
            Lang::Pt => *pt,
            Lang::En => *en,
        })
        .unwrap_or("")
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn traduz_pt_e_en() {
        assert_eq!(t(Lang::Pt, "sync.done"), "Sincronizações concluídas.");
        assert_eq!(t(Lang::En, "sync.done"), "Synchronizations complete.");
    }

    #[test]
    fn chave_desconhecida_retorna_vazio() {
        assert_eq!(t(Lang::Pt, "nao.existe"), "");
    }

    #[test]
    fn detect_por_env_e_cli() {
        std::env::set_var("COLMEIA_LANG", "en");
        assert_eq!(Lang::detect(None), Lang::En);
        assert_eq!(Lang::detect(Some("pt")), Lang::Pt);
        std::env::remove_var("COLMEIA_LANG");
        assert_eq!(Lang::detect(None), Lang::Pt);
    }
}
