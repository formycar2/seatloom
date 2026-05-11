# Acceptance: 008 — Multi-Project Isolation Schema Migration (project_id FK)

[Lyra -> Nimbus] 008 Layer A PASS, Lyra acceptance issued. Mr. Zhang Layer B over to Aegis. B2 onward unblocked.

| Field | Value |
|---|---|
| template | T3 |
| subtype | acceptance |
| id | LYRA-2026-05-09-nimbus-schema-008-project-isolation-acceptance-v1 |
| status | accepted |
| author | lyra |
| date | 2026-05-11 |
| version | v1 |
| to | nimbus, flux, aegis, mr_zhang |
| target_commit | `526d25a1db46b80659e2572fbf5baa8f38ed9543` |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-schema-008-project-isolation-v1.md` |
| delivery_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-schema-008-project-isolation-delivery-v1.md` (commit `f681251`) |
| flux_verify_ref | `docs/coordination/tasks/flux/FLUX-2026-05-09-schema-008-project-isolation-verify-delivery-v1.md` |
| layer_a_verdict | PASS (Flux: 15/15 static + 4/4 R-rules + migration applied + 4/4 NULL=0 + 4/4 GROUP BY=seatloom + idempotent re-run clean + tsc=0 + build green) |
| layer_b_verdict | PENDING — Mr. Zhang via Aegis (4-step Tauri runtime smoke per §3) |
| tags | lyra, 008, schema, multi-project, project-isolation, AD-013-v2-backend-enforcement, layer-a-pass, layer-b-pending |

---

## §1 — Verdict

**PROVISIONAL ACCEPTED — Layer A unconditional PASS.** Acceptance promotes to UNCONDITIONAL upon Mr. Zhang Layer B runtime confirmation (Inbox/WorkItems/Sessions/Handoffs surfaces post-migration + Project mode filtering + Global mode aggregation).

Layer A evidence chain solid:
- Nimbus delivery `526d25a` self-pinned 12 files / +308/-22, migration applied + idempotent re-run proven, backfill 6/5/3/35
- Flux Layer A verify independently confirmed 15/15 static invariants + 4/4 R-rules with line-anchored evidence
- Migration psql output verbatim in Flux delivery §2: `BEGIN`, 4× ADD COLUMN IF NOT EXISTS, 4× UPDATE (6/5/3/35), 8× ALTER TABLE for DROP DEFAULT + SET NOT NULL, 4× CREATE INDEX, `COMMIT`
- Backfill verification: all 4 NULL-counts = 0, all 4 GROUP BY queries show only `'seatloom'`
- Idempotent re-run: NOTICE skips for column existence + index existence, zero errors, COMMIT clean
- 60/60 cargo tests, tsc zero errors, pnpm build success

Three §10 pre-impl corrections applied during implementation, all valid:
- 10.1 `event_object_refs` columns are `(event_id, ref_type, ref_id)` — Lyra's packet §2.1 had wrong column names; Nimbus used correct schema in delivery
- 10.2 only `ref_type='seat'` emitted today; canonical_events lineage joins via workitem/session/handoff `ref_type` are forward-compat for when more ref types appear
- 10.3 `ADD COLUMN IF NOT EXISTS` supported since PG 9.6; we run PG 16; no `DO` block needed for idempotency

Architectural coverage: 4 new project-mode commands (`cmd_list_*_for_project`) + un-scoped commands preserved for AD-013 v2 Global mode = backend mirror of frontend AD-013 v2 invariants. US-P3-01 multi-project workstream gating now defendable at the API surface, not just React store. US-P0-12/13 mobile inbox no longer at risk of cross-project leakage.

## §2 — Three-layer status

| Layer | Owner | Status | Evidence |
|---|---|---|---|
| Delivery | Nimbus | ✓ delivered | `526d25a` (12 files, +308/-22) + delivery doc `f681251` |
| Pre-flight (Aegis scope/drift) | Aegis | ✓ ratified | Consolidation `ebc0e68` decision-2-corrected (008 parallel-not-blocking) + 006→007→008 numbering binding |
| Layer A commit-pinned verify | Flux | ✓ PASS | 15/15 static + 4/4 R-rules; verify delivery `FLUX-2026-05-09-schema-008-project-isolation-verify-delivery-v1.md` with verbatim psql output |
| Layer B runtime | Mr. Zhang via Aegis | ⏳ PENDING | 4-step Tauri smoke per §3 below |
| Lyra final acceptance | Lyra | ⏳ provisional pending Layer B | This document promotes to UNCONDITIONAL on Layer B PASS |

## §3 — Mr. Zhang Layer B — 4-step Tauri runtime smoke

Mr. Zhang via Aegis runs against `pnpm tauri dev` at HEAD `526d25a` or descendant. Lighter than B1's 6-step smoke because 008 is a structural migration, not a write-path landing.

| Step | Action | Expected | Verifies |
|------|--------|----------|----------|
| 1 | `pnpm tauri dev` boot, navigate through Inbox / WorkItems / Sessions / Handoffs surfaces | All 4 surfaces render correctly (no DTO mismatch crashes from new `projectId` field; backend lists return non-empty for `seatloom`) | DTO additions wire-shape across Rust → Tauri → TS chain |
| 2 | Verify Project mode (AD-013 v2 currentContextMode='project', activeProjectId='seatloom') renders all 6 workitems / 5 sessions / 3 handoffs / 35 events | Counts match Flux's verified Backfill numbers | Project-mode `cmd_list_*_for_project` commands return correctly-scoped data |
| 3 | Switch to Global mode (currentContextMode='global', activeProjectId=null) — confirm un-scoped lists still render | Existing `cmd_list_*` commands preserved for Global mode aggregation | AD-013 v2 backend Global-mode preservation |
| 4 | (Optional) Insert a test second project via SQL: `INSERT INTO projects (id, name) VALUES ('test-proj', 'Test'); INSERT INTO project_role_bindings ...; INSERT INTO workitems ... (project_id='test-proj')`; switch to test-proj Project mode | Only test-proj-scoped data visible; no leakage from seatloom | Multi-project isolation end-to-end (the actual US-P0-03 / US-P3-01 promise) |

If steps 1-3 PASS → Lyra promotes to UNCONDITIONAL. Step 4 is optional verification of the multi-project capability; Aegis decides whether Mr. Zhang should bother given there's only one project today.

If any step FAILs → HOLD with named step + observation; Nimbus dispatches a 008-v2 fix.

## §4 — Carry-forwards (acknowledged, not blocking)

1. **Pre-existing clippy lints** (`db/repositories.rs` × 3 `doc_lazy_continuation`) — Flux confirmed via static inspection that 008's new doc comments use proper `///` prefixing, no fresh lints introduced. Same disposition as B1: standalone cleanup packet.
2. **Future create-project UI** — Nimbus §7 flag-up (b): inserting a workitem/session/handoff/canonical_event for a new project requires the `projects` row to exist first (FK enforcement). Out of 008 scope. Will be addressed when create-project UI lands.
3. **`event_object_refs` `ref_type` only-`seat`-today** — Nimbus §10.2: the canonical_events backfill currently only joins via `ref_type='seat'`-style links because that's all that exists. Forward-compat for additional ref_types.

## §5 — What this acceptance unlocks

- **B2 (`cmd_append_supervisor_message` → seat write path with canonical_events emission)** is now unblocked. New events can carry `project_id`. AD-013 v2 backend enforcement is defendable in `WHERE project_id = $1` clauses.
- **Watcher / file-system events (Nimbus arch §2)** can now derive `WorkItemCreated`/`HandoffDelivered`/`ReviewVerdictIssued` events with proper `project_id` scoping.
- **Mobile companion (US-P0-12/13)** can safely query Inbox without cross-project leakage.
- **US-P3-01 parallel module workstreams** become defendable — `WHERE project_id = $1` is the API contract, not just React store policy.

## §6 — Acknowledgements

- **Nimbus**: clean delivery with all three §10 pre-impl corrections applied (the `event_object_refs` schema correction was a real bug catch on my packet — appreciated). Idempotent migration with NOTICE skip + identical UPDATE counts on re-run is the correct PostgreSQL idempotency pattern. 4 new project-mode commands + un-scoped command preservation respects AD-013 v2 invariants without needing UI changes.
- **Flux**: 15-row static matrix + line-anchored evidence + verbatim psql output captures all the migration nuance (NOTICE skip behavior, UPDATE counts, GROUP BY scoping). Idempotent re-run independently reproduced. Static-verified clippy non-regression confirmation continues to save sibling-packet roundtrips.
- **Aegis**: consolidation adjudication decision-2 correction (008 parallel-not-blocking) was right; my cross-point (c) hold position would have delayed B1 unnecessarily. Numbering binding 006→007→008 is the right ordering for predictable migration sequencing.

---

*Lyra acceptance · 2026-05-11 · PROVISIONAL pending Mr. Zhang Layer B · same three-layer protocol as A4-β + B1 · UNCONDITIONAL promotion follows Layer B PASS · B2 onward unblocked*
