use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::objects::id::{DelegationId, HandoffId, SessionId, WorkItemId};

/// Parsed from .seatloom/config/project.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub version: String,
    pub project_name: String,
    pub created_at: DateTime<Utc>,
    pub pack_engine: PackEngineConfig,
    pub pipeline: PipelineConfig,
}

impl ProjectConfig {
    pub fn new(project_name: impl Into<String>, created_at: DateTime<Utc>) -> Self {
        Self {
            version: "0.1".to_string(),
            project_name: project_name.into(),
            created_at,
            pack_engine: PackEngineConfig::default(),
            pipeline: PipelineConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackEngineConfig {
    /// P0 Worker continuity pack token budget.
    pub worker_budget_tokens: u32,
    /// P1 Supervisor continuity pack token budget.
    pub supervisor_budget_tokens: u32,
}

impl Default for PackEngineConfig {
    fn default() -> Self {
        Self {
            worker_budget_tokens: 8_192,
            supervisor_budget_tokens: 32_768,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub stage_timeout_seconds: u32,
    pub total_timeout_seconds: u32,
    pub max_retry: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            stage_timeout_seconds: 300,
            total_timeout_seconds: 900,
            max_retry: 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectPaths {
    root: PathBuf,
}

impl ProjectPaths {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn seatloom_dir(&self) -> PathBuf {
        self.root.join(".seatloom")
    }

    pub fn config_dir(&self) -> PathBuf {
        self.seatloom_dir().join("config")
    }

    pub fn project_config_path(&self) -> PathBuf {
        self.config_dir().join("project.yaml")
    }

    pub fn ledger_dir(&self) -> PathBuf {
        self.seatloom_dir().join("ledger")
    }

    pub fn ledger_events_path(&self) -> PathBuf {
        self.ledger_dir().join("events.jsonl")
    }

    pub fn seats_dir(&self) -> PathBuf {
        self.seatloom_dir().join("seats")
    }

    pub fn seat_dir(&self, seat_name: &str) -> PathBuf {
        self.seats_dir().join(seat_name)
    }

    pub fn seat_identity_path(&self, seat_name: &str) -> PathBuf {
        self.seat_dir(seat_name).join("identity.yaml")
    }

    pub fn seat_role_bindings_dir(&self, seat_name: &str) -> PathBuf {
        self.seat_dir(seat_name).join("role-bindings")
    }

    pub fn seat_role_binding_path(&self, seat_name: &str, project_id: &str) -> PathBuf {
        self.seat_role_bindings_dir(seat_name)
            .join(format!("{project_id}.yaml"))
    }

    pub fn delegations_dir(&self) -> PathBuf {
        self.seatloom_dir().join("delegations")
    }

    pub fn delegation_path(&self, delegation_id: &DelegationId) -> PathBuf {
        self.delegations_dir()
            .join(format!("{}.yaml", delegation_id.as_str()))
    }

    // --- Session paths ---

    pub fn sessions_dir(&self) -> PathBuf {
        self.seatloom_dir().join("sessions")
    }

    pub fn session_dir(&self, session_id: &SessionId) -> PathBuf {
        self.sessions_dir().join(session_id.as_str())
    }

    pub fn session_meta_path(&self, session_id: &SessionId) -> PathBuf {
        self.session_dir(session_id).join("meta.yaml")
    }

    // --- WorkItem paths ---

    pub fn workitems_dir(&self) -> PathBuf {
        self.seatloom_dir().join("workitems")
    }

    pub fn workitem_path(&self, workitem_id: &WorkItemId) -> PathBuf {
        self.workitems_dir()
            .join(format!("{}.yaml", workitem_id.as_str()))
    }

    // --- Handoff paths ---

    pub fn handoffs_dir(&self) -> PathBuf {
        self.seatloom_dir().join("handoffs")
    }

    pub fn handoff_path(&self, handoff_id: &HandoffId) -> PathBuf {
        self.handoffs_dir()
            .join(format!("{}.yaml", handoff_id.as_str()))
    }

    // --- Artifact paths ---

    pub fn artifacts_dir(&self) -> PathBuf {
        self.seatloom_dir().join("artifacts")
    }
}
