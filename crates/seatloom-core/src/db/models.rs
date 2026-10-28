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
    /// The project's home path, NULL when the corpus never recorded one (an
    /// archive-derived project has no home). Carried so the UI can show the real
    /// path instead of synthesising one — see `projectFromDto`, which for a while
    /// rendered `'~/' + name` for every project.
    pub home_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SeatRow {
    pub id: String,
    pub name: String,
    /// Runtime triple, mirrored verbatim from `seats.default_harness` /
    /// `default_agent` / `default_model` (schema 009). String passthrough: the
    /// row struct stays a faithful mirror of the SQL columns. Values are
    /// validated against `Harness` / `Agent` at the hydration boundary in
    /// `db::repositories::row_to_seat`.
    pub default_harness: Option<String>,
    pub default_agent: Option<String>,
    pub default_model: Option<String>,
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
    /// Runtime triple, mirrored verbatim from `sessions.harness` / `agent` /
    /// `model` (schema 009). `harness` and `model` are nullable — `NULL` means
    /// *not recorded*, never *direct* / *default*. Validated against
    /// `Harness` / `Agent` at the hydration boundary in
    /// `db::repositories::row_to_session`.
    pub harness: Option<String>,
    pub agent: String,
    pub model: Option<String>,
    pub native_session_id: Option<String>,
    pub workspace_path: String,
    pub branch: Option<String>,
    pub status: String,
    pub launch_pack_ref: Option<String>,
    pub last_checkpoint_id: Option<String>,
    pub pid: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub project_id: String,
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
    pub project_id: String,
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
    pub project_id: String,
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
    pub project_id: String,
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

// =============================================================================
// Schema 012: Governed dispatch (G19)
// Mirrors infra/postgres/schema/012_governed_dispatch.sql
// =============================================================================

/// A per-seat dispatch (the governed-dispatch inbox row). Read model for the
/// dispatch-inbox UI. One row = one task routed to a seat.
#[derive(Debug, Clone)]
pub struct DispatchRow {
    pub id: String,
    pub target_seat_id: String,
    pub issued_by: String,
    pub project_id: String,
    pub title: String,
    pub payload: serde_json::Value,
    pub status: String, // 'ready'|'blocked'|'in_flight'|'done'
    pub priority: i32,
    pub issued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// One archived session, as the history read path returns it.
///
/// Mirrors `archived_sessions` (011) plus the three corpus columns `021` adds.
/// The three corpus columns are `Option` because a database without `021` has
/// them absent, and absent must read as "not loaded" rather than as zero.
#[derive(Debug, Clone)]
pub struct ArchivedSessionRow {
    pub id: String,
    pub harness: String,
    pub project: Option<String>,
    pub cwd: Option<String>,
    pub session_date: Option<String>,
    pub bytes: Option<i64>,
    /// INTEGER (INT4), so `i32` — tokio-postgres does not coerce INT4 into i64.
    pub lines: Option<i32>,
    pub sha256: Option<String>,
    pub raw_path: String,
    pub src_path: Option<String>,
    /// `021`. INTEGER (INT4) → `i32`. `None` when the column does not exist.
    pub message_count: Option<i32>,
    /// `021`.
    pub first_ts: Option<String>,
    /// `021`.
    pub last_ts: Option<String>,
}

/// One project row joined to its session and message totals.
#[derive(Debug, Clone)]
pub struct HistoryProjectRow {
    pub id: String,
    pub name: String,
    pub home_path: Option<String>,
    pub session_count: i64,
    /// `None` when `archived_sessions.message_count` does not exist.
    pub message_count: Option<i64>,
}

/// One dependency edge of a dispatch (readiness is an existence query over the
/// referent; no FK, per 012).
#[derive(Debug, Clone)]
pub struct DispatchDependencyRow {
    pub dispatch_id: String,
    pub dep_kind: String, // 'event'|'artifact'|'dispatch'
    pub dep_value: String,
}

/// One memory row (schema 015, D5 knowledge layer) — a small-granularity
/// distilled knowledge unit scoped by project + topic (the ONLY scoping
/// framework, contract's unchanged ruling).
///
/// The three layers are progressive disclosure: `l0_summary` is the ONLY default
/// (~100 tokens), `l1_overview` is fetched on demand (~2K tokens), and the L2
/// pointer (`l2_ref_kind` + `l2_document_id` | `l2_file_path`) leads to the raw
/// material. Large refined products stay in `documents` (002) — memories point
/// at them, they do not fork that model.
#[derive(Debug, Clone)]
pub struct MemoryRow {
    pub id: String,
    pub project_id: String,
    pub topic_id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub l0_summary: String,
    pub l1_overview: Option<String>,
    pub l2_ref_kind: String, // 'document' | 'file'
    pub l2_document_id: Option<String>,
    pub l2_file_path: Option<String>,
    pub l2_body_digest: Option<String>,
    /// 'archived_session' | 'canonical_event' | 'document' | 'manual'.
    pub source_kind: String,
    /// Polymorphic provenance reference, NO FK (the 010 ref-fabric reasoning):
    /// `'<harness>:<id>'` | event id | doc id | actor note.
    pub source_ref: String,
    /// Record type (018): 'decision' | 'failure' | 'fact' | 'preference' |
    /// 'artifact' | 'unresolved'.
    pub kind: String,
    /// 'quoted' | 'inferred' (018) — binary, never a score.
    pub confidence: String,
    /// The memory this one supersedes (018; the same edge as a
    /// `memory_relations` row with rel='supersedes'). Create-time only.
    pub supersedes: Option<String>,
    /// First line of the L2 evidence inside `l2_file_path` (1-based). REQUIRED
    /// when l2_ref_kind='file' AND source_kind='archived_session' (018).
    pub l2_line_start: Option<i32>,
    /// Last line of the L2 evidence (>= l2_line_start).
    pub l2_line_end: Option<i32>,
    /// +1 per recall. NO automatic decay, ever.
    pub heat: i32,
    pub recall_count: i32,
    /// NULL = never recalled.
    pub last_recalled_at: Option<DateTime<Utc>>,
    /// 'cold' | 'warm' | 'hot'. Transitions ONLY via the promote tool.
    pub tier: String,
    pub tier_changed_at: DateTime<Utc>,
    /// 'active' | 'forgotten' | 'resolved' (018 widened it). forgotten is a SOFT
    /// tombstone (reason required, rows never hard-deleted); resolved is the G3
    /// closing state for 'unresolved' records, closed ONLY by the resolve tool.
    pub status: String,
    pub forgotten_reason: Option<String>,
    /// Who closed the record (018; set by the resolve tool, never a parameter).
    pub resolved_by: Option<String>,
    /// When it was closed (018).
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One row of `memory_relations` (018): a typed edge between two memories.
#[derive(Debug, Clone)]
pub struct MemoryRelationRow {
    pub from_id: String,
    pub to_id: String,
    /// 'supersedes' | 'caused_by' | 'fixed_by' | 'refines'.
    pub rel: String,
}

/// One topic row (010) as the lineage assembler needs it: the ancestor chain of
/// a recall's topic, one entry per level.
#[derive(Debug, Clone)]
pub struct TopicRow {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: String,
    pub parent_topic_id: Option<String>,
}

/// One node of the knowledge topic tree (overnight step 4): the topic, its
/// parent, and how many ACTIVE, not-superseded memories it holds — the count a
/// recall would actually return.
#[derive(Debug, Clone)]
pub struct KnowledgeTopicNode {
    pub topic_id: String,
    pub title: String,
    /// 'open' | 'closed' | 'merged'.
    pub status: String,
    pub parent_topic_id: Option<String>,
    pub memory_count: i64,
}

/// The L2 evidence pointer of a memory, returned only at depth l2.
#[derive(Debug, Clone)]
pub struct KnowledgeL2Pointer {
    /// 'document' | 'file'.
    pub ref_kind: String,
    pub document_id: Option<String>,
    pub file_path: Option<String>,
    pub body_digest: Option<String>,
    /// Present when the source is a slice of an archived session transcript.
    pub line_start: Option<i32>,
    pub line_end: Option<i32>,
}

/// One memory as the knowledge panel renders it. `l1_overview` and `l2` are
/// `None` below their disclosure depth — the caller asks for more, the shape
/// carries more.
#[derive(Debug, Clone)]
pub struct KnowledgeMemoryCard {
    pub memory_id: String,
    pub topic_id: String,
    pub title: String,
    /// 'decision' | 'failure' | 'fact' | 'preference' | 'artifact' | 'unresolved'.
    pub kind: String,
    /// 'quoted' | 'inferred'.
    pub confidence: String,
    /// 'cold' | 'warm' | 'hot'.
    pub tier: String,
    /// 'active' | 'forgotten' | 'resolved'.
    pub status: String,
    pub l0_summary: String,
    pub l1_overview: Option<String>,
    pub l2: Option<KnowledgeL2Pointer>,
    /// The memory that replaced this one, when it has been superseded.
    pub superseded_by: Option<String>,
    pub source_kind: String,
    pub source_ref: String,
}

/// A dispatch with its LIVE status and dependency edges — the `get_dispatch`
/// read tool's answer. `live_status` is recomputed by the same existence rule the
/// pull decision uses, so a reader sees what the scheduler would see.
#[derive(Debug, Clone)]
pub struct DispatchReadModel {
    pub dispatch: DispatchRow,
    pub live_status: String,
    pub dependencies: Vec<DispatchDependencyStatus>,
}

/// What a Flux verdict reference actually says, and whether it is about a given
/// dispatch — the two questions `event_or_artifact_exists` was being asked
/// instead of the one it answers.
///
/// Field order is the reading order of the gate that consumes it: is it there,
/// is it green, is it about this dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictFacts {
    /// The referent names a row in `canonical_events` or `artifacts`.
    pub exists: bool,
    /// Its `event_type` when it is an event. `None` for an artifact, which has
    /// none — never defaulted to `ReviewVerdictIssued`.
    pub event_type: Option<String>,
    /// `payload->>'verdict'`, verbatim. `None` when absent, null, or the referent
    /// is an artifact. An absent value is an absence, not a fail and not a pass.
    pub verdict: Option<String>,
    /// `payload->>'dispatch_id'`, verbatim, when the verdict states one.
    pub payload_dispatch_id: Option<String>,
    /// Whether it names the dispatch under test — via `payload->>'dispatch_id'`
    /// or via an `event_object_refs` row with `ref_type='dispatch'`. Either
    /// channel counts; both are things Flux actually writes.
    pub names_dispatch: bool,
}

impl VerdictFacts {
    /// The gate's WHOLE question: does this verdict let the work be merged?
    ///
    /// Green means exactly `pass`. Measured on live, the values in use are
    /// `pass` (27), `fail` (2) and `reject` (1) — so a `reject` is not green
    /// either, and an absent value is not a pass.
    ///
    /// And it must be about the dispatch under test. `names_dispatch` was
    /// computed by the reader for the dispatch the caller asked about, so this
    /// ANDs it in rather than deriving it again — a second derivation would be a
    /// second criterion that could drift from the first.
    ///
    /// # Why there is no `is_green()`
    ///
    /// There was one, and the proof caught what it invited: a method answering
    /// only the value half is a method a future caller will use as the whole
    /// criterion, which is precisely the defect this read exists to close. The
    /// callers that need the two halves separately — to render two different
    /// refusals — read the fields directly, which is honest about being partial.
    pub fn is_merge_criterion(&self) -> bool {
        self.exists
            && self.event_type.as_deref() == Some("ReviewVerdictIssued")
            && self.verdict.as_deref() == Some("pass")
            && self.names_dispatch
    }
}

/// One dependency edge with its referent's live existence.
#[derive(Debug, Clone)]
pub struct DispatchDependencyStatus {
    pub dep_kind: String,
    pub dep_value: String,
    pub exists: bool,
}

/// One ledger event with its object refs — the `query_ledger` read tool's answer.
#[derive(Debug, Clone)]
pub struct LedgerEventWithRefs {
    pub event: CanonicalEventRow,
    pub refs: Vec<ObjectRefRow>,
}

/// One row of `event_object_refs` (001): how an event names an object.
/// `role` was added by 010 and is NOT NULL there; the lineage assembler reads
/// refs by event id, so the role rides along for callers that need it.
#[derive(Debug, Clone)]
pub struct ObjectRefRow {
    pub event_id: String,
    pub ref_type: String, // workitem|session|handoff|artifact|document|seat|topic|project|delegation|dispatch|acceptance
    pub ref_id: String,
    pub role: String, // 'subject'|'actor'|'context'
}

/// One acceptance row (schema 014, D2 acceptance gate) — the UI's 待验收 card.
/// Its own first-class object: the actor is the operator/issuer (not the seat),
/// and it carries the delivery branch, the one-step verification, the Flux
/// verdict reference, and the decision lineage (merge commit / rework dispatch).
#[derive(Debug, Clone)]
pub struct AcceptanceRow {
    pub id: String,
    pub dispatch_id: String,
    pub project_id: String,
    pub branch: String,
    pub base: String,
    pub how_to_verify: String,
    pub flux_verdict_ref: String,
    pub status: String, // 'pending'|'accepted'|'rejected'|'cancelled'
    /// The structured acceptance plan (016, shape v1). `'{}'::jsonb` is the
    /// column default and means "no plan" — the legacy pending row is exempt
    /// from the plan CHECK until Aegis backfills it. Read as-is; the lineage
    /// assembler decides what `{}` means.
    pub acceptance_plan: serde_json::Value,
    pub feedback: Option<String>,
    pub rework_dispatch_id: Option<String>,
    pub merge_commit: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// One decision only the operator can rule on (020) — the structured answer
/// to "what is waiting on the human?", which until 020 existed only as a
/// message.
///
/// `choices` is the non-empty array of legal options (a single option is not a
/// choice); `consequence` is what happens once it is decided, and it is
/// REQUIRED — an obligation that does not say its consequence is asking the
/// operator to click in the dark. `evidence` is the support chain (dispatch
/// ids, verdict refs, ruling refs) so the claim can be checked rather than
/// trusted.
///
/// `decider` is who is entitled to rule. It defaults to the operator, and a
/// seat decider means Aegis created this for ITSELF — so it must NOT appear in
/// the operator's pending count.
///
/// `status` is `pending` or `resolved`; `resolved` is terminal and immutable.
/// `chosen` / `resolved_by` / `resolved_at` are set IFF resolved.
#[derive(Debug, Clone)]
pub struct OperatorDecisionRow {
    pub id: String,
    pub project_id: String,
    /// The concrete thing to decide — not a paragraph.
    pub target: String,
    pub choices: Vec<String>,
    pub decider: String,
    pub consequence: String,
    pub evidence: Vec<String>,
    /// 'pending' | 'resolved' (terminal).
    pub status: String,
    /// The only two creators: `human:*` or `seat:seat-aegis-001`.
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub chosen: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// An acceptance joined to the dispatch it gates plus the Flux verdict's
/// rendered facts — the read model the Tauri `cmd_list_acceptances` returns.
/// `flux_verdict` / `flux_verdict_occurred_at` are `None` when the referenced
/// row is an artifact (which carries no verdict payload) or has vanished.
/// `dispatch_priority` / `dispatch_payload` are carried so the reject path can
/// auto-enqueue the rework (priority+10, packet lineage) without a second query.
#[derive(Debug, Clone)]
pub struct AcceptanceWithDispatchRow {
    pub acceptance: AcceptanceRow,
    pub dispatch_title: String,
    pub dispatch_seat_id: String,
    pub dispatch_priority: i32,
    pub dispatch_payload: serde_json::Value,
    pub flux_verdict: Option<String>,
    pub flux_verdict_occurred_at: Option<DateTime<Utc>>,
}

/// One row of `v_memory_session_join` (022) — the session↔work bridge view.
///
/// Every Tier A field is carried **as the view computed it**, never re-derived
/// here: the row is evidence about what the view said, and re-deriving would let
/// a change in the read model silently disagree with the database.
///
/// The join is expressed on TWO keys, and both are reported rather than merged:
/// the contract's primary key A1 (the L2 corpus path, immune to id re-keying)
/// and its secondary key A2 (`source_ref` split on the first colon). Reporting
/// both is what keeps a disagreement countable instead of collapsing into one
/// number that looks fine.
#[derive(Debug, Clone)]
pub struct MemorySessionJoinRow {
    pub memory_id: String,
    pub project_id: String,
    /// Context only. NEVER a key: a topic is a segmentation, a revisable label.
    pub topic_id: String,
    pub title: String,
    pub kind: String,
    pub confidence: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    /// The L2 pointer, verbatim — the exact string the write-in declared.
    pub l2_file_path: String,
    pub l2_line_start: Option<i32>,
    pub l2_line_end: Option<i32>,
    /// A1: `l2_file_path` with the corpus root stripped. `None` when the path is
    /// neither the ~-rooted form nor the bare `raw/%` form — a NULL pointer is
    /// measured unresolved, never force-matched.
    pub a1_corpus_path: Option<String>,
    /// How many `archived_sessions` rows declare that path. `1` is the only
    /// resolving value; anything else is ambiguity, counted not guessed.
    pub a1_path_rows: Option<i64>,
    pub a1_harness: Option<String>,
    pub a1_id: Option<String>,
    pub a1_resolves: bool,
    /// A2: `source_ref` split on the FIRST colon. Both halves `None` when the
    /// reference has no colon at all, so the row measures unresolved rather than
    /// half-resolved.
    pub ref_harness: Option<String>,
    pub ref_id: Option<String>,
    pub a2_harness: Option<String>,
    pub a2_id: Option<String>,
    pub a2_resolves: bool,
    /// The joined session: A1 primary, A2 secondary. `None` when neither key
    /// resolved.
    pub session_harness: Option<String>,
    pub session_id: Option<String>,
    pub session_raw_path: Option<String>,
    pub session_project: Option<String>,
    pub session_message_count: Option<i32>,
    /// `both_agree` | `path_only` | `ref_only` | `disagree` | `none`.
    pub join_state: String,
}
