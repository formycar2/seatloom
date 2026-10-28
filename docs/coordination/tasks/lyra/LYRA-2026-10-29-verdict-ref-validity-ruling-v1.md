# RULING — what makes a `flux_verdict_ref` valid evidence for an acceptance card

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-verdict-ref-validity-ruling-v1 |
| status | active |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | ruling, acceptance, evidence, supersedes-2026-10-27-ruling |

## Why this exists

Three implementations of "is this verdict about that dispatch" now exist and they disagree:

| Implementation | Rule | Effect |
|---|---|---|
| `scripts/check-verdict-refs.sh` **on track** | `payload.dispatch_id` names the dispatch | **Too strict.** Reports BAD for three pending cards that are correct. Already caused Mira to cancel and re-open cards on 2026-10-28. |
| My fix on `packet/lyra/milestone-summary` | payload **or** any `event_object_refs` row with `ref_type='dispatch'` | **Too loose.** Accepts a verdict that merely *mentions* the dispatch. |
| The subject-only reading Aegis reached for | payload **or** `role='subject'` dispatch ref | **Too strict again.** Rejects 15 accepted cards, 12 of which the previous ruling declared correct and forbade fixing. |

This ruling supersedes the 10-27 ruling on this point and names both valid evidence shapes, so the next
person does not have to reconstruct it from three disagreeing scripts.

## The rule

A card's `flux_verdict_ref` is valid evidence for that card when **all three** hold:

1. **The event exists and is the right type.** `canonical_events.id = flux_verdict_ref` and
   `event_type = 'ReviewVerdictIssued'`.
2. **The verdict is a PASS.** `payload.verdict = 'pass'`. A `fail` verdict is never evidence for
   acceptance, whatever it names.
3. **The verdict is ABOUT the card's dispatch**, by one of two shapes:

   **Shape A — single-dispatch verdict.** `payload.dispatch_id` names the card's `dispatch_id`.
   These usually also carry `role='subject', ref_type='dispatch'` for the same id. 47 of 62 cards.

   **Shape B — batch verdict.** The verdict's `subject` is a **verdict document**, and the card's
   dispatch appears as a `role='context', ref_type='dispatch'` ref on that verdict. 15 of 62 cards.

   Shape B is not an irregularity to be normalised away. It is how Flux works: one verdict document
   per batch of related fixes, each covered dispatch named as context. A rule that only understands
   shape A will report false failures on every batch verdict ever written.

## The condition no checker can enforce, stated plainly

Shape B has a hole that no join can close: a `context` ref means the verdict **mentions** the
dispatch, not that it **covers** it. A batch verdict that mentions a dispatch in passing — "this is out
of scope, see `disp-other`" — would satisfy a `context` join while being evidence for nothing.

The missing half is: **the verdict document must actually discuss the dispatch it is offered as
evidence for.** That requires reading the document, not joining a table.

**I am ruling that this half is REPORTED, not ENFORCED, until a real check exists.** A checker that
silently accepts "mentioned anywhere" is the same failure in the other direction, and this project has
already been bitten twice today by checkers that were wrong in one direction or the other. The honest
state is: shapes A and B are machine-checkable; "the document discusses it" is not, and a human or a
model must read it.

Practical consequence for whoever opens a card: **when the verdict is shape B, open the verdict
document and confirm it covers the dispatch before requesting acceptance.** That is one file read.

## What this ruling does NOT do

- It does not re-open any decided card. `acceptances` rows are immutable once decided, and the
  operator made those calls.
- It does not declare the 15 shape-B accepted cards verified. It declares them **not disqualified**.
  Whether each was actually verified requires reading its verdict document, which nobody has done
  systematically.
- It does not touch the 3 accepted cards the 10-27 ruling already flagged as evidence defects
  (`acc-disp-history-sessions-ui`, `acc-disp-ol420Y586tNIsbbeh5nlU`,
  `acc-disp-supervisor-intent-ui-wiring-plain`). Those stand as recorded: substance delivered,
  evidence pointer wrong, record owed.

## Implementation instructions

**`scripts/check-verdict-refs.sh`** must implement condition 3 as:

```sql
exists(
  select 1 from canonical_events e join dispatches d on d.id = e.payload->>'dispatch_id'
        where e.id = a.flux_verdict_ref and d.id = a.dispatch_id
  union all
  select 1 from event_object_refs r
        where r.event_id = a.flux_verdict_ref
          and r.ref_type = 'dispatch'
          and r.ref_id    = a.dispatch_id
) as about_this_dispatch
```

and must additionally **print which shape matched**, so a shape-B card is visibly a batch verdict
rather than silently passing. When the match is shape B, the output should name the verdict document so
the reader knows there is a file to open.

It must also check condition 2 — `payload.verdict = 'pass'` — which **neither current implementation
does at all**. That is a separate gap and the more dangerous one: a `fail` verdict about the right
dispatch currently passes both checkers.

## Mutation-test requirement

Any change to this checker must be tested in **both** directions before it is called fixed:

- it must still reject a card whose verdict names a **non-existent** dispatch (the wild-pointer case
  that started this, `disp-B4Y5oAhMQwtXm5Vva1fw` vs the real `disp-B4Y5oAohMQwtXm5Vva1fw`);
- it must still reject a card whose verdict is a **`fail`**;
- it must still reject a card whose verdict is **neither** shape;
- it must **accept** a batch verdict whose document genuinely covers the dispatch.

A mutation suite that only tests the population the fix was designed around will pass, and did pass,
while the fix was wrong in a direction nobody thought to test.
