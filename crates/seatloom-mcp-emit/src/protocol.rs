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

/// `tools/list` result: the step-4 `emit_event` tool plus the three step-6
/// governed-dispatch tools, each with its JSON Schema. Descriptions carry the
/// relevant contract rules so a seat reading the tool sees the same rules every
/// artifact states.
pub fn tools_list_result() -> Value {
    json!({
        "tools": [
            emit_event_tool(),
            enqueue_dispatch_tool(),
            get_next_dispatch_tool(),
            complete_dispatch_tool(),
            cancel_dispatch_tool(),
            request_acceptance_tool(),
            cancel_acceptance_tool(),
            get_acceptance_lineage_tool(),
        ]
    })
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
                    "description": "Event type. One of the 22 PascalCase values.",
                    "enum": [
                        "SessionStarted", "WorkItemCreated", "WorkItemStatusChanged",
                        "ArtifactCreated", "HandoffSent", "HandoffAccepted",
                        "HandoffCompleted", "ReviewVerdictIssued", "SeatDelegationIssued",
                        "SeatDelegationClosed", "SupervisorMessage", "SeatResponse",
                        "SeatDispatchIssued", "SeatDispatchDelivered", "SeatDispatchCompleted",
                        "SeatDispatchFailed", "SeatDispatchRetried", "SeatDispatchCancelled",
                        "AcceptanceRequested", "AcceptanceGranted", "AcceptanceRejected",
                        "AcceptanceCancelled"
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
                                    "seat", "topic", "project", "delegation", "dispatch",
                                    "acceptance"
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

/// Shared enum of the dependency kinds, for the dispatch tools' schemas.
const DEP_KIND_ENUM: [&str; 3] = ["event", "artifact", "dispatch"];

fn enqueue_dispatch_tool() -> Value {
    json!({
        "name": "enqueue_dispatch",
        "description": "Enqueue a governed dispatch into a seat's inbox (issuer: Aegis / Lyra). \
            Inserts the dispatches row + its dependency rows and appends SeatDispatchIssued in ONE \
            transaction. The dispatch is 'ready' iff every dependency's referent already exists, \
            else 'blocked' (blocked is NOT an error). Delivery is by PULL only (get_next_dispatch); \
            there is no mid-turn injection.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["target_seat_id", "title", "payload"],
            "properties": {
                "target_seat_id": { "type": "string", "minLength": 1, "description": "The seat that must act (seats.id)." },
                "title": { "type": "string", "minLength": 1, "description": "One-line routing summary." },
                "payload": { "type": "object", "description": "Packet path + instructions, e.g. {\"packet\": \"docs/.../X.md\"}." },
                "priority": { "type": "integer", "description": "Higher pulls first; ties by issued_at. Default 0." },
                "project_id": { "type": "string", "description": "Optional; defaults to SEATLOOM_PROJECT_ID env." },
                "id": { "type": "string", "description": "Optional dispatch id; generated when absent." },
                "dependencies": {
                    "type": "array",
                    "description": "Graph edges AS DATA. The dispatch is ready iff every referent exists.",
                    "items": {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["dep_kind", "dep_value"],
                        "properties": {
                            "dep_kind": { "type": "string", "enum": DEP_KIND_ENUM },
                            "dep_value": { "type": "string", "minLength": 1, "description": "canonical_events.id | artifacts.id | dispatches.id" }
                        }
                    }
                },
                "max_retries": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Retry budget (013): how many FAILED attempts are re-opened to ready before the failure is terminal. 0 (default) = the first failure is terminal."
                }
            }
        }
    })
}

fn get_next_dispatch_tool() -> Value {
    json!({
        "name": "get_next_dispatch",
        "description": "Pull THIS seat's next ready, highest-priority dispatch when idle (end of turn), \
            mark it in_flight, and append SeatDispatchDelivered in ONE transaction. The result carries \
            the dispatch's TITLE and PAYLOAD — the payload IS your task instructions (e.g. a packet path \
            or an inline note); do exactly what it says, then call complete_dispatch with the dispatch_id. \
            Blocked items are \
            SKIPPED (never head-of-line-block). If the seat already holds an in_flight dispatch, the \
            SAME one is returned (one in_flight per seat is a database fact). A cross-seat cycle or an \
            all-blocked stall is detected and escalated (SupervisorMessage + daily-log line), never hung. \
            IDENTITY IS NOT A PARAMETER: the calling seat is resolved from the server's SEATLOOM_ACTOR_REF; \
            there is no seat_id input to spoof.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "project_id": { "type": "string", "description": "Optional; defaults to SEATLOOM_PROJECT_ID env." }
            }
        }
    })
}

fn complete_dispatch_tool() -> Value {
    json!({
        "name": "complete_dispatch",
        "description": "Close THIS seat's in_flight dispatch and append the matching event in ONE \
            transaction. outcome='done' (default, backward compatible) -> done + SeatDispatchCompleted. \
            outcome='failed' (+ optional reason) applies the 013 retry budget: retry_count < max_retries \
            re-opens the dispatch to ready (+SeatDispatchRetried, any eligible seat may pull it again); \
            otherwise it fails terminally (+SeatDispatchFailed). The seat can only complete its own \
            in_flight dispatch (identity from SEATLOOM_ACTOR_REF, ownership + in_flight state checked \
            in SQL).",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["dispatch_id"],
            "properties": {
                "dispatch_id": { "type": "string", "minLength": 1, "description": "The in_flight dispatch to complete." },
                "outcome": { "type": "string", "enum": ["done", "failed"], "description": "Default 'done'. 'failed' reports the attempt failed (013 retry/failure path)." },
                "reason": { "type": "string", "description": "Optional failure reason; carried verbatim into the SeatDispatchRetried / SeatDispatchFailed payload." }
            }
        }
    })
}

fn cancel_dispatch_tool() -> Value {
    json!({
        "name": "cancel_dispatch",
        "description": "Withdraw a non-terminal dispatch (issuer side, 013): ready|blocked|in_flight -> \
            cancelled (terminal) + SeatDispatchCancelled in ONE transaction. Guarded on \
            issued_by = the resolved SEATLOOM_ACTOR_REF — only the issuer may cancel, and terminal rows \
            are immutable. Cancelling an in_flight dispatch releases the one-in-flight slot (the work \
            is abandoned, which is the intended semantics).",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["id"],
            "properties": {
                "id": { "type": "string", "minLength": 1, "description": "The dispatch to withdraw." }
            }
        }
    })
}

fn request_acceptance_tool() -> Value {
    json!({
        "name": "request_acceptance",
        "description": "Hand a COMPLETED delivery to the D2 acceptance gate (014): insert the \
            acceptances row + append AcceptanceRequested in ONE transaction. The seat NEVER merges its \
            own work — an operator later accepts (merge into the named base, via the Tauri command) or \
            rejects (auto-enqueued rework). Guards: the merge base must be NAMED (the `base` argument, \
            else SEATLOOM_INTEGRATION_BRANCH) and must exist as a local branch — never the old \
            hardcoded 'track', which is not a branch in this repository; the dispatch must belong to the \
            calling seat and be done; flux_verdict_ref must name an existing ReviewVerdictIssued event \
            or artifact; the dispatch must have no pending acceptance; a valid acceptance_plan (shape v1) \
            is REQUIRED — a missing or malformed plan is a typed rejection before any DB contact. \
            Identity from SEATLOOM_ACTOR_REF (never a parameter).",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["dispatch_id", "branch", "how_to_verify", "flux_verdict_ref", "acceptance_plan"],
            "properties": {
                "dispatch_id": { "type": "string", "minLength": 1, "description": "The seat's COMPLETED dispatch being handed over." },
                "branch": { "type": "string", "minLength": 1, "description": "The delivery branch to merge on accept, e.g. packet/<seat>/<id>." },
                "how_to_verify": { "type": "string", "minLength": 1, "description": "The one-step verification for the operator: 'do A, see B'." },
                "flux_verdict_ref": { "type": "string", "minLength": 1, "description": "The Flux green verdict: a ReviewVerdictIssued event or artifact id. Must exist." },
                "base": { "type": "string", "minLength": 1, "description": "The merge base — the integration branch this delivery branched from (here track/infra-foundation). Optional only because SEATLOOM_INTEGRATION_BRANCH can supply it; if neither names it the request is refused. Validated to exist as a local branch at request time." },
                "acceptance_plan": {
                    "type": "object",
                    "description": "The structured acceptance plan (shape v1, PO-authored, immutable after the request). EXACTLY these keys: plan_version (integer, must be 1); scenario (non-empty string); steps (non-empty array of {action non-empty, input optional}); expected (non-empty array of {output non-empty, expect non-empty}); preconditions (optional array of non-empty strings). An unknown key or an empty object is a typed rejection before any DB contact — no plan, no card.",
                    "required": ["plan_version", "scenario", "steps", "expected"],
                    "properties": {
                        "plan_version": { "type": "integer", "description": "Must be 1." },
                        "scenario": { "type": "string", "minLength": 1 },
                        "steps": {
                            "type": "array", "minItems": 1,
                            "items": {
                                "type": "object",
                                "required": ["action"],
                                "properties": {
                                    "action": { "type": "string", "minLength": 1 },
                                    "input": { "type": "string", "minLength": 1 }
                                }
                            }
                        },
                        "expected": {
                            "type": "array", "minItems": 1,
                            "items": {
                                "type": "object",
                                "required": ["output", "expect"],
                                "properties": {
                                    "output": { "type": "string", "minLength": 1 },
                                    "expect": { "type": "string", "minLength": 1 }
                                }
                            }
                        },
                        "preconditions": { "type": "array", "items": { "type": "string", "minLength": 1 } }
                    }
                }
            }
        }
    })
}

/// `get_acceptance_lineage` — the pure-read lineage of one acceptance card
/// (acceptance-lineage milestone, Lane B). No identity parameter: a read of an
/// already-recorded card has nothing to act as, and the non-forgeable-identity
/// rule governs writes. One shared assembler with the Tauri command, so the two
/// surfaces cannot drift.
fn get_acceptance_lineage_tool() -> Value {
    json!({
        "name": "get_acceptance_lineage",
        "description": "Assemble one acceptance card's lineage (READ ONLY, zero writes): the \
            card itself, the dispatch it gates (title/issuer/when/original task payload), the full \
            event chain in occurred_at order, the Flux verdict with its evidence, the artifact/ \
            document products, the dispatch's dependency edges with each referent's existence, and \
            the rework dispatch when the card was rejected. Answers 'what is this card and where \
            did it come from' without psql. An unknown acceptance id is a typed rejection, not an \
            empty card.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["acceptance_id"],
            "properties": {
                "acceptance_id": { "type": "string", "minLength": 1, "description": "The acceptance to assemble, acc-<dispatch-id>." }
            }
        }
    })
}

/// `cancel_acceptance` — the seat pulls back its OWN pending request. The
/// acceptance-side mirror of `cancel_dispatch` (013): the actor that created the
/// row may withdraw it while it is non-terminal, and the reason is mandatory so
/// the ledger records WHY the card is no longer being offered.
fn cancel_acceptance_tool() -> Value {
    json!({
        "name": "cancel_acceptance",
        "description": "Withdraw the calling seat's OWN pending acceptance request (D2 follow-on): \
            pending -> cancelled (terminal) + AcceptanceCancelled in ONE transaction. No merge, no \
            rework — the card simply stops being offered to the operator, with the reason recorded. \
            Use it when the card was satisfied by other means (e.g. the work already landed through a \
            bootstrap install) instead of hand-editing the ledger. Guarded on the acceptance being \
            pending AND gating a dispatch owned by the resolved SEATLOOM_ACTOR_REF — terminal rows are \
            immutable, so a decided card can never be re-cancelled.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["acceptance_id", "reason"],
            "properties": {
                "acceptance_id": { "type": "string", "minLength": 1, "description": "The acceptance to withdraw, acc-<dispatch-id>." },
                "reason": { "type": "string", "minLength": 1, "description": "One line, recorded verbatim on the row and in the AcceptanceCancelled event payload." }
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
    fn tools_list_advertises_the_eight_tools() {
        let r = tools_list_result();
        let tools = r["tools"].as_array().expect("tools array");
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            vec![
                "emit_event",
                "enqueue_dispatch",
                "get_next_dispatch",
                "complete_dispatch",
                "cancel_dispatch",
                "request_acceptance",
                "cancel_acceptance",
                "get_acceptance_lineage"
            ]
        );
        // get_next_dispatch exposes NO seat identity parameter (non-forgeable).
        let get_next = &tools[2]["inputSchema"]["properties"];
        assert!(get_next.get("seat_id").is_none());
        assert!(get_next.get("target_seat_id").is_none());
        // request_acceptance likewise takes no seat parameter — identity is
        // resolved from SEATLOOM_ACTOR_REF server-side (014).
        let req = tools[5]["inputSchema"]["properties"]
            .as_object()
            .expect("properties object");
        assert!(!req.contains_key("seat_id"));
        assert!(!req.contains_key("actor_ref"));
        // cancel_acceptance is guarded on the same server-side identity.
        let cancel = tools[6]["inputSchema"]["properties"]
            .as_object()
            .expect("properties object");
        assert!(!cancel.contains_key("seat_id"));
        assert!(!cancel.contains_key("actor_ref"));
        assert!(cancel.contains_key("reason"), "the reason is mandatory");
        // The lineage READ exposes no identity either — it reads a recorded card.
        let lineage = tools[7]["inputSchema"]["properties"]
            .as_object()
            .expect("properties object");
        assert!(!lineage.contains_key("seat_id"));
        assert!(!lineage.contains_key("actor_ref"));
        // request_acceptance REQUIRES the plan on the wire (016).
        let req = &tools[5]["inputSchema"];
        assert!(
            req["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "acceptance_plan"),
            "acceptance_plan is required: {:?}",
            req["required"]
        );
    }

    #[test]
    fn tool_schema_enumerates_all_22_event_types_and_11_ref_types() {
        let tool = emit_event_tool();
        let types = tool["inputSchema"]["properties"]["type"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(types.len(), 22);
        let ref_types = tool["inputSchema"]["properties"]["refs"]["items"]["properties"]
            ["ref_type"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(ref_types.len(), 11);
        assert!(ref_types.iter().any(|v| v == "delegation"));
        assert!(ref_types.iter().any(|v| v == "acceptance"));
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
