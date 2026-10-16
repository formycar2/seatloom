//! The historical-corpus read tools — the MCP half of the read path.
//!
//! Product contract: `LYRA-2026-10-14-history-sessions-in-ui-contract-v1`
//! §8.2 slice 2 (dispatch `disp-history-sessions-read`). Three tools over ONE
//! implementation: the repository reads in `seatloom_core::db` and the turn
//! reader in `seatloom_core::history::read`. The Tauri commands
//! (`cmd_list_history_projects`, `cmd_list_history_sessions`,
//! `cmd_get_history_session_turns`) shell out to these same tools, so there is
//! no second implementation — the established thin-wrapper pattern, the same one
//! the knowledge read model used.
//!
//! # What these tools never do
//!
//! * **No transcript bodies.** The turn read returns pointers — a line number, a
//!   source path, and where the turn lives in the extracted context file. 011's
//!   rule stands: the bodies may contain secrets and the originals are on disk.
//! * **No writes.** Pure reads; they append nothing and mutate nothing.
//! * **No guessing.** A project with no sessions says why. A session with no
//!   timestamps is flagged, not dated. A session with no extracted context says
//!   so. A database without the corpus migration is reported as not loaded.

use deadpool_postgres::Pool;
use serde::Deserialize;
use serde_json::{json, Value};

use seatloom_core::db::SeatloomDb;
use seatloom_core::history::{self, HistoryContexts, HistoryReadError};

use crate::dispatch::resolve_project;

/// The governing contract, named on every answer so a reader of the ledger can
/// trace a result back to the rules it was built under.
pub const HISTORY_CONTRACT: &str = "LYRA-2026-10-14-history-sessions-in-ui-contract-v1";

/// Why a history read was refused. Every variant names what is missing.
#[derive(Debug)]
pub enum HistoryRejection {
    /// The corpus migration is not applied to this database.
    CorpusNotLoaded {
        database: String,
    },
    /// The extracted contexts are not on disk.
    ContextsUnavailable {
        detail: String,
    },
    /// The named project has no sessions (and why that is not an error).
    UnknownProject {
        project_id: String,
        known: Vec<String>,
    },
    /// The named session is not in the archive.
    UnknownSession {
        harness: String,
        id: String,
    },
    Db(String),
}

impl std::fmt::Display for HistoryRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryRejection::CorpusNotLoaded { database } => write!(
                f,
                "history read rejected: the historical corpus is not loaded in {database:?} — \
                 archived_sessions.message_count does not exist, so no session or message count \
                 can be reported. Apply the corpus migration (021) first; a zero here would be a \
                 false statement about a corpus that has not been counted."
            ),
            HistoryRejection::ContextsUnavailable { detail } => {
                write!(f, "history read rejected: {detail}")
            }
            HistoryRejection::UnknownProject { project_id, known } => write!(
                f,
                "history read rejected: no project {project_id:?} has archived sessions. \
                 Known projects ({}): {}",
                known.len(),
                if known.is_empty() {
                    "<none — the corpus may not be loaded>".to_string()
                } else {
                    known.join(", ")
                }
            ),
            HistoryRejection::UnknownSession { harness, id } => write!(
                f,
                "history read rejected: no archived session ({harness:?}, {id:?}) — \
                 the archive is keyed by (harness, id)"
            ),
            HistoryRejection::Db(e) => write!(f, "history read rejected: {e}"),
        }
    }
}

fn db_err(e: seatloom_core::db::repositories::DbError) -> HistoryRejection {
    HistoryRejection::Db(e.to_string())
}

/// Refuse a read whose database has not had the corpus migration applied.
///
/// Without `021` the table holds the 2026-10-08 out-of-band manifest load: 271
/// rows under 18 coarse slugs, 89 of them with no extracted conversation, and no
/// message counts. A project list built from that would say "1 project, 16
/// sessions" and a session list would mix inventory rows into the corpus — the
/// false "you have no history" statement the contract forbids. Refusing names
/// the missing act instead.
async fn require_corpus(db: &SeatloomDb) -> Result<(), HistoryRejection> {
    let loaded = db.corpus_columns_present().await.map_err(db_err)?;
    if !loaded {
        return Err(HistoryRejection::CorpusNotLoaded {
            database: "this database".to_string(),
        });
    }
    Ok(())
}

// =============================================================================
// Tool 1 — the projects, with their counts
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListHistoryProjectsArgs {
    #[serde(default)]
    pub project_id: Option<String>,
}

/// (a) Every project that has at least one archived session, with its session
/// count and its total message count, ordered by session count descending so the
/// numbers are checkable against the source CSV.
pub async fn list_history_projects(
    db: &SeatloomDb,
    args: ListHistoryProjectsArgs,
) -> Result<String, HistoryRejection> {
    let _ = resolve_project(args.project_id).map_err(|e| HistoryRejection::Db(e.to_string()))?;
    require_corpus(db).await?;
    let rows = db.list_history_projects().await.map_err(db_err)?;

    // A project row whose message_count is NULL means the corpus migration is
    // absent. Say so rather than printing zeros.
    let corpus_loaded = rows.iter().any(|r| r.message_count.is_some());
    let projects: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "homePath": r.home_path,
                "sessionCount": r.session_count,
                "messageCount": r.message_count,
                // The repository-root group: unmapped and labelled (015's R5),
                // never folded into a neighbour and never given a parent.
                "unclassified": r.id == "(root)",
            })
        })
        .collect();

    let value = json!({
        "contract": HISTORY_CONTRACT,
        "corpusLoaded": corpus_loaded,
        "projectCount": projects.len(),
        "sessionTotal": rows.iter().map(|r| r.session_count).sum::<i64>(),
        "messageTotal": if corpus_loaded {
            Value::from(rows.iter().filter_map(|r| r.message_count).sum::<i64>())
        } else {
            Value::Null
        },
        "projects": projects,
    });
    serde_json::to_string_pretty(&value).map_err(|e| HistoryRejection::Db(e.to_string()))
}

// =============================================================================
// Tool 2 — one project's sessions
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListHistorySessionsArgs {
    pub project_id: String,
}

/// (b) One project's sessions: harness, date range or an explicit "not
/// recorded", message count. Dated first (newest first), the undated ones last
/// and flagged — never silently interleaved.
pub async fn list_history_sessions(
    db: &SeatloomDb,
    args: ListHistorySessionsArgs,
) -> Result<String, HistoryRejection> {
    require_corpus(db).await?;
    let rows = db
        .list_history_sessions(&args.project_id)
        .await
        .map_err(db_err)?;
    if rows.is_empty() {
        let known: Vec<String> = db
            .list_history_projects()
            .await
            .map_err(db_err)?
            .into_iter()
            .map(|r| r.id)
            .collect();
        return Err(HistoryRejection::UnknownProject {
            project_id: args.project_id,
            known,
        });
    }
    let corpus_loaded = rows.iter().any(|r| r.message_count.is_some());

    let sessions: Vec<Value> = rows
        .iter()
        .map(|r| {
            // `dated` is decided by first_ts and by NOTHING else. The manifest's
            // session_date is a coarse date on an inventory row, not a
            // timestamp the harness recorded, and the SQL sorts on first_ts
            // only — so falling back to session_date flagged rows the sort put
            // in the undated block. Flux measured the contradiction on
            // `seatloom`: a gemini marker row with `firstTs: null` and
            // `dated: true`. The flag and the order now agree by construction.
            let dated = r.first_ts.is_some();
            json!({
                "id": r.id,
                "harness": r.harness,
                "project": r.project,
                "cwd": r.cwd,
                "sessionDate": r.session_date,
                "firstTs": r.first_ts,
                "lastTs": r.last_ts,
                "messageCount": r.message_count.map(i64::from),
                "lines": r.lines.map(i64::from),
                "bytes": r.bytes,
                "rawPath": r.raw_path,
                "srcPath": r.src_path,
                "sha256": r.sha256,
                // false exactly when the harness recorded no timestamps at all
                // (all 14 cursor sessions). The caller must render 时间未记录.
                "dated": dated,
            })
        })
        .collect();

    let value = json!({
        "contract": HISTORY_CONTRACT,
        "corpusLoaded": corpus_loaded,
        "projectId": args.project_id,
        "sessionCount": sessions.len(),
        "undatedCount": sessions.iter().filter(|s| s["dated"] == Value::Bool(false)).count(),
        "sessions": sessions,
    });
    serde_json::to_string_pretty(&value).map_err(|e| HistoryRejection::Db(e.to_string()))
}

// =============================================================================
// Tool 3 — one session's turns
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct GetHistorySessionTurnsArgs {
    pub harness: String,
    pub id: String,
    /// Optional: the directory holding `by-session/*.md`. Defaults to
    /// `SEATLOOM_HISTORY_CONTEXTS_DIR`, else `<repo root>/resume-contexts`.
    #[serde(default)]
    pub contexts_dir: Option<String>,
}

/// (c) One session's turns in source order, each carrying the line number it
/// came from in the original file on disk, plus the source file path.
///
/// **Pointers, never bodies.** No turn text is returned; the originals are on
/// disk and may contain secrets.
pub async fn get_history_session_turns(
    db: &SeatloomDb,
    pool: &Pool,
    args: GetHistorySessionTurnsArgs,
) -> Result<String, HistoryRejection> {
    let _ = pool;
    let row = db
        .get_archived_session(&args.harness, &args.id)
        .await
        .map_err(db_err)?
        .ok_or_else(|| HistoryRejection::UnknownSession {
            harness: args.harness.clone(),
            id: args.id.clone(),
        })?;
    if row.message_count.is_none() {
        return Err(HistoryRejection::CorpusNotLoaded {
            database: "this database".to_string(),
        });
    }

    let dir = args
        .contexts_dir
        .clone()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(history::read::default_contexts_dir);
    let contexts = HistoryContexts::build(&dir).map_err(|e: HistoryReadError| {
        HistoryRejection::ContextsUnavailable {
            detail: e.to_string(),
        }
    })?;
    let detail =
        history::read_history_session_turns(&row, &contexts).map_err(|e: HistoryReadError| {
            HistoryRejection::ContextsUnavailable {
                detail: e.to_string(),
            }
        })?;

    let value = json!({
        "contract": HISTORY_CONTRACT,
        "session": detail,
        "bodiesReturned": false,
        "note": "Pointers only: `line` is the 1-based line in the ORIGINAL file, and \
                 contextPath/contextLineStart/contextLineEnd say where the turn lives in the \
                 extracted markdown. The transcript bodies stay on disk (011's rule).",
    });
    serde_json::to_string_pretty(&value).map_err(|e| HistoryRejection::Db(e.to_string()))
}

// =============================================================================
// tools/list entries
// =============================================================================

pub fn list_history_projects_tool() -> Value {
    json!({
        "name": "list_history_projects",
        "description": "The historical session corpus, by project: every project that has at \
            least one archived session, with its session count and its total message count, \
            ordered by session count descending so the numbers are checkable against the source \
            CSV (contract LYRA-2026-10-14-history-sessions-in-ui-contract-v1 §8.2 slice 2). A \
            project is a home path an agent harness was started from; the repository-root group \
            is returned as `(root)`, explicitly labelled and never given a guessed parent. Only \
            corpus rows are counted (message_count IS NOT NULL): the 2026-10-08 manifest \
            inventory the live table also holds is not sessions and is never added to a count. \
            PURE READ — writes nothing, appends no event.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "project_id": { "type": "string", "minLength": 1, "description": "Optional only because SEATLOOM_PROJECT_ID can supply it; a project is never fabricated." }
            }
        }
    })
}

pub fn list_history_sessions_tool() -> Value {
    json!({
        "name": "list_history_sessions",
        "description": "One project's archived sessions: harness, date range (or an explicit \
            `dated: false` when the harness recorded no timestamps — all 14 cursor sessions), \
            message count, line/byte size and the source paths. Dated sessions come first, newest \
            first; the undated ones come LAST and are flagged, never silently interleaved, \
            because sorting them as if they were the oldest would be a lie the operator would \
            eventually catch. PURE READ.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["project_id"],
            "properties": {
                "project_id": { "type": "string", "minLength": 1, "description": "The project slug (the home path with the GitHub/ prefix stripped), e.g. OfficeBench." }
            }
        }
    })
}

pub fn get_history_session_turns_tool() -> Value {
    json!({
        "name": "get_history_session_turns",
        "description": "One session's turns IN SOURCE ORDER, each carrying the line number it \
            came from in the ORIGINAL file on disk, plus the source file path. POINTERS ONLY — \
            no transcript body is stored or returned (011's rule: the bodies may contain secrets \
            and the originals are already on disk); each turn says where it lives in the \
            extracted markdown so a local client can read exactly that slice. A session with no \
            extracted context, and a harness that produced no conversation, both come back with \
            an explicit reason rather than an empty list. PURE READ.",
        "inputSchema": {
            "type": "object",
            "additionalProperties": false,
            "required": ["harness", "id"],
            "properties": {
                "harness": { "type": "string", "minLength": 1, "description": "One of codex | claude-code | stepcode | cursor | gemini | opencode (011's closed set)." },
                "id": { "type": "string", "minLength": 1, "description": "The native session id for that harness (archived_sessions.id after the corpus migration re-keyed it)." },
                "contexts_dir": { "type": "string", "minLength": 1, "description": "Optional: the directory holding by-session/*.md. Defaults to SEATLOOM_HISTORY_CONTEXTS_DIR, else <repo root>/resume-contexts." }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_tools_advertise_no_identity_and_no_write() {
        for tool in [
            list_history_projects_tool(),
            list_history_sessions_tool(),
            get_history_session_turns_tool(),
        ] {
            let props = tool["inputSchema"]["properties"].as_object().unwrap();
            assert!(!props.contains_key("seat_id"), "{tool}");
            assert!(!props.contains_key("actor_ref"), "{tool}");
            let desc = tool["description"].as_str().unwrap();
            assert!(desc.contains("PURE READ"), "{desc}");
        }
    }

    #[test]
    fn the_turn_tool_states_the_no_bodies_rule() {
        let tool = get_history_session_turns_tool();
        let desc = tool["description"].as_str().unwrap();
        assert!(desc.contains("POINTERS ONLY"), "{desc}");
        assert!(
            desc.contains("no transcript body is stored or returned"),
            "{desc}"
        );
    }

    #[test]
    fn an_absent_corpus_is_reported_not_zeroed() {
        let err = HistoryRejection::CorpusNotLoaded {
            database: "seatloom".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("not loaded"), "{msg}");
        assert!(msg.contains("021"), "{msg}");
        assert!(msg.contains("false statement"), "{msg}");
    }

    #[test]
    fn an_unknown_project_names_what_is_known() {
        let err = HistoryRejection::UnknownProject {
            project_id: "atlas".to_string(),
            known: vec!["OfficeBench".to_string(), "seatloom".to_string()],
        };
        let msg = err.to_string();
        assert!(msg.contains("atlas"), "{msg}");
        assert!(msg.contains("OfficeBench, seatloom"), "{msg}");
    }

    #[test]
    fn the_session_tool_requires_harness_and_id() {
        let tool = get_history_session_turns_tool();
        let required = tool["inputSchema"]["required"].as_array().unwrap();
        assert_eq!(required.len(), 2);
        assert!(required.iter().any(|v| v == "harness"));
        assert!(required.iter().any(|v| v == "id"));
    }
}
