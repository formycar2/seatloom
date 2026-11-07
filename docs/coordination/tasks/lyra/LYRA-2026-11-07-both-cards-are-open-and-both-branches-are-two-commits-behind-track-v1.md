# Standing duty: the two cards I re-dispatched are open, and applying my own rule to them produced a finding. **Each card names a branch whose tip is no longer the commit its verdict verified — but the deliverable files are byte-identical between the two, so the subjects are unchanged. The real issue is different and it is the structural conflict I priced weeks ago, now live: both branches are 2 commits behind track.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-both-cards-are-open-and-both-branches-are-two-commits-behind-track-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, standing-duty, coordination |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 401 → 409 events and 79 → 80 dispatches.** **Two cards are now `pending`** — the first pending cards in the ledger:

| card | branch | base | verdict | dispatch |
|---|---|---|---|---|
| `acc-disp-GWV4D2pEBdr8cLzQhteTO` | `packet/nimbus/work-list-snapshot-actors` | `track/infra-foundation` | `ev-HgnCjHzmOXMc75NwJkMQ0` | `disp-GWV4D2pEBdr8cLzQhteTO` |
| `acc-disp-JLsCEk0-uugW-fJBIGyux` | `packet/nimbus/git-status-honesty` | `track/infra-foundation` | `ev-OWnf_Qn34JRWWElo_Eb1Q` | `disp-JLsCEk0-uugW-fJBIGyux` |

**So the corrected premise landed: each subject has its own card on its own branch, and each card names the verdict the dispatch specified.**

## The check I ran, and what it found

**My rule is that a verdict reference is not verified until its branch AND commit match the card's. It does not match:**

| verdict | commit it names | branch | branch tip now |
|---|---|---|---|
| `ev-HgnCjHzmOXMc75NwJkMQ0` | `310c80a` | `work-list-snapshot-actors` | **`10dfe17`** |
| `ev-OWnf_Qn34JRWWElo_Eb1Q` | `4ae18ec` | `git-status-honesty` | **`237ab85`** |

**Both branches moved.** What they gained is a merge with an earlier track tip plus Aegis's commits — `ba753e3` (salvage), `16b22a3`, `a324d30`, `3e78eb7`, `7b2434b`, `0d6d730`, `1810655`, `93ef3d3` — and finally `10dfe17` / `237ab85`, both titled "Merge branch 'track/infra-foundation'".

**And the thing that makes this a non-issue for the subjects**: I compared the deliverable files by blob between the verdict commit and the branch tip, and **all four are byte-identical.**

| file | verdict commit → branch tip |
|---|---|
| `scripts/work-list.sh` | **IDENTICAL** |
| `scripts/test-work-list.sh` | **IDENTICAL** |
| `scripts/prepare-verification-build.sh` | **IDENTICAL** |
| `scripts/test-prepare-verification-build-dry-run.sh` | **IDENTICAL** |

**So the verdict covers the code that is actually on the branch.** The commit inequality is real but it is not the failure I have recorded before — there, a verdict named a commit on a *different branch*, or a plan named a hash that was neither the tip nor what the sandbox ran. Here the verdict's commit is an ancestor of the branch tip and the files it verified are unchanged.

## The real issue, which is the structural conflict now live

**Both branches are 2 commits behind track:**

```
track tip: 0862921 (11-07 22:19) fix(status): a missing waker log is ...
           90dcb29 (11-07 22:12) fix(hooks): the card guard asked the ...
           ba753e3 (11-07 21:10) salvage: cover Nimbus's in-flight work
```

**The branches merged `ba753e3` but not the two commits after it.** So accepting either card now would merge a branch that does not contain track's tip, and the merge itself would be a fast-forward-able merge of a stale base.

**This is exactly the conflict I priced on 11-05 — committing to track while cards are pending makes their branches behind — and it has now occurred with real pending cards.** The two commits behind are unrelated to either subject: a status fix and a hooks fix. **So the fix is a merge, not a re-verification**, and the merge is the operator's action, not mine.

**What I am not doing**: merging track into those branches. That is a shared-state action, and doing it to two branches with pending cards, on my own initiative, is precisely the unsupervised infrastructure action I ruled out after 10-31.

## What this means for the operator, stated as the smallest possible ask

**Two cards are pending and ready except that their branches are 2 commits behind track.** The subjects' code is verified and unchanged. **The smallest action that unblocks both is merging `track/infra-foundation` into both branches** — or accepting them as-is, if accepting a 2-behind branch is acceptable, which is a policy question rather than a technical one and I am not the one to answer it.

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the cards and their fields, `git rev-parse --short=12` for the verdicts' commits and the branch tips, `git log` for what the branches gained, `git rev-parse <commit>:<path>` for the four blob comparisons, and `git rev-list --left-right --count` plus `git log` for the behind-track state.
