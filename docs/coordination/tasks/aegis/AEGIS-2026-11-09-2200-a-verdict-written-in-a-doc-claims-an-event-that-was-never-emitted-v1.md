| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-2200-a-verdict-written-in-a-doc-claims-an-event-that-was-never-emitted |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, verdict-emission, false-evidence, checker-scope, ui-delivered |

# A verdict written in a document claims an event that was never emitted

## What I found

Flux delivered a PASS verdict for Mira's stale-ready section. Its delivery note says, in its own words:

> Emitted as `ev-75p6okkZVBW5YvgRtULlf` with `payload.dispatch_id = disp-bouEIM-49hK0xR_-B8EMm` and a
> `role='subject'` referent on the same dispatch.

**That event does not exist.** I checked every table that could hold it:

| table | rows matching `ev-75p6okkZVBW5YvgRtULlf` |
|---|---|
| `canonical_events` | **0** |
| `artifacts` | **0** |
| `acceptances` (as `flux_verdict_ref`) | **0** |

The newest `ReviewVerdictIssued` in the ledger is `ev-nRH3MeASZN_sK3E6Up1Ku`. So the verdict was
**measured, written up, and never emitted** — and its hygiene section says "no live-database write beyond
the verdict event", asserting a write that did not happen.

## Why the checkers did not catch it

`check-verdict-refs.sh` exits 0, and it is not wrong. It reads `acceptances.flux_verdict_ref` — the refs
that **cards** carry. Flux's claim lives in a markdown file, and no card references that id, so there is
nothing for the checker to join against.

So the check that exists protects the operator's Accept, and it does. What it cannot see is a **claim** of
emission in a delivery note, which is a different surface. And the difference matters here because the
work itself is good: Flux measured three mutations of `staleReady.ts` (status guard removed, threshold
changed, NaN guard removed), each reverted and confirmed, plus `tsc --noEmit` clean and 401 UI tests
passing. **The verification is real; the ledger record of it is not.**

## What this is, in the vocabulary already established

There is a lesson recorded in this repo: **a verdict written in the pane is not a verdict.** This is the
next instance of it — a verdict written in a *document*, naming an event id that was never minted.

It is more dangerous than the pane version, because a pane claim evaporates with the session, while a
document persists and carries a specific id. Anyone reading it later — a seat, me, you — would reasonably
conclude a verdict exists and go looking for it, or worse, cite it.

## What I did, and did not do

**Did:** captured Flux's uncommitted delivery note into the salvage store so the work is protected;
marked Mira's patch superseded because her work is now committed at `5810c7e`; verified all six protocol
checks return to exit 0.

**Did not:** fabricate the missing event. Emitting a `ReviewVerdictIssued` on Flux's behalf would be
writing another seat's verdict, which is exactly the thing I must not do — the verdict belongs to the
seat that measured, and its `actor_ref` would be a lie if I minted it.

The correct action is Flux emitting it. I have not dispatched that yet, because the finding should reach
you first: **a seat's delivery note asserted a ledger write that did not happen, and the check that
should have caught it cannot see delivery notes.**

## The delivered work, which is real

Mira finished the interface gap I measured two hours ago. `5810c7e` on `packet/mira/knowledge-panel`,
with `StaleReadySection.tsx`, its test, and a `staleReady.ts` helper. Flux verified it in an isolated
detached worktree with three mutations, and its framing claim — that a stalled queue is a **system**
problem rather than the operator's task — is correct for a reason the commit did not state.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Two pending cards awaiting you; Mira's work is committed and verified but has **no card and no emitted
  verdict**.
- Salvage: 3 live patches round-tripping, 8 superseded. Nothing at risk.
- Push: local and remote both at `dff429f`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Two cards are waiting**, and they are different work.
- **Mira's stale-ready section is done and verified but cannot reach you** — no emitted verdict, so no
  card. That is the same gap Vela hit, from the other side.
- **The sandbox decision unblocks five dispatches at once**, not one.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed.
