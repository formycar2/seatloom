| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1900-thirteen-candidates-zero-lost-and-six-that-can-never-be-matched |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, unclaimed-work, classification, checker-artifact, sandbox-blocked |

# Thirteen candidates, zero lost work, and six the checker can never match

## Why I measured this

`check-unclaimed-work.sh` exits 1 with 13 candidates and has done for two days. A check that is
permanently red stops being read, so I classified every one of them instead of noting the count again.

## The classification, measured

| # | dispatch | verdict | card | where the work is |
|---|---|---|---|---|
| 1 | `disp-MZHfmhAZ9ztz9R_byYgHY` | **pass** (`ev-H6JDfr293lSV2kabLi8WI`) | no | on `packet/nimbus/candidate-final` |
| 2 | `disp-sLODoywUUUpXpPSCfZY96` | **fail** (`ev-R6oKNQif6drf1b_857MJd`) | no | superseded by #1's redo |
| 3 | `disp-Pca6m4qu1RERbP37cwlTL` | **fail** (`ev-3rJ9wLmN2pQvKx8TbYcHfA`) | no | superseded by `41da35b`, **which has a card** |
| 4 | `disp-uIOs7Wu0iP8gyR5lBVtw6` | none | no | `52cdb8b` on the branch |
| 5 | `disp-dV1TU7AxUCWhq8I0iENGV` | none | no | `a1fddfb` on the branch |
| 6 | `disp-qPfPgpcXldYZ9SiX8Ksvw` | none | no | `857ca85` on the branch |
| 7 | `disp-91B8U33bI3rzY5leIL6cg` | none | no | `33909f1` on the branch |
| 8 | `disp-gyBpZxnMiNepox6npj-c2` | none | no | on the branch |
| 9 | `disp-e5fmV66QT6HdtW1L8fnql` | none | **`acc-disp-4xBIRHNOnacKkCwCPJhJG`** | the card exists |
| 10 | `disp-eBkpELwKY_geVRG0Y8xgI` | none | **`acc-disp-MlhRWjAlGcaPUj2BqMpOr`** | the card exists |
| 11 | `disp-RLW1rHOazPBn45dZKQY6O` | none | — | the corrected card payload |
| 12 | `disp-O-3Pxj-Z4jl9VWFBH1esc` | none | no | Vela's first card attempt, blocked, superseded by #10 |
| 13 | `disp-Lw03MbrCtl-b8b4LtKwzZ` | — | — | Flux's verification; its verdict names #3's dispatch |

**Zero lost.** I checked each commit by hand: `52cdb8b`, `a1fddfb`, `857ca85` and `33909f1` are all
reachable from `packet/nimbus/candidate-final` and pushed to both remotes. Every other one produced an
artifact that exists.

## Three groups, not one

**Sandbox-blocked (5, items 1, 4–8).** These are the sandbox-lifecycle subjects. The work is on the
branch and one of them carries a passing verdict. A card cannot be opened because the acceptance method
needs a real sandbox to demonstrate against, and the sandbox is a 95-byte fake with no backup. **This is
one blocker wearing five costumes, and it is your decision, not a defect.**

**Superseded by a redo (2, items 2 and 3).** Both carry `fail` verdicts whose successor passed and was
carded. That is the verdict system working: a failed verdict was not overwritten, the work was redone,
and the redo got its own verdict and its own card.

**Artifact under a different dispatch id (6, items 9–13).** These are dispatches whose deliverable *is*
a card or a verdict, and those artifacts carry the dispatch id of the work they gate, not the id of the
dispatch that asked for them. `disp-e5fmV66QT6HdtW1L8fnql` asked Nimbus to open a card; the card is named
for the implementation dispatch. That is **the "one implementation subject needs its own verdict
identity" rule working correctly** — and the checker cannot see it, so it will report these six forever.

## The finding worth keeping

The checker counts **dispatches without cards**. Six of its thirteen candidates are dispatches whose
correct outcome is a card belonging to someone else. So the check's exit 1 is not a defect count and it
is not a stable signal either — **it is a permanent 13 that contains 0 lost, 2 superseded, 5 blocked on
one operator decision, and 6 that can never be matched by design.**

A check whose true answer is permanently non-zero for structural reasons will be ignored, and then the
one day it should have caught something it will be ignored again. The fix is not to suppress it — the
candidates are real dispatches. The fix is for the check to separate "no artifact exists" from "the
artifact exists under another id", which is a query it can run: join `acceptances.dispatch_id` and
`canonical_events.payload->>'dispatch_id'` against the *branch commits* rather than against the dispatch
that requested them.

I have not changed the checker. That is a change to a gate, and it should go through the acceptance path
rather than my judgement — the same boundary I held on Vela's doc-gate change and Nimbus's message fix.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Two pending cards, both clickable, both merge clean into current track.
- Salvage: 2 live patches round-tripping, 7 superseded. Nothing at risk.
- Push: local and remote both at `83f4e34`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Two cards are waiting**, and they are different work.
- **The sandbox decision now unblocks five dispatches at once**, not one. That is the cheapest lever
  available to you.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed. The failure is legible
  now, which is the precondition for answering it, not the answer.
