use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::objects::id::{HandoffId, SeatId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffReceipt {
    pub handoff_id: HandoffId,
    pub acknowledged_by: SeatId,
    pub acknowledged_at: DateTime<Utc>,
    pub note: Option<String>,
}
