# Delivery — `disp-ySsxkvHxKurEgKpt7Q_Hm` (Flux): the two system-hole fixes, verified

- **Dispatch**: `disp-ySsxkvHxKurEgKpt7Q_Hm`, issued by seat-aegis-001, kind `verification`.
- **Covers**: `disp-ibABeNk-nkEDAkI6Qj2sG` (blocked visibility) **and** `disp-b5VbRor1fHhP3nN4YZI_l`
  (request_acceptance checks the verdict's subject) — both were verified, so both are named in the
  verdict's `event_object_refs`.
- **Verdict**: **PASS for both fixes** — with one finding that outranks them: the deployed MCP binary
  is 6½ hours older than the hole-2 fix, so that hole is live in the running system.
- **Verdict event**: `ev-pnBnPX1pdaRysj8UYW077` — the pre-declared id, emitted exactly as required.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-17-two-system-holes-verdict-v1.md` | new — the verdict: every case with its exact input and output |
| `docs/coordination/tasks/flux/disp-ySsxkvHxKurEgKpt7Q_Hm-delivery.md` | this note |

No source file was changed. Per the dispatch's boundary C: no cards opened, no live writes, no gate
events, no merges. The hole-2 experiment ran entirely on a scratch database (`seatloom_hole2`, built
in the canonical order). The only side effect outside my worktree was a temporary local branch
`hole2-sync-branch` in the main checkout (a ref at track's tip, needed because the containment check
requires a branch that contains the base), deleted after the run.

## Hole 1 — the premise checked myself, then the view

Aegis measured the empty payload keys; I measured the table before trusting Nimbus's "half real":
`dispatch_dependencies` carries the edges for every blocked dispatch (two for `disp-session-key-ui`,
one each for the others). So the structured answer **did** already exist and the missing half was the
fleet-wide view. `list_blocked_dispatches` at `fcbd612` answers it in one call against live — 3
blocked dispatches, 4 blocking edges, each `waiting_on=<event> kind=event exists=false`, with
`blocking_only` distinguishing "waiting on X" from "X already landed". Blocked-ness is **computed**
from the edges, so a reason-less block cannot be stored (pinned by test) — a stronger shape than the
"refuse or require" the dispatch asked for, recorded as a deviation from the letter. Nothing to
backfill (every block already carries its edges). Not precomputed: the waiting duration (`issued_at`
is there; the subtraction is yours).

## Hole 2 — where the code is, and the finding that outranks the verdict

The fix is **not on this branch** (hole 1 only); it is `c63d7af` on track since 2026-10-16 19:20,
inherited through the base. And **the deployed binary — `~/Documents/GitHub/seatloom/target/debug/
seatloom-mcp-emit`, built Oct 16 12:58 — predates it**, which I proved by driving both binaries
through the same scratch scenario:

- **Deployed**: a PASS verdict about another dispatch → **card created**; a FAIL verdict about this
  dispatch → **card created** (the 10-16 incident's shape, reproducible today); a nonexistent verdict
  → refused (existence is the only dimension it checks).
- **Branch (fixed)**: all three refused with distinct messages naming the verdict, its subject/value
  and what to do; a PASS verdict naming the dispatch → **card created** (legitimate requests not
  wrongly refused); zero cards from the three bad cases.
- The two real cards (`acc-disp-4WRVFBHEc0ib_TP0WXFI_`, `acc-disp-_4-4LQz9T_ienR34isCtD`) pass the
  new logic — the gate's read resolves `ev-AbuUI51WIx1Ks-Eb_4FZp|pass` for both. The three named DB
  tests exist; the 24-test ignored suite passes.

## Needs reporting (no action taken — outside my boundary)

- **🔴 Rebuild and restart the MCP server.** Until the binary is rebuilt from track (or later), the
  fleet's card-creation path is unguarded: a card citing a fail verdict, or a foreign one, can be
  created today and nothing says no until the operator's click. This is the act that closes hole 2;
  the code alone does not.
- Same for `list_blocked_dispatches`: not in the deployed binary either, so the fleet-wide answer
  still costs the SQL it replaced until the restart.
- The dispatched list of "the three blocked" was itself stale (one was released by Nimbus's
  re-emission; a fourth joined) — the view shows current truth, which is its purpose.
