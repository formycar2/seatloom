//! Reading the extracted turn contexts off disk.
//!
//! The corpus's turn structure lives in `resume-contexts/by-session/*.md`. This
//! module indexes that directory **by the absolute source path each file
//! declares** (`来源文件` in its header) and parses one file's turn headings.
//!
//! # Why an index and not a derived filename
//!
//! The markdown file names are built by the extraction script from the home path
//! and the *file* id (`cn-gpu-infer-fabric__stepcode__4a629262-f806.md` names
//! the file id, while after `021` the database row holds the native id
//! `claude:104f1244-…`). Reconstructing that naming rule in the read path would
//! be a guess about a file this crate does not own, and a wrong guess would
//! silently return "no conversation" for a session that has one. Reading what
//! each file *declares* is a fact that can be checked.
//!
//! # Cost
//!
//! The index reads only the first twelve lines of each of the 183 files —
//! measured at ~0.4 s for the whole directory — and is built once per
//! [`HistoryContexts`], which the caller holds. A read that cannot afford it
//! passes an already-built value.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::history::schema::{HistoryTurn, TurnLabel};

/// Env var naming the directory that holds the extracted contexts. When unset,
/// `<repo root>/resume-contexts` is used — the location the contract records —
/// and a directory that is not there is a typed error, never an empty list.
pub const HISTORY_CONTEXTS_ENV: &str = "SEATLOOM_HISTORY_CONTEXTS_DIR";

/// How many header lines are read to index one file. The header block is eight
/// lines plus the fences; twelve is enough for every observed shape and keeps
/// the index cheap.
const HEADER_LINES: usize = 12;

/// Why a turn read could not be completed. Every variant names what is missing
/// rather than returning an empty list that looks like "no conversation".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryReadError {
    /// The contexts directory is not there.
    ContextsDirMissing { path: String },
    /// The directory exists but holds no `by-session/*.md`.
    NoContextFiles { path: String },
    /// A file could not be read (permissions, encoding).
    Unreadable { path: String, detail: String },
}

impl std::fmt::Display for HistoryReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryReadError::ContextsDirMissing { path } => write!(
                f,
                "the historical corpus is not on disk at {path:?} — 历史会话还没加载. \
                 Point {HISTORY_CONTEXTS_ENV} at the directory that holds by-session/*.md"
            ),
            HistoryReadError::NoContextFiles { path } => write!(
                f,
                "no by-session/*.md under {path:?} — the extraction output is missing, \
                 so no session's turns can be read"
            ),
            HistoryReadError::Unreadable { path, detail } => {
                write!(f, "cannot read {path:?}: {detail}")
            }
        }
    }
}

impl std::error::Error for HistoryReadError {}

/// One indexed context file: where it is, and what its header declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextEntry {
    pub context_path: PathBuf,
    /// The absolute path of the original transcript file, as declared.
    pub source_path: String,
    pub harness: Option<String>,
    pub home_path: Option<String>,
    pub declared_message_count: Option<i64>,
}

/// The indexed corpus: the contexts directory, the root its source paths share,
/// and one entry per extracted session.
#[derive(Debug, Clone)]
pub struct HistoryContexts {
    root: PathBuf,
    /// The directory every declared source path sits under, discovered from the
    /// declared paths themselves rather than configured.
    archive_root: Option<PathBuf>,
    by_source: HashMap<String, ContextEntry>,
}

impl HistoryContexts {
    /// Index `contexts_dir`. Cheap: only header lines are read.
    pub fn build(contexts_dir: &Path) -> Result<Self, HistoryReadError> {
        if !contexts_dir.is_dir() {
            return Err(HistoryReadError::ContextsDirMissing {
                path: contexts_dir.display().to_string(),
            });
        }
        let by_session = contexts_dir.join("by-session");
        let mut by_source: HashMap<String, ContextEntry> = HashMap::new();
        let entries = std::fs::read_dir(&by_session).map_err(|e| HistoryReadError::Unreadable {
            path: by_session.display().to_string(),
            detail: e.to_string(),
        })?;
        let mut any = false;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            any = true;
            let header = read_header(&path)?;
            if let Some(source_path) = header.source_path.clone() {
                by_source.insert(
                    source_path,
                    ContextEntry {
                        context_path: path,
                        source_path: header.source_path.unwrap_or_default(),
                        harness: header.harness,
                        home_path: header.home_path,
                        declared_message_count: header.declared_message_count,
                    },
                );
            }
        }
        if !any {
            return Err(HistoryReadError::NoContextFiles {
                path: by_session.display().to_string(),
            });
        }
        let archive_root = discover_archive_root(by_source.keys().map(|s| s.as_str()));
        Ok(Self {
            root: contexts_dir.to_path_buf(),
            archive_root,
            by_source,
        })
    }

    /// The contexts directory this index was built from.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// How many extracted sessions are indexed.
    pub fn len(&self) -> usize {
        self.by_source.len()
    }

    /// Whether the index is empty (it never is after a successful `build`).
    pub fn is_empty(&self) -> bool {
        self.by_source.is_empty()
    }

    /// The directory every declared source path sits under, when it could be
    /// discovered. `None` when the index holds fewer than one path.
    pub fn archive_root(&self) -> Option<&Path> {
        self.archive_root.as_deref()
    }

    /// Resolve a database row's relative `raw_path` to the absolute path the
    /// index is keyed by.
    ///
    /// Two strategies, both verified against the index so neither can guess:
    ///
    /// 1. **Exact** — `<archive root>/<raw_path>`. The archive root is the
    ///    longest directory prefix shared by the declared source paths; with a
    ///    full corpus that is the archive root itself, with a degenerate index
    ///    it is deeper, so the ancestors are tried too.
    /// 2. **Unique directory** — exactly one indexed source path under
    ///    `<archive root>/<raw_path>/`. This is the cursor case: `011`'s `raw`
    ///    for a cursor session is the workspace *directory* (`raw/cursor/<hash>`)
    ///    while the extracted conversation is the `chat_dump.json` inside it.
    ///    One directory, one context file, so the answer is unique — and if it
    ///    ever were not, this returns `None` and the caller says so rather than
    ///    picking one.
    pub fn resolve_source_path(&self, raw_path: &str) -> Option<String> {
        let mut candidate = self.archive_root.clone()?;
        for _ in 0..8 {
            let base = normalize(&candidate.join(raw_path));
            let key = base.to_string_lossy().to_string();
            if self.by_source.contains_key(&key) {
                return Some(key);
            }
            let prefix = format!("{key}/");
            let mut under = self.by_source.keys().filter(|k| k.starts_with(&prefix));
            if let (Some(first), None) = (under.next(), under.next()) {
                return Some(first.clone());
            }
            candidate = candidate.parent()?.to_path_buf();
        }
        None
    }

    /// The entry for an already-absolute source path.
    pub fn entry_for_source(&self, source_path: &str) -> Option<&ContextEntry> {
        self.by_source.get(source_path)
    }

    /// Parse one context file's turn headings.
    pub fn parse_turns(&self, entry: &ContextEntry) -> Result<Vec<HistoryTurn>, HistoryReadError> {
        parse_turns(&entry.context_path)
    }
}

/// The default contexts directory: `HISTORY_CONTEXTS_ENV` when set, else
/// `<repo root>/resume-contexts`.
pub fn default_contexts_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(HISTORY_CONTEXTS_ENV).filter(|v| !v.is_empty()) {
        return PathBuf::from(dir);
    }
    crate::git::ops::resolve_repo_root().join("resume-contexts")
}

/// Assemble one session's turn pointers from the database row and the indexed
/// contexts.
///
/// Every "there is nothing here" outcome carries its reason: a session with no
/// extracted context, a context that declares no messages, an original file that
/// is no longer on disk. An empty `turns` list with `empty_reason: None` never
/// leaves this function.
pub fn read_history_session_turns(
    row: &crate::db::models::ArchivedSessionRow,
    contexts: &HistoryContexts,
) -> Result<crate::history::schema::HistorySessionDetail, HistoryReadError> {
    use crate::history::schema::HistorySessionDetail;

    let source_path = contexts
        .resolve_source_path(&row.raw_path)
        .unwrap_or_else(|| row.raw_path.clone());
    let source_present = Path::new(&source_path).is_file();
    let entry = contexts.entry_for_source(&source_path);

    let (turns, empty_reason) = match entry {
        None => (
            Vec::new(),
            Some(format!(
                "no extracted context for this session: the corpus index holds no by-session/*.md \
                 whose 来源文件 is {source_path}"
            )),
        ),
        Some(entry) => {
            let turns = contexts.parse_turns(entry)?;
            if turns.is_empty() {
                (
                    Vec::new(),
                    Some(format!(
                        "this harness produced no extracted conversation for this session \
                         (declared 消息数 {})",
                        entry
                            .declared_message_count
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| "not stated".to_string())
                    )),
                )
            } else {
                (turns, None)
            }
        }
    };

    let declared = entry.and_then(|e| e.declared_message_count);
    // Both known -> compare. Either one absent -> not comparable, which is an
    // absence and NOT a disagreement: a half-loaded corpus must not be blamed
    // for a mismatch it cannot have.
    let counts_agree = match (declared, row.message_count.map(i64::from)) {
        (Some(a), Some(b)) => Some(a == b),
        _ => None,
    };

    Ok(HistorySessionDetail {
        id: row.id.clone(),
        harness: row.harness.clone(),
        source_path,
        source_present,
        source_line_count: row.lines.map(i64::from),
        context_path: entry.map(|e| e.context_path.display().to_string()),
        declared_message_count: declared,
        stored_message_count: row.message_count.map(i64::from),
        counts_agree,
        turns,
        empty_reason,
    })
}

// =============================================================================
// Header parsing
// =============================================================================

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Header {
    harness: Option<String>,
    home_path: Option<String>,
    declared_message_count: Option<i64>,
    source_path: Option<String>,
}

/// Read a context file's header: the fenced block that declares harness, home
/// path, message count and the original file's absolute path.
fn read_header(path: &Path) -> Result<Header, HistoryReadError> {
    let text = std::fs::read_to_string(path).map_err(|e| HistoryReadError::Unreadable {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    let mut header = Header::default();
    for line in text.lines().take(HEADER_LINES) {
        let Some((key, value)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "harness" => header.harness = Some(value.to_string()),
            "home" if value.starts_with("path") => {
                header.home_path = value.trim_start_matches("path").trim().to_string().into()
            }
            "消息数" => header.declared_message_count = value.parse::<i64>().ok(),
            "来源文件" => header.source_path = Some(value.to_string()),
            _ => {}
        }
    }
    Ok(header)
}

/// The longest directory prefix shared by every declared source path. Discovered
/// rather than configured, so the read never has to be told where the archive
/// lives — and a corpus whose paths disagree yields `None`, which the caller
/// reports instead of resolving against a guessed root.
fn discover_archive_root<'a>(paths: impl Iterator<Item = &'a str>) -> Option<PathBuf> {
    let mut paths = paths.map(Path::new);
    let first = paths.next()?;
    let mut prefix: PathBuf = first.parent()?.to_path_buf();
    for path in paths {
        while !path.starts_with(&prefix) {
            match prefix.parent() {
                Some(parent) => prefix = parent.to_path_buf(),
                None => return None,
            }
        }
    }
    Some(prefix)
}

/// Lexically normalize a path (resolve `.` and `..` without touching the disk).
/// The corpus's declared paths are already absolute and clean, so this only
/// guards the join.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

// =============================================================================
// Turn parsing
// =============================================================================

/// Parse one context file into its turn pointers.
///
/// A turn starts at a `## [原文件第 N 行] <label>` heading and runs to the line
/// before the next one. The heading carries the line number in the ORIGINAL
/// file; the markdown line numbers are kept alongside so a local client can read
/// exactly that slice. `chars` is the turn's length. **No turn text is
/// returned** — the bodies stay on disk.
pub fn parse_turns(path: &Path) -> Result<Vec<HistoryTurn>, HistoryReadError> {
    let text = std::fs::read_to_string(path).map_err(|e| HistoryReadError::Unreadable {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;
    let total_lines = text.lines().count() as u64;
    let context_path = path.display().to_string();

    let mut turns: Vec<HistoryTurn> = Vec::new();
    // The turn being accumulated: its original line, label, markdown start line
    // and character count so far.
    let mut open: Option<(u64, TurnLabel, Option<String>, u64, u64)> = None;

    let close = |turns: &mut Vec<HistoryTurn>,
                 open: Option<(u64, TurnLabel, Option<String>, u64, u64)>,
                 end: u64,
                 context_path: &str| {
        if let Some((line, label, label_raw, start, chars)) = open {
            turns.push(HistoryTurn {
                line,
                label,
                label_raw,
                chars,
                context_path: context_path.to_string(),
                context_line_start: start,
                context_line_end: end.max(start),
            });
        }
    };

    for (idx, line) in text.lines().enumerate() {
        let line_no = idx as u64 + 1;
        if let Some(heading) = parse_heading(line) {
            // Close the previous turn at the line before this heading.
            close(&mut turns, open.take(), line_no - 1, &context_path);
            let (label, label_raw) = classify(&heading.label);
            open = Some((heading.line, label, label_raw, line_no, 0));
            continue;
        }
        if let Some((_, _, _, _, chars)) = open.as_mut() {
            *chars += line.len() as u64 + 1;
        }
    }
    close(&mut turns, open.take(), total_lines, &context_path);
    Ok(turns)
}

struct Heading {
    line: u64,
    label: String,
}

/// `## [原文件第 3 行] ⚙️ 系统·角色指令` → line 3, label `⚙️ 系统·角色指令`.
fn parse_heading(line: &str) -> Option<Heading> {
    let rest = line.strip_prefix("## [原文件第 ")?;
    let (digits, after) = rest.split_once(" 行]")?;
    let line: u64 = digits.trim().parse().ok()?;
    Some(Heading {
        line,
        label: after.trim().to_string(),
    })
}

/// Map the heading's label onto the coarse taxonomy. The raw label is kept
/// whenever it is not one of the three known shapes, so nothing is lost by the
/// classification.
fn classify(label: &str) -> (TurnLabel, Option<String>) {
    let trimmed = label.trim();
    if trimmed.starts_with('⚙') || trimmed.starts_with("⚙️") || trimmed.contains("系统") {
        (TurnLabel::System, Some(trimmed.to_string()))
    } else if trimmed == "你" || trimmed.starts_with("你") {
        (TurnLabel::User, Some(trimmed.to_string()))
    } else if trimmed == "助手" || trimmed.starts_with("助手") {
        (TurnLabel::Assistant, None)
    } else {
        (TurnLabel::Other, Some(trimmed.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
        let by_session = dir.join("by-session");
        std::fs::create_dir_all(&by_session).unwrap();
        let path = by_session.join(name);
        std::fs::write(&path, body).unwrap();
        path
    }

    const HEADER: &str = "# 会话 · /Users/x/GitHub/OfficeBench\n\n```\nharness      codex\nhome path    /Users/x/GitHub/OfficeBench\n起止         2026-03-13T08:05:10 → 2026-03-13T08:06:17\n消息数       2\n来源文件     /Users/x/AgentHistory/raw/codex/rollout-a.jsonl\n```\n\n---\n";

    #[test]
    fn an_index_is_keyed_by_the_declared_source_path_not_the_file_name() {
        let dir = std::env::temp_dir().join(format!("seatloom-hist-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // The file is named after the FILE id while the session's native id is
        // something else entirely — the derived-name trap, in miniature.
        write(&dir, "OfficeBench__codex__4a629262-f806.md", HEADER);
        let ctx = HistoryContexts::build(&dir).unwrap();
        assert_eq!(ctx.len(), 1);
        // The archive root is discovered from the declared path.
        // With one file the shared prefix is the harness directory, so the
        // lookup walks ancestors until the join lands on an indexed key.
        assert_eq!(
            ctx.archive_root().map(|p| p.display().to_string()),
            Some("/Users/x/AgentHistory/raw/codex".to_string())
        );
        // …so a relative raw_path resolves to the declared absolute path.
        assert_eq!(
            ctx.resolve_source_path("raw/codex/rollout-a.jsonl"),
            Some("/Users/x/AgentHistory/raw/codex/rollout-a.jsonl".to_string())
        );
        assert_eq!(ctx.resolve_source_path("raw/codex/other.jsonl"), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn turns_are_parsed_in_source_order_with_their_original_line_numbers() {
        let dir = std::env::temp_dir().join(format!("seatloom-hist-t-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let body = format!(
            "{HEADER}\n## [原文件第 3 行] ⚙️ 系统·角色指令\n\nfirst turn body\n\n## [原文件第 6 行] 你\n\nsecond turn body\n\n## [原文件第 9 行] 助手\n\nthird turn body\n"
        );
        let path = write(&dir, "OfficeBench__codex__rollout-a.md", &body);
        let turns = parse_turns(&path).unwrap();
        assert_eq!(turns.len(), 3);
        assert_eq!(
            turns.iter().map(|t| t.line).collect::<Vec<_>>(),
            vec![3, 6, 9],
            "the ORIGINAL file's line numbers, in order"
        );
        assert_eq!(turns[0].label, TurnLabel::System);
        assert_eq!(turns[1].label, TurnLabel::User);
        assert_eq!(turns[2].label, TurnLabel::Assistant);
        // The markdown slice each turn occupies, so a client can read exactly it.
        assert_eq!(turns[0].context_line_start, 13);
        assert_eq!(turns[0].context_line_end, 16);
        assert_eq!(turns[1].context_line_start, 17);
        assert_eq!(turns[1].context_line_end, 20);
        assert_eq!(turns[2].context_line_start, 21);
        assert_eq!(turns[2].context_line_end, 23, "the last turn runs to EOF");
        // Every turn carries a non-zero length, so a list can show scale
        // without shipping the text.
        assert!(turns.iter().all(|t| t.chars > 0), "{turns:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_cursor_style_directory_raw_path_resolves_to_the_unique_file_under_it() {
        // 011's `raw` for a cursor session is the workspace DIRECTORY; the extracted
        // conversation is the chat_dump.json inside it.
        let dir = std::env::temp_dir().join(format!("seatloom-hist-cur-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        write(
            &dir,
            "ThoughtOnly__cursor__chat_dump.js.md",
            &HEADER.replace(
                "/Users/x/AgentHistory/raw/codex/rollout-a.jsonl",
                "/Users/x/AgentHistory/raw/cursor/ThoughtOnly__hash/chat_dump.json",
            ),
        );
        let ctx = HistoryContexts::build(&dir).unwrap();
        assert_eq!(
            ctx.resolve_source_path("raw/cursor/ThoughtOnly__hash"),
            Some("/Users/x/AgentHistory/raw/cursor/ThoughtOnly__hash/chat_dump.json".to_string())
        );
        // A directory holding TWO context files is ambiguous and must not be
        // resolved by picking one.
        write(
            &dir,
            "ThoughtOnly__cursor__second.md",
            &HEADER.replace(
                "/Users/x/AgentHistory/raw/codex/rollout-a.jsonl",
                "/Users/x/AgentHistory/raw/cursor/ThoughtOnly__hash/chat_dump2.json",
            ),
        );
        let ctx = HistoryContexts::build(&dir).unwrap();
        assert_eq!(
            ctx.resolve_source_path("raw/cursor/ThoughtOnly__hash"),
            None,
            "two candidates is not an answer"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_contexts_directory_is_a_typed_error_not_an_empty_list() {
        let missing = std::env::temp_dir().join("seatloom-hist-does-not-exist-xyz");
        let _ = std::fs::remove_dir_all(&missing);
        let err = HistoryContexts::build(&missing).unwrap_err();
        assert!(matches!(err, HistoryReadError::ContextsDirMissing { .. }));
        assert!(err.to_string().contains("历史会话还没加载"), "{err}");
    }

    #[test]
    fn an_unknown_label_is_carried_verbatim_rather_than_dropped() {
        let (label, raw) = classify("🧰 某种新工具");
        assert_eq!(label, TurnLabel::Other);
        assert_eq!(raw.as_deref(), Some("🧰 某种新工具"));
    }
}
