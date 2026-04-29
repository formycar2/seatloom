use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::objects::id::{PipelineId, PipelineRunId, WorkItemId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRun {
    pub id: PipelineRunId,
    pub pipeline_id: PipelineId,
    pub workitem_id: Option<WorkItemId>,
    pub status: PipelineRunStatus,
    pub current_stage: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineRunStatus {
    Running, Completed, Failed, Aborted,
}
