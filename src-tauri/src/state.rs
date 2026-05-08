// Tauri-managed application state.
// Holds the shared PostgreSQL connection pool wrapped in SeatloomDb,
// plus the live-PTY session registry.

use seatloom_core::db::repositories::SeatloomDb;
use seatloom_core::pty::PtySession;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    /// Canonical truth store repository.
    pub db: SeatloomDb,
    /// Repository root — used for reconcile scans and transcript paths.
    pub repo_root: PathBuf,
    /// Default project id for v0.1 (single-project mode).
    pub default_project_id: String,
    /// Live PTY session registry, keyed by SeatLoom session id.
    pub sessions: Arc<Mutex<HashMap<String, PtySession>>>,
}

impl AppState {
    pub fn new(db: SeatloomDb, repo_root: PathBuf, default_project_id: String) -> Self {
        Self {
            db,
            repo_root,
            default_project_id,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
