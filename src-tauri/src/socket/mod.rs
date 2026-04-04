use crate::sessions::{HookEvent, PendingPermission, PendingQuestion, PermissionResponse};
use crate::SharedState;

use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
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
        #[cfg(unix)]
        {
            PathBuf::from("/tmp/vibe-island.sock")
        }
        #[cfg(windows)]
        {
            PathBuf::from(r"\\.\pipe\vibe-island")
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = Self::socket_path();

        #[cfg(unix)]
        {
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
                            if let Err(e) =
                                Self::handle_connection(stream, state, handle).await
                            {
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

        #[cfg(windows)]
        {
            // Windows named pipe server
            use tokio::net::windows::named_pipe::ServerOptions;
            let pipe_name = r"\\.\pipe\vibe-island";

            loop {
                let server = ServerOptions::new()
                    .first_pipe_instance(false)
                    .create(pipe_name)?;

                server.connect().await?;

                let state = self.state.clone();
                let handle = self.handle.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_pipe_connection(server, state, handle).await {
                        tracing::error!("Pipe connection error: {}", e);
                    }
                });
            }
        }
    }

    #[cfg(unix)]
    async fn handle_connection(
        stream: tokio::net::UnixStream,
        state: SharedState,
        handle: AppHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (reader, mut writer) = stream.into_split();
        let mut buf_reader = BufReader::new(reader);
        let mut data = String::new();

        // Read all data
        buf_reader.read_to_string(&mut data).await.ok();

        if data.is_empty() {
            return Ok(());
        }

        let event: HookEvent = match serde_json::from_str(&data) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("Invalid event JSON: {}", e);
                return Ok(());
            }
        };

        let event_name = event.hook_event_name.clone();
        let session_id = event.session_id.clone();

        // Handle the event
        let s = state.read().await;
        let needs_response = s.sessions.handle_event(event.clone()).await;
        drop(s);

        // Emit to frontend
        let _ = handle.emit("session-update", &session_id);

        // If this is a permission request, set up held connection
        if let Some(evt) = needs_response {
            if event_name == "PermissionRequest" {
                let is_question =
                    evt.tool_name.as_deref() == Some("AskUserQuestion");

                if is_question {
                    let (tx, rx) = oneshot::channel::<serde_json::Value>();
                    let pending = PendingQuestion {
                        request_id: evt.request_id.unwrap_or_default(),
                        questions: evt
                            .tool_input
                            .and_then(|v| v.get("questions").cloned())
                            .and_then(|v| serde_json::from_value(v).ok())
                            .unwrap_or_default(),
                        server_port: evt.server_port.unwrap_or(4096),
                        responder: Some(tx),
                    };

                    let s = state.read().await;
                    s.sessions
                        .set_pending_question(&session_id, pending)
                        .await;
                    drop(s);

                    let _ = handle.emit("question-asked", &session_id);

                    // Wait for response (held connection)
                    if let Ok(answers) = rx.await {
                        let response = serde_json::json!({
                            "hookSpecificOutput": {
                                "decision": {
                                    "updatedInput": {
                                        "answers": answers
                                    }
                                }
                            }
                        });
                        let response_str = serde_json::to_string(&response)?;
                        writer.write_all(response_str.as_bytes()).await?;
                    }
                } else {
                    let (tx, rx) = oneshot::channel::<PermissionResponse>();
                    let pending = PendingPermission {
                        request_id: evt.request_id.unwrap_or_default(),
                        tool_name: evt.tool_name.unwrap_or_default(),
                        tool_input: evt.tool_input.unwrap_or(serde_json::Value::Null),
                        server_port: evt.server_port.unwrap_or(4096),
                        responder: Some(tx),
                    };

                    let s = state.read().await;
                    s.sessions
                        .set_pending_permission(&session_id, pending)
                        .await;
                    drop(s);

                    let _ = handle.emit("permission-asked", &session_id);

                    // Wait for response (held connection)
                    if let Ok(resp) = rx.await {
                        let response = serde_json::json!({
                            "hookSpecificOutput": {
                                "decision": {
                                    "behavior": resp.behavior,
                                    "reason": resp.reason
                                }
                            }
                        });
                        let response_str = serde_json::to_string(&response)?;
                        writer.write_all(response_str.as_bytes()).await?;
                    }
                }
            }
        }

        Ok(())
    }
}
