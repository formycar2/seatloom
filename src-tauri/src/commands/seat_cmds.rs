// Seat commands — Tauri IPC bindings that read from SeatloomDb.

use crate::dto::{DelegationDto, RoleBindingDto, SeatDto};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_seats(state: State<'_, AppState>) -> Result<Vec<SeatDto>, String> {
    state
        .db
        .list_seats()
        .await
        .map(|rows| rows.into_iter().map(SeatDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_seat(
    state: State<'_, AppState>,
    seat_id: String,
) -> Result<Option<SeatDto>, String> {
    state
        .db
        .get_seat(&seat_id)
        .await
        .map(|opt| opt.map(SeatDto::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_role_bindings(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<RoleBindingDto>, String> {
    let pid = project_id.unwrap_or_else(|| state.default_project_id.clone());
    state
        .db
        .list_role_bindings_for_project(&pid)
        .await
        .map(|rows| rows.into_iter().map(RoleBindingDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_delegations(
    state: State<'_, AppState>,
) -> Result<Vec<DelegationDto>, String> {
    state
        .db
        .list_delegations()
        .await
        .map(|rows| rows.into_iter().map(DelegationDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_delegation(
    state: State<'_, AppState>,
    delegation_id: String,
) -> Result<Option<DelegationDto>, String> {
    state
        .db
        .get_delegation(&delegation_id)
        .await
        .map(|opt| opt.map(DelegationDto::from))
        .map_err(|e| e.to_string())
}
