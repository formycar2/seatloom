// SeatLoom Tauri app entry point.
// - Initializes PostgreSQL connection pool and SeatloomDb repository.
// - Manages AppState as Tauri shared state.
// - Registers all v0.1 IPC commands (read-only).
//
// Phase 4 will add PTY session launch/attach commands alongside this.

mod commands;
mod dto;
mod state;

use seatloom_core::db::connection::create_pool;
use seatloom_core::db::repositories::SeatloomDb;
use state::AppState;
use std::path::PathBuf;

use commands::{
    artifact_cmds::{cmd_get_artifact, cmd_list_artifacts},
    document_cmds::{
        cmd_get_document, cmd_list_document_associations, cmd_list_document_sections,
        cmd_list_document_versions, cmd_list_documents,
    },
    handoff_cmds::{cmd_list_handoff_receipts, cmd_list_handoffs},
    inbox_cmds::cmd_get_inbox,
    project_cmds::{cmd_get_project, cmd_list_projects},
    prompt_cmds::{cmd_list_active_prompts, cmd_list_prompt_actions, cmd_list_prompts_for_session},
    reconcile_cmds::{
        cmd_get_reconcile_run, cmd_list_reconcile_items, cmd_list_reconcile_runs, cmd_reconcile,
    },
    seat_cmds::{
        cmd_get_delegation, cmd_get_seat, cmd_list_delegations, cmd_list_role_bindings,
        cmd_list_seats,
    },
    session_cmds::{
        cmd_attach_tmux_session, cmd_kill_session, cmd_launch_session, cmd_list_checkpoints,
        cmd_list_checkpoints_for_session, cmd_list_live_sessions, cmd_list_sessions,
        cmd_list_sessions_for_seat, cmd_list_tmux_sessions, cmd_pty_resize, cmd_pty_write,
        cmd_pty_write_bytes,
    },
    supervisor_cmds::{
        cmd_append_supervisor_message, cmd_close_supervisor_window,
        cmd_list_supervisor_messages, cmd_open_supervisor_window, cmd_supervisor_window_status,
    },
    timeline_cmds::{cmd_list_events, cmd_list_events_by_type},
    workitem_cmds::{cmd_get_workitem, cmd_list_workitems},
};

/// Basic connectivity probe exposed to the frontend so the UI can display
/// "backend online / offline" without importing every typed command.
#[tauri::command]
async fn cmd_ping(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state
        .db
        .ping()
        .await
        .map(|_| "ok".to_string())
        .map_err(|e| e.to_string())
}

fn repo_root() -> PathBuf {
    // `tauri dev` runs with CWD at `src-tauri/`, so the repo root is one level up.
    // `tauri build` + production runs with CWD at the bundle location — users launch
    // the desktop app against their own repo via SEATLOOM_REPO_ROOT env var.
    if let Ok(env_root) = std::env::var("SEATLOOM_REPO_ROOT") {
        return PathBuf::from(env_root);
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // Heuristic: if the parent contains a Cargo.toml with [workspace], use it.
    if let Some(parent) = cwd.parent() {
        if parent.join("Cargo.toml").is_file() && parent.join("docs").is_dir() {
            return parent.to_path_buf();
        }
    }
    if cwd.join("docs").is_dir() {
        return cwd;
    }
    // Fallback to the current directory; reconcile will fail loudly if this is wrong.
    cwd
}

fn build_app_state() -> AppState {
    let pool = create_pool().expect(
        "failed to create PostgreSQL connection pool — ensure DATABASE_URL is reachable \
         (default: postgresql://seatloom:seatloom@localhost:5432/seatloom). \
         Run ./scripts/bootstrap.sh to start the local Postgres container.",
    );
    let db = SeatloomDb::new(pool);
    let root = repo_root();
    AppState::new(db, root, "seatloom".to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(build_app_state())
        .invoke_handler(tauri::generate_handler![
            cmd_ping,
            // projects
            cmd_list_projects,
            cmd_get_project,
            // seats
            cmd_list_seats,
            cmd_get_seat,
            cmd_list_role_bindings,
            cmd_list_delegations,
            cmd_get_delegation,
            // workitems
            cmd_list_workitems,
            cmd_get_workitem,
            // artifacts
            cmd_list_artifacts,
            cmd_get_artifact,
            // handoffs
            cmd_list_handoffs,
            cmd_list_handoff_receipts,
            // sessions + checkpoints
            cmd_list_sessions,
            cmd_list_sessions_for_seat,
            cmd_list_checkpoints,
            cmd_list_checkpoints_for_session,
            // live tmux-mirror sessions (v0.0.1 A3)
            cmd_list_tmux_sessions,
            cmd_attach_tmux_session,
            cmd_launch_session,
            cmd_pty_write,
            cmd_pty_write_bytes,
            cmd_pty_resize,
            cmd_kill_session,
            cmd_list_live_sessions,
            // documents
            cmd_list_documents,
            cmd_get_document,
            cmd_list_document_sections,
            cmd_list_document_associations,
            cmd_list_document_versions,
            // reconcile
            cmd_reconcile,
            cmd_list_reconcile_runs,
            cmd_get_reconcile_run,
            cmd_list_reconcile_items,
            // timeline / events
            cmd_list_events,
            cmd_list_events_by_type,
            // supervisor IM
            cmd_list_supervisor_messages,
            cmd_append_supervisor_message,
            cmd_open_supervisor_window,
            cmd_close_supervisor_window,
            cmd_supervisor_window_status,
            // inbox
            cmd_get_inbox,
            // prompts
            cmd_list_active_prompts,
            cmd_list_prompts_for_session,
            cmd_list_prompt_actions,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SeatLoom Tauri app");
}
