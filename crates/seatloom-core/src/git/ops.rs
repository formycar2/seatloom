// Git operations: commit, branch, diff, last-commit hash.
// Full implementation deferred to git-ops packet.
//
// What IS implemented here is the git vocabulary the governed-dispatch
// acceptance gate (014, D2) needs before it records anything:
//   - that a branch NAME exists in the repository (`local_branch_exists`);
//   - that the delivery branch CONTAINS the base (`branch_contains`);
//   - which files tracked on the base a merge would DELETE
//     (`merge_would_delete`).
// `request_acceptance` runs all three at request time and
// `cmd_accept_delivery` re-runs them at decision time, and both call these
// helpers so each refusal has one implementation and one wording — never a
// second, divergent copy.

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

/// Is `base` an ANCESTOR of `branch`? (`git merge-base --is-ancestor`.)
///
/// This is the "your branch is synced" question, asked as a fact about the
/// repository rather than as discipline: a delivery whose branch does not
/// contain its base would, on accept, merge a branch that never saw the base's
/// later work — reverting whatever landed on the base since the fork.
/// `Ok(false)` means "a real work tree, and base is not an ancestor"; a root
/// that is not a work tree is an `Err` (see `local_branch_exists`).
pub async fn branch_contains(repo_root: &Path, branch: &str, base: &str) -> Result<bool, String> {
    let out = tokio::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", base, branch])
        .current_dir(repo_root)
        .output()
        .await
        .map_err(|e| format!("git merge-base --is-ancestor {base} {branch}: {e}"))?;
    // Exit status 0 = ancestor, 1 = not an ancestor, anything else = git itself
    // failed (unknown ref, not a repo). A failure must not read as "not an
    // ancestor", so ask the work-tree question before answering.
    if out.status.success() {
        return Ok(true);
    }
    if out.status.code() == Some(1) {
        return Ok(false);
    }
    let detail = String::from_utf8_lossy(&out.stderr).trim().to_string();
    Err(format!(
        "git merge-base --is-ancestor {base} {branch} failed: {detail}"
    ))
}

/// Which files TRACKED ON `base` would a merge of `branch` into `base` DELETE?
///
/// This is the structural defence against the failure mode that made Accept
/// destroy already-merged work: a branch that deleted files the base still
/// tracks would, on accept, delete them on the base. Previewing is cheap and
/// side-effect-free — `git merge-tree` writes nothing, touches no ref, and
/// leaves the working tree exactly as found (unlike a `merge --no-commit`
/// rehearsal, which does).
///
/// The answer is the list of paths the merge would REMOVE relative to the base's
/// current tree; an empty list means the merge deletes nothing tracked.
pub async fn merge_would_delete(
    repo_root: &Path,
    branch: &str,
    base: &str,
) -> Result<Vec<String>, String> {
    // `merge-tree --write-tree` (git >= 2.38) prints the resulting tree oid on
    // the first line and, on conflict, a conflicted-file-info block. We only
    // need the tree: diffing base..tree gives the exact file-level effect.
    let out = tokio::process::Command::new("git")
        .args(["merge-tree", "--write-tree", base, branch])
        .current_dir(repo_root)
        .output()
        .await
        .map_err(|e| format!("git merge-tree --write-tree {base} {branch}: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let Some(tree) = stdout
        .lines()
        .next()
        .map(str::trim)
        .filter(|l| !l.is_empty())
    else {
        let detail = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!(
            "git merge-tree --write-tree {base} {branch} produced no tree: {detail}"
        ));
    };
    // Files present on the base and absent from the merge result = deleted.
    let diff = tokio::process::Command::new("git")
        .args(["diff", "--name-status", &format!("{base}..{tree}")])
        .current_dir(repo_root)
        .output()
        .await
        .map_err(|e| format!("git diff --name-status {base}..{tree}: {e}"))?;
    if !diff.status.success() {
        let detail = String::from_utf8_lossy(&diff.stderr).trim().to_string();
        return Err(format!(
            "git diff --name-status {base}..{tree} failed: {detail}"
        ));
    }
    let mut deleted = Vec::new();
    for line in String::from_utf8_lossy(&diff.stdout).lines() {
        // `D<tab>path` — a deletion relative to the base.
        if let Some(rest) = line.strip_prefix("D\t") {
            let path = rest.trim();
            if !path.is_empty() {
                deleted.push(path.to_string());
            }
        }
    }
    deleted.sort();
    Ok(deleted)
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

    /// A scratch repo with a base branch, a branch that CONTAINS it, a branch
    /// that does not, and a branch that DELETES a base-tracked file — the three
    /// answers the containment defence depends on.
    #[tokio::test]
    async fn containment_helpers_answer_synced_and_would_delete() {
        let root = std::env::temp_dir().join(format!("seatloom-git-ops-ct-{}", std::process::id()));
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
        ok(&["init", "-b", "track"]);
        ok(&["config", "user.email", "test@seatloom.local"]);
        ok(&["config", "user.name", "test"]);
        std::fs::write(root.join("keep.txt"), "base\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "base"]);

        // synced: branched from the base, then the base moved on and the branch
        // merged it in.
        ok(&["checkout", "-b", "packet/seat/synced"]);
        std::fs::write(root.join("added.txt"), "delivery\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "delivery"]);
        ok(&["checkout", "track"]);
        std::fs::write(root.join("later.txt"), "later\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "base moves on"]);
        ok(&["checkout", "packet/seat/synced"]);
        ok(&["merge", "--no-edit", "track"]);

        // unsynced: forked from the base and never merged it back.
        ok(&["checkout", "track"]);
        ok(&["checkout", "-b", "packet/seat/unsynced", "HEAD~1"]);
        std::fs::write(root.join("other.txt"), "other\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "never synced"]);

        // destructive: branched from the base and DELETED a base-tracked file.
        ok(&["checkout", "track"]);
        ok(&["checkout", "-b", "packet/seat/destructive"]);
        std::fs::remove_file(root.join("keep.txt")).unwrap();
        ok(&["add", "-A"]);
        ok(&["commit", "-m", "deletes a tracked file"]);

        // The preview must not touch the working tree or any ref: capture the
        // state and compare it after every preview below.
        let before_status = String::from_utf8_lossy(&g(&["status", "--porcelain"]).stdout)
            .trim()
            .to_string();
        let before_head = String::from_utf8_lossy(&g(&["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();

        assert!(
            branch_contains(&root, "packet/seat/synced", "track")
                .await
                .unwrap(),
            "a branch that merged the base contains it"
        );
        assert!(
            !branch_contains(&root, "packet/seat/unsynced", "track")
                .await
                .unwrap(),
            "a branch that never merged the base does not contain it"
        );
        assert!(
            merge_would_delete(&root, "packet/seat/synced", "track")
                .await
                .unwrap()
                .is_empty(),
            "a clean delivery deletes nothing"
        );
        let deleted = merge_would_delete(&root, "packet/seat/destructive", "track")
            .await
            .unwrap();
        assert_eq!(deleted, vec!["keep.txt".to_string()], "{deleted:?}");
        // The unsynced branch is caught by the contains check, but its merge
        // preview must still be answerable (it deletes nothing).
        assert!(merge_would_delete(&root, "packet/seat/unsynced", "track")
            .await
            .unwrap()
            .is_empty());
        // The working tree is untouched by the preview: no ref moved, no file
        // changed.
        let after_head = String::from_utf8_lossy(&g(&["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
        let after_status = String::from_utf8_lossy(&g(&["status", "--porcelain"]).stdout)
            .trim()
            .to_string();
        assert_eq!(before_head, after_head, "the preview must not move HEAD");
        assert_eq!(
            before_status, after_status,
            "the preview must not change the working tree"
        );

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
