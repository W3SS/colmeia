use crate::{scanner, store::Store};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

const INDEX_HTML: &str = include_str!("../web/index.html");

pub fn run(store: Arc<Store>, bind: &str) -> anyhow::Result<()> {
    let server = tiny_http::Server::http(bind)
        .map_err(|e| anyhow::anyhow!("Falha ao iniciar servidor HTTP em {}: {}", bind, e))?;

    println!("Servidor Web da Colmeia iniciado em http://{}", bind);

    for mut req in server.incoming_requests() {
        if req.method() == &tiny_http::Method::Post && req.url() == "/api/scan" {
            let path = request_path(&mut req);
            let store = store.clone();
            let local_only = bind.starts_with("127.0.0.1:") || bind.starts_with("localhost:");
            std::thread::spawn(move || {
                let result = path.and_then(|path| {
                    anyhow::ensure!(local_only, "O mapeamento exige o servidor local");
                    Ok(serde_json::to_value(scanner::scan(&store, &path)?)?)
                });
                respond_json(req, result);
            });
            continue;
        }
        if req.method() == &tiny_http::Method::Post && req.url() == "/api/directories" {
            let result = (|| -> anyhow::Result<serde_json::Value> {
                anyhow::ensure!(
                    bind.starts_with("127.0.0.1:") || bind.starts_with("localhost:"),
                    "A seleção de pastas exige o servidor local"
                );
                let path = request_path(&mut req)?;
                let path = if path.is_empty() {
                    dirs::home_dir()
                        .ok_or_else(|| anyhow::anyhow!("Pasta pessoal indisponível"))?
                        .to_string_lossy()
                        .into_owned()
                } else {
                    path
                };
                Ok(serde_json::to_value(scanner::list_directories(&path)?)?)
            })();
            respond_json(req, result);
            continue;
        }
        if req.url().split('?').next().unwrap_or("/") == "/events" {
            let store = store.clone();
            std::thread::spawn(move || {
                serve_events(req, store);
            });
            continue;
        }
        let (status, ctype, body, extra_headers) = match req.url().split('?').next().unwrap_or("/")
        {
            "/" => (
                200,
                "text/html; charset=utf-8",
                INDEX_HTML.to_owned(),
                vec![("Cache-Control", "no-cache")],
            ),
            "/api/repos" => match store.list_repo_views() {
                Ok(repos) => (
                    200,
                    "application/json; charset=utf-8",
                    serde_json::to_string(&repos).unwrap_or_else(|_| "[]".into()),
                    vec![("Cache-Control", "no-store")],
                ),
                Err(error) => (500, "text/plain; charset=utf-8", error.to_string(), vec![]),
            },
            "/api/events" => match store.recent_events(50) {
                Ok(events) => (
                    200,
                    "application/json; charset=utf-8",
                    serde_json::to_string(&events).unwrap_or_else(|_| "[]".into()),
                    vec![("Cache-Control", "no-store")],
                ),
                Err(error) => (500, "text/plain; charset=utf-8", error.to_string(), vec![]),
            },
            "/api/jobs" => match store.recent_jobs(30) {
                Ok(jobs) => (
                    200,
                    "application/json; charset=utf-8",
                    serde_json::to_string(&jobs).unwrap_or_else(|_| "[]".into()),
                    vec![("Cache-Control", "no-store")],
                ),
                Err(error) => (500, "text/plain; charset=utf-8", error.to_string(), vec![]),
            },
            "/api/roots" => match store.list_scan_roots() {
                Ok(roots) => (
                    200,
                    "application/json; charset=utf-8",
                    serde_json::to_string(&roots).unwrap_or_else(|_| "[]".into()),
                    vec![("Cache-Control", "no-store")],
                ),
                Err(error) => (500, "text/plain; charset=utf-8", error.to_string(), vec![]),
            },
            _ => (
                404,
                "text/plain; charset=utf-8",
                "Não encontrado".into(),
                vec![],
            ),
        };

        let mut response = tiny_http::Response::from_string(body).with_status_code(status);
        if let Ok(header) = tiny_http::Header::from_bytes(b"Content-Type", ctype.as_bytes()) {
            response = response.with_header(header);
        }
        for (name, value) in extra_headers {
            if let Ok(header) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
                response = response.with_header(header);
            }
        }
        let _ = req.respond(response);
    }

    Ok(())
}

fn respond_json(req: tiny_http::Request, result: anyhow::Result<serde_json::Value>) {
    let (status, body) = match result {
        Ok(data) => (200, data.to_string()),
        Err(error) => (
            400,
            serde_json::json!({"error": error.to_string()}).to_string(),
        ),
    };
    let response = tiny_http::Response::from_string(body)
        .with_status_code(status)
        .with_header(
            tiny_http::Header::from_bytes(b"Content-Type", b"application/json; charset=utf-8")
                .unwrap(),
        );
    let _ = req.respond(response);
}

fn request_path(req: &mut tiny_http::Request) -> anyhow::Result<String> {
    let mut body = String::new();
    req.as_reader().take(4097).read_to_string(&mut body)?;
    anyhow::ensure!(body.len() <= 4096, "Pedido grande demais");
    let value: serde_json::Value = serde_json::from_str(&body)?;
    let path = value
        .get("path")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Informe o caminho da pasta"))?;
    Ok(path.to_owned())
}

fn serve_events(req: tiny_http::Request, store: Arc<Store>) {
    let mut writer = req.into_writer();
    if writer
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\nCache-Control: no-cache\r\nConnection: close\r\nX-Accel-Buffering: no\r\n\r\n: connected\n\n")
        .and_then(|_| writer.flush())
        .is_err()
    {
        return;
    }
    let mut last_id = store
        .recent_events(1)
        .ok()
        .and_then(|rows| rows.first().map(|row| row.0))
        .unwrap_or(0);
    let mut heartbeat = Instant::now();
    loop {
        let events = match store.events_after(last_id, 100) {
            Ok(events) => events,
            Err(_) => return,
        };
        if !events.is_empty() {
            for event in events {
                last_id = event.0;
                let Ok(data) = serde_json::to_string(&event) else {
                    return;
                };
                if writer
                    .write_all(
                        format!("id: {}\nevent: update\ndata: {}\n\n", event.0, data).as_bytes(),
                    )
                    .is_err()
                {
                    return;
                }
            }
            if writer.flush().is_err() {
                return;
            }
            heartbeat = Instant::now();
        } else if heartbeat.elapsed() >= Duration::from_secs(15) {
            if writer
                .write_all(b": keep-alive\n\n")
                .and_then(|_| writer.flush())
                .is_err()
            {
                return;
            }
            heartbeat = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
