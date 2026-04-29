// Dual-key artifact typing model: AD-008.
// template+subtype drives Detail Pane layout, Route/Gate automation, and retrieval filters.
// Missing/invalid template → generic markdown reader + visible warning in UI.

use crate::objects::id::{ArtifactId, SessionId, WorkItemId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    /// T1–T7 coordination document family. None for system-generated artifacts.
    pub template: Option<ArtifactTemplate>,
    /// Subtype from DOCUMENT_TEMPLATES.md §11.1 allow-list.
    pub subtype: Option<String>,
    /// None = not yet validated; false = unknown subtype, degraded-mode rendering.
    pub subtype_valid: Option<bool>,
    /// Set for system-generated artifacts that carry no coordination structure.
    pub system_kind: Option<SystemArtifactKind>,
    pub title: String,
    pub summary: Option<String>,
    pub source_session_id: Option<SessionId>,
    pub source_workitem_id: Option<WorkItemId>,
    /// Path relative to .seatloom/artifacts/
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

/// Template family matching DOCUMENT_TEMPLATES.md T1–T7 taxonomy (AD-008).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactTemplate {
    T1AuthorityDoc,
    T2RoleProfile,
    T3TaskPacket,
    T4Review,
    T5Acceptance,
    T6DailyMemory,
    T7GovernanceDoc,
}

/// System-generated artifacts with no coordination document structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemArtifactKind {
    DiffSummary,
    TestReport,
    BugReport,
    CheckpointSummary,
    WorkerContinuityPack,
    /// P1: assembled SupervisorPack file.
    SupervisorContinuityPack,
}
