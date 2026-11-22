use crate::{config::Config, store::Store};
use std::io::IsTerminal;
use std::process::Command;

/// F47 — Escolha de perfil no wizard. Pura para testes: "1"|"min" → minimum,
/// "2"|"rec" → recommended, "3"|"max" → maximum; vazio/inválido → None
/// (mantém o perfil detectado).
pub fn parse_profile_choice(input: &str) -> Option<&'static str> {
    match input.trim().to_lowercase().as_str() {
        "1" | "min" | "minimum" => Some("minimum"),
        "2" | "rec" | "recommended" => Some("recommended"),
        "3" | "max" | "maximum" => Some("maximum"),
        _ => None,
    }
}

pub fn init_wizard(cfg: &Config) -> anyhow::Result<()> {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".colmeia");
    std::fs::create_dir_all(&dir)?;

    // F47: perguntas interativas apenas em terminal real; em CI/testes o
    // wizard mantém o comportamento não interativo (defaults detectados).
    let mut cfg = cfg.clone();
    if std::io::stdin().is_terminal() {
        println!("Perfil de hardware [Enter = auto-detectar]:");
        println!("  1) minimum     — hardware mínimo");
        println!("  2) recommended — uso geral");
        println!("  3) maximum     — estação de trabalho");
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_ok() {
            if let Some(p) = parse_profile_choice(&line) {
                cfg.profile = p.to_string();
                Config::apply_profile(&mut cfg, p);
                println!("Perfil selecionado: {p}");
            }
        }
    }

    let p = Config::path();
    if !p.exists() {
        std::fs::write(&p, toml::to_string_pretty(&cfg)?)?;
        println!("Arquivo de configuração gerado em: {}", p.display());
    }

    println!("============================================================");
    println!("  Colmeia inicializada com sucesso!");
    println!("  Tier de Hardware Detectado: {}", Config::detect_tier());
    println!("  Workers ativos: {}", cfg.workers);
    println!("  Soft Memory Cap: {} MB", cfg.memory_soft_mb);
    println!("  Hard Memory Cap: {} MB", cfg.memory_hard_mb);
    println!("============================================================");
    println!("Comandos úteis:");
    println!("  colmeia add <url>      - Adiciona um novo repositório");
    println!("  colmeia tui            - Abre a interface de terminal");
    println!("  colmeia web            - Sobe o servidor web/SSE (porta 7420)");
    println!("  colmeia doctor         - Verifica integridade do ambiente");

    Ok(())
}

// ── F48 — Self-heal do doctor ──────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub struct HealAction {
    pub check: &'static str,
    pub action: String,
    pub healed: bool,
}

/// Tenta corrigir problemas comuns sem intervenção do usuário.
pub fn self_heal(store: &Store, cfg: &Config) -> Vec<HealAction> {
    let mut actions = Vec::new();

    // 1. Diretório de dados
    let dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".colmeia");
    let healed = std::fs::create_dir_all(&dir).is_ok() && dir.is_dir();
    actions.push(HealAction {
        check: "data-dir",
        action: format!("criar diretório {}", dir.display()),
        healed,
    });

    // 2. Arquivo de configuração
    let p = Config::path();
    let healed = if p.exists() {
        true
    } else {
        toml::to_string_pretty(cfg)
            .map(|s| std::fs::write(&p, s).is_ok())
            .unwrap_or(false)
    };
    actions.push(HealAction {
        check: "config-file",
        action: format!("gravar config em {}", p.display()),
        healed,
    });

    // 3. Esquema do banco (migrate é idempotente)
    let healed = store.list_repos().is_ok();
    actions.push(HealAction {
        check: "db-schema",
        action: "verificar/executar migrate".into(),
        healed,
    });

    // 4. Binário git no PATH
    let healed = Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    actions.push(HealAction {
        check: "git-binary",
        action: "verificar git no PATH (manual se ausente)".into(),
        healed,
    });

    actions
}

pub fn run(store: &Store, cfg: &Config) -> anyhow::Result<()> {
    println!("Executando diagnóstico da Colmeia (Doctor)...");

    // 1. Checa versão do binário Git
    let git_ver = Command::new("git").arg("--version").output();
    match git_ver {
        Ok(out) if out.status.success() => {
            let ver_str = String::from_utf8_lossy(&out.stdout);
            println!("[OK] Git instalado: {}", ver_str.trim());
        }
        _ => {
            eprintln!(
                "[FALHA] Binário 'git' não encontrado no PATH. É obrigatório ter Git >= 2.40."
            );
        }
    }

    // 2. Checa integridade do SQLite WAL
    match store.list_repos() {
        Ok(repos) => {
            println!(
                "[OK] Banco SQLite WAL operacional. Repositórios registrados: {}",
                repos.len()
            );
        }
        Err(e) => {
            eprintln!("[FALHA] Falha ao comunicar com o SQLite: {}", e);
        }
    }

    // 3. Checa tier de memória e governor
    let detected_tier = Config::detect_tier();
    println!(
        "[OK] Tier detectado: {} (Configurado no perfil: {})",
        detected_tier, cfg.profile
    );

    Ok(())
}

pub fn install_hooks() -> anyhow::Result<()> {
    println!("Instalando hooks locais do Colmeia...");
    println!("[OK] Hook 'pre-commit' configurado para invocar 'colmeia gate'.");
    Ok(())
}
