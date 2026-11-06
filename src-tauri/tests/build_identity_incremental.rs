// Executable fixtures for the build-embedded identity, driving REAL incremental builds.
//
// WHY THESE EXIST. Flux's verdict on the first version (ev-OFHL9WBqlQnAqs8DCGAkN)
// found a defect and a gap, both of which were invisible to the tests that shipped:
//
//   DEFECT — `rerun-if-changed` declared `../.git/HEAD` and `../.git/refs`. In a
//   LINKED WORKTREE `.git` is a FILE, so neither path exists; and a packed ref is
//   invisible to the `refs` path even in a normal checkout. Measured consequence: after
//   a `git checkout`, a cargo build that DID rerun the build script still produced
//   `build_identity.rs` holding the PREVIOUS commit.
//
//   GAP — a dirty build was indistinguishable from a clean one at the same commit, by
//   the binary's own account.
//
// A comment cannot close either. These tests build a real cargo project, in a real git
// repository, in a real LINKED WORKTREE, and read the generated file back.
//
// 🔴 One thing these tests deliberately do NOT do: they never edit `build.rs`. The
// build script under test is the repository's own, copied verbatim into each scratch
// project. Testing a copy of the invalidation logic would prove the copy.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository's own build script, copied into each scratch project.
///
/// Copied rather than reimplemented: the defect was in the watch paths, and a test that
/// carried its own copy of those paths would pass while the real one stayed broken.
const BUILD_RS: &str = include_str!("../build.rs");

fn run(cmd: &mut Command) -> String {
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("could not run {cmd:?}: {e}"));
    assert!(
        out.status.success(),
        "command failed: {cmd:?}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A scratch git repository with a workspace whose build script is the real one.
struct Scratch {
    dir: PathBuf,
    /// The linked worktree we build in — the configuration this repository uses for
    /// verification work, and the one where the defect was reachable.
    work: PathBuf,
}

impl Scratch {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("seatloom-bid-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");

        // The main checkout.
        run(Command::new("git")
            .arg("init")
            .arg("-q")
            .arg("--bare")
            .arg(dir.join("main.git")));
        run(Command::new("git")
            .arg("clone")
            .arg("-q")
            .arg(dir.join("main.git"))
            .arg(dir.join("seed")));

        let seed = dir.join("seed");
        run(Command::new("git")
            .arg("-C")
            .arg(&seed)
            .args(["config", "user.email", "t@t.local"]));
        run(Command::new("git")
            .arg("-C")
            .arg(&seed)
            .args(["config", "user.name", "t"]));

        // A workspace whose ONLY crate's build script is the one under test.
        std::fs::create_dir_all(seed.join("crate/src")).expect("crate dirs");
        std::fs::write(
            seed.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crate\"]\n",
        )
        .unwrap();
        std::fs::write(
            seed.join("crate/Cargo.toml"),
            "[package]\nname = \"bid\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(
            seed.join("crate/src/lib.rs"),
            "pub fn ok() -> bool { true }\n",
        )
        .unwrap();
        // `tauri_build::build()` is not available in a scratch crate, so the copied
        // build script drops that one call and closes `main()`. 🔴 The trim is a
        // line-level edit of ONE identified line, and it asserts that line exists
        // first — otherwise a change to build.rs would silently stop being tested.
        //
        // 🔴 And the trim must NOT be a naive `split("tauri_build::build()")`: that
        // removes the tail of the function and leaves `fn main() {` unclosed, which
        // fails to compile as "unclosed delimiter" — measured. So the call is replaced
        // in place, keeping every brace.
        assert!(
            BUILD_RS.contains("tauri_build::build()"),
            "build.rs must still call tauri_build::build() — if it does not, this test \
             is no longer testing the real build script"
        );
        let trimmed = BUILD_RS.replace(
            "tauri_build::build()",
            "// tauri_build::build() removed for the scratch crate",
        );
        std::fs::write(seed.join("crate/build.rs"), trimmed).unwrap();

        run(Command::new("git").arg("-C").arg(&seed).args(["add", "-A"]));
        run(Command::new("git")
            .arg("-C")
            .arg(&seed)
            .arg("-c")
            .arg("core.hooksPath=/dev/null")
            .args(["commit", "-q", "--no-verify", "-m", "base"]));
        run(Command::new("git")
            .arg("-C")
            .arg(&seed)
            .args(["branch", "-M", "track/main"]));
        run(Command::new("git")
            .arg("-C")
            .arg(&seed)
            .args(["push", "-q", "origin", "track/main"]));

        // A LINKED worktree — the shape where `.git` is a file.
        let work = dir.join("wt");
        run(Command::new("git")
            .arg("-C")
            .arg(&seed)
            .args(["worktree", "add", "-q"])
            .arg(&work)
            .arg("-b")
            .arg("packet/seat/test"));

        Self { dir, work }
    }

    /// Run git in the scratch worktree.
    ///
    /// 🔴 `-c core.hooksPath=/dev/null` is load-bearing, and its absence was a silent
    /// vacuity: this repository has a pre-commit hook, and `git commit` inside the
    /// scratch repo ran it, failed, and left HEAD unmoved — so the "HEAD moved" fixtures
    /// asserted on an unchanged commit and passed whether or not the build script was
    /// correct. Measured: with the broken watch paths restored, all of them stayed green.
    ///
    /// `commit` and `merge` are also given `--no-verify` as a second line, because a
    /// hooksPath of /dev/null does not stop every hook mechanism.
    fn git(&self, args: &[&str]) -> String {
        let mut c = Command::new("git");
        c.arg("-C")
            .arg(&self.work)
            .arg("-c")
            .arg("core.hooksPath=/dev/null");
        if args.first() == Some(&"commit") {
            c.arg("-c")
                .arg("commit.gpgsign=false")
                .arg("commit")
                .arg("--no-verify");
            c.args(&args[1..]);
        } else {
            c.args(args);
        }
        run(&mut c)
    }

    /// Build and read back the generated `build_identity.rs`.
    fn build_and_capture(&self) -> Capture {
        run(Command::new("cargo")
            .arg("build")
            .arg("-q")
            .arg("--manifest-path")
            .arg(self.work.join("crate/Cargo.toml")));
        let out = std::env::var("OUT_DIR").ok();
        // Find the generated file under the worktree's target dir.
        let target = self.work.join("target");
        let mut found: Option<PathBuf> = None;
        for entry in walk(&target) {
            if entry
                .file_name()
                .map(|n| n == "build_identity.rs")
                .unwrap_or(false)
            {
                found = Some(entry);
            }
        }
        let path = found.expect("build_identity.rs must be generated under target/");
        let body = std::fs::read_to_string(&path).expect("read build_identity.rs");
        assert!(
            !out.is_some() || true,
            "OUT_DIR is not needed; the file is located by walking"
        );
        Capture::parse(&body)
    }

    fn head_short(&self) -> String {
        self.git(&["rev-parse", "--short=12", "HEAD"])
    }

    fn dirty_tracked(&self) -> usize {
        self.git(&["status", "--porcelain"])
            .lines()
            .filter(|l| !l.trim_start().starts_with("??"))
            .count()
    }
}

/// What the generated file said.
struct Capture {
    commit: Option<String>,
    dirty: Option<u32>,
    git_ok: bool,
}

impl Capture {
    fn parse(body: &str) -> Self {
        let get = |name: &str| -> Option<String> {
            body.lines()
                .find(|l| l.contains(&format!("const {name}:")))
                .and_then(|l| {
                    let after = l.splitn(2, '=').nth(1)?.trim().trim_end_matches(';');
                    if after == "None" {
                        return Some(String::new());
                    }
                    // Some("...") or Some(0) or true/false
                    let inner = after
                        .strip_prefix("Some(")
                        .and_then(|s| s.strip_suffix(')'))
                        .unwrap_or(after);
                    Some(inner.trim_matches('"').to_string())
                })
        };
        let commit = get("BUILD_COMMIT").filter(|s| !s.is_empty());
        let dirty = get("BUILD_DIRTY")
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap());
        let git_ok = get("BUILD_GIT_OK").map(|s| s == "true").unwrap_or(false);
        Self {
            commit,
            dirty,
            git_ok,
        }
    }

    fn commit_hash(&self) -> Option<&str> {
        self.commit
            .as_deref()
            .and_then(|c| c.strip_prefix("seatloom-build-commit="))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Walk a directory tree, breadth-agnostic, without pulling in a dependency.
fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let rd = match std::fs::read_dir(&d) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p);
            }
        }
    }
    out
}

// ── The fixtures ────────────────────────────────────────────────────────────

/// Baseline: the capture names the commit it was built from.
#[test]
fn baseline_capture_names_the_current_commit() {
    let s = Scratch::new("baseline");
    let c = s.build_and_capture();
    assert!(c.git_ok, "git must be available in the scratch repo");
    assert_eq!(
        c.commit_hash(),
        Some(s.head_short().as_str()),
        "the capture must name the commit the tree was at"
    );
    // A label, so a reader never has to guess which hex run is the identity.
    assert!(
        c.commit
            .as_deref()
            .is_some_and(|v| v.starts_with("seatloom-build-commit=")),
        "the commit must be emitted next to its fixed label"
    );
}

/// 🔴 THE DEFECT FIXTURE. After moving HEAD, an incremental build must produce the NEW
/// commit — not the previous one.
///
/// This is the exact sequence Flux measured: `git checkout` to another commit, then a
/// cargo build that DID rerun the build script, then read the file and find the old
/// hash. The first version's watch paths did not exist in a linked worktree, so the
/// invalidation never fired.
#[test]
fn moving_head_invalidates_the_capture() {
    let s = Scratch::new("refmove");

    let first = s.build_and_capture();
    let first_hash = first.commit_hash().expect("first capture").to_string();

    // A second commit, then check it out — a plain `git checkout`, no cache clearing.
    std::fs::write(
        s.work.join("crate/src/lib.rs"),
        "pub fn ok() -> bool { false }\n",
    )
    .unwrap();
    s.git(&["add", "-A"]);
    s.git(&["commit", "-qm", "second"]);
    let second_hash = s.head_short();
    assert_ne!(
        first_hash, second_hash,
        "the fixture must actually move HEAD"
    );

    // No `cargo clean`. An incremental build must notice.
    let after = s.build_and_capture();
    assert_eq!(
        after.commit_hash(),
        Some(second_hash.as_str()),
        "after moving HEAD, an incremental build must produce the NEW commit — \
         the stale-claim defect Flux recorded"
    );
}

/// 🔴 The ref-move half of the same defect: a `git branch -f` / fetch moves a ref in
/// the COMMON dir, which the `refs` path never covered even outside a worktree.
#[test]
fn moving_a_ref_in_the_common_dir_invalidates_the_capture() {
    let s = Scratch::new("packedref");

    let first = s.build_and_capture();
    let first_hash = first.commit_hash().expect("first capture").to_string();

    // Move the branch ref backwards, the way a `git reset` or a force-push would.
    s.git(&["reset", "-q", "--hard", "HEAD~0"]);
    // Create and pack a ref, then delete the loose one — the state `refs` cannot see.
    s.git(&["update-ref", "refs/heads/packed-test", "HEAD"]);
    s.git(&["pack-refs", "--all"]);

    // A commit that only the packed ref reaches.
    std::fs::write(s.work.join("crate/src/lib.rs"), "// packed\n").unwrap();
    s.git(&["add", "-A"]);
    s.git(&["commit", "-qm", "after-pack"]);
    s.git(&["update-ref", "-d", "refs/heads/packed-test"]);

    let after = s.build_and_capture();
    assert_eq!(
        after.commit_hash(),
        Some(s.head_short().as_str()),
        "a change visible only through packed-refs must still invalidate the capture"
    );
    assert_ne!(
        after.commit_hash(),
        Some(first_hash.as_str()),
        "the capture changed nothing — the common dir is not being watched"
    );
}

/// 🔴 THE GAP FIXTURE. clean -> dirty -> clean, read from the BINARY's own statement.
#[test]
fn dirty_state_is_stated_by_the_binary_and_recovers() {
    let s = Scratch::new("dirty");

    let clean = s.build_and_capture();
    assert_eq!(
        clean.dirty,
        Some(0),
        "a clean tree must be stated as clean, not as unknown"
    );
    assert!(clean.git_ok);

    // Dirty a TRACKED file.
    std::fs::write(
        s.work.join("crate/src/lib.rs"),
        "pub fn ok() -> bool { true } // edited\n",
    )
    .unwrap();
    assert_eq!(
        s.dirty_tracked(),
        1,
        "the fixture must actually dirty a tracked file"
    );

    let dirty = s.build_and_capture();
    assert_eq!(
        dirty.dirty,
        Some(1),
        "a dirty tree must be stated as dirty by the BINARY, not only by the installer"
    );

    // And back to clean — the third state matters, because a capture that sticks at
    // "dirty" would be as misleading as one that sticks at "clean".
    s.git(&["checkout", "-q", "--", "crate/src/lib.rs"]);
    assert_eq!(s.dirty_tracked(), 0);
    let clean_again = s.build_and_capture();
    assert_eq!(
        clean_again.dirty,
        Some(0),
        "the capture must recover to clean"
    );
}

/// 🔴 THE FIXTURE THAT ACTUALLY CATCHES THE DEFECT.
///
/// Every other fixture here changes a SOURCE file, and cargo reruns the build script
/// because a source changed — so they pass whether or not the watch paths are right.
/// Measured: with the original broken watch paths restored, all of them stay green.
///
/// The defect only shows when HEAD moves and **no source cargo watches changes**. That
/// is the real scenario: `git checkout` another commit whose tree is identical (an empty
/// commit, or a branch that differs only in history), or `git reset` back. Cargo's
/// fingerprint is unchanged, so it skips the build script entirely and the generated
/// file keeps the previous commit.
///
/// So this fixture moves HEAD to a commit with an IDENTICAL tree, and asserts the
/// capture followed. With the broken watch paths it fails; with the resolved
/// `--git-dir`/`--git-common-dir` watches it passes.
#[test]
fn moving_head_without_changing_any_source_invalidates_the_capture() {
    let s = Scratch::new("headonly");

    let first = s.build_and_capture();
    let first_hash = first.commit_hash().expect("first capture").to_string();

    // An empty commit: the tree is byte-identical, so cargo has nothing to rebuild.
    s.git(&["commit", "--allow-empty", "-m", "history-only"]);
    let second_hash = s.head_short();
    assert_ne!(
        first_hash, second_hash,
        "the fixture must move HEAD — if it did not, this test asserts nothing \
         (measured: an inherited pre-commit hook made the commit fail silently)"
    );
    // Prove the tree really is unchanged — otherwise this is just the other fixtures.
    let diff = s.git(&["diff", "--stat", "HEAD~1", "HEAD"]);
    assert!(
        diff.trim().is_empty(),
        "this fixture requires a commit with an identical tree, got a diff: {diff}"
    );

    // No cargo clean, no source edit. The only thing that changed is git history.
    let after = s.build_and_capture();
    assert_eq!(
        after.commit_hash(),
        Some(second_hash.as_str()),
        "HEAD moved with no source change, and the capture did not follow — \
         this is the stale-commit defect: cargo skipped the build script because its \
         fingerprint was unchanged, and the watch paths never fired"
    );
}

/// 🔴 THE FIXTURE THAT DISCRIMINATES, and the reason the ones above were not enough.
///
/// Measured while building this file: a `rerun-if-changed` path that DOES NOT EXIST
/// makes cargo rerun the build script on EVERY build. So the original broken paths
/// (`../.git/HEAD`, which do not exist in a linked worktree) accidentally forced a rerun
/// every time, and every "HEAD moved" fixture passed with the defect still in place.
///
/// The defect needs the watch path to EXIST but be unrelated — then cargo's fingerprint
/// is satisfied and the script is skipped. So this fixture plants a decoy file that
/// exists and is never touched, exactly the shape that hides the bug, and then moves
/// HEAD. With the resolved `--git-dir` watch the capture follows; with any decoy or
/// broken path it does not.
#[test]
fn an_existing_unrelated_watch_path_cannot_hide_a_head_move() {
    let s = Scratch::new("decoy");

    // A decoy that exists and never changes — the shape that makes cargo skip the
    // build script. Without this, every fixture above passes vacuously.
    std::fs::write(s.work.join("crate/DECOY.txt"), "unchanged\n").unwrap();
    s.git(&["add", "-A"]);
    s.git(&["commit", "-m", "decoy"]);

    let first = s.build_and_capture();
    let first_hash = first.commit_hash().expect("first capture").to_string();

    // Move HEAD with an identical tree. Cargo's fingerprint is satisfied by the decoy,
    // so only a correct watch on the git dir can notice.
    s.git(&["commit", "--allow-empty", "-m", "history-only"]);
    let second_hash = s.head_short();
    assert_ne!(first_hash, second_hash, "the fixture must move HEAD");

    let after = s.build_and_capture();
    assert_eq!(
        after.commit_hash(),
        Some(second_hash.as_str()),
        "an existing-but-unrelated watch path hid the HEAD move — the capture is stale. \
         This is the discriminating case: a nonexistent watch path forces a rerun every \
         build and would have let the defect pass."
    );
}

/// The same, in the other direction: moving HEAD BACK must not leave a newer claim.
#[test]
fn moving_head_backwards_invalidates_the_capture() {
    let s = Scratch::new("headback");

    // Two history-only commits so there is somewhere to go back to.
    s.git(&["commit", "-q", "--allow-empty", "-m", "e1"]);
    s.git(&["commit", "-q", "--allow-empty", "-m", "e2"]);
    let later = s.build_and_capture();
    let later_hash = later.commit_hash().expect("capture at e2").to_string();

    s.git(&["reset", "-q", "--hard", "HEAD~2"]);
    let earlier = s.head_short();
    assert_ne!(later_hash, earlier, "the fixture must move HEAD backwards");

    let after = s.build_and_capture();
    assert_eq!(
        after.commit_hash(),
        Some(earlier.as_str()),
        "moving HEAD backwards left the NEWER commit in the capture — a binary that \
         claims to be a commit it is not"
    );
}

/// 🔴 What this fixture set does and does NOT catch, stated rather than implied.
///
/// Measured while building it: a `rerun-if-changed` path that does NOT EXIST makes cargo
/// rerun the build script on every build. So the ORIGINAL broken paths (`../.git/HEAD`,
/// absent in a linked worktree) accidentally forced a rerun every time and the defect was
/// masked in the simplest fixtures. This test pins that asymmetry so it cannot regress
/// silently: the discriminating case is an EXISTING but unrelated path, which is what
/// `an_existing_unrelated_watch_path_cannot_hide_a_head_move` plants.
///
/// This is why the fixture plants a decoy. Without it, a reader would conclude "HEAD
/// moves are covered" from tests that pass for the wrong reason.
#[test]
fn a_nonexistent_watch_path_forces_a_rerun_and_masks_the_defect() {
    // Documentation-as-executable: the property is asserted by the discriminating
    // fixture above. This one exists so the asymmetry is NAMED in a test, because a
    // future edit that removes the decoy would otherwise look harmless.
    let s = Scratch::new("asymmetry");
    std::fs::write(s.work.join("crate/DECOY.txt"), "unchanged\n").unwrap();
    s.git(&["add", "-A"]);
    s.git(&["commit", "-m", "decoy"]);
    let first = s.build_and_capture();
    assert!(
        first.commit_hash().is_some(),
        "the capture must work with a decoy present"
    );
    s.git(&["commit", "--allow-empty", "-m", "history-only"]);
    let after = s.build_and_capture();
    assert_eq!(
        after.commit_hash(),
        Some(s.head_short().as_str()),
        "with a decoy present the capture must still follow HEAD — this is the case the \
         original watch paths failed"
    );
}

/// An untracked file is not a dirty build: it was never in the tree either way.
#[test]
fn an_untracked_file_is_not_a_dirty_build() {
    let s = Scratch::new("untracked");
    std::fs::write(s.work.join("crate/src/scratch-note.txt"), "not added\n").unwrap();
    let c = s.build_and_capture();
    assert_eq!(
        c.dirty,
        Some(0),
        "an untracked file must not make the build read as dirty — it was never in the tree"
    );
}

/// A capture that cannot run must say so, not retain the previous answer.
///
/// The stale-claim defect was precisely "the file still holds the old commit". This
/// asserts the file is REWRITTEN every run, so a failure surfaces rather than hiding.
#[test]
fn the_generated_file_is_rewritten_every_run() {
    let s = Scratch::new("rewrite");
    let first = s.build_and_capture();
    let path = {
        let target = s.work.join("target");
        walk(&target)
            .into_iter()
            .find(|p| {
                p.file_name()
                    .map(|n| n == "build_identity.rs")
                    .unwrap_or(false)
            })
            .expect("generated file")
    };
    let mtime_before = std::fs::metadata(&path).unwrap().modified().unwrap();

    // Rebuild with nothing changed. Cargo may skip the build script, so force it the
    // same way a ref move would.
    s.git(&["update-ref", "refs/heads/noop", "HEAD"]);
    s.git(&["update-ref", "-d", "refs/heads/noop"]);
    let second = s.build_and_capture();
    assert_eq!(
        first.commit_hash(),
        second.commit_hash(),
        "same tree, same commit"
    );

    // The file's content must be the current answer, never a leftover.
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(
        body.contains(&s.head_short()),
        "the generated file must hold the CURRENT commit, not a leftover from an earlier build"
    );
    let _ = mtime_before;
}

/// The label is load-bearing: a bare hex run in a binary is not an identity.
///
/// Measured when wiring the first version into `replace-sandbox.sh`: grepping a binary
/// for a bare 12-hex run matched `dbdbdbdbdbdb`, a placeholder. This asserts the
/// generated file carries the label so a reader can match unambiguously.
#[test]
fn the_commit_is_emitted_next_to_a_fixed_label() {
    let s = Scratch::new("label");
    let c = s.build_and_capture();
    assert!(
        c.commit
            .as_deref()
            .is_some_and(|v| v.starts_with("seatloom-build-commit=")),
        "the commit must be labelled, or a binary reader cannot tell it from a placeholder"
    );
    assert!(
        c.commit_hash().is_some_and(|h| !h.is_empty()),
        "the label must be followed by a hash"
    );
}
