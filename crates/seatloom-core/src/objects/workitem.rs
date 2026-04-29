use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::objects::id::{WorkItemId, SeatId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: WorkItemId,
    pub title: String,
    pub goal: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub owner_seat_id: Option<SeatId>,
    pub status: WorkItemStatus,
    pub priority: Priority,
    pub depends_on: Vec<WorkItemId>,
    pub parent_id: Option<WorkItemId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkItemStatus {
    Draft, Ready, Active, Blocked,
    InReview, Verified, Done, Reopened, Drifted,
    // Event-first review/reissue path (AD-010, INT-05):
    // InReview → ReviewVerdictIssued event → Blocked → Ready → Active.
    // No durable Rejected/Rescoped states; verdict evidence lives in event payload.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority { Low, Medium, High }
