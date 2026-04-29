use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::objects::id::{
    EventId, SeatId, SessionId, WorkItemId, ArtifactId,
    HandoffId, DelegationId, PipelineId, PipelineRunId, CheckpointId,
};
use crate::objects::handoff::ActorRef;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEvent {
    pub event_id: EventId,
    pub event_type: EventType,
    pub occurred_at: DateTime<Utc>,
    pub actor_ref: ActorRef,
    pub object_refs: Vec<ObjectRef>,
    pub evidence_refs: Vec<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    // Session lifecycle
    SessionStarted, SessionCompleted, SessionFailed, SessionInterrupted,
    // Interactive prompt lifecycle (AD-012)
    /// payload: kind, policy, bounded_window_ref, risk
    PromptDetected,
    /// payload: operator (user/supervisor), scope, result, assist_budget_stats
    PromptInputInjected,
    // Artifact lifecycle
    ArtifactCreated,
    // Handoff lifecycle
    HandoffDrafted, HandoffSent, HandoffAccepted, HandoffReturned, HandoffCompleted,
    /// P1 live activity state (US-P1-06).
    HandoffWorking,
    // WorkItem lifecycle
    WorkItemCreated, WorkItemStatusChanged,
    // Review/reissue evidence chain (AD-010, INT-05) — event-first, no durable Rejected/Rescoped states.
    /// payload: verdict, reason, linked_evidence_artifact_id
    ReviewVerdictIssued,
    /// payload: scope_change_summary, new_ac_refs
    WorkItemRescoped,
    // Seat delegation lifecycle (AD-009)
    SeatDelegationIssued, SeatDelegationClosed,
    // Pipeline lifecycle
    PipelineStarted, PipelineStageCompleted, PipelineCompleted, PipelineFailed,
    // Checkpoint
    CheckpointCreated,
    // Reconciliation
    ReconcileCompleted, DriftDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectRef {
    Seat(SeatId),
    Session(SessionId),
    WorkItem(WorkItemId),
    Artifact(ArtifactId),
    Handoff(HandoffId),
    Delegation(DelegationId),
    Pipeline(PipelineId),
    PipelineRun(PipelineRunId),
    Checkpoint(CheckpointId),
}
