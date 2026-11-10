# Standing duty: **All three pending cards are stale, and I can now say what the operator's Accept would actually merge. Every card records only a branch name — no commit — so an Accept merges the branch tip. And every branch has advanced past the commit its verdict verified: Nimbus's by 50 commits, Vela's by 28, Mira's by 7. The verdict events are the only place a commit is recorded, and nothing reads them at merge time. So three green verdicts are attached to three trees that are not the ones that will be merged.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-all-three-cards-are-stale-and-an-accept-merges-the-tip-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 520 events and 104 dispatches** — up from 518/103, and the change was not mine.

## What changed while I was idle

| time (UTC) | event |
|---|---|
| 11-10 02:36 | my dispatch `disp-fkLMc9KBolrwtEyoKi2EZ` completed by Nimbus |
| 11-10 03:02 | **`SeatDispatchIssued` by `seat:seat-aegis-001`** → Flux, `in_flight` |

**Aegis read my document and dispatched Flux to verify Nimbus's `fb83cd6`** — specifically the part I could not do, which he frames as "a check that looks wired and is not": make `assert_macho` return 0 unconditionally and confirm the suite goes red. **He also required that the verdict state explicitly that the toolchain is now real and that no sandbox was rebuilt, "because the operator has been waiting on exactly that distinction and an ambiguous verdict would let a rebuild be read as done."**

**And he got a rule right that I had been getting wrong**: "a card is opened by the seat that owns it." I verified that in the source — Guard 1 of `request_acceptance` is `WHERE id = $1 AND target_seat_id = $2 AND status = 'done'` at `dispatch.rs:1896`. **So I cannot open the card for Nimbus's fix, and neither can he. Only Nimbus can.** That is a structural correction to a chain I have been walking all week.

## The finding

**I checked whether the three pending cards were still attached to the trees they verified, and none are.**

| card | branch | verdict commit | branch tip | commits past the verdict |
|---|---|---|---|---|
| `acc-disp-4xBIRHNOnacKkCwCPJhJG` | `packet/nimbus/candidate-final` | `41da35b` | `fb83cd6` | **50** |
| `acc-disp-MlhRWjAlGcaPUj2BqMpOr` | `packet/vela/washing-pipeline` | `4927d34` | `bc8c06c` | **28** |
| `acc-disp-bouEIM-49hK0xR_-B8EMm` | `packet/mira/knowledge-panel` | `5810c7e` | `f554ac0` | **7** |

**And the card records no commit at all.** `acceptances` has `branch`, `base`, `merge_commit` — and `merge_commit` is empty, because it is written at merge time. The only place the verified commit exists is the verdict event's payload. **Nothing reads it at merge time.**

**So the card's first verification step is `git checkout packet/nimbus/candidate-final`, which checks out the tip.** The five steps run against `fb83cd6`, not against `41da35b`. The verdict says `pass` about a tree that is 50 commits behind the one being checked out.

## What this does and does not mean

**It does not mean the cards are wrong or that the work is bad.** Nimbus's 50 commits are the containment chain plus `fb83cd6`, which I verified myself this morning. Vela's 28 and Mira's 7 are their own continued work.

**It means the artifact the operator is being asked to Accept is not the artifact that was verified.** A card is a statement "this tree, at this commit, passed these checks", and it is presented as a button. **The gap is not that the checks are stale — it is that nothing in the card says which commit they were about.**

**And this is the same failure family as the `pgrep` defect and the two-remote finding, in a third shape**: an instrument presenting an answer about the set it can see as an answer about the set that exists. Here the card presents an answer about a commit as an answer about a branch.

**The sharpest version: if Nimbus had introduced a defect in commit 45 of those 50, the card would still read `pass`, the steps would still run, and the operator's Accept would merge it.** Nothing in the chain is lying. The chain simply has no field for the thing that matters.

## What I am not claiming

**I am not claiming the cards should be rejected, and I am not claiming I know what a correct fix is.** Two candidate fixes come to mind and I have not tested either: the card could pin the verdict's commit and refuse if the branch has moved, or the verdict could be re-issued against the current tip. **Both have costs I have not measured — the first makes every card expire when its branch moves, which is every card, and the second means re-verification on every commit, which is a much larger ask.** I am reporting the gap and not recommending a fix I have not tested, because that is the mistake I made twice this morning.

## The one thing I can verify

**Whether the three tips still merge cleanly into track** — because that is the thing the operator will actually do, and it is measurable now:

| branch | merges into `track/infra-foundation` |
|---|---|
| `packet/nimbus/candidate-final` @ `fb83cd6` | clean |
| `packet/vela/washing-pipeline` @ `bc8c06c` | clean |
| `packet/mira/knowledge-panel` @ `f554ac0` | clean |

**So the mechanical act would succeed. What would be merged is not what was verified.**

## What I did not do

I did not modify any file outside my new document, dispatch anyone, merge, accept or cancel a card, install, restart any application, or write to the live database. **I did not open a card for Nimbus's fix**, having just verified that the guard would refuse me. **I did not send keys to any tmux session.** My measurements were reads: `psql` over `dispatches`, `canonical_events`, `event_object_refs` and `acceptances`; `git rev-list --count`, `git merge-base --is-ancestor` and `git merge-tree --write-tree` for the three branches; and `grep`/`sed` over `dispatch.rs` for the two guards.
