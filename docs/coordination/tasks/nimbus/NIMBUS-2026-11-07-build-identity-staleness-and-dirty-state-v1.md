# NIMBUS-2026-11-07-build-identity-staleness-and-dirty-state-v1

- seat: `seat-nimbus-001`
- dispatch: `disp-vbAHzo5D9_1cGCuetFOFW`
- branch: `packet/nimbus/deletion-guard-only`

## What changed

`src-tauri/build.rs` now resolves its watch paths from git instead of hard-coding
them, and captures three facts instead of one. Two new fields reach the UI, which
states them instead of counting an unknown as clean.

1. **Watch paths** come from `git rev-parse --git-dir` and `--git-common-dir`,
   emitted as directories before any early return. The previous
   `../.git/HEAD` / `../.git/refs` do not exist in a linked worktree, where
   `.git` is a file.
2. **Dirty state** is captured as `BUILD_DIRTY: Option<u32>` — a three-state
   value, untracked files excluded — so a dirty build is distinguishable from a
   clean one by the binary's own account.
3. **`BUILD_GIT_OK`** records whether git could be read at all, so a failure is
   not reported as a clean build.
4. The generated file is rewritten unconditionally, and a write failure panics
   rather than retaining a stale claim.
5. `RuntimeIdentityDto` carries `built_dirty` / `built_git_ok`, and
   `SandboxBanner.tsx` states both, flagging a binary that disagrees with the
   installer.

## The defect I did not catch on the first attempt, and why it matters

The first version of this work had seven fixtures that all passed. With the fix
reverted, they **all still passed**. That is the only reason this note exists.

Three separate measurements, each of which invalidated the one before it:

**1. The reported old hash was a stale artifact, as Flux suspected.**
`build_identity.rs` held `34f8681c3856` while the worktree was at `21cf1e9a36d6`.
The reading came from the *release* artifact; forcing a rerun gave the correct
commit from the *debug* one. So the observed symptom was not the defect.

**2. A HEAD move does not rerun the build script at all — but only sometimes.**
With no watch path declared, cargo skips the script when nothing it tracks
changes. With a watch on the *real* git-dir HEAD, it reruns. So the fix is
correct.

**3. A nonexistent watch path forces a rerun on every build.**
This is what made the fixtures vacuous. Cargo treats a declared path that does
not exist as "always changed", so the *original broken paths* accidentally
forced a rerun every time — and every "HEAD moved" fixture passed with the
defect restored. Measured directly: broken paths restored, all fixtures green.

The discriminating shape is an **existing but unrelated** path: cargo's
fingerprint is satisfied, the script is skipped, and the capture goes stale. The
fixture plants a decoy file that exists and is never touched, which is that
shape.

The fixtures were also passing for a second, unrelated reason: this repository
has a pre-commit hook, and `git commit` inside a scratch repo ran it, failed, and
left HEAD unmoved. The "HEAD moved" assertions were comparing a commit to
itself. `git -c core.hooksPath=/dev/null --no-verify` fixes it, and each fixture
now asserts HEAD actually moved.

## Verification

- `cargo test -p seatloom-tauri --test build_identity_incremental` — **10 passed**
  (11 with the asymmetry fixture).
- Mutation, existing-unrelated watch path: **6 fixtures red.**
- Mutation, original broken paths: **all green** — recorded above as the trap, not
  claimed as coverage.
- Mutation, dirty capture discarded: **2 fixtures red.**
- Mutation, absent dirty count rendered as clean: **4 frontend tests red.**
- `cargo build --workspace` — 0 errors.
- Full UI suite — 394 passed, 31 files.

## What I am not claiming

- I have not reopened any acceptance card, replaced the sandbox, or touched the
  main app. No independent verdict names this commit yet.
- `ui/node_modules` is untracked, so the UI suite cannot run in a linked worktree
  without it. I symlinked it to the main worktree to run the tests, then removed
  the symlink. That is a local measurement step, not part of the change.
- The frontend fixtures use CJK literals; the notes here are English, as the docs
  gate requires.
