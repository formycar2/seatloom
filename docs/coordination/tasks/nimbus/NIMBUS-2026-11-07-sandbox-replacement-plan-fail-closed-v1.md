# NIMBUS-2026-11-07-sandbox-replacement-plan-fail-closed-v1

- seat: `seat-nimbus-001`
- dispatch: `disp-gAUl7m-XmuvX-zQ-lVj3W`
- branch: `packet/nimbus/sandbox-plan-failclosed` (from `5d66aab`)

## What changed

`scripts/replace-sandbox.sh` no longer permits a fail-open path. Aegis held the operator
request at `21cf1e9` and named the reasons; all of them were real, and all of them are
closed here.

**1. A missing build identity blocked instead of warned.** The script found no commit
label in the binary, printed `⚠️ not a failure, just not verified`, and continued. The
one fact that proves the right thing is being installed could pass while entirely
unknown. Now it fails, and it says why.

**2. A git status read failure no longer reads as clean.** The same defect I fixed in
`prepare-verification-build.sh` this morning had recurred here verbatim — one pipeline
plus `|| true`, so git's exit 128 was swallowed by grep's no-match. Fixed the same way:
stderr to a temp file, exit code read directly, and unreadable is a third state that
never enters a binary `[ -n "$DIRTY" ]`.

**3. A dirty source tree is a blocker, not a remark.** It printed `⚠️` and replaced
anyway, so "this is commit X" was false at the exact moment it needed to be true.

**4. A missing build no longer points at `--apply`.** The failure message told the
operator to run `prepare-sandbox-build.sh <branch> --apply`, which does
`rm -rf` + `cp -R` on the sandbox with no backup at all — a script whose whole premise
is "back up before you destroy" was routing around itself. It now points at a new
`--build-only` mode that builds and verifies the commit, then stops: no install, no
replace, no process touched.

**5. Backup metadata describes the OLD artefact.** It recorded the *incoming* branch and
commit, which is information about the thing being installed, not the thing being
recoverable. A rollback must answer "what do I get back". It now records the old
sandbox's own commit (read from its binary label) and states plainly what the directory
is.

**6. The backup is verified before anything is destroyed**, and `$DEST` is created
before `cp` — the first version relied on `cp -R` creating the parent, so a failed
`mkdir` looked identical to a failed copy.

**7. Every destructive step fails closed.** `kill` that does not stop, the executable
rename, both `plutil` edits, and a missing `plutil` each stop the script instead of
printing a warning and continuing. A warning after a destructive step is not a
safeguard.

**8. Rollback preserves the pre-rollback state.** Rollback also does `rm -rf`, so it
now backs up the currently-running sandbox first. If the restore copy fails, the
pre-rollback build is put back and the script says the system is where it was.

**9. Sandbox and main paths cannot collide.** Both are overridable by environment
variable, and a collision means deleting the main app. Checked as strings and again via
`realpath`, before anything moves.

## Verification

`scripts/test-replace-sandbox-failclosed.sh` — 31 checks, exit 0. Fixtures with fake
apps, git, `pgrep`, and `open` in a temp directory. The load-bearing assertion is not
"it printed a warning" but **zero side effects plus a nonzero exit** — the fake `pgrep`
and `open` write a log, and any log entry fails the case.

- Baseline: clean, identity matches → exit 0, zero side effects.
- Missing identity / mismatched identity / unreadable git / dirty tree → each nonzero,
  each says why, each with zero side effects.
- Missing build → nonzero, and the guidance contains no `--apply`.
- Sandbox path == main path → nonzero, main app verified still present.
- Rollback preview → exit 0, zero side effects, no sandbox materialised, and it shows
  the backup's commit so "what do I get back" is answerable. A metadata-less backup is
  refused.
- Three mutations, each restoring one of the original fail-open shapes: all three
  reproduce exit 0, proving the assertions are not green by luck.

The real main app, the real sandbox, and the real backup root are untouched by the
whole run — verified after it.

## Bugs my own test had, which is why it exists

Every one of these made a correct script look broken, or a broken assertion look green:

- The fake `git` shifted off `-C <dir>` before `exec`, so the real git ran in the wrong
  directory and "worktree not found" was reported for five cases at once.
- `fail` was called before it was defined in the edited script.
- The fixture binary lacked the `MUST_HAVE_ME` string, so the baseline failed on an
  unrelated check.
- `[ -d ]` on a regular file — an assertion that can never pass.
- The mutation's failing state was set *after* the first run, so the mutation was never
  exercised and reported "the assertion guards nothing".
- The loop's restore dropped `MUST_HAVE_ME`, poisoning the next case into failing on an
  unrelated check.
- The `cmp` restore check compared the file to a copy just made from it — vacuously
  true. It now compares to the repository's own copy.
- The commit-missing mutation only replaced one `fail` while a second guard still
  caught the empty string, so the mutation was stopped by the wrong mechanism and
  looked like it had not reproduced.

## What I am not claiming

- **No replacement has been executed.** No sandbox replaced, no card opened, no app
  installed, no live database touched.
- **The dry-run suite is not on this branch.** `scripts/test-prepare-verification-build-
  dry-run.sh` lives on `packet/nimbus/dry-run-dirty-fixture`, which this branch does not
  contain, so I cannot and do not claim it passes here. It tests
  `prepare-verification-build.sh` and mentions neither `replace-sandbox.sh` nor
  `prepare-sandbox-build.sh` — I ran it on its own branch, unmodified by me, and it
  exits 0. That is a measurement of a sibling branch, not of this one.
- `prepare-sandbox-build.sh --apply` is still unbacked. That path is unchanged apart
  from the new `--build-only` mode and the mutual exclusion of the two flags. Replacing
  a sandbox must go through `replace-sandbox.sh`.
- I have not driven the `--apply` path of either script, and the failure-injection
  fixtures do not cover the `plutil` or `open` failures on a real bundle. Those are
  fail-closed by construction; they are not proven by execution.
