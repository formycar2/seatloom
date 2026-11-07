// The build-embedded identity: the commit this binary was compiled from, plus
// whether the tree it was compiled from was dirty, written into the executable at
// build time.
//
// WHY THIS EXISTS. The sandbox banner's branch and commit are read from environment
// variables that the *launcher* sets. That makes them a statement by the installer
// about what it launched, not a statement by the binary about what it is. Measured
// consequence: `strings` on the sandbox binary finds the four env var NAMES and six
// hex-looking strings, none of which is a hash of any commit on the branch — so
// nothing in the binary can answer "what are you?".
//
// This is the smallest change that makes an acceptance falsifiable. An operator who
// clicks Accept and sees a merge cannot currently tell whether the guard refused for
// the right reason or the merge happened for the right reason, because the running
// code cannot state its own commit. With this, the banner's commit comes from inside
// the binary and can be compared against the commit the card names.
//
// 🔴 WHAT FLUX FOUND IN THE FIRST VERSION (ev-OFHL9WBqlQnAqs8DCGAkN), both real:
//
//   DEFECT — a binary that states a commit it is not. `rerun-if-changed` declared
//   `../.git/HEAD` and `../.git/refs`. In a LINKED WORKTREE `.git` is a FILE
//   (`gitdir: .../worktrees/<name>`), so neither path exists; and a packed ref is
//   invisible to the `refs` path even in a normal checkout. Measured consequence:
//   after `git checkout` to another commit, a cargo build that DID rerun the build
//   script still produced `build_identity.rs` holding the PREVIOUS commit — exactly
//   the failure this file's own comment calls unacceptable, reachable in the
//   configuration this repository actually uses for verification work.
//
//   GAP — a dirty build is indistinguishable from a clean one at the same commit, by
//   the binary's own account. The uncommitted count the banner carries comes from the
//   installer's env var, which is the very channel this change exists to stop
//   trusting.
//
// Both are fixed here, and both fixes are asserted by executable fixtures rather than
// by a comment: see `src-tauri/tests/build_identity_incremental.rs`.
//
// WHAT IT DOES NOT CLAIM. A build-time capture describes the tree at compile time. It
// says nothing about the working directory of whoever launched it, and it is not a
// signature. If `git` is unavailable, or the directory is not a repository, the
// capture records that fact rather than retaining a previous answer.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The fixed prefix the captured commit is emitted next to.
///
/// A bare 12-hex-digit run in a binary is not an identity: measured, grepping for one
/// finds `dbdbdbdbdbdb` and other placeholders that are not commits. Anything reading
/// the value must match this LABEL, never the bare hex.
pub(crate) const BUILD_COMMIT_LABEL: &str = "seatloom-build-commit=";

/// Run `git` in `dir` and return stdout on success.
///
/// 🔴 `Some("")` and `None` are DIFFERENT FACTS and must stay different.
///
/// The first version ended with `.filter(|s| !s.is_empty())`, which threw away a
/// perfectly successful, perfectly informative answer. `git status --porcelain` on a
/// clean tree emits ZERO BYTES, so that filter turned "we looked and it was clean"
/// into `None` — the same value used for "we could not look". Measured at 5d66aab by
/// an independent verdict (ev-CJ-PZ6nmSkizgDI8QfAT9): the generated file said
/// `BUILD_DIRTY: Option<u32> = None` with `BUILD_GIT_OK: bool = true`, so every clean
/// build rendered "build-time dirty state not recorded" on screen. `Some(0)` was
/// reachable without the filter and unreachable with it.
///
/// So this helper reports exactly what git said. Whether an empty answer is
/// meaningful is the CALLER's decision, because it depends on the subcommand:
///   - `status --porcelain`  empty = clean (meaningful)
///   - `rev-parse --short`   empty = something went wrong (meaningless)
/// A shared filter cannot make that distinction, and applying one here silently
/// made it for every caller. The per-caller decisions are below.
fn git_in(dir: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

/// The directory cargo is building, which is the tree whose sources are compiled.
///
/// `CARGO_MANIFEST_DIR` is the crate's manifest directory; the workspace root is one
/// level up from `src-tauri/`. This is deliberately NOT the process CWD: a build
/// script may be invoked from anywhere, and reading the wrong directory would capture
/// the wrong tree while still succeeding.
fn workspace_root() -> Option<PathBuf> {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")?;
    let p = PathBuf::from(manifest);
    // src-tauri/ -> workspace root
    p.parent()
        .map(|parent| parent.to_path_buf())
        .filter(|p| p.is_dir())
}

/// Paths whose change must invalidate the capture, resolved through git itself.
///
/// 🔴 This is the fix for the staleness defect. The previous version hard-coded
/// `../.git/HEAD` and `../.git/refs`, which do not exist in a linked worktree (where
/// `.git` is a file) and miss packed refs everywhere. Instead, ask git for the two
/// directories that actually hold the state:
///
///   - `--git-dir`         : where HEAD lives. In a linked worktree this is
///                            `<common>/worktrees/<name>`, so a checkout that moves
///                            this worktree's HEAD changes a file under it.
///   - `--git-common-dir`  : where refs/ and packed-refs live, shared by every
///                            worktree. A `git branch -f`, a fetch, or a pack all
///                            land here.
///
/// Both are emitted as `rerun-if-changed` DIRECTORY paths. Cargo watches a directory
/// recursively for changes, so a new loose ref, an updated `packed-refs`, or a moved
/// HEAD all invalidate — without enumerating files that may not exist.
fn watch_paths(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for args in [
        ["rev-parse", "--git-dir"],
        ["rev-parse", "--git-common-dir"],
    ] {
        // `--git-dir` may be relative to the CWD, so resolve it against `root`.
        // A directory path is never empty, so an empty answer means the read went
        // wrong — filtered here for the same reason as the commit, not in the helper.
        if let Some(raw) = git_in(root, &args).filter(|s| !s.is_empty()) {
            let p = {
                let candidate = PathBuf::from(&raw);
                if candidate.is_absolute() {
                    candidate
                } else {
                    root.join(candidate)
                }
            };
            if p.is_dir() {
                out.push(p);
            }
        }
    }
    out
}

/// Whether the tree had uncommitted changes to TRACKED files at capture time.
///
/// Untracked files are excluded, matching `prepare-sandbox-build.sh`: a file that was
/// never `git add`ed was never going to be in the build either, so it is not what the
/// dirty clause is about. This is the BINARY's own statement — the point of the GAP
/// fix is that the banner must not have to trust the installer for this.
fn dirty_count(root: &Path) -> Option<u32> {
    let out = git_in(root, &["status", "--porcelain"])?;
    // `--porcelain` is stable and machine-readable; `^??` are untracked.
    //
    // 🔴 An EMPTY output is the answer "clean", not a failure. `git_in` no longer
    // discards empty stdout, so a successful read of a clean tree lands here as
    // `Some("")` and must become `Some(0)`. Collapsing it to `None` is what made the
    // banner claim "not recorded" on every clean build.
    let n = out
        .lines()
        .filter(|l| !l.trim_start().starts_with("??"))
        .count();
    Some(n as u32)
}

fn main() {
    // The workspace root is the tree whose sources cargo compiles.
    let root = workspace_root();

    // ── Invalidation, resolved through git rather than hard-coded ────────────
    // Emitted BEFORE any early return: if the capture later fails, the watch paths
    // are still registered, so the next build retries instead of reusing a stale
    // answer. Ordering matters here — the first version registered them only on the
    // success path.
    let watches = root.as_ref().map(|r| watch_paths(r)).unwrap_or_default();
    for w in &watches {
        println!("cargo:rerun-if-changed={}", w.display());
    }

    // ── Capture ─────────────────────────────────────────────────────────────
    // `None` for any part means "could not read", and it is recorded as such. A
    // capture failure must never leave the PREVIOUS answer in place: the file is
    // rewritten unconditionally on every run, so an IO failure below surfaces as a
    // build error rather than as a silently retained stale claim.
    //
    // 🔴 `BUILD_GIT_OK` means **the required git commands succeeded**, not "a
    //    repository was found".
    //
    //    The first version set it from `root` being `Some(...)`. That answers a
    //    different question: it says the workspace root exists, which is a fact about
    //    the filesystem. So a git whose `status --porcelain` exits 128 produced
    //    `BUILD_DIRTY = None` with `BUILD_GIT_OK = true` — the field whose entire
    //    purpose is to make "git failed" machine-readable was true on the exact
    //    failure path. Measured by an independent verdict at b843af4
    //    (ev-5Z2eSzH1_B-VZa62_CoIh).
    //
    //    The screen stayed honest, because `built_dirty` is `None` and the banner
    //    renders "not recorded". That is luck, not design: a reader of the field —
    //    a script, another seat, a future version of the banner — cannot tell from
    //    `git_ok` whether git worked. The DTO's own comment says it means "whether
    //    git answered at all during the capture", and it did not compute that.
    //
    //    So it is computed from the capture's own results. `dirty` is the required
    //    command whose answer carries the most states, so it is the load-bearing
    //    one; the commit is required too, and both must have answered.
    let root_present = root.is_some();
    let commit = root
        .as_ref()
        // A commit hash is never empty, so an empty answer here means the read went
        // wrong even though git exited 0 — that is `None`, not a commit. This is
        // the one caller where an empty string is meaningless, which is exactly why
        // the filter lives HERE and not in the shared helper.
        .and_then(|r| git_in(r, &["rev-parse", "--short=12", "HEAD"]))
        .filter(|s| !s.is_empty());
    let dirty = root.as_ref().and_then(|r| dirty_count(r));
    let git_ok = root_present && commit.is_some() && dirty.is_some();

    // The commit is emitted next to its label so a reader can match unambiguously.
    let commit_lit = commit
        .as_deref()
        .map(|c| format!("{BUILD_COMMIT_LABEL}{c}"))
        .unwrap_or_default();

    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    let dest = Path::new(&out_dir).join("build_identity.rs");
    // 🔴 `include!` cannot appear in expression position (`const X: T = include!(..)`
    // is a syntax error, because the generated file contains `pub const` — an item,
    // not an expression). So this writes a complete MODULE file, included at module
    // position by the consumer.
    let body = format!(
        "/// The commit this binary was compiled from, captured by `build.rs`.\n\
         ///\n\
         /// A literal: the value is compiled in, never read at runtime. A runtime `git`\n\
         /// call would make the banner depend on the launcher's working directory,\n\
         /// which is exactly what this replaces.\n\
         ///\n\
         /// The label is not decoration. Measured: grepping a binary for a bare\n\
         /// 12-hex-digit run finds `dbdbdbdbdbdb` and other placeholders that are not\n\
         /// commits. So the hash is emitted next to a fixed label, and anything reading\n\
         /// it must match the LABEL, never the bare hex.\n\
         pub const BUILD_COMMIT: Option<&str> = {commit_opt};\n\
         ///\n\
         /// Whether the tree had uncommitted changes to TRACKED files at capture time.\n\
         ///\n\
         /// 🔴 This closes the GAP Flux recorded: a dirty build was indistinguishable\n\
         /// from a clean one at the same commit, by the binary's own account. The count\n\
         /// the banner carries used to come from the installer's env var — the very\n\
         /// channel this whole change exists to stop trusting.\n\
         ///\n\
         /// Three states, deliberately not collapsed into a boolean: `Some(0)` means the\n\
         /// capture looked and the tree was clean; `None` means the capture could not\n\
         /// tell. Rendering those the same would let an unreadable state pass as clean.\n\
         pub const BUILD_DIRTY: Option<u32> = {dirty_opt};\n\
         /// Whether git answered at all. A capture that could not run must not be able\n\
         /// to read as \"we looked and it was clean\".\n\
         pub const BUILD_GIT_OK: bool = {git_ok};\n",
        commit_opt = match &commit {
            Some(_) => format!("Some({commit_lit:?})"),
            None => "None".to_string(),
        },
        dirty_opt = match dirty {
            Some(n) => format!("Some({n})"),
            None => "None".to_string(),
        },
        git_ok = git_ok,
    );

    // A failed write is a BUILD ERROR, never a silent skip. Retaining the previous
    // `build_identity.rs` after a failed capture is exactly the stale-claim defect;
    // failing the build is the honest direction.
    std::fs::write(&dest, body)
        .unwrap_or_else(|e| panic!("could not write {}: {e}", dest.display()));

    tauri_build::build()
}
