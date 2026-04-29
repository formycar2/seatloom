use crate::objects::id::{ArtifactId, CheckpointId, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: CheckpointId,
    pub session_id: SessionId,
    pub trigger: CheckpointTrigger,
    pub summary: CheckpointSummary,
    pub artifact_ids_at_checkpoint: Vec<ArtifactId>,
    pub branch: Option<String>,
    pub last_commit: Option<String>,
    pub transcript_tail_ref: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointTrigger {
    SessionEnded,
    ArtifactProduced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSummary {
    pub what_was_done: String,
    pub current_state: String,
    pub open_questions: Vec<String>,
    pub quality: SummaryQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SummaryQuality {
    Full,
    Minimal,
}
