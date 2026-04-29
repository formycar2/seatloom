use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::objects::artifact::{Artifact, ArtifactTemplate};
use crate::objects::id::ArtifactId;
use crate::storage::project::ProjectPaths;
use crate::storage::yaml_io::{read_yaml, write_yaml, YamlIoError};

/// Subtype allow-list from DOCUMENT_TEMPLATES.md §11.1.
const T1_SUBTYPES: &[&str] = &[
    "prd",
    "ux_spec",
    "interaction_spec",
    "acceptance_spec",
    "architecture_design",
    "architecture_decisions",
];
const T2_SUBTYPES: &[&str] = &["seat_role"];
const T3_SUBTYPES: &[&str] = &["task", "fix", "integration", "verification"];
const T4_SUBTYPES: &[&str] = &[
    "gap_review",
    "benchmark",
    "process_mapping",
    "design_proposal",
];
const T5_SUBTYPES: &[&str] = &["acceptance_review", "gate_decision"];
const T6_SUBTYPES: &[&str] = &["daily_log"];
const T7_SUBTYPES: &[&str] = &[
    "coordination_rules",
    "workflow_principles",
    "collaboration_protocol",
    "document_templates",
];

pub struct ArtifactStore {
    paths: ProjectPaths,
}

impl ArtifactStore {
    pub fn new(root: &Path) -> Self {
        Self {
            paths: ProjectPaths::new(root),
        }
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn load_artifact(&self, artifact_id: &ArtifactId) -> Result<Artifact, ArtifactStoreError> {
        Ok(read_yaml(&self.paths.artifact_meta_path(artifact_id))?)
    }

    pub fn save_artifact(&self, artifact: &Artifact) -> Result<(), ArtifactStoreError> {
        write_yaml(&self.paths.artifact_meta_path(&artifact.id), artifact)?;
        Ok(())
    }

    /// List all artifacts by scanning `.seatloom/artifacts/*/meta.yaml`.
    /// Returns empty Vec when the artifacts directory does not exist.
    /// Sort: by `created_at` descending (most recent first, deterministic).
    ///
    /// Supports optional filtering by `template`, `subtype`, and `source_workitem_id`.
    pub fn list_artifacts(
        &self,
        template_filter: Option<&ArtifactTemplate>,
        subtype_filter: Option<&str>,
        workitem_id_filter: Option<&str>,
    ) -> Result<Vec<Artifact>, ArtifactStoreError> {
        let artifacts_dir = self.paths.artifacts_dir();
        if !artifacts_dir.is_dir() {
            return Ok(vec![]);
        }
        let mut entries: Vec<_> = fs::read_dir(&artifacts_dir)
            .map_err(|source| ArtifactStoreError::Io {
                path: artifacts_dir.clone(),
                source,
            })?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut artifacts = Vec::new();
        for entry in entries {
            let meta_path = entry.path().join("meta.yaml");
            if meta_path.is_file() {
                let artifact: Artifact = read_yaml(&meta_path)?;

                if let Some(tf) = template_filter {
                    match (&artifact.template, tf) {
                        (Some(a), b) if std::mem::discriminant(a) == std::mem::discriminant(b) => {}
                        _ => continue,
                    }
                }

                if let Some(sf) = subtype_filter {
                    match &artifact.subtype {
                        Some(s) if s == sf => {}
                        _ => continue,
                    }
                }

                if let Some(wf) = workitem_id_filter {
                    match &artifact.source_workitem_id {
                        Some(wi) if wi.as_str() == wf => {}
                        _ => continue,
                    }
                }

                artifacts.push(artifact);
            }
        }
        artifacts.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        Ok(artifacts)
    }
}

/// Validate a subtype against the DOCUMENT_TEMPLATES.md §11.1 allow-list.
/// Returns `true` if the subtype is valid for the given template.
pub fn validate_subtype(template: &ArtifactTemplate, subtype: &str) -> bool {
    let allowed = match template {
        ArtifactTemplate::T1AuthorityDoc => T1_SUBTYPES,
        ArtifactTemplate::T2RoleProfile => T2_SUBTYPES,
        ArtifactTemplate::T3TaskPacket => T3_SUBTYPES,
        ArtifactTemplate::T4Review => T4_SUBTYPES,
        ArtifactTemplate::T5Acceptance => T5_SUBTYPES,
        ArtifactTemplate::T6DailyMemory => T6_SUBTYPES,
        ArtifactTemplate::T7GovernanceDoc => T7_SUBTYPES,
    };
    allowed.contains(&subtype)
}

#[derive(Debug, Error)]
pub enum ArtifactStoreError {
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

    use crate::objects::artifact::{Artifact, ArtifactTemplate};
    use crate::objects::id::ArtifactId;
    use crate::storage::artifact_store::{validate_subtype, ArtifactStore};

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("seatloom-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    fn make_artifact(
        id: &ArtifactId,
        template: ArtifactTemplate,
        subtype: &str,
        created_at: chrono::DateTime<Utc>,
    ) -> Artifact {
        Artifact {
            id: id.clone(),
            template: Some(template),
            subtype: Some(subtype.to_string()),
            subtype_valid: Some(true),
            system_kind: None,
            title: format!("Test artifact {}", id.as_str()),
            summary: None,
            source_session_id: None,
            source_workitem_id: None,
            storage_path: format!("{}/meta.yaml", id.as_str()),
            created_at,
        }
    }

    #[test]
    fn missing_artifacts_dir_returns_empty_vec() {
        let dir = temp_dir("artifact-missing-dir");
        let store = ArtifactStore::new(&dir);
        let artifacts = store
            .list_artifacts(None, None, None)
            .expect("missing dir should return empty");
        assert!(artifacts.is_empty());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn artifact_round_trip_and_list() {
        let dir = temp_dir("artifact-list-roundtrip");
        let store = ArtifactStore::new(&dir);

        let ts1 = Utc.with_ymd_and_hms(2026, 4, 29, 1, 0, 0).single().unwrap();
        let ts2 = ts1 + Duration::hours(2);
        let id1 = ArtifactId::new();
        let id2 = ArtifactId::new();

        let a1 = make_artifact(&id1, ArtifactTemplate::T1AuthorityDoc, "prd", ts1);
        let a2 = make_artifact(&id2, ArtifactTemplate::T3TaskPacket, "task", ts2);

        store.save_artifact(&a1).expect("write a1");
        store.save_artifact(&a2).expect("write a2");

        let all = store.list_artifacts(None, None, None).expect("list all");
        assert_eq!(all.len(), 2);
        // Sort: created_at descending — a2 (newer) first
        assert_eq!(all[0].id.as_str(), id2.as_str());
        assert_eq!(all[1].id.as_str(), id1.as_str());

        // Filter by template
        let t1_only = store
            .list_artifacts(Some(&ArtifactTemplate::T1AuthorityDoc), None, None)
            .expect("filter T1");
        assert_eq!(t1_only.len(), 1);
        assert_eq!(t1_only[0].id.as_str(), id1.as_str());

        // Filter by subtype
        let tasks = store
            .list_artifacts(None, Some("task"), None)
            .expect("filter task");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id.as_str(), id2.as_str());

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn subtype_validation_follows_allowlist() {
        assert!(validate_subtype(&ArtifactTemplate::T1AuthorityDoc, "prd"));
        assert!(validate_subtype(
            &ArtifactTemplate::T1AuthorityDoc,
            "ux_spec"
        ));
        assert!(!validate_subtype(&ArtifactTemplate::T1AuthorityDoc, "task"));
        assert!(validate_subtype(&ArtifactTemplate::T3TaskPacket, "task"));
        assert!(validate_subtype(&ArtifactTemplate::T3TaskPacket, "fix"));
        assert!(!validate_subtype(&ArtifactTemplate::T3TaskPacket, "prd"));
        assert!(validate_subtype(
            &ArtifactTemplate::T7GovernanceDoc,
            "coordination_rules"
        ));
        assert!(!validate_subtype(&ArtifactTemplate::T7GovernanceDoc, "prd"));
    }

    #[test]
    fn resolves_artifact_paths_deterministically() {
        let store = ArtifactStore::new(std::path::Path::new("/tmp/seatloom-project"));
        let paths = store.paths();
        let ar_id = ArtifactId::new();
        assert_eq!(
            paths.artifact_meta_path(&ar_id),
            PathBuf::from(format!(
                "/tmp/seatloom-project/.seatloom/artifacts/{}/meta.yaml",
                ar_id.as_str()
            ))
        );
    }
}
