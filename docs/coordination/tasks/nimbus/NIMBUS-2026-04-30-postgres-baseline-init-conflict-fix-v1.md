# Task: Nimbus PostgreSQL Baseline Init-Conflict Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-delivery-v1.md`, `scripts/verify-postgres-baseline.sh`, `infra/postgres/docker-compose.yml` |
| tags | nimbus, infrastructure, postgres, docker, verifier, repeat-run, init-conflict |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | Interrupt the broader prompt/channel-authority packet. Close this verifier blocker first. One bounded infrastructure packet only. Do not touch UI/frontend files and do not widen into business logic. |

## Objective

Close the exact repeat-run verifier blocker that Flux found on the commit-pinned PostgreSQL baseline hardening path.

This is a narrow infrastructure-only repair. Do not work on prompt/channel authority until this blocker is fixed, committed, and pushed.

## Verified Blocker

Flux verified commit:
- `24c820b81de8f8a479e9d229f5c106ed440eb675`

Flux HOLD result:
- `cargo check -p seatloom-core` => PASS
- `cargo test -p seatloom-core` => PASS
- first `bash scripts/verify-postgres-baseline.sh` run => FAIL at schema apply

Root cause narrowed by Flux:
- `infra/postgres/docker-compose.yml` mounts `./schema` into `/docker-entrypoint-initdb.d`
- on a fresh volume, PostgreSQL init auto-applies the schema during first boot
- `scripts/verify-postgres-baseline.sh` then manually applies the same schema again in Step 2
- repeat-run hardening claim is therefore invalid because the verifier conflicts with its own container bootstrap path

This is the only issue to fix in this packet.

## Required Read Order

1. `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-delivery-v1.md`
2. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md`
3. `scripts/verify-postgres-baseline.sh`
4. `infra/postgres/docker-compose.yml`
5. this packet

## Required Fix Boundary

Primary files:
- `scripts/verify-postgres-baseline.sh`
- `infra/postgres/docker-compose.yml`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1.md`

Allowed supporting files only if strictly required:
- `docs/infra/ssh-tunnel-workspace.md`
- `docs/architecture-design.md`
- `docs/architecture-decisions.md`

Disallowed:
- any schema SQL content changes unless strictly required to keep the script functional after the conflict fix
- any seed content changes
- any Rust business/domain changes
- any UI, Tauri, or API changes

## Preferred Resolution

Make the verifier script the explicit authority for schema application.

That means:
1. remove the hidden schema auto-apply path from `docker-compose.yml`,
2. keep schema application explicit inside `scripts/verify-postgres-baseline.sh`,
3. if needed, copy schema files into the container just like seed files are copied now,
4. preserve the accepted proof sequence as visible script steps.

Why this is preferred:
- the verifier remains explicit rather than relying on Docker entrypoint side effects,
- the accepted proof sequence stays understandable (`schema -> seed -> reconcile -> tests`),
- repeat-run behavior becomes deterministic,
- there is only one authority for schema apply in the verification path.

If you believe the opposite choice is materially better, you may take it only if you document the tradeoff clearly and keep the proof path deterministic and Flux-verifiable.

## Acceptance Criteria

1. There is no duplicate schema application path on a clean verifier run.
2. `bash scripts/verify-postgres-baseline.sh` is logically safe for consecutive runs on the same seat.
3. The schema apply step remains explicit and understandable.
4. No seed, Rust product logic, or UI scope is widened.
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
4. explicitly state that the earlier broader packet is still pending or resumed afterward.

## Required Delivery Artifact

Write:
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1.md`

Required sections:
1. Flux blocker restated
2. Resolution chosen
3. Files changed
4. Validation commands and results
5. Docker availability note
6. Exact branch and commit
7. Status of the broader prompt/channel packet

## Done Definition

- [ ] Delivery artifact written.
- [ ] Duplicate schema-apply path removed.
- [ ] Verifier script remains deterministic and explicit.
- [ ] Exact commit hash reported.
- [ ] tmux reply sent to Lyra.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_pg_init_conflict_fix.txt
[Nimbus -> Lyra] PostgreSQL Baseline Init-Conflict Fix
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
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-init-conflict-fix-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_pg_init_conflict_fix /tmp/nimbus_to_lyra_pg_init_conflict_fix.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_pg_init_conflict_fix
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
