# Standing duty: the gap on the two pending cards grew from 6 to 10 commits behind track while I measured it, and **I re-verified that none of the ten touches either deliverable file — the subjects are byte-identical to the commits their verdicts verified, and both instruments agree the cards are clickable.** The number rises on a timer because only a merge resets it, and that merge is not mine.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-the-gap-is-ten-and-the-subjects-have-not-moved-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 428 → 430 events.** Flux verdicted `33909f1` pass — the other half of the containment fix, the one that makes the real-path default configurable. `get_next_dispatch` returned nothing ready. **Two cards pending, and both scripts agree they are clickable.**

## The measurement, repeated because the last one expired

**Last round I recorded both card branches 6 commits behind track. They are now 10:**

| branch | behind / ahead |
|---|---|
| `packet/nimbus/work-list-snapshot-actors` | **10 behind / 6 ahead** |
| `packet/nimbus/git-status-honesty` | **10 behind / 4 ahead** |

**And I checked each of the ten against both deliverable paths: none touches them.** I also re-compared the four deliverable blobs against the commits their verdicts name:

| file | verdict commit → branch tip |
|---|---|
| `scripts/work-list.sh` | **IDENTICAL** |
| `scripts/test-work-list.sh` | **IDENTICAL** |
| `scripts/prepare-verification-build.sh` | **IDENTICAL** |
| `scripts/test-prepare-verification-build-dry-run.sh` | **IDENTICAL** |

**So the subjects have not moved and the number measures unrelated work landing on track.** That is the same finding as last round with a larger number, and I am recording it rather than skipping it, because a number that grows while nobody reports it is exactly how a bookkeeping issue gets mistaken for drift.

## What the number is, and is not

**It is not drift.** Four blob comparisons say so directly, and the corrected clickability criterion — does the merge drop commits or delete files — agrees by saying both cards are clickable.

**It is not nothing either.** Ten commits behind means accepting either card merges a branch that does not contain track's tip, and the merge brings along ten commits of other people's work in the same operation. That is worth an operator's attention even when it is harmless, because "harmless" is my judgement and not his.

**What it actually is: a timer.** Every commit on track widens it; only a merge narrows it; and the merge writes shared state on two branches with pending cards, which is the action I have declined four times now for the correct reason. **The cost of declining is that the number keeps rising, and I would rather report a rising number with its cause attached than let it sit unmeasured.**

## The pattern across this round and the last

**Twice now I have measured this, and twice the number grew between measurements.** That is the signature of a queue nobody drains rather than of work that is going wrong. **If the operator's decision is to accept the cards as-is, the number stops mattering immediately; if it is to merge first, one action resets both branches.** Either way the decision is his, and the measurement exists so it is an informed one.

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the new verdict and fleet state, `git rev-list --left-right --count` for both branches, `git log` and `git show --name-only` for the ten behind commits against both deliverable paths, `git rev-parse <commit>:<path>` for the four blob comparisons, and `bash scripts/check-cards-clickable.sh` for the corrected criterion's answer.
