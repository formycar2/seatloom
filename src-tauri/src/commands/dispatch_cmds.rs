// Governed-dispatch (G19) read commands — the seat-inbox view (schema 012).

use std::collections::HashMap;

use crate::dto::{DispatchDependencyDto, DispatchDto};
use crate::state::AppState;
use tauri::State;

/// List a project's dispatches (seat inboxes) with their dependency edges
/// embedded. Ordered for an inbox view: by seat, then in_flight → ready →
/// blocked → done.
///
/// **This command is the one that hid the defect.** It used to take
/// `project_id: Option<String>` and fall back to `state.default_project_id` when
/// it was absent, with a doc comment that said "Defaults to the app's default
/// project" as though that were a feature. The UI never passed the id, so the
/// dispatch list under `ql_delivery` was `seatloom`'s dispatches — a populated,
/// plausible, **wrong** list. A fallback that silently substitutes another
/// project's rows is worse than an error: the error says something is missing,
/// while the fallback says nothing at all.
#[tauri::command]
pub async fn cmd_list_dispatches(
    state: State<'_, AppState>,
    project_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<DispatchDto>, String> {
    let project = crate::commands::require_project_id("cmd_list_dispatches", project_id)?;

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

/// One dependency edge as the UI sends it — the SAME shape `enqueue_dispatch`
/// accepts (`dep_kind` + `dep_value`), so the wrapper passes it through without
/// translating anything.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueDependencyArg {
    pub dep_kind: String,
    pub dep_value: String,
}

/// Enqueue a governed dispatch from the UI (the operator commanding a seat).
///
/// Shells out to the already-proven `seatloom-mcp-emit` binary's `enqueue_dispatch`
/// tool over stdio, rather than duplicating the validated enqueue path (atomic
/// row + SeatDispatchIssued event + NOTIFY) into this process. The NOTIFY wakes
/// the target seat's waker; the seat then pulls autonomously.
///
/// **V5, fixed.** This wrapper used to pass only `target_seat_id`, `title`,
/// `payload:{note}` and `priority`, which silently downgraded any intent of the
/// form "do X, but only after Y lands" — the underlying MCP tool has always
/// accepted `dependencies`. The gap was here, not in the tool. `dependencies`
/// is now a first-class parameter and is forwarded verbatim; an empty or absent
/// list is forwarded as absent, so the tool's own validation still decides.
#[tauri::command]
pub async fn cmd_enqueue_dispatch(
    state: State<'_, AppState>,
    target_seat_id: String,
    title: String,
    instructions: String,
    priority: Option<i64>,
    dependencies: Option<Vec<EnqueueDependencyArg>>,
) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    // Forward the edges in the tool's own spelling. `Option` in, `Option` out:
    // an omitted list is omitted, never rewritten to `[]`, because the two mean
    // different things to the readiness rule.
    let deps: Option<Vec<serde_json::Value>> = dependencies.map(|edges| {
        edges
            .into_iter()
            .map(|e| serde_json::json!({ "dep_kind": e.dep_kind, "dep_value": e.dep_value }))
            .collect()
    });

    let mut args = serde_json::json!({
        "target_seat_id": target_seat_id,
        "title": title,
        "payload": { "note": instructions },
        "priority": priority.unwrap_or(5),
    });
    if let Some(deps) = deps {
        args["dependencies"] = serde_json::Value::Array(deps);
    }
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

/// Plan an intent from the UI: one sentence in, a typed result out.
///
/// The thin-wrapper half of the intent tool surface (product contract
/// `LYRA-2026-10-13-supervisor-intent-entry-contract-v1`, slice 1). It shells out
/// to the `plan_intent` MCP tool exactly the way [`cmd_enqueue_dispatch`] shells
/// out to `enqueue_dispatch`, so the two surfaces cannot drift: there is one
/// planner (`seatloom_core::intent`) and one gatherer
/// (`SeatloomDb::gather_intent_facts`), and this command adds nothing to either.
///
/// `SEATLOOM_ACTOR_REF=human:supervisor` is set for the same reason the enqueue
/// path sets it: when the operator confirms, the write must be attributed to the
/// person who asked, not to a seat.
#[tauri::command]
pub async fn cmd_plan_intent(state: State<'_, AppState>, intent: String) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    if intent.trim().is_empty() {
        return Err("plan_intent rejected: the intent is empty — there is nothing to plan".into());
    }

    let args = serde_json::json!({ "intent": intent });
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}"#;
    let inited = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call = serde_json::json!({
        "jsonrpc": "2.0", "id": 9, "method": "tools/call",
        "params": { "name": "plan_intent", "arguments": args },
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
        .ok_or("plan_intent produced no output")?;
    let v: serde_json::Value = serde_json::from_str(last).map_err(|e| e.to_string())?;
    let r = &v["result"];
    let text = r["content"][0]["text"].as_str().unwrap_or("").to_string();
    if r["isError"].as_bool().unwrap_or(false) {
        // A rejection is returned verbatim, including the empty-intent guard
        // above: the UI shows the reason rather than an empty card.
        Err(text)
    } else {
        Ok(text)
    }
}
