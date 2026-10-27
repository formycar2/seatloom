# Delivery: combined project-filtering acceptance card prepared

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-19-project-filtering-combined-card-v1 |
| status | delivered; pending operator decision |
| author | nimbus |
| date | 2026-10-19 |
| version | v1 |
| tags | acceptance, project-filtering, combined-candidate |
| dispatch | disp-LwXIdFy2vsKCnBnQVWpqs |

## Verified prerequisites

Read the entire evidence document
`FLUX-2026-10-19-project-filtering-combined-verdict-v1.md` and the ledger event
`ev-K3Zfw40dtF7AQfLLh_kTY`: ReviewVerdictIssued, actor Flux, verdict pass,
payload dispatch_id and context dispatch reference both name
`disp-9H3OCmprBFAkiYBa5MDwE`. The implementation dispatch is done and owned by Nimbus.

Candidate remains `packet/nimbus/project-filtering-combined` at `4149b7f`, product
`2c69adb`. Track remains `98876aa`; ancestor check returned 0. No synchronization
or product change was necessary. Original delivery branches were preserved.

## Governed actions and readback

Created `acc-disp-9H3OCmprBFAkiYBa5MDwE`, gating the combined implementation dispatch,
branch `packet/nimbus/project-filtering-combined`, base `track/infra-foundation`,
Flux verdict `ev-K3Zfw40dtF7AQfLLh_kTY`. Read back via get_acceptance_lineage:
status pending, correct branch/base/verdict, seven Chinese steps, seven expected
outcomes, six preconditions. All requested scenarios are present: SeatLoom versus
QL membership; unbound corpus explanation; rapid switching; late-response isolation;
hidden-not-destroyed mirrors; visible global runtime-artifact disclosure.

Cancelled only Nimbus's owned pending old card `acc-disp-_Xfb_jysuptYTOT4pCBx_`.
Readback is cancelled with the supersession reason naming the new combined card.
This creates no rework and performs no merge.

Read Mira's card `acc-disp-RUnXBR3uICuWmmFizJ7vX`: still pending, old branch
`packet/mira/knowledge-panel`, owner `seat-mira-001`. Did not attempt cancellation.
Aegis must coordinate its withdrawal with Mira to avoid offering the superseded
separate-branch card alongside the combined candidate.

## Scope retained in the card

Copied Flux's exact does_not_assert text into a precondition:

> the rendered UI and the IPC path (screen locked from the seat's shell; no app
> installed, no running app touched, per the dispatch); the migration's effect
> (a live act, not applied); the numbers as permanent assertions.

Chinese preconditions explicitly preserve these exclusions. Runtime artifacts remain
global with visible disclosure; the migration was not applied. Counts are verification
snapshots, not perpetual assertions. Equal legitimate counts alone are not a rejection:
wrong project provenance is. Nothing claims rendered-app, IPC or click-level success.

## Boundaries

No card re-opening retries, installation, track merge, gate event, raw live database
write, session restart or model switch. Only sanctioned request_acceptance and
cancel_acceptance performed governed lifecycle writes. This delivery adds documentation
only; no new product behavior needing a changed Flux verdict.
