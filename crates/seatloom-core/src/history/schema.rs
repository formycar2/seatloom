//! The typed read shapes for the historical session corpus.
//!
//! Value-domain contract (ENGINEERING_STANDARDS.md, explicit value-domain rule).
//! Reproduced verbatim in the governing delivery note.
//!
//! | Domain | Rust type | Serialized spelling | Nullable |
//! |---|---|---|---|
//! | harness | `String` | `codex` \| `claude-code` \| `stepcode` \| `cursor` \| `gemini` \| `opencode` (011's closed set, echoed verbatim) | no |
//! | dated | `bool` | `true` / `false` | no |
//! | unclassified (root group) | `bool` | `true` / `false` | no |
//! | turn label | [`TurnLabel`] | `system` \| `user` \| `assistant` \| `other` | no |
//! | line number | `u64` | 1-based | no |
//! | counts | `i64` | non-negative | no |
//! | `message_count` | `Option<i64>` | `NULL` when the corpus migration is not applied | yes |
//!
//! `message_count` is `Option` on purpose: `021` adds the column, and a database
//! without it must read as "not loaded" rather than as zero. A zero would be a
//! false statement about a corpus that has not been counted yet.

use serde::{Deserialize, Serialize};

/// One project: a home path an agent harness was started from.
///
/// `id` and `name` are the home path with the operator's `GitHub/` prefix
/// stripped (unique; the basename collides — `private_knowledge_base` is the
/// basename of four distinct homes). `home_path` keeps the absolute path so
/// nothing is lost. `unclassified` marks the `(root)` group: the bare repository
/// root, which strips to the empty string and is a path-prefix ancestor of 37
/// of the other homes. It is never given a guessed parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryProject {
    pub id: String,
    pub name: String,
    pub home_path: Option<String>,
    pub session_count: i64,
    /// Sum of the sessions' `message_count`. `None` when the column does not
    /// exist yet — never a fabricated zero.
    pub message_count: Option<i64>,
    /// The repository-root group: unmapped and labelled, per 015's R5.
    pub unclassified: bool,
}

/// One archived session, as the read returns it.
///
/// `dated` is `false` exactly when the harness recorded no timestamps (all 14
/// cursor sessions). `session_date` / `first_ts` / `last_ts` are then `None` and
/// the caller must render "时间未记录" rather than a date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySession {
    pub id: String,
    pub harness: String,
    /// The stripped-path project slug this session belongs to.
    pub project: String,
    pub cwd: Option<String>,
    pub session_date: Option<String>,
    pub first_ts: Option<String>,
    pub last_ts: Option<String>,
    pub message_count: Option<i64>,
    pub lines: Option<i64>,
    pub bytes: Option<i64>,
    /// Relative to the archive root (`raw/<harness>/<file>`).
    pub raw_path: String,
    pub src_path: Option<String>,
    pub sha256: Option<String>,
    pub dated: bool,
}

/// What kind of thing a turn is, classified from the extracted context file's
/// own heading. Deliberately coarse: a finer taxonomy would be an invention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnLabel {
    /// A system / role-instruction turn (`⚙️ 系统…`).
    System,
    /// The operator's turn (`你`).
    User,
    /// The agent's turn (`助手`).
    Assistant,
    /// Anything else, carried verbatim in [`HistoryTurn::label_raw`].
    Other,
}

impl TurnLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            TurnLabel::System => "system",
            TurnLabel::User => "user",
            TurnLabel::Assistant => "assistant",
            TurnLabel::Other => "other",
        }
    }
}

impl std::fmt::Display for TurnLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One turn — **a pointer, never a body**.
///
/// `line` is the 1-based line number in the ORIGINAL file on disk, which is what
/// the operator checks a conversation against. `context_path` /
/// `context_line_start` / `context_line_end` say where this turn lives in the
/// extracted markdown, so a local client can read exactly that slice. `chars` is
/// the turn's length, so a list can show scale without shipping text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTurn {
    pub line: u64,
    pub label: TurnLabel,
    /// The heading text verbatim, when it is not one of the three known labels.
    pub label_raw: Option<String>,
    pub chars: u64,
    pub context_path: String,
    pub context_line_start: u64,
    pub context_line_end: u64,
}

/// One session's turns, plus everything needed to say why there might be none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySessionDetail {
    pub id: String,
    pub harness: String,
    /// The absolute path of the original transcript file.
    pub source_path: String,
    /// Whether that file is on disk right now.
    pub source_present: bool,
    /// The line count the database records for it (011's `lines`).
    pub source_line_count: Option<i64>,
    /// The extracted context file, when one exists for this session.
    pub context_path: Option<String>,
    /// The message count the context file declares in its header.
    pub declared_message_count: Option<i64>,
    /// The message count the database holds for the same session.
    pub stored_message_count: Option<i64>,
    /// Whether the two agree, when both are known. `Some(false)` is a defect to
    /// report, not a rounding difference (contract §3.3); `None` means one side
    /// is absent, which is an absence and not a disagreement — a half-loaded
    /// corpus must not be blamed for a mismatch it cannot have.
    pub counts_agree: Option<bool>,
    pub turns: Vec<HistoryTurn>,
    /// Why there are no turns, when there are none. Never an empty list with no
    /// explanation.
    pub empty_reason: Option<String>,
}
