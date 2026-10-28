//! The session↔work read tools — the MCP half of the read model.
//!
//! Product contract: `LYRA-2026-10-16-session-work-bridge-contract-v1`
//! (Lyra, `packet/lyra/milestone-summary` @ `8601167`) §11.2 slice 2, dispatch
//! `disp-session-key-readmodel`. Two tools over ONE implementation:
//! [`seatloom_core::db::session_work`]. The Tauri commands shell out to these
//! same tools, so there is no second implementation — the established
//! thin-wrapper pattern.
//!
//! # What these tools never do
//!
//! * **No transcript bodies.** Pointers and line ranges only (011's red line).
//! * **No topic id as a key.** The reverse tool resolves a memory **by key** —
//!   the declared path pair or the recorded session reference. The topic id rides
//!   along as context and is never joined on: `wash-sess-cc-621ae382` is a session
//!   id truncated to 8 characters by `s5_writein.py`, and a topic is a
//!   segmentation, a revisable label, not an identity.
//! * **No writes.** Pure reads; they append nothing and mutate nothing.
//! * **No guessing.** A database without the bridge view refuses. A session with
//!   no records says so, and keeps that state distinct from "records that do not
//!   resolve".

use serde::Deserialize;
use serde_json::{json, Value};

use seatloom_core::db::session_work::{self, SessionRef, SessionWorkError};
use seatloom_core::db::SeatloomDb;

/// Why a session↔work read was refused. Every variant names what is missing.
#[derive(Debug)]
pub enum SessionWorkRejection {
    /// The bridge view (022) is not in this database.
    ViewMissing,
    /// No `archived_sessions` row carries that key. Distinct from "the session
    /// exists and produced no records", which is an answer, not a refusal.
    SessionNotFound {
        harness: String,
        id: String,
    },
    /// The caller named a session with an empty harness or id.
    EmptyKey {
        harness: String,
        id: String,
    },
    Db(String),
}

impl From<SessionWorkError> for SessionWorkRejection {
    fn from(e: SessionWorkError) -> Self {
        use session_work::SessionWorkError as E;
        match e {
            E::ViewMissing => SessionWorkRejection::ViewMissing,
            E::SessionNotFound { harness, id } => {
                SessionWorkRejection::SessionNotFound { harness, id }
            }
            E::EmptyKey { harness, id } => SessionWorkRejection::EmptyKey { harness, id },
            E::Db(e) => SessionWorkRejection::Db(e.to_string()),
        }
    }
}

impl std::fmt::Display for SessionWorkRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionWorkRejection::ViewMissing => write!(
                f,
                "session_work rejected: the session↔work bridge is not available in this database — \
                 the view `v_memory_session_join` does not exist. Apply migration 022 first; a read \
                 that cannot measure the join must not look like a read that measured nothing."
            ),
            SessionWorkRejection::SessionNotFound { harness, id } => write!(
                f,
                "session_work rejected: no archived session ({harness:?}, {id:?}) exists — the \
                 archive is keyed by (harness, id), and this session is not in it. That is a \
                 different fact from a session that IS in the archive and produced no records."
            ),
            SessionWorkRejection::EmptyKey { harness, id } => write!(
                f,
                "session_work rejected: a session is keyed by (harness, id) and one of them is empty \
                 (harness={harness:?}, id={id:?}) — there is nothing to look up"
            ),
            SessionWorkRejection::Db(e) => write!(f, "session_work rejected: {e}"),
        }
    }
}

// =============================================================================
// Tool 1 — the forward direction: one session's distilled records
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct SessionWorkArgs {
    pub harness: String,
    pub id: String,
}

/// The forward read, rendered as the assembled object itself — there is no
/// second, summarised shape that could drift from what the UI renders.
pub async fn session_work(
    db: &SeatloomDb,
    args: SessionWorkArgs,
) -> Result<String, SessionWorkRejection> {
    let r = SessionRef {
        harness: args.harness,
        id: args.id,
    };
    let out = session_work::session_work(db, &r).await?;
    serde_json::to_string_pretty(&out).map_err(|e| SessionWorkRejection::Db(e.to_string()))
}

// =============================================================================
// Tool 2 — the reverse direction: the session one memory came from
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct MemorySessionArgs {
    pub memory_id: String,
}

/// The reverse read. An unknown memory id answers `null` — a caller asking about
/// a memory that is not an `archived_session` record is asking a question with no
/// answer, and the caller renders that. It is NOT an error, and it is NOT an empty
/// object that could be mistaken for a resolved one.
pub async fn memory_session(
    db: &SeatloomDb,
    args: MemorySessionArgs,
) -> Result<String, SessionWorkRejection> {
    match session_work::memory_session(db, &args.memory_id).await? {
        Some(out) => {
            serde_json::to_string_pretty(&out).map_err(|e| SessionWorkRejection::Db(e.to_string()))
        }
        None => Ok(Value::Null.to_string()),
    }
}

// =============================================================================
// tools/list entries
// =============================================================================

pub fn session_work_tool() -> Value {
    json!({
        "name": "session_work",
        "description": "The session↔work bridge, FORWARD: given one historical session \
            (harness, id), return the distilled records that came out of it — title, kind, \
            confidence, source path and the line range each cites — AND three SEPARATE numbers: \
            how many records are attributed to it, how many fully resolve, and how many do not. \
            Every unresolved record names WHICH of its two recorded keys failed and the exact \
            value that failed to resolve. \
            \
            THE LOAD-BEARING DISTINCTION: 'no records at all' and 'records that exist but do not \
            resolve' look identical in the ledger and are structurally different answers here — \
            the first is records: [] with numbers.total == 0, the second is a non-empty list with \
            numbers.unresolved > 0. Both carry their reason in `notes`. \
            \
            Keys: the declared path pair (l2_file_path with the corpus root stripped ↔ \
            archived_sessions.raw_path) is primary because both sides DECLARE it and it is immune \
            to id re-keying; the recorded session reference (source_ref split on the first colon ↔ \
            (harness, id)) is secondary. A topic id is NEVER used as a key. \
            \
            Pointers and line ranges only — never a transcript body (011's red line). PURE READ. \
            Refuses, rather than answering 'no records', when the bridge view (022) is absent.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["harness", "id"],
            "properties": {
                "harness": { "type": "string", "minLength": 1, "description": "The harness half of archived_sessions' primary key: one of codex | claude-code | stepcode | cursor | gemini | opencode (011's closed set)." },
                "id": { "type": "string", "minLength": 1, "description": "The id half — after 021 the native session id for corpus rows. Required: half a key is not a key." }
            }
        }
    })
}

pub fn memory_session_tool() -> Value {
    json!({
        "name": "memory_session",
        "description": "The session↔work bridge, REVERSE: given a memory, return the session it \
            came from — resolved BY KEY, never by topic id. The declared path pair is primary and \
            the recorded session reference is secondary; the topic id rides along as context and \
            is never joined on, because a topic is a segmentation (a revisable label) and not an \
            identity. \
            \
            Three distinguishable answers: a resolved session; `null` when the memory is not an \
            archived_session record at all (no answer, not an error); and a present answer with \
            session: null when the memory DOES carry a session reference but NEITHER key resolves \
            — the record exists and is readable, what is missing is the session it came from. \
            \
            Pointers and line ranges only — never a transcript body. PURE READ.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["memory_id"],
            "properties": {
                "memory_id": { "type": "string", "minLength": 1, "description": "The memory to look up, mem-<nanoid>." }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both tools advertise no identity parameter and no write capability, and
    /// both state the no-bodies rule.
    #[test]
    fn the_tools_advertise_no_identity_and_no_write() {
        for tool in [session_work_tool(), memory_session_tool()] {
            let props = tool["inputSchema"]["properties"].as_object().unwrap();
            for banned in ["seat_id", "actor_ref", "project_id", "payload", "event_id"] {
                assert!(
                    !props.contains_key(banned),
                    "{banned} must not be a parameter"
                );
            }
            let desc = tool["description"].as_str().unwrap();
            assert!(desc.contains("PURE READ"), "{desc}");
            assert!(desc.contains("never a transcript body"), "{desc}");
        }
    }

    /// The forward tool names the load-bearing distinction in its own
    /// description, so a caller cannot use it without being told.
    #[test]
    fn the_forward_tool_states_the_load_bearing_distinction() {
        let desc = session_work_tool()["description"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(desc.contains("no records at all"), "{desc}");
        assert!(desc.contains("do not resolve"), "{desc}");
        assert!(desc.contains("three SEPARATE numbers"), "{desc}");
        assert!(desc.contains("NEVER used as a key"), "{desc}");
    }

    /// The reverse tool states that a topic id is never a key.
    #[test]
    fn the_reverse_tool_states_the_topic_id_is_never_a_key() {
        let desc = memory_session_tool()["description"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(desc.contains("never by topic id"), "{desc}");
        assert!(desc.contains("not an identity"), "{desc}");
    }

    /// A missing view is a refusal that names the migration, not an empty answer.
    #[test]
    fn a_missing_view_is_a_refusal_not_an_empty_answer() {
        let msg = SessionWorkRejection::ViewMissing.to_string();
        assert!(msg.contains("v_memory_session_join"), "{msg}");
        assert!(msg.contains("022"), "{msg}");
        assert!(
            msg.contains("must not look like a read that measured nothing"),
            "{msg}"
        );
    }

    /// Half a key is a refusal naming which half is missing.
    #[test]
    fn half_a_key_is_a_refusal() {
        let msg = SessionWorkRejection::EmptyKey {
            harness: "codex".into(),
            id: "".into(),
        }
        .to_string();
        assert!(msg.contains("codex"), "{msg}");
        assert!(msg.contains("harness, id"), "{msg}");
    }
}
