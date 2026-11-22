use clap::Parser;
use colmeia::{
    cli::{Cli, Cmd, VaultOp},
    config::Config,
    doctor, gate,
    git::{Git, GitBackend},
    scanner, scheduler,
    store::Store,
    tui, web,
};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load()?;

    let db_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".colmeia/colmeia.db");
    let store = Arc::new(Store::open(&db_path)?);
    let git = Git::new(cfg.git_child_cap_mb);

    match cli.cmd {
        Cmd::Init => {
            doctor::init_wizard(&cfg)?;
        }

        Cmd::Add { url, mode } => {
            let repo_name = url
                .rsplit('/')
                .next()
                .unwrap_or("repo")
                .trim_end_matches(".git");
            let local_path = format!("./repos/{}", repo_name);
            let dest = std::path::Path::new(&local_path);

            println!("Clonando {} em {} (modo: {})...", url, local_path, mode);
            git.clone_repo(dest, &url, mode == "mirror")?;
            let id = store.add_repo(&local_path, Some(url.as_str()), &mode)?;
            println!("Repositório #{} registrado com sucesso!", id);
        }

        Cmd::Import { dir } => {
            let summary = scanner::scan(&store, &dir)?;
            println!(
                "{} repositórios mapeados em {} ({} entradas inspecionadas{})",
                summary.discovered,
                summary.root,
                summary.inspected,
                if summary.truncated {
                    ", limite atingido"
                } else {
                    ""
                }
            );
        }

        Cmd::List => {
            let repos = store.list_repos()?;
            if repos.is_empty() {
                println!("Nenhum repositório cadastrado. Utilize `colmeia add <url>`.");
            } else {
                println!("{:<5} {:<10} {:<10} CAMINHO", "ID", "MODO", "HEAD");
                println!("{:-<5} {:-<10} {:-<10} {:-<40}", "", "", "", "");
                for (id, path, mode, sha) in repos {
                    let short_sha = if sha.len() >= 7 { &sha[..7] } else { &sha };
                    println!("{:<5} {:<10} {:<10} {}", id, mode, short_sha, path);
                }
            }
        }

        Cmd::Sync { repo } => {
            let repos = store.list_repos()?;
            for (id, _, _, _) in repos {
                if repo.map(|target| target == id).unwrap_or(true) {
                    store.enqueue("fetch", id, &serde_json::json!({ "remote": "origin" }), 5)?;
                }
            }
            println!("Despachando sincronizações...");
            scheduler::run(store.clone(), cfg)?;
            println!("Sincronizações concluídas.");
        }

        Cmd::Gate { range, format } => {
            let repos = store.list_repos()?;
            let mut all_findings = Vec::new();
            for (id, _, _, _) in repos {
                gate::run_repo(
                    &store,
                    &git,
                    &cfg,
                    id,
                    &serde_json::json!({ "range": range }),
                )?;
                for f in store.list_findings(id)? {
                    all_findings.push(f);
                }
            }

            if format == "sarif" {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&gate::sarif(&all_findings))?
                );
            } else {
                println!("{}", serde_json::to_string_pretty(&all_findings)?);
            }
        }

        Cmd::Slice { commit, plan_only } => {
            let repos = store.list_repos()?;
            for (id, _, _, _) in repos {
                store.enqueue(
                    "slice_plan",
                    id,
                    &serde_json::json!({ "commit": commit, "max_lines": 400 }),
                    5,
                )?;
            }
            let _ = plan_only;
            scheduler::run(store.clone(), cfg)?;
            println!("Planejamento de fatias de commit concluído.");
        }

        Cmd::Merge { refspec } => {
            let repos = store.list_repos()?;
            for (id, _, _, _) in repos {
                store.enqueue(
                    "merge",
                    id,
                    &serde_json::json!({ "ref": refspec, "policy": "escalate" }),
                    3,
                )?;
            }
            scheduler::run(store.clone(), cfg)?;
            println!("Processamento de merge finalizado.");
        }

        Cmd::Tui => {
            tui::run(store.clone())?;
        }

        Cmd::Web { bind } => {
            web::run(store.clone(), &bind)?;
        }

        Cmd::Doctor { heal } => {
            doctor::run(&store, &cfg)?;
            if heal {
                println!("\nSelf-heal (F48):");
                for action in doctor::self_heal(&store, &cfg) {
                    let mark = if action.healed { "[OK]" } else { "[PENDENTE]" };
                    println!("  {} {} — {}", mark, action.check, action.action);
                }
            }
        }

        Cmd::FirmwareContract => {
            println!(
                "{}",
                serde_json::to_string_pretty(&colmeia::firmware::contract_t0())?
            );
        }

        Cmd::Hook { action } => {
            if action == "install" {
                doctor::install_hooks()?;
            } else {
                eprintln!("Ação desconhecida: {}", action);
            }
        }

        Cmd::Policy {
            kind,
            src,
            dst,
            refspec,
            glob,
            cron,
        } => {
            let id = store.add_policy(
                &kind,
                src,
                dst,
                refspec.as_deref(),
                glob.as_deref(),
                cron.as_deref(),
            )?;
            println!("Política #{} criada (kind={} cron={:?})", id, kind, cron);
        }

        Cmd::CronTick => {
            let fired = colmeia::cadence::tick(&store)?;
            println!("Políticas disparadas: {}", fired);
            scheduler::run(store.clone(), cfg)?;
        }

        Cmd::Vault { op } => {
            let passphrase = match op {
                VaultOp::Set { ref passphrase, .. } | VaultOp::Get { ref passphrase, .. } => {
                    passphrase.clone()
                }
            }
            .or_else(|| std::env::var("COLMEIA_PASSPHRASE").ok())
            .ok_or_else(|| anyhow::anyhow!("informe --passphrase ou defina COLMEIA_PASSPHRASE"))?;
            match op {
                VaultOp::Set { name, value, .. } => {
                    let value = match value {
                        Some(v) => v,
                        None => {
                            use std::io::Read;
                            let mut buf = String::new();
                            std::io::stdin().read_to_string(&mut buf)?;
                            buf.trim_end_matches('\n').to_string()
                        }
                    };
                    colmeia::secretbox::vault_set(&store, &name, &value, &passphrase)?;
                    println!("segredo '{}' armazenado no cofre", name);
                }
                VaultOp::Get { name, .. } => match colmeia::secretbox::vault_get(&store, &name, &passphrase)? {
                    Some(v) => println!("{}", v),
                    None => println!("segredo '{}' não encontrado", name),
                },
            }
        }

        Cmd::HooksInstall { repo } => {
            let installed = colmeia::hooks::install(std::path::Path::new(&repo))?;
            for p in installed {
                println!("hook instalado: {}", p.display());
            }
        }

        Cmd::CiScan {
            repo,
            limit,
            explain,
            rerun,
        } => {
            let gh = colmeia::ghci::GhCli::default();
            if !gh.available() {
                anyhow::bail!(
                    "GitHub CLI (gh) não encontrado. Instale: https://cli.github.com — \
                     autenticação não-interativa via GH_TOKEN/GITHUB_TOKEN."
                );
            }

            // Slugs a varrer: --repo explícito ou todos os repos cadastrados
            // cujo remote `origin` aponte para github.com.
            let slugs: Vec<String> = if let Some(r) = repo {
                vec![r]
            } else {
                let mut out = Vec::new();
                for (_id, path, _mode, _head) in store.list_repos()? {
                    let p = std::path::Path::new(&path);
                    let remote_out = git.run(p, &["remote", "get-url", "origin"])?;
                    if !remote_out.status.success() {
                        continue;
                    }
                    let remote = String::from_utf8_lossy(&remote_out.stdout);
                    if let Some(slug) =
                        colmeia::ghci::GhCli::repo_slug_from_remote(remote.trim())
                    {
                        out.push(slug);
                    }
                }
                out
            };

            if slugs.is_empty() {
                println!("nenhum repositório GitHub encontrado no registry");
                return Ok(());
            }

            let hf = if explain {
                match colmeia::hf::HfClient::from_env() {
                    Some(h) => Some(h),
                    None => {
                        eprintln!("aviso: HF_TOKEN ausente; --explain desativado");
                        None
                    }
                }
            } else {
                None
            };

            let report = colmeia::ghci::ci_scan(&store, &gh, &slugs, limit, explain, hf.as_ref())?;

            for f in &report.findings {
                println!(
                    "[CI-FALHA] {} #{} {} ({}) {}",
                    f.repo, f.run.database_id, f.run.workflow, f.run.branch, f.run.url
                );
                if let Some(exp) = &f.explanation {
                    println!("  causa: {}", exp);
                }
                if rerun {
                    match gh.rerun_failed(&f.repo, f.run.database_id) {
                        Ok(()) => println!("  rerun solicitado (jobs falhos)"),
                        Err(e) => eprintln!("  rerun falhou: {}", e),
                    }
                }
            }
            println!(
                "ci-scan: {} novo(s) achado(s), {} dedupado(s), {} rate-limited",
                report.findings.len(),
                report.deduped,
                report.rate_limited
            );
        }
    }

    Ok(())
}
