// Three-layer seat model: AD-009.
// Layer 1 (SeatIdentity) is global and project-agnostic.
// Layer 2 (ProjectRoleBind) is per-project and stored separately.
// Layer 3 (SeatDelegation) is a scoped overlay that does not rewrite identity.

use crate::objects::id::{DelegationId, SeatId, WorkItemId};
use crate::objects::session::Runtime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Layer 1: Durable global seat identity. Stable across projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatIdentity {
    pub id: SeatId,
    pub name: String,
    pub default_runtime: Option<Runtime>,
    pub capability_tags: Vec<String>,
    pub status: SeatStatus,
    pub created_at: DateTime<Utc>,
}

/// Layer 2: Per-project role binding. Encodes role, authority docs, constraints,
/// and active delegation for one project. Stored separately from global identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRoleBind {
    pub seat_id: SeatId,
    pub project_id: String,
    pub role: SeatRole,
    pub authority_doc_refs: Vec<String>,
    pub constraints: Vec<String>,
    pub collaboration_template_ref: Option<String>,
    pub active_delegation_id: Option<DelegationId>,
}

/// Layer 3: Scoped delegation overlay. Does not rewrite original seat identity.
/// Timeline shows: "{to_seat} acting for {from_seat} on {workitem_id}".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatDelegation {
    pub id: DelegationId,
    pub issuer_seat_id: SeatId,
    pub from_seat_id: SeatId,
    pub to_seat_id: SeatId,
    pub workitem_id: Option<WorkItemId>,
    pub scope_description: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: DelegationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelegationStatus {
    Active,
    Closed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeatRole {
    ProductOwner,
    Architect,
    Verifier,
    Designer,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeatStatus {
    Active,
    Paused,
    Archived,
}
