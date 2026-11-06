# NIMBUS-2026-11-07-sandbox-flag-guard-first-v1

- seat: `seat-nimbus-001`
- dispatch: `disp-bnLh40QG32MBg7W3ONO1d`
- branch: `packet/nimbus/sandbox-flag-guard-first` (from `a48a694`)

## What changed

The `--apply` / `--build-only` conflict guard moved from line 255 to line 90 —
**immediately after argument parsing**. The diff is a pure relocation; nothing else in
`prepare-sandbox-build.sh` changed.

Flux's `ev-wZhxxQuaY0xUrg7IHNF2X` measured the guard sitting after branch resolution,
worktree resolution, identity, dirty, database and behind-track checks. A flag conflict
is a property of the arguments alone, so an operator passing both flags got a message
about a worktree rather than about their flags.

## Why this also made the test unfalsifiable

This is the part worth recording. With the guard after the worktree check, deleting it
changed nothing observable: the run still exited nonzero and still had zero side
effects, because the earlier failure fired first. So the assertion could not distinguish
"the guard refused this" from "something else refused this". The suite's own mutation
check reported exactly that — *"what it guards is empty"* — and refused to pass.

A test that cannot fail, and a check that detects its own vacuity, are the same subject.
The suite was right and the guard placement was wrong.

## The test now covers both shapes

The existing case (a valid fixture where the worktree resolves) is kept. A new case
passes a **branch that does not exist**, with no worktree, no build artefact and no
snapshot database — the shape where only the flag conflict can produce its own reason.
That case is what makes the placement itself falsifiable: with the guard relocated back
after the worktree checks, it goes red.

## Verification

- Relocation mutation (guard moved back after the worktree checks): **the new case goes
  red**, with the message naming the placement, not the worktree.
- Guard-removal mutation (the suite's own check): still red.
- **38 checks, exit 0** (was 36).
- `--build-only` alone still works — the guard cannot pass by refusing the flag outright.
- `bash -n` clean. The `--apply` path is otherwise byte-identical apart from the move.
- The dry-run suite on `packet/nimbus/dry-run-dirty-fixture` still exits 0 — a sibling
  branch I do not own, measured rather than assumed.
- No real `/Applications`, no main app (pid 90805), no sandbox, no ledger, no database.

## Reconciling the evidence

Flux measured at `e238a04` **plus an uncommitted test file** and concluded the test
"does not yet exist as an addressable commit". My artifact event said `a48a694` with 36
passing. Both are true and they are not in conflict:

- `a48a694` is the commit that added the test, pushed to both remotes.
- `e238a04` is the branch's *previous* commit, and the test was measured as uncommitted
  at that point.

So Flux verified `e238a04` (without the test) and carried the test in by hand; I
delivered `a48a694` (with it). The baseline difference is the commit each of us pinned,
not a disagreement about the work. I have not reused either verdict as a pass for the
other's commit — this dispatch exists because `a48a694`'s own test was still unfalsifiable
for the placement reason above.

## What I am not claiming

- No install, no restart, no sandbox or main-app replacement, no live database, no card.
- I have not built a fixture where worktree resolution *succeeds* and both flags are
  given, so the guard's message is not isolated in a fully valid context — Flux noted
  the same gap, and it is closed from the other direction (the invalid context) instead.
- `prepare-sandbox-build.sh --apply` remains unbacked and unchanged; replacing a sandbox
  goes through `replace-sandbox.sh`.
