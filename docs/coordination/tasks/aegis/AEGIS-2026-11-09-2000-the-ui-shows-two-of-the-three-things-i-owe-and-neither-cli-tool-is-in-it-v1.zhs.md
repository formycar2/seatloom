| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-2000-the-ui-shows-two-of-the-three-things-i-owe-and-neither-cli-tool-is-in-it |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, ui-integration, unclaimed-gap, work-list, measurement-limits |

# The UI shows two of the three things I owe, and neither CLI tool is in it

## What I measured, and the limit on it

I have carried "the CLI diagnostics are not integrated into the UI" as an undelivered gap for two days.
Today I measured it rather than repeating it.

**The limit, stated first: I cannot tell from the installed binary whether the UI section renders.**
Tauri embeds the frontend compressed, so `strings` finds the Rust side and not the interface text — the
Chinese strings `待验收`, `没有待验收的交付` and `验证方式` all return 0 hits, and that is an artifact of
compression, not evidence of absence. What I can verify is the Rust side and the source, and that is what
this note is based on.

## What is wired, verified

`strings` on `/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri` finds **`cmd_list_acceptances`,
twice**. So the acceptance-loop command is registered in the app that is running right now. The
acceptance UI itself landed on track at `2f743f5` (2026-10-10), well before the running binary was built
(2026-10-28), so the code is there in the source the binary was built from.

And in the source, `DispatchInboxView` renders both:

- `AcceptanceSection` — the pending cards, with `how_to_verify` rendered as "验证方式 · 做 A 看到 B"
- `OperatorDecisionSection` — the decisions waiting on the operator

Both cards currently waiting have a `how_to_verify` (127 and 897 characters), so there is something real
to render.

## What is missing, measured

`work-list.sh` exists because the operator's actionable obligations sit in **three** tables and each
existing surface reads one of them:

| source | in the UI? | measured |
|---|---|---|
| `acceptances` pending | **yes** | `AcceptanceSection`, live command registered |
| `operator_decisions` pending | **yes** | `OperatorDecisionSection`, `cmd_list_operator_decisions` exists |
| `dispatches` ready **and over 30 minutes** | **no** | no staleness rule anywhere in the UI |

The third one is not "less wired" — it is absent. I grepped `AcceptanceSection.tsx` and
`DispatchInboxView.tsx` for a staleness threshold and found nothing. `DispatchInboxView` sorts and
filters dispatches by status, and has no notion of "this has been ready too long and nobody has moved
it". So the question the CLI was built to answer — **one list of everything I currently owe, with where
each came from** — has no interface equivalent, and one of its three inputs has no interface
representation at all.

And neither CLI tool is reachable from the interface: `work-list.sh` and
`prepare-verification-build.sh` appear **zero** times in the running binary, and zero times in
`ui/src/` and `src-tauri/src/`. They are terminal tools with no path in.

## Why this matters, and why I am not fixing it now

The operator's own words about the product were that a thing you can only do in a terminal is a thing
that does not exist for the person who has to act on it. The acceptance cards are the clearest case:
the interface can show you a card and its verification steps, but it cannot hand you a program to run
them against — that is what `prepare-verification-build.sh` was built for, and it has no way in.

So the honest statement of the gap is narrower and more useful than the one I had been carrying: it is
not "the CLI tools are not integrated". It is **the third obligation source is missing from the
interface entirely, and neither tool has a way in**.

I have not started it. This is a UI change to the surface the operator accepts on, and it should be
dispatched and verified like anything else rather than written by me in a heartbeat. The measurement is
the deliverable this hour; the work is the next dispatch, and it should go to Mira with the three-source
list as its spec.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Two pending cards, both clickable, both merge clean into current track.
- Salvage: 2 live patches round-tripping, 7 superseded. Nothing at risk.
- Push: local and remote both at `0610cc6`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Two cards are waiting**, and they are different work.
- **The sandbox decision unblocks five dispatches at once**, not one. That is the cheapest lever you
  have.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed. The failure is legible now,
  which is the precondition for answering it, not the answer.
- **The UI gap above** is now measured and dispatchable. I have not started it.
