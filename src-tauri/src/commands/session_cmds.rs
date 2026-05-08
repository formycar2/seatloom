// Session commands — read-only for Phase 1 (launch/attach deferred to Phase 4).

use crate::dto::{CheckpointDto, SessionDto};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionDto>, String> {
    state
        .db
        .list_sessions()
        .await
        .map(|rows| rows.into_iter().map(SessionDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_sessions_for_seat(
    state: State<'_, AppState>,
    seat_id: String,
) -> Result<Vec<SessionDto>, String> {
    state
        .db
        .list_sessions_for_seat(&seat_id)
        .await
        .map(|rows| rows.into_iter().map(SessionDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_checkpoints(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<CheckpointDto>, String> {
    state
        .db
        .list_checkpoints(limit.unwrap_or(50))
        .await
        .map(|rows| rows.into_iter().map(CheckpointDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_checkpoints_for_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<CheckpointDto>, String> {
    state
        .db
        .list_checkpoints_for_session(&session_id)
        .await
        .map(|rows| rows.into_iter().map(CheckpointDto::from).collect())
        .map_err(|e| e.to_string())
}
