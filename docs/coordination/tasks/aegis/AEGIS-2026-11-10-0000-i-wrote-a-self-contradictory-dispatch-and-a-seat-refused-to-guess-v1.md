| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-0000-i-wrote-a-self-contradictory-dispatch-and-a-seat-refused-to-guess |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, own-error, self-contradictory-dispatch, acceptance-card, honest-plan |

# I wrote a self-contradictory dispatch, and a seat refused to guess

## My error, stated first

I issued `disp-zJpydAwfyrRj1VPSJj0Q6` to Mira with two instructions that cannot both be satisfied:

1. the card's plan **must state** that the change does not decide whether the application can invoke a
   shell script
2. **do not touch the cards**

The only way to add a statement to a pending card is `cancel_acceptance` followed by `request_acceptance`.
I verified the toolset myself: `cancel_acceptance`, `get_acceptance_lineage`, `request_acceptance` — and
**no edit tool**. So instruction 1 requires exactly what instruction 2 forbids.

I also issued the same request twice. `disp-vDUTp_4rWlPjqD8TwRoAy` had already asked for the card, and she
had already opened it. My second dispatch was a duplicate.

## What Mira did, which was right

She did not cancel her own card, because `pending -> cancelled` is terminal and doing it against an
explicit prohibition was not her call. She did not open a second card, because two cards for one dispatch
would leave you choosing between them, which is worse than one incomplete one. She **reported the gap with
the exact text that is missing** so it could be added by whoever has the authority.

That is the correct handling of an impossible instruction, and it is better than either way of "succeeding".

## The gap, and why it is not cosmetic

Her card `acc-disp-bouEIM-49hK0xR_-B8EMm` is pending, clickable, on the right branch and base, referencing
a real passing verdict. But its plan contains **zero mentions of a shell script**. The missing statement,
in her words:

> It **does not decide whether the application can invoke a shell script.** `work-list.sh` and
> `prepare-verification-build.sh` still have no path in from the interface — that is the second half of the
> interface gap, it is a product decision belonging to the operator, and I deliberately did not touch it.
> Accepting this card does **not** mean the CLI tools are wired into the interface.

**The risk is specific: you could accept this card believing it also wires the CLI tools into the
interface. It does not.** That is a false premise for an Accept, which is the one thing the whole card
mechanism exists to prevent.

## What I did about it, and what I did not

I verified her claim rather than taking it: no edit tool exists, and I am the issuer of the dispatch
(`issued_by=seat:seat-aegis-001`), so a governed cancel would be available to me.

**I did not cancel it.** Cancelling is terminal for that card id, and the decision is yours: an informed
Accept is possible if you have the statement, and you now have it — in this note and in her delivery note,
which is committed on her branch. Taking an irreversible action on your behalf, to fix a contradiction I
created, would be the same category of error as the contradiction itself.

**So the card is waiting for you, and this is the statement it is missing.** If you want it inside the card
rather than beside it, say so and I will cancel and re-request with it included.

## The work itself is verified, independently, on the mainline

Per the rule that a merge is the first thing rather than the last, I ran Mira's work against current
track in an isolated worktree:

| check | result |
|---|---|
| merge `track` into her branch | clean, 0 behind afterwards |
| `tsc --noEmit` on the merged tree | **0 errors** |
| `StaleReadySection.test.tsx` | **10 passed** |
| full UI suite on the merged tree | **32 files / 401 tests passed** |
| Mira's own worktree | untouched, clean |

Her worktree is at `f554ac0`, clean. The rule itself is right: `STALE_AFTER_MINUTES = 30` matching the CLI,
the status guard first so `in_flight`/`done`/`blocked` can never appear at any age, and a NaN guard on
`issued_at`.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Three pending cards awaiting you: Nimbus's `db error` fix, Vela's doc-gate change, Mira's stale-ready
  section. All clickable.
- Salvage: 3 live patches round-tripping, 8 superseded. Nothing at risk.
- Push: local and remote both at `d7c6dbe`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Three cards are waiting, and Mira's is missing one statement you should read before accepting.** It is
  quoted above and it is the only thing wrong with that card.
- **Say whether you want it inside the card.** If yes, I cancel and re-request with it. If no, you have it
  here and an informed Accept is still possible.
- **The sandbox decision unblocks five dispatches at once**, not one.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed.
