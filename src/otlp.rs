//! F53 — Exportação OTLP (JSON) opt-in para observabilidade externa.
//!
//! Quando `COLMEIA_OTLP_ENDPOINT` está definido, eventos de telemetria são
//! POSTados como OTLP/JSON (envelope `resourceLogs` com um `logRecord` por
//! evento). Sem a variável, tudo é silencioso — zero custo por padrão,
//! conforme o contrato de leveza.

/// Envia um evento (melhor esforço, nunca panica). No-op sem endpoint.
pub fn emit(event_name: &str, attrs: serde_json::Value) {
    let Ok(ep) = std::env::var("COLMEIA_OTLP_ENDPOINT") else {
        return;
    };
    if ep.is_empty() {
        return;
    }
    let now_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let body = serde_json::json!({
        "resourceLogs": [{
            "resource": {
                "attributes": [{
                    "key": "service.name",
                    "value": { "stringValue": "colmeia" }
                }]
            },
            "scopeLogs": [{
                "logRecords": [{
                    "timeUnixNano": now_ns.to_string(),
                    "severityText": "INFO",
                    "body": { "stringValue": event_name },
                    "attributes": serde_json::Value::Array(
                        attrs.as_object().map(|m| {
                            m.iter().map(|(k, v)| serde_json::json!({
                                "key": k,
                                "value": { "stringValue": v.to_string() }
                            })).collect()
                        }).unwrap_or_default()
                    )
                }]
            }]
        }]
    });
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(3))
        .timeout_read(std::time::Duration::from_secs(5))
        .build();
    // melhor esforço: ignora falhas silenciosamente (telemetria não pode
    // derrubar a operação principal)
    let _ = agent
        .post(&ep)
        .set("Content-Type", "application/json")
        .send_json(body);
}

#[cfg(test)]
mod unit {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Servidor OTLP mínimo que captura o POST e responde 200.
    fn otlp_sink() -> (String, std::sync::mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let mut acc = Vec::new();
                let mut buf = [0u8; 4096];
                // lê até o fim do corpo (ou timeout/EOF); o Content-Length
                // vem depois dos headers, então um read só não basta
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
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
            }
        });
        (format!("http://{addr}"), rx)
    }

    /// Teste único (evita corrida de env entre threads de teste paralelas).
    #[test]
    fn no_op_sem_endpoint_e_envelope_quando_configurado() {
        // 1) sem endpoint: no-op, sem pânico
        std::env::remove_var("COLMEIA_OTLP_ENDPOINT");
        emit("nao.vai", serde_json::json!({}));

        // 2) com endpoint local: envelope OTLP chega ao sink
        let (url, rx) = otlp_sink();
        std::env::set_var("COLMEIA_OTLP_ENDPOINT", &url);
        emit("merge.concluido", serde_json::json!({"repo": "x"}));
        std::env::remove_var("COLMEIA_OTLP_ENDPOINT");

        let body = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(body.starts_with("POST"), "esperava um POST: {body}");
        assert!(body.contains("resourceLogs"), "envelope OTLP ausente");
        assert!(body.contains("merge.concluido"));
    }
}
