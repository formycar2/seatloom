use seatloom_core::objects::session::Session;
use seatloom_core::storage::session_store::SessionStore;

/// List all sessions for the current project.
/// Bounded assumption: project root is `std::env::current_dir()`.
/// Sort: by `created_at` descending.
#[allow(dead_code)] // scaffold: not yet registered with Tauri invoke handler
pub fn list_sessions() -> Vec<Session> {
    let root = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };
    let store = SessionStore::new(&root);
    store.list_sessions().unwrap_or_default()
}

/// Attach SeatLoom to an already-running agent process.
#[allow(dead_code)] // scaffold: not yet registered with Tauri invoke handler
pub fn attach_session(_seat_id: String, _runtime: String, _pid: u32) -> Option<Session> {
    // TODO: write session meta + emit SessionStarted event
    None
}
