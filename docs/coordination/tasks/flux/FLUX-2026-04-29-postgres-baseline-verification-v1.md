# Task: PostgreSQL Baseline Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-postgres-baseline-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-29-infra-method-review-gate-v1.md` |
| tags | flux, verification, postgres, docker, persistence, real-data, repositories |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No feature implementation. No schema edits. No scope widening beyond verification of the delivered PostgreSQL baseline. |

## Objective

Verify Nimbus's PostgreSQL collaboration-truth baseline on a docker-capable seat.

This is a read-mostly verification packet with one execution goal:

- prove that the repo-managed PostgreSQL path can actually start,
- apply schema,
- seed the real collaboration baseline,
- and pass the DB integration test path end to end.

Nimbus has already completed the implementation packet and validated Rust-only checks. The open gap is the docker-capable end-to-end proof.

## What You Are Verifying

Implementation under verification:

- `infra/postgres/docker-compose.yml`
- `infra/postgres/schema/001_seatloom_core.sql`
- `infra/postgres/seed/001_real_collaboration_baseline.sql`
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/src/db/`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `.seatloom/bootstrap/source-map.yaml`

Delivery artifact under review:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`

## Read Scope

Read only these before execution:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
3. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`
4. this packet

Read in code only what you need to verify:

- `infra/postgres/**`
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/src/db/**`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- any directly referenced Cargo manifest files

Do not widen into UI, archived docs, or unrelated Rust modules.

## Write Boundary

Allowed write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`
- `.local/evidence/2026-04-29-postgres-baseline-verification/**`

No product code patches. If you find implementation issues, report them; do not fix them in this packet.

## Execution Steps

Run in this order and capture full stdout/stderr to evidence files.

### 1. Environment capture

```bash
docker --version

docker compose version

$HOME/.cargo/bin/cargo --version
```

### 2. Rust-only sanity check

```bash
$HOME/.cargo/bin/cargo check

$HOME/.cargo/bin/cargo test -p seatloom-core
```

### 3. End-to-end PostgreSQL bootstrap path

Primary command:

```bash
bash scripts/verify-postgres-baseline.sh
```

This command must be treated as the primary proof point. Do not replace it with ad hoc partial checks unless it fails.

### 4. If the primary script fails, isolate the exact failing step

If `scripts/verify-postgres-baseline.sh` fails, run these stepwise commands and capture outputs separately:

```bash
cd infra/postgres && docker compose up -d --wait

docker ps --filter name=seatloom-postgres

docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM seats;"

docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM artifacts;"

docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM canonical_events;"

$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored
```

### 5. Coverage spot-checks

Confirm the seeded baseline includes these truths:

- 5 seats: aegis, lyra, mira, nimbus, flux
- at least 1 delegation capturing Flux acting for Mira
- active contract artifacts present
- at least one artifact spanning each family `T1` through `T7`

Use SQL queries or repository-driven evidence, but keep it bounded and deterministic.

## Required Evidence Output

Write full outputs under:

- `.local/evidence/2026-04-29-postgres-baseline-verification/`

Required evidence files:

- `docker-version.txt`
- `docker-compose-version.txt`
- `cargo-version.txt`
- `cargo-check.txt`
- `cargo-test-unit.txt`
- `verify-postgres-baseline.txt`
- `docker-ps.txt` (if fallback path used)
- `sql-seat-count.txt` (if fallback path used)
- `sql-artifact-count.txt` (if fallback path used)
- `sql-event-count.txt` (if fallback path used)
- `cargo-test-include-ignored.txt` (if fallback path used)

## Verification Questions You Must Answer

Answer all of these explicitly in the delivery artifact:

1. Does the repo-managed PostgreSQL service actually start on a verification-capable seat?
2. Does the baseline script run end to end without manual repair?
3. Do ignored DB integration tests pass when explicitly included?
4. Does seeded data match Nimbus's claimed object-family coverage closely enough to accept the packet?
5. Is the packet still infrastructure-only, with no business/UI widening?
6. Is there any hidden operational flaw in the docker-compose or seed strategy that should block acceptance?

## Done Definition

- [ ] Environment versions captured.
- [ ] `cargo check` result captured.
- [ ] `cargo test -p seatloom-core` result captured.
- [ ] `bash scripts/verify-postgres-baseline.sh` executed and result captured.
- [ ] If script fails, fallback isolation commands executed and captured.
- [ ] Coverage spot-checks completed.
- [ ] Delivery artifact written at `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`.
- [ ] Delivery artifact issues a clear verdict: `PASS`, `HOLD`, or `RE-SCOPED`.

## Pass / Hold / Re-Scoped Rules

### PASS

Use `PASS` only if all of the following are true:

- docker and docker compose are available;
- `bash scripts/verify-postgres-baseline.sh` succeeds end to end;
- DB integration tests pass when included;
- spot-check coverage is materially consistent with Nimbus's delivery claim;
- no blocking infra flaw is found.

### HOLD

Use `HOLD` if the implementation looks correct but the verification seat lacks the required docker capability or another environment-only prerequisite. In that case, separate environment blockers from implementation findings.

### RE-SCOPED

Use `RE-SCOPED` if a real implementation flaw is found. List:

- exact failing command
- exact error
- probable root cause
- recommended owner
- bounded fix scope

## Non-Goals

- No code changes.
- No schema redesign.
- No new migrations.
- No API work.
- No UI work.
- No reopening method choice; PostgreSQL is already sponsor-directed.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`

Required sections:

1. Verdict (`PASS`, `HOLD`, or `RE-SCOPED`)
2. Execution environment
3. Command results matrix
4. Coverage spot-check results
5. Findings / blockers
6. Evidence file paths
7. Recommended next action

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_postgres_baseline_verification.txt
[Flux -> Lyra] PostgreSQL Baseline Verification
completed:
- ...
validation:
- `docker --version` => ...
- `docker compose version` => ...
- `$HOME/.cargo/bin/cargo check` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash scripts/verify-postgres-baseline.sh` => ...
blockers:
- none / ...
verdict:
- PASS / HOLD / RE-SCOPED
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md
MSG

tmux load-buffer -b flux_to_lyra_postgres_baseline_verification /tmp/flux_to_lyra_postgres_baseline_verification.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_postgres_baseline_verification
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
