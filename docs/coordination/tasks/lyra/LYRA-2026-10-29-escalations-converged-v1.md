# Standing-duty finding: my two escalations converged, and the mechanism I said was missing exists

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-escalations-converged-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, escalation, closure, operator-decisions |

## What happened

I escalated twice last round:

1. **The branch-sync refusal** — 18 emergency dispatches, 5 seats, no ruling in 26 hours.
2. **The verdict-ref checker** — three cards cancelled by a script that contradicts the product's own guard.

Both are now answered, and neither by me.

## Escalation 2 — the mechanism I claimed was missing, is not missing

Last round I recorded that `operator_decisions` was **empty** and concluded the operator's questions had
never been asked through the proper channel. That was a true observation and a wrong conclusion: the
table was empty **because the tool refuses seats**, and Aegis — who is allowed — had simply not used it
yet. He has now.

At 22:04 he emitted `OperatorDecisionRequested` asking the operator whether Aegis's own non-product
commits should go to track while acceptance cards are pending, with the cost measured:

```
commits_to_track_by_me:            6
seats_that_had_to_sync_each_time:  3
times_all_23_cards_became_unclickable: 5
```

**That is my escalation 1, measured from the other side.** I counted 19 emergency sync dispatches from
the ledger; he counted 5 occasions on which every card became unclickable, from the commit side. Same
defect, two instruments, and his is the one that reaches the operator.

He also records four things he did wrong today, and the fourth is the exact lesson I wrote down last
round in my own retraction:

> [translated] Then I wrote a script arguing that "pure-doc commits need not go to track", wired it into
> the hook, and the hook let it through — **I tested whether the classification was right, not whether
> the cards were still clickable.** All 23 unclickable. Script deleted, hook restored.

He wrote a script arguing that pure-doc commits need not go to track, wired it into the
hook, the hook let it through — **he tested whether the classification was right, not whether the cards
were still clickable.** All 23 unclickable. Script deleted, hook restored.

That is the same shape as my mutation-suite error: **the test passed against the population it was
designed around, and the failure was in a direction nobody tested.** Two seats, same day, same mistake,
independently. That is worth more than either of us noticing alone.

## Escalation 1 — verified independently, and his correction is right

Aegis corrected his own duplicate-row count: he had reported 6 groups with a worst case of 4 rows, and
the truth is **5 groups of 2**. His root cause: he grouped on four columns and printed three, so the
report misdescribed its own result and he read the misdescription.

I re-ran his corrected query myself:

```
5 groups, every one of them 2 rows
```

**He is right, and the correction matters in the direction that is easy to miss** — he explicitly says
it does **not** shrink the defect. Five groups is the count, not a smaller problem. A correction that
makes a number smaller is exactly the kind a reader should distrust, and he headed that off himself.

I also checked his `scripts/land-023-memory-dedup.sh` for the thing that would make it dangerous, and
it is clean: dry-run by default, an explicit `YES` typed before any delete, and **no mechanical
"keep the oldest" default** — his comment says a mechanical choice would be a guess. That is the right
shape for a live-database act.

## What I got wrong, recorded

**I said the operator-decision channel was absent. It was not.** I observed an empty table and stopped
there instead of asking *why* it was empty. The answer was in the tool's own entry rules, which I read
the same day I made the claim: *"the caller is the operator or Aegis (a seat is refused outright).*
An empty table plus a tool that refuses seats is not a missing mechanism — it is a mechanism working,
waiting for the one caller allowed to use it.

That is the third time this round I have drawn a conclusion from an absence without checking what the
absence meant. The other two: probing a dead-stripped binary and concluding staleness, and running a
checker whose population had no instance of the defect and concluding it worked.

**The pattern is consistent enough to name: I keep treating "I could not measure it" as "it is not
there."** The corrective is the same each time — before concluding absence, find the instrument that
would show presence, and check whether it is the right instrument.

## State

| Escalation | Status |
|---|---|
| Branch-sync refusal cost | **Reached the operator** via Aegis's `OperatorDecisionRequested`. No ruling yet. |
| Verdict-ref checker contradicts the guard | **Fix and ruling exist on my branch**; not on track. No pending card is currently at risk (all 7 pending are shape A). |
| ROADMAP operator list | Landed `f8ca6e9` on both remotes. |

No dispatch for me. No card of mine pending. The two things I cannot do — land the checker on track,
rule on the sync refusal — are correctly not mine.
