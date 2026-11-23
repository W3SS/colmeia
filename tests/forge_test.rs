//! F32/F34 — Testes de integração dos adaptadores de forge com servidor local.


#![cfg(unix)] // hook de execução usa script shell: Unix-only
use colmeia::forge::{ForgeAdapter, ForgeError, GitHubAdapter, GitLabLikeAdapter};
use colmeia::store::Store;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver};

/// Servidor HTTP mínimo: responde `body` com `status` para a primeira
/// requisição e captura a requisição recebida.
fn http_sink(status: &str, body: &str) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = channel();
    let status = status.to_string();
    let body = body.to_string();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
            let mut acc = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        acc.extend_from_slice(&buf[..n]);
                        let text = String::from_utf8_lossy(&acc);
                        if let Some(header_end) = text.find("\r\n\r\n") {
                            let headers = &text[..header_end];
                            let len: usize = headers
                                .lines()
                                .find_map(|l| {
                                    l.to_ascii_lowercase()
                                        .strip_prefix("content-length:")?
                                        .trim()
                                        .parse()
                                        .ok()
                                })
                                .unwrap_or(0);
                            if acc.len() >= header_end + 4 + len {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = tx.send(String::from_utf8_lossy(&acc).to_string());
            let resp = format!(
                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    (format!("http://{addr}"), rx)
}

#[test]
fn github_cria_pr() {
    let (url, rx) = http_sink(
        "201 Created",
        r#"{"number":42,"title":"bump","state":"open","html_url":"https://example/pr/42"}"#,
    );
    let gh = GitHubAdapter::with_api_base("token-teste", url);
    let pr = gh
        .create_pr("org/repo", "bump", "corpo", "colmeia/bump-x-1", "main")
        .unwrap();
    assert_eq!(pr.number, 42);
    assert_eq!(pr.url, "https://example/pr/42");

    let req = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(req.starts_with("POST /repos/org/repo/pulls"), "req: {req}");
    assert!(req.contains("Authorization: Bearer token-teste"));
    assert!(req.contains("\"head\":\"colmeia/bump-x-1\""));
}

#[test]
fn github_lista_prs_abertos() {
    let (url, _rx) = http_sink(
        "200 OK",
        r#"[{"number":1,"title":"a","state":"open","html_url":"u1"},
            {"number":2,"title":"b","state":"open","html_url":"u2"}]"#,
    );
    let gh = GitHubAdapter::with_api_base("t", url);
    let prs = gh.list_open_prs("org/repo").unwrap();
    assert_eq!(prs.len(), 2);
    assert_eq!(prs[1].number, 2);
}

#[test]
fn github_403_vira_rate_limited_com_retry_after() {
    let (url, _rx) = http_sink(
        "403 Forbidden",
        r#"{"message":"API rate limit exceeded"}"#,
    );
    let gh = GitHubAdapter::with_api_base("t", url);
    let err = gh.list_open_prs("org/repo").unwrap_err();
    match err {
        ForgeError::RateLimited { retry_after_secs } => assert!(retry_after_secs >= 60),
        other => panic!("esperava RateLimited, veio {other:?}"),
    }
}

#[test]
fn gitlab_cria_mr_com_source_branch() {
    let (url, rx) = http_sink(
        "201 Created",
        r#"{"iid":7,"title":"mr","state":"opened","web_url":"https://gl/mr/7"}"#,
    );
    let gl = GitLabLikeAdapter::gitlab("t", url);
    let pr = gl
        .create_pr("grp/proj", "mr", "corpo", "feature-x", "main")
        .unwrap();
    assert_eq!(pr.number, 7);
    let req = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(req.contains("source_branch"), "req: {req}");
    assert!(req.contains("PRIVATE-TOKEN: t"));
}

#[test]
fn forgejo_cria_pr() {
    let (url, rx) = http_sink(
        "201 Created",
        r#"{"number":3,"title":"pr","state":"open","html_url":"https://fj/pr/3"}"#,
    );
    let fj = GitLabLikeAdapter::forgejo("t", url);
    let pr = fj.create_pr("o/r", "pr", "c", "head-x", "main").unwrap();
    assert_eq!(pr.number, 3);
    let req = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(req.contains("/repos/o/r/pulls"), "req: {req}");
}

#[test]
fn rate_budget_bloqueia_apos_cap() {
    let store = Store::open_in_memory().unwrap();
    use colmeia::forge::rate_gate;
    assert!(rate_gate(&store, "github", 3).is_ok());
    assert!(rate_gate(&store, "github", 3).is_ok());
    assert!(rate_gate(&store, "github", 3).is_ok());
    // 4ª chamada na mesma janela excede o cap de 3
    assert!(matches!(
        rate_gate(&store, "github", 3),
        Err(ForgeError::RateLimited { .. })
    ));
    // orçamento é independente por forge
    assert!(rate_gate(&store, "gitlab", 3).is_ok());
}

#[test]
fn hooks_out_exec_entrega_e_registra() {
    let store = Store::open_in_memory().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("hook.sh");
    std::fs::write(&script, "#!/bin/sh\ncat > /dev/null\nprintf 'ok-exec'\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let d = colmeia::hooks_out::deliver(
        &store,
        "meu-hook",
        &colmeia::hooks_out::HookTarget::Exec(vec![script.to_str().unwrap().to_string()]),
        "merge.done",
        &serde_json::json!({"a": 1}),
    )
    .unwrap();
    assert_eq!(d.status, colmeia::hooks_out::DeliveryStatus::Delivered);
    assert_eq!(d.detail, "ok-exec");

    let log = store.hook_deliveries(10).unwrap();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].0, "meu-hook");
    assert_eq!(log[0].1, "merge.done");
    assert_eq!(log[0].2, "delivered");
}

#[test]
fn hooks_out_http_entrega() {
    let store = Store::open_in_memory().unwrap();
    let (url, rx) = http_sink("200 OK", "{}");
    let d = colmeia::hooks_out::deliver(
        &store,
        "web-hook",
        &colmeia::hooks_out::HookTarget::Http(url),
        "slice.done",
        &serde_json::json!({"recipe": 9}),
    )
    .unwrap();
    assert_eq!(d.status, colmeia::hooks_out::DeliveryStatus::Delivered);
    let req = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(req.contains("POST /"), "req: {req}");
    assert!(req.contains("\"event\":\"slice.done\""));
    assert!(req.contains("\"recipe\":9"));
}

#[test]
fn webhooks_integracao_assina_e_verifica() {
    use colmeia::secretbox::{derive_key, vault_get, vault_set};
    use colmeia::webhooks;

    let store = Store::open_in_memory().unwrap();
    vault_set(&store, "webhook-secret", "segredo-compartilhado", "p").unwrap();
    let secret = vault_get(&store, "webhook-secret", "p").unwrap().unwrap();

    let body = br#"{"ref":"refs/heads/main"}"#;
    let sig = webhooks::sign(secret.as_bytes(), body);
    assert!(webhooks::verify(secret.as_bytes(), body, &sig));

    // chave derivada errada rejeita
    let other = derive_key("outra-pass", &[0u8; 16]);
    assert!(!webhooks::verify(&other, body, &sig));
}
