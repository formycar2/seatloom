// Operator decisions (020) — the operator's read and resolve path.
//
// The 014 split, applied to the human-obligation object: the MCP side
// (`request_operator_decision` / `resolve_operator_decision` in
// `crates/seatloom-mcp-emit/src/decisions.rs`) is how an obligation is CREATED
// and how Aegis resolves the rows it made for itself. These commands are how
// the OPERATOR sees and rules on what is waiting for HIM — the desktop app
// acting as the user's agent, exactly as `cmd_accept_delivery` is the operator's
// half of the acceptance gate.
//
// WHY THERE IS NO OPERATOR MCP SERVER. There is no `human:*` process to launch
// one per seat, and inventing a fake seat for the operator would put a seat id
// where a human identity belongs. The app IS the operator's agent: it resolves
// the operator ref from the same `~/.seatloom/operator` file (or the env var)
// that `cmd_accept_delivery` uses, with the same precedence and the same
// refusal when it is unconfigured.
//
// The resolve path is GUARDED, not trusted: the app resolves the operator ref
// and the repository refuses a ruling from anyone who is not the row's decider.
// A row Aegis created for itself (decider = `seat:seat-aegis-001`) therefore
// cannot be resolved from the operator's click — which is the point of the
// decider field.

use crate::dto::{OperatorDecisionCardDto, OperatorDecisionResolveResultDto};
use crate::state::AppState;
use tauri::State;

/// The operator decisions waiting on the operator, oldest first.
///
/// The decider is ALWAYS the resolved operator ref — never a parameter. A
/// caller that could pass a decider could read (and, with the resolve command,
/// appear to rule on) obligations belonging to someone else. This is the same
/// non-forgeable-identity rule the MCP tools follow.
#[tauri::command]
pub async fn cmd_list_operator_decisions(
    state: State<'_, AppState>,
    project_id: Option<String>,
    status: Option<String>,
) -> Result<Vec<OperatorDecisionCardDto>, String> {
    let operator = operator_ref()?;
    let project = crate::commands::require_project_id("cmd_list_operator_decisions", project_id)?;
    let status = match status.as_deref() {
        None | Some("pending") => "pending",
        Some("resolved") => "resolved",
        Some(other) => {
            return Err(format!(
                "status {other:?} is not one of pending | resolved — an unknown status is never defaulted"
            ));
        }
    };
    let rows = if status == "pending" {
        state
            .db
            .list_pending_operator_decisions(&project, Some(&operator), 500)
            .await
    } else {
        state
            .db
            .list_resolved_operator_decisions(&project, Some(&operator), 500)
            .await
    }
    .map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(OperatorDecisionCardDto::from)
        .collect())
}

/// How many decisions are waiting on the operator — the number the object exists
/// to answer. A separate command because it is a different question from the
/// list: the UI shows the count without fetching every card.
#[tauri::command]
pub async fn cmd_count_pending_operator_decisions(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<i64, String> {
    let operator = operator_ref()?;
    let project =
        crate::commands::require_project_id("cmd_count_pending_operator_decisions", project_id)?;
    let rows = state
        .db
        .list_pending_operator_decisions(&project, Some(&operator), 10_000)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.len() as i64)
}

/// Resolve an operator decision: the ruling lands + `OperatorDecisionResolved`
/// in ONE transaction.
///
/// Refuses — writing nothing — when the operator ref is unconfigured, the
/// obligation names no row, the operator is not its decider (a row Aegis made
/// for itself cannot be ruled on from here), it is already resolved (terminal
/// and immutable), or `chosen` is not one of the row's own choices.
#[tauri::command]
pub async fn cmd_resolve_operator_decision(
    state: State<'_, AppState>,
    decision_id: String,
    chosen: String,
) -> Result<OperatorDecisionResolveResultDto, String> {
    let operator = operator_ref()?;
    let Some(row) = state
        .db
        .get_operator_decision(&decision_id)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Err(format!(
            "operator decision {decision_id:?} names no obligation — a read of a mistyped id must \
             say so, not render an empty card"
        ));
    };
    if row.decider != operator {
        return Err(format!(
            "operator decision {decision_id:?} is for {decider:?} to rule on, not {operator:?} — \
             a ruling from anyone else is not a ruling. (A row Aegis created for itself is \
             resolved by Aegis, not from this click.)",
            decider = row.decider
        ));
    }
    if row.status == "resolved" {
        return Err(format!(
            "operator decision {decision_id:?} is already resolved — a recorded ruling is \
             immutable, exactly like a decided acceptance. A new question is a new decision"
        ));
    }
    let resolved = state
        .db
        .resolve_operator_decision(&decision_id, &chosen, &operator)
        .await
        .map_err(|e| e.to_string())?;
    Ok(OperatorDecisionResolveResultDto {
        decision_id: resolved.id,
        status: resolved.status,
        chosen: resolved.chosen,
        resolved_by: resolved.resolved_by,
        target: resolved.target,
    })
}

/// The operator identity, resolved through the same two channels and the same
/// precedence as `cmd_accept_delivery` — one home for the rule, in
/// [`seatloom_core::config`]. No default and no guess: when neither channel
/// states an identity, this refuses with a message that names the file and its
/// exact path.
fn operator_ref() -> Result<String, String> {
    seatloom_core::config::read_operator_ref()
        .map(|r| r.value)
        .map_err(|e| seatloom_core::config::operator_ref_refusal_message(&e))
}
