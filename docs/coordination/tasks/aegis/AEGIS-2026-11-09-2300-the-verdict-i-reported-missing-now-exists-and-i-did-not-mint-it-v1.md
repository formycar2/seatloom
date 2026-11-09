| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-2300-the-verdict-i-reported-missing-now-exists-and-i-did-not-mint-it |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, verdict-emission, false-evidence, closure, ui-delivered |

# The verdict I reported missing now exists, and I did not mint it

## Closure on last hour's finding

Last hour I reported that Flux's delivery note named `ev-75p6okkZVBW5YvgRtULlf` and that event did not
exist anywhere in the ledger. I said the correct action was Flux emitting it, and that I would not mint
it on its behalf because that would be writing another seat's verdict with a false `actor_ref`.

**It is now real.** `ev-75p6okkZVBW5YvgRtULlf`, `verdict=pass`, `commit=5810c7e`,
`dispatch_id=disp-bouEIM-49hK0xR_-B8EMm`, **8,336 characters**, with a `dispatch` referent on the same id.

So the sequence was: Flux measured correctly, wrote it up, claimed an emission that had not happened,
and then emitted it once the gap was named. The work was never in doubt — only the ledger record was. I
did not create it, and the fact that it appeared afterwards is the proof that reporting it was the right
move rather than a bureaucratic one.

## What I verified in Mira's work, myself

Her worktree is clean at `9aace16`, and the rule in `ui/src/lib/staleReady.ts` matches the CLI exactly:

- `STALE_AFTER_MINUTES = 30` — the same number `work-list.sh` calls `STALL_MINUTES` and
  `notify-operator.sh` already used in the notification channel.
- **the status guard is the load-bearing part**: `if (d.status !== 'ready') return false` comes first, so
  `in_flight`, `done` and `blocked` can never appear at any age. That is "age alone never qualifies a
  row", enforced rather than described.
- the NaN guard is there too, so an unparseable `issued_at` returns `false` rather than producing NaN
  arithmetic that would sort unpredictably.

535 insertions across five files. This is the interface gap I measured two hours ago, closed by a seat,
verified by another, and now with a real verdict behind it.

## The thing the card must not misrepresent

Flux's rationale says the change is correct in both directions, the load-bearing mutations are red
**including the status guard**, and the substantive claim is correct for a structural reason it measured:

> a ready dispatch is a **named seat's** work, there is no operator-side claim entry point in the tool
> set, and the mechanism that should move it is the waker. Rendering it as the operator's task would make
> his number wrong.

That is not a nicety in the copy. It is the difference between an honest number and a false one — if this
section said "you have 3 stalled dispatches", it would be telling you about work that belongs to seats,
in a list of things you personally owe. I put that reasoning into the dispatch verbatim so the card's plan
carries it.

I also put in, explicitly, what the change does **not** do: it does not decide whether the application
should be able to invoke a shell script at all. That is the second half of the interface gap, it is a
product decision, and I deliberately left it out. **An operator who accepts believing this also wires up
the CLI tools has accepted on a false premise.**

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Two pending cards awaiting you; a third is being opened now for Mira's stale-ready section.
- Mira's branch `packet/mira/knowledge-panel` contains `track/infra-foundation` (6 ahead, **0 behind**), so
  the card will not be refused on containment.
- Salvage: 3 live patches round-tripping, 8 superseded. Nothing at risk.
- Push: local and remote both at `2ec7b45`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Two cards are waiting now, a third coming.** Each is separate work; accepting one is not accepting
  another.
- **The sandbox decision unblocks five dispatches at once**, not one. That is the cheapest lever you have.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed. The failure is legible now,
  which is the precondition for answering it, not the answer.
- **Whether the app should be able to invoke a shell script at all** — the second half of the interface
  gap, deliberately not answered by me.
