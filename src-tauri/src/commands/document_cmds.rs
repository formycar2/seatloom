// Document commands — read, filter, and search the document authority layer.

use crate::commands::require_project_id;
use crate::dto::{DocumentAssociationDto, DocumentDto, DocumentSectionDto, DocumentVersionDto};
use crate::state::AppState;
use tauri::State;

/// Documents for one project.
///
/// **This had the same silent-default fallback as `cmd_list_dispatches`.** It took
/// `project_id: Option<String>` and substituted `state.default_project_id` when it
/// was absent, and the UI called it as `api.listDocuments({})` — no project at all.
/// So the Artifacts tab's document half was always `seatloom`'s documents under
/// every project. Measured on live: `documents` is empty (0 rows), so the damage is
/// currently latent rather than visible, which is exactly when a defect like this
/// gets to survive.
#[tauri::command]
pub async fn cmd_list_documents(
    state: State<'_, AppState>,
    project_id: Option<String>,
    template: Option<String>,
    subtype: Option<String>,
) -> Result<Vec<DocumentDto>, String> {
    let pid = require_project_id("cmd_list_documents", project_id)?;
    state
        .db
        .list_documents(&pid, template.as_deref(), subtype.as_deref())
        .await
        .map(|rows| rows.into_iter().map(DocumentDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_document(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Option<DocumentDto>, String> {
    state
        .db
        .get_document(&document_id)
        .await
        .map(|opt| opt.map(DocumentDto::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_document_sections(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<DocumentSectionDto>, String> {
    state
        .db
        .list_document_sections(&document_id)
        .await
        .map(|rows| rows.into_iter().map(DocumentSectionDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_document_associations(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<DocumentAssociationDto>, String> {
    state
        .db
        .list_document_associations(&document_id)
        .await
        .map(|rows| rows.into_iter().map(DocumentAssociationDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_document_versions(
    state: State<'_, AppState>,
    document_id: String,
) -> Result<Vec<DocumentVersionDto>, String> {
    state
        .db
        .list_document_versions(&document_id)
        .await
        .map(|rows| rows.into_iter().map(DocumentVersionDto::from).collect())
        .map_err(|e| e.to_string())
}
