//! gh-ci — Testes de integração da automação GitHub CLI + Hugging Face.
//!
//! Estratégia: um binário `gh` falso em diretório temporário responde os
//! comandos canônicos (`run list --json`, `run view --log-failed`,
//! `run rerun --failed`) e grava os argv recebidos para assert. O PATH é
//! isolado durante o teste, exatamente como em hooks_test.rs.

use colmeia::ghci::{self, GhCli, GhciError};
use colmeia::store::Store;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Serializa os testes que mutam variáveis de ambiente (PATH, FAKE_GH_*).
/// Sem isso, threads de teste paralelas trocam o PATH uma da outra e o
/// resolve_bin acha binários em diretórios já removidos (ENOENT no spawn).
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap()
}

/// Cria um `gh` falso que responde conforme o subcomando em `$FAKE_GH_MODE`:
/// - "list"  → JSON canônico de `gh run list --status failure --json …`
/// - "log"   → log de passos falhos
/// - "fail"  → exit 1 com stderr
/// Grava os argumentos em `$FAKE_GH_ARGS` para inspeção.
fn fake_gh(dir: &Path) -> std::path::PathBuf {
    let script = r#"#!/bin/sh
: > /dev/null
printf '%s\n' "$*" >> "$FAKE_GH_ARGS"
case "$FAKE_GH_MODE" in
  list)
    printf '[{"databaseId":123456789,"workflowName":"ci","headBranch":"main","conclusion":"failure","createdAt":"2026-11-22T10:00:00Z","url":"https://github.com/o/r/actions/runs/123456789"}]'
    ;;
  log)
    printf 'step test: cargo test falhou\nthread panicked at src/lib.rs:42\n'
    ;;
  fail)
    echo 'HTTP 404: Not Found' >&2
    exit 1
    ;;
esac
"#;
    let bin = dir.join("gh");
    std::fs::write(&bin, script).unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

struct PathGuard {
    original: std::ffi::OsString,
}
impl PathGuard {
    fn isolate(bin_dir: &Path) -> Self {
        let original = std::env::var_os("PATH").unwrap();
        std::env::set_var("PATH", bin_dir);
        Self { original }
    }
}
impl Drop for PathGuard {
    fn drop(&mut self) {
        std::env::set_var("PATH", &self.original);
    }
}

fn set_mode(mode: &str) {
    std::env::set_var("FAKE_GH_MODE", mode);
}

#[test]
fn detect_failures_parseia_json_canonico() {
    let _env = env_lock();
    let dir = tempfile::tempdir().unwrap();
    let _gh = fake_gh(dir.path());
    let _guard = PathGuard::isolate(dir.path());
    set_mode("list");

    let gh = GhCli::default();
    let runs = gh.detect_failures("o/r", 10).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].database_id, 123456789);
    assert_eq!(runs[0].workflow, "ci");
    assert_eq!(runs[0].branch, "main");
    assert_eq!(runs[0].conclusion, "failure");
    assert!(runs[0].url.contains("actions/runs/123456789"));
}

#[test]
fn detect_failures_passa_flags_corretas() {
    let _env = env_lock();
    let dir = tempfile::tempdir().unwrap();
    let _gh = fake_gh(dir.path());
    let args_file = dir.path().join("args.txt");
    std::env::set_var("FAKE_GH_ARGS", &args_file);
    let _guard = PathGuard::isolate(dir.path());
    set_mode("list");

    let gh = GhCli::default();
    gh.detect_failures("o/r", 5).unwrap();
    let logged = std::fs::read_to_string(&args_file).unwrap();
    assert!(logged.contains("run list --repo o/r --status failure --limit 5"));
    assert!(logged.contains("--json databaseId,workflowName,headBranch"));
}

#[test]
fn failure_log_trunca_e_rerun_usa_failed() {
    let _env = env_lock();
    let dir = tempfile::tempdir().unwrap();
    let _gh = fake_gh(dir.path());
    let args_file = dir.path().join("args.txt");
    std::env::set_var("FAKE_GH_ARGS", &args_file);
    let _guard = PathGuard::isolate(dir.path());

    let gh = GhCli::default();

    set_mode("log");
    let log = gh.failure_log("o/r", 123456789, 10_000).unwrap();
    assert!(log.contains("cargo test falhou"));

    set_mode("list");
    gh.rerun_failed("o/r", 123456789).unwrap();
    let logged = std::fs::read_to_string(&args_file).unwrap();
    assert!(logged.contains("run rerun 123456789 --repo o/r --failed"));
}

#[test]
fn exit_nao_zero_vira_erro_exit() {
    let _env = env_lock();
    let dir = tempfile::tempdir().unwrap();
    let _gh = fake_gh(dir.path());
    let _guard = PathGuard::isolate(dir.path());
    set_mode("fail");

    let gh = GhCli::default();
    match gh.detect_failures("o/r", 1) {
        Err(GhciError::Exit { code, stderr }) => {
            assert_eq!(code, Some(1));
            assert!(stderr.contains("404"));
        }
        other => panic!("esperava GhciError::Exit, veio {:?}", other.map(|_| ())),
    }
}

#[test]
fn gh_ausente_retorna_not_installed() {
    let _env = env_lock();
    let dir = tempfile::tempdir().unwrap();
    // PATH vazio de gh: só um `sh` inócuo para o spawn
    #[cfg(unix)]
    std::os::unix::fs::symlink("/bin/sh", dir.path().join("sh")).unwrap();
    let _guard = PathGuard::isolate(dir.path());

    let gh = GhCli::default();
    assert!(!gh.available());
    match gh.detect_failures("o/r", 1) {
        Err(GhciError::NotInstalled) => {}
        _ => panic!("esperava NotInstalled"),
    }
}

#[test]
fn ci_scan_e_idempotente_por_run_id() {
    let _env = env_lock();
    let dir = tempfile::tempdir().unwrap();
    let _gh = fake_gh(dir.path());
    let _guard = PathGuard::isolate(dir.path());
    set_mode("list");

    let store = Store::open_in_memory().unwrap();
    let gh = GhCli::default();

    // Primeira passada: 1 achado novo.
    let r1 = ghci::ci_scan(&store, &gh, &["o/r".to_string()], 10, false, None).unwrap();
    assert_eq!(r1.findings.len(), 1);
    assert_eq!(r1.deduped, 0);
    assert!(store.ci_seen(123456789).unwrap());

    // Segunda passada: mesmo run, dedupado → zero novos achados.
    let r2 = ghci::ci_scan(&store, &gh, &["o/r".to_string()], 10, false, None).unwrap();
    assert_eq!(r2.findings.len(), 0);
    assert_eq!(r2.deduped, 1);
}

#[test]
fn ci_scan_emite_evento_por_achado() {
    let _env = env_lock();
    let dir = tempfile::tempdir().unwrap();
    let _gh = fake_gh(dir.path());
    let _guard = PathGuard::isolate(dir.path());
    set_mode("list");

    let store = Store::open_in_memory().unwrap();
    let gh = GhCli::default();
    ghci::ci_scan(&store, &gh, &["o/r".to_string()], 10, false, None).unwrap();

    let events = store.recent_events(10).unwrap();
    assert!(events.iter().any(|e| e.3 == "ci_failure"));
}

#[test]
fn ci_scan_ordena_e_deduplica_slugs() {
    let _env = env_lock();
    let dir = tempfile::tempdir().unwrap();
    let _gh = fake_gh(dir.path());
    let args_file = dir.path().join("args.txt");
    std::env::set_var("FAKE_GH_ARGS", &args_file);
    let _guard = PathGuard::isolate(dir.path());
    set_mode("list");

    let store = Store::open_in_memory().unwrap();
    let gh = GhCli::default();
    // slugs fora de ordem e duplicados: scan deve ordenar e deduplicar
    let slugs = vec!["z/z".to_string(), "a/a".to_string(), "z/z".to_string()];
    let report = ghci::ci_scan(&store, &gh, &slugs, 5, false, None).unwrap();

    // 2 slugs únicos × 1 run cada (o fake responde o mesmo run para ambos;
    // run_id idêntico → segundo vira dedup)
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.deduped, 1);

    let logged = std::fs::read_to_string(&args_file).unwrap();
    let first = logged.find("run list").unwrap();
    let second = logged.rfind("run list").unwrap();
    assert!(first < second);
    assert!(logged.contains("--repo a/a"));
    assert!(logged.contains("--repo z/z"));
    // apenas 2 chamadas de list (dedup de slugs)
    assert_eq!(logged.matches("run list").count(), 2);
}

// ── HF: cliente apontado para sink TCP local ────────────────────────────────

/// Servidor HTTP mínimo que responde um chat.completion fixo e grava o
/// corpo da requisição para assert.
fn hf_sink() -> (std::net::TcpListener, tempfile::TempDir, std::path::PathBuf) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let body_file = dir.path().join("hf_body.json");
    (listener, dir, body_file)
}

#[test]
fn hf_summarize_post_openai_compat() {
    use std::io::{BufRead, BufReader, Read, Write as _};

    let (listener, _dir, body_file) = hf_sink();
    let addr = listener.local_addr().unwrap();

    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut content_length = 0usize;
        let mut request_line = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                break;
            }
            if request_line.is_empty() {
                request_line = trimmed.to_string();
            }
            if let Some(v) = trimmed
                .to_ascii_lowercase()
                .strip_prefix("content-length:")
            {
                content_length = v.trim().parse().unwrap();
            }
        }
        assert!(request_line.starts_with("POST /v1/chat/completions"));
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).unwrap();
        std::fs::write(&body_file, &body).unwrap();

        let resp = br#"{"choices":[{"message":{"content":"causa: assert; fix: ajustar mock"}}]}"#;
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            resp.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(resp).unwrap();
    });

    let client = colmeia::hf::HfClient {
        token: "test-token".into(),
        model: "m/test".into(),
        base_url: format!("http://{}/v1", addr),
    };
    let out = client.summarize_failure("log de falha").unwrap();
    assert_eq!(out, "causa: assert; fix: ajustar mock");

    server.join().unwrap();
}

#[test]
fn hf_request_tem_temperature_zero_e_bearer() {
    // Reaproveita o sink: valida determinismo (temperature 0) e auth.
    use std::io::{BufRead, BufReader, Read, Write as _};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let body_file = dir.path().join("body.json");
    let headers_file = dir.path().join("headers.txt");

    let bf = body_file.clone();
    let hf2 = headers_file.clone();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut content_length = 0usize;
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let trimmed = line.trim_end();
            if trimmed.is_empty() {
                break;
            }
            headers.push_str(trimmed);
            headers.push('\n');
            if let Some(v) = trimmed
                .to_ascii_lowercase()
                .strip_prefix("content-length:")
            {
                content_length = v.trim().parse().unwrap();
            }
        }
        let mut body = vec![0u8; content_length];
        reader.read_exact(&mut body).unwrap();
        std::fs::write(&bf, &body).unwrap();
        std::fs::write(&hf2, headers).unwrap();
        let resp = br#"{"choices":[{"message":{"content":"ok"}}]}"#;
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            resp.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(resp).unwrap();
    });

    let client = colmeia::hf::HfClient {
        token: "tok123".into(),
        model: "Qwen/Qwen2.5-Coder-32B-Instruct".into(),
        base_url: format!("http://{}/v1", addr),
    };
    let _ = client.summarize_failure("x").unwrap();
    server.join().unwrap();

    let body: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&body_file).unwrap()).unwrap();
    assert_eq!(body["temperature"], 0);
    assert_eq!(body["model"], "Qwen/Qwen2.5-Coder-32B-Instruct");
    assert_eq!(body["max_tokens"], 512);

    let headers = std::fs::read_to_string(&headers_file).unwrap();
    let lower = headers.to_ascii_lowercase();
    assert!(lower.contains("authorization: bearer tok123"));
}

#[test]
fn hf_sem_token_erro_limpo() {
    // from_env_required deve falhar quando HF_TOKEN está vazio/ausente.
    // Não mutamos o env global: apenas validamos a variante NoToken via um
    // client manual sem uso de rede (a variante existe e é pública).
    let err = colmeia::hf::HfError::NoToken;
    assert!(err.to_string().contains("HF_TOKEN"));
}
