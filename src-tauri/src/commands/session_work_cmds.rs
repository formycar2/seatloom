// The session↔work bridge — the Tauri half of the read model.
//
// Product contract: LYRA-2026-10-16-session-work-bridge-contract-v1 §11.2 slice 2
// (dispatch disp-session-key-readmodel). Each command shells out to the SAME
// seatloom-mcp-emit binary the seat's own MCP tool calls, exactly the way
// `cmd_plan_intent` shells out to `plan_intent`, the three history commands to
// theirs, and `cmd_lineage` to `lineage`. There is ONE implementation
// (`seatloom_core::db::session_work`); these two commands add the stdio envelope
// and nothing else.
//
// No transcript body crosses this boundary either: the reads return pointers and
// line ranges, and the UI reads the slice it is pointed at from the local file.

use crate::state::AppState;
use tauri::State;

/// Drive the MCP binary over stdio and return the tool's text result, or the
/// tool's own rejection verbatim (never a transport error).
async fn call_tool(
    state: &AppState,
    tool: &str,
    arguments: serde_json::Value,
) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}"#;
    let inited = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call = serde_json::json!({
        "jsonrpc": "2.0", "id": 9, "method": "tools/call",
        "params": { "name": tool, "arguments": arguments },
    });
    let input = format!("{init}\n{inited}\n{call}\n");

    let bin = state.repo_root.join("target/debug/seatloom-mcp-emit");
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://seatloom:seatloom@localhost:5432/seatloom".to_string());

    let mut child = Command::new(&bin)
        .env("DATABASE_URL", &db_url)
        // The operator is who reads the bridge, so the actor is the operator.
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
        .ok_or(format!("{tool} produced no output"))?;
    let v: serde_json::Value = serde_json::from_str(last).map_err(|e| e.to_string())?;
    let r = &v["result"];
    let text = r["content"][0]["text"].as_str().unwrap_or("").to_string();
    if r["isError"].as_bool().unwrap_or(false) {
        Err(text)
    } else {
        Ok(text)
    }
}

/// `session_work(harness, id)` — the forward read.
///
/// Returns the assembled object as the tool's own JSON text. A refusal (the
/// bridge view is absent, or half a key) is the tool's typed message verbatim, so
/// the pane can name what is missing rather than rendering an empty list.
#[tauri::command]
pub async fn cmd_session_work(
    state: State<'_, AppState>,
    harness: String,
    id: String,
) -> Result<String, String> {
    call_tool(
        &state,
        "session_work",
        serde_json::json!({ "harness": harness, "id": id }),
    )
    .await
}

/// `memory_session(memory_id)` — the reverse read.
///
/// The tool answers the JSON literal `null` when the memory is not an
/// `archived_session` record, so the caller gets `"null"` and must render "no
/// answer" rather than an empty object that could be mistaken for a resolved one.
#[tauri::command]
pub async fn cmd_memory_session(
    state: State<'_, AppState>,
    memory_id: String,
) -> Result<String, String> {
    call_tool(
        &state,
        "memory_session",
        serde_json::json!({ "memory_id": memory_id }),
    )
    .await
}
