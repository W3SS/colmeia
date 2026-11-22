use crate::store::Store;
use serde::Serialize;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize)]
pub struct ScanSummary {
    pub root: String,
    pub discovered: usize,
    pub registered: usize,
    pub inspected: usize,
    pub errors: usize,
    pub truncated: bool,
    pub paths: Vec<String>,
}

pub fn canonical_dir(path: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(path);
    anyhow::ensure!(path.is_absolute(), "Use um caminho absoluto para a pasta");
    anyhow::ensure!(path.exists(), "Pasta não encontrada: {}", path.display());
    let canonical = path.canonicalize().map_err(|error| {
        anyhow::anyhow!(
            "Não foi possível acessar a pasta {}: {}",
            path.display(),
            error
        )
    })?;
    anyhow::ensure!(canonical.is_dir(), "O caminho não é uma pasta");
    Ok(canonical)
}

fn is_repository(path: &Path) -> bool {
    path.join(".git").is_dir()
        || path.join(".git").is_file()
        || (path.join("HEAD").is_file()
            && path.join("objects").is_dir()
            && path.join("refs").is_dir())
}

fn ignored(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "node_modules" | "target" | ".venv" | "venv" | ".next" | "dist" | "build")
    )
}

struct DirectoryInspection {
    path: PathBuf,
    repository: bool,
    head: Option<String>,
    branch: Option<String>,
    children: Vec<PathBuf>,
    errors: usize,
}

fn git_value(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn inspect_directory(path: PathBuf) -> DirectoryInspection {
    let repository = is_repository(&path);
    let head = repository
        .then(|| git_value(&path, &["rev-parse", "--verify", "HEAD"]))
        .flatten();
    let branch = repository
        .then(|| git_value(&path, &["symbolic-ref", "--quiet", "--short", "HEAD"]))
        .flatten();
    let mut inspection = DirectoryInspection {
        path,
        repository,
        head,
        branch,
        children: Vec::new(),
        errors: 0,
    };
    // Bare repositories contain Git object directories, not project trees.
    let bare = repository && !inspection.path.join(".git").exists();
    if bare {
        return inspection;
    }
    match std::fs::read_dir(&inspection.path) {
        Ok(children) => {
            for child in children {
                match child {
                    Ok(child) => match child.file_type() {
                        Ok(kind) if kind.is_dir() && !ignored(&child.path()) => {
                            inspection.children.push(child.path());
                        }
                        Ok(_) => {}
                        Err(_) => inspection.errors += 1,
                    },
                    Err(_) => inspection.errors += 1,
                }
            }
        }
        Err(_) => inspection.errors += 1,
    }
    inspection
}

pub fn scan(store: &Store, root: &str) -> anyhow::Result<ScanSummary> {
    let root = canonical_dir(root)?;
    let root_text = root.to_string_lossy().into_owned();
    let mut summary = ScanSummary {
        root: root_text.clone(),
        discovered: 0,
        registered: 0,
        inspected: 0,
        errors: 0,
        truncated: false,
        paths: Vec::new(),
    };
    let mut directories = VecDeque::from([root]);
    while !directories.is_empty() {
        let mut workers = Vec::with_capacity(8);
        for _ in 0..8 {
            if let Some(path) = directories.pop_front() {
                workers.push(std::thread::spawn(move || inspect_directory(path)));
            }
        }
        for worker in workers {
            let inspected = worker
                .join()
                .map_err(|_| anyhow::anyhow!("Falha ao inspecionar uma pasta"))?;
            summary.inspected += 1;
            summary.errors += inspected.errors;
            if inspected.repository {
                summary.discovered += 1;
                let text = inspected.path.to_string_lossy().into_owned();
                let id = store.add_repo(&text, None, "managed")?;
                if let Some(head) = inspected.head {
                    store.update_repo_head(id, &head)?;
                }
                if let Some(branch) = inspected.branch {
                    store.update_repo_branch(id, &branch)?;
                }
                summary.registered += 1;
                if summary.paths.len() < 200 {
                    summary.paths.push(text);
                }
            }
            directories.extend(inspected.children);
            if summary.inspected.is_multiple_of(500) {
                store.event(
                    0,
                    "scan.progress",
                    &serde_json::json!({
                        "root": root_text,
                        "inspected": summary.inspected,
                        "discovered": summary.discovered
                    }),
                )?;
            }
        }
        if summary.inspected >= 100_000 || summary.discovered >= 10_000 {
            summary.truncated = true;
            break;
        }
    }
    store.register_scan_root(&root_text)?;
    store.event(
        0,
        "scan.done",
        &serde_json::json!({
            "root": root_text,
            "discovered": summary.discovered,
            "errors": summary.errors,
            "truncated": summary.truncated
        }),
    )?;
    Ok(summary)
}

#[derive(Debug, Serialize)]
pub struct DirectoryListing {
    pub path: String,
    pub parent: Option<String>,
    pub directories: Vec<String>,
}

pub fn list_directories(path: &str) -> anyhow::Result<DirectoryListing> {
    let path = canonical_dir(path)?;
    let mut directories = std::fs::read_dir(&path)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| !name.starts_with('.'))
        .collect::<Vec<_>>();
    directories.sort();
    directories.truncate(200);
    Ok(DirectoryListing {
        path: path.to_string_lossy().into_owned(),
        parent: path.parent().map(|p| p.to_string_lossy().into_owned()),
        directories,
    })
}
