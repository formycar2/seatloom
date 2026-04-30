# Task: Remote Workspace PostgreSQL Review + Continuity Re-Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v2 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-verification-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-verification-delivery-v1.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-operational-review-continuity-supplement-v1.md` |
| tags | flux, verification, postgres, reverify, remote-workspace, commit-pinned |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No product code edits. No parallel subtasks. Verification scope is narrowed and must stay narrowed. |

## Why This v2 Packet Exists

The v1 verification HOLD does **not** indicate a proven schema-004 or seed-003 defect.
It indicates that the v1 packet used an over-broad preflight command:

```bash
cargo check
```

At repo root on the remote Linux seat, that command pulls in the Tauri GTK stack and fails on missing `gdk-3.0`, which is **outside the intended infrastructure verification surface** for this packet.

The actual slice under verification is:
- PostgreSQL schema / seed bootstrap,
- `seatloom-core` DB reads/tests,
- `scripts/verify-postgres-baseline.sh`.

Therefore v2 narrows verification to the intended infra surface only.
Do **not** rerun the repo-root workspace `cargo check` in this packet.

## Objective

Re-run the verification on the same remote workspace and the same exact pinned code commit, but only against the intended infrastructure surface.

You must prove or disprove that:
- `cargo check -p seatloom-core` passes,
- `cargo test -p seatloom-core` passes,
- `bash scripts/verify-postgres-baseline.sh` runs end to end,
- schema 004 / seed 003 boot cleanly and yield the expected persisted objects.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `ee0159fa440b910de38b29824574760722a18316`
- `compare_base_commit`: `a696f8e17dde4b9be95a7e99a1cf2517237cd871`

Hard rule:
- do **not** substitute a later `HEAD`
- do **not** verify local working tree state
- do **not** rerun repo-root workspace `cargo check`

## Remote Login

Use the sponsor-provided workspace:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

## Read Scope

Read only:

1. `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-verification-delivery-v1.md`
2. `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-operational-review-continuity-supplement-v1.md`
3. this packet

Code/files to inspect only as needed:
- `infra/postgres/schema/004_operational_review_and_continuity.sql`
- `infra/postgres/seed/003_operational_review_and_continuity_seed.sql`
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/tests/db_baseline_integration.rs`

## Write Boundary

Allowed local write locations only:
- `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v2.md`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/**`

No code edits.
If you discover a real implementation defect, report `HOLD` or `RE-SCOPED` by exact command and error. Do not patch.

## Execution Steps

### 1. Local evidence directory

```bash
mkdir -p .local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/
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
  git checkout ee0159fa440b910de38b29824574760722a18316 && \
  git rev-parse --abbrev-ref HEAD && \
  git rev-parse HEAD && \
  git status --short \
'
```

### 4. Capture remote environment

Capture only what matters for the narrowed infra path:

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

If `git rev-parse HEAD` is not exactly `ee0159fa440b910de38b29824574760722a18316`, stop and return `HOLD`.

### 6. Narrow Rust sanity checks

Run exactly these, in this order:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
```

Do **not** run repo-root `cargo check`.

### 7. Primary end-to-end proof point

Run:

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh
```

This is the main proof point.

### 8. If the primary script fails, isolate the exact failing step

```bash
cd /data/seatloom-verify/repo/infra/postgres && docker compose down -v || true
cd /data/seatloom-verify/repo/infra/postgres && docker compose up -d --wait
docker ps --filter name=seatloom-postgres

docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM checkpoints;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoff_receipts;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM pipeline_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_threads;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_comments;"

$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored
```

### 9. SQL spot-checks

Even if the primary script passes, run these and capture outputs:

```bash
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

## Required Evidence Output

Write full outputs under:
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/`

Required evidence files:
- `ssh-probe.txt`
- `git-target.txt`
- `remote-env.txt`
- `cargo-check-seatloom-core.txt`
- `cargo-test-seatloom-core.txt`
- `verify-postgres-baseline.txt`
- `docker-ps.txt` (if fallback path used)
- `sql-checkpoints.txt`
- `sql-handoff-receipts.txt`
- `sql-pipeline-runs.txt`
- `sql-review-threads.txt`
- `sql-review-comments.txt`
- `sql-review-thread-rows.txt`
- `sql-checkpoint-rows.txt`
- `sql-handoff-receipt-rows.txt`
- `sql-pipeline-run-rows.txt`
- `cargo-test-include-ignored.txt` (if fallback path used)

## Verification Questions You Must Answer

1. Was the exact pinned commit `ee0159fa440b910de38b29824574760722a18316` verified?
2. Does `cargo check -p seatloom-core` pass on the remote seat?
3. Does `cargo test -p seatloom-core` pass on the remote seat?
4. Does `bash scripts/verify-postgres-baseline.sh` pass end to end?
5. Are the 5 new canonical families present with the expected seeded counts?
6. Do the included DB integration tests pass materially enough to accept schema 004 / seed 003?
7. Is there any actual infra blocker after removing the irrelevant GTK/Tauri preflight?

## Required Delivery Artifact

Write:
- `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v2.md`

Sections required:
1. Verdict
2. Execution environment
3. Command results matrix
4. SQL spot-check results
5. Findings / blockers
6. Evidence file paths
7. Recommended next action

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_pg_review_continuity_reverification_v2.txt
[Flux -> Lyra] PostgreSQL Review + Continuity Re-Verification
completed:
- ...
validation:
- `git rev-parse HEAD` => `ee0159fa440b910de38b29824574760722a18316`
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash scripts/verify-postgres-baseline.sh` => ...
- SQL spot-checks => ...
blockers:
- none / ...
verdict:
- PASS / HOLD / RE-SCOPED
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v2.md
MSG

tmux load-buffer -b flux_to_lyra_pg_review_continuity_reverification_v2 /tmp/flux_to_lyra_pg_review_continuity_reverification_v2.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_pg_review_continuity_reverification_v2
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
