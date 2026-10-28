//! The session↔work read model — one read-only function, two thin surfaces.
//!
//! Product contract: `LYRA-2026-10-16-session-work-bridge-contract-v1`
//! (Lyra, `packet/lyra/milestone-summary` @ `8601167`) §11.2 slice 2, dispatched
//! as `disp-session-key-readmodel`.
//!
//! # What this answers
//!
//! Mr Zhang's question, in his words: *what came out of that session?* The corpus
//! screens let him **read** a session; the knowledge panel holds what the washing
//! pipeline distilled from it; nothing joined the two. The join already exists in
//! the data — 90 memories carry `source_kind='archived_session'` — and was
//! invisible in the product.
//!
//! So this module is one read-only function in each direction:
//!
//! * **forward** — given a historical session, its distilled records (title,
//!   kind, confidence, line range, source path) **and three separate numbers**;
//! * **reverse** — given a memory, the session it came from, **by key**.
//!
//! # The three numbers are the load-bearing wall
//!
//! > "no records" and "records that exist but do not resolve" are the same thing
//! in the ledger and must never be the same thing in the product.
//!
//! Before 021, 3 codex references resolved against nothing and nothing reported
//! it, so those sessions looked like they had produced nothing at all. So the
//! forward read reports `total` / `resolved` / `unresolved` as three separate
//! fields, and every unresolved record names **what is missing** — which of the
//! two recorded keys failed, and the exact value that failed to resolve. A session
//! with zero records is a fourth state, carried by `notes` and by
//! `records.is_empty()` together with `total == 0`, and it is structurally
//! distinguishable from `total > 0 && unresolved > 0`.
//!
//! # The keys, and the one that is never used
//!
//! | Key | Role |
//! |---|---|
//! | **A1** `l2_file_path` with the corpus root stripped ↔ `archived_sessions.raw_path` | primary — both sides *declare* it, so it is immune to id re-keying |
//! | **A2** `source_ref` split on the first colon ↔ `archived_sessions (harness, id)` | secondary — exact and already written, but a free-text column that has failed once |
//! | **A3** a topic id | **never a key.** `wash-sess-cc-621ae382` is a session id truncated to 8 characters by `s5_writein.py`; a topic is a segmentation, a revisable label, not an identity |
//!
//! The reverse read resolves a memory **by key**. It carries the topic id as
//! context and never joins on it.
//!
//! # Read only
//!
//! No writes, no events, no gate. Pointers and line ranges only — never a
//! transcript body (011's red line).

use serde::Serialize;

use crate::db::models::MemorySessionJoinRow;
use crate::db::repositories::SeatloomDb;

/// The governing contract, named on every answer.
pub const SESSION_WORK_CONTRACT: &str = "LYRA-2026-10-16-session-work-bridge-contract-v1";

/// The view this read model reads through (022, Onyx's slice 1).
pub const SESSION_WORK_VIEW: &str = "v_memory_session_join";

/// What a caller points at, in the forward direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRef {
    /// The harness half of `archived_sessions`' primary key.
    pub harness: String,
    /// The id half. After 021 this is the native session id for corpus rows.
    pub id: String,
}

/// One distilled record, as the read returns it.
///
/// Pointers and line ranges only. `source_path` is the L2 pointer **verbatim** —
/// the exact string the write-in declared — so a client that opens the file opens
/// the one the record cites.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWorkRecord {
    pub memory_id: String,
    pub title: String,
    pub kind: String,
    pub confidence: String,
    pub status: String,
    /// The source file path, verbatim.
    pub source_path: String,
    /// The lines this record cites in that file.
    pub line_start: Option<i32>,
    pub line_end: Option<i32>,
    /// `both_agree` | `path_only` | `ref_only` | `disagree` | `none`.
    pub join_state: String,
    /// Which of the two recorded keys resolved. Empty when neither did.
    pub resolved_by: Vec<String>,
    /// What is missing for THIS record, named. Empty when it fully resolved.
    pub missing: Vec<String>,
    /// Context only — never a key.
    pub topic_id: String,
}

/// The three numbers, kept separate on purpose.
///
/// `unresolved == total - resolved` by construction; the separation exists so a
/// caller cannot accidentally render only one of them and call it the answer.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWorkNumbers {
    /// Records attributed to this session by either key.
    pub total: usize,
    /// Records where BOTH keys resolve and agree.
    pub resolved: usize,
    /// Records where at least one key does not resolve.
    pub unresolved: usize,
}

/// The forward read's answer.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWork {
    /// The session as the caller named it.
    #[serde(rename = "ref")]
    pub ref_: SessionRef,
    /// The session row's own facts, when the session exists.
    pub session: Option<SessionWorkSession>,
    /// The distilled records, in `created_at` then `memory_id` order.
    pub records: Vec<SessionWorkRecord>,
    /// The three numbers.
    pub numbers: SessionWorkNumbers,
    /// Measured statements about this answer that are not facts about the
    /// records — why a slot is empty, and which states apply.
    pub notes: Vec<String>,
    /// The drift number, measured over the WHOLE view rather than this session:
    /// how many memories carry a session reference that resolves against no row.
    /// A session-local read cannot see it, so it rides along.
    pub drift_unresolved_refs: i64,
    pub contract: String,
}

/// The session's own row facts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWorkSession {
    pub harness: String,
    pub id: String,
    pub raw_path: Option<String>,
    pub project: Option<String>,
    /// `None` for one of the 89 inventory-only rows outside the 183-session
    /// corpus. Reported, never defaulted to zero.
    pub message_count: Option<i32>,
    /// Whether this row is a corpus session at all. An inventory-only row's
    /// records must be read differently, so it is stated rather than inferred.
    pub corpus_session: bool,
}

/// The reverse read's answer.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySession {
    pub memory_id: String,
    /// The session it came from, by key. `None` when neither key resolved — a
    /// different answer from "this memory has no session".
    pub session: Option<SessionWorkSession>,
    pub join_state: String,
    pub resolved_by: Vec<String>,
    pub missing: Vec<String>,
    /// Context only — never a key.
    pub topic_id: String,
    pub notes: Vec<String>,
    pub contract: String,
}

/// Why a read was refused. Every variant names what is missing.
#[derive(Debug, thiserror::Error)]
pub enum SessionWorkError {
    #[error(
        "the session↔work bridge is not available in this database — the view \\
         `{SESSION_WORK_VIEW}` does not exist. Apply migration 022 first; a read \\
         that cannot measure the join must not look like a read that measured \\
         nothing."
    )]
    ViewMissing,
    /// No `archived_sessions` row carries that `(harness, id)`. Distinct from
    /// "the session exists and produced no records", which is an answer rather
    /// than a refusal.
    #[error(
        "session_work rejected: no archived session ({harness:?}, {id:?}) exists — the archive \
         is keyed by (harness, id), and this session is not in it"
    )]
    SessionNotFound { harness: String, id: String },
    /// The caller named a session with an empty harness or id. The archive is
    /// keyed by `(harness, id)`, so half a key is not a key.
    #[error(
        "session_work rejected: a session is keyed by (harness, id) and one of them is empty \
         (harness={harness:?}, id={id:?}) — there is nothing to look up"
    )]
    EmptyKey { harness: String, id: String },
    #[error("session_work rejected: {0}")]
    Db(crate::db::repositories::DbError),
}

impl From<crate::db::repositories::DbError> for SessionWorkError {
    fn from(e: crate::db::repositories::DbError) -> Self {
        SessionWorkError::Db(e)
    }
}

/// The forward read: one session's distilled records, and the three numbers.
pub async fn session_work(
    db: &SeatloomDb,
    r: &SessionRef,
) -> Result<SessionWork, SessionWorkError> {
    require_view(db).await?;
    if r.harness.trim().is_empty() || r.id.trim().is_empty() {
        return Err(SessionWorkError::EmptyKey {
            harness: r.harness.clone(),
            id: r.id.clone(),
        });
    }

    // "That session is not in the archive" is a different fact from "that
    // session is in the archive and produced no records". Asked before the join
    // so the two cannot collapse into one empty answer.
    if !db.archived_session_exists(&r.harness, &r.id).await? {
        return Err(SessionWorkError::SessionNotFound {
            harness: r.harness.clone(),
            id: r.id.clone(),
        });
    }

    let rows = db.session_work_records(&r.harness, &r.id).await?;
    let drift = db.session_work_drift_count().await?;
    let join_states = db.session_work_join_states().await?;

    // The session's own row, from the records' joined columns. Every attributed
    // record carries the same session facts (they resolved to the same row), so
    // the first is the row — and if there are no records, there is no row to
    // show, which is itself reported.
    let session = rows.first().map(|first| SessionWorkSession {
        harness: first
            .session_harness
            .clone()
            .unwrap_or_else(|| r.harness.clone()),
        id: first.session_id.clone().unwrap_or_else(|| r.id.clone()),
        raw_path: first.session_raw_path.clone(),
        project: first.session_project.clone(),
        message_count: first.session_message_count,
        corpus_session: first.session_message_count.is_some(),
    });

    let records: Vec<SessionWorkRecord> = rows.iter().map(record_of).collect();
    let resolved = records
        .iter()
        .filter(|r| r.join_state == "both_agree")
        .count();
    let numbers = SessionWorkNumbers {
        total: records.len(),
        resolved,
        unresolved: records.len() - resolved,
    };

    let mut notes = Vec::new();
    if numbers.total == 0 {
        notes.push(
            "no distilled records are attributed to this session. Two different reasons produce \\
             this state and they are not merged: the session was outside the washing pipeline's \\
             scope, or it was washed and contributed no qualifying record. Nothing here guesses \\
             which."
                .to_string(),
        );
    } else if numbers.unresolved > 0 {
        notes.push(format!(
            "{} of {} records attributed to this session do not fully resolve: at least one of \\
             their two recorded keys points at no row. Each one names which key and the exact \\
             value that failed. This state is deliberately distinct from 'no records'.",
            numbers.unresolved, numbers.total
        ));
    }
    if let Some(s) = &session {
        if !s.corpus_session {
            notes.push(
                "this archived_sessions row is one of the 89 inventory-only rows outside the \\
                 183-session corpus (message_count IS NULL), so its records must be read as \\
                 inventory rather than as corpus output — reported, not defaulted."
                    .to_string(),
            );
        }
    }
    notes.push(format!(
        "the load-bearing number over the whole view: {drift} memories carry a session reference \\
         that resolves against no row. That is a property of the corpus, not of this session, so \\
         it rides along rather than being divided among sessions."
    ));
    let states: Vec<String> = join_states
        .iter()
        .map(|(s, n)| format!("{s}={n}"))
        .collect();
    // Named for the view's own column (`resolved_by`), not the read model's
    // `join_state`: the note describes what the database says, and the read
    // model's word for it is a DIFFERENT word — the view coalesces s1/s2 and so
    // cannot express a disagreement, which the read model's vocabulary can.
    notes.push(format!(
        "resolved_by across the view: {}",
        states.join(", ")
    ));

    Ok(SessionWork {
        ref_: r.clone(),
        session,
        records,
        numbers,
        notes,
        drift_unresolved_refs: drift,
        contract: SESSION_WORK_CONTRACT.to_string(),
    })
}

/// The reverse read: the session one memory came from, by key.
pub async fn memory_session(
    db: &SeatloomDb,
    memory_id: &str,
) -> Result<Option<MemorySession>, SessionWorkError> {
    require_view(db).await?;
    let Some(row) = db.memory_session(memory_id).await? else {
        // The memory exists but is not an `archived_session` record, or does not
        // exist at all. Either way there is no session to report, and the caller
        // says so — this is not an error.
        return Ok(None);
    };
    let (resolved_by, missing) = keys_of(&row);
    let session = row.session_harness.clone().map(|h| SessionWorkSession {
        harness: h,
        id: row.session_id.clone().unwrap_or_default(),
        raw_path: row.session_raw_path.clone(),
        project: row.session_project.clone(),
        message_count: row.session_message_count,
        corpus_session: row.session_message_count.is_some(),
    });

    let mut notes = Vec::new();
    if session.is_none() {
        notes.push(
            "this memory carries a session reference but NEITHER recorded key resolves against an \\
             archived_sessions row. The record exists and is readable; what is missing is the \\
             session it came from. That is a different state from 'this memory has no session', \\
             and it is the state the load-bearing number counts."
                .to_string(),
        );
    }
    notes.push(format!(
        "resolved by key, never by topic id. The topic id {:?} rides along as context; a topic is \\
         a segmentation — a revisable label — not an identity.",
        row.topic_id
    ));

    Ok(Some(MemorySession {
        memory_id: row.memory_id,
        session,
        join_state: row.join_state,
        resolved_by,
        missing,
        topic_id: row.topic_id,
        notes,
        contract: SESSION_WORK_CONTRACT.to_string(),
    }))
}

/// Which of the two recorded keys resolved, and what is missing for the ones
/// that did not. Both are named with the exact value that failed, so the
/// operator can check it against the row rather than trusting a verdict.
fn keys_of(row: &MemorySessionJoinRow) -> (Vec<String>, Vec<String>) {
    let mut resolved_by = Vec::new();
    let mut missing = Vec::new();

    if row.a1_resolves {
        resolved_by.push("a1_declared_path".to_string());
    } else {
        match (&row.a1_corpus_path, row.a1_path_rows) {
            (None, _) => missing.push(format!(
                "the file pointer {:?} is neither the ~-rooted form nor the bare raw/% form, so it \
                 names no corpus path at all",
                row.l2_file_path
            )),
            (Some(path), Some(n)) if n > 1 => missing.push(format!(
                "the corpus path {path:?} is declared by {n} archived_sessions rows, so it is \
                 ambiguous — counted, never resolved by picking one"
            )),
            (Some(path), _) => missing.push(format!(
                "the corpus path {path:?} resolves against no archived_sessions row"
            )),
        }
    }

    if row.a2_resolves {
        resolved_by.push("a2_session_ref".to_string());
    } else {
        match (&row.ref_harness, &row.ref_id) {
            (None, _) => missing.push(format!(
                "the recorded session reference {:?} has no colon, so it names neither a harness \
                 nor an id",
                row.ref_id.as_deref().unwrap_or("<none>")
            )),
            (Some(h), Some(i)) => missing.push(format!(
                "the recorded session reference {h:?} + {i:?} resolves against no \
                 archived_sessions row"
            )),
            (Some(h), None) => missing.push(format!(
                "the recorded session reference names harness {h:?} but no id"
            )),
        }
    }
    (resolved_by, missing)
}

/// One view row as a record.
fn record_of(row: &MemorySessionJoinRow) -> SessionWorkRecord {
    let (resolved_by, missing) = keys_of(row);
    SessionWorkRecord {
        memory_id: row.memory_id.clone(),
        title: row.title.clone(),
        kind: row.kind.clone(),
        confidence: row.confidence.clone(),
        status: row.status.clone(),
        source_path: row.l2_file_path.clone(),
        line_start: row.l2_line_start,
        line_end: row.l2_line_end,
        join_state: row.join_state.clone(),
        resolved_by,
        missing,
        topic_id: row.topic_id.clone(),
    }
}

/// Refuse a read whose database has not had the bridge view applied.
async fn require_view(db: &SeatloomDb) -> Result<(), SessionWorkError> {
    if db.session_work_view_present().await? {
        Ok(())
    } else {
        Err(SessionWorkError::ViewMissing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(join_state: &str) -> MemorySessionJoinRow {
        MemorySessionJoinRow {
            memory_id: "mem-1".into(),
            project_id: "seatloom".into(),
            topic_id: "wash-sess-cc-621ae382".into(),
            title: "a distilled record".into(),
            kind: "decision".into(),
            confidence: "quoted".into(),
            status: "active".into(),
            created_at: chrono::Utc::now(),
            l2_file_path: "/Users/x/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff.jsonl"
                .into(),
            l2_line_start: Some(13),
            l2_line_end: Some(63),
            a1_corpus_path: Some("raw/claude-code/b0100cff.jsonl".into()),
            a1_path_rows: Some(1),
            a1_harness: Some("claude-code".into()),
            a1_id: Some("b0100cff".into()),
            a1_resolves: true,
            ref_harness: Some("claude-code".into()),
            ref_id: Some("b0100cff".into()),
            a2_harness: Some("claude-code".into()),
            a2_id: Some("b0100cff".into()),
            a2_resolves: true,
            session_harness: Some("claude-code".into()),
            session_id: Some("b0100cff".into()),
            session_raw_path: Some("raw/claude-code/b0100cff.jsonl".into()),
            session_project: Some("seatloom".into()),
            session_message_count: Some(30),
            join_state: join_state.into(),
        }
    }

    /// A record where both keys resolve and agree has nothing missing and is
    /// counted as resolved.
    #[test]
    fn a_record_where_both_keys_agree_has_nothing_missing() {
        let r = record_of(&row("both_agree"));
        assert_eq!(r.join_state, "both_agree");
        assert_eq!(r.resolved_by, vec!["a1_declared_path", "a2_session_ref"]);
        assert!(r.missing.is_empty(), "{:?}", r.missing);
        // Pointers and a line range, and NO body.
        assert_eq!(
            r.source_path,
            "/Users/x/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff.jsonl"
        );
        assert_eq!(r.line_start, Some(13));
        assert_eq!(r.line_end, Some(63));
    }

    /// THE LOAD-BEARING CASE: the path resolves and the recorded reference does
    /// not. The record is present, one key is named as broken, and the exact
    /// value that failed is carried — so this state is distinguishable from
    /// "no records" and from "resolved".
    #[test]
    fn a_path_only_record_names_the_reference_that_failed() {
        let mut r = row("path_only");
        r.a2_resolves = false;
        r.a2_harness = None;
        r.a2_id = None;
        let rec = record_of(&r);
        assert_eq!(rec.join_state, "path_only");
        assert_eq!(rec.resolved_by, vec!["a1_declared_path"]);
        assert_eq!(rec.missing.len(), 1, "{:?}", rec.missing);
        assert!(rec.missing[0].contains("claude-code"), "{:?}", rec.missing);
        assert!(
            rec.missing[0].contains("resolves against no archived_sessions row"),
            "{:?}",
            rec.missing
        );
        // And it is NOT counted as resolved.
        let resolved = ["both_agree"].contains(&rec.join_state.as_str());
        assert!(!resolved, "a path_only record must not count as resolved");
    }

    /// The reverse half of the same break: the reference resolves, the path does
    /// not. Named the other way round, so the two are never conflated.
    #[test]
    fn a_ref_only_record_names_the_pointer_that_failed() {
        let mut r = row("ref_only");
        r.a1_resolves = false;
        r.a1_corpus_path = None;
        r.a1_path_rows = None;
        let rec = record_of(&r);
        assert_eq!(rec.resolved_by, vec!["a2_session_ref"]);
        assert_eq!(rec.missing.len(), 1, "{:?}", rec.missing);
        assert!(
            rec.missing[0].contains("names no corpus path at all"),
            "{:?}",
            rec.missing
        );
    }

    /// An ambiguous path is counted, never resolved by picking one. This is the
    /// fan-out guard: `raw_path` is not unique, one cursor directory is declared
    /// by two sessions, and a plain join would double-count the memory.
    #[test]
    fn an_ambiguous_path_is_reported_not_resolved() {
        // A1 is ambiguous (two rows declare the path) and A2 does resolve — so
        // the record is NOT resolved-by-path, and the ambiguity is named rather
        // than settled by picking a row. The fan-out guard: a plain join would
        // have counted this memory twice.
        let mut r = row("ref_only");
        r.a1_resolves = false;
        r.a1_path_rows = Some(2);
        let rec = record_of(&r);
        assert_eq!(
            rec.resolved_by,
            vec!["a2_session_ref"],
            "{:?}",
            rec.resolved_by
        );
        let m = rec.missing.iter().find(|m| m.contains("ambiguous"));
        assert!(m.is_some(), "{:?}", rec.missing);
        assert!(
            m.unwrap().contains("2 archived_sessions rows"),
            "{:?}",
            rec.missing
        );
        // And it is not counted as resolved.
        assert_ne!(rec.join_state, "both_agree");
    }

    /// A reference with no colon at all names NEITHER half — so the row measures
    /// unresolved rather than half-resolved with a bogus harness.
    #[test]
    fn a_colonless_reference_names_neither_half() {
        let mut r = row("none");
        r.a1_resolves = false;
        r.a1_corpus_path = None;
        r.a2_resolves = false;
        r.ref_harness = None;
        r.ref_id = None;
        let rec = record_of(&r);
        assert!(rec.resolved_by.is_empty());
        let m = rec.missing.iter().find(|m| m.contains("no colon"));
        assert!(m.is_some(), "{:?}", rec.missing);
    }

    /// The three numbers are three separate fields, and `unresolved` is not a
    /// derived alias a caller could forget to render.
    #[test]
    fn the_three_numbers_are_three_separate_fields() {
        let v: serde_json::Value = serde_json::to_value(SessionWorkNumbers {
            total: 7,
            resolved: 4,
            unresolved: 3,
        })
        .unwrap();
        for k in ["total", "resolved", "unresolved"] {
            assert!(v.get(k).is_some(), "missing {k}");
        }
        assert_eq!(v["total"], 7);
        assert_eq!(v["resolved"], 4);
        assert_eq!(v["unresolved"], 3);
    }

    /// "No records" and "records that do not resolve" are structurally different
    /// answers, which is the contract's load-bearing wall.
    #[test]
    fn no_records_and_unresolved_records_are_different_shapes() {
        // No records: empty list AND total == 0.
        let none = SessionWorkNumbers {
            total: 0,
            resolved: 0,
            unresolved: 0,
        };
        // Some, but broken.
        let broken = SessionWorkNumbers {
            total: 3,
            resolved: 0,
            unresolved: 3,
        };
        assert_ne!(none.total, broken.total);
        assert_eq!(none.unresolved, 0);
        assert_eq!(broken.unresolved, 3);
        // And on the wire the two are distinguishable without reading a note.
        let a: serde_json::Value = serde_json::to_value(none).unwrap();
        let b: serde_json::Value = serde_json::to_value(broken).unwrap();
        assert_ne!(a["total"], b["total"]);
    }

    /// The topic id rides along as context and is never used as a key. A caller
    /// reading the record can see it, and nothing in the shape joins on it.
    #[test]
    fn the_topic_id_is_context_and_never_a_key() {
        let r = record_of(&row("both_agree"));
        assert_eq!(r.topic_id, "wash-sess-cc-621ae382");
        // The record's identity is the memory id; the session's is (harness, id).
        assert_eq!(r.memory_id, "mem-1");
        // No field on the shape is named after a topic, and none carries a
        // truncated id.
        let v: serde_json::Value = serde_json::to_value(&r).unwrap();
        assert!(v.get("topicId").is_some(), "carried as context");
        assert!(v.get("topicKey").is_none());
        assert!(v.get("sessionKeyFromTopic").is_none());
    }

    /// An inventory-only row is reported as such rather than defaulted to a
    /// corpus session, because its records must be read differently.
    #[test]
    fn an_inventory_only_row_is_reported_not_defaulted() {
        let mut r = row("path_only");
        r.session_message_count = None;
        let corpus = r.session_message_count.is_some();
        assert!(!corpus, "message_count IS NULL means inventory-only");
        let s = SessionWorkSession {
            harness: "codex".into(),
            id: "rollout-x".into(),
            raw_path: r.session_raw_path.clone(),
            project: r.session_project.clone(),
            message_count: r.session_message_count,
            corpus_session: corpus,
        };
        assert!(!s.corpus_session);
        assert_eq!(s.message_count, None, "never defaulted to zero");
    }
}
