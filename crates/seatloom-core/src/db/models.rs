/// Flat DB row models for PostgreSQL queries (tokio-postgres).
/// Covers schema 001-004: core objects, document authority, reconcile bookkeeping,
/// and operational review/continuity objects.
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub worker_budget_tokens: i32,
    pub supervisor_budget_tokens: i32,
}

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

// =============================================================================
// Schema 004: Operational review + continuity authority
// Mirrors infra/postgres/schema/004_operational_review_and_continuity.sql
// =============================================================================

/// Structured continuity snapshot for a session.
#[derive(Debug, Clone)]
pub struct CheckpointRow {
    pub id: String,
    pub session_id: String,
    pub trigger: String,
    pub summary_what_was_done: String,
    pub summary_current_state: String,
    pub summary_open_questions: Vec<String>,
    pub summary_quality: String,
    pub artifact_ids_at_checkpoint: Vec<String>,
    pub branch: Option<String>,
    pub last_commit: Option<String>,
    pub transcript_tail_ref: Option<String>,
    pub continuity_tier0: Option<serde_json::Value>,
    pub continuity_tier1: Option<serde_json::Value>,
    pub continuity_tier2: Option<serde_json::Value>,
    pub continuity_budget_tokens: Option<i32>,
    pub delta_context: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Explicit receipt/audit object for a handoff acknowledgment.
#[derive(Debug, Clone)]
pub struct HandoffReceiptRow {
    pub id: String,
    pub handoff_id: String,
    pub acknowledged_by: String,
    pub acknowledged_at: DateTime<Utc>,
    pub note: Option<String>,
    pub source_channel: String,
    pub created_at: DateTime<Utc>,
}

/// Deterministic execution/verification run history.
#[derive(Debug, Clone)]
pub struct PipelineRunRow {
    pub id: String,
    pub pipeline_id: String,
    pub workitem_id: Option<String>,
    pub status: String,
    pub current_stage: Option<String>,
    pub stage_index: Option<i32>,
    pub trigger: String,
    pub initiated_by: Option<String>,
    pub result_summary: Option<String>,
    pub artifact_ids: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub run_metadata: Option<serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Shared thread object for desktop review, supervisor assist, and mobile feedback.
#[derive(Debug, Clone)]
pub struct ReviewThreadRow {
    pub id: String,
    pub project_id: String,
    pub source_channel: String,
    pub mode: String,
    pub target_kind: String,
    pub target_id: String,
    pub document_id: Option<String>,
    pub artifact_id: Option<String>,
    pub workitem_id: Option<String>,
    pub handoff_id: Option<String>,
    pub session_id: Option<String>,
    pub anchor_kind: String,
    pub anchor_ref: Option<String>,
    pub anchor_label: Option<String>,
    pub title: Option<String>,
    pub status: String,
    pub requires_followup: bool,
    pub review_tier: Option<String>,
    pub change_tier_record: Option<serde_json::Value>,
    pub evidence_refs: Vec<String>,
    pub created_by: String,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
}

/// Ordered comment rows for a review thread.
#[derive(Debug, Clone)]
pub struct ReviewCommentRow {
    pub id: String,
    pub thread_id: String,
    pub parent_comment_id: Option<String>,
    pub author_ref: String,
    pub body_text: String,
    pub mode: String,
    pub source_channel: String,
    pub state: String,
    pub evidence_refs: Vec<String>,
    pub comment_metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Schema 005: Prompt + channel action authority
// Mirrors infra/postgres/schema/005_prompt_and_channel_action_authority.sql
// =============================================================================

/// Active or historical prompt-blocked instance for a session.
/// One row answers "what prompt is blocking this session right now?"
/// without rereading terminal logs. (INT-16 / US-P0-11)
#[derive(Debug, Clone)]
pub struct PromptInstanceRow {
    pub id: String,
    pub project_id: String,
    pub session_id: String,
    pub status: String, // 'active'|'resolved'|'stopped'|'expired'|'superseded'
    pub prompt_kind: String, // 'deterministic'|'wizard_menu'|'freeform'|'sensitive'
    pub prompt_policy: String, // 'auto_allowed'|'needs_approval'|'human_required'
    pub evidence_ref: Option<String>,
    pub evidence_preview: Option<String>,
    pub available_actions: Vec<String>,
    pub assist_max_steps: Option<i32>,
    pub assist_max_tokens: Option<i32>,
    pub assist_steps_used: i32,
    pub assist_tokens_used: i32,
    pub expected_next_pattern: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub result_event_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One resolution step in the prompt-action audit trail. (INT-16)
#[derive(Debug, Clone)]
pub struct PromptActionRow {
    pub id: String,
    pub prompt_id: String,
    pub action_kind: String, // 'approve'|'human_takeover'|'supervisor_assist'|'stop'|'input_injected'|'auto_completed'
    pub actor_ref: String,
    pub source_channel: String, // 'desktop'|'mobile'|'supervisor'|'system'
    pub note: Option<String>,
    pub steps_budget_used: Option<i32>,
    pub tokens_budget_used: Option<i32>,
    pub result_status: String, // 'applied'|'rejected'|'stopped'|'conflict'
    pub result_event_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Canonical receipt for a state-changing desktop/mobile/supervisor action.
/// Covers approve / reject / escalate / reserve-desktop-takeover / etc.
/// Unified across all target families to keep audit single-canonical.
/// (US-P0-13, US-P0-14, US-P0-15, INT-18, INT-19, INT-20)
#[derive(Debug, Clone)]
pub struct ChannelActionReceiptRow {
    pub id: String,
    pub project_id: String,
    pub target_kind: String, // 'workitem'|'handoff'|'review_thread'|'prompt'|'session'|'artifact'|'document'|'project'
    pub target_id: String,
    pub action_kind: String, // 'approve'|'reject'|'escalate'|'reserve_desktop_takeover'|'return'|'comment_submit'|'stop'
    pub actor_ref: String,
    pub source_channel: String, // 'desktop'|'mobile'|'supervisor'|'system'
    pub note: Option<String>,
    pub evidence_refs: Vec<String>,
    pub policy_summary: Option<String>,
    pub idempotency_key: String,
    pub expected_revision: Option<i32>,
    pub applied_revision: Option<i32>,
    pub receipt_status: String, // 'applied'|'rejected'|'conflicted'|'noop'
    pub result_event_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
