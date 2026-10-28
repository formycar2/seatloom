// Historical session corpus — the Tauri half of the read path.
//
// Product contract: LYRA-2026-10-14-history-sessions-in-ui-contract-v1 §8.2
// slice 2. Each command shells out to the SAME seatloom-mcp-emit binary the
// seat's own MCP tool calls, exactly the way `cmd_enqueue_dispatch` shells out to
// `enqueue_dispatch` and `cmd_plan_intent` shells out to `plan_intent`. There is
// one implementation (seatloom_core::db + seatloom_core::history); these three
// commands add the stdio envelope and nothing else.
//
// No transcript body crosses this boundary either: the turn read returns
// pointers, and the UI reads the slice it is pointed at from the local file.
//
// ─── Ownership ──────────────────────────────────────────────────────────────
// The three tool wrappers above are Nimbus's slice-2 code, taken verbatim from
// `packet/nimbus/gate-vs-announcement` @ 0505636 (his dispatch
// `disp-history-sessions-read` is `done`). Mira added them here because slice 3
// cannot call a command that is not registered in this build, and integrated
// them path-limited (3 registration lines, 0 deletions — the 2026-10-14 lesson:
// taking his whole `main.rs` would have deleted the HOME fallback that a
// path-limited integration had already destroyed once).
//
// The two commands at the bottom (`cmd_read_history_context_slice` /
// `cmd_open_local_path`) are Mira's slice-3 additions, NOT Nimbus's: slice 2
// deliberately returns pointers only, and nothing in the stack could
// dereference them, so the operator could not read the conversation his
// acceptance card step 3 asks him to read. They read the operator's own local
// file; no body is stored in the database and none crosses MCP (011's rule
// stands).

use crate::state::AppState;
use tauri::State;

/// Drive the MCP binary over stdio and return the tool's text result, or the
/// tool's own rejection verbatim (never a transport error).
async fn call_tool(
    state: &AppState,
    tool: &str,
    arguments: serde_json::Value,
) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}"#;
    let inited = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call = serde_json::json!({
        "jsonrpc": "2.0", "id": 9, "method": "tools/call",
        "params": { "name": tool, "arguments": arguments },
    });
    let input = format!("{init}\n{inited}\n{call}\n");

    let bin = state.repo_root.join("target/debug/seatloom-mcp-emit");
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://seatloom:seatloom@localhost:5432/seatloom".to_string());

    let mut child = Command::new(&bin)
        .env("DATABASE_URL", &db_url)
        // The operator is who reads the corpus, so the actor is the operator.
        .env("SEATLOOM_ACTOR_REF", "human:supervisor")
        .env("SEATLOOM_PROJECT_ID", &state.default_project_id)
        // The extracted contexts live at `<repo root>/resume-contexts` (the location the
        // contract records). Set it explicitly instead of letting the child inherit this
        // process's CWD: launched from /Applications the CWD is `/`, and
        // `default_contexts_dir()` would then look at `/resume-contexts` — the same
        // failure mode `repo_root()`'s HOME fallback above already guards against for the
        // binary path. Following `state.repo_root` keeps the two consistent. When the
        // directory is absent the read refuses with that path named, never an empty list.
        .env(
            "SEATLOOM_HISTORY_CONTEXTS_DIR",
            state.repo_root.join("resume-contexts"),
        )
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", bin.display()))?;

    {
        let mut stdin = child.stdin.take().ok_or("no stdin")?;
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        // drop(stdin) at scope end closes the pipe -> server sees EOF and exits.
    }

    let out = child.wait_with_output().await.map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last = stdout
        .lines()
        .rfind(|l| !l.trim().is_empty())
        .ok_or(format!("{tool} produced no output"))?;
    let v: serde_json::Value = serde_json::from_str(last).map_err(|e| e.to_string())?;
    let r = &v["result"];
    let text = r["content"][0]["text"].as_str().unwrap_or("").to_string();
    if r["isError"].as_bool().unwrap_or(false) {
        Err(text)
    } else {
        Ok(text)
    }
}

/// (a) The projects that have archived sessions, with session and message counts.
#[tauri::command]
pub async fn cmd_list_history_projects(state: State<'_, AppState>) -> Result<String, String> {
    call_tool(&state, "list_history_projects", serde_json::json!({})).await
}

/// (b) One project's sessions. Dated first, undated last and flagged.
#[tauri::command]
pub async fn cmd_list_history_sessions(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<String, String> {
    call_tool(
        &state,
        "list_history_sessions",
        serde_json::json!({ "project_id": project_id }),
    )
    .await
}

/// (c) One session's turns — pointers, never bodies.
#[tauri::command]
pub async fn cmd_get_history_session_turns(
    state: State<'_, AppState>,
    harness: String,
    id: String,
) -> Result<String, String> {
    call_tool(
        &state,
        "get_history_session_turns",
        serde_json::json!({ "harness": harness, "id": id }),
    )
    .await
}

// =============================================================================
// Mira, slice 3 — dereferencing the pointers on the operator's own disk
// =============================================================================

/// Resolve `context_path` and require it to live under `contexts_root`.
///
/// The path arrives from the read tool's own index, but a command that reads a
/// file should still say which files it reads: this one reads only files under
/// `<repo root>/resume-contexts`. Anything else is refused with the path named,
/// never silently read.
fn resolve_context_path(
    contexts_root: &std::path::Path,
    context_path: &str,
) -> Result<std::path::PathBuf, String> {
    let canonical_root = contexts_root.canonicalize().map_err(|e| {
        format!(
            "contexts root {} is not readable: {e}",
            contexts_root.display()
        )
    })?;
    let candidate = std::path::Path::new(context_path);
    let canonical = candidate
        .canonicalize()
        .map_err(|e| format!("context file {context_path} is not readable: {e}"))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "refused: {context_path} is not under the contexts root {}",
            canonical_root.display()
        ));
    }
    Ok(canonical)
}

/// One turn's text, read off the operator's disk from the pointer the read returned.
#[derive(serde::Serialize)]
pub struct HistoryContextSlice {
    /// The absolute path actually read (canonicalized).
    pub path: String,
    /// 1-based, inclusive.
    pub line_start: u64,
    pub line_end: u64,
    /// The lines, in file order. May be shorter than requested at end-of-file.
    pub lines: Vec<String>,
    /// The file's total line count, so a caller can say when it truncated.
    pub total_lines: u64,
}

/// Read `line_start..=line_end` of a context file so the operator can read the
/// conversation the turn pointers point at (contract §4 session detail).
///
/// 011's rule is about the DATABASE — no transcript body is ever stored there and
/// none crosses MCP. This reads the operator's own extracted markdown, which is
/// where the body already lives, and returns only the slice asked for.
#[tauri::command]
pub async fn cmd_read_history_context_slice(
    state: State<'_, AppState>,
    context_path: String,
    line_start: u64,
    line_end: u64,
) -> Result<HistoryContextSlice, String> {
    if line_start < 1 {
        return Err(format!("line_start must be 1-based, got {line_start}"));
    }
    if line_end < line_start {
        return Err(format!(
            "line_end {line_end} is before line_start {line_start}"
        ));
    }
    let path = resolve_context_path(&state.repo_root.join("resume-contexts"), &context_path)?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let all: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    let total_lines = all.len() as u64;
    let from = (line_start as usize).saturating_sub(1);
    let to = (line_end as usize).min(all.len());
    let lines = if from >= all.len() {
        Vec::new()
    } else {
        all[from..to].to_vec()
    };
    Ok(HistoryContextSlice {
        path: path.display().to_string(),
        line_start,
        line_end,
        lines,
        total_lines,
    })
}

/// Open a local path with the OS (contract §4: "the source file path is shown
/// and is openable"). Opens only; it reads nothing and returns nothing.
///
/// The path is one the app itself received from the history read for the session
/// currently open — the UI never passes a path the operator typed.
#[tauri::command]
pub async fn cmd_open_local_path(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.is_file() {
        return Err(format!("{path} is not a file on this machine"));
    }
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let opener = return Err("opening a local file is only wired for macOS and Linux".to_string());
    std::process::Command::new(opener)
        .arg(&path)
        .spawn()
        .map_err(|e| format!("{opener} {path}: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_context_path_outside_the_contexts_root_is_refused() {
        let dir = std::env::temp_dir().join("seatloom-history-cmds-test-outside");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("resume-contexts/by-session")).unwrap();
        let outside = dir.join("secret.txt");
        std::fs::write(&outside, "not a context file\n").unwrap();
        let err = resolve_context_path(&dir.join("resume-contexts"), outside.to_str().unwrap())
            .unwrap_err();
        assert!(err.contains("refused"), "{err}");
        assert!(err.contains("secret.txt"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_context_path_inside_the_contexts_root_resolves() {
        let dir = std::env::temp_dir().join("seatloom-history-cmds-test-inside");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("resume-contexts/by-session")).unwrap();
        let inside = dir.join("resume-contexts/by-session/a.md");
        std::fs::write(&inside, "l1\nl2\nl3\n").unwrap();
        let resolved =
            resolve_context_path(&dir.join("resume-contexts"), inside.to_str().unwrap()).unwrap();
        assert_eq!(resolved, inside.canonicalize().unwrap());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_contexts_root_names_the_path_rather_than_reading_nothing() {
        let dir = std::env::temp_dir().join("seatloom-history-cmds-test-missing-root");
        let _ = std::fs::remove_dir_all(&dir);
        let err = resolve_context_path(&dir.join("resume-contexts"), "/whatever/a.md").unwrap_err();
        assert!(err.contains("not readable"), "{err}");
        assert!(err.contains("resume-contexts"), "{err}");
    }
}
