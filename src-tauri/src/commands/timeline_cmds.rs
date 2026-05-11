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

/// Project-mode timeline list. AD-013 v2 backend invariant — schema 008.
#[tauri::command]
pub async fn cmd_list_events_for_project(
    state: State<'_, AppState>,
    project_id: String,
    limit: Option<i64>,
) -> Result<Vec<CanonicalEventDto>, String> {
    state
        .db
        .list_events_for_project(&project_id, limit.unwrap_or(200))
        .await
        .map(|rows| rows.into_iter().map(CanonicalEventDto::from).collect())
        .map_err(|e| e.to_string())
}
