// Runtime identity commands — the backend half of the sandbox banner.
//
// WHY. AEGIS-2026-10-21-sandbox-build-banner-v1. The app icon distinguishes the
// sandbox from the main build in the Dock, but once a window is open the two look
// identical — the icon is not visible inside the window. The operator asked for a
// permanent in-window identifier so he can tell which build he is looking at without
// alt-tabbing.
//
// WHY ENVIRONMENT VARIABLES AND NOT localStorage. The knowledge note
// `docs/coordination/knowledge/ui_cannot_launch_app_yet_and_two_holes_must_not_stack.md`
// records that of the four localStorage keys, three have no namespace, so the sandbox
// and the main build read the same state when both are open and overwrite each other.
// Anything the banner shows must therefore come from outside the webview's own storage.
//
// 🔴 MEASURED, not assumed (2026-10-21): I built a throwaway .app bundle, had it print
// its environment, and launched it the way `prepare-sandbox-build.sh` does. `open -a`
// with `SEATLOOM_SANDBOX=1 ...` exported in the launching shell **does** reach the
// application — the child saw the variable. So the launch mechanism the script already
// uses is sound and does not need changing. (Also measured: `launchctl setenv` does
// not propagate to a subsequently launched app, which is why that alternative was not
// used.)
//
// WHAT THIS DOES NOT DO. It does not start any process, open any connection, write to
// the database, or grant the frontend any new capability. It reads four environment
// variables that the installer already sets and reports them. A missing value is
// reported as `None` so the frontend can print "未记录" rather than inventing one.

use serde::{Deserialize, Serialize};

/// What the frontend needs to render the banner.
///
/// `is_sandbox` is the only field that is ever a hard yes/no: it answers "am I the
/// sandbox", and everything else describes *which* sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeIdentityDto {
    /// True when the `SEATLOOM_SANDBOX` env var is exactly `"1"`.
    ///
    /// Not merely present-and-truthy in a loose sense: an empty string, a `"0"` or a
    /// typo must not light the banner. If the installer ever writes something other
    /// than `"1"`, the frontend will show "not recorded" and the operator will notice,
    /// which is the correct failure direction — a wrongly-lit banner is worse than a
    /// missing one.
    pub is_sandbox: bool,
    /// `SEATLOOM_SANDBOX_BRANCH` — the branch this build came from.
    pub branch: Option<String>,
    /// `SEATLOOM_SANDBOX_COMMIT` — the short commit this build came from.
    pub commit: Option<String>,
    /// `SEATLOOM_SANDBOX_BUILT` — when it was built, as the installer formatted it.
    pub built_at: Option<String>,
}

/// Read an env var, treating an empty string as absent.
///
/// An exported-but-empty variable is not information. `export X=` yields `""`, and
/// passing that to the frontend would render a blank where "未记录" belongs, which is
/// what this whole banner exists to prevent.
fn env_value(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

/// Report the runtime identity of this process.
///
/// This command cannot fail: there is no I/O it performs that can fail. That is
/// deliberate — a banner that errors out would be indistinguishable from a missing one,
/// and the caller cannot tell "the command is broken" from "this is the main build",
/// which is precisely the confusion the banner exists to remove. The three description
/// fields are `Option`s so an absent value stays absent all the way to the screen.
#[tauri::command]
pub async fn cmd_get_runtime_identity() -> Result<RuntimeIdentityDto, String> {
    Ok(RuntimeIdentityDto {
        is_sandbox: env_value("SEATLOOM_SANDBOX").as_deref() == Some("1"),
        branch: env_value("SEATLOOM_SANDBOX_BRANCH"),
        commit: env_value("SEATLOOM_SANDBOX_COMMIT"),
        built_at: env_value("SEATLOOM_SANDBOX_BUILT"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests mutate process-global environment state, so they must not run in
    // parallel with each other. `serial_test` is not a dependency here, so the guard is
    // a mutex held across each test body instead — and it is held by `set_sandbox_env`,
    // so acquiring it once per test is what makes the sequence safe.
    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Snapshots the sandbox env keys, and restores them on drop.
    ///
    /// Without this, every test in this file is order-dependent: `sandbox_one_means_sandbox`
    /// sets BRANCH/COMMIT/BUILT, and a later test that expects them absent would fail
    /// depending on where the scheduler happened to put it. (That exact failure
    /// happened here first — the snapshot/restore was written as a comment, not as code,
    /// and a comment does not restore anything.)
    ///
    /// It also holds the process-wide lock, so tests cannot interleave their writes.
    struct SandboxEnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        saved: Vec<(&'static str, Option<String>)>,
    }

    fn set_sandbox_env(vars: &[(&'static str, Option<&str>)]) -> SandboxEnvGuard {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved: Vec<(&'static str, Option<String>)> =
            KEYS.iter().map(|k| (*k, std::env::var(k).ok())).collect();
        for (k, v) in vars {
            match v {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
        SandboxEnvGuard { _lock: lock, saved }
    }

    impl Drop for SandboxEnvGuard {
        fn drop(&mut self) {
            for (k, v) in &self.saved {
                match v {
                    Some(v) => std::env::set_var(k, v),
                    None => std::env::remove_var(k),
                }
            }
        }
    }

    /// The keys every test touches, so each one starts from a clean slate.
    const KEYS: [&str; 4] = [
        "SEATLOOM_SANDBOX",
        "SEATLOOM_SANDBOX_BRANCH",
        "SEATLOOM_SANDBOX_COMMIT",
        "SEATLOOM_SANDBOX_BUILT",
    ];

    fn clear_sandbox_env() -> SandboxEnvGuard {
        let none: Vec<(&str, Option<&str>)> = KEYS.iter().map(|k| (*k, None)).collect();
        set_sandbox_env(&none)
    }

    #[tokio::test]
    async fn sandbox_one_means_sandbox() {
        let _guard = set_sandbox_env(&[
            ("SEATLOOM_SANDBOX", Some("1")),
            (
                "SEATLOOM_SANDBOX_BRANCH",
                Some("packet/mira/knowledge-panel"),
            ),
            ("SEATLOOM_SANDBOX_COMMIT", Some("abc1234")),
            ("SEATLOOM_SANDBOX_BUILT", Some("2026-10-21 13:20")),
        ]);
        let id = cmd_get_runtime_identity().await.unwrap();
        assert!(id.is_sandbox);
        assert_eq!(id.branch.as_deref(), Some("packet/mira/knowledge-panel"));
        assert_eq!(id.commit.as_deref(), Some("abc1234"));
        assert_eq!(id.built_at.as_deref(), Some("2026-10-21 13:20"));
    }

    /// The main build never sets the variable, and everything else is absent with it.
    #[tokio::test]
    async fn main_build_reports_no_sandbox_and_no_details() {
        let _guard = clear_sandbox_env();
        let id = cmd_get_runtime_identity().await.unwrap();
        assert!(!id.is_sandbox);
        assert!(id.branch.is_none());
        assert!(id.commit.is_none());
        assert!(id.built_at.is_none());
    }

    /// A near-miss must not light the banner. If the installer ever writes `"true"`
    /// instead of `"1"`, the frontend prints "not recorded" and the operator notices —
    /// a wrongly-claimed sandbox would be the more dangerous outcome.
    #[tokio::test]
    async fn only_exactly_one_counts() {
        let _guard = clear_sandbox_env();
        for near_miss in ["", "0", "true", "yes", "01", " 1", "1 "] {
            std::env::set_var("SEATLOOM_SANDBOX", near_miss);
            let id = cmd_get_runtime_identity().await.unwrap();
            assert!(!id.is_sandbox, "{near_miss:?} must not read as sandbox");
        }
    }

    /// An exported-but-empty variable is not information; it must arrive as `None` so
    /// the UI can print "not recorded" instead of a blank.
    #[tokio::test]
    async fn empty_and_whitespace_values_are_absent() {
        let _guard = set_sandbox_env(&[
            ("SEATLOOM_SANDBOX", Some("1")),
            ("SEATLOOM_SANDBOX_BRANCH", Some("")),
            ("SEATLOOM_SANDBOX_COMMIT", Some("   ")),
        ]);
        let id = cmd_get_runtime_identity().await.unwrap();
        assert!(id.is_sandbox);
        assert!(id.branch.is_none(), "empty is absent");
        assert!(id.commit.is_none(), "whitespace is absent");
    }

    /// The description fields are independent of the flag. A sandbox whose installer
    /// failed to record the branch must still render, saying "not recorded" for that
    /// one field — not refuse to draw, and not guess.
    #[tokio::test]
    async fn sandbox_without_details_still_states_it_is_a_sandbox() {
        let _guard = set_sandbox_env(&[("SEATLOOM_SANDBOX", Some("1"))]);
        let id = cmd_get_runtime_identity().await.unwrap();
        assert!(id.is_sandbox);
        assert!(id.branch.is_none());
        assert!(id.commit.is_none());
        assert!(id.built_at.is_none());
    }
}
