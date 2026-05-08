// Inbox commands — Phase 1 returns a derived view from workitems + handoffs
// needing attention. Real InboxItem persistence arrives with route engine work.

use crate::dto::{HandoffDto, WorkItemDto};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Composite inbox payload: the current supervisor decision queue derived from
/// blocked/in-review workitems plus pending handoffs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxPayloadDto {
    pub blocked_workitems: Vec<WorkItemDto>,
    pub in_review_workitems: Vec<WorkItemDto>,
    pub pending_handoffs: Vec<HandoffDto>,
}

#[tauri::command]
pub async fn cmd_get_inbox(state: State<'_, AppState>) -> Result<InboxPayloadDto, String> {
    let workitems = state
        .db
        .list_workitems()
        .await
        .map_err(|e| e.to_string())?;
    let handoffs = state
        .db
        .list_handoffs()
        .await
        .map_err(|e| e.to_string())?;

    let blocked: Vec<WorkItemDto> = workitems
        .iter()
        .filter(|w| w.status == "blocked")
        .cloned()
        .map(WorkItemDto::from)
        .collect();
    let in_review: Vec<WorkItemDto> = workitems
        .iter()
        .filter(|w| w.status == "in_review")
        .cloned()
        .map(WorkItemDto::from)
        .collect();
    let pending: Vec<HandoffDto> = handoffs
        .into_iter()
        .filter(|h| matches!(h.status.as_str(), "sent" | "received"))
        .map(HandoffDto::from)
        .collect();

    Ok(InboxPayloadDto {
        blocked_workitems: blocked,
        in_review_workitems: in_review,
        pending_handoffs: pending,
    })
}
