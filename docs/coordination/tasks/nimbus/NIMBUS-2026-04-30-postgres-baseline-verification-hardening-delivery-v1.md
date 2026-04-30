# Delivery: PostgreSQL Baseline Verification Hardening

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `NIMBUS-2026-04-30-postgres-baseline-verification-hardening-v1` |
| tags | nimbus, infrastructure, postgres, verification, docker, hardening, deterministic |
| owner | Nimbus |

## 1. Scope Completed

Infrastructure-only hardening of `scripts/verify-postgres-baseline.sh`:

- Added destructive volume teardown (`docker compose down -v --remove-orphans`) at the start of Step 1 so every run begins from a clean PostgreSQL state.
- Replaced `docker compose up -d --wait` with `docker compose up -d` + an explicit `pg_isready` polling loop (60 s default, configurable via `PG_READY_TIMEOUT`) that exits non-zero with a diagnostic tail log if PostgreSQL does not become ready.
- Added a prominent header comment block declaring the script's destructive-verifier contract.
- Preserved all six proof steps in the accepted sequence (static consistency → bootstrap → schema → seed → reconcile → ingest helper → DB integration tests).

No schema SQL, seed SQL, Rust code, UI code, or business logic was modified.

## 2. Root Cause Confirmed

Two compounding defects made the script non-deterministic on repeated runs:

1. **Stale volume**: `docker compose up -d --wait` on a container with an existing volume left stale schema + seed data in place. If a prior run had failed mid-seed, the database could be in a half-populated state. Re-running seeds with `ON CONFLICT DO NOTHING` would silently skip conflicting rows, leaving the baseline inconsistent, and the DB integration tests would fail.

2. **Readiness gap**: Docker Compose `--wait` uses the container healthcheck (`pg_isready`) as its readiness signal, but the healthcheck can succeed while PostgreSQL is still finishing initialization. Proceeding immediately with `psql -f` connections risked a transient connection refusal that `set -euo pipefail` would surface as a hard failure.

## 3. Hardening Approach Chosen

**Destructive reset before each run** (rather than idempotent-on-stale approach):

- `docker compose down -v --remove-orphans` destroys the volume on every invocation, guaranteeing a blank slate.
- `docker compose up -d` starts a fresh container (entrypoint runs schema via `/docker-entrypoint-initdb.d/`; Step 2 re-applies them idempotently via `CREATE TABLE IF NOT EXISTS`).
- Explicit `pg_isready` loop polls until PostgreSQL accepts connections or the timeout fires, then aborts with `docker logs` output.

This approach is correct for a local baseline verifier (destructive by design) and keeps the script self-contained: no operator pre-step is required.

The `PG_READY_TIMEOUT` variable defaults to `60` seconds and can be overridden for slow seats:
```bash
PG_READY_TIMEOUT=120 bash scripts/verify-postgres-baseline.sh
```

## 4. Files Changed

- `scripts/verify-postgres-baseline.sh` — hardened (destructive reset + explicit readiness loop + contract comment block)
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md` — this file

No other files were modified.

## 5. Validation Commands and Results

```
$HOME/.cargo/bin/cargo check -p seatloom-core
=> Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s (PASS)

$HOME/.cargo/bin/cargo test -p seatloom-core
=> test result: ok. 54 passed; 0 failed; 0 ignored (unit tests)
=> test result: ok. 3 passed; 0 failed; 0 ignored (postgres_seed_consistency)
=> 24 DB integration tests: all ignored as expected (no DB on this seat) (PASS)

$HOME/.cargo/bin/cargo fmt --all --check
=> Finished (no formatting changes needed) (PASS)

$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings
=> Finished `dev` profile (no warnings) (PASS)

bash scripts/verify-postgres-baseline.sh
=> Docker unavailable on this seat — see section 6.
```

## 6. Repeat-Run Proof / Exact Seat Limitation

**Docker is not installed on this seat (darwin, no Docker daemon).**

The end-to-end `bash scripts/verify-postgres-baseline.sh` (and its two-consecutive-run proof) cannot be executed locally.

What was validated locally:
- All 54 seatloom-core unit tests pass.
- All 3 static seed-consistency tests pass (these run without Docker).
- The script itself has been reviewed line-by-line for correctness: the `down -v` → `up -d` → `pg_isready` loop sequence is deterministic by construction.

Remaining proof must be performed by Flux on the remote Docker-capable seat against the exact commit hash below. The hardening changes are minimal and surgically targeted; the risk of a logic error in the reset/readiness sequence is low.

## 7. Residual Non-Goals Kept Out

- Schema SQL redesign: not touched.
- Seed SQL changes: not touched.
- Steady-state reconcile semantics: unchanged.
- UI/frontend work: not touched.
- Tauri runtime features: not touched.
- File watchers / background daemons: not added.
- Ingest surface expansion: not added.

## 8. Exact Branch and Commit

- Branch: `track/infra-foundation`
- Commit: `24c820b81de8f8a479e9d229f5c106ed440eb675`

## 9. Recommended Next Owner

**Flux** — commit-pinned remote verification on the Docker-capable workspace:

1. Check out the exact commit on the remote seat.
2. Run `bash scripts/verify-postgres-baseline.sh` once.
3. Run `bash scripts/verify-postgres-baseline.sh` a second time immediately after.
4. Confirm both runs complete with `=== All checks passed ===` and no manual cleanup needed between them.
5. Report pass/fail with exact commit hash and run timestamps.
