// Supervisor IM commands (R2).
//
// Persists supervisor chat messages into canonical_events and, when the target
// seat has a live PTY session, auto-pipes the message into that PTY. Emits a
// Tauri event `canonical:appended` so all subscribed webviews (embedded IM in
// main window + detached IM window once R3 lands) refresh in real time.

use crate::dto::CanonicalEventDto;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

fn new_event_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("evt-{nanos:016x}")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendSupervisorRequest {
    /// Seat id the message is addressed to. Null = broadcast / global supervisor chat.
    pub target_seat_id: Option<String>,
    pub content: String,
    /// 'SupervisorMessage' (default, from human) or 'SeatResponse'.
    pub event_type: Option<String>,
    /// Who sent the message; defaults to 'human:user'.
    pub actor_ref: Option<String>,
    /// Project id the event belongs to (AD-013 v2 backend isolation per
    /// schema 008). Frontend Supervisor store passes activeProjectId here when
    /// in project mode; defaults to AppState.default_project_id otherwise.
    pub project_id: Option<String>,
}

#[tauri::command]
pub async fn cmd_append_supervisor_message(
    state: State<'_, AppState>,
    app: AppHandle,
    request: AppendSupervisorRequest,
) -> Result<CanonicalEventDto, String> {
    let event_type = request
        .event_type
        .unwrap_or_else(|| "SupervisorMessage".to_string());
    let actor_ref = request.actor_ref.unwrap_or_else(|| "human:user".to_string());
    let project_id = request
        .project_id
        .unwrap_or_else(|| state.default_project_id.clone());
    let id = new_event_id();

    let row = state
        .db
        .append_supervisor_message(
            &id,
            &actor_ref,
            request.target_seat_id.as_deref(),
            &request.content,
            &event_type,
            &project_id,
        )
        .await
        .map_err(|e| e.to_string())?;

    // If the target seat has a live PTY session, forward the text into that PTY
    // so the wrapped agent sees the user's message in its terminal.
    if let Some(target_seat) = request.target_seat_id.as_deref() {
        let sessions = state.sessions.lock().await;
        // Best-effort: live session registry is keyed by session id, not seat
        // id. We look up by seat match on PtySession metadata is not yet
        // captured; for v0.1 we accept that this inject only fires when the
        // caller already knows the session id. Callers may separately call
        // cmd_pty_write. Leaving this branch as an integration hook for later.
        drop(sessions);
        let _ = target_seat; // silence unused warning
    }

    let dto: CanonicalEventDto = row.into();
    let _ = app.emit("canonical:appended", dto.clone());
    Ok(dto)
}

#[tauri::command]
pub async fn cmd_list_supervisor_messages(
    state: State<'_, AppState>,
    target_seat_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<CanonicalEventDto>, String> {
    state
        .db
        .list_supervisor_messages(target_seat_id.as_deref(), limit.unwrap_or(200))
        .await
        .map(|rows| rows.into_iter().map(CanonicalEventDto::from).collect())
        .map_err(|e| e.to_string())
}

// -----------------------------------------------------------------------------
// Multi-window management (R3)
// -----------------------------------------------------------------------------

const SUPERVISOR_WINDOW_LABEL: &str = "supervisor";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisorWindowStatus {
    pub detached: bool,
}

/// Open (or focus) the detached Supervisor IM window.
#[tauri::command]
pub async fn cmd_open_supervisor_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(SUPERVISOR_WINDOW_LABEL) {
        win.set_focus().map_err(|e| e.to_string())?;
        let _ = app.emit("supervisor:detached", ());
        return Ok(());
    }
    WebviewWindowBuilder::new(
        &app,
        SUPERVISOR_WINDOW_LABEL,
        WebviewUrl::App("supervisor.html".into()),
    )
    .title("SeatLoom · Supervisor")
    .inner_size(720.0, 620.0)
    .min_inner_size(480.0, 420.0)
    .build()
    .map_err(|e| e.to_string())?;
    let _ = app.emit("supervisor:detached", ());
    Ok(())
}

/// Close the detached Supervisor IM window (if present).
#[tauri::command]
pub async fn cmd_close_supervisor_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(SUPERVISOR_WINDOW_LABEL) {
        win.close().map_err(|e| e.to_string())?;
    }
    let _ = app.emit("supervisor:reembedded", ());
    Ok(())
}

#[tauri::command]
pub async fn cmd_supervisor_window_status(
    app: AppHandle,
) -> Result<SupervisorWindowStatus, String> {
    Ok(SupervisorWindowStatus {
        detached: app.get_webview_window(SUPERVISOR_WINDOW_LABEL).is_some(),
    })
}
