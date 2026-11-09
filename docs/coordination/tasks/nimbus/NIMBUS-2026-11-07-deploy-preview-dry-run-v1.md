# Deploy preview: `--dry-run` on the verification build script

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task-packet |
| id | NIMBUS-2026-11-07-deploy-preview-dry-run-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | scripts/prepare-verification-build.sh, scripts/test-prepare-verification-build-dry-run.sh |
| supersedes | |
| tags | dry-run, deploy-preview, verification-build, seatloom-scripts, acceptance |

## What changed

`scripts/prepare-verification-build.sh` gained `--dry-run`. Every step of the real
path is irreversible (pkill, rm -rf, cp -R, tauri build, restart), and before
clicking Accept the operator's only question is "what will this touch". Dry run
computes and prints that, and executes none of it.

It prints the source branch and commit; whether the worktree is dirty (and, if so,
lists the changes and says the build would contain more than that commit); the
target app path; the processes it would stop, with pids; the paths it would write;
the seven actions in real order; and what needs human approval.

The conclusion is a verdict, not a blessing. A missing tauri binary, a missing
launch script, or a dirty worktree each exit non-zero and do NOT print
"prerequisites met". The launch script matters specifically because that is where
the operator identity comes from — installing without it produces a build whose
Accept cannot work.

## A pre-existing defect found and fixed

`WROOT` was hardcoded to `"$REPO/../seatloom-worktrees"`. That is right from the
main checkout and WRONG from a seat's own worktree: there `$REPO/..` is already
`seatloom-worktrees`, so the path became `seatloom-worktrees/seatloom-worktrees`.
Measured: both call shapes failed with "worktree that checks out <branch> not
found". I classify it as a USABILITY defect, not a safety one, because it fails
closed with a clear message and never mis-installs. The resolution now tries both
shapes and picks the one that actually holds seat worktrees. Strictly additive;
the main-checkout behaviour is unchanged.

## One of my own

In the dry-run block, `$WT_COMMIT。` — a variable immediately followed by a
full-width full stop — made bash treat the multi-byte punctuation as part of the
name under `set -u`, reporting `WT_COMMIT?: unbound variable`. That is the exact
trap AGENTS.md warns about and I walked into it anyway. Braced as `${WT_COMMIT}`.

## The test

`scripts/test-prepare-verification-build-dry-run.sh`, seven groups, fixtures all in
a temp directory: it never touches the main checkout, the main index, or the
running app.

The load-bearing property is not "it printed something" but "those commands did
not run once". The fixture's fake tauri/pnpm/launch each append to `executed.log`,
and after a dry run that file must not exist.

Assertion 7 is the mutation: making the dry-run guard always false sends
`--dry-run` down the real path, and the suite exits 1 failing on exactly that
assertion. So the check is not guarding an empty room.

Also tested: unknown args exit non-zero; a second `--dry-run` exits non-zero
rather than being silently ignored; a missing launch script exits non-zero and
does not print "prerequisites met"; a dirty worktree lists the changes and blocks
install; a clean one passes.

## Gates, exit codes not implications

- `cargo build --workspace` → 0
- `cargo test --workspace` → 367 passed, 0 failed
- `cargo clippy --workspace --all-targets -- -D warnings` → 3 errors, all in
  TRACK'S OWN files (`src-tauri/src/state.rs:18` from commit 61a32bc`)
- `cargo fmt --all -- --check` → DIRTY, only track's own
  `crates/seatloom-core/tests/pipe_guard_real_steal.rs`
- `scripts/test-prepare-verification-build-dry-run.sh` → exit 0

This change contains no Rust, so those two reds are not mine. I did not quietly
fix another seat's files inside a sync merge — that would make the diff lie about
what it does.

## What was not done

Nothing merged into track, nothing deployed, the running app untouched (pid 90805
before and after), no live-DB writes. The branch is separate
(`packet/nimbus/deploy-preview-dry-run`) from the deletion-guard work and its card.

## Acceptance, operator view

Run `bash scripts/prepare-verification-build.sh --dry-run <branch>`. It prints the
exact planned scope, and the running app and its processes do not change at all.
An actual rebuild and install requires human approval, and dry run does neither.
