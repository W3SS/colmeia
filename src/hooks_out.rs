//! F38 — Hooks de saída: entrega de eventos para alvos externos.
//!
//! Dois tipos de alvo: `Exec` (processo local com o payload no stdin e
//! `COLMEIA_EVENT`/`COLMEIA_HOOK` no ambiente) e `Http` (POST JSON com
//! timeout). Cada entrega é registrada em `hook_deliveries`.

use crate::store::Store;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum HookTarget {
    Exec(Vec<String>),
    Http(String),
}

#[derive(Debug, PartialEq)]
pub enum DeliveryStatus {
    Delivered,
    Failed(String),
    TimedOut,
}

#[derive(Debug)]
pub struct Delivery {
    pub status: DeliveryStatus,
    pub detail: String,
}

/// Entrega um evento e registra o resultado no store.
pub fn deliver(
    store: &Store,
    hook: &str,
    target: &HookTarget,
    event: &str,
    payload: &serde_json::Value,
) -> anyhow::Result<Delivery> {
    let delivery = match target {
        HookTarget::Exec(argv) => deliver_exec(hook, argv, event, payload),
        HookTarget::Http(url) => deliver_http(hook, url, event, payload),
    };
    let (status_name, detail) = match &delivery.status {
        DeliveryStatus::Delivered => ("delivered", delivery.detail.clone()),
        DeliveryStatus::Failed(e) => ("failed", format!("{}: {}", delivery.detail, e)),
        DeliveryStatus::TimedOut => ("timeout", delivery.detail.clone()),
    };
    store.hook_delivery_insert(hook, event, status_name, &detail)?;
    Ok(delivery)
}

fn deliver_exec(
    hook: &str,
    argv: &[String],
    event: &str,
    payload: &serde_json::Value,
) -> Delivery {
    if argv.is_empty() {
        return Delivery {
            status: DeliveryStatus::Failed("argv vazio".into()),
            detail: String::new(),
        };
    }
    let body = serde_json::to_string(payload).unwrap_or_else(|_| "{}".into());
    let mut child = match Command::new(&argv[0])
        .args(&argv[1..])
        .env("COLMEIA_HOOK", hook)
        .env("COLMEIA_EVENT", event)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            return Delivery {
                status: DeliveryStatus::Failed(e.to_string()),
                detail: argv[0].clone(),
            }
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(body.as_bytes());
    }
    match child.wait_with_output() {
        Ok(out) if out.status.success() => Delivery {
            status: DeliveryStatus::Delivered,
            detail: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        },
        Ok(out) => Delivery {
            status: DeliveryStatus::Failed(format!("exit {}", out.status)),
            detail: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        },
        Err(e) => Delivery {
            status: DeliveryStatus::Failed(e.to_string()),
            detail: argv[0].clone(),
        },
    }
}

fn deliver_http(hook: &str, url: &str, event: &str, payload: &serde_json::Value) -> Delivery {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(10))
        .build();
    let body = serde_json::json!({
        "hook": hook,
        "event": event,
        "payload": payload,
    });
    match agent
        .post(url)
        .set("Content-Type", "application/json")
        .set("User-Agent", "colmeia/0.1")
        .send_json(body)
    {
        Ok(resp) => {
            let code = resp.status();
            Delivery {
                status: if (200..300).contains(&code) {
                    DeliveryStatus::Delivered
                } else {
                    DeliveryStatus::Failed(format!("HTTP {code}"))
                },
                detail: url.to_string(),
            }
        }
        Err(ureq::Error::Status(code, _)) => Delivery {
            status: DeliveryStatus::Failed(format!("HTTP {code}")),
            detail: url.to_string(),
        },
        Err(ureq::Error::Transport(t)) => {
            if t.to_string().contains("timed out") {
                Delivery {
                    status: DeliveryStatus::TimedOut,
                    detail: url.to_string(),
                }
            } else {
                Delivery {
                    status: DeliveryStatus::Failed(t.to_string()),
                    detail: url.to_string(),
                }
            }
        }
    }
}
