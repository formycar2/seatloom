// Prompt commands — expose schema 005 prompt_instances / prompt_actions.

use crate::state::AppState;
use seatloom_core::db::models::{PromptActionRow, PromptInstanceRow};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptInstanceDto {
    pub id: String,
    pub project_id: String,
    pub session_id: String,
    pub status: String,
    pub prompt_kind: String,
    pub prompt_policy: String,
    pub evidence_preview: Option<String>,
    pub available_actions: Vec<String>,
    pub assist_max_steps: Option<i32>,
    pub assist_max_tokens: Option<i32>,
    pub assist_steps_used: i32,
    pub assist_tokens_used: i32,
    pub detected_at: String,
    pub resolved_at: Option<String>,
}

impl From<PromptInstanceRow> for PromptInstanceDto {
    fn from(r: PromptInstanceRow) -> Self {
        Self {
            id: r.id,
            project_id: r.project_id,
            session_id: r.session_id,
            status: r.status,
            prompt_kind: r.prompt_kind,
            prompt_policy: r.prompt_policy,
            evidence_preview: r.evidence_preview,
            available_actions: r.available_actions,
            assist_max_steps: r.assist_max_steps,
            assist_max_tokens: r.assist_max_tokens,
            assist_steps_used: r.assist_steps_used,
            assist_tokens_used: r.assist_tokens_used,
            detected_at: r.detected_at.to_rfc3339(),
            resolved_at: r.resolved_at.map(|t| t.to_rfc3339()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptActionDto {
    pub id: String,
    pub prompt_id: String,
    pub action_kind: String,
    pub actor_ref: String,
    pub source_channel: String,
    pub note: Option<String>,
    pub steps_budget_used: Option<i32>,
    pub tokens_budget_used: Option<i32>,
    pub result_status: String,
    pub created_at: String,
}

impl From<PromptActionRow> for PromptActionDto {
    fn from(r: PromptActionRow) -> Self {
        Self {
            id: r.id,
            prompt_id: r.prompt_id,
            action_kind: r.action_kind,
            actor_ref: r.actor_ref,
            source_channel: r.source_channel,
            note: r.note,
            steps_budget_used: r.steps_budget_used,
            tokens_budget_used: r.tokens_budget_used,
            result_status: r.result_status,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[tauri::command]
pub async fn cmd_list_active_prompts(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<PromptInstanceDto>, String> {
    let pid = project_id.unwrap_or_else(|| state.default_project_id.clone());
    state
        .db
        .list_active_prompt_instances(&pid)
        .await
        .map(|rows| rows.into_iter().map(PromptInstanceDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_prompts_for_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<PromptInstanceDto>, String> {
    state
        .db
        .list_prompt_instances_for_session(&session_id)
        .await
        .map(|rows| rows.into_iter().map(PromptInstanceDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_list_prompt_actions(
    state: State<'_, AppState>,
    prompt_id: String,
) -> Result<Vec<PromptActionDto>, String> {
    state
        .db
        .list_prompt_actions_for_prompt(&prompt_id)
        .await
        .map(|rows| rows.into_iter().map(PromptActionDto::from).collect())
        .map_err(|e| e.to_string())
}
