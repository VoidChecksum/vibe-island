use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Manages SSH tunnel processes for remote session monitoring.
/// Each remote host gets a tunnel: remote socket → local forwarded socket.
pub struct SshRemoteManager {
    tunnels: Arc<Mutex<HashMap<String, TunnelState>>>,
}

pub struct TunnelState {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub local_socket: PathBuf,
    pub remote_socket: String,
    pub process: Child,
    pub status: TunnelStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TunnelStatus {
    Connecting,
    Connected,
    Failed,
    Disconnected,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteInfo {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub local_socket: String,
    pub status: TunnelStatus,
}

impl SshRemoteManager {
    pub fn new() -> Self {
        Self {
            tunnels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect(
        &self,
        host: String,
        user: String,
        port: u16,
        key_path: Option<String>,
        remote_socket: Option<String>,
    ) -> Result<RemoteInfo, String> {
        let local_socket = dirs::home_dir()
            .or_else(|| dirs::data_local_dir())
            .unwrap_or_else(|| PathBuf::from("."))
            .join(format!(".vibe-island/run/remote-{}.sock", sanitize_host(&host)));

        let _ = std::fs::remove_file(&local_socket);

        let remote_sock = remote_socket.unwrap_or_else(|| "~/.vibe-island/run/vibe-island.sock".into());
        let local_str = local_socket.to_string_lossy().to_string();
        let forward_arg = format!("{}:{}", local_str, remote_sock);

        let mut cmd = Command::new("ssh");
        cmd.args(["-N", "-o", "StrictHostKeyChecking=accept-new", "-o", "ExitOnForwardFailure=yes"]);
        if let Some(ref key) = key_path {
            cmd.args(["-i", key]);
        }
        cmd.args(["-p", &port.to_string(), "-L", &forward_arg]);
        cmd.arg(format!("{}@{}", user, host));

        let child = cmd.spawn().map_err(|e| format!("SSH spawn failed: {}", e))?;

        let info = RemoteInfo {
            host: host.clone(),
            user: user.clone(),
            port,
            local_socket: local_str,
            status: TunnelStatus::Connecting,
        };

        let state = TunnelState {
            host: host.clone(),
            user,
            port,
            local_socket,
            remote_socket: remote_sock,
            process: child,
            status: TunnelStatus::Connecting,
        };

        self.tunnels.lock().await.insert(host, state);
        Ok(info)
    }

    pub async fn disconnect(&self, host: &str) -> Result<(), String> {
        let mut tunnels = self.tunnels.lock().await;
        if let Some(mut state) = tunnels.remove(host) {
            let _ = state.process.kill();
            let _ = std::fs::remove_file(&state.local_socket);
        }
        Ok(())
    }

    pub async fn list(&self) -> Vec<RemoteInfo> {
        let mut tunnels = self.tunnels.lock().await;
        let mut infos = Vec::new();
        for (host, state) in tunnels.iter_mut() {
            let status = match state.process.try_wait() {
                Ok(Some(_)) => TunnelStatus::Failed,
                Ok(None) => TunnelStatus::Connected,
                Err(_) => TunnelStatus::Failed,
            };
            state.status = status.clone();
            infos.push(RemoteInfo {
                host: host.clone(),
                user: state.user.clone(),
                port: state.port,
                local_socket: state.local_socket.to_string_lossy().to_string(),
                status,
            });
        }
        infos
    }
}

fn sanitize_host(host: &str) -> String {
    host.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' }).collect()
}
