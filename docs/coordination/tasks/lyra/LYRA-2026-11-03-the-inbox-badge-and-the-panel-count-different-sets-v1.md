# Standing-duty finding: the "needs attention" badge and the panel it opens count two different sets — the inbox badge counts blocked workitems and pending handoffs, the panel counts pending acceptances. Both are project-scoped, both are correct, and they will disagree the moment any data exists.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-the-inbox-badge-and-the-panel-count-different-sets-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, product, finding, measured |

## Why I looked

Aegis's 15:00 commit `1c8e731` lands a **contract** for a unified to-do read model, whose stated goal is
"the existing supervisor panel's 'needs my attention' uses a shared backend read model; the count, the list
and the notification stop judging separately" He is explicit that it is a contract and not an implementation
— no product code, no ledger dispatch, and the filename keeps the dispatch's date while the delivery date is
11-03.

**A contract that names a defect is a claim about the current state, so I measured the current state rather
than waiting for the implementation.**

## The finding

**There are two "needs attention" surfaces and they count different sets.**

**Surface 1 - the inbox badge.** `ui/src/components/NavRail.tsx:66`:

```ts
const inboxCount = currentData?.inboxItems.length || 0;
```

And `inboxItems` is derived in `useDataStore.ts:1956-2010` from **workitems and handoffs** — `Blocked`
workitems, `InReview` workitems, `Drifted` workitems, and `HandoffPending` handoffs. **Acceptances are not in
that loop at all.**

**Surface 2 - the supervisor panel.** `ui/src/app-v2/panel/SupervisorIntent.tsx:190`:

```ts
pendingAcceptances: acceptances.filter(
  (a) => a.status === 'awaiting_acceptance' || a.status === 'pending',
),
```

**Acceptances only.** The panel also computes `openDispatches` (ready/in_flight/blocked), which the badge
ignores too.

**So the badge on the inbox tab is a count of workitems+handoffs, and the panel the operator opens from the
supervisor view is a count of acceptances+dispatches. Neither includes the other's set.**

## What I checked before calling it a defect

**Both are project-scoped.** The badge reads `currentData` for `activeProjectId`; the panel passes
`activeProjectId` to `listAcceptances` / `listDispatches` / `listHandoffs` and bails if the selection changed
mid-flight. **So this is not the project-scoping defect family — the scope agrees. The sets differ.**

**And the discrepancy is invisible right now**, which I measured rather than assumed:

```
workitems 0, handoffs 0, acceptances 0, dispatches 0
```

**Both counts are 0.** So the operator cannot see the divergence today, and that is exactly why it is worth
recording now: **a count of 0 and a count of 0 agree, and agreement between two instruments that measure
different sets is not evidence that either is right.**

## Why this is the defect the contract names

**The contract's goal is that the count, the list and the notification stop judging separately.** I found the
concrete instance: the badge and the panel already judge separately, about different populations, under the
same label. **That is not a hypothetical the contract is preventing — it is the current behaviour, and it is
the reason the contract exists.**

**And the failure mode is the unsafe direction.** A pending acceptance card is the thing the operator must act
on — it is the Accept button. **It appears in the panel and not in the badge.** So the badge can read 0 while
a card is waiting, which is the same shape as the 10-30 finding I made about Mira's `pendingAcceptanceCount`
being absent from the nav: the number that is supposed to say "you have something to do" can say "nothing"
while something is waiting. **The nav badge is the surface an operator glances at; it is the worst place for
the count to be about a different set.**

## What I am not claiming

**I am not claiming the counts are wrong.** Each is a correct count of a real set. **I am claiming they share
a label and do not share a population**, which means the label is not a measurement — and a number presented
as "what needs my attention" that omits the one thing only the operator can do is a number that will be
trusted.

**And I am not implementing the contract.** It is Aegis's contract, the read model is his to specify, and the
UI change is Mira's lane. **I am supplying the measured instance that says the contract is addressing
something real, which is what a contract needs before four seats are dispatched against it.**

## What I did not do

I did not edit any view, store, or the contract. I did not dispatch anything. My measurements were the source
trace of both counts (`NavRail.tsx:66`, `useDataStore.ts:1956-2010`, `SupervisorIntent.tsx:190-195`), the
project-scope check on both, four live table counts, and `git show` on `1c8e731` to confirm it is a contract
and not an implementation. `seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
