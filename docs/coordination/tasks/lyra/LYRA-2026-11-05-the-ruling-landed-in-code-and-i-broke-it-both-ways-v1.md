# Standing-duty verification of the clickability gate rewrite, the first time this week a ruling has landed in code: `6dbc3af` implements the operator's ruling. **The level-with-base refusal is gone; being behind is now a note, not a refusal; and the two questions he named are asked instead — would the merge delete files the base tracks, and does the branch contain the base.** I mutation-tested both directions myself without reading his test: restoring the level refusal turns it red 2 of 17, and gutting the deletion check turns it red 1 of 17. Both restored byte-identical. **And the test drives the gate itself with a stubbed row source rather than reimplementing it, which is the only thing that makes the mutation mean anything.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-ruling-landed-in-code-and-i-broke-it-both-ways-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, verification, measured, ruling |

## What landed

`6dbc3af` on `track/infra-foundation`, +90 to `check-cards-clickable.sh`, a new `test-clickability-criterion.sh`
(309 lines), and a delivery note. **All four of my dispatches to Aegis are `done`.**

**The gate now asks the ruling's question.** Measured in the script: the `merge-base --is-ancestor` refusal is gone
(it survives only in comments explaining what used to be there and why), being behind is computed and becomes a
`behind N` note, and the refusal comes from two checks — `merge-tree --write-tree` then
`diff --diff-filter=D` against the base, minus the ruled exemptions, and containment.

**And he wrote the reason into the code**, which is the part that will matter in six months: *a gate that stops
refusing without starting to check anything is worse than one that refuses on the wrong ground.*

## What I verified without reading his test

**Baseline:** `scripts/test-clickability-criterion.sh` → **exit 0, 17 passed, 0 failed.**

**My mutation 1 — restore the level refusal** (make `behind != 0` a refusal again):

```
EXIT=1
gutted gate says: FAIL: 1 of 1 pending cards are UNCLICKABLE
level-restored gate says: FAIL: 2 of 1 pending cards are UNCLICKABLE
15 passed, 2 failed
```

**My mutation 2 — gut the deletion check** (`deleted=""`, so nothing is ever refused for deleting):

```
EXIT=1
16 passed, 1 failed
```

**Both restored from a `cp` backup, verified byte-identical with `diff -q`, baseline back to exit 0.**

**So the gate is load-bearing in both directions, established by me and not by his own mutation.**

## Why his test is the right shape, which I checked rather than assumed

**It drives the gate itself.** The gate reads its rows from `podman`/`psql`; the test stubs `podman` so the gate sees
exactly the scenario under test. **It does not reimplement the gate's logic and test that.** A test of a
reimplementation proves the reimplementation, and the mutation below it would be meaningless.

**And it proves the scenarios are what they claim before reading any answer** — measured, `behind-only`'s merge
really deletes nothing and really is behind, and `deletes-base-file`'s merge really deletes `README.md`. *A scenario
that is not what it claims proves nothing.*

**He also recorded a real defect in his own first attempt**: the mutated copies lived in a `mktemp` dir, and the
gate's second line is `cd "$(dirname "$0")/.."`, so every mutation reported exit 2 *branch
track/infra-foundation does not exist* — **which is the test failing, not the gate.** He moved them inside the
repository. That is the failure family I have recorded all week, caught by him in his own instrument.

## The measured behaviour, both scenarios, through the real gate

```
behind-only        -> exit 0  OK: all 1 pending cards are clickable
deletes-base-file  -> exit 1  FAIL: 1 of 1 pending cards are UNCLICKABLE
```

**So a branch that is merely behind is now clickable, and a branch whose merge deletes is refused.** That is the
ruling, in behaviour, not in prose.

**And the live gate with zero pending cards says *no pending cards, nothing can be blocked* and exits 0** — which is
honest, and no longer the only thing it can say.

## What this closes, and what it does not

**Closed, in code:** the criterion the operator ruled invalid. The five-hours-a-day cost of the level-with-base
proxy is gone for new cards.

**Not closed, and unchanged:** the deletion guard is still not on `track` — `acceptance_cmds.rs` still goes straight
to `merge_branch_into_base`. Aegis measured this himself at 12:03 and I measured it at 01:29. **So the operator's
Accept is now protected by a gate that reads the merge correctly, but the merge itself is still unguarded.** The
gate is a check the operator runs before clicking; the guard is what stops the merge if the check was skipped.

**The honest statement: the ruling is implemented, and the thing it was traded for is still on a branch.** Those are
different layers and only one of them moved.

## What I did not do

I did not edit the gate or any script beyond the two mutations, which I restored and verified byte-identical. I did
not merge, push, accept, cancel, or re-open any card, restart any application, or write to the live database. I did
not merge `6e04dae` into `track`. My measurements were reads plus two mutations with restores: `grep` on the gate for
`is-ancestor` and the two new checks, `git show --stat` for the commit, the test file's baseline and both mutations,
`check-cards-clickable.sh` run live, and the dispatch rows. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 37.
