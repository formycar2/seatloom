// Session commands.
//
// Read commands (Phase 1) list DB-persisted session rows.
//
// Live commands (v0.0.1 A3) no longer spawn CLIs via portable-pty — per the
// tmux-mirror architecture (a5998c1), SeatLoom attaches to existing tmux
// sessions. Each live session emits two Tauri event channels:
//   - "session:output" — payload { sessionId, data (base64) } — bytes
//     mirrored from `tmux pipe-pane` into /tmp/seatloom-mirror/<id>.fifo
//   - "session:exit"   — payload { sessionId, exitCode, signal } — emitted
//     when the pipe-pane tail closes (e.g., kill_session detaches the mirror)
//
// R3 failure isolation: SeatLoom is NEVER the parent process of tmux. Even if
// SeatLoom crashes, tmux continues running and user workflow is unaffected.

use crate::dto::{CheckpointDto, SessionDto};
use crate::state::AppState;
use base64::Engine as _;
use seatloom_core::pty::{
    default_transcripts_dir, list_tmux_sessions, LaunchOptions, PtyEvent, PtySession,
    TmuxSessionInfo,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

// -----------------------------------------------------------------------------
// Read commands (Phase 1)
// -----------------------------------------------------------------------------

#[tauri::command]
pub async fn cmd_list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionDto>, String> {
    state
        .db
        .list_sessions()
        .await
        .map(|rows| rows.into_iter().map(SessionDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_sessions_for_seat(
    state: State<'_, AppState>,
    seat_id: String,
) -> Result<Vec<SessionDto>, String> {
    state
        .db
        .list_sessions_for_seat(&seat_id)
        .await
        .map(|rows| rows.into_iter().map(SessionDto::from).collect())
        .map_err(|e| e.to_string())
}

/// Project-mode session list. AD-013 v2 backend invariant — schema 008.
#[tauri::command]
pub async fn cmd_list_sessions_for_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<SessionDto>, String> {
    state
        .db
        .list_sessions_for_project(&project_id)
        .await
        .map(|rows| rows.into_iter().map(SessionDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_checkpoints(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<CheckpointDto>, String> {
    state
        .db
        .list_checkpoints(limit.unwrap_or(50))
        .await
        .map(|rows| rows.into_iter().map(CheckpointDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_checkpoints_for_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<CheckpointDto>, String> {
    state
        .db
        .list_checkpoints_for_session(&session_id)
        .await
        .map(|rows| rows.into_iter().map(CheckpointDto::from).collect())
        .map_err(|e| e.to_string())
}

// -----------------------------------------------------------------------------
// Live tmux-mirror commands (v0.0.1 A3)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachTmuxRequest {
    /// Seat to attribute the mirrored session to (optional).
    pub seat_id: Option<String>,
    /// tmux session name to mirror (e.g., "Lyra-po-seatloom").
    pub tmux_session_name: String,
    /// Optional explicit SeatLoom session id; generated if absent.
    pub session_id: Option<String>,
    /// Initial pane size (used by `tmux resize-pane`).
    pub rows: Option<u16>,
    pub cols: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSessionDto {
    pub id: String,
    pub seat_id: Option<String>,
    pub runtime: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub transcript_path: String,
    /// tmux session name being mirrored (empty for legacy callers).
    #[serde(default)]
    pub tmux_session_name: String,
    /// Path to the pipe-pane FIFO (empty for legacy callers).
    #[serde(default)]
    pub fifo_path: String,
    /// Actual tmux pane dimensions at attach time (queried via display-message).
    #[serde(default)]
    pub pane_rows: u16,
    #[serde(default)]
    pub pane_cols: u16,
    /// Base64-encoded snapshot of the tmux pane at attach time (capture-pane).
    /// Frontend writes this to xterm before subscribing to session:output events
    /// so idle panes don't render as a black screen.
    #[serde(default)]
    pub initial_snapshot_b64: String,
}

fn new_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ses-{nanos:016x}")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputEventPayload {
    session_id: String,
    data: String, // base64-encoded raw bytes — preserves ANSI/UTF-8 fidelity
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExitEventPayload {
    session_id: String,
    exit_code: Option<i32>,
    signal: Option<String>,
}

/// Discover tmux sessions whose name ends in `-seatloom`. Returns an empty
/// list if tmux is not running or no matching sessions exist.
#[tauri::command]
pub async fn cmd_list_tmux_sessions() -> Result<Vec<TmuxSessionInfo>, String> {
    list_tmux_sessions().map_err(|e| e.to_string())
}

/// Attach to an existing tmux session and begin mirroring its first pane into
/// a FIFO under /tmp/seatloom-mirror/. Emits `session:output` events for each
/// chunk of bytes read from the FIFO.
///
/// R3 rule: SeatLoom never becomes the parent of tmux. If SeatLoom crashes,
/// tmux continues running and the FIFO is re-attachable on restart.
#[tauri::command]
pub async fn cmd_attach_tmux_session(
    state: State<'_, AppState>,
    app: AppHandle,
    request: AttachTmuxRequest,
) -> Result<LiveSessionDto, String> {
    let id = request.session_id.unwrap_or_else(new_session_id);
    let transcripts_dir = default_transcripts_dir(&state.repo_root);

    let opts = LaunchOptions {
        command: "tmux".to_string(),
        args: vec!["pipe-pane".to_string(), request.tmux_session_name.clone()],
        working_dir: state.repo_root.clone(),
        rows: request.rows.unwrap_or(30),
        cols: request.cols.unwrap_or(120),
        env: LaunchOptions::default_env(),
        transcripts_dir,
    };

    let session = PtySession::attach_tmux(&id, &request.tmux_session_name, opts)
        .map_err(|e| e.to_string())?;

    // Forward tmux-mirror events to the Tauri app as JSON events.
    let mut rx = session.subscribe();
    let app_for_events = app.clone();
    let id_for_events = id.clone();
    let sessions_for_exit = state.sessions.clone();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(PtyEvent::Output(bytes)) => {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    let payload = OutputEventPayload {
                        session_id: id_for_events.clone(),
                        data: encoded,
                    };
                    let _ = app_for_events.emit("session:output", payload);
                }
                Ok(PtyEvent::Exited { exit_code, signal }) => {
                    let payload = ExitEventPayload {
                        session_id: id_for_events.clone(),
                        exit_code,
                        signal,
                    };
                    let _ = app_for_events.emit("session:exit", payload);
                    let mut sessions = sessions_for_exit.lock().await;
                    sessions.remove(&id_for_events);
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let dto = LiveSessionDto {
        id: id.clone(),
        seat_id: request.seat_id.clone(),
        runtime: "tmux-mirror".to_string(),
        command: "tmux pipe-pane".to_string(),
        args: vec![request.tmux_session_name.clone()],
        working_dir: state.repo_root.display().to_string(),
        transcript_path: session.transcript_path.display().to_string(),
        tmux_session_name: session.tmux_session_name.clone(),
        fifo_path: session.fifo_path.display().to_string(),
        pane_rows: session.pane_rows,
        pane_cols: session.pane_cols,
        initial_snapshot_b64: base64::engine::general_purpose::STANDARD
            .encode(&session.initial_snapshot),
    };

    state.sessions.lock().await.insert(id.clone(), session);
    Ok(dto)
}

/// v0.0.1 transitional shim: kept so existing UI callers that still invoke
/// `cmd_launch_session({runtime:'tmux', command:<tmux_session_name>, ...})`
/// can reach `cmd_attach_tmux_session` without a breaking rename. B1
/// (v0.0.2) introduces true `tmux send-keys` write support at which point the
/// command surface may be further consolidated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    pub seat_id: Option<String>,
    pub runtime: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub rows: Option<u16>,
    pub cols: Option<u16>,
}

#[tauri::command]
pub async fn cmd_launch_session(
    state: State<'_, AppState>,
    app: AppHandle,
    request: LaunchRequest,
) -> Result<LiveSessionDto, String> {
    // The `command` field is reinterpreted as the tmux session name under the
    // v0.0.1 mirror architecture; the legacy portable-pty spawn path is gone.
    let tmux_session_name = if request.runtime == "tmux" || request.runtime == "tmux-mirror" {
        request.command.clone()
    } else {
        // For non-tmux runtimes, fail loudly — v0.0.1 has no other launch mode.
        return Err(format!(
            "cmd_launch_session: runtime {:?} no longer supported; use cmd_attach_tmux_session",
            request.runtime
        ));
    };
    let attach = AttachTmuxRequest {
        seat_id: request.seat_id,
        tmux_session_name,
        session_id: None,
        rows: request.rows,
        cols: request.cols,
    };
    cmd_attach_tmux_session(state, app, attach).await
}

/// v0.0.2 bidirectional write path: routes UTF-8 text into the mirrored tmux
/// pane via `PtySession::write` (tmux load-buffer|paste-buffer).
#[tauri::command]
pub async fn cmd_pty_write(
    state: State<'_, AppState>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let sessions = state.sessions.lock().await;
    let sess = sessions
        .get(&session_id)
        .ok_or_else(|| format!("session not found: {session_id}"))?;
    sess.write(data.as_bytes()).map_err(|e| e.to_string())
}

/// v0.0.2 bidirectional byte-level write path: routes raw bytes (including
/// NUL, 0x03 Ctrl-C, ANSI control sequences) through `PtySession::write`.
/// Used by xterm.js `onData` for arbitrary keystroke / paste payloads.
#[tauri::command]
pub async fn cmd_pty_write_bytes(
    state: State<'_, AppState>,
    session_id: String,
    bytes: Vec<u8>,
) -> Result<(), String> {
    let sessions = state.sessions.lock().await;
    let sess = sessions
        .get(&session_id)
        .ok_or_else(|| format!("session not found: {session_id}"))?;
    sess.write(&bytes).map_err(|e| e.to_string())
}

/// Resize the mirrored tmux pane (maps to `tmux resize-pane`).
#[tauri::command]
pub async fn cmd_pty_resize(
    state: State<'_, AppState>,
    session_id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let sessions = state.sessions.lock().await;
    let sess = sessions
        .get(&session_id)
        .ok_or_else(|| format!("session not found: {session_id}"))?;
    sess.resize(rows, cols).map_err(|e| e.to_string())
}

/// Detach the tmux mirror for this session.
///
/// Stops the `tmux pipe-pane` copy, removes the FIFO, and drops the session
/// handle from the live registry. The tmux session itself continues running
/// — that is the R3 failure-isolation contract: SeatLoom observes, never owns.
#[tauri::command]
pub async fn cmd_kill_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().await;
    if let Some(sess) = sessions.remove(&session_id) {
        sess.kill().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn cmd_list_live_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let sessions = state.sessions.lock().await;
    Ok(sessions.keys().cloned().collect())
}
