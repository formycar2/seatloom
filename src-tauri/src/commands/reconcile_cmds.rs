// Reconcile commands — trigger manual reconcile + inspect run history.

use crate::dto::{ReconcileItemDto, ReconcileResultDto, ReconcileRunDto};
use crate::state::AppState;
use seatloom_core::db::connection::create_pool;
use seatloom_core::db::reconcile::{run_reconcile, ReconcileTrigger};
use tauri::State;

#[tauri::command]
pub async fn cmd_reconcile(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<ReconcileResultDto, String> {
    let pid = project_id.unwrap_or_else(|| state.default_project_id.clone());
    let pool = create_pool().map_err(|e| e.to_string())?;
    run_reconcile(&pool, ReconcileTrigger::Manual, &state.repo_root, &pid)
        .await
        .map(ReconcileResultDto::from)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_reconcile_runs(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<ReconcileRunDto>, String> {
    state
        .db
        .list_reconcile_runs(limit.unwrap_or(20))
        .await
        .map(|rows| rows.into_iter().map(ReconcileRunDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_reconcile_run(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Option<ReconcileRunDto>, String> {
    state
        .db
        .get_reconcile_run(&run_id)
        .await
        .map(|opt| opt.map(ReconcileRunDto::from))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_reconcile_items(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Vec<ReconcileItemDto>, String> {
    state
        .db
        .list_reconcile_items(&run_id)
        .await
        .map(|rows| rows.into_iter().map(ReconcileItemDto::from).collect())
        .map_err(|e| e.to_string())
}
