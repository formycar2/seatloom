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
            // --- D5 knowledge layer (Lane B) ---
            recall_tool(),
            remember_tool(),
            promote_tool(),
            forget_tool(),
            resolve_tool(),
            get_dispatch_tool(),
            query_ledger_tool(),
            open_topic_tool(),
            get_knowledge_tree_tool(),
            get_topic_memories_tool(),
            crate::intent::plan_intent_tool(),
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
                    "description": "Event type. One of the 28 PascalCase values.",
                    "enum": [
                        "SessionStarted", "WorkItemCreated", "WorkItemStatusChanged",
                        "ArtifactCreated", "HandoffSent", "HandoffAccepted",
                        "HandoffCompleted", "ReviewVerdictIssued", "SeatDelegationIssued",
                        "SeatDelegationClosed", "SupervisorMessage", "SeatResponse",
                        "SeatDispatchIssued", "SeatDispatchDelivered", "SeatDispatchCompleted",
                        "SeatDispatchFailed", "SeatDispatchRetried", "SeatDispatchCancelled",
                        "AcceptanceRequested", "AcceptanceGranted", "AcceptanceRejected",
                        "AcceptanceCancelled",
                        "MemoryCreated", "MemoryRecalled", "MemoryPromoted",
                        "MemoryForgotten", "MemoryResolved", "TopicOpened"
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
                },
                "verdict_event_id": {
                    "type": "string",
                    "pattern": "^ev-.+",
                    "description": "Only when payload.kind is 'verification' (Aegis ruling 2026-10-11 part 2). The pre-declared id of the ReviewVerdictIssued event this request asks for: the tool MINTS one when you omit it and VALIDATES yours when you supply it, then returns it in the result. The verifier MUST emit its verdict with exactly this id — hang your gated self-dispatch on dep_kind=event, dep_value=<that id> so the existing blocked -> ready trigger wakes you. Supplying it for a non-verification payload is refused."
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

/// `recall` — progressive-disclosure retrieval. Depth defaults to l0; the
/// topic's ancestor chain rides along so a memory is never read out of context.
fn recall_tool() -> Value {
    json!({
        "name": "recall",
        "description": "Recall a project+topic's ACTIVE memories with progressive disclosure: \
            depth l0 (default) returns only the ~100-token summaries, l1 adds the ~2K overview, l2 \
            adds the raw pointer (document id or file path). The response carries the topic's \
            ANCESTOR CHAIN (topics.parent_topic_id) so a memory is never read out of context. \
            Optional text query over title + l0_summary. Side effect: exactly one MemoryRecalled \
            event per returned memory (heat+1, recall_count+1) — every recall is audited; zero hits \
            append zero events. Identity from SEATLOOM_ACTOR_REF (never a parameter).",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["topic_id"],
            "properties": {
                "project_id": { "type": "string", "minLength": 1, "description": "Optional only because SEATLOOM_PROJECT_ID can supply it; a project is never fabricated." },
                "topic_id": { "type": "string", "minLength": 1, "description": "The topic to recall from. project + topic is the ONLY scoping framework — there is no all-projects recall." },
                "query": { "type": "string", "description": "Text filter over title + l0_summary. Omitted = every active memory of the topic." },
                "depth": { "type": "string", "enum": ["l0", "l1", "l2"], "description": "Which layer to return. Default l0. An unknown depth is a typed rejection." },
                "include_superseded": { "type": "boolean", "default": false, "description": "Include memories that a newer one superseded (018). They are excluded by default — a stale decision served as current is worse than no memory — and each included row carries superseded_by." },
                "limit": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Page size, default 20, hard ceiling 100. Ordered by heat DESC then updated_at DESC." }
            }
        }
    })
}

/// `remember` — create a memory. Tier starts cold and status starts active; the
/// caller cannot specify either.
fn remember_tool() -> Value {
    json!({
        "name": "remember",
        "description": "Create a memory (small-granularity distilled knowledge, scoped by project + \
            topic) and append MemoryCreated in ONE transaction. l0_summary is required and is the \
            only default layer; l1_overview and the l2 pointer are optional. The l2 pointer pair must \
            match: l2_ref_kind 'document' requires l2_document_id (and no file path), 'file' requires \
            l2_file_path (and no document id). source_kind/source_ref record provenance. Tier starts \
            'cold', status starts 'active' — neither is caller-settable. Identity from \
            SEATLOOM_ACTOR_REF (never a parameter).",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["topic_id", "title", "l0_summary", "l2_ref_kind", "source_kind", "source_ref", "kind", "confidence"],
            "properties": {
                "project_id": { "type": "string", "minLength": 1, "description": "Optional only because SEATLOOM_PROJECT_ID can supply it." },
                "topic_id": { "type": "string", "minLength": 1 },
                "title": { "type": "string", "minLength": 1 },
                "l0_summary": { "type": "string", "minLength": 1, "description": "~100 tokens; the only default layer." },
                "l1_overview": { "type": "string", "description": "~2K tokens; fetched only when a recall asks for depth l1." },
                "l2_ref_kind": { "type": "string", "enum": ["document", "file"] },
                "l2_document_id": { "type": "string", "description": "Required when l2_ref_kind is 'document'." },
                "l2_file_path": { "type": "string", "description": "Required when l2_ref_kind is 'file'; repo-relative." },
                "l2_body_digest": { "type": "string", "description": "sha256 hex of the referenced body, documents.body_digest style." },
                "source_kind": { "type": "string", "enum": ["archived_session", "canonical_event", "document", "manual"] },
                "source_ref": { "type": "string", "minLength": 1, "description": "Polymorphic provenance: '<harness>:<id>' for archived_session, an event id, a doc id, or an actor note. No FK — recorded, never guessed." },
                "kind": { "type": "string", "enum": ["decision", "failure", "fact", "preference", "artifact", "unresolved"], "description": "REQUIRED (018). The record type; the DB default is a backfill only, never a write path." },
                "confidence": { "type": "string", "enum": ["quoted", "inferred"], "description": "REQUIRED (018). Binary, never a score: quoted = the raw text says it, inferred = derived." },
                "supersedes": { "type": "string", "description": "The memory this one replaces (018). Create-time only; the edge is written in BOTH spellings (the column and a memory_relations row) in the SAME transaction." },
                "l2_line_start": { "type": "integer", "description": "First line of the L2 evidence (1-based). REQUIRED when l2_ref_kind='file' AND source_kind='archived_session'." },
                "l2_line_end": { "type": "integer", "description": "Last line of the L2 evidence (>= l2_line_start)." },
                "relations": {
                    "type": "array",
                    "description": "Extra typed edges to other memories (018). Each target must already exist; a duplicate edge in one call is refused.",
                    "items": {
                        "type": "object",
                        "required": ["to_id", "rel"],
                        "properties": {
                            "to_id": { "type": "string", "minLength": 1 },
                            "rel": { "type": "string", "enum": ["supersedes", "caused_by", "fixed_by", "refines"] }
                        }
                    }
                },
                "tags": { "type": "array", "items": { "type": "string" } }
            }
        }
    })
}

/// `promote` — the ONLY tier channel. Explicit target tier, up OR down; there is
/// no automatic decay anywhere.
fn promote_tool() -> Value {
    json!({
        "name": "promote",
        "description": "Change a memory's tier — the ONLY channel, and the only way a tier ever \
            moves. to_tier may be higher OR lower than the current tier; naming the tier the memory \
            already has is a typed refusal (no transition, nothing written). Appends MemoryPromoted \
            with from_tier/to_tier in ONE transaction. There is NO automatic heat decay and NO \
            background daemon. Identity from SEATLOOM_ACTOR_REF (never a parameter).",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["memory_id", "to_tier"],
            "properties": {
                "memory_id": { "type": "string", "minLength": 1 },
                "to_tier": { "type": "string", "enum": ["cold", "warm", "hot"] }
            }
        }
    })
}

/// `forget` — the soft tombstone. The row is never hard-deleted.
fn forget_tool() -> Value {
    json!({
        "name": "forget",
        "description": "Soft-forget a memory: status active -> forgotten, the reason recorded on the \
            row and in the MemoryForgotten event, ONE transaction. The row is NEVER hard-deleted \
            (ledger discipline), and a second forget of a tombstone is refused. A forgotten memory is \
            excluded from recall. Identity from SEATLOOM_ACTOR_REF (never a parameter).",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["memory_id", "reason"],
            "properties": {
                "memory_id": { "type": "string", "minLength": 1 },
                "reason": { "type": "string", "minLength": 1, "description": "Mandatory: a tombstone with no recorded why is an unexplained edit to the ledger." }
            }
        }
    })
}

/// `get_dispatch` — the dispatch read model. Pure read, no identity.
/// `resolve` — the G3 closure path (018): the only way an `unresolved` record
/// becomes `resolved`. No background sweep, ever.
fn resolve_tool() -> Value {
    json!({
        "name": "resolve",
        "description": "Close an ACTIVE memory: active -> resolved + MemoryResolved in ONE transaction, \
            with resolved_by taken from the server-side SEATLOOM_ACTOR_REF (never a parameter) and \
            resolved_at = now. This is the ONLY closure path — there is no background sweep and no \
            automatic anything. Refused: an unknown id, an already-resolved record (immutable), a \
            forgotten tombstone (immutable). Identity from SEATLOOM_ACTOR_REF (never a parameter).",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["memory_id"],
            "properties": {
                "memory_id": { "type": "string", "minLength": 1 }
            }
        }
    })
}

/// `open_topic` — the governed creation of a recall scope (Aegis ruling
/// 2026-10-11). `remember` cannot create one implicitly: a typo would open a
/// scope nobody ever recalls from. `opened_by` is server-side, never a parameter.
fn open_topic_tool() -> Value {
    json!({
        "name": "open_topic",
        "description": "Open a recall scope (topic) — idempotently. A second open of the same \
            topic_id succeeds, changes nothing and emits no event. opened_by is resolved from \
            SEATLOOM_ACTOR_REF server-side (never a parameter). Refusals (typed, zero writes): \
            the parent topic does not exist (open the parent first — a scope is never created \
            implicitly); the project does not exist; the parent is in another project; the parent \
            would make the new topic its own ancestor. On success the row and one TopicOpened \
            event land in ONE transaction.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["topic_id", "title"],
            "properties": {
                "topic_id": { "type": "string", "minLength": 1, "description": "The scope id, e.g. 'd5-knowledge-layer'. Stable: memories reference it forever." },
                "title": { "type": "string", "minLength": 1 },
                "project_id": { "type": "string", "minLength": 1, "description": "Optional only because SEATLOOM_PROJECT_ID can supply it; a project is never fabricated." },
                "parent_topic_id": { "type": "string", "description": "An existing topic in the SAME project. Omit for a root scope." }
            }
        }
    })
}

/// `get_knowledge_tree` — the topic tree of a project, with memory counts.
/// PURE READ: no write, no event (a panel query, not a recall).
fn get_knowledge_tree_tool() -> Value {
    json!({
        "name": "get_knowledge_tree",
        "description": "The knowledge topic tree for a project: every topic with its parent and its \
            ACTIVE, not-superseded memory count. PURE READ — writes nothing, appends no event (a \
            panel query, not a recall; recall keeps its MemoryRecalled audit). Scope is hard: one \
            project, never an all-projects read. An empty project is an empty tree, not an error.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "project_id": { "type": "string", "minLength": 1, "description": "Optional only because SEATLOOM_PROJECT_ID can supply it; a project is never fabricated." }
            }
        }
    })
}

/// `get_topic_memories` — the memories of one topic, progressively disclosed.
/// PURE READ: no write, no event.
fn get_topic_memories_tool() -> Value {
    json!({
        "name": "get_topic_memories",
        "description": "The memories of one topic at a disclosure depth, WITH the topic's ancestor \
            chain (a memory is never returned out of its context). depth l0 (default) returns only \
            the ~100-token summaries; l1 adds the overview; l2 adds the evidence pointer with the \
            line range. The token budget belongs to the caller. Superseded rows are excluded unless \
            include_superseded is set, and then each carries supersededBy. PURE READ — writes \
            nothing, appends no event.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["topic_id"],
            "properties": {
                "topic_id": { "type": "string", "minLength": 1 },
                "depth": { "type": "string", "enum": ["l0", "l1", "l2"], "description": "Which layer to return. Default l0. An unknown depth is a typed refusal." },
                "include_superseded": { "type": "boolean", "default": false, "description": "Include memories a newer one superseded; each is annotated with supersededBy." },
                "limit": { "type": "integer", "minimum": 1, "maximum": 200, "description": "Page size, default 50, hard ceiling 200. Ordered by heat DESC then updated_at DESC." }
            }
        }
    })
}

fn get_dispatch_tool() -> Value {
    json!({
        "name": "get_dispatch",
        "description": "One dispatch's read model: the row (including the ORIGINAL task statement in \
            payload), its LIVE status (recomputed from its dependencies' existence, the same rule the \
            pull decision uses), and each dependency edge with its referent's existence. Pure read — \
            zero writes, zero events, no write capability. Coordination state is visible to the fleet, \
            so no seat identity is required.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["dispatch_id"],
            "properties": {
                "dispatch_id": { "type": "string", "minLength": 1 }
            }
        }
    })
}

/// `query_ledger` — a filtered ledger query. Pure read, no identity.
fn query_ledger_tool() -> Value {
    json!({
        "name": "query_ledger",
        "description": "Query the canonical ledger: canonical_events with their event_object_refs, \
            filtered by project, event types (validated against the closed vocabulary), a ref \
            (ref_type + ref_id together), and an occurred_at window; newest first, paged. Pure read — \
            zero writes, zero events. This is the tool that ends the psql era.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "project_id": { "type": "string", "minLength": 1 },
                "event_types": { "type": "array", "items": { "type": "string" }, "description": "Event types from the closed vocabulary; an unknown type is a typed rejection." },
                "ref_type": { "type": "string", "description": "With ref_id: filter to events naming that object." },
                "ref_id": { "type": "string" },
                "from": { "type": "string", "description": "RFC3339 inclusive lower bound on occurred_at." },
                "to": { "type": "string", "description": "RFC3339 inclusive upper bound on occurred_at." },
                "limit": { "type": "integer", "minimum": 1, "maximum": 200, "description": "Default 50, hard ceiling 200." },
                "offset": { "type": "integer", "minimum": 0 }
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
    fn tools_list_advertises_the_nineteen_tools() {
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
                "get_acceptance_lineage",
                "recall",
                "remember",
                "promote",
                "forget",
                "resolve",
                "get_dispatch",
                "query_ledger",
                "open_topic",
                "get_knowledge_tree",
                "get_topic_memories",
                "plan_intent",
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
        // The five D5 memory tools expose no identity either — identity is
        // resolved server-side from SEATLOOM_ACTOR_REF.
        for name in [
            "recall",
            "remember",
            "promote",
            "forget",
            "resolve",
            "open_topic",
        ] {
            let t = tools
                .iter()
                .find(|t| t["name"] == name)
                .unwrap_or_else(|| panic!("{name} must be advertised"));
            let props = t["inputSchema"]["properties"].as_object().expect("props");
            assert!(
                !props.contains_key("seat_id") && !props.contains_key("actor_ref"),
                "{name} must not expose identity"
            );
        }
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
        // remember REQUIRES kind + confidence on the wire (018).
        let remember = &tools[9]["inputSchema"];
        for field in ["kind", "confidence"] {
            assert!(
                remember["required"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|v| v == field),
                "remember must require {field}: {:?}",
                remember["required"]
            );
        }
    }

    #[test]
    fn tool_schema_enumerates_all_28_event_types_and_11_ref_types() {
        let tool = emit_event_tool();
        let types = tool["inputSchema"]["properties"]["type"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(types.len(), 28);
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
