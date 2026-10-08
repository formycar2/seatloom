// Governed-dispatch (G19) read commands — the seat-inbox view (schema 012).

use std::collections::HashMap;

use crate::dto::{DispatchDependencyDto, DispatchDto};
use crate::state::AppState;
use tauri::State;

/// List a project's dispatches (seat inboxes) with their dependency edges
/// embedded. Ordered for an inbox view: by seat, then in_flight → ready →
/// blocked → done. Defaults to the app's default project.
#[tauri::command]
pub async fn cmd_list_dispatches(
    state: State<'_, AppState>,
    project_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<DispatchDto>, String> {
    let project = project_id.unwrap_or_else(|| state.default_project_id.clone());

    let rows = state
        .db
        .list_dispatches_for_project(&project, limit.unwrap_or(500))
        .await
        .map_err(|e| e.to_string())?;

    let deps = state
        .db
        .list_dispatch_dependencies_for_project(&project)
        .await
        .map_err(|e| e.to_string())?;

    // Group dependency edges by dispatch id.
    let mut by_id: HashMap<String, Vec<DispatchDependencyDto>> = HashMap::new();
    for d in deps {
        by_id
            .entry(d.dispatch_id.clone())
            .or_default()
            .push(DispatchDependencyDto::from(d));
    }

    Ok(rows
        .into_iter()
        .map(|r| {
            let edges = by_id.remove(&r.id).unwrap_or_default();
            DispatchDto::from_row(r, edges)
        })
        .collect())
}
