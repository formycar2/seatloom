# NIMBUS-2026-11-07-sandbox-flag-exclusion-pin-v1

- seat: `seat-nimbus-001`
- dispatch: `disp-0qqsqGE3XW6G8gk-xPMD6`
- branch: `packet/nimbus/sandbox-flag-exclusion-pin` (from `e238a04`)

## What changed

`scripts/test-replace-sandbox-failclosed.sh` only. The guard it pins is unchanged —
this is a coverage gap, not a defect.

Flux's `ev-B5dq_hybh4VZbA-dkGvbl` passed the replacement plan and left exactly one
item unpinned: `--apply` and `--build-only` given together. The two flags mean
opposite things — `--apply` replaces the sandbox, `--build-only` builds and stops — so
allowing both makes the outcome depend on which branch is evaluated later rather than
on what the operator asked for. The guard was present and correct; nothing tested it.

Section 9 now asserts three things: nonzero exit, the refusal says why, and **zero
destructive side effects** — which is also what proves the guard sits *before* the
`pnpm build` / `tauri build` / `rm -rf` / `cp -R` path rather than after it. A fourth
assertion runs `--build-only` alone and confirms it is not refused, so the guard cannot
"pass" by rejecting the flag outright.

Section 10 is the mutation: it removes the guard and asserts section 9 goes red. Its
anchor asserts the anchor text exists, so a mutation that cannot find its anchor fails
the test rather than becoming a test that always passes. The restore compares against
the repository's own copy, not a copy made inside the loop.

## The fixture, which took five attempts

Each attempt below exited nonzero, and every one of them would have let section 9 pass
for the wrong reason. They are listed because a reader should know how easily a
fail-closed test becomes a vacuous one:

1. `$FIX/repo` was not a git repository → "cannot resolve this ref".
2. The branch pointed at a different commit → "not the same tree".
3. The script copy was placed inside the worktree → `WROOT` found no worktrees.
4. `WROOT` was built as a *child* of `REPO`; the script computes it as a *sibling*
   (`$REPO/../seatloom-worktrees`).
5. The main checkout sat on the branch being added, so `git worktree add` refused and
   the worktree had no `.git` at all.
6. The worktree was left dirty by the fixture's own `target/` and `scripts/`, so the
   script stopped on "refuses to build from a dirty worktree" before reaching the guard.

The fixture now mirrors the main-checkout shape the script assumes: a main repo on a
different branch, a linked worktree checked out on the target branch, committed clean,
with the build artefact inside it. The dirty count is asserted to be zero before the
case runs, so a future edit that leaves the fixture dirty fails loudly instead of
silently changing what is measured.

## One assertion that was measuring the wrong thing

The zero-side-effect check originally treated *any* entry in the fake command log as
"it acted". `pgrep` is read-only and the script legitimately calls it to report whether
the main app is running — so the assertion was permanently red, and its redness proved
the opposite of what it claimed. It now counts destructive calls only (`PNPM`, `TAURI`,
`OPEN`, `PKILL`, `RM`, `CP`).

## Verification

- 36 checks, exit 0. Previously 31.
- Mutation (guard removed): section 9 red.
- All prior sections preserved and green: identity missing/mismatched/unreadable, dirty
  source, missing build pointing only at `--build-only`, sandbox/main path collision,
  rollback preview zero effects, metadata required, and the three original fail-open
  mutations.
- The real `/Applications/SeatLoom.app` is intact, no real backup directory was created,
  no temp directory leaked.
- No replacement executed, no install, no live database, no card.

## What I am not claiming

- I have not run `prepare-sandbox-build.sh --apply` for real, and nothing here proves
  the `--apply` path's own side effects — only that the guard precedes them.
- The other `--build-only` behaviour (that it really builds and really stops) is covered
  by Flux's verdict, not by this change.
