// Frontend-shaped DTOs for Tauri IPC.
// These structs are serialized to JSON and consumed by the React frontend,
// so their field names use camelCase and all timestamps are ISO-8601 strings.

use chrono::{DateTime, Utc};
use seatloom_core::db::models::{
    ArtifactRow, CanonicalEventRow, CheckpointRow, DocumentAssociationRow, DocumentRow,
    DocumentSectionRow, DocumentVersionRow, HandoffReceiptRow, HandoffRow, PipelineRunRow,
    ProjectRoleBindingRow, ProjectRow, ReconcileItemRow, ReconcileRunRow, ReviewCommentRow,
    ReviewThreadRow, SeatDelegationRow, SeatRow, SessionRow, WorkItemRow,
};
use serde::{Deserialize, Serialize};

fn iso(ts: DateTime<Utc>) -> String {
    ts.to_rfc3339()
}

fn iso_opt(ts: Option<DateTime<Utc>>) -> Option<String> {
    ts.map(iso)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub worker_budget_tokens: i32,
    pub supervisor_budget_tokens: i32,
}

impl From<ProjectRow> for ProjectDto {
    fn from(r: ProjectRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            created_at: iso(r.created_at),
            worker_budget_tokens: r.worker_budget_tokens,
            supervisor_budget_tokens: r.supervisor_budget_tokens,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatDto {
    pub id: String,
    pub name: String,
    pub default_runtime: Option<String>,
    pub capability_tags: Vec<String>,
    pub status: String,
    pub created_at: String,
}

impl From<SeatRow> for SeatDto {
    fn from(r: SeatRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            default_runtime: r.default_runtime,
            capability_tags: r.capability_tags,
            status: r.status,
            created_at: iso(r.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleBindingDto {
    pub seat_id: String,
    pub project_id: String,
    pub role: String,
    pub authority_doc_refs: Vec<String>,
    pub constraints: Vec<String>,
    pub collaboration_template_ref: Option<String>,
    pub active_delegation_id: Option<String>,
}

impl From<ProjectRoleBindingRow> for RoleBindingDto {
    fn from(r: ProjectRoleBindingRow) -> Self {
        Self {
            seat_id: r.seat_id,
            project_id: r.project_id,
            role: r.role,
            authority_doc_refs: r.authority_doc_refs,
            constraints: r.constraints,
            collaboration_template_ref: r.collaboration_template_ref,
            active_delegation_id: r.active_delegation_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegationDto {
    pub id: String,
    pub issuer_seat_id: String,
    pub from_seat_id: String,
    pub to_seat_id: String,
    pub workitem_id: Option<String>,
    pub scope_description: String,
    pub issued_at: String,
    pub expires_at: Option<String>,
    pub status: String,
}

impl From<SeatDelegationRow> for DelegationDto {
    fn from(r: SeatDelegationRow) -> Self {
        Self {
            id: r.id,
            issuer_seat_id: r.issuer_seat_id,
            from_seat_id: r.from_seat_id,
            to_seat_id: r.to_seat_id,
            workitem_id: r.workitem_id,
            scope_description: r.scope_description,
            issued_at: iso(r.issued_at),
            expires_at: iso_opt(r.expires_at),
            status: r.status,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
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
    pub created_at: String,
    pub ended_at: Option<String>,
    pub project_id: String,
}

impl From<SessionRow> for SessionDto {
    fn from(r: SessionRow) -> Self {
        Self {
            id: r.id,
            seat_id: r.seat_id,
            runtime: r.runtime,
            native_session_id: r.native_session_id,
            workspace_path: r.workspace_path,
            branch: r.branch,
            status: r.status,
            launch_pack_ref: r.launch_pack_ref,
            last_checkpoint_id: r.last_checkpoint_id,
            pid: r.pid,
            created_at: iso(r.created_at),
            ended_at: iso_opt(r.ended_at),
            project_id: r.project_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemDto {
    pub id: String,
    pub title: String,
    pub goal: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub owner_seat_id: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_at: String,
    pub updated_at: String,
    pub project_id: String,
}

impl From<WorkItemRow> for WorkItemDto {
    fn from(r: WorkItemRow) -> Self {
        Self {
            id: r.id,
            title: r.title,
            goal: r.goal,
            acceptance_criteria: r.acceptance_criteria,
            owner_seat_id: r.owner_seat_id,
            status: r.status,
            priority: r.priority,
            created_at: iso(r.created_at),
            updated_at: iso(r.updated_at),
            project_id: r.project_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffDto {
    pub id: String,
    pub from_ref: String,
    pub to_ref: String,
    pub workitem_id: String,
    pub purpose: String,
    pub expected_outcome: String,
    pub required_receipt: bool,
    pub status: String,
    pub created_at: String,
    pub sent_at: Option<String>,
    pub project_id: String,
}

impl From<HandoffRow> for HandoffDto {
    fn from(r: HandoffRow) -> Self {
        Self {
            id: r.id,
            from_ref: r.from_ref,
            to_ref: r.to_ref,
            workitem_id: r.workitem_id,
            purpose: r.purpose,
            expected_outcome: r.expected_outcome,
            required_receipt: r.required_receipt,
            status: r.status,
            created_at: iso(r.created_at),
            sent_at: iso_opt(r.sent_at),
            project_id: r.project_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactDto {
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
    pub created_at: String,
}

impl From<ArtifactRow> for ArtifactDto {
    fn from(r: ArtifactRow) -> Self {
        Self {
            id: r.id,
            template: r.template,
            subtype: r.subtype,
            subtype_valid: r.subtype_valid,
            system_kind: r.system_kind,
            title: r.title,
            summary: r.summary,
            source_session_id: r.source_session_id,
            source_workitem_id: r.source_workitem_id,
            storage_path: r.storage_path,
            created_at: iso(r.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalEventDto {
    pub id: String,
    pub event_type: String,
    pub occurred_at: String,
    pub actor_ref: String,
    pub payload: Option<serde_json::Value>,
    pub created_at: String,
    pub project_id: String,
}

impl From<CanonicalEventRow> for CanonicalEventDto {
    fn from(r: CanonicalEventRow) -> Self {
        Self {
            id: r.id,
            event_type: r.event_type,
            occurred_at: iso(r.occurred_at),
            actor_ref: r.actor_ref,
            payload: r.payload,
            created_at: iso(r.created_at),
            project_id: r.project_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDto {
    pub id: String,
    pub project_id: String,
    pub artifact_id: Option<String>,
    pub template: Option<String>,
    pub subtype: Option<String>,
    pub subtype_valid: Option<bool>,
    pub doc_id: Option<String>,
    pub title: String,
    pub status: Option<String>,
    pub author: Option<String>,
    pub doc_date: Option<String>,
    pub version: Option<String>,
    pub depends_on: Vec<String>,
    pub supersedes: Option<String>,
    pub tags: Vec<String>,
    pub file_path: String,
    pub body_text: Option<String>,
    pub body_digest: Option<String>,
    pub body_length: Option<i32>,
    pub parse_status: String,
    pub revision: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<DocumentRow> for DocumentDto {
    fn from(r: DocumentRow) -> Self {
        Self {
            id: r.id,
            project_id: r.project_id,
            artifact_id: r.artifact_id,
            template: r.template,
            subtype: r.subtype,
            subtype_valid: r.subtype_valid,
            doc_id: r.doc_id,
            title: r.title,
            status: r.status,
            author: r.author,
            doc_date: r.doc_date.map(|d| d.to_string()),
            version: r.version,
            depends_on: r.depends_on,
            supersedes: r.supersedes,
            tags: r.tags,
            file_path: r.file_path,
            body_text: r.body_text,
            body_digest: r.body_digest,
            body_length: r.body_length,
            parse_status: r.parse_status,
            revision: r.revision,
            created_at: iso(r.created_at),
            updated_at: iso(r.updated_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSectionDto {
    pub id: String,
    pub document_id: String,
    pub ordinal: i32,
    pub heading_text: String,
    pub heading_level: i32,
    pub anchor_slug: String,
    pub body_excerpt: Option<String>,
    pub created_at: String,
}

impl From<DocumentSectionRow> for DocumentSectionDto {
    fn from(r: DocumentSectionRow) -> Self {
        Self {
            id: r.id,
            document_id: r.document_id,
            ordinal: r.ordinal,
            heading_text: r.heading_text,
            heading_level: r.heading_level,
            anchor_slug: r.anchor_slug,
            body_excerpt: r.body_excerpt,
            created_at: iso(r.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentAssociationDto {
    pub id: String,
    pub document_id: String,
    pub assoc_type: String,
    pub assoc_id: String,
    pub is_primary: bool,
    pub created_at: String,
}

impl From<DocumentAssociationRow> for DocumentAssociationDto {
    fn from(r: DocumentAssociationRow) -> Self {
        Self {
            id: r.id,
            document_id: r.document_id,
            assoc_type: r.assoc_type,
            assoc_id: r.assoc_id,
            is_primary: r.is_primary,
            created_at: iso(r.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentVersionDto {
    pub id: String,
    pub document_id: String,
    pub revision: i32,
    pub body_digest: String,
    pub header_snapshot: Option<serde_json::Value>,
    pub run_id: Option<String>,
    pub created_at: String,
}

impl From<DocumentVersionRow> for DocumentVersionDto {
    fn from(r: DocumentVersionRow) -> Self {
        Self {
            id: r.id,
            document_id: r.document_id,
            revision: r.revision,
            body_digest: r.body_digest,
            header_snapshot: r.header_snapshot,
            run_id: r.run_id,
            created_at: iso(r.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileRunDto {
    pub id: String,
    pub trigger: String,
    pub status: String,
    pub scanned: i32,
    pub inserted: i32,
    pub updated: i32,
    pub unchanged: i32,
    pub failed: i32,
    pub conflicted: i32,
    pub started_at: String,
    pub completed_at: Option<String>,
}

impl From<ReconcileRunRow> for ReconcileRunDto {
    fn from(r: ReconcileRunRow) -> Self {
        Self {
            id: r.id,
            trigger: r.trigger,
            status: r.status,
            scanned: r.scanned,
            inserted: r.inserted,
            updated: r.updated,
            unchanged: r.unchanged,
            failed: r.failed,
            conflicted: r.conflicted,
            started_at: iso(r.started_at),
            completed_at: iso_opt(r.completed_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileItemDto {
    pub id: String,
    pub run_id: String,
    pub file_path: String,
    pub document_id: Option<String>,
    pub outcome: String,
    pub digest_before: Option<String>,
    pub digest_after: Option<String>,
    pub revision_before: Option<i32>,
    pub revision_after: Option<i32>,
    pub parse_status: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: String,
}

impl From<ReconcileItemRow> for ReconcileItemDto {
    fn from(r: ReconcileItemRow) -> Self {
        Self {
            id: r.id,
            run_id: r.run_id,
            file_path: r.file_path,
            document_id: r.document_id,
            outcome: r.outcome,
            digest_before: r.digest_before,
            digest_after: r.digest_after,
            revision_before: r.revision_before,
            revision_after: r.revision_after,
            parse_status: r.parse_status,
            failure_reason: r.failure_reason,
            created_at: iso(r.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconcileResultDto {
    pub run_id: String,
    pub trigger: String,
    pub scanned: u32,
    pub inserted: u32,
    pub updated: u32,
    pub unchanged: u32,
    pub failed: u32,
    pub conflicted: u32,
}

impl From<seatloom_core::db::reconcile::ReconcileResult> for ReconcileResultDto {
    fn from(r: seatloom_core::db::reconcile::ReconcileResult) -> Self {
        Self {
            run_id: r.run_id,
            trigger: r.trigger,
            scanned: r.scanned,
            inserted: r.inserted,
            updated: r.updated,
            unchanged: r.unchanged,
            failed: r.failed,
            conflicted: r.conflicted,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointDto {
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
    pub continuity_budget_tokens: Option<i32>,
    pub created_at: String,
}

impl From<CheckpointRow> for CheckpointDto {
    fn from(r: CheckpointRow) -> Self {
        Self {
            id: r.id,
            session_id: r.session_id,
            trigger: r.trigger,
            summary_what_was_done: r.summary_what_was_done,
            summary_current_state: r.summary_current_state,
            summary_open_questions: r.summary_open_questions,
            summary_quality: r.summary_quality,
            artifact_ids_at_checkpoint: r.artifact_ids_at_checkpoint,
            branch: r.branch,
            last_commit: r.last_commit,
            continuity_budget_tokens: r.continuity_budget_tokens,
            created_at: iso(r.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandoffReceiptDto {
    pub id: String,
    pub handoff_id: String,
    pub acknowledged_by: String,
    pub acknowledged_at: String,
    pub note: Option<String>,
    pub source_channel: String,
    pub created_at: String,
}

impl From<HandoffReceiptRow> for HandoffReceiptDto {
    fn from(r: HandoffReceiptRow) -> Self {
        Self {
            id: r.id,
            handoff_id: r.handoff_id,
            acknowledged_by: r.acknowledged_by,
            acknowledged_at: iso(r.acknowledged_at),
            note: r.note,
            source_channel: r.source_channel,
            created_at: iso(r.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineRunDto {
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
    pub started_at: String,
    pub finished_at: Option<String>,
}

impl From<PipelineRunRow> for PipelineRunDto {
    fn from(r: PipelineRunRow) -> Self {
        Self {
            id: r.id,
            pipeline_id: r.pipeline_id,
            workitem_id: r.workitem_id,
            status: r.status,
            current_stage: r.current_stage,
            stage_index: r.stage_index,
            trigger: r.trigger,
            initiated_by: r.initiated_by,
            result_summary: r.result_summary,
            artifact_ids: r.artifact_ids,
            evidence_refs: r.evidence_refs,
            started_at: iso(r.started_at),
            finished_at: iso_opt(r.finished_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewThreadDto {
    pub id: String,
    pub project_id: String,
    pub source_channel: String,
    pub mode: String,
    pub target_kind: String,
    pub target_id: String,
    pub title: Option<String>,
    pub status: String,
    pub requires_followup: bool,
    pub review_tier: Option<String>,
    pub created_by: String,
    pub assigned_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<String>,
}

impl From<ReviewThreadRow> for ReviewThreadDto {
    fn from(r: ReviewThreadRow) -> Self {
        Self {
            id: r.id,
            project_id: r.project_id,
            source_channel: r.source_channel,
            mode: r.mode,
            target_kind: r.target_kind,
            target_id: r.target_id,
            title: r.title,
            status: r.status,
            requires_followup: r.requires_followup,
            review_tier: r.review_tier,
            created_by: r.created_by,
            assigned_to: r.assigned_to,
            created_at: iso(r.created_at),
            updated_at: iso(r.updated_at),
            resolved_at: iso_opt(r.resolved_at),
            resolved_by: r.resolved_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCommentDto {
    pub id: String,
    pub thread_id: String,
    pub parent_comment_id: Option<String>,
    pub author_ref: String,
    pub body_text: String,
    pub mode: String,
    pub source_channel: String,
    pub state: String,
    pub evidence_refs: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ReviewCommentRow> for ReviewCommentDto {
    fn from(r: ReviewCommentRow) -> Self {
        Self {
            id: r.id,
            thread_id: r.thread_id,
            parent_comment_id: r.parent_comment_id,
            author_ref: r.author_ref,
            body_text: r.body_text,
            mode: r.mode,
            source_channel: r.source_channel,
            state: r.state,
            evidence_refs: r.evidence_refs,
            created_at: iso(r.created_at),
            updated_at: iso(r.updated_at),
        }
    }
}
