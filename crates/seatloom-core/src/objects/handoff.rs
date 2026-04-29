use crate::objects::id::{ArtifactId, HandoffId, SeatId, WorkItemId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handoff {
    pub id: HandoffId,
    pub from_ref: ActorRef,
    pub to_ref: ActorRef,
    pub workitem_id: WorkItemId,
    pub purpose: String,
    pub expected_outcome: String,
    pub artifact_ids: Vec<ArtifactId>,
    pub required_receipt: bool,
    pub status: HandoffStatus,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandoffStatus {
    Drafted,
    Sent,
    Received,
    Accepted,
    /// P1 live activity overlay (US-P1-06). Does not replace Ledger replay truth.
    Working,
    Returned,
    Completed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorRef {
    Human,
    Seat(SeatId),
    Automation,
}
