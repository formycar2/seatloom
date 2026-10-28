# Delivery — `disp-lGY3Glo1UE3TTi9eVC_0E` (Flux): the verdict re-issued, naming Nimbus's two implementation dispatches

- **Dispatch**: `disp-lGY3Glo1UE3TTi9eVC_0E`, issued by seat-aegis-001, kind `verification`
  (re-issue of my 2026-10-17 harness/Agent-Pi verdict).
- **Done**: `ev-AbuUI51WIx1Ks-Eb_4FZp` — `ReviewVerdictIssued`, `verdict: pass`, with
  `event_object_refs` rows `ref_type='dispatch'` for **`disp-4WRVFBHEc0ib_TP0WXFI_`** (enum Agent
  gains Pi) and **`disp-_4-4LQz9T_ienR34isCtD`** (enum Harness gains Pi), one each, plus the
  verification dispatch and the project as context and the verdict document as subject. Verified in
  the ledger: the gate's own read (`payload->>'dispatch_id'` OR an `event_object_refs` row with
  `ref_type='dispatch'`, AND `verdict='pass'`) now resolves **`ev-AbuUI51WIx1Ks-Eb_4FZp | pass`** for
  both implementation dispatches. Nimbus's card can be opened and will pass the merge gate.
- **The carried `does_not_assert`, verbatim** (as instructed, not one word changed): *"the rendered
  page (screen locked); anything about Mira's two UI fixes (not in this branch); any live write
  (none by me — the operator's scripts made the data changes, and fix-ql-role-domain.sh remains his
  to run)."* With one clearly-marked post-script, because a stale implication must not pass as
  current: `fix-ql-role-domain.sh` **has since been run** — measured in my integration-round verdict,
  `bindings[ql_delivery]` now reads 1 seat (`custom(worker)`, in domain).
- **Boundaries honoured**: no live writes, no gate events, no merges. Nothing was re-measured for the
  re-issue; the verified code is byte-identical (the `crates/` diff between my pin `ace1b2a` and the
  branch tip after Nimbus's track sync is empty).

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/disp-lGY3Glo1UE3TTi9eVC_0E-delivery.md` | this note |
| (ledger) `ev-AbuUI51WIx1Ks-Eb_4FZp` | the re-issued verdict — the actual deliverable |

No source file was changed.

## The vulnerability Aegis asked me to write up — measured, and the premise does not hold

Aegis's brief: *"request_acceptance 不验结论的指向,那个检查在合并门里。所以系统会爽快建一张后来必然被
合并门拒掉的卡。"* **I checked, and this is not true of the current code.** The evidence:

1. `request_acceptance`'s **guard 2** (`crates/seatloom-mcp-emit/src/dispatch.rs`, on
   `track/infra-foundation` at lines ~1764–1809) reads `flux_verdict_facts(flux_verdict_ref,
   dispatch_id)` and refuses, **before any DB contact**, when the referent is absent, is not a
   `ReviewVerdictIssued`, does not say `pass` (`VerdictNotGreen`), or does not name the dispatch
   (`VerdictNotAboutThisDispatch`). It was added by `c63d7af` on 2026-10-16 19:20 and has been on
   track since the `f81e913` merge at 19:37 that evening — a day before Nimbus's analysis — and it
   was in `packet/nimbus/harness-pi` even at my verification pin (`ace1b2a`).
2. The **only** production `INSERT INTO acceptances` in the tree is `dispatch.rs:1955`, inside
   `request_acceptance`, downstream of that guard (the other inserts are test fixtures). There is no
   other card-creation path: no Tauri command creates cards (the app only lists/accepts/rejects),
   and no other MCP tool writes the table.
3. So a card whose verdict does not name its dispatch **cannot be created** — the request is refused
   with a message that names the verdict, its subject, and what is wrong. What Nimbus actually hit
   was the guards working *correctly* from the other side: his verdict (`ev-Nfy-64T_UWlAa_INtzgSF`)
   named only the verification dispatch, so `is_merge_criterion` was false for both of his — his
   own measurement, via the same `flux_verdict_facts` the guards call. Had he called
   `request_acceptance` anyway, guard 2 would have refused it at request time. His decision not to
   open the card was right, and the system agreed with him.

**What is real, and worth a slice (reported, not fixed — outside this dispatch's boundary):**

- **Nothing detects a dispatch closed without its declared announcement id.** This is the gap that
  actually bit today, twice more: `complete_dispatch` closes a dispatch whether or not its required
  announcement was emitted under the required id. Measured: `ev-first-screen-default-tab-landed` was
  never emitted (that dispatch is `done`), both integration-round announcements went out under
  server-minted ids, and `disp-integration-round-verify-v2` — correctly edged on the fixes — sits
  `blocked` on two ids that do not exist. The fix shape: a guard at `complete_dispatch` (the
  declared emission id must exist in `canonical_events` before the row can close), or a sweep
  comparing declared-vs-emitted. Recorded in memory on 2026-10-17.
- **`cancel_acceptance` is terminal** (the row is never deleted; the id stays occupied). Nimbus
  named this as why a bad card would be unrecoverable. It is what makes the request-time guard
  load-bearing: with no clean withdrawal, any regression of guard 2 would produce cards the
  operator cannot accept and the seat cannot retract.
- **The two guards are two renderings of one criterion.** Guard 2 (`dispatch.rs`) and
  `verdict_gate_message` (`acceptance_cmds.rs`) share the `flux_verdict_facts` read but render
  separately; `c63d7af` fixed them together and nothing enforces that they stay together. A future
  edit to one without the other reopens exactly the asymmetry Aegis described.

## State of the round this unblocks

With this verdict emitted, Nimbus's pre-computed card arguments (his delivery
`NIMBUS-2026-10-17-harness-pi-acceptance-card-blocked-v1.md` §4) are complete: `dispatch_id`,
`branch=packet/nimbus/harness-pi` (0 behind track), `base=track/infra-foundation`,
`flux_verdict_ref=ev-AbuUI51WIx1Ks-Eb_4FZp`. The operator's click and the card opening are his and
Nimbus's acts, not mine. Separately measured today: Mira's UI fixes (including the provenance-badge
fix) are now **on track** via merge `5c2b96f`; the harness fix is **not** — and `af0275a` re-ruled
the data toward `Stepcode` with live still on `Pi` ×8, so `list_seats` still fails in the product
until one of the two exits is taken.
