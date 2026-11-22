//! F39/F40 — Plugins: packs Starlark + host JSON-RPC stdio com allowlist.
//!
//! Um plugin é um executável que fala JSON-RPC 2.0 por stdin/stdout.
//! O host (esta struct) inicia o processo, expõe apenas os métodos da
//! `allowlist` e encerra o plugin no drop. Regras declarativas de packs
//! Starlark (F39) são avaliadas localmente via `starlark::run_pack`.

use std::collections::HashSet;
use std::io::{BufRead, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Debug)]
pub enum PluginError {
    Spawn(String),
    Timeout,
    Protocol(String),
    NotAllowed(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Spawn(e) => write!(f, "spawn: {e}"),
            PluginError::Timeout => write!(f, "timeout aguardando resposta do plugin"),
            PluginError::Protocol(e) => write!(f, "protocolo: {e}"),
            PluginError::NotAllowed(m) => write!(f, "método '{m}' fora da allowlist"),
        }
    }
}
impl std::error::Error for PluginError {}

pub struct PluginHost {
    child: Child,
    stdin: ChildStdin,
    stdout: std::io::BufReader<ChildStdout>,
    allowlist: HashSet<String>,
    next_id: u64,
}

impl PluginHost {
    /// Inicia o plugin e devolve o host. `caps` é a allowlist de métodos.
    pub fn spawn(command: &str, args: &[&str], caps: HashSet<String>) -> Result<Self, PluginError> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| PluginError::Spawn(e.to_string()))?;
        let stdin = child.stdin.take().ok_or_else(|| PluginError::Spawn("sem stdin".into()))?;
        let stdout = child.stdout.take().ok_or_else(|| PluginError::Spawn("sem stdout".into()))?;
        Ok(Self {
            child,
            stdin,
            stdout: std::io::BufReader::new(stdout),
            allowlist: caps,
            next_id: 1,
        })
    }

    /// Chama um método permitido e aguarda a resposta (com prazo simples
    /// baseado em tentativas de leitura).
    pub fn request(
        &mut self,
        method: &str,
        params: serde_json::Value,
        timeout: std::time::Duration,
    ) -> Result<serde_json::Value, PluginError> {
        if !self.allowlist.contains(method) {
            return Err(PluginError::NotAllowed(method.to_string()));
        }
        let id = self.next_id;
        self.next_id += 1;
        let req = serde_json::json!({
            "jsonrpc": "2.0", "id": id, "method": method, "params": params
        });
        let line = serde_json::to_string(&req).map_err(|e| PluginError::Protocol(e.to_string()))?;
        self.stdin
            .write_all(line.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|e| PluginError::Spawn(e.to_string()))?;

        let deadline = std::time::Instant::now() + timeout;
        loop {
            let mut buf = String::new();
            match self.stdout.read_line(&mut buf) {
                Ok(0) => return Err(PluginError::Protocol("plugin encerrou o stdout".into())),
                Ok(_) => {
                    let msg: serde_json::Value = serde_json::from_str(buf.trim())
                        .map_err(|e| PluginError::Protocol(format!("resposta inválida: {e}")))?;
                    // ignora notificações e ids que não são os nossos
                    if msg["id"] != id {
                        continue;
                    }
                    if let Some(err) = msg.get("error") {
                        return Err(PluginError::Protocol(err.to_string()));
                    }
                    return Ok(msg["result"].clone());
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    if std::time::Instant::now() >= deadline {
                        return Err(PluginError::Timeout);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => return Err(PluginError::Spawn(e.to_string())),
            }
        }
    }
}

impl Drop for PluginHost {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    /// Plugin stub em shell: lê uma linha e responde com eco id/result.
    #[test]
    fn host_chama_metodo_permitido() {
        let script = r#"#!/bin/sh
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9]*\).*/\1/p')
  printf '{"jsonrpc":"2.0","id":%s,"result":{"ok":true}}\n' "$id"
done
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("plugin.sh");
        std::fs::write(&path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut caps = HashSet::new();
        caps.insert("scan".to_string());
        let mut host = PluginHost::spawn(
            "sh",
            &[path.to_str().unwrap()],
            caps,
        )
        .unwrap();
        let r = host
            .request("scan", serde_json::json!({"path": "x"}), std::time::Duration::from_secs(5))
            .unwrap();
        assert_eq!(r["ok"], true);
    }

    #[test]
    fn metodo_fora_da_allowlist_rejeitado_antes_de_falar_com_o_plugin() {
        // `/bin/cat` nunca responde JSON — se a chamada passasse, daria
        // erro de protocolo/timeout, então qualquer erro diferente de
        // NotAllowed("secrets.delete") prova o bloqueio antecipado.
        let mut caps = HashSet::new();
        caps.insert("scan".to_string());
        let mut host = PluginHost::spawn("cat", &[], caps).unwrap();
        let err = host
            .request("secrets.delete", serde_json::json!({}), std::time::Duration::from_millis(200))
            .unwrap_err();
        assert!(matches!(err, PluginError::NotAllowed(m) if m == "secrets.delete"));
    }
}
