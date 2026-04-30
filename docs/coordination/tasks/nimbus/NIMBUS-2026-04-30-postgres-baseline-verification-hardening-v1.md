# Task: Nimbus PostgreSQL Baseline Verification Hardening

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-postgres-baseline-verification-hardening-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/architecture-design.md`, `docs/architecture-decisions.md`, `scripts/verify-postgres-baseline.sh`, `infra/postgres/docker-compose.yml`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-document-metadata-reconcile-fix-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v5.md` |
| tags | nimbus, infrastructure, postgres, verification, docker, hardening, deterministic |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded infrastructure packet only. You are not alone in the codebase; do not revert others' work, do not widen into product/business logic, and do not touch UI/frontend files. |

## Objective

Harden the PostgreSQL baseline verification path so it is deterministic on repeated runs and does not require manual cleanup or ad hoc operator judgment.

This is an infrastructure-only follow-up to a now-functional baseline. Do not widen into business code, runtime features, APIs, or frontend work.

## Why This Packet Exists

Flux's remote verification established two things:

1. the PostgreSQL baseline implementation is functionally correct after clean isolation,
2. the current verification script is not yet robust as an engineering gate.

Observed behavior:
- a clean isolated run passes,
- repeated runs against a stale Docker volume can fail with duplicate-seed / duplicate-projection symptoms,
- Docker Compose health can report ready before PostgreSQL is fully ready for deterministic seeding/reconcile.

So the functional blocker is closed, but the verification gate still needs hardening.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/architecture-design.md`
3. `docs/architecture-decisions.md`
4. `scripts/verify-postgres-baseline.sh`
5. `infra/postgres/docker-compose.yml`
6. `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-document-metadata-reconcile-fix-v1.md`
7. `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v5.md`
8. this packet

## Need-to-Know Scope

In scope:
- make the baseline verification script self-resetting or equivalently deterministic for repeated local/remote runs,
- add explicit PostgreSQL readiness waiting beyond Docker health if needed,
- keep the script's contract clear: this is a destructive baseline-bootstrap verifier, not a steady-state runtime sync path,
- preserve the bounded PG baseline flow: schema -> seed -> reconcile -> helper health check -> DB integration tests,
- update any directly related coordination docs if the script contract becomes more explicit,
- commit only the bounded infra changes and report the exact commit for Flux.

Out of scope:
- schema redesign,
- changing steady-state reconcile semantics,
- product/business logic,
- UI/frontend work,
- Tauri runtime feature work,
- adding file watchers/background daemons,
- expanding the ingest surface.

## Branch and Commit Discipline

Stay on:
- `track/infra-foundation`

When done:
1. commit only the bounded infrastructure changes from this packet,
2. push the branch,
3. report the exact commit hash in the delivery artifact and tmux reply,
4. keep the packet Flux-verifiable by exact commit.

## Write Boundary

Primary targets:
- `scripts/verify-postgres-baseline.sh`
- `infra/postgres/docker-compose.yml` (only if strictly required for deterministic verification behavior)
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md`

Allowed supporting touch points only if strictly needed:
- `docs/architecture-decisions.md`
- `docs/architecture-design.md`

Disallowed in this packet:
- files under `ui/`
- schema SQL changes unless you find a real infra defect that makes the script-only hardening impossible
- widening CLI/Tauri behavior
- business/domain model changes

## Required Outcome

### 1. Make repeated verification deterministic

`bash scripts/verify-postgres-baseline.sh` should no longer depend on the operator manually doing `docker compose down -v` before use.

Preferred outcome:
- the script itself performs the required clean reset for the local verification database volume,
- or an equally deterministic bounded mechanism is built in and documented.

The verification gate should be safe to run repeatedly on the same machine without stale-volume false failures.

### 2. Add explicit readiness gating

Docker `--wait` alone is not sufficient proof of Postgres readiness for this seed/reconcile path.

Add an explicit readiness loop after container startup, for example using `pg_isready`, with:
- deterministic timeout,
- clear error if readiness is not reached,
- no silent fallthrough.

### 3. Keep the contract explicit

The script should clearly communicate that it is:
- a destructive local baseline verifier for the repo-managed PG baseline,
- not the steady-state production write path,
- and therefore allowed to reset the verification volume.

If you choose an environment variable or flag for cleanup policy, keep it simple and document it. Do not over-engineer.

### 4. Preserve the accepted baseline flow

Do not change the high-level proof sequence except where hardening requires it:
1. static seed consistency,
2. clean PG bootstrap,
3. schema apply,
4. seed apply,
5. bounded reconcile,
6. helper health check,
7. DB integration tests.

### 5. Prove repeatability

If Docker is available on your seat, prove the hardening by running the full script at least twice consecutively.

If Docker is unavailable on your seat:
- validate everything else you can locally,
- state the exact limitation,
- and keep the result Flux-ready by exact commit.

## Acceptance Criteria

1. `scripts/verify-postgres-baseline.sh` is deterministic across repeated runs on the same seat.
2. The script no longer requires manual `docker compose down -v` pre-clean by the verifier.
3. PostgreSQL readiness is explicitly checked before schema/seed/reconcile steps continue.
4. The script still exits non-zero on real failures.
5. The accepted baseline proof sequence remains intact.
6. No UI/business/runtime feature scope is added.
7. The resulting commit is Flux-verifiable by exact hash.

## Required Validation

At minimum:

```bash
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
bash scripts/verify-postgres-baseline.sh
bash scripts/verify-postgres-baseline.sh
```

If only one full run is possible due seat limits, state that explicitly and why.

## Required Delivery Artifact

Write:
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md`

Required sections:
1. Scope completed
2. Root cause confirmed
3. Hardening approach chosen
4. Files changed
5. Validation commands and results
6. Repeat-run proof or exact seat limitation
7. Residual non-goals kept out
8. Exact branch and commit
9. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] The script has deterministic cleanup behavior.
- [ ] The script has explicit readiness gating.
- [ ] The script remains infra-only in scope.
- [ ] Validation commands/results are recorded.
- [ ] Exact commit hash is reported.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_pg_verify_hardening.txt
[Nimbus -> Lyra] PostgreSQL Baseline Verification Hardening
branch:
- track/infra-foundation
commit:
- <exact commit>
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash scripts/verify-postgres-baseline.sh` first run => ...
- `bash scripts/verify-postgres-baseline.sh` second run => ... / not run because ...
blockers:
- none / ...
next action:
- wait for Flux commit-pinned verification and Lyra acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_pg_verify_hardening /tmp/nimbus_to_lyra_pg_verify_hardening.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_pg_verify_hardening
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
