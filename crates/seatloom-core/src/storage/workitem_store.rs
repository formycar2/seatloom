use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::objects::id::WorkItemId;
use crate::objects::workitem::WorkItem;
use crate::storage::project::ProjectPaths;
use crate::storage::yaml_io::{read_yaml, write_yaml, YamlIoError};

pub struct WorkItemStore {
    paths: ProjectPaths,
}

impl WorkItemStore {
    pub fn new(root: &Path) -> Self {
        Self {
            paths: ProjectPaths::new(root),
        }
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn load_workitem(&self, workitem_id: &WorkItemId) -> Result<WorkItem, WorkItemStoreError> {
        Ok(read_yaml(&self.paths.workitem_path(workitem_id))?)
    }

    pub fn save_workitem(&self, workitem: &WorkItem) -> Result<(), WorkItemStoreError> {
        write_yaml(&self.paths.workitem_path(&workitem.id), workitem)?;
        Ok(())
    }

    /// List all workitems by scanning `.seatloom/workitems/*.yaml`.
    /// Returns empty Vec when the workitems directory does not exist.
    /// Sort: by `updated_at` descending (most recently updated first, deterministic).
    pub fn list_workitems(&self) -> Result<Vec<WorkItem>, WorkItemStoreError> {
        let workitems_dir = self.paths.workitems_dir();
        if !workitems_dir.is_dir() {
            return Ok(vec![]);
        }
        let mut entries: Vec<_> = fs::read_dir(&workitems_dir)
            .map_err(|source| WorkItemStoreError::Io {
                path: workitems_dir.clone(),
                source,
            })?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "yaml"))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut workitems = Vec::new();
        for entry in entries {
            workitems.push(read_yaml::<WorkItem>(&entry.path())?);
        }
        workitems.sort_by_key(|w| std::cmp::Reverse(w.updated_at));
        Ok(workitems)
    }
}

#[derive(Debug, Error)]
pub enum WorkItemStoreError {
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

    use crate::objects::id::WorkItemId;
    use crate::objects::workitem::{Priority, WorkItem, WorkItemStatus};
    use crate::storage::workitem_store::WorkItemStore;

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("seatloom-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    fn make_workitem(
        id: &WorkItemId,
        title: &str,
        created_at: chrono::DateTime<Utc>,
        updated_at: chrono::DateTime<Utc>,
    ) -> WorkItem {
        WorkItem {
            id: id.clone(),
            title: title.to_string(),
            goal: None,
            acceptance_criteria: vec![],
            owner_seat_id: None,
            status: WorkItemStatus::Ready,
            priority: Priority::Medium,
            depends_on: vec![],
            parent_id: None,
            created_at,
            updated_at,
        }
    }

    #[test]
    fn missing_workitems_dir_returns_empty_vec() {
        let dir = temp_dir("workitem-missing-dir");
        let store = WorkItemStore::new(&dir);
        let items = store
            .list_workitems()
            .expect("missing dir should return empty");
        assert!(items.is_empty());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn workitem_round_trip_and_list() {
        let dir = temp_dir("workitem-list-roundtrip");
        let store = WorkItemStore::new(&dir);

        let ts = Utc.with_ymd_and_hms(2026, 4, 29, 1, 0, 0).single().unwrap();
        let id1 = WorkItemId::new();
        let id2 = WorkItemId::new();

        let w1 = make_workitem(&id1, "first task", ts, ts);
        let w2 = make_workitem(&id2, "second task", ts, ts + Duration::hours(1));

        store.save_workitem(&w1).expect("write w1");
        store.save_workitem(&w2).expect("write w2");

        let items = store.list_workitems().expect("list should succeed");
        assert_eq!(items.len(), 2);
        // Sort: updated_at descending — w2 (newer update) first
        assert_eq!(items[0].id.as_str(), id2.as_str());
        assert_eq!(items[1].id.as_str(), id1.as_str());

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn resolves_workitem_paths_deterministically() {
        let store = WorkItemStore::new(std::path::Path::new("/tmp/seatloom-project"));
        let paths = store.paths();
        let wi_id = WorkItemId::new();
        assert_eq!(
            paths.workitem_path(&wi_id),
            PathBuf::from(format!(
                "/tmp/seatloom-project/.seatloom/workitems/{}.yaml",
                wi_id.as_str()
            ))
        );
    }
}
