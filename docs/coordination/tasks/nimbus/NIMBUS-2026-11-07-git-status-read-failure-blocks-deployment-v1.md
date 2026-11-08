# A git status read failure blocks deployment instead of reading as clean

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-git-status-read-failure-blocks-deployment-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | disp-ccZ7ZSP0IWCRrQADHwrZU |
| supersedes | |
| tags | nimbus, delivery, git-status-honesty |

> Header added by Aegis on 2026-11-08 after the acceptance merge landed this file on
> `track/infra-foundation` without one. Every value below is copied from the file's own
> provenance lines — nothing is inferred. The doc gate refuses new docs without a
> universal header, and this file arrived through a merge rather than a commit of mine,
> so it would have been invisible to the gate until now.

- seat: `seat-nimbus-001`
- dispatch: `disp-ccZ7ZSP0IWCRrQADHwrZU`
- branch: `packet/nimbus/git-status-honesty` (bounded, from `b9060f7`)

## What changed

`scripts/prepare-verification-build.sh --dry-run` no longer reads a git failure as a
clean tree.

The line was:

```
DIRTY="$(git -C "$WT" status --porcelain 2>/dev/null | grep -v '^??' || true)"
```

Three things were wrong at once, and they compounded:

1. `2>/dev/null` discarded stderr — the only evidence that distinguishes "clean" from
   "I could not read it".
2. `|| true` made the whole pipeline report success, so git's exit 128 never reached
   the caller.
3. `grep -v '^??'` reporting no match (exit 1, "no untracked files") was
   indistinguishable from git failing (exit 128, "I read nothing").

With git exiting 128, `DIRTY` became the empty string, `[ -n "$DIRTY" ]` was false, the
`else` branch printed **"working tree clean — the build result is commit $WT_COMMIT"**,
and that branch never sets `BLOCKERS=1` — so the dry run exited **0** and told the
operator the build was installable. The build's relationship to the commit had never
been determined.

## The repair

The load-bearing rule: **git's exit status and grep's no-match are captured
separately.** stderr goes to a temp file rather than `/dev/null`, the exit code is read
directly, and the untracked-file filter only runs when git actually succeeded — so
`|| true` on it can no longer swallow a git failure.

Unreadable is a **third state**, not a second one. It is neither clean nor dirty, and it
must not fall into the `[ -n "$DIRTY" ]` binary. It sets `BLOCKERS=1` and prints git's
exit code and its own message, so the operator can tell a permissions problem from a
corrupt index from a disk error.

If `mktemp` itself fails, that is also unreadable (`GIT_STATUS_RC=99`), not clean.

## Verification

Reproduced first, with an isolated fixture: a `git` that behaves normally for every
subcommand except `status --porcelain`, which exits 128 with `fatal: unable to read
the index`. Before the fix: exit **0**, and "working tree clean". After: exit **1**,
"cannot determine the working-tree state", with the message and the exit code.

The fixture is now section 8 of `scripts/test-prepare-verification-build-dry-run.sh`,
with six assertions: nonzero exit, does not say clean, does not say "preconditions met",
names the third state, reproduces git's own message, and states the exit code.

Section 9 is the mutation that proves section 8 is not green by luck: it restores the
original broken shape and asserts the failure is swallowed again. Its Python anchor
asserts the anchor text exists — a mutation that cannot find its anchor must fail the
test, not silently become a test that always passes.

All previous criteria are preserved and still pass: P-2 (dirty tree), the >20-change
truncation marker, zero side effects, unknown arguments, the dry-run guard mutation.
32 checks, exit 0.

## Measured, not claimed

- Section 8 under the mutation: **three of its six assertions go red** (exit code, the
  "clean" claim, and the third state's name).
- The mutation's restore is byte-identical — verified with `cmp`, not asserted.
- No temp file leaks: `mktemp` output removed, count before and after a full run equal.
- No install, no restart, no live DB write, no card. The fixture is a temp directory
  with its own fake `tauri`, `pnpm`, and launch script; the running app (pid 90805) is
  never touched.

## What I am not claiming

- I have not verified the real `prepare-verification-build.sh` against a real broken
  git on a real worktree. The fixture injects a git into `PATH`; the injected git is a
  shape, not the real failure.
- The other git reads in the script (`rev-parse` at lines 137-139, the `branch --list`
  at 97) still use `2>/dev/null || echo '?'`. They substitute a visible `?` rather than
  a claim of cleanliness, so they fail differently — but they are not covered by a
  fixture, and I have not claimed otherwise.
