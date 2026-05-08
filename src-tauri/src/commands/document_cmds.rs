// Document commands — read, filter, and search the document authority layer.

use crate::dto::{
    DocumentAssociationDto, DocumentDto, DocumentSectionDto, DocumentVersionDto,
};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_documents(
    state: State<'_, AppState>,
    project_id: Option<String>,
    template: Option<String>,
    subtype: Option<String>,
) -> Result<Vec<DocumentDto>, String> {
    let pid = project_id.unwrap_or_else(|| state.default_project_id.clone());
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
