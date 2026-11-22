use colmeia::{config::Config, scheduler, store::Store};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let config = Config::load()?;
    let db_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".colmeia/colmeia.db");
    let store = Arc::new(Store::open(&db_path)?);
    println!(
        "colmeiad iniciado; aguardando jobs em {}",
        db_path.display()
    );
    scheduler::run_daemon(store, config)
}
