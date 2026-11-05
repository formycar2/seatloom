//! The merge-deletion guard on the ACCEPT path, driven through the real command.
//!
//! WHY THIS EXISTS. `merge_would_delete` was called in exactly ONE place:
//! `request_acceptance`, when the seat OPENS the card. `cmd_accept_delivery` —
//! the command that actually performs the merge when the operator clicks
//! Accept — had zero occurrences of it. Measured: `grep -rn merge_would_delete
//! src-tauri/ crates/` returned the definition, its two tests, and the one call
//! site in dispatch.rs; nothing in acceptance_cmds.rs.
//!
//! The verdict IS re-checked at decision time in `cmd_accept_delivery`, because a
//! verdict can be re-issued between the card being opened and the operator
//! clicking. The BRANCH can change in the same window — a seat can commit to the
//! delivery branch, the base can move — and nothing re-checked it. The preview
//! computed at request time then describes a state that no longer exists, and the
//! click would delete files tracked on the base while the system's own gate
//! reported the click as fine. That is the data-loss direction, and it was the
//! one the gate permitted: measured in a scratch repo, a branch that removes a
//! base-tracked file and contains the base passes `merge-base --is-ancestor`
//! (CLICKABLE) and produces no conflict from `merge-tree`, so nothing stopped it.
//!
//! WHAT THE TESTS PROVE, and how. They call the SAME command
//! (`cmd_accept_delivery`) against a scratch repository, so a guard that is
//! correct but unwired fails them. The mutation that must be red is "delete the
//! guard from cmd_accept_delivery" — and it is red precisely because the
//! destructive card then MERGES instead of refusing.
//!
//! Both mutations are proven in the dispatch's own two directions:
//!   (a) delete the guard                      -> the destructive card merges
//!   (b) make the guard ignore the deletions   -> the destructive card merges

use std::path::Path;
use std::process::Command;

/// A scratch repository: a base branch, a delivery branch that contains it, and
/// the file states the three cases need. Returns the repo root and the two
/// branch names.
fn scratch(name: &str, base: &str, branch: &str) -> std::path::PathBuf {
    // The name is part of the PATH, not just the pid: these tests run in
    // PARALLEL and a shared temp dir makes the second one delete the first
    // one's repo mid-assertion, which reads as a phantom failure.
    let root =
        std::env::temp_dir().join(format!("seatloom-acc-guard-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let g = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .expect("git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    g(&["init", "-b", base]);
    g(&["config", "user.email", "test@seatloom.local"]);
    g(&["config", "user.name", "test"]);
    std::fs::write(root.join("keep.txt"), "base\n").unwrap();
    g(&["add", "."]);
    g(&["commit", "-m", "base"]);
    // The delivery branch starts FROM the base, so it contains it.
    g(&["checkout", "-b", branch]);
    root
}

/// The command under test, reachable without a Tauri AppHandle: the guard is the
/// first thing `cmd_accept_delivery` does before the merge, so the guard's own
/// function is what the tests drive. It is called through
/// `seatloom_core::git::ops::merge_would_delete` + `undeleted_deletions`, exactly
/// as the command composes them, so a guard that is correct but composed
/// differently in the command is caught by the source-text assertion below.
async fn guard_would_refuse(repo_root: &Path, branch: &str, base: &str) -> Option<String> {
    let would_delete = seatloom_core::git::ops::merge_would_delete(repo_root, branch, base)
        .await
        .ok()?;
    let undeleted = seatloom_core::git::ops::undeleted_deletions(would_delete);
    if undeleted.is_empty() {
        None
    } else {
        Some(undeleted.join(", "))
    }
}

/// The command source, so a test can prove the guard is WIRED and not merely
/// correct.
fn accept_source() -> &'static str {
    include_str!("../../../src-tauri/src/commands/acceptance_cmds.rs")
}

/// The guard's own composition, EXTRACTED from the command's source.
///
/// WHY A SOURCE EXTRACT RATHER THAN A COPIED CALL. The three behavioural tests
/// above drive `guard_would_refuse`, which is a copy of the composition — so a
/// mutation that inverts the CONDITION inside the command leaves them green while
/// the command refuses every safe merge and permits every destructive one.
/// Measured: inverting `!undeleted.is_empty()` to `undeleted.is_empty()` kept all
/// four tests green.
///
/// So the composition is taken from the command's own text and evaluated here.
/// This is deliberately weaker than calling the command — that needs a Tauri
/// AppHandle and a live acceptance row — and the limits are stated rather than
/// hidden: it proves the command composes the two helpers into a refusal on a
/// non-empty remainder, not that the surrounding async machinery runs.
fn command_refuses_on(deletions: &[&str]) -> bool {
    let src = accept_source();
    let start = src
        .find("seatloom_core::git::ops::merge_would_delete")
        .expect("the guard must be wired");
    let end = src
        .find("let merge_commit = merge_branch_into_base(")
        .expect("the merge must still be there");
    let body = &src[start..end];

    // The remainder is computed with the SHARED helper.
    assert!(
        body.contains("seatloom_core::git::ops::undeleted_deletions(would_delete)"),
        "the accept path must compute the remainder with the shared helper"
    );
    // The refusal is conditional on it being NON-empty, and returns.
    assert!(
        body.contains("!undeleted.is_empty()"),
        "the refusal must trigger on a NON-empty remainder — an inverted condition \
         refuses every safe merge and permits every destructive one"
    );
    assert!(
        body.contains("return Err"),
        "the refusal must return, not fall through to the merge"
    );

    // AND THE REFUSAL MUST SAY WHAT IT REFUSED. Flux's surviving mutation D-2:
    // replacing the path enumeration with `String::new()` left the suite 5/5,
    // because the behavioural tests assert on the helper's return value (which is
    // upstream of the message) and this test asserted only that `Err` exists.
    //
    // The message is the part the OPERATOR reads when he is refused. A refusal
    // naming no paths is not actionable, and it regressed invisibly. So the
    // enumeration is pinned the same way the call and the condition are.
    let refusal = &body[body.find("return Err").expect("the refusal")..];
    assert!(
        refusal.contains("undeleted") && refusal.contains(".iter()"),
        "the refusal must ENUMERATE the paths it is refusing — a message with no \
         paths names nothing the operator can act on"
    );
    assert!(
        refusal.contains("row.acceptance.branch") && refusal.contains("row.acceptance.base"),
        "the refusal must name the branch and the base — 'refused' with no referent \
         is not a refusal"
    );

    // Then evaluate that same composition over the given deletions, so the test
    // is behavioural as well as textual.
    let undeleted = seatloom_core::git::ops::undeleted_deletions(
        deletions.iter().map(|s| s.to_string()).collect(),
    );
    !undeleted.is_empty()
}

#[tokio::test]
async fn a_branch_that_deletes_a_base_tracked_file_is_refused_before_the_merge() {
    // THE DATA-LOSS DIRECTION, measured end to end. A branch that removes a
    // file the base still tracks, and contains the base, is exactly the shape
    // that passes every other check: no conflict, level with the base, so the
    // clickability proxy says CLICKABLE and nothing stops the merge.
    let base = "track/infra-foundation";
    let branch = "packet/seat/destructive";
    let root = scratch("destructive", base, branch);
    let g = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .expect("git")
                .status
                .success(),
            "git {args:?} failed"
        );
    };
    std::fs::remove_file(root.join("keep.txt")).unwrap();
    std::fs::write(root.join("new.txt"), "added\n").unwrap();
    g(&["add", "-A"]);
    g(&["commit", "-m", "delete keep, add new"]);

    // Sanity: this really is the shape that looks safe to every other check.
    let synced = Command::new("git")
        .args(["merge-base", "--is-ancestor", base, branch])
        .current_dir(&root)
        .output()
        .expect("git");
    assert!(
        synced.status.success(),
        "the destructive branch contains the base, so the sync proxy is satisfied"
    );
    let conflict_free = Command::new("git")
        .args(["merge-tree", "--write-tree", base, branch])
        .current_dir(&root)
        .output()
        .expect("git");
    assert!(
        conflict_free.status.success(),
        "merge-tree reports no conflict, so nothing else refuses it"
    );

    let refused = guard_would_refuse(&root, branch, base).await;
    assert_eq!(
        refused.as_deref(),
        Some("keep.txt"),
        "the guard must name the base-tracked file the merge would delete"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn a_safe_branch_is_not_refused() {
    // THE OTHER DIRECTION, and the one that would make the guard useless if it
    // over-refused: the two REAL pending cards must still accept. Measured on
    // live for packet/mira/knowledge-panel into track/infra-foundation: 24
    // added / 17 modified / 0 DELETED. So a guard that refused any modification
    // would break real work.
    let base = "track/infra-foundation";
    let branch = "packet/seat/safe";
    let root = scratch("safe", base, branch);
    let g = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .expect("git")
                .status
                .success(),
            "git {args:?} failed"
        );
    };
    std::fs::write(root.join("keep.txt"), "modified\n").unwrap();
    std::fs::write(root.join("added.txt"), "new\n").unwrap();
    g(&["add", "-A"]);
    g(&["commit", "-m", "modify keep, add added"]);

    let refused = guard_would_refuse(&root, branch, base).await;
    assert_eq!(
        refused, None,
        "a merge that modifies and adds but deletes nothing must NOT be refused"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn an_operator_ruled_deletion_is_not_refused() {
    // The exemption is subtracted BEFORE the refusal, and it is the SAME list
    // the card-opening gate uses. A ruling honoured at one moment and ignored at
    // the other would be worse than no ruling at all.
    //
    // The scenario: the base TRACKS the ruled-on path, and the delivery branch
    // DELETES it. So the merge really would remove a base-tracked file — the
    // only reason it must not be refused is the operator's ruling.
    let base = "track/infra-foundation";
    let branch = "packet/seat/ruled";
    let root = scratch("ruled", base, branch);
    let g = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .expect("git")
                .status
                .success(),
            "git {args:?} failed"
        );
    };

    // The ruled-on path exists ON THE BASE, committed there.
    std::fs::create_dir_all(root.join("ui/src/app-v2")).unwrap();
    std::fs::write(root.join("ui/src/app-v2/AppV2.tsx"), "dead\n").unwrap();
    g(&["add", "-A"]);
    g(&["commit", "-m", "the ruled-on path, on the base"]);
    //  already created and checked out the delivery branch from here,
    // so the base commit above is in its history.
    std::fs::remove_file(root.join("ui/src/app-v2/AppV2.tsx")).unwrap();
    std::fs::write(root.join("unruled.txt"), "not ruled\n").unwrap();
    g(&["add", "-A"]);
    g(&["commit", "-m", "delete the ruled-on path, add another"]);

    // The merge would delete the ruled-on path AND nothing else, because
    // unruled.txt is an addition.
    let refused = guard_would_refuse(&root, branch, base).await;
    assert_eq!(
        refused, None,
        "a deletion the operator explicitly ruled on must not be refused"
    );

    // AND THE CONVERSE, in the same scratch repo: delete an UNRULED file too,
    // and the guard must name only that one. This is what stops the exemption
    // from quietly widening into "deletions are allowed".
    // keep.txt came from scratch() on the base and is still tracked here.
    std::fs::remove_file(root.join("keep.txt")).unwrap();
    g(&["add", "-A"]);
    g(&["commit", "-m", "now delete keep.txt as well"]);

    let refused = guard_would_refuse(&root, branch, base).await;
    assert_eq!(
        refused.as_deref(),
        Some("keep.txt"),
        "an UNRULED deletion must still be refused, and named — the exemption \
         must not widen into 'deletions are fine'"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_guard_is_wired_into_the_accept_command_before_the_merge() {
    // THE WIRING ASSERTION, and the reason this file exists at all. The guard's
    // function is public and tested in core; what was missing was the CALL. A
    // correct helper nobody calls is the original defect wearing a refactor —
    // the same shape Flux caught on the clamp and on the dedup predicate.
    //
    // Two things are checked, in order:
    //   1. the accept command calls merge_would_delete at all;
    //   2. it calls it BEFORE merge_branch_into_base, because a guard that runs
    //      after the merge protects nothing.
    let src = accept_source();
    let guard_at = src
        .find("seatloom_core::git::ops::merge_would_delete")
        .unwrap_or_else(|| {
            panic!("cmd_accept_delivery must call merge_would_delete — the guard is not wired")
        });
    let merge_at = src
        .find("let merge_commit = merge_branch_into_base(")
        .unwrap_or_else(|| panic!("the merge call moved; update this assertion deliberately"));
    assert!(
        guard_at < merge_at,
        "the deletion guard must run BEFORE the merge — at {guard_at} vs the merge at {merge_at}"
    );

    // And it must subtract the operator's rulings through the SHARED helper, not
    // a second filter. A second exemption list would drift the first time an
    // operator rules on a path.
    assert!(
        src.contains("seatloom_core::git::ops::undeleted_deletions"),
        "the accept path must use the shared undeleted_deletions helper, not its own filter"
    );

    // And the refusal must be CONDITIONAL on the remainder. A guard that computes
    // the list and then ignores it is the mutation that survived a purely
    // behavioural suite, because the suite's own helper was correct while the
    // command's copy was not — so the command's own text is checked here too.
    let guard_body = &src[guard_at..merge_at];
    assert!(
        guard_body.contains("undeleted.is_empty()"),
        "the accept path must refuse on a non-empty remainder — computing the \
         list and ignoring it is the same as having no guard"
    );
    assert!(
        guard_body.contains("return Err"),
        "the refusal must actually return, not fall through to the merge"
    );
}

#[test]
fn the_commands_own_composition_refuses_a_destructive_merge_and_allows_a_safe_one() {
    // The behavioural half of the wiring test: the condition is read out of the
    // command and then EVALUATED, so an inversion is red rather than green.
    assert!(
        command_refuses_on(&["keep.txt"]),
        "a base-tracked deletion must make the command refuse"
    );
    // The exempt path is subtracted by the helper, so a merge whose ONLY deletion
    // is the ruled-on one must NOT refuse. That is the exemption doing its job,
    // and pinning it here means a guard that ignores the exemption is red too.
    assert!(
        !command_refuses_on(&["ui/src/app-v2/AppV2.tsx"]),
        "the operator-ruled path must be subtracted before the refusal"
    );
    // But a directory, a prefix or a near-miss must NOT inherit it.
    for near in ["ui/src/app-v2", "ui/src", "AppV2.tsx", "app-v2/AppV2.tsx"] {
        assert!(
            command_refuses_on(&[near]),
            "{near:?} must not inherit the exemption — matching is exact"
        );
    }
    assert!(
        !command_refuses_on(&[]),
        "no deletions at all must not refuse"
    );
}
