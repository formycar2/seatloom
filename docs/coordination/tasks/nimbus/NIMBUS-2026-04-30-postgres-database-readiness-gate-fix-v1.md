# Task: Nimbus PostgreSQL Database-Readiness Gate Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-delivery-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1.md`, `scripts/verify-postgres-baseline.sh`, `infra/postgres/docker-compose.yml` |
| tags | nimbus, infrastructure, postgres, docker, readiness, verifier, repeat-run |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | Interrupt the broader prompt/channel-authority packet again. Close this verifier blocker first. One bounded infrastructure packet only. Do not touch UI/frontend files or widen into business logic. Preserve your broader packet drafts; do not revert them. |

## Objective

Close the exact PostgreSQL readiness blocker that Flux found on commit `b63656c778cc7f7f4bd508168ceb6c40930d76a9`.

This is a narrow infrastructure-only repair. Do not resume prompt/channel authority until this verifier blocker is fixed, committed, pushed, and reverified by Flux.

## Verified Blocker

Flux verified commit:
- `b63656c778cc7f7f4bd508168ceb6c40930d76a9`

Flux HOLD result:
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => PASS
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => PASS
- `bash scripts/verify-postgres-baseline.sh` run 1 => FAIL at Step 2 (`001_seatloom_core.sql`)

Exact failing command:
- `docker exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom -f "/tmp/001_seatloom_core.sql"`

Exact error:
- `psql: FATAL: database "seatloom" does not exist`

Flux narrowing:
- `pg_isready -U seatloom -d seatloom` can return success before the `seatloom` database actually exists
- the verifier loop exits early on connection acceptance, not database existence
- the container healthcheck currently uses the same insufficient signal

This is the only issue to fix in this packet.

## Important Clarification

Do not treat this as only a Docker Compose healthcheck bug.

The verifier script itself currently waits on:
- `docker exec seatloom-postgres pg_isready -U seatloom -d seatloom -q`

So even if you improve Compose healthcheck timings, the verifier can still proceed too early unless the verifier's own readiness gate is tightened to prove the actual `seatloom` database exists and is queryable.

Fix both layers so they agree on the same durable readiness contract.

## Required Read Order

1. `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-delivery-v1.md`
2. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1.md`
3. `scripts/verify-postgres-baseline.sh`
4. `infra/postgres/docker-compose.yml`
5. this packet

## Required Fix Boundary

Primary files:
- `scripts/verify-postgres-baseline.sh`
- `infra/postgres/docker-compose.yml`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-delivery-v1.md`

Allowed supporting files only if strictly required:
- `docs/infra/ssh-tunnel-workspace.md`
- `docs/architecture-design.md`
- `docs/architecture-decisions.md`

Disallowed:
- schema SQL content changes
- seed SQL changes
- Rust business/domain changes
- UI, Tauri, API, or workflow logic changes
- reverting or disturbing your broader prompt/channel draft files

## Preferred Resolution

Adopt one explicit readiness contract across both Compose healthcheck and the verifier script:
- readiness means the target database `seatloom` exists and can be used for SQL work,
- not merely that the Postgres server socket is accepting connections.

Preferred implementation shape:
1. use a `psql`-based readiness probe that connects to a guaranteed database (`postgres`) and checks catalog existence of `seatloom`, or successfully runs a bounded SQL statement against `seatloom` itself,
2. update `scripts/verify-postgres-baseline.sh` to wait on that stronger condition,
3. update `infra/postgres/docker-compose.yml` healthcheck to use the same contract or an equivalent one.

Examples of acceptable proof conditions:
- `psql -U seatloom -d postgres -tAc "SELECT 1 FROM pg_database WHERE datname = 'seatloom'" | grep -q 1`
- or an equivalent query that proves the target DB exists before Step 2 proceeds.

Keep it simple. Do not over-engineer.

## Acceptance Criteria

1. The verifier no longer advances to schema apply before `seatloom` exists.
2. Compose healthcheck and verifier readiness logic no longer rely on bare `pg_isready` as the sole truth signal.
3. The proof sequence remains explicit and deterministic.
4. No schema/seed/Rust/UI scope is widened.
5. The result is commit-pinned and ready for Flux remote reverification.

## Required Validation

At minimum:

```bash
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
bash -n scripts/verify-postgres-baseline.sh
```

If Docker is available on your seat, also run:

```bash
bash scripts/verify-postgres-baseline.sh
bash scripts/verify-postgres-baseline.sh
```

If Docker is unavailable, record that limitation exactly and stop there. Flux will do the remote proof.

## Branch and Commit Discipline

Stay on:
- `track/infra-foundation`

When done:
1. commit only the bounded fix from this packet,
2. push the branch,
3. report the exact commit hash,
4. explicitly state that the broader prompt/channel packet remains paused and intact.

## Required Delivery Artifact

Write:
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-delivery-v1.md`

Required sections:
1. Flux blocker restated
2. Readiness contract chosen
3. Files changed
4. Validation commands and results
5. Docker availability note
6. Exact branch and commit
7. Status of the broader prompt/channel packet

## Done Definition

- [ ] Delivery artifact written.
- [ ] Verifier readiness waits for actual database existence/useability.
- [ ] Compose healthcheck no longer gives a weaker success signal than the verifier.
- [ ] Exact commit hash reported.
- [ ] tmux reply sent to Lyra.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_pg_db_readiness_fix.txt
[Nimbus -> Lyra] PostgreSQL Database-Readiness Gate Fix
branch:
- track/infra-foundation
commit:
- <exact commit>
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash -n scripts/verify-postgres-baseline.sh` => ...
- `bash scripts/verify-postgres-baseline.sh` first run => ... / not run because ...
- `bash scripts/verify-postgres-baseline.sh` second run => ... / not run because ...
blockers:
- none / ...
next action:
- wait for Flux commit-pinned reverification and Lyra acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_pg_db_readiness_fix /tmp/nimbus_to_lyra_pg_db_readiness_fix.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_pg_db_readiness_fix
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
