use crate::objects::id::{ArtifactId, PipelineId, PipelineRunId, WorkItemId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRun {
    pub id: PipelineRunId,
    pub pipeline_id: PipelineId,
    pub workitem_id: Option<WorkItemId>,
    pub status: PipelineRunStatus,
    pub current_stage: Option<String>,
    pub stage_index: Option<i32>,
    pub trigger: PipelineRunTrigger,
    pub initiated_by: Option<String>,
    pub result_summary: Option<String>,
    pub artifact_ids: Vec<ArtifactId>,
    pub evidence_refs: Vec<String>,
    pub metadata: Option<Value>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineRunStatus {
    Running,
    Completed,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineRunTrigger {
    Manual,
    Handoff,
    Gate,
    Scheduled,
}
