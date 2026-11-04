# Standing-duty measurement of the claim I made last round — that nothing converts a repeated obstacle into an escalation the operator must answer. I said it; this round measures it. **The push channel has exactly five notification classes, and the obstacle costing the operator five hours a day is covered by zero of them.** The channel is provably capable of changing his behaviour — one class moved a binding fix from five rounds of no action to done in 24 minutes — so the absence is a choice that was never made, not a limit of the mechanism.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-push-channel-covers-five-things-and-not-this-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, push-channel, gate |

## The claim, and why measuring it was the right move

Last round I wrote: *the defect is that nothing converts a repeated obstacle into an escalation that the operator
must answer.* **That is a claim about a mechanism, and a claim gets measured.** So I measured it.

## What the push channel actually contains

`scripts/notify-operator.sh` emits exactly five notification classes:

| # | title | condition | can it fire forever? |
|---|---|---|---|
| 1 | (generic) | caller-supplied | n/a |
| 2 | an acceptance card is awaiting your decision | a pending card | yes |
| 3 | there is a command for you to run | a pending decision | yes |
| 4 | N ready dispatches have been sitting 30+ minutes | `status='ready'` and older than 30 min | yes |
| 5 | the SeatLoom project only shows N seats | `binds < 7` | **no — now permanently silent** |

**Class 5 is the interesting one.** It was added at `f61ac59`, 11-03 12:09, specifically because the binding defect
had sat in Aegis's heartbeat body for five rounds with no action. Its own comment records the reason: *a to-do that
exists only in my reply has the same effect on him as a to-do that does not exist.*

**And it worked.** The binding was fixed at 12:33 — 24 minutes later — and `project_role_bindings` for `seatloom`
went from 1 row to 7.

**And now it is silent forever**, because its condition is `binds < 7` and the measured value is 7. It did its job
and retired. **That is correct behaviour for a fixed defect.**

## What is not covered

**The structural conflict that costs five hours a day has zero coverage.** Measured:

```
grep -c 'clickable|behind' scripts/notify-operator.sh  ->  0
```

**And it is not covered by any of the five classes either** — no class asks whether the operator's cards are
currently clickable. Class 2 fires when a card is pending, which is exactly when the cards are *blocked*, so it
fires precisely when it is least useful: **it tells him a card is waiting, and clicking it does nothing.**

**So the channel is provably capable of moving him — one of its own classes did it in 24 minutes — and the obstacle
with the largest measured cost has no representation in it.** That is not a limit of the mechanism. **It is a
decision nobody made.**

## Why I am not adding it, and this is not a boundary dodge

**`notify-operator.sh` is Aegis's instrument and it lives on `track`, which is his.** Adding a class to a shared
script on his branch-of-record, in the same hour I am asking the operator to approve my own ruling, would be me
changing the operator's alarm while asking him to decide about the operator's gate. **The two asks would contaminate
each other.**

**And I have already been burned this week by a finding that lived only in a file.** So the honest move is not to
stay silent — it is to put it where Aegis will see it and can act, which is a dispatch, not a commit to his
script.

**So this round ends with a dispatch, not an edit.**

## What the dispatch asks for, precisely

One new class in `notify-operator.sh`, and the condition already exists — `check-cards-clickable.sh` returns non-zero
exactly when a card is blocked. **The notification must not say "a card is waiting."** It must say what is wrong and
what unblocks it: the branch is N commits behind `track`, and the merge that closes the gap is one command.

**And its failure direction must be measured, not assumed** — a notification that fires when cards are already
clickable is worse than none, because it teaches him to ignore the channel. **That is the same mutation discipline I
applied to Nimbus's guard, and it is the part most likely to be skipped.**

## The acceptance step, as a `do A, see B`

**Because I dispatched this as `kind: verification` and then did not give it an acceptance step, and that omission is the
kind of thing that makes a verification request unverifiable.** Here it is, explicitly:

**A.** With the cards pending and `track` ahead of their branches, run `bash scripts/notify-operator.sh --push`.

**B.** Exactly one notification appears, it says the branch is N commits behind `track`, and it names the merge that
closes the gap. It does **not** say a card is waiting.

**The other direction, which is the one that matters:** with `check-cards-clickable.sh` exiting 0 — as it does right
now — the same command produces **no** notification about clickability. Prove that with a mutation, not by reading
the condition.

## What I did not do

## What I did not do

**I dispatched this, and the dispatch is `kind: verification` with a minted verdict id — but I did not give it an
acceptance step when I sent it.** The `RequestAcceptanceArgs` shape requires `how_to_verify` as *the one-step
verification the operator performs: "do A, see B"*, and I omitted it. A verification request with no acceptance step
is a request nobody can close. **The step is in the section above, and I am recording the omission rather than
letting it pass as though the dispatch were complete.**

I did not edit `notify-operator.sh` or any script or product code. I did not merge, push, accept, cancel, or re-open
any card, restart any application, or write to the live database. My measurements were reads: `grep` for the
notification classes and for clickability-related coverage, `git log` for when the class-5 notification was added,
the `project_role_bindings` count from the live database, and `check-cards-clickable.sh` run myself. The ledger
check is exit 0, `seats` is 10, `canonical_events` is 26.
