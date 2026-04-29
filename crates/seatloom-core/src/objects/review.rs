use crate::objects::id::{
    ArtifactId, HandoffId, ReviewCommentId, ReviewThreadId, SessionId, WorkItemId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewThread {
    pub id: ReviewThreadId,
    pub project_id: String,
    pub source_channel: ReviewSourceChannel,
    pub mode: ReviewMode,
    pub target: ReviewTarget,
    pub anchor: ReviewAnchor,
    pub title: Option<String>,
    pub status: ReviewThreadStatus,
    pub requires_followup: bool,
    pub review_tier: Option<ReviewTier>,
    pub change_tier_record: Option<ChangeTierRecord>,
    pub evidence_refs: Vec<String>,
    pub created_by: String,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub id: ReviewCommentId,
    pub thread_id: ReviewThreadId,
    pub parent_comment_id: Option<ReviewCommentId>,
    pub author_ref: String,
    pub body_text: String,
    pub mode: ReviewMode,
    pub source_channel: ReviewSourceChannel,
    pub state: ReviewCommentState,
    pub evidence_refs: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewAnchor {
    pub kind: ReviewAnchorKind,
    pub reference: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTierRecord {
    pub tier: ReviewTier,
    pub reason: String,
    pub changed_clauses: Vec<String>,
    pub impact_level: String,
    pub executor: String,
    pub reviewer: String,
    pub ack_mode: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewSourceChannel {
    Desktop,
    Mobile,
    Supervisor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewMode {
    Manual,
    SupervisorAssisted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewThreadStatus {
    Open,
    Resolved,
    Disputed,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewCommentState {
    Active,
    Resolved,
    Disputed,
    Retracted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewTier {
    L1,
    L2,
    L3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewAnchorKind {
    Document,
    Section,
    TextRange,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewTarget {
    Artifact(ArtifactId),
    Document(String),
    WorkItem(WorkItemId),
    Handoff(HandoffId),
    Session(SessionId),
    Project(String),
}
