pub mod acceptance_cmds;
pub mod artifact_cmds;
pub mod delegation_cmds;
pub mod dispatch_cmds;
pub mod document_cmds;
pub mod handoff_cmds;
pub mod history_cmds;
pub mod inbox_cmds;
pub mod knowledge_cmds;
pub mod operator_decision_cmds;
pub mod project_cmds;
pub mod prompt_cmds;
pub mod reconcile_cmds;
pub mod runtime_cmds;
pub mod seat_cmds;
pub mod session_cmds;
pub mod session_work_cmds;
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

/// Source-level contract for the four write/spawn paths.
///
/// These commands cannot be unit-tested directly — they spawn a child process and
/// speak JSON-RPC over stdio — so the claim that matters ("they no longer reach
/// for the startup constant") is checked against their own source.
///
/// Why a source check and not a mock: the defect is precisely that the child is
/// spawned with a CONSTANT, and the observable consequence (a dispatch filed under
/// the wrong project) only appears against a live database with two projects. A
/// source check fails the moment the constant comes back, which is the direction
/// the regression would come from.
///
/// The strings are spelled so that this file's own prose cannot satisfy them.
#[cfg(test)]
mod spawn_path_project_scoping_tests {
    use std::fs;

    fn source(rel: &str) -> String {
        // CARGO_MANIFEST_DIR is src-tauri/, so the commands live under src/.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/commands")
            .join(rel);
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    /// Every one of the four must call `require_project_id`. That call is the
    /// refusal; without it an absent id becomes a default somewhere downstream.
    #[test]
    fn each_of_the_four_calls_require_project_id() {
        let cases = [
            ("dispatch_cmds.rs", "cmd_enqueue_dispatch"),
            ("dispatch_cmds.rs", "cmd_plan_intent"),
            ("history_cmds.rs", "cmd_list_history_projects"),
            ("history_cmds.rs", "cmd_get_history_session_turns"),
            ("session_work_cmds.rs", "cmd_session_work"),
            ("session_work_cmds.rs", "cmd_memory_session"),
        ];
        for (file, command) in cases {
            let src = source(file);
            let needle = format!("require_project_id(\"{command}\"");
            assert!(
                src.contains(&needle),
                "{file}: {command} must refuse an absent project_id via require_project_id — \
                 a silent fallback files the row under the startup constant instead"
            );
        }
    }

    /// The `SEATLOOM_PROJECT_ID` handed to a child must come from the caller's
    /// project, never from `state.default_project_id`. This is the mutation that
    /// matters: the four commands still compile and still work when this regresses,
    /// and the only visible symptom is a dispatch landing in the wrong project.
    #[test]
    fn no_spawn_path_hands_the_child_the_startup_constant() {
        for file in [
            "dispatch_cmds.rs",
            "history_cmds.rs",
            "session_work_cmds.rs",
        ] {
            let src = source(file);
            // The comment blocks legitimately NAME the constant to record what was
            // removed, so count the `.env(...)` calls rather than any occurrence.
            let spawns = src
                .lines()
                .filter(|l| {
                    let t = l.trim();
                    t.starts_with(".env(\"SEATLOOM_PROJECT_ID\"")
                })
                .filter(|l| l.contains("default_project_id"))
                .count();
            assert_eq!(
                spawns, 0,
                "{file}: a spawn still hands the child state.default_project_id — \
                 that constant is one project set once at startup, and every dispatch \
                 created under any other project is filed there"
            );
        }
    }
}
