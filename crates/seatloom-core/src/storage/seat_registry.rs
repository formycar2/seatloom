use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::objects::id::DelegationId;
use crate::objects::seat::{DelegationStatus, ProjectRoleBind, SeatDelegation, SeatIdentity};
use crate::storage::project::ProjectPaths;
use crate::storage::yaml_io::{read_yaml, write_yaml, YamlIoError};

pub struct SeatRegistry {
    paths: ProjectPaths,
}

impl SeatRegistry {
    pub fn new(root: &Path) -> Self {
        Self {
            paths: ProjectPaths::new(root),
        }
    }

    pub fn paths(&self) -> &ProjectPaths {
        &self.paths
    }

    pub fn save_identity(&self, identity: &SeatIdentity) -> Result<(), SeatRegistryError> {
        write_yaml(&self.paths.seat_identity_path(&identity.name), identity)?;
        Ok(())
    }

    pub fn load_identity(&self, seat_name: &str) -> Result<SeatIdentity, SeatRegistryError> {
        Ok(read_yaml(&self.paths.seat_identity_path(seat_name))?)
    }

    pub fn save_role_binding(
        &self,
        seat_name: &str,
        role_bind: &ProjectRoleBind,
    ) -> Result<(), SeatRegistryError> {
        write_yaml(
            &self
                .paths
                .seat_role_binding_path(seat_name, &role_bind.project_id),
            role_bind,
        )?;
        Ok(())
    }

    pub fn load_role_binding(
        &self,
        seat_name: &str,
        project_id: &str,
    ) -> Result<ProjectRoleBind, SeatRegistryError> {
        Ok(read_yaml(
            &self.paths.seat_role_binding_path(seat_name, project_id),
        )?)
    }

    pub fn save_delegation(
        &self,
        delegation: &SeatDelegation,
    ) -> Result<(), SeatRegistryError> {
        validate_delegation(delegation)?;
        write_yaml(&self.paths.delegation_path(&delegation.id), delegation)?;
        Ok(())
    }

    pub fn load_delegation(
        &self,
        delegation_id: &DelegationId,
    ) -> Result<SeatDelegation, SeatRegistryError> {
        Ok(read_yaml(&self.paths.delegation_path(delegation_id))?)
    }

    /// List all seat identities by scanning `.seatloom/seats/*/identity.yaml`.
    /// Returns empty Vec when the seats directory does not exist.
    /// Sort: by `name` ascending (deterministic).
    pub fn list_seat_identities(&self) -> Result<Vec<SeatIdentity>, SeatRegistryError> {
        let seats_dir = self.paths.seats_dir();
        if !seats_dir.is_dir() {
            return Ok(vec![]);
        }
        let mut entries: Vec<_> = fs::read_dir(&seats_dir)
            .map_err(|source| SeatRegistryError::Io {
                path: seats_dir.clone(),
                source,
            })?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut identities = Vec::new();
        for entry in entries {
            let identity_path = entry.path().join("identity.yaml");
            if identity_path.is_file() {
                identities.push(read_yaml::<SeatIdentity>(&identity_path)?);
            }
        }
        identities.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(identities)
    }

    /// List all delegations by scanning `.seatloom/delegations/*.yaml`.
    /// Returns empty Vec when the delegations directory does not exist.
    /// Sort: by `issued_at` descending (most recent first, deterministic).
    pub fn list_delegations(&self) -> Result<Vec<SeatDelegation>, SeatRegistryError> {
        let delegations_dir = self.paths.delegations_dir();
        if !delegations_dir.is_dir() {
            return Ok(vec![]);
        }
        let mut entries: Vec<_> = fs::read_dir(&delegations_dir)
            .map_err(|source| SeatRegistryError::Io {
                path: delegations_dir.clone(),
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

        let mut delegations = Vec::new();
        for entry in entries {
            delegations.push(read_yaml::<SeatDelegation>(&entry.path())?);
        }
        delegations.sort_by(|a, b| b.issued_at.cmp(&a.issued_at));
        Ok(delegations)
    }
}

#[derive(Debug, Error)]
pub enum SeatRegistryError {
    #[error(transparent)]
    Yaml(#[from] YamlIoError),
    #[error(transparent)]
    Validation(#[from] SeatRegistryValidationError),
    #[error("failed to read directory {}: {source}", path.display())]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SeatRegistryValidationError {
    #[error("delegation {delegation_id} cannot be saved without scope")]
    MissingScope { delegation_id: String },
    #[error("delegation {delegation_id} cannot be saved without issuer")]
    MissingIssuer { delegation_id: String },
    #[error("delegation {delegation_id} cannot be saved without target seat")]
    MissingTargetSeat { delegation_id: String },
    #[error("active delegation {delegation_id} cannot be saved without expiry")]
    ActiveDelegationMissingExpiry { delegation_id: String },
}

pub fn validate_delegation(
    delegation: &SeatDelegation,
) -> Result<(), SeatRegistryValidationError> {
    let delegation_id = delegation.id.as_str().to_string();

    if delegation.scope_description.trim().is_empty() {
        return Err(SeatRegistryValidationError::MissingScope { delegation_id });
    }

    if delegation.issuer_seat_id.as_str().trim().is_empty() {
        return Err(SeatRegistryValidationError::MissingIssuer { delegation_id });
    }

    if delegation.to_seat_id.as_str().trim().is_empty() {
        return Err(SeatRegistryValidationError::MissingTargetSeat { delegation_id });
    }

    if matches!(delegation.status, DelegationStatus::Active) && delegation.expires_at.is_none() {
        return Err(SeatRegistryValidationError::ActiveDelegationMissingExpiry {
            delegation_id,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::{Duration, TimeZone, Utc};

    use crate::objects::id::{DelegationId, SeatId, WorkItemId};
    use crate::objects::seat::{
        DelegationStatus, ProjectRoleBind, SeatDelegation, SeatIdentity, SeatRole, SeatStatus,
    };
    use crate::objects::session::Runtime;
    use crate::storage::seat_registry::{
        validate_delegation, SeatRegistry, SeatRegistryError, SeatRegistryValidationError,
    };

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("seatloom-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    fn timestamp() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 29, 2, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn empty_seat_id() -> SeatId {
        serde_yaml::from_str("''").expect("empty string should deserialize into SeatId")
    }

    #[test]
    fn resolves_seat_layer_paths_deterministically() {
        let registry = SeatRegistry::new(Path::new("/tmp/seatloom-project"));
        let paths = registry.paths();

        assert_eq!(
            paths.seat_identity_path("nimbus"),
            PathBuf::from("/tmp/seatloom-project/.seatloom/seats/nimbus/identity.yaml")
        );
        assert_eq!(
            paths.seat_role_binding_path("nimbus", "seatloom"),
            PathBuf::from(
                "/tmp/seatloom-project/.seatloom/seats/nimbus/role-bindings/seatloom.yaml"
            )
        );
        let delegation_id = DelegationId::new();
        assert_eq!(
            paths.delegation_path(&delegation_id),
            PathBuf::from(format!(
                "/tmp/seatloom-project/.seatloom/delegations/{}.yaml",
                delegation_id.as_str()
            ))
        );
    }

    #[test]
    fn seat_identity_round_trips() {
        let dir = temp_dir("seat-identity-roundtrip");
        let registry = SeatRegistry::new(&dir);
        let identity = SeatIdentity {
            id: SeatId::new(),
            name: "nimbus".to_string(),
            default_runtime: Some(Runtime::Codex),
            capability_tags: vec!["architecture".to_string(), "rust".to_string()],
            status: SeatStatus::Active,
            created_at: timestamp(),
        };

        registry.save_identity(&identity).expect("identity should save");
        let loaded = registry
            .load_identity("nimbus")
            .expect("identity should load");

        assert_eq!(loaded.id.as_str(), identity.id.as_str());
        assert_eq!(loaded.name, "nimbus");
        assert_eq!(loaded.capability_tags, identity.capability_tags);
        assert!(matches!(loaded.default_runtime, Some(Runtime::Codex)));
        assert!(matches!(loaded.status, SeatStatus::Active));

        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }

    #[test]
    fn role_binding_round_trips() {
        let dir = temp_dir("role-binding-roundtrip");
        let registry = SeatRegistry::new(&dir);
        let seat_id = SeatId::new();
        let binding = ProjectRoleBind {
            seat_id: seat_id.clone(),
            project_id: "seatloom".to_string(),
            role: SeatRole::Architect,
            authority_doc_refs: vec!["docs/architecture-design.md".to_string()],
            constraints: vec!["no-runtime-widening".to_string()],
            collaboration_template_ref: Some("baseline-v1".to_string()),
            active_delegation_id: None,
        };

        registry
            .save_role_binding("nimbus", &binding)
            .expect("role binding should save");
        let loaded = registry
            .load_role_binding("nimbus", "seatloom")
            .expect("role binding should load");

        assert_eq!(loaded.seat_id.as_str(), seat_id.as_str());
        assert_eq!(loaded.project_id, "seatloom");
        assert!(matches!(loaded.role, SeatRole::Architect));
        assert_eq!(loaded.authority_doc_refs.len(), 1);
        assert_eq!(loaded.constraints, vec!["no-runtime-widening"]);
        assert_eq!(loaded.collaboration_template_ref.as_deref(), Some("baseline-v1"));

        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }

    #[test]
    fn delegation_round_trips() {
        let dir = temp_dir("delegation-roundtrip");
        let registry = SeatRegistry::new(&dir);
        let delegation = SeatDelegation {
            id: DelegationId::new(),
            issuer_seat_id: SeatId::new(),
            from_seat_id: SeatId::new(),
            to_seat_id: SeatId::new(),
            workitem_id: Some(WorkItemId::new()),
            scope_description: "review architecture delta".to_string(),
            issued_at: timestamp(),
            expires_at: Some(timestamp() + Duration::hours(8)),
            status: DelegationStatus::Active,
        };

        registry
            .save_delegation(&delegation)
            .expect("delegation should save");
        let loaded = registry
            .load_delegation(&delegation.id)
            .expect("delegation should load");

        assert_eq!(loaded.id.as_str(), delegation.id.as_str());
        assert_eq!(loaded.scope_description, "review architecture delta");
        assert_eq!(loaded.to_seat_id.as_str(), delegation.to_seat_id.as_str());
        assert!(matches!(loaded.status, DelegationStatus::Active));
        assert!(loaded.expires_at.is_some());

        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }

    #[test]
    fn delegation_validation_rejects_required_boundary_gaps() {
        let base = SeatDelegation {
            id: DelegationId::new(),
            issuer_seat_id: SeatId::new(),
            from_seat_id: SeatId::new(),
            to_seat_id: SeatId::new(),
            workitem_id: None,
            scope_description: "scoped delegation".to_string(),
            issued_at: timestamp(),
            expires_at: Some(timestamp() + Duration::hours(2)),
            status: DelegationStatus::Active,
        };

        let missing_scope = SeatDelegation {
            scope_description: "   ".to_string(),
            ..base.clone()
        };
        assert!(matches!(
            validate_delegation(&missing_scope),
            Err(SeatRegistryValidationError::MissingScope { .. })
        ));

        let missing_issuer = SeatDelegation {
            issuer_seat_id: empty_seat_id(),
            ..base.clone()
        };
        assert!(matches!(
            validate_delegation(&missing_issuer),
            Err(SeatRegistryValidationError::MissingIssuer { .. })
        ));

        let missing_target = SeatDelegation {
            to_seat_id: empty_seat_id(),
            ..base.clone()
        };
        assert!(matches!(
            validate_delegation(&missing_target),
            Err(SeatRegistryValidationError::MissingTargetSeat { .. })
        ));

        let missing_expiry = SeatDelegation {
            expires_at: None,
            ..base
        };
        assert!(matches!(
            validate_delegation(&missing_expiry),
            Err(SeatRegistryValidationError::ActiveDelegationMissingExpiry { .. })
        ));
    }

    #[test]
    fn save_delegation_returns_typed_validation_error() {
        let dir = temp_dir("delegation-typed-error");
        let registry = SeatRegistry::new(&dir);
        let invalid = SeatDelegation {
            id: DelegationId::new(),
            issuer_seat_id: SeatId::new(),
            from_seat_id: SeatId::new(),
            to_seat_id: SeatId::new(),
            workitem_id: None,
            scope_description: " ".to_string(),
            issued_at: timestamp(),
            expires_at: Some(timestamp() + Duration::hours(1)),
            status: DelegationStatus::Active,
        };

        let error = registry
            .save_delegation(&invalid)
            .expect_err("invalid delegation should fail");

        assert!(matches!(
            error,
            SeatRegistryError::Validation(SeatRegistryValidationError::MissingScope { .. })
        ));

        fs::remove_dir_all(dir).expect("temp dir should be removed");
    }

    #[test]
    fn missing_seats_dir_returns_empty_vec() {
        let dir = temp_dir("seats-missing-dir");
        let registry = SeatRegistry::new(&dir);
        let seats = registry
            .list_seat_identities()
            .expect("missing dir should return empty");
        assert!(seats.is_empty());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn list_seat_identities_sorted_by_name() {
        let dir = temp_dir("seats-list-sorted");
        let registry = SeatRegistry::new(&dir);

        let mira = SeatIdentity {
            id: SeatId::new(),
            name: "mira".to_string(),
            default_runtime: None,
            capability_tags: vec![],
            status: SeatStatus::Active,
            created_at: timestamp(),
        };
        let flux = SeatIdentity {
            id: SeatId::new(),
            name: "flux".to_string(),
            default_runtime: None,
            capability_tags: vec![],
            status: SeatStatus::Active,
            created_at: timestamp(),
        };

        registry.save_identity(&mira).expect("save mira");
        registry.save_identity(&flux).expect("save flux");

        let seats = registry
            .list_seat_identities()
            .expect("list should succeed");
        assert_eq!(seats.len(), 2);
        // Sort: name ascending — flux before mira
        assert_eq!(seats[0].name, "flux");
        assert_eq!(seats[1].name, "mira");

        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn missing_delegations_dir_returns_empty_vec() {
        let dir = temp_dir("delegations-missing-dir");
        let registry = SeatRegistry::new(&dir);
        let delegations = registry
            .list_delegations()
            .expect("missing dir should return empty");
        assert!(delegations.is_empty());
        fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn list_delegations_sorted_by_issued_at_descending() {
        let dir = temp_dir("delegations-list-sorted");
        let registry = SeatRegistry::new(&dir);

        let d1 = SeatDelegation {
            id: DelegationId::new(),
            issuer_seat_id: SeatId::new(),
            from_seat_id: SeatId::new(),
            to_seat_id: SeatId::new(),
            workitem_id: None,
            scope_description: "first delegation".to_string(),
            issued_at: timestamp(),
            expires_at: Some(timestamp() + Duration::hours(4)),
            status: DelegationStatus::Active,
        };
        let d2 = SeatDelegation {
            id: DelegationId::new(),
            issuer_seat_id: SeatId::new(),
            from_seat_id: SeatId::new(),
            to_seat_id: SeatId::new(),
            workitem_id: None,
            scope_description: "second delegation".to_string(),
            issued_at: timestamp() + Duration::hours(2),
            expires_at: Some(timestamp() + Duration::hours(6)),
            status: DelegationStatus::Active,
        };

        registry.save_delegation(&d1).expect("save d1");
        registry.save_delegation(&d2).expect("save d2");

        let delegations = registry
            .list_delegations()
            .expect("list should succeed");
        assert_eq!(delegations.len(), 2);
        // Sort: issued_at descending — d2 (newer) first
        assert_eq!(delegations[0].id.as_str(), d2.id.as_str());
        assert_eq!(delegations[1].id.as_str(), d1.id.as_str());

        fs::remove_dir_all(dir).expect("cleanup");
    }
}
