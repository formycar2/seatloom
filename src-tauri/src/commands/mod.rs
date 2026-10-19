pub mod acceptance_cmds;
pub mod artifact_cmds;
pub mod delegation_cmds;
pub mod dispatch_cmds;
pub mod document_cmds;
pub mod handoff_cmds;
pub mod history_cmds;
pub mod inbox_cmds;
pub mod knowledge_cmds;
pub mod project_cmds;
pub mod prompt_cmds;
pub mod reconcile_cmds;
pub mod seat_cmds;
pub mod session_cmds;
pub mod supervisor_cmds;
pub mod timeline_cmds;
pub mod workitem_cmds;

/// Every project-scoped read demands its `project_id`.
///
/// **The defect this exists to stop.** `cmd_list_dispatches` used to take
/// `project_id: Option<String>` and fall back to the app's default project when
/// it was absent. The UI never passed it, so the command silently returned the
/// DEFAULT project's rows for whichever project the operator had selected — the
/// operator saw a populated dispatch list under `ql_delivery` that was actually
/// `seatloom`'s. Data that looked real and belonged to another project. That is
/// the shape of every defect in this round: **a check that passes while the thing
/// it stands for is absent.**
///
/// **Why an error rather than an empty list.** An empty list cannot be told apart
/// from "this project genuinely has no rows", which is a real state — the
/// historical-corpus projects drive no seat and correctly return zero. So
/// returning empty for a missing parameter would collide with a legitimate answer
/// and still read as data. An error cannot be mistaken for data, and it surfaces
/// in the caller's `catch` where the surface can say what is missing.
///
/// The message names the command and the reason, so a caller reading the failure
/// knows it is a missing argument and not a dead database.
pub fn require_project_id(command: &str, project_id: Option<String>) -> Result<String, String> {
    project_id.ok_or_else(|| {
        format!(
            "{command}: project_id is required. An unscoped read would return every project's rows at once, which is the defect being fixed — the UI must pass the project the operator selected."
        )
    })
}

#[cfg(test)]
mod require_project_id_tests {
    use super::require_project_id;

    /// The whole point of the helper: an absent project_id must FAIL, not fall
    /// back. A test that only checks the happy path would pass while the defect
    /// is still there.
    #[test]
    fn absent_project_id_is_an_error_not_a_default() {
        let err = require_project_id("cmd_list_seats", None)
            .expect_err("an absent project_id must be an error, never a default");
        assert!(
            err.contains("cmd_list_seats"),
            "the message must name the command so a caller knows which read failed: {err}"
        );
        assert!(
            err.contains("project_id is required"),
            "the message must say what is missing: {err}"
        );
    }

    /// An empty string is still a project id the caller chose; only absence is
    /// refused. Refusing `Some("")` here would push the emptiness down into SQL,
    /// where it reads as "no rows" instead of "you asked for nothing".
    #[test]
    fn a_supplied_project_id_passes_through_unchanged() {
        let pid = require_project_id("cmd_list_seats", Some("ql_delivery".into()))
            .expect("a supplied project_id must pass through");
        assert_eq!(pid, "ql_delivery");
    }

    /// The message must not be generic enough to be mistaken for a dead database.
    #[test]
    fn the_error_names_the_reason_not_just_the_symptom() {
        let err = require_project_id("cmd_list_dispatches", None).expect_err("must error");
        assert!(
            err.contains("unscoped read"),
            "the message must explain WHY, so a reader does not debug the database: {err}"
        );
    }
}
