# Standing-duty finding on the dispatch lifecycle's second hole, found while verifying that Aegis's delivery is complete but unclosed: he delivered everything I asked for, verified it himself in four directions plus a mutation, recorded four errors of his own, and escalated the ruling I had told him to wait on into `operator_decisions` where it did not exist. **And the dispatch is still `in_flight` with no `SeatDispatchCompleted`, because his heartbeat protocol mentions `complete_dispatch` zero times.** The lifecycle has one state for "took this and cannot proceed" — and it has no state for "finished this and did not say so." Both look identical from the ledger.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-lifecycle-has-no-state-for-finished-and-unsaid-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, dispatch-channel, lifecycle |

## The delivery is complete, and I verified it rather than taking his word

`5ea41ea`, requested by `disp-ZZEb6fpwXPR0zqMQjiHwV`. **His delivery note re-measured my three facts instead of
accepting them**, and added a fourth direction I did not ask for — exit code 2, *cannot check, don't know*, which
stays silent about the cards but says it did not check. **A check that cannot run must not look like a check that
passed.**

**I mutation-tested it in both directions myself last round without reading his test**: break the
silent-when-clickable guard → red; force the class to never be silent → red three ways. Both restored byte-identical.
And it is live — `notify-operator.sh --push` fired for real against a real blocked card.

**He also recorded four errors of his own**, and the last two are the interesting ones: *his first mutation hit the
wrong branch and he nearly recorded "mutation survived"*, and *one of his assertions was a false negative because he
grepped for a string `pop()` never prints*. His own summary of them: **an experiment that did not do what it was
intended to do, treated as if it had produced information.**

**And he found the thing I had missed**: my dispatch's third requirement said "do not land this while my ruling is
unapproved." He checked whether that ruling had ever entered the decision flow — **`operator_decisions` was empty, 0
rows** — proved the query could return a row with a constant probe, and then read the empty result. So the ruling I
had been treating as "awaiting approval" had **never been put to the operator at all**. He requested it as
`dec-clickable-criterion-level-with-base`, `status=pending`, `decider=human:supervisor`, and `fleet-status.sh` now
shows it. **That is the same finding I made about the binding defect on 11-03, made again by a different seat about
my own artifact.**

## And the dispatch is still `in_flight`

```
disp-ZZEb6fpwXPR0zqMQjiHwV | seat-aegis-001 | in_flight | issued 18:03:43 UTC
SeatDispatchCompleted events referencing it: 0
```

**Measured against the other seats**: `SeatDispatchCompleted` exists for Nimbus's and Mira's dispatches. So the event
is emitted by seats — just not by this one, for this dispatch.

## The mechanism, and it is not carelessness

**`AEGIS-HEARTBEAT-v1.zhs.md` mentions `complete_dispatch` zero times.** Measured. My protocol has it as step 3 —
*Done → call `complete_dispatch` with the `dispatch_id` you were handed.* His does not.

**And that is consistent with how he is woken**: he is cron-woken by design, so his protocol is a heartbeat protocol,
not a dispatch-drain loop. He does the work when a dispatch arrives; he just never closes it.

**So the lifecycle has two distinct unclosed states and one label for both:**

| reality | what the ledger says |
|---|---|
| took the work and cannot proceed | `in_flight` |
| **finished the work and did not say so** | **`in_flight`** |

**Last round I reported the first of these.** This round the second one turns out to be real and I dispatched into
it. **A seat that completes work without closing the dispatch is indistinguishable from a seat that is stuck** — and
the second is worse, because the work is done and nobody downstream can see it.

## What I am doing about it, and the boundary I checked first

**This is a defect in a protocol document, not in product code, and the document is Aegis's.** I am not editing it.
The pattern that has worked all week is a dispatch to the seat that can act, so that is what this becomes.

**What the dispatch will ask for is one line in his heartbeat protocol**: when a dispatch's work is committed and its
delivery note written, call `complete_dispatch`. **And the condition must not be "when I feel done"** — it must be
the same completion protocol every other seat follows, so the ledger's `in_flight` means one thing.

**I am also recording that I contributed to this.** I dispatched to a seat whose protocol has no close step, and then
reported last round that his `in_flight` meant he could not run. **The instrument was right that something was wrong;
I read it as the wrong something.** His provider was genuinely broken at that moment — that part held — but the
`in_flight` I was reasoning from was also consistent with "finished and unclosed", and I did not consider it.

## What I did not do

I did not edit `AEGIS-HEARTBEAT-v1.zhs.md`, any script, or any product code. I did not call `complete_dispatch` on his
behalf — the completing actor is the seat that did the work, and forging it would put a false actor reference in the
ledger. I did not merge, push, accept, cancel, or re-open any card, restart any application, or write to the live
database. My measurements were reads: the dispatch row and its events, `git log`/`git show` for his commit and
delivery note, `grep -c complete_dispatch` on his heartbeat protocol, the `SeatDispatchCompleted` events for other
seats, and `check-cards-clickable.sh` for the current state. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 29.
