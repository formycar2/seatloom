# Task: Flux PostgreSQL Database-Readiness Gate Re-Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-30-postgres-database-readiness-gate-reverification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-delivery-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-delivery-v1.md`, `docs/infra/ssh-tunnel-workspace.md` |
| tags | flux, verification, postgres, docker, remote-workspace, commit-pinned, readiness, repeat-run |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No code edits. No parallel subtasks. Verify only the bounded infra surface and stop on the first real blocker. |

## Objective

Prove or disprove that Nimbus's database-readiness gate fix actually closes the remaining repeat-run PostgreSQL verifier blocker.

This is a narrow, commit-pinned remote verification packet. Do not widen into UI, Tauri, or general repo review.

## What Changed Since the Last HOLD

Earlier HOLD, verified by you on commit `b63656c778cc7f7f4bd508168ceb6c40930d76a9`:
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => PASS
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => PASS
- `bash scripts/verify-postgres-baseline.sh` run 1 => FAIL at Step 2
- exact error: `psql: FATAL: database "seatloom" does not exist`

Root cause you identified:
- bare `pg_isready` can succeed before the `seatloom` database actually exists
- both the compose healthcheck and the verifier script used that weaker signal
- the verifier advanced to schema apply before the target database was truly queryable

Nimbus now claims this is fixed by commit `b9bb7340416e3729f30dc751a4bb1f41ee726520`:
- verifier readiness loop now waits on `psql -U seatloom -d seatloom -c "SELECT 1" -q`
- compose healthcheck now uses the same `psql`-based readiness contract
- compose retries raised from 10 to 20
- both layers now require actual `seatloom` DB queryability, not just socket readiness

Your job is to verify that exact claim against the exact commit.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `b9bb7340416e3729f30dc751a4bb1f41ee726520`
- `compare_base_commit`: `b63656c778cc7f7f4bd508168ceb6c40930d76a9`

Hard rules:
- do not substitute a later `HEAD`
- do not verify local working-tree state
- do not patch code even if you find a defect
- do not run repo-root `cargo check`
- do not widen into GTK/Tauri dependency issues

## Remote Login

Use the sponsor-provided workspace exactly as documented:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

## Read Scope

Read only:
1. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-database-readiness-gate-fix-delivery-v1.md`
2. `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-init-conflict-reverification-delivery-v1.md`
3. `docs/infra/ssh-tunnel-workspace.md`
4. this packet

Inspect code/files only as needed:
- `scripts/verify-postgres-baseline.sh`
- `infra/postgres/docker-compose.yml`
- `infra/postgres/schema/*.sql`
- `infra/postgres/seed/*.sql`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `crates/seatloom-core/tests/postgres_seed_consistency.rs`

## Write Boundary

Allowed local write locations only:
- `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-database-readiness-gate-reverification-delivery-v1.md`
- `.local/evidence/2026-04-30-postgres-database-readiness-gate-reverification-v1/**`

No code edits.
No doc edits outside the delivery artifact.
If you discover a real implementation defect, return `HOLD` with the exact failing command, exact failing step, and the narrowest real blocker only.

## Execution Steps

### 1. Local evidence directory

```bash
mkdir -p .local/evidence/2026-04-30-postgres-database-readiness-gate-reverification-v1/
```

### 2. SSH connectivity probe

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com 'uname -a && hostname'
```

### 3. Refresh exact remote commit in a clean verification repo

Use a fresh path to avoid dirty-state confusion from earlier verification runs.

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com '\
  rm -rf /data/seatloom-verify-db-readiness && \
  mkdir -p /data/seatloom-verify-db-readiness && \
  git clone git@github.com:formycar2/seatloom.git /data/seatloom-verify-db-readiness/repo && \
  cd /data/seatloom-verify-db-readiness/repo && \
  git fetch origin track/infra-foundation && \
  git checkout b9bb7340416e3729f30dc751a4bb1f41ee726520 && \
  git rev-parse --abbrev-ref HEAD && \
  git rev-parse HEAD && \
  git status --short \
'
```

### 4. Capture bounded remote environment

Run and capture:

```bash
uname -a
hostname
pwd
which docker || true
docker --version || true
docker compose version || true
$HOME/.cargo/bin/cargo --version || true
```

### 5. Commit identity gate

```bash
cd /data/seatloom-verify-db-readiness/repo
git rev-parse HEAD
git status --short
git log --oneline -n 5
```

Stop and return `HOLD` if either condition is true:
- `git rev-parse HEAD` is not exactly `b9bb7340416e3729f30dc751a4bb1f41ee726520`
- `git status --short` is non-empty before verification starts

### 6. Narrow Rust sanity checks

Run exactly these commands, in this order:

```bash
cd /data/seatloom-verify-db-readiness/repo
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
```

Do not run repo-root `cargo check`.

### 7. Primary proof point: first verifier run

```bash
cd /data/seatloom-verify-db-readiness/repo
bash scripts/verify-postgres-baseline.sh | tee .local/evidence/2026-04-30-postgres-database-readiness-gate-reverification-v1/run-1.txt
```

### 8. Primary proof point: second verifier run immediately after the first

Do not manually run `docker compose down -v`, `docker compose up`, or any cleanup between runs.

```bash
cd /data/seatloom-verify-db-readiness/repo
bash scripts/verify-postgres-baseline.sh | tee .local/evidence/2026-04-30-postgres-database-readiness-gate-reverification-v1/run-2.txt
```

### 9. SQL spot checks after the second passing run

Run and capture these exact commands:

```bash
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM reconcile_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM document_versions;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE body_text IS NOT NULL;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE template IS NULL;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT template, COUNT(*) FROM documents GROUP BY template ORDER BY template;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM checkpoints;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoff_receipts;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM pipeline_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_threads;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_comments;"
```

Expected minimum acceptance signals:
- both verifier runs exit `0`
- both verifier runs end with `=== All checks passed ===`
- `documents WHERE template IS NULL` returns `0`
- `reconcile_runs` is non-zero
- `document_versions` is non-zero
- `documents WHERE body_text IS NOT NULL` is non-zero
- `checkpoints`, `handoff_receipts`, `pipeline_runs`, `review_threads`, and `review_comments` are all non-zero

### 10. If any verifier run fails, isolate the narrowest real blocker

Use this fallback sequence and capture all output:

```bash
cd /data/seatloom-verify-db-readiness/repo/infra/postgres && docker compose ps
cd /data/seatloom-verify-db-readiness/repo/infra/postgres && docker compose logs --tail 120

docker exec seatloom-postgres psql -U seatloom -d postgres -c "SELECT datname FROM pg_database ORDER BY datname;" || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM reconcile_runs;" || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM document_versions;" || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE template IS NULL;" || true
```

Do not patch. Return `HOLD`.

## Delivery Format

Return exactly this structure in the delivery markdown and tmux summary:

```text
[Flux -> Lyra] PostgreSQL Database-Readiness Gate Re-Verification
completed:
- ...
validation:
- `git rev-parse HEAD` => ...
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash scripts/verify-postgres-baseline.sh` run 1 => ...
- `bash scripts/verify-postgres-baseline.sh` run 2 => ...
- SQL spot-checks => ...
blockers:
- none / ...
verdict:
- PASS | HOLD
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-database-readiness-gate-reverification-delivery-v1.md
- .local/evidence/2026-04-30-postgres-database-readiness-gate-reverification-v1/
```
