use crate::sessions::{HookEvent, PendingPermission, PendingQuestion, PermissionResponse};
use crate::SharedState;

use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::oneshot;

#[cfg(unix)]
use tokio::net::UnixListener;

pub struct SocketServer {
    state: SharedState,
    handle: AppHandle,
}

impl SocketServer {
    pub fn new(state: SharedState, handle: AppHandle) -> Self {
        Self { state, handle }
    }

    fn socket_path() -> PathBuf {
        PathBuf::from("/tmp/vibe-island.sock")
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = Self::socket_path();

        // Remove stale socket
        let _ = std::fs::remove_file(&path);

        let listener = UnixListener::bind(&path)?;
        tracing::info!("Socket server listening on {}", path.display());

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let state = self.state.clone();
                    let handle = self.handle.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, state, handle).await {
                            tracing::error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                }
            }
        }
    }

    async fn handle_connection(
        stream: tokio::net::UnixStream,
        state: SharedState,
        handle: AppHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (mut reader, mut writer) = stream.into_split();
        let mut data = Vec::new();
        let mut buf = [0u8; 4096];

        // Read all data from the client
        loop {
            match reader.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => data.extend_from_slice(&buf[..n]),
                Err(e) => {
                    tracing::warn!("Read error: {}", e);
                    break;
                }
            }
            // If we have a complete JSON object, stop reading
            if serde_json::from_slice::<serde_json::Value>(&data).is_ok() {
                break;
            }
        }

        if data.is_empty() {
            return Ok(());
        }

        let event: HookEvent = match serde_json::from_slice(&data) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Invalid event JSON: {}", e);
                return Ok(());
            }
        };

        let session_id = event.session_id.clone();
        let is_question = event.tool_name.as_deref() == Some("AskUserQuestion");

        // Process the event
        let s = state.read().await;
        let needs_held = s.sessions.handle_event(&event).await;
        drop(s);

        // Emit update to frontend
        let _ = handle.emit("session-update", &session_id);

        // If permission/question request, set up held connection
        if needs_held {
            if is_question {
                let (tx, rx) = oneshot::channel::<serde_json::Value>();
                let pending = PendingQuestion {
                    request_id: event.request_id.unwrap_or_default(),
                    questions: event
                        .tool_input
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

                // Wait for response (held connection, 5 min timeout)
                match tokio::time::timeout(
                    std::time::Duration::from_secs(300),
                    rx,
                ).await {
                    Ok(Ok(answers)) => {
                        let response = serde_json::json!({
                            "hookSpecificOutput": {
                                "decision": {
                                    "updatedInput": { "answers": answers }
                                }
                            }
                        });
                        let _ = writer.write_all(serde_json::to_string(&response)?.as_bytes()).await;
                    }
                    _ => {} // Timeout or cancelled
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

                // Wait for response (held connection, 5 min timeout)
                match tokio::time::timeout(
                    std::time::Duration::from_secs(300),
                    rx,
                ).await {
                    Ok(Ok(resp)) => {
                        let response = serde_json::json!({
                            "hookSpecificOutput": {
                                "decision": {
                                    "behavior": resp.behavior,
                                    "reason": resp.reason
                                }
                            }
                        });
                        let _ = writer.write_all(serde_json::to_string(&response)?.as_bytes()).await;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
