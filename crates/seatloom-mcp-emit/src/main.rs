//! `seatloom-mcp-emit` — the P0 step 4 MCP emit server.
//!
//! Reads newline-delimited JSON-RPC 2.0 on stdin, writes responses on stdout,
//! and logs to stderr (never stdout — stdout is the protocol channel). Exposes
//! the governed-dispatch toolset (step 4 + step 6 + 013/014): `emit_event`,
//! `enqueue_dispatch`, `get_next_dispatch`, `complete_dispatch`,
//! `cancel_dispatch`, `request_acceptance`. See
//! NIMBUS-2026-10-08-p0-step4-mcp-emit-v1.

mod dispatch;
mod emit;
mod memory;
mod protocol;
mod read;
mod scheduler;

use deadpool_postgres::Pool;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use seatloom_core::db::{create_pool, SeatloomDb};

use crate::protocol::{
    classify, error, initialize_result, success, tool_call_result, tools_list_result, Inbound,
};

/// JSON-RPC reserved error code for "method not found".
const METHOD_NOT_FOUND: i64 = -32601;
/// JSON-RPC reserved error code for an invalid request / parse issue.
const INVALID_REQUEST: i64 = -32600;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("seatloom-mcp-emit: fatal: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Pool once at startup. create_pool() reads DATABASE_URL (or the default
    // local URL). The server is launched per seat with DATABASE_URL in its env
    // (never the live `seatloom` DB when under test — the operator points it at
    // the right database via `pi mcp add --env`).
    let pool = create_pool()?;
    let db = SeatloomDb::new(pool.clone());
    eprintln!("seatloom-mcp-emit: ready (tools: emit_event, enqueue_dispatch, get_next_dispatch, complete_dispatch, cancel_dispatch, request_acceptance, cancel_acceptance, get_acceptance_lineage, recall, remember, promote, forget, resolve, get_dispatch, query_ledger)");

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_line(&db, &pool, &line).await;
        if let Some(resp) = response {
            let mut bytes = serde_json::to_vec(&resp)?;
            bytes.push(b'\n');
            stdout.write_all(&bytes).await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}

/// Handle one inbound line. Returns `Some(response)` for a request, `None` for a
/// notification (which must never be answered).
async fn handle_line(db: &SeatloomDb, pool: &Pool, line: &str) -> Option<Value> {
    match classify(line) {
        Inbound::Notification { method } => {
            eprintln!("seatloom-mcp-emit: notification {method} (no response)");
            None
        }
        Inbound::Malformed { id, message } => Some(error(id, INVALID_REQUEST, &message)),
        Inbound::Request { id, method, params } => {
            Some(handle_request(db, pool, id, &method, params).await)
        }
    }
}

async fn handle_request(
    db: &SeatloomDb,
    pool: &Pool,
    id: Value,
    method: &str,
    params: Value,
) -> Value {
    match method {
        "initialize" => success(id, initialize_result(&params)),
        "ping" => success(id, serde_json::json!({})),
        "tools/list" => success(id, tools_list_result()),
        "tools/call" => handle_tools_call(db, pool, id, params).await,
        other => error(id, METHOD_NOT_FOUND, &format!("method not found: {other}")),
    }
}

/// `tools/call` dispatch. Fifteen tools: `emit_event` (step 4), the six
/// governed-dispatch tools (step 6 + 013/014 + the D2 follow-on), the
/// acceptance-lineage READ (Lane B), the five D5 memory tools
/// (`recall`/`remember`/`promote`/`forget`/`resolve`) and the two pure READS
/// (`get_dispatch`/`query_ledger`). Any other name is a tool error. A rejection
/// is a tool error with `isError: true`, carrying the typed message verbatim —
/// never a transport error, and never a partial write.
async fn handle_tools_call(db: &SeatloomDb, pool: &Pool, id: Value, params: Value) -> Value {
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
    match name {
        "emit_event" => call_emit(db, id, arguments).await,
        "enqueue_dispatch" => call_enqueue(pool, id, arguments).await,
        "get_next_dispatch" => call_get_next(pool, id, arguments).await,
        "complete_dispatch" => call_complete(pool, id, arguments).await,
        "cancel_dispatch" => call_cancel(pool, id, arguments).await,
        "request_acceptance" => call_request_acceptance(pool, id, arguments).await,
        "cancel_acceptance" => call_cancel_acceptance(pool, id, arguments).await,
        "get_acceptance_lineage" => call_get_acceptance_lineage(db, id, arguments).await,
        // --- D5 knowledge layer (Lane B): memory management, explicit tools ---
        "recall" => match memory_args::<memory::RecallArgs>(&id, arguments) {
            Ok(a) => render_memory(id, memory::recall(db, a).await),
            Err(resp) => resp,
        },
        "remember" => match memory_args::<memory::RememberArgs>(&id, arguments) {
            Ok(a) => render_memory(id, memory::remember(db, a).await),
            Err(resp) => resp,
        },
        "promote" => match memory_args::<memory::PromoteArgs>(&id, arguments) {
            Ok(a) => render_memory(id, memory::promote(db, a).await),
            Err(resp) => resp,
        },
        "forget" => match memory_args::<memory::ForgetArgs>(&id, arguments) {
            Ok(a) => render_memory(id, memory::forget(db, a).await),
            Err(resp) => resp,
        },
        "resolve" => match memory_args::<memory::ResolveArgs>(&id, arguments) {
            Ok(a) => render_memory(id, memory::resolve(db, a).await),
            Err(resp) => resp,
        },
        // --- D5 knowledge layer (Lane B): the pure reads that end the psql era ---
        "get_dispatch" => match read_args::<read::GetDispatchArgs>(&id, arguments) {
            Ok(a) => render_read(id, read::get_dispatch(db, a).await),
            Err(resp) => resp,
        },
        "query_ledger" => match read_args::<read::QueryLedgerArgs>(&id, arguments) {
            Ok(a) => render_read(id, read::query_ledger(db, a).await),
            Err(resp) => resp,
        },
        other => success(
            id,
            tool_call_result(
                &format!("unknown tool: {other:?} (emit_event, enqueue_dispatch, get_next_dispatch, complete_dispatch, cancel_dispatch, request_acceptance, cancel_acceptance, get_acceptance_lineage, recall, remember, promote, forget, resolve, get_dispatch, query_ledger)"),
                true,
            ),
        ),
    }
}

async fn call_emit(db: &SeatloomDb, id: Value, arguments: Value) -> Value {
    let args: emit::EmitArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return success(
                id,
                tool_call_result(&format!("emit_event rejected: bad arguments: {e}"), true),
            )
        }
    };
    match emit::emit(db, args).await {
        Ok(summary) => success(
            id,
            tool_call_result(
                &format!(
                    "emitted {} event {} -> projection {}",
                    summary.event_type, summary.event_id, summary.projection
                ),
                false,
            ),
        ),
        Err(message) => success(id, tool_call_result(&message, true)),
    }
}

async fn call_enqueue(pool: &Pool, id: Value, arguments: Value) -> Value {
    let args: dispatch::EnqueueArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return success(
                id,
                tool_call_result(
                    &format!("enqueue_dispatch rejected: bad arguments: {e}"),
                    true,
                ),
            )
        }
    };
    match dispatch::enqueue(pool, args).await {
        Ok(msg) => success(id, tool_call_result(&msg, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

async fn call_get_next(pool: &Pool, id: Value, arguments: Value) -> Value {
    // The only accepted argument is an optional project_id; identity is NEVER a
    // parameter (non-forgeable seat identity).
    let project_id = arguments
        .get("project_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    match dispatch::get_next(pool, project_id).await {
        Ok(msg) => success(id, tool_call_result(&msg, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

async fn call_complete(pool: &Pool, id: Value, arguments: Value) -> Value {
    let args: dispatch::CompleteArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return success(
                id,
                tool_call_result(
                    &format!("complete_dispatch rejected: bad arguments: {e}"),
                    true,
                ),
            )
        }
    };
    match dispatch::complete(pool, args).await {
        Ok(msg) => success(id, tool_call_result(&msg, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

async fn call_cancel(pool: &Pool, id: Value, arguments: Value) -> Value {
    let args: dispatch::CancelArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return success(
                id,
                tool_call_result(
                    &format!("cancel_dispatch rejected: bad arguments: {e}"),
                    true,
                ),
            )
        }
    };
    match dispatch::cancel(pool, args).await {
        Ok(msg) => success(id, tool_call_result(&msg, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

async fn call_request_acceptance(pool: &Pool, id: Value, arguments: Value) -> Value {
    let args: dispatch::RequestAcceptanceArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return success(
                id,
                tool_call_result(
                    &format!("request_acceptance rejected: bad arguments: {e}"),
                    true,
                ),
            )
        }
    };
    match dispatch::request_acceptance(pool, args).await {
        Ok(msg) => success(id, tool_call_result(&msg, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

/// The lineage READ. Takes the shared `SeatloomDb` (not the pool) because it
/// writes nothing — the same read handle `emit_event` uses.
async fn call_get_acceptance_lineage(
    db: &seatloom_core::db::SeatloomDb,
    id: Value,
    arguments: Value,
) -> Value {
    let args: dispatch::GetAcceptanceLineageArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return success(
                id,
                tool_call_result(
                    &format!("get_acceptance_lineage rejected: bad arguments: {e}"),
                    true,
                ),
            )
        }
    };
    match dispatch::get_acceptance_lineage(db, args).await {
        Ok(json) => success(id, tool_call_result(&json, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

/// Parse one memory tool's arguments, or return the rendered error response.
/// The tool functions each take their own argument type, so the parsing is
/// generic and the dispatch is explicit — no higher-ranked trait bound games.
fn memory_args<A: serde::de::DeserializeOwned>(id: &Value, arguments: Value) -> Result<A, Value> {
    match serde_json::from_value::<A>(arguments) {
        Ok(a) => Ok(a),
        Err(e) => Err(success(
            id.clone(),
            tool_call_result(&format!("memory tool rejected: bad arguments: {e}"), true),
        )),
    }
}

/// Render a memory tool's outcome: the typed rejection verbatim on failure.
fn render_memory(id: Value, outcome: Result<String, memory::MemoryRejection>) -> Value {
    match outcome {
        Ok(msg) => success(id, tool_call_result(&msg, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

/// The same pair for the two pure reads.
fn read_args<A: serde::de::DeserializeOwned>(id: &Value, arguments: Value) -> Result<A, Value> {
    match serde_json::from_value::<A>(arguments) {
        Ok(a) => Ok(a),
        Err(e) => Err(success(
            id.clone(),
            tool_call_result(&format!("read tool rejected: bad arguments: {e}"), true),
        )),
    }
}

fn render_read(id: Value, outcome: Result<String, read::ReadRejection>) -> Value {
    match outcome {
        Ok(msg) => success(id, tool_call_result(&msg, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

async fn call_cancel_acceptance(pool: &Pool, id: Value, arguments: Value) -> Value {
    let args: dispatch::CancelAcceptanceArgs = match serde_json::from_value(arguments) {
        Ok(a) => a,
        Err(e) => {
            return success(
                id,
                tool_call_result(
                    &format!("cancel_acceptance rejected: bad arguments: {e}"),
                    true,
                ),
            )
        }
    };
    match dispatch::cancel_acceptance(pool, args).await {
        Ok(msg) => success(id, tool_call_result(&msg, false)),
        Err(e) => success(id, tool_call_result(&e.to_string(), true)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A notification line produces no response — the protocol forbids it.
    #[tokio::test]
    async fn notification_produces_no_response() {
        // No DB needed: notifications short-circuit before any DB contact. Use a
        // pool that will never be queried.
        let line = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        match classify(line) {
            Inbound::Notification { .. } => {}
            _ => panic!("expected notification classification"),
        }
    }

    /// `initialize` / `tools/list` / `ping` do not touch the DB and must answer.
    #[test]
    fn protocol_methods_build_a_success_envelope_without_db() {
        let init = success(
            Value::from(1),
            initialize_result(&serde_json::json!({"protocolVersion": "2024-11-05"})),
        );
        assert_eq!(init["result"]["serverInfo"]["name"], "seatloom-mcp-emit");
        let list = success(Value::from(2), tools_list_result());
        assert_eq!(list["result"]["tools"][0]["name"], "emit_event");
    }

    #[test]
    fn method_not_found_is_a_jsonrpc_error() {
        let e = error(Value::from(9), METHOD_NOT_FOUND, "method not found: foo");
        assert_eq!(e["error"]["code"], METHOD_NOT_FOUND);
    }
}
