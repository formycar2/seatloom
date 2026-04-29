use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::objects::id::HandoffId;
use crate::objects::handoff::Handoff;
use crate::storage::project::ProjectPaths;
use crate::storage::yaml_io::{read_yaml, write_yaml, YamlIoError};

pub struct HandoffStore {
    paths: ProjectPaths,
}

impl HandoffStore {
    pub fn new(root: &Path) -> Self {
        Self {
            paths: ProjectPaths::new(root),
        }
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn load_handoff(&self, handoff_id: &HandoffId) -> Result<Handoff, HandoffStoreError> {
        Ok(read_yaml(&self.paths.handoff_path(handoff_id))?)
    }

    pub fn save_handoff(&self, handoff: &Handoff) -> Result<(), HandoffStoreError> {
        write_yaml(&self.paths.handoff_path(&handoff.id), handoff)?;
        Ok(())
    }

    /// List all handoffs by scanning `.seatloom/handoffs/*.yaml`.
    /// Returns empty Vec when the handoffs directory does not exist.
    /// Sort: by `created_at` descending (most recent first, deterministic).
    pub fn list_handoffs(&self) -> Result<Vec<Handoff>, HandoffStoreError> {
        let handoffs_dir = self.paths.handoffs_dir();
        if !handoffs_dir.is_dir() {
            return Ok(vec![]);
        }
        let mut entries: Vec<_> = fs::read_dir(&handoffs_dir)
            .map_err(|source| HandoffStoreError::Io {
                path: handoffs_dir.clone(),
                source,
            })?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "yaml")
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut handoffs = Vec::new();
        for entry in entries {
            handoffs.push(read_yaml::<Handoff>(&entry.path())?);
        }
        handoffs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(handoffs)
    }
}

#[derive(Debug, Error)]
pub enum HandoffStoreError {
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

    use crate::objects::handoff::{ActorRef, Handoff, HandoffStatus};
    use crate::objects::id::{HandoffId, SeatId, WorkItemId};
    use crate::storage::handoff_store::HandoffStore;

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("seatloom-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    fn make_handoff(id: &HandoffId, created_at: chrono::DateTime<Utc>) -> Handoff {
        Handoff {
            id: id.clone(),
            from_ref: ActorRef::Seat(SeatId::new()),
            to_ref: ActorRef::Seat(SeatId::new()),
            workitem_id: WorkItemId::new(),
            purpose: "deliver review".to_string(),
            expected_outcome: "review complete".to_string(),
            artifact_ids: vec![],
            required_receipt: false,
            status: HandoffStatus::Sent,
            created_at,
            sent_at: Some(created_at),
        }
    }

    #[test]
    fn missing_handoffs_dir_returns_empty_vec() {
        let dir = temp_dir("handoff-missing-dir");
        let store = HandoffStore::new(&dir);
        let handoffs = store.list_handoffs().expect("missing dir should return empty");
        assert!(handoffs.is_empty());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn handoff_round_trip_and_list() {
        let dir = temp_dir("handoff-list-roundtrip");
        let store = HandoffStore::new(&dir);

        let ts1 = Utc.with_ymd_and_hms(2026, 4, 29, 1, 0, 0).single().unwrap();
        let ts2 = ts1 + Duration::hours(3);
        let id1 = HandoffId::new();
        let id2 = HandoffId::new();

        let h1 = make_handoff(&id1, ts1);
        let h2 = make_handoff(&id2, ts2);

        store.save_handoff(&h1).expect("write h1");
        store.save_handoff(&h2).expect("write h2");

        let handoffs = store.list_handoffs().expect("list should succeed");
        assert_eq!(handoffs.len(), 2);
        // Sort: created_at descending — h2 (newer) first
        assert_eq!(handoffs[0].id.as_str(), id2.as_str());
        assert_eq!(handoffs[1].id.as_str(), id1.as_str());

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn resolves_handoff_paths_deterministically() {
        let store = HandoffStore::new(std::path::Path::new("/tmp/seatloom-project"));
        let paths = store.paths();
        let ho_id = HandoffId::new();
        assert_eq!(
            paths.handoff_path(&ho_id),
            PathBuf::from(format!(
                "/tmp/seatloom-project/.seatloom/handoffs/{}.yaml",
                ho_id.as_str()
            ))
        );
    }
}
