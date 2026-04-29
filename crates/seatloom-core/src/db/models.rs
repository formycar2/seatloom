/// Flat DB row models for PostgreSQL queries (tokio-postgres).
/// These mirror the schema in infra/postgres/schema/001_seatloom_core.sql.
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SeatRow {
    pub id: String,
    pub name: String,
    pub default_runtime: Option<String>,
    pub capability_tags: Vec<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ProjectRoleBindingRow {
    pub seat_id: String,
    pub project_id: String,
    pub role: String,
    pub authority_doc_refs: Vec<String>,
    pub constraints: Vec<String>,
    pub collaboration_template_ref: Option<String>,
    pub active_delegation_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SeatDelegationRow {
    pub id: String,
    pub issuer_seat_id: String,
    pub from_seat_id: String,
    pub to_seat_id: String,
    pub workitem_id: Option<String>,
    pub scope_description: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub id: String,
    pub seat_id: String,
    pub runtime: String,
    pub native_session_id: Option<String>,
    pub workspace_path: String,
    pub branch: Option<String>,
    pub status: String,
    pub launch_pack_ref: Option<String>,
    pub last_checkpoint_id: Option<String>,
    pub pid: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct WorkItemRow {
    pub id: String,
    pub title: String,
    pub goal: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub owner_seat_id: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HandoffRow {
    pub id: String,
    pub from_ref: String,
    pub to_ref: String,
    pub workitem_id: String,
    pub purpose: String,
    pub expected_outcome: String,
    pub required_receipt: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ArtifactRow {
    pub id: String,
    pub template: Option<String>,
    pub subtype: Option<String>,
    pub subtype_valid: Option<bool>,
    pub system_kind: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub source_session_id: Option<String>,
    pub source_workitem_id: Option<String>,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CanonicalEventRow {
    pub id: String,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub actor_ref: String,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
