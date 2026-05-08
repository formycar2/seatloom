// Session commands.
//
// Read commands (Phase 1) list DB-persisted session rows.
// Live commands (Phase 4) spawn PTY-wrapped agent processes, stream output
// via Tauri events, accept keystroke/resize/kill operations.
//
// Each live session emits two Tauri event channels:
//   - "session:output" — payload { sessionId, data (base64) }
//   - "session:exit"   — payload { sessionId, exitCode, signal }

use crate::dto::{CheckpointDto, SessionDto};
use crate::state::AppState;
use base64::Engine as _;
use seatloom_core::pty::{default_transcripts_dir, LaunchOptions, PtyEvent, PtySession};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
// Live PTY commands (Phase 4)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequest {
    /// Seat to attribute the session to (optional — "human" seat is default).
    pub seat_id: Option<String>,
    /// Runtime label for bookkeeping ("ClaudeCode" / "GeminiCli" / "Custom").
    pub runtime: String,
    /// Command to exec (e.g. "claude", "gemini", "bash").
    pub command: String,
    /// Command args.
    pub args: Vec<String>,
    /// Working directory; defaults to the repo root.
    pub working_dir: Option<String>,
    /// Initial terminal size.
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

#[tauri::command]
pub async fn cmd_launch_session(
    state: State<'_, AppState>,
    app: AppHandle,
    request: LaunchRequest,
) -> Result<LiveSessionDto, String> {
    let id = new_session_id();
    let working_dir = request
        .working_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| state.repo_root.clone());
    let transcripts_dir = default_transcripts_dir(&state.repo_root);

    let opts = LaunchOptions {
        command: request.command.clone(),
        args: request.args.clone(),
        working_dir: working_dir.clone(),
        rows: request.rows.unwrap_or(30),
        cols: request.cols.unwrap_or(120),
        env: LaunchOptions::default_env(),
        transcripts_dir,
    };

    let session = PtySession::launch(&id, opts).map_err(|e| e.to_string())?;

    // Spawn a tokio task that forwards PTY events to the Tauri app as JSON events.
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
                    // Auto-deregister from live registry on exit.
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
        runtime: request.runtime.clone(),
        command: request.command.clone(),
        args: request.args.clone(),
        working_dir: working_dir.display().to_string(),
        transcript_path: session.transcript_path.display().to_string(),
    };

    state.sessions.lock().await.insert(id.clone(), session);
    Ok(dto)
}

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
    // `data` is interpreted as raw UTF-8 keystrokes. Frontend can send "\n" etc.
    sess.write(data.as_bytes()).map_err(|e| e.to_string())
}

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

#[tauri::command]
pub async fn cmd_kill_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let sessions = state.sessions.lock().await;
    if let Some(sess) = sessions.get(&session_id) {
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
