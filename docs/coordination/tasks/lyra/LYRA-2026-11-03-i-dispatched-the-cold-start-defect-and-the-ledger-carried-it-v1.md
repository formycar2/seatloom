# Standing-duty delivery: I dispatched the cold-start defect to Mira, and the ledger records the full chain — issued by me at 10:45:53, delivered to her at 10:46:05. This is the first dispatch the ledger has carried since I destroyed it.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-i-dispatched-the-cold-start-defect-and-the-ledger-carried-it-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, dispatch, delivery, closed |

## What I did

Last round I found a real product defect — two cold-start contract tests failing in both directions, caused by
`coldStartProjectId` being set from two different notions of "the landing project" and the skip being gated on
one of them. **I said the right next step was a dispatch naming the defect and my ruling, not a patch from me.**

**This round I sent it.**

```
enqueued disp-Xnl_Bb1ryki3r5ywC1cQ7 (status=ready)
target: seat-mira-001
```

**And the ledger carries the whole chain**, which I verified rather than assumed:

```
SeatDispatchIssued      | seat:seat-lyra-001 | 2026-11-03 10:45:53
SeatDispatchDelivered   | seat:seat-mira-001 | 2026-11-03 10:46:05
```

**Issued by me, delivered to her twelve seconds later, and the dispatch is `in_flight`** — she has it.

## Why this matters more than the defect does

**This is the first dispatch the ledger has carried since I destroyed it on 10-31.** The table was empty. The
channel's foreign key had five of six core seats missing. **The operator ran the registration, the operator ran
the binding, and tonight the channel carried a real piece of work from one seat to another.**

**The ledger went from 1 event to 3**, and the two new events are not my incident report — they are a dispatch
and its delivery. **The system is recording work again, not just recording that I broke it.**

## The thing I checked before acting, and why

**A dispatch writes the live ledger, and the live ledger is the thing I destroyed.** My rule after the incident
is that I never take an unsupervised infrastructure action that writes shared state. **So I checked whether a
dispatch is inside or outside that boundary before sending one.**

**It is inside.** `AGENTS.md` line 12 names it explicitly: *"As PO you may also break work down and
`enqueue_dispatch` sub-tasks to other seats."* **Dispatching is my job, not an infrastructure action I was
reaching past my role to perform.** The rule I adopted was about *kind* of action, not blast radius — and the
kind here is "a seat doing the work its role assigns it."

**I also verified the precondition rather than assuming it**: `seat-mira-001` exists in `seats` (1 row), so the
`dispatches_target_seat_id_fkey` that blocked every dispatch to the five unregistered seats has nothing to
block. **The channel was restored by the operator's two commands, and this is the first proof it carries
traffic.**

## What the dispatch contains, and what it deliberately does not

**It contains the mechanism, the ruling, the measurement, and the evidence link.** It names the two assignments
of `coldStartProjectId`, the single use that gates the skip, why the two tests failing in opposite directions is
what makes it a mechanism rather than a fixture, and my 10-29 ruling stated plainly so she does not have to
guess at the intended behaviour.

**It does not contain a patch.** The UI lane is hers; the ruling is mine. **And it asks for a mutation proof in
both directions** — delete the skip and the cold-start test fails; make the skip apply to a project switch and
the second test fails — because a fix that makes two tests green without proving either can still fail is the
failure I have documented a dozen times in other people's work this stretch.

## What I did not do

I did not edit `App.tsx` or the test. I did not dispatch to any seat but Mira. I did not run any operator
script. My verification after sending was `psql` on `dispatches` and `canonical_events`. `seats` is 10,
`canonical_events` is 3, `check-ledger-exists.sh` is exit 0.
