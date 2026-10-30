# Standing-duty finding: the red line is not blocked on a decision, it is blocked on a missing link in the chain — and I own one of them

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-card-chain-has-a-missing-link-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, blocking, acceptance, chain, delivery |

## What I got wrong last round

Last round I told the operator the red UI line was "waiting on one merge, and nothing else." **That was
wrong, and it was wrong in the way that makes it useless to him:** it implied the decision was his and
everything upstream was ready. The merge is the last link. **Three links before it do not exist.**

## The actual chain, and where it breaks

The operator's merge behaviour is card-driven, and I verified that rather than assuming it:

```
4cda353 Merge branch 'packet/onyx/topic-opened-019'   <- came minutes after
acc-disp-p-zIkNv1Z1jbUQk4voEfr was granted at 08:41
be898cb Merge branch 'packet/nimbus/project-scoped-reads'   <- after acc-disp-kBqZCgemDbYwcHglVppbn
```

So the chain is:

```
[1] a verification dispatch to Flux        <- MISSING
[2] Flux emits ReviewVerdictIssued, pass, naming the dispatch
[3] a seat calls request_acceptance        <- needs [2]
[4] the operator grants the card           <- he does this, and does it fast
[5] the merge lands                        <- then the line goes green
```

**Link [1] is missing.** I grepped for any dispatch since 10-29 whose title touches persist, chooseTab,
`:489`, or restore. The only two are **mine to Mira** — `disp-EVFMbz9OxT2gyoFlYfzBE` and
`disp-faEWBt8kSGgv-6pJs_Rqw`, both `done`. **Neither is a verification dispatch, and neither can produce a
verdict**, because `request_acceptance` requires a `flux_verdict_ref` that names an existing event with
`verdict='pass'` about the dispatch.

Consequence, measured:

```
pending cards:      0
accepted today:     2   (neither is the UI work)
acceptances rows for the UI work:  none
```

**So the operator granted two cards this morning, saw his queue go to zero, and merged the branches those
cards covered. There was never a card in front of him for the UI work.** He did not decline it. It was
never offered.

## The part I own

**I issued two dispatches into this chain and neither of them produced the link that unblocks it.** I
dispatched the implementation and then the missing test. Both were right and both were done. **What I never
did was dispatch the verification** — the one that turns completed work into something the operator can
accept.

And I have the authority: my standing instructions say *"As PO you may also break work down and
`enqueue_dispatch` sub-tasks to other seats"*, and the tool lists Lyra as an issuer. **I used that authority
twice this morning and stopped one link short of the thing that mattered.**

## What I am doing about it

Dispatching Flux to verify Mira's branch, with the verdict declared in advance so the tool mints the id and
the verdict can name it. **That is link [1], and it is mine.**

## What I am not doing

I am **not** opening the card myself. `request_acceptance` requires the dispatch to be a *completed dispatch
of the calling seat* — and more to the point, a PO opening an acceptance card for another seat's work
without a verdict in front of him would be exactly the "accepted on false premises" pattern I reported on
2026-10-29. **The card comes after the verdict, not instead of it.**

I am **not** merging. That is his action, and it is the one action in the chain he has never once declined.

## The honest summary of where this stands

The work is done and verified. The chain that delivers it has a hole in it at link [1]. **I put two links
in and stopped one short, and then wrote to the operator that everything upstream was ready.** The
correction is one dispatch to Flux, and it should have gone out when I dispatched the missing test three
hours ago.
