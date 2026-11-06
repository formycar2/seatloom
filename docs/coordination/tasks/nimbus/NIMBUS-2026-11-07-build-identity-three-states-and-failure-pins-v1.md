# NIMBUS-2026-11-07-build-identity-three-states-and-failure-pins-v1

- seat: `seat-nimbus-001`
- dispatch: `disp-l6G-5OqExN38XUlRL_oLa`
- branch: `packet/nimbus/build-identity-three-states` (from `5d66aab`)

## What changed

**The defect.** `git_in` ended with `.filter(|s| !s.is_empty())`. `git status --porcelain`
on a clean tree emits zero bytes, so the filter discarded a successful, informative
answer and returned `None` — the same value used for "could not read". The generated
file said `BUILD_DIRTY = None` with `BUILD_GIT_OK = true`, and the banner rendered
"build-time dirty state not recorded" on **every clean build**. The operator was told the
build could not say whether it was clean, when it had looked and found nothing.

Three states had collapsed into two, in the opposite direction to the one the dispatch
names, but the same defect class.

**The fix** moves the emptiness decision out of the shared helper and into each caller,
because whether an empty answer is meaningful depends on the subcommand: empty for
`status` is "clean", empty for `rev-parse` is "went wrong". A shared filter cannot make
that distinction, and having one there made it silently for every caller. `dirty_count`
now turns a successful empty read into `Some(0)`; the commit and watch-path reads keep
their own filter, because a hash or a directory path is never legitimately empty.

**The three UI states are now pinned** (`Some(0)` / `Some(n>0)` / `built_git_ok === false`),
including a pairwise-distinct assertion, so two of them rendering the same on screen is
the failure.

**The failed write is now pinned.** `R-3` showed that replacing the panicking write with
`let _ = ...` left the suite green. A new fixture makes the generated file read-only,
forces the build script to rerun, and asserts the build fails and names the write.

## The fixture that was lying

The clean-tree test initially passed with the defect restored. The reason is the
important part: the scratch repository had **no `.gitignore`**, so `Cargo.lock` and
`target/` were always untracked and `git status --porcelain` always had output. The
filter therefore never had an empty string to discard, and the mutation had nothing to
bite on. The fixture had built the wrong state — a tree that is never clean.

Adding a `.gitignore` for `/target` and `Cargo.lock`, matching the real repository's
shape, made the mutation bite and the test fail for the right reason. Verified both
directions: with the defect, `a_clean_tree_is_some_zero_not_none` and
`dirty_state_is_stated_by_the_binary_and_recovers` both fail; with the fix, 15 pass.

## Two other traps I hit, recorded because they nearly shipped

- I briefly introduced `arg("--C")` where the code had `arg("-C")`. Every git read then
  failed, and my earlier "verification" had read a **stale `OUT_DIR` artifact** — the
  exact trap the verdict names. Fixed, and every capture since is taken after removing
  the generated files first.
- `run()` in the test harness asserts success, so using it in a test whose point is that
  a command fails would panic before the assertion. Added `run_outcome()`.
- Making the directory read-only does not make `fs::write` fail — the file already
  exists, so it is opened for truncation, which needs write permission on the **file**.

## Verification

- `cargo build --workspace` — 0 errors.
- `cargo test --workspace` — **388 passed, 0 failed**.
- `build_identity_incremental` — 15 passed, including all previously-passing incremental,
  linked/packed/detached worktree cases.
- Mutation, the original `!s.is_empty()` filter: **2 fixtures red.**
- Mutation, silent write (`let _ =`): **1 fixture red.**
- Mutation, `built_git_ok === false` rendering clean: **1 UI test red.**
- UI suite — 398 passed, 31 files.
- `rustfmt --check` clean on both files I touched. The one remaining `cargo fmt --check`
  diff is `src-tauri/src/commands/history_cmds.rs`, which is track-owned and untouched.

## What I am not claiming

- No install, no restart, no sandbox or main-app replacement, no live database, no card.
- The commit-capture filter and the watch-path filter are unchanged in behaviour; the
  change is that they now live at the call site instead of in the helper. I have not
  added a fixture that proves an empty `rev-parse` becomes `None`, only that a
  successful one is captured.
