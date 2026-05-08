// Timeline commands — delegate to canonical_events.

use crate::dto::CanonicalEventDto;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_events(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<CanonicalEventDto>, String> {
    state
        .db
        .list_events(limit.unwrap_or(200))
        .await
        .map(|rows| rows.into_iter().map(CanonicalEventDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_events_by_type(
    state: State<'_, AppState>,
    event_type: String,
) -> Result<Vec<CanonicalEventDto>, String> {
    state
        .db
        .list_events_by_type(&event_type)
        .await
        .map(|rows| rows.into_iter().map(CanonicalEventDto::from).collect())
        .map_err(|e| e.to_string())
}
