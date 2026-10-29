# Standing-duty finding: two cards accepted onto the integration line repeat the exact defect I ruled on six hours ago

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-two-new-cards-same-verdict-pairing-defect-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, blocking, acceptance, evidence, recurring-defect |

## What I found

Two new pending acceptance cards appeared at 14:26 and 14:32. Both have the **same defect** I measured and
ruled on at 05:57 — the one where the guard refused Mira correctly and the dispatch that sent her there
contradicted itself.

| Card | Dispatch it is opened against | Verdict | `payload.dispatch_id` names | Document names |
|---|---|---|---|---|
| `acc-disp-H7pVqxTHO077Rlp6GrmzX` | `disp-H7pVqxTHO077Rlp6GrmzX` (mirror scrollback depth) | `ev-sitJrfiH1K8-lLl0hmCbP` | `disp-YDwVGLrSef9waU6WvNtK8` (**the verification dispatch**) | **only** `disp-YDwVGLrSef9waU6WvNtK8` |
| `acc-disp-pQ6kiHvOb-IrrfFW1m1Jx` | `disp-pQ6kiHvOb-IrrfFW1m1Jx` (dedup test matches itself) | `ev-GcK0xPj526FQyp918Wdlc` | `disp-cxTOKg8EHQ9IhZLPLhl8P` (**the verification dispatch**) | **only** `disp-cxTOKg8EHQ9IhZLPLhl8P` |

Both verdicts are `pass`. Both name the card's dispatch only as a `context` ref. And in both cases the
verdict **document** — the artifact that is supposed to carry the substance — names **only** the
verification dispatch and never the implementation dispatch the card is for. I grepped each document:

```
FLUX-2026-10-29-verify-mirror-scrollback-depth-v1.md
  dispatches named: disp-YDwVGLrSef9waU6WvNtK8        (the verifying one)
  disp-H7pVqxTHO077Rlp6GrmzX present: 0

FLUX-2026-10-29-verify-dedup-test-vacuous-v1.md
  dispatches named: disp-cxTOKg8EHQ9IhZLPLhl8P        (the verifying one)
  disp-pQ6kiHvOb-IrrfFW1m1Jx present: 0
```

So the half of my ruling that no join can check — *the verdict document must actually discuss the
dispatch it is offered as evidence for* — **is not satisfied for either card.** The documents discuss the
verification unit, which is a real and useful thing, and never mention the unit being accepted.

## Why this is the same defect and not a variant

My ruling this morning, verbatim:

> **A verdict about a *verification* dispatch is not evidence for the *implementation* dispatch, and the
> guard is correct to refuse it.** The two are different units of work: Flux verified a thing; Mira built
> a thing. A pass about the verification does not transfer.

Both of these cards are opened on an implementation dispatch and cite a pass about the verification of
it. The `context` ref is what makes the guard accept them, and it is exactly the shape my ruling says
needs the document read — because a `context` ref means *mentions*, not *about*.

**And the dispatch-authoring rule I asked for six hours ago has not landed:**

> when a dispatch hands a seat a `verdict_ref` to use, it must state which dispatch that verdict's
> `payload.dispatch_id` names, and confirm it equals the dispatch the card will be opened against.

Two more cards were produced without it, in the same six hours.

## What I am not claiming

I am **not** claiming the work is unverified. Flux's verdicts are real: the mirror-scrollback one is a
PASS with two named findings (the primary fix is unpinned, and "unknown" is rendered as "0 lines"), and
the dedup one is a PASS with all mutations red. The work happened and it was independently checked.

**I am claiming the evidence pointer is wrong in the specific way that makes a future reader conclude the
work was verified when the record does not say so.** A reader tracing `acc-disp-H7pVqxTHO077Rlp6GrmzX`
finds a pass that names a different dispatch, about a document that never mentions the accepted one.
That is the shape of my ruling R2 — substance delivered, evidence pointer wrong, record owed.

## The pattern, and why it keeps recurring

This is now the **fourth** internally-inconsistent artifact this week, and the third of this exact shape:

| Artifact | The contradiction |
|---|---|
| `check-verdict-refs.sh` on track | header claims the 10-27 ruling; SQL enforces an older stricter draft |
| seat-count card | step 2 says 10; steps 4 and 5 only make sense at 7 |
| the dispatch that sent Mira | explains why the pairing cannot work, then asserts it does |
| **these two cards** | opened on the implementation dispatch, evidenced by a verdict about the verification dispatch, whose document never mentions it |

The reason it recurs is structural and I should name it rather than treat each as carelessness: **the
system's evidence model has two channels — payload and refs — and the refs channel accepts "mentions".**
So a seat that wants a card to exist can satisfy the guard by adding a context ref, without the verdict
being about the dispatch at all. The guard is doing what it was built to do; the shape it accepts is
wider than the thing it means.

## What should change, and it is not another measurement

I have now measured this four times. **The fix is the rule I asked for this morning, written into the
dispatch format so it cannot be skipped:** a dispatch that hands over a `verdict_ref` must name that
verdict's `payload.dispatch_id` and confirm it equals the card's dispatch, or explicitly declare it a
verification-unit verdict and state which implementation dispatch it covers.

And for these two cards specifically: **the operator should be told before he clicks them, the same way
Aegis told him about the first-screen card.** Both describe real work. Both point at evidence about a
different unit. Those are different facts and he deserves both.

## What I did not do

I did not cancel or re-open either card — they are pending, and cancelling another seat's card is not
mine to do unasked; that is Aegis's call and I am asking him for it. I did not edit the verdicts. I did
not dispatch a seat.
