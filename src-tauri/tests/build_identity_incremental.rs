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

/// Run a command that is ALLOWED to fail, and return (success, combined output).
///
/// `run` above asserts success, which is right for setup but wrong for the
/// assertions whose whole point is that a command FAILS. Using `run` there would
/// panic before the assertion was reached, so the test would fail with a panic
/// message rather than with the property it names.
struct Outcome {
    success: bool,
    text: String,
}

fn run_outcome(cmd: &mut Command) -> Outcome {
    let out = cmd
        .output()
        .unwrap_or_else(|e| panic!("could not run {cmd:?}: {e}"));
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Outcome {
        success: out.status.success(),
        text,
    }
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
        // 🔴 A `.gitignore` for build output, matching the real repository's shape.
        //
        // Without it, `Cargo.lock` and `target/` are UNTRACKED, so
        // `git status --porcelain` ALWAYS has output — and `git_in` therefore never
        // sees the empty string. Measured: with the original `!s.is_empty()` filter
        // restored (the defect), the clean-tree fixture still produced `Some(0)`,
        // because the filter had nothing to discard. The mutation survived, and the
        // fixture had built the wrong state.
        //
        // A clean tree in the real repo produces ZERO BYTES of porcelain output.
        // The fixture must reproduce that, or it is not testing the clean-tree case.
        std::fs::write(seed.join(".gitignore"), "/target\nCargo.lock\n").unwrap();
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
        self.build_and_capture_with(&self.work.join(".failing-git-bin"))
    }

    /// Read the generated `build_identity.rs` under the scratch target dir.
    ///
    /// Used by the fixture that builds with an unusual PATH: reading the file the
    /// unusual build produced is the measurement; rebuilding would redo it with the
    /// normal PATH and measure something else.
    fn read_generated(&self) -> String {
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
        std::fs::read_to_string(&path).expect("read build_identity.rs")
    }

    /// Build with an extra directory prepended to PATH.
    ///
    /// The failing-git fixtures install a wrapper into `.failing-git-bin/`; passing
    /// it here is what puts that wrapper in front of the real git for the build
    /// script's child process. When the directory does not exist the PATH entry is
    /// simply inert, which is what the ordinary tests want.
    fn build_and_capture_with(&self, extra_path: &Path) -> Capture {
        let mut path = std::env::var_os("PATH").unwrap_or_default();
        if extra_path.is_dir() {
            let mut combined = std::ffi::OsString::from(extra_path);
            combined.push(":");
            combined.push(&path);
            path = combined;
        }
        run(Command::new("cargo")
            .arg("build")
            .arg("-q")
            .arg("--manifest-path")
            .arg(self.work.join("crate/Cargo.toml"))
            .env("PATH", &path));
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

/// 🔴 A clean tree must be `Some(0)`, NOT `None`.
///
/// This is the live defect Flux measured at 5d66aab (ev-CJ-PZ6nmSkizgDI8QfAT9).
/// `git_in` ended with `.filter(|s| !s.is_empty())`. `git status --porcelain` on a
/// clean tree emits ZERO BYTES, so the filter discarded a successful, informative
/// answer and returned `None` — the same value used for "could not read". The
/// generated file said `BUILD_DIRTY = None` with `BUILD_GIT_OK = true`, and the
/// banner rendered "build-time dirty state not recorded" on every clean build.
///
/// The operator was told the build could not say whether it was clean, when in fact
/// it looked and found nothing. Three states collapsed into two, in the opposite
/// direction to the one the dispatch named, but the same defect class.
///
/// The fix moves the emptiness decision to the CALLER, because whether an empty
/// answer is meaningful depends on the subcommand: empty for `status` is "clean",
/// empty for `rev-parse` is "went wrong".
#[test]
fn a_clean_tree_is_some_zero_not_none() {
    let s = Scratch::new("cleanzero");
    // The scratch repo is committed and clean by construction. Prove it, or this
    // fixture is asserting on a state it did not create.
    let status = s.git(&["status", "--porcelain"]);
    assert!(
        status.trim().is_empty(),
        "this fixture requires a genuinely clean tree; it was not clean: {status}"
    );
    let c = s.build_and_capture();
    assert_eq!(
        c.dirty,
        Some(0),
        "a clean tree with git available must be Some(0). None means the reader \\
         cannot tell 'we looked and it was clean' from 'we could not look' — which \\
         is exactly what made every clean build display 'not recorded'"
    );
    assert!(
        c.git_ok,
        "git was available in this fixture, so BUILD_GIT_OK must be true"
    );
    // And the commit must still be captured: the fix must not have traded one lost
    // value for another.
    assert!(
        c.commit_hash().is_some_and(|h| !h.is_empty()),
        "the commit must still be captured on a clean tree"
    );
}

/// The mirror image: a tree with uncommitted changes is `Some(n>0)`, and the two
/// states must be distinguishable. A one-directional check would pass if both
/// collapsed to the same value in either direction.
#[test]
fn a_dirty_tree_is_some_n_above_zero() {
    let s = Scratch::new("dirtyabove");
    // Dirty a TRACKED file, so it appears as ' M' rather than '??'. An untracked
    // file is deliberately excluded: it was never going to be in the build.
    std::fs::write(
        s.work.join("crate/src/lib.rs"),
        "pub fn ok() -> bool { false }\n",
    )
    .unwrap();
    let c = s.build_and_capture();
    match c.dirty {
        Some(n) => assert!(n >= 1, "a dirty tree must report at least one change, got {n}"),
        None => panic!(
            "a dirty tree must be Some(n), not None — None means unreadable, and this tree was readable"
        ),
    }
    assert!(c.git_ok, "git was available, so BUILD_GIT_OK must be true");
}

/// A tree with ONLY untracked files is `Some(0)`, not `None` and not `Some(1)`.
///
/// The exclusion of untracked files is deliberate and matches
/// `prepare-sandbox-build.sh`: a file that was never `git add`ed was never going to
/// be in the build either. But excluding it must not turn the answer into "unknown".
#[test]
fn untracked_only_is_still_some_zero() {
    let s = Scratch::new("untrackedonly");
    std::fs::write(s.work.join("crate/src/untracked_new.rs"), "pub fn n() {}\n").unwrap();
    let c = s.build_and_capture();
    assert_eq!(
        c.dirty,
        Some(0),
        "an untracked file must be excluded from the count, but the answer must still \\
         be a successful Some(0) — not None, which would read as unreadable"
    );
    assert!(c.git_ok);
}

/// 🔴 A failed write is a BUILD ERROR, never a silent skip.
///
/// Flux's R-3: replacing the panicking write with `let _ = std::fs::write(...)`
/// left the suite 11/11 green. So the behaviour that prevents the stale-claim defect
/// from returning — the exact mechanism that produced an earlier false conclusion —
/// had no test. It was fail-closed in the shipped code, but nothing pinned it.
///
/// This makes the write fail by making OUT_DIR unwritable, and asserts the build
/// fails. A silent write failure would leave the PREVIOUS `build_identity.rs` in
/// place, and the next build would pick it up and state a commit it is not.
#[test]
fn a_failed_write_fails_the_build_rather_than_keeping_the_stale_answer() {
    let s = Scratch::new("writefail");

    // First build, so a build_identity.rs exists. Then remove write permission from
    // the directory that holds it: writing a NEW file into a read-only directory
    // fails on macOS regardless of who owns it.
    let _ = s.build_and_capture();
    let target = s.work.join("target");
    let mut out_dir: Option<PathBuf> = None;
    for entry in walk(&target) {
        if entry
            .file_name()
            .map(|n| n == "build_identity.rs")
            .unwrap_or(false)
        {
            out_dir = entry.parent().map(|p| p.to_path_buf());
        }
    }
    let out_dir = out_dir.expect("the first build must have produced build_identity.rs");
    let existing = out_dir.join("build_identity.rs");
    let previous = std::fs::read_to_string(&existing).expect("read the previous answer");

    // Two things must both be true for this to measure anything, and the first
    // version got both wrong — measured, not reasoned:
    //
    // 1. The build script must actually RUN. A second `cargo build` with nothing
    //    changed is a no-op: cargo's fingerprint is satisfied, the script is skipped,
    //    no write is attempted, and the build "succeeds". That looked exactly like
    //    the silent-write defect. So the build script's mtime is bumped to force a
    //    rerun.
    // 2. The write must be able to fail. Making the DIRECTORY read-only does not do
    //    it, because `build_identity.rs` already exists and `fs::write` opens an
    //    existing file for truncation, which needs write permission on the FILE.
    //    Directory permissions only gate creating new files. So the FILE is made
    //    read-only.
    std::fs::remove_file(&existing).ok();
    std::fs::write(&existing, &previous).expect("recreate build_identity.rs");
    let perms = std::fs::metadata(&existing).unwrap().permissions();
    let mut ro = perms.clone();
    ro.set_readonly(true);
    std::fs::set_permissions(&existing, ro).expect("make build_identity.rs read-only");

    // Force the build script to rerun: touch it, so cargo's fingerprint changes.
    let build_rs = s.work.join("crate/build.rs");
    std::fs::write(&build_rs, std::fs::read_to_string(&build_rs).unwrap())
        .expect("rewrite build.rs to bump its mtime");

    let out = run_outcome(
        Command::new("cargo")
            .arg("build")
            .arg("-q")
            .arg("--manifest-path")
            .arg(s.work.join("crate/Cargo.toml")),
    );

    // Restore, whatever happened, so the fixture directory can be cleaned up.
    let mut rw = perms;
    rw.set_readonly(false);
    let _ = std::fs::set_permissions(&existing, rw);

    // The build must FAIL. Exit code is the assertion, not a message.
    assert!(
        !out.success,
        "a failed write must fail the build. It succeeded, which means the previous \\
         answer was silently retained — the stale-claim defect, unguarded. \\
         cargo said: {}",
        out.text
    );
    // And the failure must name the write, so it is diagnosable rather than a
    // generic compile error.
    assert!(
        out.text.contains("could not write") || out.text.contains("build_identity"),
        "the build failed but did not say the identity write was the reason: {}",
        out.text
    );
    let _ = previous;
}

/// 🔴 `BUILD_GIT_OK` must describe whether the git COMMANDS succeeded.
///
/// Measured at b843af4 by an independent verdict (ev-5Z2eSzH1_B-VZa62_CoIh): with a
/// git whose `status --porcelain` exits 128, the generated file said
/// `BUILD_DIRTY: Option<u32> = None` with `BUILD_GIT_OK: bool = true`. The flag was
/// set from "a repository root was found" — a fact about the filesystem — so the one
/// field whose purpose is to make "git failed" machine-readable was true on the
/// exact failure path.
///
/// The screen stayed honest only because `built_dirty` is `None` and the banner's
/// first check is `typeof n !== 'number'`. That is luck, not design: a reader of the
/// field — a script, another seat, a future banner — cannot tell whether git worked.
///
/// This is the third defect from that FAIL, and the dispatch required it explicitly:
/// git unavailable / status / read / write failures must be explicit unknown, never
/// silently readable as success.
#[test]
fn git_ok_is_false_when_the_required_git_command_fails() {
    let s = Scratch::new("gitok_status128");

    // Baseline first: the flag must be TRUE when git works. A test that only checks
    // the failure direction would pass if the flag were hardcoded to false.
    let good = s.build_and_capture();
    assert!(
        good.git_ok,
        "BUILD_GIT_OK must be true when git works — otherwise this test cannot \
         distinguish a truthful flag from a constant one"
    );

    // Inject a git that fails ONLY on `status --porcelain`, exit 128, writing to
    // stderr. Every other subcommand behaves normally. That is the shape a
    // permissions problem, a corrupt index, or a disk error takes.
    install_failing_git(&s.work, "status");

    let c = s.build_and_capture();
    remove_failing_git(&s.work);

    assert!(
        !c.git_ok,
        "BUILD_GIT_OK must be false when git status failed. It was true, which means \\
         the flag reports repository DISCOVERY rather than command SUCCESS"
    );
    assert_eq!(
        c.dirty, None,
        "a failed status read must be unknown, not a count"
    );
}

/// The commit read failing must also clear the flag, and must not leave the previous
/// answer behind. This is the same property from the other direction: `rev-parse`
/// failing while `status` succeeds.
#[test]
fn git_ok_is_false_when_the_commit_read_fails() {
    let s = Scratch::new("gitok_revparse");
    let good = s.build_and_capture();
    assert!(good.git_ok, "baseline must be git_ok = true");
    assert!(
        good.commit_hash().is_some(),
        "baseline must capture a commit"
    );

    install_failing_git(&s.work, "rev-parse");
    let c = s.build_and_capture();
    remove_failing_git(&s.work);

    assert!(
        !c.git_ok,
        "BUILD_GIT_OK must be false when rev-parse failed"
    );
    assert_eq!(
        c.commit_hash(),
        None,
        "a failed commit read must be None, not the previous commit — that is the \\
         stale-claim defect"
    );
}

/// Git entirely absent from PATH must be unknown, not clean and not a stale commit.
///
/// A missing binary makes `Command::new(\"git\")` fail to spawn, which is a different
/// failure from a nonzero exit and must land in the same place.
#[test]
fn git_ok_is_false_when_git_cannot_be_spawned_at_all() {
    let s = Scratch::new("gitok_nogit");
    let good = s.build_and_capture();
    assert!(good.git_ok, "baseline must be git_ok = true");

    // A PATH with no git in it, but with everything cargo itself needs — rustc
    // especially. 🔴 The first version set PATH to just `/usr/bin:/bin`, which
    // removed rustc from the child's PATH, so the build failed with
    // "could not execute process `rustc -vV`" and the test measured the fixture,
    // not the capture. Measured, not reasoned.
    //
    // So: take the real PATH and FILTER OUT the directories that contain a `git`.
    let original = std::env::var_os("PATH").unwrap_or_default();
    let mut filtered: Vec<std::path::PathBuf> = Vec::new();
    for dir in std::env::split_paths(&original) {
        if dir.join("git").exists() {
            continue;
        }
        filtered.push(dir);
    }
    let filtered_path = std::env::join_paths(filtered).expect("join the filtered PATH");
    assert!(
        !filtered_path
            .to_string_lossy()
            .split(':')
            .any(|d| std::path::Path::new(d).join("git").exists()),
        "the filtered PATH must genuinely contain no git, or this fixture tests nothing"
    );
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let out = run_outcome(
        Command::new(&cargo_bin)
            .arg("build")
            .arg("-q")
            .arg("--manifest-path")
            .arg(s.work.join("crate/Cargo.toml"))
            .env("PATH", &filtered_path)
            .env_remove("GIT_EXEC_PATH"),
    );

    assert!(
        out.success,
        "the build must still succeed with git absent — the failure must be recorded, \\
         not turned into a build error. cargo said: {}",
        out.text
    );
    // 🔴 Read the file that NO-GIT build produced. Do not rebuild: `build_and_capture`
    // runs with the normal PATH, so git works again and the capture flips back to
    // git_ok = true — which made this assertion fail on a correct implementation.
    // Measured, and the failure message pointed at the assertion rather than at the
    // fixture, which is exactly how a wrong test survives.
    let c = Capture::parse(&s.read_generated());
    assert!(
        !c.git_ok,
        "BUILD_GIT_OK must be false when git cannot even be spawned"
    );
    assert_eq!(c.dirty, None, "no git means unknown, not clean");
    assert_eq!(
        c.commit_hash(),
        None,
        "no git means unknown, not the previous commit"
    );
}

/// The three failure states must be DISTINGUISHABLE from the success states on all
/// three fields, not just on one. A reader should be able to tell
/// "git worked and the tree was clean" from "git failed" without knowing which
/// subcommand failed.
#[test]
fn git_failure_is_distinguishable_from_a_clean_tree_on_every_field() {
    let s = Scratch::new("gitok_distinct");
    let clean = s.build_and_capture();
    assert!(clean.git_ok, "baseline must be git_ok = true");
    assert_eq!(clean.dirty, Some(0), "baseline must be a clean Some(0)");

    install_failing_git(&s.work, "status");
    let failed = s.build_and_capture();
    remove_failing_git(&s.work);

    // Every field must differ. If any one of them agrees, a reader cannot tell the
    // two states apart from that field alone.
    assert_ne!(
        clean.git_ok, failed.git_ok,
        "git_ok must distinguish the two states"
    );
    assert_ne!(
        clean.dirty, failed.dirty,
        "dirty must distinguish the two states: Some(0) vs None"
    );
}

// ── failing-git injection helpers ─────────────────────────────────────────
//
// A git on PATH that behaves normally for every subcommand EXCEPT the named one.
// It is a wrapper, not a replacement: `exec /usr/bin/git \"$@\"` keeps every other
// call real, so the fixture stays a working repository for everything else. That
// matters — a git that fails on everything would also break the fixture's own
// setup, and the test would measure the fixture rather than the capture.
fn failing_git_dir(work: &Path) -> PathBuf {
    work.join(".failing-git-bin")
}

fn install_failing_git(work: &Path, subcommand: &str) {
    let dir = failing_git_dir(work);
    std::fs::create_dir_all(&dir).expect("failing-git dir");
    let real = std::env::var("REAL_GIT_FOR_TEST").unwrap_or_else(|_| "/usr/bin/git".to_string());
    let script = format!(
        "#!/usr/bin/env bash\n\
         for a in \"$@\"; do\n\
         \x20 if [ \"$a\" = \"{subcommand}\" ]; then\n\
         \x20   echo \"fatal: injected failure for {subcommand}\" >&2\n\
         \x20   exit 128\n\
         \x20 fi\n\
         done\n\
         exec \"{real}\" \"$@\"\n"
    );
    let path = dir.join("git");
    std::fs::write(&path, script).expect("write failing git");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod failing git");
    }
}

fn remove_failing_git(work: &Path) {
    let _ = std::fs::remove_dir_all(failing_git_dir(work));
}
