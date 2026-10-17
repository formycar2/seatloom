// Seat commands — Tauri IPC bindings that read from SeatloomDb.

use crate::commands::require_project_id;
use crate::dto::{DelegationDto, RoleBindingDto, SeatDto};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_seats(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<SeatDto>, String> {
    let pid = require_project_id("cmd_list_seats", project_id)?;
    state
        .db
        .list_seats_for_project(&pid)
        .await
        .map(|rows| rows.into_iter().map(SeatDto::from).collect())
        .map_err(|e| e.to_string())
}

/// The GLOBAL seat registry, unfiltered.
///
/// Deliberately separate from [`cmd_list_seats`], which is project-scoped and
/// refuses to run without a project. This one exists for exactly one purpose:
/// naming the seats that are bound in **no** project at all, so an empty seat
/// list can say why it is empty instead of rendering as `席位 (0)` that reads as
/// broken. Measured on live: `seat-vela-001` has no binding anywhere.
///
/// It is unfiltered ON PURPOSE, and that is not the defect being fixed — the
/// defect was a project-scoped surface silently reading another project's rows.
/// A read whose whole job is "every seat, regardless of project" cannot be
/// scoped, and pretending otherwise would break the one thing it is for.
#[tauri::command]
pub async fn cmd_list_all_seats(state: State<'_, AppState>) -> Result<Vec<SeatDto>, String> {
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
