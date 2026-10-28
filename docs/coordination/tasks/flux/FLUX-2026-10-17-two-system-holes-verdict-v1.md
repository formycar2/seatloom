# Verdict — the two system-hole fixes: blocked-dispatch visibility, and request_acceptance checking the verdict's subject

- **Verification dispatch**: `disp-ySsxkvHxKurEgKpt7Q_Hm` (issued by seat-aegis-001), covering
  - `disp-ibABeNk-nkEDAkI6Qj2sG` — blocked-dispatch visibility (`list_blocked_dispatches`)
  - `disp-b5VbRor1fHhP3nN4YZI_l` — `request_acceptance` verifies the verdict's subject before creating a card
- **Branch under test**: `packet/nimbus/blocked-dispatch-visibility` @ `fcbd612` (hole 1 only — hole 2's
  code is **not** on this branch; see below).
- **Verdict event**: `ev-pnBnPX1pdaRysj8UYW077` — the pre-declared id, emitted exactly as required,
  with one `ref_type='dispatch'` context ref per covered dispatch. **Both dispatches are covered: I
  verified both.**
- **VERDICT: PASS for both fixes — with one finding that matters more than either: the deployed MCP
  binary is 6½ hours older than the fix, so the hole-2 hole is LIVE in the system the fleet is
  running right now.**

## A. Gates — run by me at `fcbd612`, all green

`cargo fmt --all -- --check` clean · `cargo clippy --workspace --all-targets -- -D warnings` clean ·
`cargo test --workspace` **271/0** · `tsc --noEmit` clean · vitest **194/194** · Nimbus's
`blocked_dispatch_visibility_proof.sh` **ALL POINTS PASS** (7) · the ignored DB suite **24/0** (the
three hole-2 tests among them).

## Hole 1 — `list_blocked_dispatches`. Nimbus's "half real" claim is TRUE, and I checked it myself

The dispatch told me not to trust his summary, so I measured the premise against live before reading
his code:

- The payload keys Aegis measured (`payload->>'blocked_reason'`, `'dep_kind'`, `'dep_value'`) **are**
  empty for the blocked rows. But the structured answer exists one table over:
  `dispatch_dependencies` carries `event:ev-blocked-on-whom-dto-landed`,
  `event:ev-first-screen-default-tab-landed`, and **two** edges for `disp-session-key-ui`
  (`ev-session-key-readmodel-live` + `ev-history-sessions-read-live`). Every blocked row has at least
  one edge. **So "the structured answer already existed" is true** — Aegis measured the payload, not
  the table — and the missing half was the fleet-wide view, not the fields.
- The view runs and answers, measured by me against live (probe + proof):
  `dispatch=… stored=blocked` + one `waiting_on=<event id> kind=event exists=false` line per blocking
  edge, for every blocked dispatch in one call. Current live: **3 blocked dispatches, 4 blocking
  edges** (of 5 total — `blocking_only=true/false` are distinguished, and one dispatch has both a
  satisfied and an unsatisfied edge, which the two modes keep apart rather than collapsing).
- **"Blocked" is computed from the edges, so the hole cannot reopen.** `enqueue_dispatch` derives the
  initial status (`ready` iff every dep is satisfied), and a row with no edges is stored `ready` —
  "an empty block cannot be stored" is a test. The implementation dispatch asked for "refuse or
  require"; Nimbus chose a third shape that is stronger than either: the reason-less state is
  **unrepresentable** through the sanctioned path. There is no refusal message naming what is missing
  because there is no failure to refuse — I record the deviation from the letter, and judge the
  intent met.
- **Nothing to backfill** — measured: every existing blocked row already carries its edges, so the
  script-for-the-operator the dispatch asked for is unnecessary. (The dispatch's list of three was
  itself slightly stale: `disp-blocked-on-whom-ui` is no longer blocked — Nimbus's re-emission
  released it — and a fourth, `disp-integration-round-verify-v2`, has joined, on the two announcement
  ids that were never emitted. The view shows the current truth, which is the point of it.)
- **What the view does NOT precompute: the waiting duration.** It returns `issued_at` (the raw fact);
  "压了多久" is one subtraction away but not stated. Minor, and named rather than passed over.

## Hole 2 — the fix is verified; the code is NOT on this branch; and the hole is live in the deployed system

**Where the code is (per the dispatch's instruction D, stated plainly):** hole 2's fix is **not** on
`packet/nimbus/blocked-dispatch-visibility` — that branch carries hole 1 only. The fix is `c63d7af`
("the merge criterion reads the verdict's VALUE and its SUBJECT"), which added guard 2 to
`request_acceptance` and landed on `track/infra-foundation` on 2026-10-16 19:20 (merged 19:37). This
branch inherits it through its base. I am covering `disp-b5VbRor1fHhP3nN4YZI_l` because I verified the
fix's **behavior**, not because the code is on the branch under test.

**🔴 The deployed binary predates the fix.** The MCP server the fleet runs is
`~/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit`, **built Oct 16 12:58** — six and a half
hours before `c63d7af` landed. `strings` confirms it carries the OLD guard (existence-only:
"names no existing event or artifact"; zero occurrences of "is not about"). **So the hole is live
right now**, and I proved it behaviorally rather than by reading: on a scratch database
(`seatloom_hole2`, built in the canonical order), driving the **deployed** binary through a real
scenario (dispatch enqueued → pulled → completed → verdict emitted → card requested):

| Case | Deployed binary (Oct 16 12:58) | Branch binary (with the fix) |
|---|---|---|
| A — PASS verdict about **another** dispatch | **card CREATED** (`acc-disp-hole2-target … pending`) | refused: *the verdict "ev-hole2-pass-other" is not about "disp-hole2-target" — it is about "disp-hole2-other". Another unit of work's verdict cannot stand in for this one's; ask Flux to verify THIS dispatch.* |
| B — **FAIL** verdict about this dispatch | **card CREATED** (`… flux_verdict_ref=ev-hole2-fail-target … pending`) | refused: *the verdict "ev-hole2-fail-target" says "fail", not 'pass' — a red verdict is not a merge criterion…* |
| C — nonexistent verdict | refused (existence — the only dimension it checks) | refused: *names no existing event or artifact — the Flux green verdict is the merge criterion's first half and must exist* |
| D — PASS verdict naming this dispatch | (creates — unguarded) | **card CREATED** — legitimate requests are not wrongly refused |

Zero cards were created by the branch binary in cases A–C. Case B on the deployed binary is the
2026-10-16 incident's exact shape, reproducible today: a card citing a **fail** verdict, created
without a word of complaint, waiting for an operator click that the merge gate will refuse.

**The two real cards pass the new logic** (the dispatch's requirement 3): `acc-disp-4WRVFBHEc0ib_TP0WXFI_`
and `acc-disp-_4-4LQz9T_ienR34isCtD` both cite `ev-AbuUI51WIx1Ks-Eb_4FZp`, and the gate's own read
(`payload->>'dispatch_id'` OR an `event_object_refs` dispatch row, AND `verdict='pass'`) resolves
**pass for both** — measured, not asserted. The three named DB tests exist and the 24-test ignored
suite passes.

## What this verdict does not assert

- It does not assert the deployed system is fixed: it is not. The fix is in the repository and works;
  the running binary is stale. **The closing act is a rebuild + restart of the MCP server — an
  operational act, the operator's or Aegis's, not mine** (I restarted nothing this round).
- It does not assert anything about the running app's rendering (the screen is locked from my seat —
  recorded on 2026-10-17).
- No live writes, no gate events, no merges, no cards opened — per the dispatch's boundary C, and the
  whole hole-2 experiment ran on a scratch database. The one git side effect outside my worktree: a
  temporary local branch `hole2-sync-branch` in the main checkout (a ref at track's tip, needed
  because the containment check requires a branch that contains the base), deleted after the run.
- The `list_blocked_dispatches` view is likewise not in the deployed binary; until it is, the
  fleet-wide answer still costs the SQL it replaced.

## Bottom line

**PASS for both fixes.** Hole 1: the structured answer did already exist (verified against live, not
from his summary), the view answers "what is stuck and on what" in one call, and the reason-less
blocked state is now unrepresentable. Hole 2: the fix refuses all three bad shapes with messages that
name what is missing and still creates legitimate cards (verified by driving both binaries on a
scratch DB, plus the two real cards through the gate's read). **And the finding that outranks both:
the fleet is running a binary from before the fix, so today a card can still be created on a fail or
a foreign verdict. Rebuild and restart the MCP server — that is the act that closes hole 2, and
nothing else does.**
