# Acceptance: Nimbus Real Collaboration PostgreSQL Baseline

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-29-nimbus-real-collaboration-postgres-baseline-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v2 |
| target | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md` |
| verdict | PASS |
| tags | acceptance, nimbus, postgres, persistence, infrastructure, seed-data, verification |

## Verdict

**PASS**

Lyra accepts the real-collaboration PostgreSQL baseline as scope-complete and verified.

The earlier `HOLD` was environment-only. That hold is now resolved by Flux's commit-pinned remote re-verification on the sponsor workspace:

- exact verified branch: `track/infra-foundation`
- exact verified commit: `a658086b54323259fda2ad2a958d097701f1fbbd`
- PostgreSQL execution path: `A (Docker)`
- PG16 blocker status: **closed**
- live DB integration result: **9 / 9 PASS**

Nimbus's bounded PG16 fix (`de3aefc33f1905aea27b0078be6bb0e2324604fd`) preserved semantics and removed the only concrete blocker (`ARRAY[]` -> `ARRAY[]::text[]` in the seed). No new blocker was introduced.

## Scope Reviewed

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-commit-pinned-infra-baseline-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-commit-pinned-infra-baseline-delivery-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-delivery-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-v2.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-reverification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-reverification-delivery-v1.md`
- `docs/coordination/tasks/lyra/LYRA-2026-04-29-infra-method-review-gate-v1.md`
- `infra/postgres/docker-compose.yml`
- `infra/postgres/schema/001_seatloom_core.sql`
- `infra/postgres/seed/001_real_collaboration_baseline.sql`
- `scripts/verify-postgres-baseline.sh`
- `.seatloom/bootstrap/source-map.yaml`
- `crates/seatloom-core/src/db/mod.rs`
- `crates/seatloom-core/src/db/connection.rs`
- `crates/seatloom-core/src/db/models.rs`
- `crates/seatloom-core/src/db/repositories.rs`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `docs/infra/ssh-tunnel-workspace.md`

## Coverage Matrix

| Requirement slice | Evidence paths | Result | Notes |
|---|---|---|---|
| Structured collaboration truth is persisted in PostgreSQL, not file-only state | schema, repositories, Nimbus baseline delivery | PASS | Authority is now a real DB-backed structured layer. |
| Seed data reflects real collaboration rather than demo-only content | seed SQL, source map, Flux remote re-verification | PASS | Remote seed committed successfully with 41 inserted rows. |
| Artifact metadata preserves `template + subtype` dual-key classification | schema `artifacts`, source map, DB tests | PASS | Typed artifact taxonomy is present in seeded truth. |
| Commit-pinned verification can reproduce the exact infra baseline remotely | Flux remote verification + re-verification deliveries | PASS | Exact commit `a658086...` was fetched, checked out, and verified on sponsor workspace. |
| Repository-managed PostgreSQL bootstrap path works end to end | `scripts/verify-postgres-baseline.sh`, Flux remote re-verification | PASS | Docker-backed bootstrap, schema, seed, and live tests all passed. |
| Ignored DB integration tests succeed on a live PostgreSQL instance | Flux re-verification delivery | PASS | `9 / 9` live DB tests passed. |
| Five real seats, delegation, artifacts T1-T7, and contract artifacts exist in the seeded truth | Flux SQL spot-checks and DB tests | PASS | Coverage confirmed with live data, not file-only inspection. |
| Packet remains infrastructure-only | Nimbus deliveries, Flux verification | PASS | No UI or product-scope widening occurred. |

## Findings

### Resolved blocker

1. **PG16 seed compatibility issue is closed**
   - Prior blocker: PostgreSQL 16 rejected an untyped empty array in `infra/postgres/seed/001_real_collaboration_baseline.sql`.
   - Fix: Nimbus cast the empty array to `text[]` in commit `de3aefc33f1905aea27b0078be6bb0e2324604fd`.
   - Verification: Flux re-ran the live Docker-backed PostgreSQL path on the sponsor workspace and confirmed `PASS` at exact commit `a658086b54323259fda2ad2a958d097701f1fbbd`.

### Non-blocking notes

1. The sponsor workspace now serves as the approved docker-capable verification seat for this baseline.
2. `docs/infra/ssh-tunnel-workspace.md` is now the durable operator note for connecting to that workspace over SSH tunnel for PostgreSQL or frontend inspection.
3. Full workspace GUI dependency verification remains separate from this packet; this acceptance closes the PostgreSQL baseline only.

## Required Fixes for Nimbus

None.

## Go / No-Go Recommendation

- **Implementation quality:** **GO**
- **Close packet as final PASS now:** **GO**
- **Resume file-only persistence as authority:** **NO-GO**
- **Open business/backend expansion from this packet automatically:** **NO-GO**

## Evidence Paths

- Nimbus baseline packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
- Nimbus baseline delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`
- Nimbus PG16 fix packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-v1.md`
- Nimbus PG16 fix delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-delivery-v1.md`
- Initial Flux verification (environment path established): `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`
- Final Flux re-verification (PASS): `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-reverification-delivery-v1.md`
- SSH tunnel operator note: `docs/infra/ssh-tunnel-workspace.md`
- Verification script: `scripts/verify-postgres-baseline.sh`
- Schema: `infra/postgres/schema/001_seatloom_core.sql`
- Seed: `infra/postgres/seed/001_real_collaboration_baseline.sql`
- Provenance: `.seatloom/bootstrap/source-map.yaml`
