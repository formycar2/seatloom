# Standing-duty finding: Aegis's step 2d works, and my measurement of its failure was wrong

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-step-2d-works-and-my-measurement-was-stale-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, closure, self-audit, instrument |

## What I set out to check

Aegis landed `e43c97e` — protocol step 2d — which says that a seat with pending cards on a behind
branch must be **dispatched** a sync, because the waker only nudges seats that have a ready dispatch and
nothing else turns "your cards are unclickable" into something the owner is told. It is the answer to my
escalation from two rounds ago, reached independently and with two rules I did not have:

- **a branch with no pending card gets no dispatch.** The behind-count is not the severity; the number of
  cards on that branch is. He measured four branches 71–179 behind after the operator accepted 27 cards,
  and only one of them needed a sync.
- **say "this is not your fault".** The behind is usually his own commit.

## I measured it as failing, and I was wrong

I ran `git rev-list --left-right --count track/infra-foundation...<branch>` for each pending card and got
`behind=14` and `behind=29`. I was about to report that step 2d had dispatched two syncs, marked them
`done`, and left the cards stranded anyway.

**The measurement was wrong, not the mechanism.** `packet/mira/knowledge-panel` was at `6ead8d8`, a merge
of `track/infra-foundation` committed at **07:03** — after my local ref for track. I had not fetched. The
seats had synced correctly and my `track/infra-foundation` ref was the stale one.

After fetching, every one of the seven cards reads `track_only=0`, and the repository's own instrument
agrees:

```
$ bash scripts/check-cards-clickable.sh
OK: all 7 pending cards are clickable
EXIT=0
```

## What I got right, and it is the same lesson a fourth time

Before writing anything I fetched both remotes and re-ran. That is the only reason this document is a
correction rather than a false alarm. But the sequence matters: **I had already formed the conclusion
"step 2d dispatched and left them stranded" before I checked whether my instrument was current.** The
order was wrong. Fetch first, then measure, then conclude.

This is the fourth instance this round of the same family:

| Round | What I concluded from a bad instrument |
|---|---|
| MCP binary | "16 hours stale" — probed a dead-stripped binary for source strings |
| Verdict checker | "the mutation survived" — added a column, shifted the shell's read order |
| Operator decisions | "the channel is absent" — read an empty table without reading the tool's entry rules |
| Step 2d | "cards still stranded" — compared against an unfetched ref |

**The pattern is now unmistakable and I am naming it as a standing risk rather than a series of
accidents: every one of these was a measurement taken with an instrument I had not first verified was
current.** The corrective is procedural, not insight — before concluding anything from a measurement,
verify the instrument against a known-good source. For git that means fetch. For a checker that means
mutation-test it. For a table that means read the code that writes it.

## What step 2d got right, on the evidence

- Two dispatches, `disp-spBuVAVQmm6S3hwQPsWS0` (Mira, 2 cards) and `disp-l7liKn-Ka-iwpl1EmYRI4` (Onyx,
  5 cards on 3 branches) — both `done`, both branches merged within the turn.
- **Zero sync dispatches today** outside that pair. Compare the 19 emergency dispatches I counted
  yesterday across 5 seats. The bounded rule — dispatch only for branches that carry pending cards —
  cut the volume to the seats that actually needed it.
- The cards are clickable, verified by the repository's own script, not by my arithmetic.

## State

| Item | Status |
|---|---|
| All 7 pending acceptance cards | **clickable**, verified with `scripts/check-cards-clickable.sh` |
| Sync-refusal escalation | **Answered procedurally** by step 2d; the underlying refusal is unchanged |
| Verdict-ref checker on track | still payload-only; my fix and ruling remain `on_track=0` |
| Provenance badge (6 tabs) | finding handed to Mira last round, no action seen yet |
| Operator decisions | 0 pending, 0 resolved — `request_operator_decision` refuses seats, so the 023 deadlock and the Aegis-commit question still wait on the operator |
