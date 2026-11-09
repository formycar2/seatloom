# Standing duty: **I checked whether the pattern I found twice — a verdict with no route to a card — had any remaining instances, and the answer is no for live work. The 18 Nimbus dispatches with verdicts and no cards are the containment chain's audit trail, including the `fail`s that were superseded, spread across a dozen packet branches. Every live deliverable has a card.** So the gap I closed for Vela and Mira is closed, and this round confirmed it rather than assumed it.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-verdictless-dispatches-are-the-audit-trail-and-not-unlanded-work-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. **Three cards pending, all three verified by me.**

## Why I checked, and what I found

**Twice now I have found a seat with finished, verified work and no route to the mainline.** For Vela and for Mira's newest work, the cause was the same: no verdict named their dispatch, and the route to a verdict was a dispatch to Flux that nobody had issued.

**So the obvious follow-up is to ask whether there are more. I asked it as a query rather than by inspection:**

```sql
select d.target_seat_id, count(distinct d.id)
from dispatches d
where d.status='done'
  and exists (select 1 from canonical_events e
              where e.payload->>'dispatch_id'=d.id
                and e.event_type='ReviewVerdictIssued')
  and not exists (select 1 from acceptances a where a.dispatch_id=d.id)
group by 1
```

**Result: 18 verdict events across 16 Nimbus dispatches, and nothing for any other seat.**

**And they are not unlanded work.** Reading the verdicts themselves:

| verdict | commit | what it was |
|---|---|---|
| pass | `34f8681` | the build-identity capability |
| **fail** | `5d66aab` | superseded |
| pass | `e238a04` | superseded |
| **fail** | `b843af4` | superseded |
| pass | `5af0e16` | superseded |
| pass | `649b449` | the deletion guard |
| pass | `889d687`, `77a7c5f`, `9de5fb5` | build-failure regression, superseded |
| pass | `6dd68c5`, `7d6fa22` | the candidate lineage |
| **fail** | `0a7f1a7` | **the `--apply` gap I found and dispatched** |
| **fail** | `857ca85` | superseded |
| pass | `33909f1` | the containment fix |
| **fail** | `1d2cabb` | **the db-error defect, later fixed at `41da35b`** |

**So the 18 are the containment chain's history: the passes that were superseded by later passes, and the fails that were fixed and re-verdicted.** Each of those commits lives on a packet branch — `build-git-ok-truthful`, `build-identity-three-states`, `sandbox-flag-exclusion-pin`, `build-failure-regression`, `verification-candidate`, `candidate-final` — and none is on `track`.

**A verdict is a record of a measurement at a moment. It is not a claim that the work is still current.** The `fail` at `1d2cabb` is in this list, and that defect was closed two rounds later at `41da35b`, which *does* have a card. **Reading the 18 as "18 pieces of unlanded work" would be reading a history as a backlog.**

## And the live work is covered

| deliverable | branch | card |
|---|---|---|
| db-error caller-path fix at `41da35b` | `packet/nimbus/candidate-final` | `acc-disp-4xBIRHNOnacKkCwCPJhJG`, **pending** |
| knowledge-base gate convention | `packet/vela/washing-pipeline` | `acc-disp-MlhRWjAlGcaPUj2BqMpOr`, **pending** |
| operator's third obligation UI at `5810c7e` | `packet/mira/knowledge-panel` | `acc-disp-bouEIM-49hK0xR_-B8EMm`, **pending** |

**And I re-verified the one whose branch has moved furthest**: `candidate-final` is 49 commits past the verdict at `41da35b`, and none of the 49 touches `memory.rs` or `test-memory-rejection-message.sh`. **That is the fifth time this week a growing count has turned out not to touch the deliverable.**

## What this establishes about the pattern

**The gap was real and it was specific**: two seats had finished work with no verdict naming their dispatch, because nobody had dispatched a verifier for them. Both now have cards.

**What was not true, and would have been easy to claim, is that the acceptance path is generally broken.** It is not. Nimbus has 16 verdicts and one live card; Mira has verdicts and now a card; Vela has a verdict and now a card. **The path works for the seats somebody dispatches a verifier for, and the seats it excluded were excluded by an absence of dispatches rather than by a rule.**

**That distinction matters for the escalation.** "The acceptance path excludes seats Flux hasn't reviewed" reads as a structural defect needing a rule change. **"Nobody had dispatched a verifier for these two seats' finished work, and I have now done so" reads as a task, and it was one I could do.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict-without-card population and its verdict contents, `git merge-base --is-ancestor` and `git branch --contains` for where each commit lives, `git rev-list --count` and `git show --name-only` for the branch movement against the deliverable files, and `psql` for the card-to-branch mapping.
