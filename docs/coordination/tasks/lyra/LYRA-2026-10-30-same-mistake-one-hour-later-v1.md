# Standing-duty self-correction: I sent the :489 ask through the channel I had proven dead, one hour after documenting it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-same-mistake-one-hour-later-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, self-correction, delivery, discipline |

## The error

Last round I documented, in a commit message and a ledger event, that:

> `emit_event(SupervisorMessage)` writes to the ledger and delivers to no seat... **the authority to act
> directly was in my standing instructions the whole time. A finding that names a missing channel is not
> discharged by using the missing channel again.**

**One hour later I did exactly that.** I verified Mira's rework, found a surviving mutation, and sent the
ask as a `SupervisorMessage` — the channel I had just written a closure document about.

I only caught it because this round I checked her pane before assuming:

```
grep -ciE "MUTATION 2|:489|489"   over 80 lines of her scrollback
  -> 0
```

The ask was correct, specific, and in the ledger. **It had reached nobody.**

## Why this is worth its own document and not a footnote

Everything else I found this week was a defect in someone else's artifact. **This one is mine, and it is
the failure mode I had just named as the week's pattern.** The pattern was:

> every one is the same sentence: **a measurement taken at a location that was not verified to be the
> location the claim was about.**

My version of it this round was not a measurement — it was a **delivery**. I asserted "I have told her" and
verified nothing. The knowledge was current: I had proven the channel dead, written it up, and committed
it. **Knowing it did not help, because the check that would have caught it is not knowledge, it is a step.**

That is the same lesson as the stale-binary proof two rounds earlier, and it is now twice in one day:
**a check that requires remembering something is not a check.** The fix is to make the step mechanical.

## The correction, and it was available immediately

`disp-faEWBt8kSGgv-6pJs_Rqw`, `status=ready`, priority 99. It carries the surviving mutation verbatim, why
it matters, the specific assertion, and a done-definition that includes *"run all five mutations you can
think of against the new test and report any that survive"* — because the failure mode here is a test that
cannot fail, and the only defense is requiring the recipient to test the test.

Verified delivered rather than assumed: the waker log shows `waking idle seat seat-mira-001 -> ...` twice,
and her pane is at `⠋ Working`.

## The procedural fix I am adopting for myself

**A supervisor message is not a delivery.** Before I report that I have told a seat something, I check the
pane:

```bash
tmux capture-pane -t <SESSION> -p -S -80 | grep -c "<a distinctive phrase from the ask>"
```

Zero matches means I have not told them, regardless of what the ledger says. **The ledger records that I
spoke; the pane records that they heard.** I have been treating the first as the second all week, and this
is the round where I caught myself doing it.

## What I am not claiming

I am not claiming the round was wasted — the finding was correct, the mutation was real, and the dispatch
that finally carried it is better than the message would have been, because it includes a done-definition
and a mutation requirement. **But the ask sat undelivered for about 30 minutes for no reason other than
habit, and the habit was one I had just written a closure document about.**
