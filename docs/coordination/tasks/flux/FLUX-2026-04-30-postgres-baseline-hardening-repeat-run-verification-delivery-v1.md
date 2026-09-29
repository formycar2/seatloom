# Delivery: Flux PostgreSQL Baseline Hardening Repeat-Run Verification

| Field | Value |
|---|---|
| ID | FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-delivery-v1 |
| Status | Delivered |
| Author | Flux |
| Date | 2026-04-30 |
| Verdict | HOLD |

## Completed

- Target commit `24c820b81de8f8a479e9d229f5c106ed440eb675` verified on remote workspace
- Rust sanity checks passed (`cargo check -p seatloom-core`, `cargo test -p seatloom-core`)
- First verifier run FAILED at Step 2 (duplicate key on enum type `project_role_bindings`)
- Second run NOT executed (stopped at first real blocker per packet instructions)

## Validation

- `git rev-parse HEAD` => `24c820b81de8f8a479e9d229f5c106ed440eb675` ✓
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => EXIT 0 ✓
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => 54 passed, 0 failed ✓
- `bash scripts/verify-postgres-baseline.sh` run 1 => **FAILED** (see blockers)
- `bash scripts/verify-postgres-baseline.sh` run 2 => NOT REACHED
- SQL spot-checks => NOT REACHED

## Blockers

**Step 2 (Apply schema)** — script redundantly applies schema files that PostgreSQL auto-applies during init:

```
psql:/docker-entrypoint-initdb.d/001_seatloom_core.sql:42: ERROR: duplicate key value violates unique constraint "pg_type_typname_nsp_index"
DETAIL: Key (typname, typnamespace)=(project_role_bindings, 2200) already exists.
```

**Root cause:** Docker Compose mounts `infra/postgres/schema/` at `/docker-entrypoint-initdb.d/`. On fresh volume (after `docker compose down -v`), PostgreSQL's official entrypoint auto-applies all `.sql` files during first startup. The script then manually re-applies the same files in Step 2, causing immediate duplicate-key failures on enum types and tables.

## Verdict

**HOLD** — The verification script has an architectural conflict: it cannot both mount schema at the PostgreSQL auto-init directory and manually re-apply the same files. Nimbus must resolve this before the repeat-run hardening claim can be validated.

## Next Action

Nimbus must choose one approach:

1. **Remove schema mount** from `infra/postgres/docker-compose.yml` — keep manual schema+seed application in the script (original design), OR
2. **Remove Steps 2-3** from `scripts/verify-postgres-baseline.sh` — rely entirely on PostgreSQL auto-init from the mounted schema/seed directories

Either approach makes repeat runs deterministic. The current state is broken.

## Evidence Paths

- `.local/evidence/2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1/run-1.txt`
- `.local/evidence/2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1/cargo-check.txt`
- `.local/evidence/2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1/cargo-test.txt`
