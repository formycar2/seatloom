/// Flat DB row models for PostgreSQL queries (tokio-postgres).
/// Covers schema 001 (core objects) and 002 (document authority layer).
use chrono::{DateTime, NaiveDate, Utc};

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

// =============================================================================
// Schema 002: Document authority layer
// Mirrors infra/postgres/schema/002_document_authority.sql
// =============================================================================

/// Full typed coordination document with universal header fields and body.
/// PostgreSQL is the canonical storage authority; disk files are evidence/export only.
#[derive(Debug, Clone)]
pub struct DocumentRow {
    pub id: String,
    pub project_id: String,
    pub artifact_id: Option<String>,
    // Universal header (DOCUMENT_TEMPLATES.md §2)
    pub template: Option<String>,
    pub subtype: Option<String>,
    pub subtype_valid: Option<bool>,
    pub doc_id: Option<String>,
    pub title: String,
    pub status: Option<String>,
    pub author: Option<String>,
    pub doc_date: Option<NaiveDate>,
    pub version: Option<String>,
    pub depends_on: Vec<String>,
    pub supersedes: Option<String>,
    pub tags: Vec<String>,
    // Body storage
    pub file_path: String,
    pub body_text: Option<String>,
    pub body_digest: Option<String>,
    pub body_length: Option<i32>,
    pub parse_status: String,
    // Optimistic concurrency
    pub revision: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Heading/anchor projection for a typed document.
#[derive(Debug, Clone)]
pub struct DocumentSectionRow {
    pub id: String,
    pub document_id: String,
    pub ordinal: i32,
    pub heading_text: String,
    pub heading_level: i32,
    pub anchor_slug: String,
    pub body_excerpt: Option<String>,
    pub search_text: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Many-to-many association between a document and a project object.
#[derive(Debug, Clone)]
pub struct DocumentAssociationRow {
    pub id: String,
    pub document_id: String,
    pub assoc_type: String, // 'workitem' | 'session' | 'handoff' | 'project'
    pub assoc_id: String,
    pub is_primary: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Schema 003: Write/ingest/reconcile bookkeeping
// Mirrors infra/postgres/schema/003_write_ingest_reconcile.sql
// =============================================================================

/// Aggregate record for one reconcile run.
#[derive(Debug, Clone)]
pub struct ReconcileRunRow {
    pub id: String,
    pub trigger: String, // 'startup' | 'pre_pipeline' | 'manual'
    pub status: String,  // 'running' | 'completed' | 'failed'
    pub scanned: i32,
    pub inserted: i32,
    pub updated: i32,
    pub unchanged: i32,
    pub failed: i32,
    pub conflicted: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Per-file outcome within a reconcile run.
#[derive(Debug, Clone)]
pub struct ReconcileItemRow {
    pub id: String,
    pub run_id: String,
    pub file_path: String,
    pub document_id: Option<String>,
    pub outcome: String, // 'inserted'|'updated'|'unchanged'|'failed'|'conflict'
    pub digest_before: Option<String>,
    pub digest_after: Option<String>,
    pub revision_before: Option<i32>,
    pub revision_after: Option<i32>,
    pub parse_status: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Durable revision snapshot for a document after a content change.
#[derive(Debug, Clone)]
pub struct DocumentVersionRow {
    pub id: String,
    pub document_id: String,
    pub revision: i32,
    pub body_digest: String,
    pub body_text: Option<String>,
    pub header_snapshot: Option<serde_json::Value>,
    pub run_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
