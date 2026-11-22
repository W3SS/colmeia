use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "colmeia",
    version,
    about = "Gestor Git de frotas determinístico"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Inicializa o ambiente e arquivos de configuração da Colmeia
    Init,

    /// Adiciona e clona um novo repositório ao registry
    Add {
        url: String,
        #[arg(long, default_value = "mirror")]
        mode: String,
    },

    /// Importa em lote todos os repositórios existentes sob um diretório
    Import { dir: String },

    /// Lista todos os repositórios cadastrados na Colmeia
    List,

    /// Executa sincronizações programadas
    Sync {
        #[arg(long)]
        repo: Option<i64>,
    },

    /// Executa o scan SAST / gate de segurança sobre uma faixa de commits
    Gate {
        #[arg(long, default_value = "HEAD~1..HEAD")]
        range: String,
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Divide commits grandes de IA em fatias atômicas e testáveis
    Slice {
        commit: String,
        #[arg(long)]
        plan_only: bool,
    },

    /// Executa reconciliação e merge determinístico de branch
    Merge {
        #[arg(long)]
        refspec: String,
    },

    /// Abre a interface TUI (Terminal User Interface)
    Tui,

    /// Inicia o servidor Web; o mapeamento de pastas exige localhost
    Web {
        #[arg(long, default_value = "127.0.0.1:7420")]
        bind: String,
    },

    /// Executa autodiagnóstico do ambiente local e hardware
    Doctor {
        /// Tenta corrigir automaticamente os problemas encontrados (F48)
        #[arg(long)]
        heal: bool,
    },

    /// Emite o contrato JSON do protocolo de hardware T0 (F43)
    FirmwareContract,

    /// Gerencia hooks nativos do Git
    Hook { action: String },

    /// Cria uma política de sincronização agendada (cron) ou por evento
    Policy {
        #[arg(long)]
        kind: String,
        #[arg(long)]
        src: Option<i64>,
        #[arg(long)]
        dst: Option<i64>,
        #[arg(long)]
        refspec: Option<String>,
        #[arg(long)]
        glob: Option<String>,
        #[arg(long)]
        cron: Option<String>,
    },

    /// Executa um tick do scheduler de cron imediatamente (útil em testes)
    CronTick,

    /// Cofre de segredos cifrados (F51). Passphrase via --passphrase ou env COLMEIA_PASSPHRASE
    Vault {
        #[command(subcommand)]
        op: VaultOp,
    },

    /// Instala hooks git locais (pre-commit/pre-push) em um repositório (F25)
    HooksInstall { repo: String },

    /// Detecta falhas de CI via GitHub CLI (gh), com dedup determinístico
    CiScan {
        /// Repositório GitHub no formato owner/repo (default: todos os
        /// repos cadastrados com remote github.com)
        #[arg(long)]
        repo: Option<String>,
        /// Máximo de runs falhos consultados por repositório
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Explica cada falha via Hugging Face (requer HF_TOKEN)
        #[arg(long)]
        explain: bool,
        /// Re-executa apenas os jobs falhos dos runs recém-detectados
        /// (triagem de flaky; nunca disparado sem esta flag)
        #[arg(long)]
        rerun: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum VaultOp {
    /// Armazena um segredo cifrado (lê stdin se --value for omitido)
    Set {
        name: String,
        #[arg(long)]
        value: Option<String>,
        #[arg(long)]
        passphrase: Option<String>,
    },
    /// Abre um segredo do cofre
    Get {
        name: String,
        #[arg(long)]
        passphrase: Option<String>,
    },
}
