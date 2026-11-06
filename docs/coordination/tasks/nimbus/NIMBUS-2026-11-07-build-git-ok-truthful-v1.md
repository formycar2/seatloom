# NIMBUS-2026-11-07-build-git-ok-truthful-v1

- seat: `seat-nimbus-001`
- dispatch: `disp-4wghY-NA3m9HXs6A4Kj-w`
- branch: `packet/nimbus/build-git-ok-truthful` (from `b843af4`)

## What changed

`BUILD_GIT_OK` now means **the required git commands succeeded**, not "a repository root
was found".

At `b843af4` the flag was set from `root` being `Some(...)`. That answers a different
question: it says the workspace root exists, which is a fact about the filesystem. So a
git whose `status --porcelain` exits 128 produced `BUILD_DIRTY = None` with
`BUILD_GIT_OK = true` — the one field whose entire purpose is to make "git failed"
machine-readable was true on the exact failure path. Measured by an independent verdict
(`ev-5Z2eSzH1_B-VZa62_CoIh`), and it was the third defect from that FAIL.

The screen stayed honest only by luck: `built_dirty` is `None` and the banner's first
check is `typeof n !== 'number'`, so it rendered "not recorded". That is not design. A
reader of the field — a script, another seat, a future banner — could not tell from
`git_ok` whether git worked, and the DTO's own comment claimed it did.

It is now computed from the capture's own results: `root` present, the commit captured,
and the dirty count captured.

## Fixtures

Four new tests, and each asserts the **baseline** first, so none of them can pass by a
flag that is constant:

- `status --porcelain` exits 128 → `git_ok` false, `dirty` None. Baseline asserted true.
- `rev-parse` fails → `git_ok` false, `commit` None (not the previous commit).
- git cannot be spawned at all → `git_ok` false, both values None, and the build still
  succeeds rather than becoming a build error.
- Every field distinguishes a git failure from a clean tree, not just one.

The injection is a **wrapper**, not a replacement: a script on `PATH` that fails only the
named subcommand and `exec`s the real git for everything else. A git that failed on
everything would break the fixture's own setup, and the test would measure the fixture
rather than the capture.

## Two bugs in my own fixture, both of which measured the wrong thing

- The no-git case set `PATH` to `/usr/bin:/bin`, which removed **rustc**. The build
  failed with "could not execute process `rustc -vV`", so the test measured the fixture
  rather than the capture. It now takes the real PATH and filters out only the
  directories that contain a `git`, and asserts the filtered PATH really has none.
- The no-git case then called `build_and_capture()`, which rebuilds with the normal PATH
  — so git worked again and the capture flipped back to `git_ok = true`. The assertion
  failed on a correct implementation, and the failure pointed at the assertion rather
  than at the fixture. It now reads the file that the no-git build produced.

Both are recorded because a wrong test that fails on correct code is how a correct
implementation gets "fixed" into an incorrect one.

## Verification

- `cargo build --workspace` — 0 errors.
- `cargo test --workspace` — **392 passed, 0 failed**.
- `build_identity_incremental` — **19 passed**, including every previously-passing
  incremental, linked/packed/detached worktree case, and the `Some(0)` clean-tree case
  from the last round.
- Mutation (`git_ok` derived from repository discovery again): **4 fixtures red.**
- Mutation re-checked after `rustfmt`: still 4 red.
- UI suite — 398 passed, 31 files. No UI change was needed; the banner was already
  correct, which is why the screen was honest while the field was not.
- `rustfmt --check` clean on both files I touched. The remaining `cargo fmt --check` diff
  is `src-tauri/src/commands/history_cmds.rs`, track-owned and untouched.

## What I am not claiming

- No install, no restart, no sandbox or main-app replacement, no live database, no card.
- I have not verified the running binary, and `b843af4` remains unpinned on `origin`
  from the previous round (github unreachable).
- `git_ok` is computed from the two capture results. It does not attempt to distinguish
  *which* git command failed — the fixtures cover status, rev-parse and spawn failure
  separately, but the field itself is a single boolean, and a reader who needs to know
  which one failed still has to look at the other two fields.
