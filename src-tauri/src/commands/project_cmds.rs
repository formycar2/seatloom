// Project commands — top-level project registry exposed to the UI NavRail.

use crate::dto::ProjectDto;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_projects(state: State<'_, AppState>) -> Result<Vec<ProjectDto>, String> {
    state
        .db
        .list_projects()
        .await
        .map(|rows| rows.into_iter().map(ProjectDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Option<ProjectDto>, String> {
    state
        .db
        .get_project(&project_id)
        .await
        .map(|opt| opt.map(ProjectDto::from))
        .map_err(|e| e.to_string())
}
