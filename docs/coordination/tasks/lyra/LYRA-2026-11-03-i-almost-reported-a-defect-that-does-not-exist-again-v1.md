# Standing-duty correction: I was two steps from publishing a second false defect — I found that the UI's inbox filter matches 3 of 9 workitem statuses and started building a finding, then measured the mapper and the seed and found both halves of my concern were wrong. The case mismatch is normalized, and the 6 uncounted statuses are correctly not actionable.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-i-almost-reported-a-defect-that-does-not-exist-again-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, retraction, near-miss, correction |

## The near-miss, in order, because the order is the lesson

**Last round I reported a real finding:** the inbox badge counts workitems+handoffs while the supervisor panel
counts acceptances+dispatches — two sets under one label. That finding stands.

**This round I followed it one step further and started building a second finding on top of it.** The chain I
was constructing:

1. The inbox derivation matches `w.status === 'Blocked'`, `'InReview'`, `'Drifted'`.
2. The UI's `WorkItemStatus` type declares **nine** values.
3. **Therefore six statuses are silently dropped from the count.**

**And it got worse, which is what made it feel like a real finding:** the seed writes lowercase
`'done'` while the UI type declares `'Done'`. **A case mismatch between the database and the UI looked like
the mechanism that would make the whole comparison fail.** I was one measurement from publishing "the inbox
counts nothing, because the status vocabularies don't even match case."

## Both halves were wrong, and I only found out by finishing the measurement

**Half one — the case mismatch.** I read `workitemFromDto` in `ui/src/lib/dto-to-v1.ts:196` and found the
status does not pass through raw. It goes through `coerceWorkitemStatus`, which consults
`WORKITEM_STATUS_MAP` (`dto-to-v1.ts:80`):

```ts
const WORKITEM_STATUS_MAP: Record<string, WorkItemStatus> = {
  draft: 'Draft',  ready: 'Ready',  active: 'Active',  blocked: 'Blocked',
  in_review: 'InReview',  verified: 'Verified',  done: 'Done',
  reopened: 'Reopened',  drifted: 'Drifted',   // keyed on backend, returns UI
};
```

**The map is keyed on the backend's lowercase values and returns the UI's Capitalized ones.** `'done'` →
`'Done'`. **The case mismatch I was about to report as a defect is normalized at the boundary, deliberately,
with a table.** There is no mismatch. My "mechanism" was reading a value at one layer and comparing it to a
value at another.

**Half two — the six dropped statuses.** Once the mapping is correct, the question becomes whether
`Draft`, `Ready`, `Active`, `Verified`, `Done`, `Reopened` *should* be in a "needs my attention" count. **They
should not.** A Draft nobody has started, a Ready item in a queue, an Active item a seat is working, a Verified
item awaiting nobody, a Done item, and a Reopened item that has been picked up again — **none of those is
waiting on the operator.** The three that are counted (`Blocked`, `InReview`, `Drifted`) are exactly the three
that are.

**So the filter is not dropping six statuses. It is selecting three, correctly, and the type declaring nine is
a vocabulary and not a to-do list.**

## Why I am writing this instead of just dropping it

**This is the second time in three days I have been one measurement from publishing a defect that does not
exist** — the first was `check-ledger-exists.sh` on 11-02, where I reported a missing branch and had not read
far enough.

**Both times the shape was identical:** I found a *discrepancy between two layers* and treated the
discrepancy as the defect, without checking whether something between the layers reconciles them. **A
vocabulary of nine and a filter of three is a discrepancy. A map that converts one to the other is not a bug —
it is the thing that makes the discrepancy harmless.** I went looking for the defect at the point where two
representations differ, which is precisely where a correct system is *supposed* to differ.

**And the second half is worse than the first.** Last time I published before measuring. **This time I
measured both halves before publishing — and only because I did, I found out I was wrong.** That is the
discipline working. **The lesson is not "be more careful," it is "finish the measurement," because the
measurement is what caught me, not the caution.**

## What survives from last round

**The finding I published stands and is unaffected.** The inbox badge counts workitems+handoffs; the panel
counts acceptances+dispatches; they share the label "needs attention" and not the population. **None of that
depends on the status vocabularies** — the two sets differ in *which tables they read*, not in how they filter
statuses. **I checked, rather than assuming my correction invalidated my earlier finding.**

## What I did not do

I did not publish the second finding. I did not edit any mapper, type, or store. My measurements were the
inbox derivation's three comparisons, the `WorkItemStatus` union, the seed's actual status strings, the
`git grep` for a writer of `'Blocked'` outside a probe example, `workitemFromDto`, `coerceWorkitemStatus`, and
`WORKITEM_STATUS_MAP`. `seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
