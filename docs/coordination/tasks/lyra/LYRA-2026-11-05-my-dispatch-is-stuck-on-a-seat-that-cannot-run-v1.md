# Standing-duty finding that my own dispatch is stuck on a seat that cannot run, and the mechanism that will not tell anyone: I dispatched `disp-ZZEb6fpwXPR0zqMQjiHwV` to Aegis last round. It was **delivered** — `SeatDispatchDelivered` at 02:14 local — and it has produced no work, because **his pane is showing seventeen consecutive model failures ending in "All fallback models failed"**. Measured across the fleet, he is the only seat in that state. **The dispatch system records that he took the work. It has no representation of whether he can do it.** So an infrastructure outage looks exactly like a slow seat, and the one thing I dispatched this round is silently not happening.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-my-dispatch-is-stuck-on-a-seat-that-cannot-run-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, dispatch-channel, outage |

## What I measured

**The dispatch is delivered and unworked:**

```
disp-ZZEb6fpwXPR0zqMQjiHwV | seat-aegis-001 | in_flight | issued 18:03:43 UTC
SeatDispatchDelivered | seat-aegis-001 | 2026-11-04 18:14:07 UTC
events referencing it: 2   (issued + delivered; no completed)
```

**And `notify-operator.sh` is unchanged since `f61ac59`** — the work has not landed.

**His pane, measured now:**

```
Model fallback: sp-personal/step-5-preview → sp-team/step-5-preview (primary restored)
Error: Request timed out.            x 5
Error: All fallback models failed (last: provider unavailable).
       Chain: sp-team/step-5-preview → modelproxy-anthropic/step-5-preview-sys → sp-personal/step-5-preview
Error: Retry failed after 3 attempts: Request timed out.
```

**Seventeen timeout or total-failure lines in his scrollback. Every other seat: zero or one**, and Flux's one was a
transient fallback that recovered on the same line.

## Why this is a finding and not just an outage

**An outage is an event. What I am reporting is that the system cannot tell the difference between an outage and a
slow seat.**

**Measured:** the dispatch lifecycle records `issued`, `delivered`, `completed`. There is no state for "the seat
took this and cannot proceed." So:

- **Aegis looks like he is working on it.** `status='in_flight'`, delivered 70 minutes ago.
- **The truth is he has failed seventeen times and stopped.**
- **And nothing surfaces the difference** — no notification class, no ledger event, no gate.

**This is the same failure family I have recorded all week, in a new costume: an instrument answering about the set
it can see, presented as an answer about the set that exists.** Here the instrument is the dispatch lifecycle, it
can see that he took the work, and it presents that as "the work is in progress."

**And it has a concrete cost right now.** The one thing I dispatched this round — a notification class that would
stop the operator losing five hours a day to unclickable cards — is not happening, and from the ledger's point of
view it is happening.

## What I did, and the one thing I deliberately did not do

**I did not retry the dispatch, reassign it, or take over the work.** Three reasons, and they are not scruples:

1. **The failure is upstream of Aegis.** Re-issuing a dispatch to a seat whose model chain is down produces another
   delivery record and no work — it would make the ledger look busier and change nothing.
2. **`notify-operator.sh` is his instrument and the finding is his to implement.** Doing it myself is the scope
   expansion my role forbids, and it is the exact action I refused two rounds ago.
3. **The outage may be transient.** Flux failed and recovered on the same line. Issuing a structural change in
   response to a transient is how you get a wrong change that outlives the thing that prompted it.

**What I did do is put it where the operator will see it** — this document, and the finding recorded here rather
than in a reply, because a finding that lives only in a reply is a finding that does not exist.

## What the mechanism would need, stated precisely

**Not a heartbeat on Aegis's pane** — reading another seat's scrollback is surveillance, and the fleet already has
a channel that works without it.

**A dispatch age with a floor.** `dispatches.issued_at` already exists and `notify-operator.sh` already has a class
that uses exactly this shape — class 4 fires when ready dispatches sit more than 30 minutes. **The same query with
`status='in_flight'` instead of `status='ready'` would catch this one**: in flight, older than the interval, no
completion event.

**And its wording must not say "Aegis is stuck"**, because that would be the instrument guessing again. It should
say the measured thing: *a dispatch has been in flight for N minutes with no completion; either the seat cannot run
or the work is larger than expected, and here is which one to check.*

## What I did not do

I did not retry, reassign, or cancel the dispatch. I did not implement the notification class myself. I did not edit
any script or product code, merge, push, accept, cancel, or re-open any card, restart any application, or write to
the live database. My measurements were reads: the dispatch row and its events from the database, `git log` on
`notify-operator.sh`, `git worktree list`, `tmux capture-pane` on all six seat panes with `-S` for scrollback, and
the grep counts for timeout lines per pane. The ledger check is exit 0, `seats` is 10, `canonical_events` is 28.
