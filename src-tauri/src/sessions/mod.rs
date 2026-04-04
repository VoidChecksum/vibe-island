use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

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
    // Codex-specific
    pub codex_model: Option<String>,
    pub codex_permission_mode: Option<String>,
    pub codex_thread_id: Option<String>,
    pub codex_title: Option<String>,
    // Subagent info
    pub subagent_parent_id: Option<String>,
    pub subagent_nickname: Option<String>,
    pub subagent_role: Option<String>,
    // Pending approval/question (not serialized to frontend)
    #[serde(skip)]
    pub pending_permission: Option<PendingPermission>,
    #[serde(skip)]
    pub pending_question: Option<PendingQuestion>,
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

#[derive(Debug)]
pub struct PendingPermission {
    pub request_id: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub server_port: u16,
    pub responder: Option<oneshot::Sender<PermissionResponse>>,
}

#[derive(Debug)]
pub struct PendingQuestion {
    pub request_id: String,
    pub questions: Vec<serde_json::Value>,
    pub server_port: u16,
    pub responder: Option<oneshot::Sender<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub behavior: String, // "allow", "always", "deny"
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

pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    peak_count: Arc<Mutex<usize>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            peak_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn list(&self) -> Vec<Session> {
        let sessions = self.sessions.blocking_lock();
        sessions.values().cloned().collect()
    }

    pub async fn list_async(&self) -> Vec<Session> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    pub async fn handle_event(&self, event: HookEvent) -> Option<HookEvent> {
        let mut sessions = self.sessions.lock().await;

        match event.hook_event_name.as_str() {
            "SessionStart" => {
                let session = Session {
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
                    pending_permission: None,
                    pending_question: None,
                };
                sessions.insert(event.session_id.clone(), session);

                let mut peak = self.peak_count.lock().await;
                *peak = (*peak).max(sessions.len());
            }

            "SessionEnd" => {
                sessions.remove(&event.session_id);
            }

            "UserPromptSubmit" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.last_user_text = event.prompt.clone();
                    s.status = SessionStatus::InProgress;
                    s.last_activity = Utc::now();
                }
            }

            "PreToolUse" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.tool_name = event.tool_name.clone();
                    s.tool_input = event.tool_input.clone();
                    s.status = SessionStatus::InProgress;
                    s.last_activity = Utc::now();
                }
            }

            "PostToolUse" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.tool_name = None;
                    s.tool_input = None;
                    s.last_activity = Utc::now();
                }
            }

            "PermissionRequest" => {
                if let Some(s) = sessions.get_mut(&event.session_id) {
                    s.status = SessionStatus::WaitingForApproval;
                    s.tool_name = event.tool_name.clone();
                    s.tool_input = event.tool_input.clone();
                    s.last_activity = Utc::now();

                    // Check if this is an AskUserQuestion
                    if event.tool_name.as_deref() == Some("AskUserQuestion") {
                        s.status = SessionStatus::WaitingForAnswer;
                    }
                }
                // Return event so socket handler can set up held connection
                return Some(event);
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
            }

            _ => {}
        }

        None
    }

    pub async fn respond_permission(
        &self,
        session_id: &str,
        decision: &str,
        reason: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.sessions.lock().await;
        if let Some(s) = sessions.get_mut(session_id) {
            if let Some(mut pending) = s.pending_permission.take() {
                if let Some(tx) = pending.responder.take() {
                    let _ = tx.send(PermissionResponse {
                        behavior: decision.to_string(),
                        reason: reason.map(String::from),
                    });
                }
                s.status = SessionStatus::InProgress;
            }
        }
        Ok(())
    }

    pub async fn respond_question(
        &self,
        session_id: &str,
        answers: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut sessions = self.sessions.lock().await;
        if let Some(s) = sessions.get_mut(session_id) {
            if let Some(mut pending) = s.pending_question.take() {
                if let Some(tx) = pending.responder.take() {
                    let _ = tx.send(answers);
                }
                s.status = SessionStatus::InProgress;
            }
        }
        Ok(())
    }

    pub async fn set_pending_permission(
        &self,
        session_id: &str,
        pending: PendingPermission,
    ) {
        let mut sessions = self.sessions.lock().await;
        if let Some(s) = sessions.get_mut(session_id) {
            s.pending_permission = Some(pending);
        }
    }

    pub async fn set_pending_question(
        &self,
        session_id: &str,
        pending: PendingQuestion,
    ) {
        let mut sessions = self.sessions.lock().await;
        if let Some(s) = sessions.get_mut(session_id) {
            s.pending_question = Some(pending);
        }
    }
}
