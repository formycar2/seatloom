// Frontend-shaped DTOs for Tauri IPC.
// These structs are serialized to JSON and consumed by the React frontend,
// so their field names use camelCase and all timestamps are ISO-8601 strings.

use chrono::{DateTime, Utc};
use seatloom_core::db::models::{
    AcceptanceRow, AcceptanceWithDispatchRow, ArtifactRow, CanonicalEventRow, CheckpointRow,
    DispatchDependencyRow, DispatchRow, DocumentAssociationRow, DocumentRow, DocumentSectionRow,
    DocumentVersionRow, HandoffReceiptRow, HandoffRow, PipelineRunRow, ProjectRoleBindingRow,
    ProjectRow, ReconcileItemRow, ReconcileRunRow, ReviewCommentRow, ReviewThreadRow,
    SeatDelegationRow, SeatRow, SessionRow, WorkItemRow,
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
    pub default_harness: Option<String>,
    pub default_agent: Option<String>,
    pub default_model: Option<String>,
    pub capability_tags: Vec<String>,
    pub status: String,
    pub created_at: String,
}

impl From<SeatRow> for SeatDto {
    fn from(r: SeatRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            default_harness: r.default_harness,
            default_agent: r.default_agent,
            default_model: r.default_model,
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
    pub created_at: String,
    pub ended_at: Option<String>,
    pub project_id: String,
}

impl From<SessionRow> for SessionDto {
    fn from(r: SessionRow) -> Self {
        Self {
            id: r.id,
            seat_id: r.seat_id,
            harness: r.harness,
            agent: r.agent,
            model: r.model,
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

/// One dependency edge of a dispatch (schema 012).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchDependencyDto {
    pub dispatch_id: String,
    pub dep_kind: String,
    pub dep_value: String,
}

impl From<DispatchDependencyRow> for DispatchDependencyDto {
    fn from(r: DispatchDependencyRow) -> Self {
        Self {
            dispatch_id: r.dispatch_id,
            dep_kind: r.dep_kind,
            dep_value: r.dep_value,
        }
    }
}

/// A governed-dispatch inbox row (schema 012), with its dependency edges
/// embedded so the UI renders a seat's queue in one pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchDto {
    pub id: String,
    pub target_seat_id: String,
    pub issued_by: String,
    pub project_id: String,
    pub title: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub priority: i32,
    pub issued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub deps: Vec<DispatchDependencyDto>,
}

impl DispatchDto {
    pub fn from_row(r: DispatchRow, deps: Vec<DispatchDependencyDto>) -> Self {
        Self {
            id: r.id,
            target_seat_id: r.target_seat_id,
            issued_by: r.issued_by,
            project_id: r.project_id,
            title: r.title,
            payload: r.payload,
            status: r.status,
            priority: r.priority,
            issued_at: iso(r.issued_at),
            started_at: iso_opt(r.started_at),
            completed_at: iso_opt(r.completed_at),
            deps,
        }
    }
}

// =============================================================================
// Acceptance gate (014, D2) — the 待验收 read model + the accept/reject result
// =============================================================================

/// The Flux green verdict, rendered for the card. `verdict` is `None` when the
/// referenced row is an artifact (which carries no verdict payload).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FluxVerdictDto {
    pub ref_id: String,
    pub verdict: Option<String>,
    pub occurred_at: Option<String>,
}

/// One acceptance card (schema 014): the delivery awaiting the operator's
/// decision, or the recorded decision. This is the DTO contract Mira's UI
/// consumes (LYRA-2026-10-09-d2-acceptance-loop-design-v1 §4/D).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceCard {
    pub id: String,
    pub dispatch_id: String,
    pub dispatch_title: String,
    pub seat_id: String,
    pub branch: String,
    pub base: String,
    pub how_to_verify: String,
    pub flux_verdict: FluxVerdictDto,
    /// 'pending' | 'accepted' | 'rejected' | 'cancelled' (accepted/rejected/
    /// cancelled are terminal; a cancelled card was withdrawn by the seat that
    /// requested it, so it is offered to the operator no more).
    pub status: String,
    pub created_at: String,
    pub merge_commit: Option<String>,
    pub feedback: Option<String>,
    pub rework_dispatch_id: Option<String>,
}

impl From<AcceptanceWithDispatchRow> for AcceptanceCard {
    fn from(r: AcceptanceWithDispatchRow) -> Self {
        let AcceptanceWithDispatchRow {
            acceptance,
            dispatch_title,
            dispatch_seat_id,
            // dispatch_priority / dispatch_payload serve the reject path's
            // rework enqueue; the card does not surface them.
            dispatch_priority: _,
            dispatch_payload: _,
            flux_verdict,
            flux_verdict_occurred_at,
        } = r;
        let AcceptanceRow {
            id,
            dispatch_id,
            project_id: _,
            branch,
            base,
            how_to_verify,
            flux_verdict_ref,
            status,
            feedback,
            rework_dispatch_id,
            merge_commit,
            decided_by: _,
            decided_at: _,
            created_at,
            // The plan is NOT surfaced on the pending-list card: the 待验收
            // list is a scan, and the plan is the operator's reading material
            // on the card's own detail view (the lineage read model).
            acceptance_plan: _,
        } = acceptance;
        Self {
            id,
            dispatch_id,
            dispatch_title,
            seat_id: dispatch_seat_id,
            branch,
            base,
            how_to_verify,
            flux_verdict: FluxVerdictDto {
                ref_id: flux_verdict_ref,
                verdict: flux_verdict,
                occurred_at: iso_opt(flux_verdict_occurred_at),
            },
            status,
            created_at: iso(created_at),
            merge_commit,
            feedback,
            rework_dispatch_id,
        }
    }
}

// =============================================================================
// Acceptance lineage (acceptance-lineage milestone, Lane B) — the read model
// =============================================================================

/// One acceptance card's full lineage, as the UI renders it. The seven blocks
/// are the value-domain contract's §3, in the operator's reading order; the
/// camelCase wire shape is produced here, ONCE, from the core assembler's
/// already-camelCase structs (the boundary normalises, it does not re-shape).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageDto {
    pub acceptance: AcceptanceLineageCardDto,
    pub dispatch: AcceptanceLineageDispatchDto,
    pub events: Vec<AcceptanceLineageEventDto>,
    pub flux_verdict: Option<AcceptanceLineageVerdictDto>,
    pub products: Vec<AcceptanceLineageProductDto>,
    pub dependencies: Vec<AcceptanceLineageDependencyDto>,
    pub rework: Option<AcceptanceLineageDispatchDto>,
}

impl From<seatloom_core::db::lineage::AcceptanceLineage> for AcceptanceLineageDto {
    fn from(l: seatloom_core::db::lineage::AcceptanceLineage) -> Self {
        Self {
            acceptance: l.acceptance.into(),
            dispatch: l.dispatch.into(),
            events: l.events.into_iter().map(Into::into).collect(),
            flux_verdict: l.flux_verdict.map(Into::into),
            products: l.products.into_iter().map(Into::into).collect(),
            dependencies: l.dependencies.into_iter().map(Into::into).collect(),
            rework: l.rework.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageCardDto {
    pub id: String,
    pub dispatch_id: String,
    pub project_id: String,
    pub branch: String,
    pub base: String,
    pub how_to_verify: String,
    /// The structured plan (016). `None` = the legacy row has no plan yet.
    pub acceptance_plan: Option<serde_json::Value>,
    pub flux_verdict_ref: String,
    pub status: String,
    pub feedback: Option<String>,
    pub rework_dispatch_id: Option<String>,
    pub merge_commit: Option<String>,
    pub decided_by: Option<String>,
    pub decided_at: Option<String>,
    pub created_at: String,
}

impl From<seatloom_core::db::lineage::AcceptanceLineageCard> for AcceptanceLineageCardDto {
    fn from(c: seatloom_core::db::lineage::AcceptanceLineageCard) -> Self {
        Self {
            id: c.id,
            dispatch_id: c.dispatch_id,
            project_id: c.project_id,
            branch: c.branch,
            base: c.base,
            how_to_verify: c.how_to_verify,
            acceptance_plan: c.acceptance_plan,
            flux_verdict_ref: c.flux_verdict_ref,
            status: c.status,
            feedback: c.feedback,
            rework_dispatch_id: c.rework_dispatch_id,
            merge_commit: c.merge_commit,
            decided_by: c.decided_by,
            decided_at: iso_opt(c.decided_at),
            created_at: iso(c.created_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageDispatchDto {
    pub id: String,
    pub title: String,
    pub issued_by: String,
    pub target_seat_id: String,
    pub priority: i32,
    pub status: String,
    pub issued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    /// The original task statement (packet path / instructions), verbatim.
    pub payload: serde_json::Value,
}

impl From<seatloom_core::db::lineage::AcceptanceLineageDispatch> for AcceptanceLineageDispatchDto {
    fn from(d: seatloom_core::db::lineage::AcceptanceLineageDispatch) -> Self {
        Self {
            id: d.id,
            title: d.title,
            issued_by: d.issued_by,
            target_seat_id: d.target_seat_id,
            priority: d.priority,
            status: d.status,
            issued_at: iso(d.issued_at),
            started_at: iso_opt(d.started_at),
            completed_at: iso_opt(d.completed_at),
            payload: d.payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageEventDto {
    pub id: String,
    pub event_type: String,
    pub occurred_at: String,
    pub actor_ref: String,
    pub payload: Option<serde_json::Value>,
}

impl From<seatloom_core::db::lineage::AcceptanceLineageEvent> for AcceptanceLineageEventDto {
    fn from(e: seatloom_core::db::lineage::AcceptanceLineageEvent) -> Self {
        Self {
            id: e.id,
            event_type: e.event_type,
            occurred_at: iso(e.occurred_at),
            actor_ref: e.actor_ref,
            payload: e.payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageVerdictDto {
    pub ref_id: String,
    pub verdict: Option<String>,
    pub evidence: Option<String>,
    pub evidence_detail: Option<String>,
    pub target_branch: Option<String>,
    pub lane: Option<String>,
    pub occurred_at: Option<String>,
}

impl From<seatloom_core::db::lineage::AcceptanceLineageVerdict> for AcceptanceLineageVerdictDto {
    fn from(v: seatloom_core::db::lineage::AcceptanceLineageVerdict) -> Self {
        Self {
            ref_id: v.ref_id,
            verdict: v.verdict,
            evidence: v.evidence,
            evidence_detail: v.evidence_detail,
            target_branch: v.target_branch,
            lane: v.lane,
            occurred_at: iso_opt(v.occurred_at),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageProductDto {
    pub ref_type: String,
    pub ref_id: String,
    /// `None` when the referent could not be joined — the id is still carried.
    pub title: Option<String>,
    pub path: Option<String>,
    /// `false` = the referent is not in the database (carried verbatim, never
    /// guessed at).
    pub resolved: bool,
}

impl From<seatloom_core::db::lineage::AcceptanceLineageProduct> for AcceptanceLineageProductDto {
    fn from(p: seatloom_core::db::lineage::AcceptanceLineageProduct) -> Self {
        Self {
            ref_type: p.ref_type,
            ref_id: p.ref_id,
            title: p.title,
            path: p.path,
            resolved: p.resolved,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceLineageDependencyDto {
    pub dep_kind: String,
    pub dep_value: String,
    /// The referent's live existence (012: readiness is an existence query).
    pub exists: bool,
}

impl From<seatloom_core::db::lineage::AcceptanceLineageDependency>
    for AcceptanceLineageDependencyDto
{
    fn from(d: seatloom_core::db::lineage::AcceptanceLineageDependency) -> Self {
        Self {
            dep_kind: d.dep_kind,
            dep_value: d.dep_value,
            exists: d.exists,
        }
    }
}

/// The outcome of an accept/reject command (D2 contract D, widened by the
/// Flux HOLD fix). A refusal (merge conflict, missing operator config,
/// not-pending, unverified) is an `Err` carrying the reason — `error` is the
/// soft-failure field the contract asks the UI to render, kept for contract
/// fidelity.
///
/// `status` + `rework_dispatch_id` make the response a SUPERSET of the
/// acceptance-card normaliser (`FLUX-2026-10-09-mira-d2-ui-verify-delivery-v1`
/// §4): on a reject the UI can render the rejected badge, echo the feedback
/// (already on the card) and jump to the rework dispatch from the decision
/// response alone — no follow-up read, no dead-end "already handled" label.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceResult {
    pub accepted: bool,
    /// The decided acceptance's status: 'accepted' | 'rejected'.
    pub status: String,
    pub merge_commit: Option<String>,
    /// The auto-enqueued rework dispatch on a reject (lineage jump link).
    pub rework_dispatch_id: Option<String>,
    pub error: Option<String>,
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
// Scaffolding: read-model DTO for a pipeline-run list/get command not yet wired.
// Pre-existing at track 4c13ffc; surfaced by the clippy -D warnings gate. Wire or
// remove when the pipeline command lands (tracked in MR !4 reconcile notes).
#[allow(dead_code)]
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
// Scaffolding: read-model DTO for a review-thread list/get command not yet wired.
// Pre-existing at track 4c13ffc; surfaced by the clippy -D warnings gate. Wire or
// remove when the review command lands (tracked in MR !4 reconcile notes).
#[allow(dead_code)]
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
// Scaffolding: read-model DTO for a review-comment list/get command not yet wired.
// Pre-existing at track 4c13ffc; surfaced by the clippy -D warnings gate. Wire or
// remove when the review command lands (tracked in MR !4 reconcile notes).
#[allow(dead_code)]
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
