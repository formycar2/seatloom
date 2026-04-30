# Task: Remote Workspace PostgreSQL Review + Continuity Re-Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v3 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v3 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-seed-ingest-hardening-v1.md` |
| tags | flux, verification, postgres, remote-workspace, commit-pinned, reverification |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No code edits. No parallel subtasks. Verify only the bounded infra surface and stop on the first real blocker. |

## Why This v3 Packet Exists

v2 proved a real infrastructure blocker on the exact pinned commit `ee0159fa440b910de38b29824574760722a18316`:
- missing artifact baseline row for `ar-task-db-baseline`,
- verification script not stopping immediately on SQL seed errors.

That defect has now been repaired in a new infra-only commit. This v3 packet exists only to re-run the same commit-pinned remote proof path on the repaired commit.

## Objective

Prove or disprove that the PostgreSQL authority + review/continuity baseline is acceptance-ready at the exact repaired commit.

You must verify all of the following on the sponsor-provided docker-capable workspace:
1. the exact target commit is checked out,
2. `$HOME/.cargo/bin/cargo check -p seatloom-core` passes,
3. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes,
4. `bash scripts/verify-postgres-baseline.sh` passes end to end,
5. the seeded document authority and schema-004 families are materially present after bootstrap.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b`
- `compare_base_commit`: `ee0159fa440b910de38b29824574760722a18316`

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
1. `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2.md`
2. `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-seed-ingest-hardening-v1.md`
3. this packet

Inspect code/files only as needed:
- `infra/postgres/seed/001_real_collaboration_baseline.sql`
- `infra/postgres/seed/002_document_seed.sql`
- `infra/postgres/seed/003_operational_review_and_continuity_seed.sql`
- `scripts/verify-postgres-baseline.sh`
- `scripts/ingest-documents.sh`
- `crates/seatloom-core/tests/postgres_seed_consistency.rs`
- `crates/seatloom-core/tests/db_baseline_integration.rs`

## Write Boundary

Allowed local write locations only:
- `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v3.md`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v3/**`

No code edits.
If you discover any real implementation defect, return `HOLD` with the exact command, exact failing step, and exact error text.

## Execution Steps

### 1. Local evidence directory

```bash
mkdir -p .local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v3/
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
  git checkout 7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b && \
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

If `git rev-parse HEAD` is not exactly `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b`, stop and return `HOLD`.
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

docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM artifacts WHERE id = 'ar-task-db-baseline';"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE artifact_id = 'ar-task-db-baseline';"
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
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM artifacts WHERE id = 'ar-task-db-baseline';"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE artifact_id = 'ar-task-db-baseline';"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE body_text IS NOT NULL;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM checkpoints;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoff_receipts;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM pipeline_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_threads;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_comments;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, review_tier, status FROM review_threads ORDER BY id;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, session_id, trigger FROM checkpoints ORDER BY created_at;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, handoff_id, acknowledged_by FROM handoff_receipts ORDER BY id;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, pipeline_id, workitem_id, status FROM pipeline_runs ORDER BY id;"
```

Expected minimum acceptance signals:
- `artifacts WHERE id = 'ar-task-db-baseline'` => `1`
- `documents WHERE artifact_id = 'ar-task-db-baseline'` => `1`
- `documents WHERE body_text IS NOT NULL` => non-zero
- `checkpoints` => `3`
- `handoff_receipts` => `3`
- `pipeline_runs` => `1`
- `review_threads` => `1`
- `review_comments` => `2`

## Required Evidence Output

Write full outputs under:
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v3/`

Required evidence files:
- `ssh-probe.txt`
- `git-target.txt`
- `remote-env.txt`
- `cargo-check-seatloom-core.txt`
- `cargo-test-seatloom-core.txt`
- `verify-postgres-baseline.txt`
- `docker-ps.txt` if fallback path used
- `sql-artifact-db-baseline.txt`
- `sql-document-db-baseline.txt`
- `sql-documents-with-body.txt`
- `sql-checkpoints.txt`
- `sql-handoff-receipts.txt`
- `sql-pipeline-runs.txt`
- `sql-review-threads.txt`
- `sql-review-comments.txt`
- `sql-review-thread-rows.txt`
- `sql-checkpoint-rows.txt`
- `sql-handoff-receipt-rows.txt`
- `sql-pipeline-run-rows.txt`
- `cargo-test-include-ignored.txt` if fallback path used

## Verification Questions You Must Answer

1. Was the exact pinned commit `7646d5ac1800b3af87e2f33f3da090fd7cfb9c3b` verified?
2. Does `$HOME/.cargo/bin/cargo check -p seatloom-core` pass on the remote seat?
3. Does `$HOME/.cargo/bin/cargo test -p seatloom-core` pass on the remote seat?
4. Does `bash scripts/verify-postgres-baseline.sh` pass end to end?
5. Is the repaired `ar-task-db-baseline` artifact/document chain materially present after bootstrap?
6. Are the schema-004 continuity/review families present with the expected seeded counts?
7. Do the included-ignored DB integration tests pass materially enough to accept this repaired baseline?
8. Is any real infra blocker still present on this exact commit?

## Required Delivery Artifact

Write:
- `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v3.md`

Required sections:
1. Verdict
2. Execution environment
3. Command results matrix
4. SQL spot-check results
5. Findings / blockers
6. Evidence file paths
7. Recommended next action

## Direct reply contract

Return using this exact structure:

```text
[Flux -> Lyra] PostgreSQL Review + Continuity Re-Verification v3
completed:
- ...
validation:
- `git rev-parse HEAD` => ...
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash scripts/verify-postgres-baseline.sh` => ...
- SQL spot-checks => ...
blockers:
- none OR exact blocker
verdict:
- PASS / HOLD
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v3.md
```
