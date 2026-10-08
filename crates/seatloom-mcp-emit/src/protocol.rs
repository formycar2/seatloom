//! Minimal MCP server over newline-delimited JSON-RPC 2.0 on stdio. Hand-rolled
//! per NIMBUS-2026-10-08-p0-step4-mcp-emit-v1 §2.7: a one-tool server is a small,
//! well-specified surface, so the protocol framing is implemented directly rather
//! than pulling an MCP framework crate.
//!
//! Protocol surface (just enough for pi to connect and call one tool):
//!   - `initialize`                -> capabilities + serverInfo
//!   - `notifications/initialized` -> no response (it is a notification)
//!   - `notifications/*`           -> no response
//!   - `tools/list`                -> advertise exactly one tool, `emit_event`
//!   - `tools/call`                -> invoke `emit_event`
//!   - `ping`                      -> `{}`
//!
//! Framing: one JSON value per line (LSP-style Content-Length framing is not
//! used by pi's stdio transport; newline-delimited JSON is). A request carries
//! an `id`; a notification does not, and MUST NOT be answered.

use serde_json::{json, Value};

/// The MCP protocol version this server speaks. Echoed from the client's
/// `initialize` when recognised, else this default is offered.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// Classification of one inbound line.
pub enum Inbound {
    /// A request expecting a response with this `id`.
    Request {
        id: Value,
        method: String,
        params: Value,
    },
    /// A notification — no response may be emitted.
    Notification { method: String },
    /// A line that did not parse as JSON-RPC; `id` is echoed if we could find one.
    Malformed { id: Value, message: String },
}

/// Parse one line of stdin into an [`Inbound`].
pub fn classify(line: &str) -> Inbound {
    let v: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return Inbound::Malformed {
                id: Value::Null,
                message: format!("parse error: {e}"),
            }
        }
    };
    let method = v.get("method").and_then(|m| m.as_str()).map(str::to_string);
    let has_id = v.get("id").is_some();
    let params = v.get("params").cloned().unwrap_or(Value::Null);
    match (method, has_id) {
        (Some(method), true) => Inbound::Request {
            id: v.get("id").cloned().unwrap_or(Value::Null),
            method,
            params,
        },
        (Some(method), false) => Inbound::Notification { method },
        (None, _) => Inbound::Malformed {
            id: v.get("id").cloned().unwrap_or(Value::Null),
            message: "missing method".to_string(),
        },
    }
}

/// A successful JSON-RPC result envelope.
pub fn success(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

/// A JSON-RPC error envelope.
pub fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// `initialize` result. Advertises a single `tools` capability.
pub fn initialize_result(client_params: &Value) -> Value {
    // Echo the client's protocolVersion when present so we stay in lockstep;
    // otherwise offer our default.
    let proto = client_params
        .get("protocolVersion")
        .and_then(|p| p.as_str())
        .unwrap_or(PROTOCOL_VERSION);
    json!({
        "protocolVersion": proto,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": "seatloom-mcp-emit", "version": env!("CARGO_PKG_VERSION") }
    })
}

/// `tools/list` result: exactly one tool, `emit_event`, with its JSON Schema.
/// The description carries the value-domain contract verbatim so a seat reading
/// the tool sees the same rules every artifact states.
pub fn tools_list_result() -> Value {
    json!({
        "tools": [ emit_event_tool() ] })
}

fn emit_event_tool() -> Value {
    json!({
        "name": "emit_event",
        "description": EMIT_EVENT_DESCRIPTION,
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["type", "payload", "refs"],
            "properties": {
                "type": {
                    "type": "string",
                    "description": "Event type. One of the 12 PascalCase values.",
                    "enum": [
                        "SessionStarted", "WorkItemCreated", "WorkItemStatusChanged",
                        "ArtifactCreated", "HandoffSent", "HandoffAccepted",
                        "HandoffCompleted", "ReviewVerdictIssued", "SeatDelegationIssued",
                        "SeatDelegationClosed", "SupervisorMessage", "SeatResponse"
                    ]
                },
                "payload": {
                    "type": "object",
                    "description": "Required JSON object. Must match the typed contract for (type, schema_version)."
                },
                "refs": {
                    "type": "array",
                    "description": "Object references. EXACTLY ONE must have role 'subject'.",
                    "items": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["ref_type", "ref_id", "role"],
                        "properties": {
                            "ref_type": {
                                "type": "string",
                                "enum": [
                                    "workitem", "session", "handoff", "artifact", "document",
                                    "seat", "topic", "project", "delegation"
                                ]
                            },
                            "ref_id": { "type": "string", "minLength": 1 },
                            "role": { "type": "string", "enum": ["subject", "actor", "context"] }
                        }
                    }
                },
                "schema_version": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional, default 1. Set explicitly per event_type; never rely on the DB default."
                },
                "actor_ref": {
                    "type": "string",
                    "description": "Who did it. Required unless SEATLOOM_ACTOR_REF is set in the server env. Never invented."
                },
                "project_id": {
                    "type": "string",
                    "description": "Project scope. Optional; defaults to SEATLOOM_PROJECT_ID env. Never fabricated."
                },
                "event_id": {
                    "type": "string",
                    "description": "Optional. Generated (nanoid) when absent."
                },
                "occurred_at": {
                    "type": "string",
                    "description": "Optional RFC 3339 timestamp. Defaults to now."
                },
                "turn_seq": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Optional turn ordinal. OMIT in P0 (NULL = not recorded); never send 0 as a placeholder."
                }
            }
        }
    })
}

/// A `tools/call` result carrying one text block. `is_error` maps to the MCP
/// `isError` flag so a rejection is a tool error, not a transport error.
pub fn tool_call_result(text: &str, is_error: bool) -> Value {
    json!({
        "content": [ { "type": "text", "text": text } ],
        "isError": is_error
    })
}

/// The `emit_event` tool description — the value-domain contract verbatim
/// (NIMBUS-2026-10-08-p0-step4-mcp-emit-v1 §1).
pub const EMIT_EVENT_DESCRIPTION: &str = "\
Append a governed event to the SeatLoom canonical ledger. This is the ONLY \
sanctioned way a seat writes a governed event: it routes through the write API \
(append(event) -> projection) in one transaction. A malformed call is rejected \
with a typed error and writes nothing.

VALUE-DOMAIN CONTRACT (verbatim):

EVENT TYPE (12, PascalCase — matches what is already in PostgreSQL):
  SessionStarted | WorkItemCreated | WorkItemStatusChanged | ArtifactCreated
  | HandoffSent | HandoffAccepted | HandoffCompleted | ReviewVerdictIssued
  | SeatDelegationIssued | SeatDelegationClosed | SupervisorMessage | SeatResponse

REF ROLE (3, snake_case):
  subject | actor | context
  subject = what the event is ABOUT
  actor   = who did it
  context = related, WITHOUT asserting agency or aboutness (weakest claim)

REF TYPE (9, snake_case):
  workitem | session | handoff | artifact | document | seat | topic | project
  | delegation

RULES:
  * schema_version is set EXPLICITLY, per event_type, >= 1. Never rely on 010's DEFAULT 1.
  * role is stated EXPLICITLY on every ref. Never rely on 010's DEFAULT 'context'.
  * EXACTLY ONE ref with role='subject' per event.
  * turn_seq is NULL = NOT RECORDED. Never 0. P0 emits pass None.
  * payload is REQUIRED and must be a JSON object.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_distinguishes_request_notification_and_malformed() {
        match classify(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#) {
            Inbound::Request { method, .. } => assert_eq!(method, "tools/list"),
            _ => panic!("expected request"),
        }
        match classify(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#) {
            Inbound::Notification { method } => assert_eq!(method, "notifications/initialized"),
            _ => panic!("expected notification"),
        }
        match classify("not json") {
            Inbound::Malformed { .. } => {}
            _ => panic!("expected malformed"),
        }
    }

    #[test]
    fn tools_list_advertises_exactly_one_tool_named_emit_event() {
        let r = tools_list_result();
        let tools = r["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "emit_event");
    }

    #[test]
    fn tool_schema_enumerates_all_12_event_types_and_9_ref_types() {
        let tool = emit_event_tool();
        let types = tool["inputSchema"]["properties"]["type"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(types.len(), 12);
        let ref_types = tool["inputSchema"]["properties"]["refs"]["items"]["properties"]
            ["ref_type"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(ref_types.len(), 9);
        assert!(ref_types.iter().any(|v| v == "delegation"));
        // ArtifactProduced must NOT appear.
        assert!(!types.iter().any(|v| v == "ArtifactProduced"));
    }

    #[test]
    fn tool_description_carries_the_contract_verbatim() {
        assert!(EMIT_EVENT_DESCRIPTION.contains("EXACTLY ONE ref with role='subject'"));
        assert!(EMIT_EVENT_DESCRIPTION.contains("turn_seq is NULL = NOT RECORDED"));
        assert!(EMIT_EVENT_DESCRIPTION.contains("delegation"));
    }

    #[test]
    fn tool_call_result_sets_is_error_flag() {
        let ok = tool_call_result("done", false);
        assert_eq!(ok["isError"], false);
        assert_eq!(ok["content"][0]["type"], "text");
        let bad = tool_call_result("nope", true);
        assert_eq!(bad["isError"], true);
    }

    #[test]
    fn initialize_echoes_client_protocol_version() {
        let r = initialize_result(&json!({"protocolVersion": "2025-06-18"}));
        assert_eq!(r["protocolVersion"], "2025-06-18");
        assert_eq!(r["serverInfo"]["name"], "seatloom-mcp-emit");
        let d = initialize_result(&json!({}));
        assert_eq!(d["protocolVersion"], PROTOCOL_VERSION);
    }
}
