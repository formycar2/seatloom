//! The `emit_event` tool: the only sanctioned way a seat writes a governed
//! event. Parses an MCP tool call into an [`EventDraft`] and routes it through
//! [`SeatloomDb::append_event`] (A1). It **never** mutates a projection, never
//! inserts into `canonical_events` directly, never bypasses `validate()` — the
//! whole point is that there is exactly one write path and this is a thin,
//! rejecting front door onto it.
//!
//! # Value-domain contract (verbatim — NIMBUS-2026-10-08-p0-step4-mcp-emit-v1 §1)
//!
//! ```text
//! EVENT TYPE (12, PascalCase — matches what is already in PostgreSQL):
//!   SessionStarted | WorkItemCreated | WorkItemStatusChanged | ArtifactCreated
//!   | HandoffSent | HandoffAccepted | HandoffCompleted | ReviewVerdictIssued
//!   | SeatDelegationIssued | SeatDelegationClosed | SupervisorMessage | SeatResponse
//!
//! REF ROLE (3, snake_case):
//!   subject | actor | context
//!   subject = what the event is ABOUT
//!   actor   = who did it
//!   context = related, WITHOUT asserting agency or aboutness (weakest claim)
//!
//! REF TYPE (9, snake_case):
//!   workitem | session | handoff | artifact | document | seat | topic | project
//!   | delegation        <-- 'delegation' is a DEVIATION from the original packet
//!                           contract, accepted: it exists in committed seed data
//!                           (ev-020/ev-021). Do not drop it and do not re-add
//!                           'ArtifactProduced' (dropped on Lyra sign-off; capture
//!                           reuses the existing 'ArtifactCreated').
//!
//! RULES:
//!   * schema_version is set EXPLICITLY, per event_type, >= 1. Never rely on
//!     010's DEFAULT 1.
//!   * role is stated EXPLICITLY on every ref. Never rely on 010's DEFAULT
//!     'context'.
//!   * EXACTLY ONE ref with role='subject' per event. 010 enforces AT MOST ONE
//!     (partial unique index); presence is the write API's job, and the MCP layer
//!     must not weaken it.
//!   * turn_seq is NULL = NOT RECORDED. Never 0. T2 capture is P1, so P0 emits
//!     pass None.
//!   * payload is REQUIRED and must be a JSON object. Which (type, version) pairs
//!     have a typed payload contract is the write API's contract, not a column
//!     constraint.
//! ```

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

use seatloom_core::db::event_domain::{EventType, RefRole, RefType};
use seatloom_core::db::write_api::{EventDraft, ObjectRefDraft};
use seatloom_core::db::SeatloomDb;

/// Env var naming the default actor when a call omits `actor_ref`. The server
/// is launched per seat, so this is how a seat gets a stable identity without
/// restating it on every call. Absence of both is a typed rejection, never a
/// fabricated actor.
pub const ACTOR_REF_ENV: &str = "SEATLOOM_ACTOR_REF";
/// Env var naming the default project when a call omits `project_id`.
pub const PROJECT_ID_ENV: &str = "SEATLOOM_PROJECT_ID";

/// Parameters of one `emit_event` call, as received over MCP. All optionality
/// and defaulting is resolved in [`EmitArgs::into_draft`], not here, so the
/// JSON shape stays a faithful mirror of the `tools/list` schema.
#[derive(Debug, Deserialize)]
pub struct EmitArgs {
    #[serde(rename = "type")]
    pub type_: String,
    pub payload: Value,
    #[serde(default)]
    pub refs: Vec<RefArg>,
    #[serde(default)]
    pub schema_version: Option<i64>,
    #[serde(default)]
    pub actor_ref: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub occurred_at: Option<String>,
    #[serde(default)]
    pub turn_seq: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RefArg {
    pub ref_type: String,
    pub ref_id: String,
    pub role: String,
}

/// A rejection produced *before* the write API is reached: a parse or
/// resolution failure in the MCP layer. Rendered verbatim into the MCP tool
/// error. Distinct from [`seatloom_core::db::write_api::WriteError`], which the
/// write API raises; both are surfaced to the caller as `isError: true`.
#[derive(Debug)]
pub enum EmitRejection {
    UnknownEventType {
        value: String,
        detail: String,
    },
    UnknownRefType {
        value: String,
        detail: String,
    },
    UnknownRole {
        value: String,
        detail: String,
    },
    SchemaVersionOutOfRange {
        schema_version: i64,
    },
    SchemaVersionNotI32 {
        schema_version: i64,
    },
    TurnSeqNegative {
        turn_seq: i64,
    },
    TurnSeqNotI32 {
        turn_seq: i64,
    },
    /// Neither the call nor `SEATLOOM_ACTOR_REF` supplied an actor. Never
    /// invented — §2.3 of the packet: "an empty/absent actor is a typed
    /// rejection, not a default".
    MissingActorRef,
    /// Neither the call nor `SEATLOOM_PROJECT_ID` supplied a project. Never
    /// fabricated.
    MissingProjectId,
    BadOccurredAt {
        value: String,
        detail: String,
    },
}

impl std::fmt::Display for EmitRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmitRejection::UnknownEventType { value, detail } => write!(
                f,
                "emit_event rejected: type {value:?} is not one of the 12 event types ({detail})"
            ),
            EmitRejection::UnknownRefType { value, detail } => write!(
                f,
                "emit_event rejected: ref_type {value:?} is not one of the 9 ref types ({detail})"
            ),
            EmitRejection::UnknownRole { value, detail } => write!(
                f,
                "emit_event rejected: role {value:?} is not one of subject|actor|context ({detail})"
            ),
            EmitRejection::SchemaVersionOutOfRange { schema_version } => write!(
                f,
                "emit_event rejected: schema_version {schema_version} must be >= 1"
            ),
            EmitRejection::SchemaVersionNotI32 { schema_version } => write!(
                f,
                "emit_event rejected: schema_version {schema_version} does not fit in i32"
            ),
            EmitRejection::TurnSeqNegative { turn_seq } => write!(
                f,
                "emit_event rejected: turn_seq {turn_seq} must be >= 0 when recorded (omit it for 'not recorded'; never send 0 as a placeholder)"
            ),
            EmitRejection::TurnSeqNotI32 { turn_seq } => write!(
                f,
                "emit_event rejected: turn_seq {turn_seq} does not fit in i32"
            ),
            EmitRejection::MissingActorRef => write!(
                f,
                "emit_event rejected: actor_ref is required (pass it in the call or set {ACTOR_REF_ENV} in the server env); an absent actor is never invented"
            ),
            EmitRejection::MissingProjectId => write!(
                f,
                "emit_event rejected: project_id is required (pass it in the call or set {PROJECT_ID_ENV} in the server env); a project is never fabricated"
            ),
            EmitRejection::BadOccurredAt { value, detail } => write!(
                f,
                "emit_event rejected: occurred_at {value:?} is not an RFC 3339 timestamp ({detail})"
            ),
        }
    }
}

impl EmitArgs {
    /// Resolve this call into an [`EventDraft`]. Pure, synchronous, no database
    /// contact — every defaulting rule lives here so it is testable without a
    /// connection. The draft is handed to `append_event`, whose `validate()`
    /// then enforces the subject/payload/field contract; this function only
    /// resolves the MCP-layer concerns (vocabulary parse, env defaults,
    /// id/time generation, integer-range narrowing).
    pub fn into_draft(self) -> Result<EventDraft, EmitRejection> {
        let event_type =
            self.type_
                .parse::<EventType>()
                .map_err(|e| EmitRejection::UnknownEventType {
                    value: self.type_.clone(),
                    detail: e.to_string(),
                })?;

        // schema_version: optional, default 1, must be >= 1 and fit in i32.
        let schema_version: i32 = match self.schema_version {
            None => 1,
            Some(v) if v < 1 => {
                return Err(EmitRejection::SchemaVersionOutOfRange { schema_version: v })
            }
            Some(v) => i32::try_from(v)
                .map_err(|_| EmitRejection::SchemaVersionNotI32 { schema_version: v })?,
        };

        // turn_seq: P0 omits it. If a caller sends it anyway, honour the domain
        // (>= 0, fits i32); 0 is a legitimate recorded ordinal, not a placeholder.
        let turn_seq: Option<i32> = match self.turn_seq {
            None => None,
            Some(v) if v < 0 => return Err(EmitRejection::TurnSeqNegative { turn_seq: v }),
            Some(v) => {
                Some(i32::try_from(v).map_err(|_| EmitRejection::TurnSeqNotI32 { turn_seq: v })?)
            }
        };

        // actor_ref: call value, else env, else reject. Never invented.
        let actor_ref =
            resolve_or_env(self.actor_ref, ACTOR_REF_ENV).ok_or(EmitRejection::MissingActorRef)?;

        // project_id: call value, else env, else reject. Never fabricated.
        let project_id = resolve_or_env(self.project_id, PROJECT_ID_ENV)
            .ok_or(EmitRejection::MissingProjectId)?;

        // event_id: generate when absent (nanoid, already a dependency).
        let event_id = match self.event_id.map(|s| s.trim().to_string()) {
            Some(s) if !s.is_empty() => s,
            _ => format!("ev-{}", nanoid::nanoid!()),
        };

        // occurred_at: RFC 3339 when supplied, else now.
        let occurred_at: DateTime<Utc> = match self.occurred_at {
            None => Utc::now(),
            Some(ref s) => DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| EmitRejection::BadOccurredAt {
                    value: s.clone(),
                    detail: e.to_string(),
                })?,
        };

        let mut refs = Vec::with_capacity(self.refs.len());
        for r in self.refs {
            let ref_type =
                r.ref_type
                    .parse::<RefType>()
                    .map_err(|e| EmitRejection::UnknownRefType {
                        value: r.ref_type.clone(),
                        detail: e.to_string(),
                    })?;
            let role = r
                .role
                .parse::<RefRole>()
                .map_err(|e| EmitRejection::UnknownRole {
                    value: r.role.clone(),
                    detail: e.to_string(),
                })?;
            refs.push(ObjectRefDraft::new(ref_type, r.ref_id, role));
        }

        Ok(EventDraft {
            event_id,
            event_type,
            schema_version,
            occurred_at,
            actor_ref,
            project_id,
            turn_seq,
            // payload is handed through verbatim; append_event's validate()
            // rejects a non-object, and the (type, version) contract rejects a
            // wrong shape. The MCP layer does not second-guess it.
            payload: self.payload,
            refs,
        })
    }
}

/// A call-supplied value wins; a blank one is treated as absent. Falls back to
/// the named env var, also treating blank as absent. Returns `None` when
/// neither yields a non-blank value — the caller turns that into a typed
/// rejection rather than inventing a default.
fn resolve_or_env(call_value: Option<String>, env_key: &str) -> Option<String> {
    if let Some(v) = call_value {
        if !v.trim().is_empty() {
            return Some(v);
        }
    }
    match std::env::var(env_key) {
        Ok(v) if !v.trim().is_empty() => Some(v),
        _ => None,
    }
}

/// Outcome of a successful emit, formatted for the MCP tool result text.
pub struct EmitSummary {
    pub event_id: String,
    pub event_type: EventType,
    pub projection: String,
}

/// Route a parsed call through the write API. On success, a one-line summary;
/// on any failure, the typed error rendered verbatim (MCP-layer rejection or
/// `WriteError`) — the caller wraps it as `isError: true`. A rejection never
/// writes a partial event: that is `append_event`'s single transaction, and
/// this function adds no second write path.
pub async fn emit(db: &SeatloomDb, args: EmitArgs) -> Result<EmitSummary, String> {
    let draft = args.into_draft().map_err(|e| e.to_string())?;
    let event_type = draft.event_type;
    let outcome = db.append_event(&draft).await.map_err(|e| e.to_string())?;
    Ok(EmitSummary {
        event_id: outcome.event_id,
        event_type,
        projection: format!("{:?}", outcome.projection),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_args() -> serde_json::Value {
        serde_json::json!({
            "type": "WorkItemCreated",
            "payload": {"title": "t", "status": "draft", "priority": "low"},
            "refs": [
                {"ref_type": "workitem", "ref_id": "wi-1", "role": "subject"},
                {"ref_type": "seat", "ref_id": "seat-nimbus-001", "role": "actor"}
            ],
            "actor_ref": "seat:seat-nimbus-001",
            "project_id": "p0-mcp-proof"
        })
    }

    fn parse(v: serde_json::Value) -> EmitArgs {
        serde_json::from_value(v).expect("args parse")
    }

    #[test]
    fn happy_draft_sets_schema_version_one_and_keeps_turn_seq_none() {
        let d = parse(base_args()).into_draft().expect("valid");
        assert_eq!(d.event_type, EventType::WorkItemCreated);
        assert_eq!(d.schema_version, 1, "default when omitted");
        assert_eq!(d.turn_seq, None, "P0 never sends turn_seq");
        assert_eq!(d.actor_ref, "seat:seat-nimbus-001");
        assert_eq!(d.project_id, "p0-mcp-proof");
        assert_eq!(d.refs.len(), 2);
        assert!(
            d.event_id.starts_with("ev-"),
            "generated id: {}",
            d.event_id
        );
    }

    #[test]
    fn supplied_event_id_and_occurred_at_are_honoured() {
        let mut v = base_args();
        v["event_id"] = serde_json::json!("ev-explicit-1");
        v["occurred_at"] = serde_json::json!("2026-10-08T12:00:00Z");
        let d = parse(v).into_draft().expect("valid");
        assert_eq!(d.event_id, "ev-explicit-1");
        assert_eq!(d.occurred_at.to_rfc3339(), "2026-10-08T12:00:00+00:00");
    }

    #[test]
    fn unknown_event_type_is_rejected_by_name() {
        let mut v = base_args();
        v["type"] = serde_json::json!("NotAThing");
        match parse(v).into_draft() {
            Err(EmitRejection::UnknownEventType { value, .. }) => assert_eq!(value, "NotAThing"),
            other => panic!("expected UnknownEventType, got {other:?}"),
        }
    }

    #[test]
    fn unknown_ref_type_and_role_are_rejected_by_name() {
        let mut v = base_args();
        v["refs"][0]["ref_type"] = serde_json::json!("widget");
        match parse(v).into_draft() {
            Err(EmitRejection::UnknownRefType { value, .. }) => assert_eq!(value, "widget"),
            other => panic!("expected UnknownRefType, got {other:?}"),
        }
        let mut v = base_args();
        v["refs"][0]["role"] = serde_json::json!("owner");
        match parse(v).into_draft() {
            Err(EmitRejection::UnknownRole { value, .. }) => assert_eq!(value, "owner"),
            other => panic!("expected UnknownRole, got {other:?}"),
        }
    }

    #[test]
    fn schema_version_zero_is_rejected_but_absence_defaults_to_one() {
        let mut v = base_args();
        v["schema_version"] = serde_json::json!(0);
        assert!(matches!(
            parse(v).into_draft(),
            Err(EmitRejection::SchemaVersionOutOfRange { schema_version: 0 })
        ));
        // absence already covered by happy path -> defaults to 1
    }

    #[test]
    fn turn_seq_negative_rejected_zero_allowed_absent_is_none() {
        let mut v = base_args();
        v["turn_seq"] = serde_json::json!(-1);
        assert!(matches!(
            parse(v).into_draft(),
            Err(EmitRejection::TurnSeqNegative { turn_seq: -1 })
        ));
        let mut v = base_args();
        v["turn_seq"] = serde_json::json!(0);
        assert_eq!(
            parse(v).into_draft().expect("0 is recorded").turn_seq,
            Some(0)
        );
    }

    #[test]
    fn missing_actor_ref_is_a_typed_rejection_not_a_default() {
        // Ensure the env fallback is not set for this assertion. The lock is
        // shared crate-wide: these tests mutate the process env, and so do
        // dispatch.rs's identity/base tests — unsynchronised, one test's
        // set_var is another test's "not set".
        let _guard = crate::dispatch::ENV_LOCK.blocking_lock();
        std::env::remove_var(ACTOR_REF_ENV);
        let mut v = base_args();
        v.as_object_mut().unwrap().remove("actor_ref");
        assert!(matches!(
            parse(v).into_draft(),
            Err(EmitRejection::MissingActorRef)
        ));
    }

    #[test]
    fn missing_project_id_is_a_typed_rejection_not_a_default() {
        let _guard = crate::dispatch::ENV_LOCK.blocking_lock();
        std::env::remove_var(PROJECT_ID_ENV);
        let mut v = base_args();
        v.as_object_mut().unwrap().remove("project_id");
        assert!(matches!(
            parse(v).into_draft(),
            Err(EmitRejection::MissingProjectId)
        ));
    }

    #[test]
    fn bad_occurred_at_is_rejected() {
        let mut v = base_args();
        v["occurred_at"] = serde_json::json!("not-a-time");
        assert!(matches!(
            parse(v).into_draft(),
            Err(EmitRejection::BadOccurredAt { .. })
        ));
    }

    #[test]
    fn blank_actor_ref_in_call_falls_through_to_rejection_without_env() {
        let _guard = crate::dispatch::ENV_LOCK.blocking_lock();
        std::env::remove_var(ACTOR_REF_ENV);
        let mut v = base_args();
        v["actor_ref"] = serde_json::json!("   ");
        assert!(matches!(
            parse(v).into_draft(),
            Err(EmitRejection::MissingActorRef)
        ));
    }
}
