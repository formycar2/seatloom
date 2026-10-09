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

/// Enqueue a governed dispatch from the UI (the operator commanding a seat).
///
/// Shells out to the already-proven `seatloom-mcp-emit` binary's `enqueue_dispatch`
/// tool over stdio, rather than duplicating the validated enqueue path (atomic
/// row + SeatDispatchIssued event + NOTIFY) into this process. The NOTIFY wakes
/// the target seat's waker; the seat then pulls autonomously.
#[tauri::command]
pub async fn cmd_enqueue_dispatch(
    state: State<'_, AppState>,
    target_seat_id: String,
    title: String,
    instructions: String,
    priority: Option<i64>,
) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    let args = serde_json::json!({
        "target_seat_id": target_seat_id,
        "title": title,
        "payload": { "note": instructions },
        "priority": priority.unwrap_or(5),
    });
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}"#;
    let inited = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call = serde_json::json!({
        "jsonrpc": "2.0", "id": 9, "method": "tools/call",
        "params": { "name": "enqueue_dispatch", "arguments": args },
    });
    let input = format!("{init}\n{inited}\n{call}\n");

    let bin = state.repo_root.join("target/debug/seatloom-mcp-emit");
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://seatloom:seatloom@localhost:5432/seatloom".to_string());

    let mut child = Command::new(&bin)
        .env("DATABASE_URL", &db_url)
        .env("SEATLOOM_ACTOR_REF", "human:supervisor")
        .env("SEATLOOM_PROJECT_ID", &state.default_project_id)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", bin.display()))?;

    {
        let mut stdin = child.stdin.take().ok_or("no stdin")?;
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        // drop(stdin) at scope end closes the pipe -> server sees EOF and exits.
    }

    let out = child.wait_with_output().await.map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last = stdout
        .lines()
        .rfind(|l| !l.trim().is_empty())
        .ok_or("enqueue produced no output")?;
    let v: serde_json::Value = serde_json::from_str(last).map_err(|e| e.to_string())?;
    let r = &v["result"];
    let text = r["content"][0]["text"].as_str().unwrap_or("").to_string();
    if r["isError"].as_bool().unwrap_or(false) {
        Err(text)
    } else {
        Ok(text)
    }
}
