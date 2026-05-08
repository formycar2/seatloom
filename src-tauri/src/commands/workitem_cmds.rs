// WorkItem commands — read-only for Phase 1. Mutations added in later phases.

use crate::dto::WorkItemDto;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_workitems(state: State<'_, AppState>) -> Result<Vec<WorkItemDto>, String> {
    state
        .db
        .list_workitems()
        .await
        .map(|rows| rows.into_iter().map(WorkItemDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_workitem(
    state: State<'_, AppState>,
    workitem_id: String,
) -> Result<Option<WorkItemDto>, String> {
    state
        .db
        .get_workitem(&workitem_id)
        .await
        .map(|opt| opt.map(WorkItemDto::from))
        .map_err(|e| e.to_string())
}
