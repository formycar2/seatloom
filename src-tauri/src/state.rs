// Tauri-managed application state.
// Holds the shared PostgreSQL connection pool wrapped in SeatloomDb.

use seatloom_core::db::repositories::SeatloomDb;
use std::path::PathBuf;

pub struct AppState {
    /// Canonical truth store repository.
    pub db: SeatloomDb,
    /// Repository root — used for reconcile scans.
    pub repo_root: PathBuf,
    /// Default project id for v0.1 (single-project mode).
    pub default_project_id: String,
}

impl AppState {
    pub fn new(db: SeatloomDb, repo_root: PathBuf, default_project_id: String) -> Self {
        Self {
            db,
            repo_root,
            default_project_id,
        }
    }
}
