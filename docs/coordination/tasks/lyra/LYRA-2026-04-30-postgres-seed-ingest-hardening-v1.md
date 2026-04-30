# Delivery: PostgreSQL Seed + Ingest Hardening

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-04-30-postgres-seed-ingest-hardening-v1 |
| status | delivered |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-operational-review-continuity-supplement-v1.md` |
| tags | postgres, infrastructure, seed, ingest, verification, hardening |
| owner | Lyra |

## Scope

This was a bounded infrastructure-only repair to close the real blocker discovered by Flux on the docker-capable remote verification seat.

Out of scope:
- UI or frontend code
- Tauri or GTK dependency work
- API work
- product behavior expansion
- non-infra coordination cleanup

## Trigger

Flux's commit-pinned remote reverification on commit `ee0159fa440b910de38b29824574760722a18316` proved that the narrowed Rust surface was healthy, but the PostgreSQL baseline bootstrap was not.

Exact failure class:
1. `infra/postgres/seed/002_document_seed.sql` referenced `ar-task-db-baseline`, but the artifact baseline seed did not create that artifact row.
2. `scripts/verify-postgres-baseline.sh` used `psql -f` without `ON_ERROR_STOP=1`, so SQL failures rolled back and only surfaced later during DB integration tests.

## Delivered

### 1. Baseline artifact seed repaired

Added the missing artifact row `ar-task-db-baseline` to `infra/postgres/seed/001_real_collaboration_baseline.sql` so the typed document seed no longer violates the `documents.artifact_id` foreign key.

### 2. Document seed metadata corrected

Updated `infra/postgres/seed/002_document_seed.sql` to reflect the corrected artifact count (`15 artifacts T1-T7`) for the Nimbus real-collaboration PostgreSQL baseline document.

### 3. Operational continuity seed aligned

Updated `infra/postgres/seed/003_operational_review_and_continuity_seed.sql` so the Nimbus checkpoint artifact evidence includes `ar-task-db-baseline` alongside the storage and hardening packets.

### 4. Provenance map aligned

Updated `.seatloom/bootstrap/source-map.yaml` so the bootstrap provenance includes the same `ar-task-db-baseline` artifact identity and path.

### 5. Verification scripts hardened

Updated `scripts/verify-postgres-baseline.sh` to:
- run a static cross-seed consistency test before Docker bootstrap,
- force `psql` into `ON_ERROR_STOP=1` for schema and seed application,
- ingest full document bodies before the included-ignored DB integration path.

Updated `scripts/ingest-documents.sh` to:
- force `psql` into `ON_ERROR_STOP=1`,
- assert that each `documents.id` update touches exactly one row,
- fail immediately if a seeded document row is missing.

### 6. Static consistency tests added

Created `crates/seatloom-core/tests/postgres_seed_consistency.rs` with three infra-only guardrails:
- every document seed artifact reference must exist in the artifact baseline seed,
- ingest script document IDs must exactly match seeded document IDs,
- every ingest target file path must exist in the repo.

## Files Changed

Updated:
- `.seatloom/bootstrap/source-map.yaml`
- `infra/postgres/seed/001_real_collaboration_baseline.sql`
- `infra/postgres/seed/002_document_seed.sql`
- `infra/postgres/seed/003_operational_review_and_continuity_seed.sql`
- `scripts/verify-postgres-baseline.sh`
- `scripts/ingest-documents.sh`

Created:
- `crates/seatloom-core/tests/postgres_seed_consistency.rs`

## Validation

Passed locally on the bounded infra surface:
- `$HOME/.cargo/bin/cargo test -p seatloom-core postgres_seed_consistency -- --nocapture`
- `$HOME/.cargo/bin/cargo test -p seatloom-core`
- `$HOME/.cargo/bin/cargo check -p seatloom-core`
- `$HOME/.cargo/bin/cargo fmt --all --check`
- `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings`
- `bash -n scripts/verify-postgres-baseline.sh scripts/ingest-documents.sh`

Environment note:
- Docker is still unavailable on this seat, so the end-to-end PostgreSQL bootstrap path remains pending remote commit-pinned re-verification.

## Commit Identity

Infrastructure repair commit pushed for verification:
- branch: `track/infra-foundation`
- target commit: `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b`
- compare base: `ee0159fa440b910de38b29824574760722a18316`

## Next Step

Issue one more commit-pinned Flux packet against commit `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b` on the sponsor-provided docker-capable workspace.
The packet should prove:
1. the exact commit was checked out,
2. the narrowed `seatloom-core` Rust surface still passes,
3. `bash scripts/verify-postgres-baseline.sh` now passes end to end,
4. the seeded review/continuity families and document authority data are materially present after bootstrap.
