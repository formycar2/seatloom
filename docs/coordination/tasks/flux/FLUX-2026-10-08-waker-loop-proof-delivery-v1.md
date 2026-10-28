# Task: Acceptance proof for the autonomous governed-dispatch pickup loop (DB + binary layer)

Note closing dispatch `disp-flux-loop-proof` (delivered to `seat-flux-001`). The loop `enqueue -> pull -> do -> complete -> idle` is now backed by a reproducible acceptance script, so the capability is provable rather than asserted.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | FLUX-2026-10-08-waker-loop-proof-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-08 |
| to | lyra |
| version | v1 |
| governing dispatch | `disp-flux-loop-proof` (autonomous governed-dispatch pickup loop) |
| modelled on | `crates/seatloom-mcp-emit/proof/step6_dispatch_proof.sh` |
| artifact | `crates/seatloom-mcp-emit/proof/waker_loop_proof.sh` |
| run location | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/flux` (branch `packet/flux/dispatch-acceptance`) |
| scratch database | `seatloom_mcp_waker_proof` (podman `seatloom-postgres`; never the live `seatloom` DB) |
| tags | flux, delivery, governed-dispatch, g19, p0, acceptance, proof |

---

## One-line summary

`crates/seatloom-mcp-emit/proof/waker_loop_proof.sh` drives the real `seatloom-mcp-emit` binary over stdio against the scratch database `seatloom_mcp_waker_proof` (schema 001-012 applied via `scripts/apply-db.sh`, so the canonical ordering is reused rather than re-derived) and asserts 42 facts: an enqueued dispatch is immediately `ready`; a seat identity (`SEATLOOM_ACTOR_REF`) pulls it and receives title + payload marked `in_flight`; `complete_dispatch` marks it `done` and leaves exactly the three lifecycle events (`SeatDispatchIssued` / `SeatDispatchDelivered` / `SeatDispatchCompleted`) in `canonical_events`, making the row replayable from the ledger alone; and one-in-flight is enforced by the `uq_dispatches_one_in_flight_per_seat` partial unique index, by the pull semantics (a second pull resumes the same item), and by ownership (a seat cannot complete another seat's work, and a rejection writes no event).

## Scope

DB + MCP-binary layer only, as dispatched. Not covered: the `seatloom-waker` daemon (`LISTEN dispatch_ready` -> nudge an idle seat's tmux pane) and the seat runtime (pi). No tmux, no live seat, and no live database was touched.

## Result

42 passed / 0 failed, exit 0, reproduced from a dropped-and-rebuilt scratch database (`SEATLOOM_PROOF_KEEP_DB=1` reuses it). The script refuses to run against any database whose name does not advertise itself as scratch, and rebuilds the binary if `target/debug/seatloom-mcp-emit` is missing.
