use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::objects::id::SessionId;
use crate::objects::session::Session;
use crate::storage::project::ProjectPaths;
use crate::storage::yaml_io::{read_yaml, YamlIoError};

pub struct SessionStore {
    paths: ProjectPaths,
}

impl SessionStore {
    pub fn new(root: &Path) -> Self {
        Self {
            paths: ProjectPaths::new(root),
        }
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn load_session(&self, session_id: &SessionId) -> Result<Session, SessionStoreError> {
        Ok(read_yaml(&self.paths.session_meta_path(session_id))?)
    }

    /// List all sessions by scanning `.seatloom/sessions/*/meta.yaml`.
    /// Returns empty Vec when the sessions directory does not exist.
    /// Sort: by `created_at` descending (most recent first, deterministic).
    pub fn list_sessions(&self) -> Result<Vec<Session>, SessionStoreError> {
        let sessions_dir = self.paths.sessions_dir();
        if !sessions_dir.is_dir() {
            return Ok(vec![]);
        }
        let mut entries: Vec<_> = fs::read_dir(&sessions_dir)
            .map_err(|source| SessionStoreError::Io {
                path: sessions_dir.clone(),
                source,
            })?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut sessions = Vec::new();
        for entry in entries {
            let meta_path = entry.path().join("meta.yaml");
            if meta_path.is_file() {
                sessions.push(read_yaml::<Session>(&meta_path)?);
            }
        }
        sessions.sort_by_key(|s| std::cmp::Reverse(s.created_at));
        Ok(sessions)
    }
}

#[derive(Debug, Error)]
pub enum SessionStoreError {
    #[error(transparent)]
    Yaml(#[from] YamlIoError),
    #[error("failed to read directory {}: {source}", path.display())]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::{Duration, TimeZone, Utc};

    use crate::objects::id::{SeatId, SessionId};
    use crate::objects::session::{Runtime, Session, SessionStatus};
    use crate::storage::session_store::SessionStore;
    use crate::storage::yaml_io::write_yaml;

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("seatloom-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    fn make_session(
        id: &SessionId,
        seat_id: &SeatId,
        created_at: chrono::DateTime<Utc>,
    ) -> Session {
        Session {
            id: id.clone(),
            seat_id: seat_id.clone(),
            runtime: Runtime::ClaudeCode,
            native_session_id: None,
            workspace_path: PathBuf::from("/tmp/test"),
            branch: None,
            status: SessionStatus::Completed,
            launch_pack_ref: None,
            last_checkpoint_id: None,
            pid: None,
            created_at,
            ended_at: None,
        }
    }

    #[test]
    fn missing_sessions_dir_returns_empty_vec() {
        let dir = temp_dir("session-missing-dir");
        let store = SessionStore::new(&dir);
        let sessions = store
            .list_sessions()
            .expect("missing dir should return empty");
        assert!(sessions.is_empty());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn session_round_trip_and_list() {
        let dir = temp_dir("session-list-roundtrip");
        let store = SessionStore::new(&dir);
        let seat_id = SeatId::new();

        let ts1 = Utc.with_ymd_and_hms(2026, 4, 29, 1, 0, 0).single().unwrap();
        let ts2 = ts1 + Duration::hours(2);
        let id1 = SessionId::new();
        let id2 = SessionId::new();

        let s1 = make_session(&id1, &seat_id, ts1);
        let s2 = make_session(&id2, &seat_id, ts2);

        write_yaml(&store.paths().session_meta_path(&id1), &s1).expect("write s1");
        write_yaml(&store.paths().session_meta_path(&id2), &s2).expect("write s2");

        let sessions = store.list_sessions().expect("list should succeed");
        assert_eq!(sessions.len(), 2);
        // Sort: created_at descending — s2 (newer) first
        assert_eq!(sessions[0].id.as_str(), id2.as_str());
        assert_eq!(sessions[1].id.as_str(), id1.as_str());

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn resolves_session_paths_deterministically() {
        let store = SessionStore::new(std::path::Path::new("/tmp/seatloom-project"));
        let paths = store.paths();
        let session_id = SessionId::new();
        assert_eq!(
            paths.session_meta_path(&session_id),
            PathBuf::from(format!(
                "/tmp/seatloom-project/.seatloom/sessions/{}/meta.yaml",
                session_id.as_str()
            ))
        );
    }
}
