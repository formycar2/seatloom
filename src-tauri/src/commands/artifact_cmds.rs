// Artifact commands — read-only with template/subtype filtering.
//
// 🔴 ARTIFACTS ARE **NOT** PROJECT-SCOPED, AND THIS COMMAND SAYS SO OUT LOUD.
//
// `artifacts` has no `project_id` column (measured on live: its columns are id,
// template, subtype, subtype_valid, system_kind, title, summary, source_session_id,
// source_workitem_id, storage_path, created_at). So there is nothing to filter on.
//
// A derived join was considered and rejected on measurement, not on principle:
//   - 15 artifacts on live
//   - 0 have `source_session_id` populated → 0 resolvable through `sessions`
//   - 4 have `source_workitem_id` populated → 4 resolvable through `workitems`
//   - 11 have NEITHER source → 11 attributable to no project by any derivation
//
// A join would therefore return 4 of 15 rows and silently drop the other 11 —
// which is precisely "looks filtered but isn't": the operator would see a short
// list under one project and a different short list under another, and neither
// would be the whole truth. That is the defect class this round keeps finding, so
// the command refuses instead: it takes no `project_id`, and the DTO carries no
// project field, so no caller can believe it was scoped.
//
// The bulk of what the Artifacts tab renders is NOT this table anyway: the UI
// merges `documents` (which DO have `project_id`) with these 15 rows. The
// document half is project-scoped; this half is not, and the delivery note says
// which is which.
//
// If the operator wants this partitioned, `artifacts` needs a `project_id`
// column. That is a live migration and therefore his act — a script for it sits
// at `infra/postgres/migrations/015_artifacts_project_id.sql` on this branch.

use crate::dto::ArtifactDto;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn cmd_list_artifacts(
    state: State<'_, AppState>,
    template: Option<String>,
    subtype: Option<String>,
) -> Result<Vec<ArtifactDto>, String> {
    state
        .db
        .list_artifacts(template.as_deref(), subtype.as_deref())
        .await
        .map(|rows| rows.into_iter().map(ArtifactDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_get_artifact(
    state: State<'_, AppState>,
    artifact_id: String,
) -> Result<Option<ArtifactDto>, String> {
    state
        .db
        .get_artifact(&artifact_id)
        .await
        .map(|opt| opt.map(ArtifactDto::from))
        .map_err(|e| e.to_string())
}
