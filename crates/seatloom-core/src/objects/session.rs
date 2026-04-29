use crate::objects::id::{CheckpointId, SeatId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub seat_id: SeatId,
    pub runtime: Runtime,
    pub native_session_id: Option<String>,
    pub workspace_path: PathBuf,
    pub branch: Option<String>,
    pub status: SessionStatus,
    /// Path relative to .seatloom/ for the Worker continuity pack.
    pub launch_pack_ref: Option<String>,
    pub last_checkpoint_id: Option<CheckpointId>,
    pub pid: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Runtime {
    ClaudeCode,
    Codex,
    CursorCli,
    GeminiCli,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Launching,
    Running,
    /// Wrapped runtime is blocked on interactive stdin (AD-012).
    /// Distinct from InputRequired (SeatLoom-level request).
    PromptBlocked,
    InputRequired,
    Suspended,
    Completed,
    Failed,
    Interrupted,
}

/// Active prompt state when SessionStatus == PromptBlocked (AD-012).
/// bounded_window is bounded to last 10-20 terminal lines — never full session history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptState {
    pub kind: PromptKind,
    pub policy: PromptPolicy,
    pub bounded_window: Vec<String>,
    pub available_actions: Vec<PromptAction>,
    pub assist_budget: Option<AssistBudget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptKind {
    Deterministic,
    WizardMenu,
    Freeform,
    /// Password / OTP / sudo / secret — SupervisorAssist is disabled at architecture level.
    Sensitive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptPolicy {
    /// SeatLoom may inject without asking (Deterministic + config permit).
    AutoAllowed,
    /// Requires explicit user confirmation before inject.
    NeedsApproval,
    /// Human must type directly; no automation permitted.
    HumanRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromptAction {
    Approve,
    HumanTakeover,
    /// Disabled when PromptKind == Sensitive.
    SupervisorAssist,
    Stop,
}

/// Hard caps on Supervisor assist loops (AD-012). Exceed either limit → force HumanTakeover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistBudget {
    pub max_steps: u32,
    pub max_tokens: u32,
    pub steps_used: u32,
    pub tokens_used: u32,
}
