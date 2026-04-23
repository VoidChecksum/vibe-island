use crate::sessions::{HookEvent, PendingPermission, PendingQuestion, PermissionResponse};
use crate::SharedState;

use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

pub struct SocketServer {
    state: SharedState,
    handle: AppHandle,
}

impl SocketServer {
    pub fn new(state: SharedState, handle: AppHandle) -> Self {
        Self { state, handle }
    }

    pub fn socket_path() -> PathBuf {
        let run_dir = dirs::home_dir()
            .or_else(|| dirs::data_local_dir())
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".vibe-island/run");
        std::fs::create_dir_all(&run_dir).ok();
        run_dir.join("vibe-island.sock")
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(unix)]
        {
            self.start_unix().await
        }
        #[cfg(windows)]
        {
            self.start_windows().await
        }
        #[cfg(not(any(unix, windows)))]
        {
            tracing::warn!("IPC socket not supported on this platform");
            Ok(())
        }
    }

    // ── Unix (macOS + Linux) ──────────────────────────────────────────────────

    #[cfg(unix)]
    async fn start_unix(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::net::UnixListener;

        let path = Self::socket_path();
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        tracing::info!("Socket server listening on {}", path.display());

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let state = self.state.clone();
                    let handle = self.handle.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_unix(stream, state, handle).await {
                            tracing::error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => tracing::error!("Accept error: {}", e),
            }
        }
    }

    #[cfg(unix)]
    async fn handle_unix(
        stream: tokio::net::UnixStream,
        state: SharedState,
        handle: AppHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let (mut reader, mut writer) = stream.into_split();
        let mut data = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => data.extend_from_slice(&buf[..n]),
                Err(e) => { tracing::warn!("Read error: {}", e); break; }
            }
            if serde_json::from_slice::<serde_json::Value>(&data).is_ok() { break; }
        }

        if data.is_empty() { return Ok(()); }

        let event: HookEvent = match serde_json::from_slice(&data) {
            Ok(e) => e,
            Err(e) => { tracing::warn!("Invalid event JSON: {}", e); return Ok(()); }
        };

        let response = Self::process_event(event, state, &handle).await?;
        if let Some(bytes) = response {
            let _ = writer.write_all(&bytes).await;
        }
        Ok(())
    }

    // ── Windows (named pipe) ──────────────────────────────────────────────────

    #[cfg(windows)]
    async fn start_windows(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::net::windows::named_pipe::{PipeMode, ServerOptions};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let pipe_name = r"\\.\pipe\vibe-island";
        tracing::info!("Named pipe server listening on {}", pipe_name);

        loop {
            let server = ServerOptions::new()
                .pipe_mode(PipeMode::Byte)
                .in_buffer_size(65536)
                .out_buffer_size(65536)
                .create(pipe_name)?;

            server.connect().await?;

            let state = self.state.clone();
            let handle = self.handle.clone();
            tokio::spawn(async move {
                let mut data = Vec::new();
                let mut buf = [0u8; 4096];
                let (mut reader, mut writer) = tokio::io::split(server);

                loop {
                    match reader.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => data.extend_from_slice(&buf[..n]),
                        Err(e) => { tracing::warn!("Pipe read error: {}", e); break; }
                    }
                    if serde_json::from_slice::<serde_json::Value>(&data).is_ok() { break; }
                }

                if data.is_empty() { return; }

                let event: HookEvent = match serde_json::from_slice(&data) {
                    Ok(e) => e,
                    Err(e) => { tracing::warn!("Invalid pipe JSON: {}", e); return; }
                };

                if let Ok(Some(bytes)) = Self::process_event(event, state, &handle).await {
                    let _ = writer.write_all(&bytes).await;
                }
            });
        }
    }

    // ── Shared event processing ───────────────────────────────────────────────

    async fn process_event(
        event: HookEvent,
        state: SharedState,
        handle: &AppHandle,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let session_id = event.session_id.clone();
        let is_question = event.tool_name.as_deref() == Some("AskUserQuestion");

        let s = state.read().await;
        let needs_held = s.sessions.handle_event(&event).await;
        drop(s);

        let _ = handle.emit("session-update", &session_id);

        if !needs_held {
            return Ok(None);
        }

        if is_question {
            let (tx, rx) = oneshot::channel::<serde_json::Value>();
            let pending = PendingQuestion {
                request_id: event.request_id.unwrap_or_default(),
                questions: event
                    .tool_input
                    .as_ref()
                    .and_then(|v| v.get("questions").cloned())
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                server_port: event.server_port.unwrap_or(4096),
                responder: tx,
            };

            let s = state.read().await;
            s.sessions.set_pending_question(session_id.clone(), pending).await;
            drop(s);

            let _ = handle.emit("question-asked", &session_id);

            match tokio::time::timeout(std::time::Duration::from_secs(300), rx).await {
                Ok(Ok(answers)) => {
                    let resp = serde_json::json!({
                        "hookSpecificOutput": { "decision": { "updatedInput": { "answers": answers } } }
                    });
                    return Ok(Some(serde_json::to_string(&resp)?.into_bytes()));
                }
                _ => {}
            }
        } else {
            let (tx, rx) = oneshot::channel::<PermissionResponse>();
            let pending = PendingPermission {
                request_id: event.request_id.unwrap_or_default(),
                tool_name: event.tool_name.unwrap_or_default(),
                tool_input: event.tool_input.unwrap_or(serde_json::Value::Null),
                server_port: event.server_port.unwrap_or(4096),
                responder: tx,
            };

            let s = state.read().await;
            s.sessions.set_pending_permission(session_id.clone(), pending).await;
            drop(s);

            let _ = handle.emit("permission-asked", &session_id);

            match tokio::time::timeout(std::time::Duration::from_secs(300), rx).await {
                Ok(Ok(resp)) => {
                    let response = serde_json::json!({
                        "hookSpecificOutput": { "decision": { "behavior": resp.behavior, "reason": resp.reason } }
                    });
                    return Ok(Some(serde_json::to_string(&response)?.into_bytes()));
                }
                _ => {}
            }
        }

        Ok(None)
    }
}
