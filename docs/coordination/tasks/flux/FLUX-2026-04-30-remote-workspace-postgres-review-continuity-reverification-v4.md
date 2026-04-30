# Task: Remote Workspace PostgreSQL Review + Continuity Re-Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v4 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v4 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v3.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-reconcile-verification-alignment-v1.md` |
| tags | flux, verification, postgres, remote-workspace, commit-pinned, reverification |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No code edits. No parallel subtasks. Verify only the bounded infra surface and stop on the first real blocker. |

## Why This v4 Packet Exists

v3 proved the earlier seed/ingest hardening was real, but the primary verification script still missed one accepted contract requirement.

Exact failure class on commit `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b`:
- `scripts/verify-postgres-baseline.sh` populated seeded docs but never executed the real bounded reconcile path before the schema-003 tests,
- so `reconcile_runs` remained empty and `document_versions` were never created,
- which failed:
  - `db_baseline::reconcile_run_recorded_after_run`
  - `db_baseline::document_versions_created_on_first_reconcile`

That defect has now been repaired in a new infra-only commit. This v4 packet exists only to re-run the same commit-pinned remote proof path on the repaired commit.

## Objective

Prove or disprove that the PostgreSQL authority + review/continuity baseline is acceptance-ready at the exact repaired commit.

You must verify all of the following on the sponsor-provided docker-capable workspace:
1. the exact target commit is checked out,
2. `$HOME/.cargo/bin/cargo check -p seatloom-core` passes,
3. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes,
4. `bash scripts/verify-postgres-baseline.sh` passes end to end,
5. reconcile bookkeeping and document version snapshots are materially present after bootstrap,
6. the seeded document authority and schema-004 families are still materially present after bootstrap.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `62f6f901655e3b9a92acae8d84f8277fc126fdd8`
- `compare_base_commit`: `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b`

Hard rules:
- do not substitute a later `HEAD`,
- do not verify local working tree state,
- do not run repo-root `cargo check`,
- do not patch code even if you find a defect.

## Remote Login

Use the sponsor-provided workspace exactly as before:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

## Read Scope

Read only:
1. `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v3.md`
2. `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-reconcile-verification-alignment-v1.md`
3. this packet

Inspect code/files only as needed:
- `scripts/verify-postgres-baseline.sh`
- `scripts/ingest-documents.sh`
- `src-cli/src/main.rs`
- `crates/seatloom-core/src/db/reconcile.rs`
- `crates/seatloom-core/tests/db_baseline_integration.rs`

## Write Boundary

Allowed local write locations only:
- `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v4.md`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v4/**`

No code edits.
If you discover any real implementation defect, return `HOLD` with the exact command, exact failing step, and exact error text.

## Execution Steps

### 1. Local evidence directory

```bash
mkdir -p .local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v4/
```

### 2. SSH connectivity probe

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com 'uname -a && hostname'
```

### 3. Refresh exact remote commit

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com '\
  mkdir -p /data/seatloom-verify /data/seatloom-verify/scratch && \
  if [ ! -d /data/seatloom-verify/repo/.git ]; then \
    git clone git@github.com:formycar2/seatloom.git /data/seatloom-verify/repo; \
  fi && \
  cd /data/seatloom-verify/repo && \
  git fetch origin track/infra-foundation && \
  git checkout 62f6f901655e3b9a92acae8d84f8277fc126fdd8 && \
  git rev-parse --abbrev-ref HEAD && \
  git rev-parse HEAD && \
  git status --short \
'
```

### 4. Capture remote environment

Capture only the bounded infra surface:

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
cd /data/seatloom-verify/repo
git rev-parse HEAD
git status --short
git log --oneline -n 5
```

If `git rev-parse HEAD` is not exactly `62f6f901655e3b9a92acae8d84f8277fc126fdd8`, stop and return `HOLD`.
If `git status --short` is non-empty before verification, stop and return `HOLD`.

### 6. Narrow Rust sanity checks

Run exactly these commands, in this order:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
```

Do not run repo-root `cargo check`.

### 7. Primary end-to-end proof point

Run:

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh
```

This is the main acceptance proof point.

### 8. If the primary script fails, isolate the exact failing step

Use this exact fallback sequence and capture every output:

```bash
cd /data/seatloom-verify/repo/infra/postgres && docker compose down -v || true
cd /data/seatloom-verify/repo/infra/postgres && docker compose up -d --wait
docker ps --filter name=seatloom-postgres

docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM reconcile_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM document_versions;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE body_text IS NOT NULL;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM checkpoints;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoff_receipts;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM pipeline_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_threads;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_comments;"

$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored
```

### 9. SQL spot-checks after a passing primary script

Even if the primary script passes, run and capture these:

```bash
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM reconcile_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM document_versions;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE body_text IS NOT NULL;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM checkpoints;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoff_receipts;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM pipeline_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_threads;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_comments;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, trigger, status, scanned, inserted, updated, unchanged FROM reconcile_runs ORDER BY started_at DESC;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT document_id, revision FROM document_versions ORDER BY document_id, revision LIMIT 20;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, review_tier, status FROM review_threads ORDER BY id;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, session_id, trigger FROM checkpoints ORDER BY created_at;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, handoff_id, acknowledged_by FROM handoff_receipts ORDER BY id;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, pipeline_id, workitem_id, status FROM pipeline_runs ORDER BY id;"
```

Expected minimum acceptance signals:
- `reconcile_runs` => non-zero
- `document_versions` => non-zero
- `documents WHERE body_text IS NOT NULL` => non-zero
- `checkpoints` => `3`
- `handoff_receipts` => `3`
- `pipeline_runs` => `1`
- `review_threads` => `1`
- `review_comments` => `2`

## Required Evidence Output

Write full outputs under:
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v4/`

At minimum capture:
- `remote-env.txt`
- `commit-gate.txt`
- `cargo-check-core.txt`
- `cargo-test-core.txt`
- `verify-postgres-baseline.txt`
- `sql-spot-checks.txt`
- if failed: `fallback-isolation.txt`

## Required Delivery Format

Return exactly this structure in your delivery file:

```text
[Flux -> Lyra] PostgreSQL Review + Continuity Re-Verification v4
completed:
- ...
validation:
- `git rev-parse HEAD` => ...
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash scripts/verify-postgres-baseline.sh` => ...
- SQL spot-checks => ...
blockers:
- none
verdict:
- PASS | HOLD
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v4.md
- .local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v4/
```

## Acceptance Rule

Return `PASS` only if all of the following are true:
1. commit identity is exact,
2. bounded `seatloom-core` Rust checks pass,
3. `bash scripts/verify-postgres-baseline.sh` passes end to end,
4. reconcile bookkeeping exists,
5. document versions exist,
6. schema-004 seeded families are present at or above the counts listed above.

Otherwise return `HOLD` with the exact blocker.
