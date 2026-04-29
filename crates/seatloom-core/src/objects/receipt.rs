use crate::objects::id::{HandoffId, HandoffReceiptId, SeatId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffReceipt {
    pub id: HandoffReceiptId,
    pub handoff_id: HandoffId,
    pub acknowledged_by: SeatId,
    pub acknowledged_at: DateTime<Utc>,
    pub note: Option<String>,
    pub source_channel: ReceiptSourceChannel,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReceiptSourceChannel {
    Desktop,
    Mobile,
    Supervisor,
}
