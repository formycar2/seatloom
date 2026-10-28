//! The historical session corpus read path — **one implementation, two thin
//! surfaces** (an MCP tool and a Tauri command).
//!
//! Product contract: `LYRA-2026-10-14-history-sessions-in-ui-contract-v1`
//! (Lyra, `packet/lyra/milestone-summary` @ `10504bc`), §8.2 slice 2. The data
//! it reads was loaded by `021_history_sessions_corpus.sql` (Onyx,
//! `packet/onyx/topic-opened-019` @ `33f456b`).
//!
//! # What this module is for
//!
//! Mr. Zhang's own agent history — 183 sessions, 36,680 messages, 38 projects —
//! is extracted and line-anchored on disk and was invisible inside the product.
//! This is the read path: projects with their counts, the sessions of one
//! project, and one session's turns with the line number each came from in the
//! original file.
//!
//! # The three rules the whole module is built around
//!
//! 1. **No transcript bodies are stored, and none are returned.** The read
//!    returns *pointers*: a line number, the source file path, and where the
//!    turn lives in the extracted context file. 011's rule stands — the bodies
//!    may contain secrets and the originals are already on disk.
//! 2. **Undated sessions come last and are flagged.** All 14 cursor sessions
//!    have no timestamps at all (cursor's `chat_dump.json` carries none), so
//!    sorting them as if they were the oldest would be a lie the operator would
//!    eventually catch.
//! 3. **Unknown is reported, never guessed.** A session whose home path does
//!    not resolve to a project shows up in an explicit unclassified group (the
//!    `(root)` case), never folded into a neighbour. A session with no
//!    extracted context says so instead of returning an empty turn list that
//!    looks like "no conversation".
//!
//! # Where the turns come from, and why it is not a derived filename
//!
//! There is no messages table: the corpus's turn structure lives in
//! `resume-contexts/by-session/*.md`, one file per session, each carrying the
//! original file's absolute path in its `来源文件` header line and one
//! `## [原文件第 N 行] <label>` heading per turn. The markdown file *name*
//! cannot be derived from the database row — after 021 re-keyed the ids to the
//! native session id, the stepcode file is named after the file id while the row
//! holds `claude:<uuid>` — so this module **indexes the directory by the
//! absolute source path each file declares** and looks a session up by
//! resolving its `raw_path` against the root that index discovers. A derived
//! filename would be a guess; an index keyed on a declared path is a fact.

pub mod read;
pub mod schema;

pub use read::{
    default_contexts_dir, read_history_session_turns, HistoryContexts, HistoryReadError,
    HISTORY_CONTEXTS_ENV,
};
pub use schema::{HistoryProject, HistorySession, HistorySessionDetail, HistoryTurn, TurnLabel};
