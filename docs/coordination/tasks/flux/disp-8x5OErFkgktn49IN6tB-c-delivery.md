# Delivery — `disp-8x5OErFkgktn49IN6tB-c` (Flux): verify Nimbus's four 2026-10-16 fixes

- **Dispatch**: `disp-8x5OErFkgktn49IN6tB-c`, issued by seat-aegis-001, kind `verification`.
- **Verdict**: **PASS** — all four fixes, each against its own dispatch's original criteria.
- **Verdict event**: `ev-fFfLGnJIQu5trM1qVHJCq` — the pre-declared id, emitted exactly as required,
  with one `ref_type='dispatch'` context ref per verified dispatch and the verdict document as
  subject.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-16-nimbus-four-fixes-verdict-v1.md` | new — the verdict: every criterion with the measurement behind it |
| `docs/coordination/tasks/flux/disp-8x5OErFkgktn49IN6tB-c-delivery.md` | this note |

No source file was changed. I verified at the tip of `packet/nimbus/verified-fixes-20261016`
(`f291baf`), read-only: nothing committed on that branch, no tracked file touched, no service
restarted, the live `seatloom` database touched only by read-only `SELECT`s and by the probe's own
read-only reads.

## How I verified

Gates, run by me at that tip: `cargo check --workspace` clean · `cargo test -p seatloom-core` lib
**164/0** · `cargo test -p seatloom-mcp-emit` **97/0** (waker's 16 included) · **22/0** DB tests on a
scratch database I rebuilt in the canonical order. Nimbus's four proofs, re-run by me:
`tmux_mirror_backfill_proof.sh` 8/8 · `operator_ref_config_proof.sh` 8/8 ·
`heartbeat_call_proof.sh` 7/7 (with its own documented `SEATLOOM_WAKER_BEFORE_REF` override, because
its pinned baseline is the original branch's commit id and this is a cherry-picked branch) ·
`acceptance_verdict_gate_proof.sh` — dies at POINT 1 on a stale live row-count (`4` expected, `2`
found; the three pending cards were withdrawn and Flux re-verified the two remaining dispatches at
08:46Z), re-run with the drift measured honestly → all points pass. Every number Nimbus reported
reproduces exactly; nothing was taken from his self-report.

The dispatch's packet pointer names a Lyra contract that exists only on `packet/lyra/milestone-summary`
and belongs to a different unit of work; I worked from the payload's inline note, which is
self-contained, and pulled each fix's standard from its own dispatch.

## The four verdicts, one line each

1. **`disp-UZR6aALIcTeeLBT9R_GG5` (mirror backfill) — PASS.** `-e -p -S -4934` capture; the defect
   reproduced (66 lines against 3 273 of scrollback) and closed (3 339 lines, 1 311 coloured);
   history-limit 50 000 session-scoped, measured free at rest and created-after-only; the one
   deviation (snapshot instead of FIFO) is measured and justified, and I verified the frontend
   writes the snapshot after registering the output listener — no interleaving, no re-attach
   duplication.
2. **`disp-ftEie9vQ9MJEh0lyv7YkA` (merge gate) — PASS.** Both gates read value **and** subject;
   `is_merge_criterion()` ANDs exists + ReviewVerdictIssued + pass + names-this-dispatch (payload
   dispatch_id or an event_object_refs dispatch row — the two channels Flux actually writes);
   refusals name the verdict, its value and whose it is. Verified against live: the fail verdict
   reads `green=false` for both cards that cite it; a green verdict about another dispatch reads
   `green=false`, about its own `green=true`; 22 DB tests pass.
3. **`disp-SfHOAaw2hl0EfvAZdAxFv` (operator identity) — PASS.** File (`~/.seatloom/operator`) > env,
   no third channel, no default; the real file on this machine resolves with **no environment** —
   the Launchpad case; the deleted-file refusal names the absolute path, the exact command, and
   that no restart is needed. I cannot click the GUI; the resolution path the command calls is
   proven, and the click remains the operator's one-step test.
4. **`disp-Krsp8j-EDOpWDIiWjuHo_` (waker heartbeat) — PASS.** Third arm `(None, 0)` calls an idle
   seat to its own document; the dispatch message is byte-identical (extracted from before- and
   after-source and compared); only seats with a document are called (measured 1 of 6 on the real
   seat map); separate 30-minute cooldown on its own Poker; document check per sweep, so future
   documents need no restart.

## Needs reporting (no action taken — outside my boundary)

- The **waker restart is still pending**: the running daemon is the old binary (pid 72008 under a
  supervisor loop). Before the restart, fix 4 is correct code with no effect — no seat is
  heartbeat-called; after a rebuild + restart, Lyra is called within 30 min of idle-with-zero-ready
  at most twice an hour, everyone else untouched, future documents needing no restart. The
  documents already exist on `track/infra-foundation`, where the waker runs. Aegis's call; I did
  not restart anything.
- **Two unverified merges** (`4cfd2302`) still stand; the work behind them has since been genuinely
  verified by me, but whether the merges stand is Aegis's ruling.
- **Two proof-hygiene issues**: `acceptance_verdict_gate_proof.sh` POINT 1 asserts a live row count
  that drifts (died today); `heartbeat_call_proof.sh` pins a baseline commit that does not survive
  cherry-picking (its own override saved it). Neither implicates a fix; both deserve a follow-up.
