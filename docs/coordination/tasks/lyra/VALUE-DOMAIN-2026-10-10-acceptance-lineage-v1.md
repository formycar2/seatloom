# Value-domain contract — acceptance card lineage + structured acceptance plan

**Owner:** Lyra (PO). **Status:** ACTIVE — signed, lanes dispatched. **Date:** 2026-10-10.
**Applies to:** `ONYX-2026-10-10-acceptance-plan-schema-v1.md` (Lane C),
`NIMBUS-2026-10-10-acceptance-lineage-readmodel-v1.md` (Lane B),
`MIRA-2026-10-10-acceptance-lineage-ui-v1.md` (Lane A), and Lyra's own Lane D
(acceptance-plan authorship). **Trigger:** blocking operator feedback on the
pending-acceptance card, 2026-10-10.

This is the single contract the three lane packets carry **verbatim**. Neither seat
re-derives it; a deviation is reported, not silently reconciled. Extends
`VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` + D2 addendum and
`VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md`; re-derives nothing.

```
ACCEPTANCE CARD LINEAGE + STRUCTURED ACCEPTANCE PLAN — value-domain contract, Lyra sign-off 2026-10-10

BACKGROUND (blocking feedback from the operator, 2026-10-10): the pending-acceptance
card shows only branch + one-line how_to_verify. The operator cannot judge value from
it — no cause-and-effect, no lineage, no acceptance plan. Two DISTINCT gaps:
  A. lineage = pure display gap — all data is already in the ledger; nothing new to store.
  B. acceptance plan = real data gap — needs a structured carrier, required at card creation.

1. ACCEPTANCE PLAN (NEW structured payload on acceptances, migration 016) — shape v1:
   acceptance_plan JSONB, an object with EXACTLY these keys (unknown key = typed rejection):
     plan_version  INTEGER = 1
     scenario      STRING  non-empty   -- what the user is trying to achieve (product language)
     steps         ARRAY   non-empty   -- the product-operation script; each item:
       { action: STRING non-empty,     -- where to click / what to fill / what to say
         input:  STRING optional }     -- the input for that step
     expected      ARRAY   non-empty   -- each item:
       { output: STRING non-empty,     -- the observable output
         expect: STRING non-empty }    -- the pass criterion (seeing X = pass)
   preconditions  ARRAY of STRING, optional (empty allowed) -- required prior state
   Empty object '{}' is NOT a plan. One plan per acceptance, set at request time,
   immutable after (same immutability rule as how_to_verify).

2. ENFORCEMENT (two layers, both required):
   - TOOL (request_acceptance, MCP): a missing/malformed plan is a TYPED rejection
     before any DB contact (house discipline: typed errors, never a DB-error crutch).
     No plan -> no card.
   - DB (016): CHECK (acceptance_plan <> '{}'::jsonb) added NOT VALID (additive: the one
     legacy pending row is exempt until Aegis backfills its Lyra-authored plan and
     VALIDATEs — that backfill is a live-DB act, Aegis lane).

3. LINEAGE (pure read assembly — NO new storage; the ledger already has everything):
   From an acceptance id, assemble:
     acceptance  (id/status/branch/base/how_to_verify/acceptance_plan/flux_verdict_ref/
                  created_at/decided_*/merge_commit/feedback/rework_dispatch_id)
     dispatch    (id/title/issued_by/issued_at/payload — the ORIGINAL task statement)
     events      (the dispatch's canonical_events in occurred_at order: SeatDispatchIssued
                  -> SeatDispatchDelivered -> ArtifactCreated* -> SeatDispatchCompleted
                  -> AcceptanceRequested -> ReviewVerdictIssued; actor + occurred_at + payload)
     fluxVerdict (the ReviewVerdictIssued event named by flux_verdict_ref:
                  verdict/evidence/evidence_detail/target_branch/lane)
     products    (artifact + document refs found in the event chain: ids/titles/paths)
     dependencies(dispatch_dependencies of the dispatch: dep_kind/dep_value + exists?)
     rework      (rework_dispatch_id -> that dispatch's row, when rejected)
   Assembly lives in ONE module (seatloom-core db lineage), first instance = acceptance
   lineage. D5's recall (memory + topic ancestor chain) is the SAME SHAPE — object plus
   its context chain, progressively disclosed. One design, two assemblers; do not build
   a second ad-hoc query per feature.

4. CARD DISCLOSURE LEVELS (UI, same three-level idea as D5's L0/L1/L2 — do not invent
   a fourth pattern):
     L0 default (enough to judge): milestone/lane + task title; issuer + when;
          Flux verdict badge; acceptance-plan scenario + step count + first expected;
          accept/reject actions.
     L1 expanded: the dispatch's original payload (full task statement); the doing seat;
          products/delivery-doc links; the full event chain; the full plan.
     L2 ids/links: branch, commit SHAs, evidence doc paths, dispatch/acceptance ids.
   Card section order (the operator's own words): which milestone/lane/task -> who
   issued it and when -> the original task requirements (dispatch payload) -> who did
   the work -> where the products/delivery notes are -> Flux verdict and evidence ->
   the structured acceptance plan -> and only then accept/reject.

5. AUTHORSHIP: the acceptance plan is authored by the PO (Lyra), never by Mira/Nimbus/
   Flux. Every acceptance packet from now on carries the follow-it-to-verify product
   script; a one-line how_to_verify alone is no longer an acceptable acceptance packet.
   how_to_verify REMAINS the one-line criterion; the plan is the script that executes it.

6. RED LINES (binding): lineage adds NO new table and NO new event type (display gap);
   the plan adds ONE JSONB column + ONE NOT VALID CHECK (016, additive + idempotent,
   second apply = zero catalog writes); no whole-branch merges (branches are stale vs
   track; path-scoped only, via the D2 accept mechanism); event-driven, zero polling —
   cross-seat sequencing uses enqueue_dispatch dependency edges only.
```

---

## Wiring (cross-seat sequencing — data edges, not polling)

```
disp-accplan-lane-c-onyx   (Onyx,   READY)   migration 016
        |  on delivery emits event ev-016-acceptance-plan-landed
        v
disp-accplan-lane-b-nimbus (Nimbus, BLOCKED) dependency {event, ev-016-acceptance-plan-landed}
        |  on delivery emits event ev-acceptance-lineage-readmodel-landed
        v
disp-accplan-lane-a-mira   (Mira,   BLOCKED) dependency {event, ev-acceptance-lineage-readmodel-landed}
```

Each gate event lands in `canonical_events`; 013's `trg_dispatch_ready_on_event` flips
the dependent dispatch `blocked -> ready` and `pg_notify('dispatch_ready', seat)` — the
waker pokes the seat. Zero polling, zero DB queries by seats. Critical path is
deliberately serial: each lane consumes the previous lane's landed contract surface,
which is what keeps the D2-style "merge half green, code half blocked" cross-lane break
from recurring.

## Legacy pending card (handled, not ignored)

`acc-disp-me6Nm8e9D6ZRe4fKyVokY` (Mira D2 UI) predates the plan column. Lyra authors its
structured plan as the worked example (`docs/coordination/acceptance/2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1.md`);
Aegis backfills it into the row and runs `VALIDATE CONSTRAINT` once 016 has landed — a
live-DB act, Aegis lane, not guessed by any seat.
