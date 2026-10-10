// Git operations: commit, branch, diff, last-commit hash.
// Full implementation deferred to git-ops packet.
//
// What IS implemented here is the one operation the governed-dispatch
// acceptance gate (014, D2) needs before it records anything: proving that a
// branch NAME exists in the repository. `request_acceptance` validates the
// merge base at request time and `cmd_accept_delivery` validates both refs at
// decision time, and both call this helper so "the branch does not exist" is
// one refusal with one wording, not two.

use std::path::{Path, PathBuf};

/// Env var naming the repository root to operate on. Mirrors
/// `src-tauri/src/main.rs::repo_root`'s first branch: an EXPLICIT root wins,
/// because "wherever this process happened to start" is a guess, and a merge
/// must never run against a guessed repository.
pub const REPO_ROOT_ENV: &str = "SEATLOOM_REPO_ROOT";

#[derive(Default)]
pub struct GitOps;

impl GitOps {
    pub fn new() -> Self {
        Self
    }
}

/// Resolve the repository root: the explicit `SEATLOOM_REPO_ROOT` when set,
/// otherwise the process working directory. Never anything cleverer — a
/// directory-scanning heuristic would silently pick the wrong repo, and a
/// caller that cares passes the env var.
pub fn resolve_repo_root() -> PathBuf {
    if let Some(root) = std::env::var_os(REPO_ROOT_ENV).filter(|v| !v.is_empty()) {
        return PathBuf::from(root);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Does `branch` exist as a LOCAL branch (`refs/heads/<branch>`) in the
/// repository that contains `repo_root`?
///
/// Worktrees share the `refs/heads` namespace, so this answers correctly from
/// any worktree of the main checkout — which is what makes a request-time check
/// possible from a seat's own worktree at all.
///
/// `Ok(false)` means "a real git work tree, and that branch is absent". A root
/// that is not a work tree is an `Err`, NOT a false: a mistyped
/// `SEATLOOM_REPO_ROOT` must not read as "the branch is missing", because the
/// two failures call for different fixes.
pub async fn local_branch_exists(repo_root: &Path, branch: &str) -> Result<bool, String> {
    let verify = format!("refs/heads/{branch}");
    let out = tokio::process::Command::new("git")
        .args(["rev-parse", "--verify", &verify])
        .current_dir(repo_root)
        .output()
        .await
        .map_err(|e| format!("git rev-parse --verify {verify}: {e}"))?;
    if out.status.success() {
        return Ok(true);
    }
    // Absent or not-a-repo? Ask git directly rather than guessing from the
    // exit code, whose meaning differs across versions.
    let probe = tokio::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_root)
        .output()
        .await
        .map_err(|e| format!("git rev-parse --is-inside-work-tree: {e}"))?;
    let inside = probe.status.success() && String::from_utf8_lossy(&probe.stdout).trim() == "true";
    if !inside {
        return Err(format!(
            "{} is not a git work tree — refusing to answer a branch question about it",
            repo_root.display()
        ));
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch repo with two branches: one present, one absent, and a
    /// non-git directory beside it. The three answers must stay distinct.
    #[tokio::test]
    async fn local_branch_exists_separates_present_absent_and_non_git() {
        let root = std::env::temp_dir().join(format!("seatloom-git-ops-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let g = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap()
        };
        let ok = |args: &[&str]| assert!(g(args).status.success(), "git {args:?} failed");
        ok(&["init", "-b", "track/infra-foundation"]);
        ok(&["config", "user.email", "test@seatloom.local"]);
        ok(&["config", "user.name", "test"]);
        std::fs::write(root.join("f.txt"), "base\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "base"]);
        ok(&["checkout", "-b", "packet/seat/delivery"]);

        assert!(
            local_branch_exists(&root, "packet/seat/delivery")
                .await
                .unwrap(),
            "a checked-out branch exists"
        );
        assert!(
            local_branch_exists(&root, "track/infra-foundation")
                .await
                .unwrap(),
            "a branch left behind on another worktree's HEAD still exists in refs/heads"
        );
        assert!(
            !local_branch_exists(&root, "packet/seat/nope")
                .await
                .unwrap(),
            "an absent branch is Ok(false), not an error"
        );

        // A directory OUTSIDE any repository (a sibling at the temp root, not a
        // child of the scratch repo — a subdirectory of a repo IS inside its
        // work tree, so the question there is answerable).
        let not_git =
            std::env::temp_dir().join(format!("seatloom-git-ops-nogit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&not_git);
        std::fs::create_dir_all(&not_git).unwrap();
        let err = local_branch_exists(&not_git, "any")
            .await
            .expect_err("a non-git root must not read as 'branch absent'");
        assert!(err.contains("not a git work tree"), "{err}");
        let _ = std::fs::remove_dir_all(&not_git);

        let _ = std::fs::remove_dir_all(&root);
    }

    /// The env var wins over the CWD, and an EMPTY value is ignored rather than
    /// resolving to an empty path (an empty root would make every git call run
    /// in the CWD while claiming to be configured). Env mutation is serialised
    /// by a mutex because cargo runs this file's tests on parallel threads.
    #[test]
    fn resolve_repo_root_prefers_the_explicit_env_var() {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let had = std::env::var_os(REPO_ROOT_ENV);

        std::env::set_var(REPO_ROOT_ENV, "/tmp/explicit-seatloom-root");
        assert_eq!(
            resolve_repo_root(),
            PathBuf::from("/tmp/explicit-seatloom-root"),
            "an explicit root wins over the CWD"
        );

        std::env::set_var(REPO_ROOT_ENV, "");
        assert_eq!(
            resolve_repo_root(),
            std::env::current_dir().expect("cwd"),
            "an empty env value is ignored, never resolved as a path"
        );

        match had {
            Some(v) => std::env::set_var(REPO_ROOT_ENV, v),
            None => std::env::remove_var(REPO_ROOT_ENV),
        }
    }
}
