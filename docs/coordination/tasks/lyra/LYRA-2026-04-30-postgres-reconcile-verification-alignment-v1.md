# Delivery: PostgreSQL Reconcile Verification Alignment

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-04-30-postgres-reconcile-verification-alignment-v1 |
| status | delivered |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v2.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-seed-ingest-hardening-v1.md`, `scripts/verify-postgres-baseline.sh`, `src-cli/src/main.rs`, `crates/seatloom-core/tests/db_baseline_integration.rs` |
| tags | postgres, infrastructure, reconcile, verification, commit-pinned, hardening |
| owner | Lyra |

## Scope

This was a bounded infrastructure-only correction to the PostgreSQL baseline verification path.

Out of scope:
- UI or frontend code
- schema redesign
- seed content expansion
- runtime API work
- non-infra coordination cleanup

## Trigger

Flux v3 remote verification got materially further on commit `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b`, but `bash scripts/verify-postgres-baseline.sh` still failed at the DB integration stage.

Exact failing tests:
1. `db_baseline::reconcile_run_recorded_after_run`
2. `db_baseline::document_versions_created_on_first_reconcile`

## Root Cause

The verification flow was still proving only the direct body-ingest helper before running the included-ignored DB integration tests.

That was insufficient because:
- `scripts/ingest-documents.sh` updates `documents.body_text`, `body_digest`, and `revision`,
- but it does **not** create `reconcile_runs`, `reconcile_items`, or `document_versions`,
- while the schema-003 tests explicitly require a completed reconcile run and version snapshots.

So the verification script was healthy on seed integrity, but misaligned with the accepted reconcile contract.

## Delivered

Updated `scripts/verify-postgres-baseline.sh` so the end-to-end baseline proof now executes the real bounded reconcile path before the included-ignored DB integration suite:

1. seed baseline,
2. run `cargo run -p seatloom-cli -- reconcile --project seatloom --root "$REPO_ROOT"`,
3. then run `scripts/ingest-documents.sh` as a helper-health check,
4. then run `cargo test -p seatloom-core -- --include-ignored`.

This aligns the verification flow with the architecture contract:
- PostgreSQL is the authority,
- `seatloom reconcile` is the bounded import/update path,
- reconcile bookkeeping and document versioning must exist before the schema-003 tests run.

## Files Changed

Updated:
- `scripts/verify-postgres-baseline.sh`

## Validation

Passed locally on the bounded infra surface:
- `bash -n scripts/verify-postgres-baseline.sh`
- `$HOME/.cargo/bin/cargo check -p seatloom-core`
- `$HOME/.cargo/bin/cargo test -p seatloom-core`
- `$HOME/.cargo/bin/cargo fmt --all --check`
- `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings`
- `$HOME/.cargo/bin/cargo run -p seatloom-cli -- --help`

Environment note:
- Docker is still unavailable on this seat, so end-to-end PostgreSQL proof remains commit-pinned remote verification work for Flux.

## Commit Identity

Infrastructure repair commit pushed for verification:
- branch: `track/infra-foundation`
- target commit: `62f6f901655e3b9a92acae8d84f8277fc126fdd8`
- compare base: `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b`

## Next Step

Issue one more commit-pinned Flux packet against commit `62f6f901655e3b9a92acae8d84f8277fc126fdd8` on the sponsor-provided docker-capable workspace.

The packet should prove:
1. the exact commit was checked out,
2. the narrowed `seatloom-core` Rust surface still passes,
3. `bash scripts/verify-postgres-baseline.sh` now passes end to end,
4. reconcile bookkeeping and document version snapshots are materially present after bootstrap.
