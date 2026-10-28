// Timeline commands — delegate to canonical_events.

use crate::commands::require_project_id;
use crate::dto::CanonicalEventDto;
use crate::state::AppState;
use tauri::State;

/// The timeline, scoped to one project.
///
/// **This used to read every project's events at once.** `cmd_list_events` had no
/// `project_id` parameter and queried `canonical_events` unfiltered, so the
/// timeline under `ql_delivery` showed `seatloom`'s 1353 events. Measured on live:
/// every one of the 1353 rows has `project_id = 'seatloom'`, so a project-scoped
/// read gives `seatloom` all of them and every other project none. That is the
/// truth, and it is what "the numbers change when you switch projects" means —
/// the unscoped read made them identical everywhere.
///
/// The per-project query already existed (`cmd_list_events_for_project`,
/// `list_events_for_project`); this command simply never used it.
#[tauri::command]
pub async fn cmd_list_events(
    state: State<'_, AppState>,
    project_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<CanonicalEventDto>, String> {
    let pid = require_project_id("cmd_list_events", project_id)?;
    state
        .db
        .list_events_for_project(&pid, limit.unwrap_or(200))
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
