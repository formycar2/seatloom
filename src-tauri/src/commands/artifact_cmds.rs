// Artifact commands — read-only with template/subtype filtering.

use crate::dto::ArtifactDto;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_artifacts(
    state: State<'_, AppState>,
    template: Option<String>,
    subtype: Option<String>,
) -> Result<Vec<ArtifactDto>, String> {
    state
        .db
        .list_artifacts(template.as_deref(), subtype.as_deref())
        .await
        .map(|rows| rows.into_iter().map(ArtifactDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_artifact(
    state: State<'_, AppState>,
    artifact_id: String,
) -> Result<Option<ArtifactDto>, String> {
    state
        .db
        .get_artifact(&artifact_id)
        .await
        .map(|opt| opt.map(ArtifactDto::from))
        .map_err(|e| e.to_string())
}
