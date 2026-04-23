use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

/// Session data sent to the frontend (must be Clone + Serialize)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub source: String,
    pub status: SessionStatus,
    pub cwd: Option<String>,
    pub last_user_text: Option<String>,
    pub last_assistant_message: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub title: Option<String>,
    pub env: HashMap<String, String>,
    pub tty: Option<String>,
    pub terminal_bundle_id: Option<String>,
    pub server_port: Option<u16>,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub codex_model: Option<String>,
    pub codex_permission_mode: Option<String>,
    pub codex_thread_id: Option<String>,
    pub codex_title: Option<String>,
    pub subagent_parent_id: Option<String>,
    pub subagent_nickname: Option<String>,
    pub subagent_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Idle,
    Pending,
    InProgress,
    Completed,
    WaitingForApproval,
    WaitingForAnswer,
}

/// Held connection for permission approval (NOT cloneable, stored separately)
pub struct PendingPermission {
    pub request_id: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub server_port: u16,
    pub responder: oneshot::Sender<PermissionResponse>,
}

/// Held connection for question answering
pub struct PendingQuestion {
    pub request_id: String,
    pub questions: Vec<serde_json::Value>,
    pub server_port: u16,
    pub responder: oneshot::Sender<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub behavior: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    pub session_id: String,
    pub hook_event_name: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub last_assistant_message: Option<String>,
    #[serde(default)]
    pub codex_title: Option<String>,
    #[serde(rename = "_source", default)]
    pub source: Option<String>,
    #[serde(rename = "_ppid", default)]
    pub ppid: Option<u32>,
    #[serde(rename = "_env", default)]
    pub env: Option<HashMap<String, String>>,
    #[serde(rename = "_tty", default)]
    pub tty: Option<String>,
    #[serde(rename = "_server_port", default)]
    pub server_port: Option<u16>,
    #[serde(rename = "_opencode_request_id", default)]
    pub request_id: Option<String>,
}

/// Stores sessions and held connections separately
pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    pending_permissions: Arc<Mutex<HashMap<String, PendingPermission>>>,
    pending_questions: Arc<Mutex<HashMap<String, PendingQuestion>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            pending_permissions: Arc::new(Mutex::new(HashMap::new())),
            pending_questions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn list(&self) -> Vec<Session> {
        // Use try_lock for sync context, fall back to empty
        match self.sessions.try_lock() {
            Ok(sessions) => sessions.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    pub async fn list_async(&self) -> Vec<Session> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    /// Handle an inbound hook event. Returns true if this is a permission/question request
    /// that needs a held connection.
    fn make_session(event: &HookEvent) -> Session {
        Session {
            id: event.session_id.clone(),
            source: event.source.clone().unwrap_or_else(|| "claude".into()),
            status: SessionStatus::Active,
            cwd: event.cwd.clone(),
            last_user_text: None,
            last_assistant_message: None,
            tool_name: None,
            tool_input: None,
            title: None,
            env: event.env.clone().unwrap_or_default(),
            tty: event.tty.clone(),
            terminal_bundle_id: None,
            server_port: event.server_port,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            codex_model: None,
            codex_permission_mode: None,
            codex_thread_id: None,
            codex_title: None,
            subagent_parent_id: None,
            subagent_nickname: None,
            subagent_role: None,
        }
    }

    pub async fn handle_event(&self, event: &HookEvent) -> bool {
        let mut sessions = self.sessions.lock().await;

        // Auto-create session on first event if it doesn't exist yet.
        // Needed for tools (e.g. Claude Code) that don't fire a SessionStart hook.
        if event.hook_event_name != "SessionEnd"
            && !sessions.contains_key(&event.session_id)
        {
            sessions.insert(event.session_id.clone(), Self::make_session(event));
        }

        match event.hook_event_name.as_str() {
            "SessionStart" => {
                // Upsert: refresh fields in case session was auto-created earlier
                let s = sessions
                    .entry(event.session_id.clone())
                    .or_insert_with(|| Self::make_session(event));
                s.source = event.source.clone().unwrap_or_else(|| "claude".into());
                s.cwd = event.cwd.clone();
                s.env = event.env.clone().unwrap_or_default();
                s.tty = event.tty.clone();
                s.server_port = event.server_port;
                s.last_activity = Utc::now();
                false
            }

            "SessionEnd" => {
                sessions.remove(&event.session_id);
                false
            }

            "UserPromptSubmit" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.last_user_text = event.prompt.clone();
                    s.status = SessionStatus::InProgress;
                    s.last_activity = Utc::now();
                }
                false
            }

            "PreToolUse" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.tool_name = event.tool_name.clone();
                    s.tool_input = event.tool_input.clone();
                    s.status = SessionStatus::InProgress;
                    s.last_activity = Utc::now();
                }
                false
            }

            "PostToolUse" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.tool_name = None;
                    s.tool_input = None;
                    s.last_activity = Utc::now();
                }
                false
            }

            "PermissionRequest" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    if event.tool_name.as_deref() == Some("AskUserQuestion") {
                        s.status = SessionStatus::WaitingForAnswer;
                    } else {
                        s.status = SessionStatus::WaitingForApproval;
                    }
                    s.tool_name = event.tool_name.clone();
                    s.tool_input = event.tool_input.clone();
                    s.last_activity = Utc::now();
                }
                true // Needs held connection
            }

            "Stop" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.status = SessionStatus::Idle;
                    if let Some(msg) = &event.last_assistant_message {
                        s.last_assistant_message = Some(msg.clone());
                    }
                    if let Some(title) = &event.codex_title {
                        s.codex_title = Some(title.clone());
                        s.title = Some(title.clone());
                    }
                    s.tool_name = None;
                    s.tool_input = None;
                    s.last_activity = Utc::now();
                }
                false
            }

            "Notification" => {
                // Update last_activity so the session stays alive; no status change
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.last_activity = Utc::now();
                }
                false
            }

            _ => false,
        }
    }

    pub async fn set_pending_permission(&self, session_id: String, pending: PendingPermission) {
        self.pending_permissions.lock().await.insert(session_id, pending);
    }

    pub async fn set_pending_question(&self, session_id: String, pending: PendingQuestion) {
        self.pending_questions.lock().await.insert(session_id, pending);
    }

    pub async fn respond_permission(
        &self,
        session_id: &str,
        decision: &str,
        reason: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(pending) = self.pending_permissions.lock().await.remove(session_id) {
            let _ = pending.responder.send(PermissionResponse {
                behavior: decision.to_string(),
                reason: reason.map(String::from),
            });
        }
        // Update session status
        if let Some(s) = self.sessions.lock().await.get_mut(session_id) {
            s.status = SessionStatus::InProgress;
        }
        Ok(())
    }

    pub async fn respond_question(
        &self,
        session_id: &str,
        answers: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(pending) = self.pending_questions.lock().await.remove(session_id) {
            let _ = pending.responder.send(answers);
        }
        if let Some(s) = self.sessions.lock().await.get_mut(session_id) {
            s.status = SessionStatus::InProgress;
        }
        Ok(())
    }

    /// Remove sessions idle for longer than `timeout_secs`. Returns count removed.
    pub async fn cleanup_idle(&self, timeout_secs: u64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::seconds(timeout_secs as i64);
        let mut sessions = self.sessions.lock().await;
        let before = sessions.len();
        sessions.retain(|_, s| {
            // Keep if waiting for approval/answer (don't abandon pending requests)
            if s.status == SessionStatus::WaitingForApproval
                || s.status == SessionStatus::WaitingForAnswer
            {
                return true;
            }
            // Keep if active/in-progress (had recent activity)
            if s.last_activity > cutoff {
                return true;
            }
            false
        });
        before - sessions.len()
    }
}
