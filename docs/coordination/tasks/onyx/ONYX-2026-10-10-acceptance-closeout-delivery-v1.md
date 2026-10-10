# Delivery: acceptance-cards closeout — 017 landed; three cards BLOCKED by the deployed binary (escalated)

[Onyx -> Lyra/Aegis] `disp-acclose-onyx`: migration **017 landed and scratch-verified**
(`packet/onyx/acceptance-cancel-schema` @ `cddc6ce`, Flux verification self-requested
`disp-ma_3Y78wao3WVcKL1-Uk5`). The **three acceptance cards (015/016/017) could NOT be
created**: the fleet's deployed MCP binary predates the structured-acceptance-plan
enforcement, so `request_acceptance` rejects the `acceptance_plan` argument — no seat can
create a plan-carrying card right now. Escalated to Aegis
(`ev-haW7F7e-rRARRwehXYUhX`); per "no plan -> no card" the cards were **not** created
plan-less. This dispatch stays **in_flight** (not completed) — the cards are identity-guarded
to this seat and will be created the moment the binary accepts plans.

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | implementation-delivery (PARTIAL — externally blocked) |
| id | ONYX-2026-10-10-acceptance-closeout-delivery-v1 |
| status | delivered-with-blocker |
| author | onyx |
| date | 2026-10-10 |
| version | v1 |
| responds to | governed dispatch `disp-acclose-onyx` (acceptance-cards closeout, priority 40) |
| packet | `docs/coordination/tasks/onyx/ONYX-2026-10-10-acceptance-closeout-v1.md` (found via commit `3a269e2`; not on my base branch — worked from the payload + that packet verbatim) |
| governing docs | `LYRA-2026-10-10-acceptance-cards-closeout.md`; `NIMBUS-2026-10-10-d2-acceptance-base-and-cancel-v1.md`; the three Lyra plan docs (`2026-10-10-lyra-onyx-015/016/017-*-acceptance-plan-v1.md`) |
| base branch | `track/infra-foundation` @ `b6751a1` |
| delivery branch | `packet/onyx/acceptance-cancel-schema` @ `cddc6ce` (017); cards target `packet/onyx/d5-knowledge-schema` @ `4520758` (015) and `packet/onyx/acceptance-plan-schema` @ `a6f2ac8` (016) — both already Flux-PASS and containing base `b6751a1` |
| files changed | `infra/postgres/schema/017_acceptance_cancel.sql` (new), `scripts/apply-db.sh` (+1 POST_SEED entry), this doc |
| rust / tsx touched | none (verified: `git status --short` shows only the two files above) |
| live db touched | none — live READ only (state census); all writes on scratch DBs, dropped after |

---

## 1. What shipped — migration 017 (the surfaced gap, closed)

`infra/postgres/schema/017_acceptance_cancel.sql` (additive + idempotent, 013/015 guarded pattern):

1. **`acceptances_status_vocab` 3 → 4** (+`cancelled`) — guarded DROP+ADD, guard checks `'cancelled'`. `cancelled` = the **requesting seat** withdrawing its OWN pending request (terminal, immutable; distinct actor from the operator's accepted/rejected); sits outside `uq_acceptances_one_pending_per_dispatch`, so a dispatch may be re-requested after a cancel.
2. **`canonical_events_event_type_vocab` → the COMPLETE 26-value list** (+`AcceptanceCancelled`) — guarded DROP+ADD, guard checks `'AcceptanceCancelled'`. The 26 = 22 (010/012/013/014) + 4 memory (015) + AcceptanceCancelled. **Cross-checked verbatim against Nimbus's pinned Rust `EventType::ALL: [EventType; 26]` on `packet/nimbus/d5-mcp-recall` (`crates/seatloom-core/src/db/event_domain.rs`) — identical set AND order**; his closeout (`disp-acclose-nimbus`) was still in_flight at delivery time with the branch tip unchanged at `db7e1f3`. Full lists (not deltas) are what let 017 apply from either the track state (3/21) or the live state (3/25).
3. **COMMENT refresh** on `acceptances.status` (fourth value + the actor distinction) and `acceptances.feedback` (the cancel path records its mandatory reason there, same column as reject — the event type distinguishes the two).
4. **`scripts/apply-db.sh`** POST_SEED registration with a rule-based reason (widened CHECKs exist only post-seed; seed writes no acceptances rows; canonical order 014 → … → 017). `bash -n` clean; completeness guard passes.

Red lines held: no acceptances column changes, no data rows, no renumbering, no `crates/**`.

## 2. Verification — scratch only (live read-only)

Environment: `postgres:16-alpine` runner container sharing the `seatloom-postgres` network; two scratch DBs (`seatloom_scratch_017` = fresh track-state build; `seatloom_scratch_017_live` = 001–014 + 015 + 016 files, i.e. the live 25/12 state, **without** 017); both dropped after.

1. **Full build from track state** — `apply-db.sh` exit 0, 017 applied after 014, zero errors.
2. **Vocab final state** — `acceptances_status_vocab` = `pending|accepted|rejected|cancelled` (4); `canonical_events_event_type_vocab` = **26 values**, the full list printed and matching Nimbus's Rust enum verbatim.
3. **Before/after gap proof (the Aegis blocker, reproduced and closed)** — on the live-state DB *before* 017:
   `INSERT ... status='cancelled'` → `ERROR: new row for relation "acceptances" violates check constraint "acceptances_status_vocab"`;
   `INSERT ... 'AcceptanceCancelled'` → `ERROR: ... violates check constraint "canonical_events_event_type_vocab"`.
   After applying 017 to the same DB: **both inserts succeed** (`INSERT 0 1` each). This is exactly the DB block on Aegis's void of the old empty card — now removable once 017 is live-applied.
4. **Applies from the live state (25/12) too** — 017 onto the 015+016 DB: exit 0, vocab → 4/26.
5. **Idempotency** — second apply of 017: track-state DB `pg_class=611 pg_constraint=227 pg_attribute=4084` unchanged; live-state DB `620/237/4140` unchanged. Zero catalog writes from both states.
6. **Positive inserts** (track-state DB): a `cancelled` acceptance row (decided_by/feedback set, cancel semantics) and an `AcceptanceCancelled` canonical event both land.

## 3. BLOCKER — the three cards cannot be created (escalated, not worked around)

**What happened:** with 017 committed, `request_acceptance` was called for card 1 (015) with Lyra's §2 plan JSON verbatim. The **server's own schema validation rejected the argument**:

```
Validation failed for tool "mcp__seatloom__request_acceptance":
  - acceptance_plan: schema is false
  - root: must not have additional properties
```

**Root cause (precise):** this fleet's seats run `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit`, built Oct-10 19:34 from the main repo @ `b6751a1` (= track tip) — i.e. **before** Nimbus's structured-plan enforcement. That code exists only on `packet/nimbus/acceptance-lineage-readmodel` @ `f90665f` and `packet/nimbus/d5-mcp-recall` @ `db7e1f3` (`RequestAcceptanceArgs.acceptance_plan: serde_json::Value`, required; missing/empty/malformed = typed rejection). Neither branch is on main/track, so the deployed binary cannot carry plans.

**Blast radius: fleet-wide.** All six Lyra-authored plans are uncreatable — Onyx 015/016/017, Nimbus lineage + recall, Mira UI. Nimbus's and Mira's closeout dispatches hit the same wall.

**What I did NOT do (deliberately):** did not create plan-less cards (that is the exact defect this closeout exists to fix — "no plan -> no card"); did not hand-insert `acceptances` rows via psql (write-path discipline: cards are created by the tool in one transaction with its guards — base existence, ownership, no-pending, verdict-ref existence); did not repoint my seat's MCP config at a worktree binary or rebuild anything (unsupervised config/ops changes are outside my lane — AGENTS.md).

**Escalated:** `SupervisorMessage` `ev-haW7F7e-rRARRwehXYUhX` (actor seat-onyx-001, subject project:seatloom, context dispatch:disp-acclose-onyx) asking Aegis for: (a) rebuild+redeploy the MCP binary with the plan-enforcing code; (b) live-apply 015/016/017 (Aegis act); (c) re-nudge seat-onyx-001 once the binary accepts `acceptance_plan` — `disp-acclose-onyx` stays in_flight and the cards are identity-guarded to this seat.

**Live-state correction (for the record):** live `seatloom` is at the **014 state** — `event_type` 21 / `ref_type` 11 / status 3 values / **no `acceptances.acceptance_plan` column**. 015/016 were never applied live (the closeout packet's "live = 25" was stale; consistent with both deliveries' "live db touched: none"). The void of the old empty card `acc-disp-me6Nm8e9D6ZRe4fKyVokY` remains DB-blocked until 017 is live-applied.

## 4. Card parameters staged (executable the moment the binary accepts plans)

| card | dispatch_id | branch | base | flux_verdict_ref | plan source |
|---|---|---|---|---|---|
| 1 — 015 | `disp-d5-lane-a-onyx` | `packet/onyx/d5-knowledge-schema` | `track/infra-foundation` | `ev-zCWuVawAa1nPL5HLgqu45` (PASS, pin 4520758) | `2026-10-10-lyra-onyx-015-knowledge-layer-acceptance-plan-v1.md` §2 |
| 2 — 016 | `disp-accplan-lane-c-onyx` | `packet/onyx/acceptance-plan-schema` | `track/infra-foundation` | `ev-jWG35mnOsceuDoXmXqA7c` (PASS, pin a6f2ac8) | `2026-10-10-lyra-onyx-016-plan-carrier-acceptance-plan-v1.md` §2 |
| 3 — 017 | `disp-acclose-onyx` | `packet/onyx/acceptance-cancel-schema` | `track/infra-foundation` | `<Flux verdict for cddc6ce — self-requested `disp-ma_3Y78wao3WVcKL1-Uk5`> | `2026-10-10-lyra-onyx-017-cancel-widening-acceptance-plan-v1.md` §2 |

how_to_verify strings per the plan docs §3 (015/016) and §3 (017), carried verbatim. Containment note: both 015/016 branches contain base `b6751a1` (Flux-verified) — if Nimbus's merge-containment defense has landed, the requests pass its check; that is the defense's first positive proof.

## 5. Completion-flow status

1. ✅ 017 committed to `packet/onyx/acceptance-cancel-schema` (`cddc6ce`, commit carries the dispatch id).
2. ✅ Flux verification self-requested for 017 (commit-pinned branch/SHA/base) — `disp-ma_3Y78wao3WVcKL1-Uk5` (ready). 015/016 cite their existing PASS verdicts (event ids above).
3. ⛔ Three cards — **blocked** by the deployed binary (§3); parameters staged (§4); escalation emitted.
4. ✅ This delivery doc.
5. ⏸ `complete_dispatch` **withheld**: the done-definition (§3 落地验绿 + §4 三卡全部建出) is not met, and neither `done` (false completion) nor `failed` (017 did not fail; terminal failure would kill the dispatch and the identity-guarded cards with it) is honest. The dispatch stays in_flight; on the next nudge, `get_next_dispatch` surfaces it and the cards get created once the binary accepts plans (017's Flux verdict is expected by then).
6. ✅ The seat never merges its own work — unchanged.
