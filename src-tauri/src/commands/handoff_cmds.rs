// Handoff commands.

use crate::dto::{HandoffDto, HandoffReceiptDto};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_handoffs(state: State<'_, AppState>) -> Result<Vec<HandoffDto>, String> {
    state
        .db
        .list_handoffs()
        .await
        .map(|rows| rows.into_iter().map(HandoffDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_handoff_receipts(
    state: State<'_, AppState>,
) -> Result<Vec<HandoffReceiptDto>, String> {
    state
        .db
        .list_handoff_receipts()
        .await
        .map(|rows| rows.into_iter().map(HandoffReceiptDto::from).collect())
        .map_err(|e| e.to_string())
}
