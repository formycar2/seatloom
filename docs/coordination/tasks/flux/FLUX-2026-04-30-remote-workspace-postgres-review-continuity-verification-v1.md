# Task: Remote Workspace PostgreSQL Review + Continuity Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-30-remote-workspace-postgres-review-continuity-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-operational-review-continuity-supplement-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-authority-and-typed-documents-delivery-v1.md`, `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md` |
| tags | flux, verification, postgres, remote-workspace, continuity, review, commit-pinned |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. Verification first. No parallel subtasks. No feature work. If you discover a low-level defect, stop, isolate it, and report by exact commit and command before touching code. |

## Objective

Verify the new PostgreSQL authority supplement for operational review + continuity on a docker-capable remote workspace, pinned to the exact published commit.

This packet proves that the new schema-004 slice is not just locally green, but actually boots end to end on PostgreSQL with the real repo-managed verification path.

The slice under verification adds first-class persistence for:
- checkpoints,
- handoff receipts,
- pipeline runs,
- review threads,
- review comments.

This is infrastructure-only verification.
Do not widen into UI, product behavior, business logic, or frontend review.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `ee0159fa440b910de38b29824574760722a18316`
- `compare_base_commit`: `a696f8e17dde4b9be95a7e99a1cf2517237cd871`
- `verified_scope`: schema 004 + seed 003 + Rust repository/model expansion + audit/doc artifacts

Hard rule:
- do **not** silently verify a newer `HEAD`
- do **not** verify your local working tree state
- do **not** substitute a partial checkout or scp snapshot

## Remote Login

Use the sponsor-provided workspace:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

## Read Scope

Read only:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-operational-review-continuity-supplement-v1.md`
4. `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`
5. this packet

Read in code only what is needed:

- `infra/postgres/schema/004_operational_review_and_continuity.sql`
- `infra/postgres/seed/003_operational_review_and_continuity_seed.sql`
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/src/objects/review.rs`
- `crates/seatloom-core/src/objects/checkpoint.rs`
- `crates/seatloom-core/src/objects/pipeline.rs`
- `crates/seatloom-core/src/objects/receipt.rs`
- `crates/seatloom-core/src/db/models.rs`
- `crates/seatloom-core/src/db/repositories.rs`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `.seatloom/bootstrap/source-map.yaml`

Do not widen into `ui/`, archived docs, or unrelated Rust modules.

## Write Boundary

Allowed local write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-verification-delivery-v1.md`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-verification/**`

Remote workspace writes are allowed only inside:

- `/data/seatloom-verify/`
- `/data/seatloom-verify/repo/`
- `/data/seatloom-verify/scratch/`

No product-code edits in this packet.

If you hit a low-level defect and believe a one-line/one-file infra patch is obviously required, stop first and report `HOLD` with:
- failing command
- exact error
- probable root cause
- smallest safe owner/fix scope

Only if Lyra later explicitly re-scopes you may patch, and then you must:
- commit the fix on `track/infra-foundation`
- cite the new commit hash
- write a second delivery artifact

## Execution Steps

### 1. Local evidence directory

Create and use:

```bash
mkdir -p .local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-verification/
```

### 2. SSH connectivity probe

Run and save exact output:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com 'uname -a && hostname'
```

If batch auth fails, retry interactively and record the blocker if login still fails.

### 3. Refresh exact remote commit

Operate only inside `/data/seatloom-verify/`.

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

Hard rules:
- do not inspect or reuse any non-SeatLoom project on the workspace
- if Git auth or network fails, stop and return `HOLD`
- do not switch to `scp` snapshot mode

### 4. Capture remote environment

On the remote workspace, capture these exactly:

```bash
uname -a
hostname
pwd
which docker || true
docker --version || true
docker compose version || true
which psql || true
psql --version || true
which cargo || true
cargo --version || true
```

### 5. Commit identity gate

Inside `/data/seatloom-verify/repo`, capture and save:

```bash
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
git log --oneline -n 5
```

Hard rule:
- if `git rev-parse HEAD` is not exactly `ee0159fa440b910de38b29824574760722a18316`, stop and return `HOLD`

### 6. Rust sanity checks

Run:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo check || cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core || cargo test -p seatloom-core
```

### 7. End-to-end PostgreSQL bootstrap path

Primary command:

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh
```

This is the primary proof point. Do not replace it with ad hoc SQL unless the script fails.

### 8. If the primary script fails, isolate the exact failing step

Run these and capture each separately:

```bash
cd /data/seatloom-verify/repo/infra/postgres && docker compose down -v || true
cd /data/seatloom-verify/repo/infra/postgres && docker compose up -d --wait
docker ps --filter name=seatloom-postgres

docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM checkpoints;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoff_receipts;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM pipeline_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_threads;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_comments;"

$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored || cargo test -p seatloom-core -- --include-ignored
```

### 9. Coverage and truth spot-checks

Even if the primary script passes, run these deterministic checks and capture outputs:

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

Then answer these explicitly:
- Are all 5 new canonical families present and queryable?
- Does the seeded baseline include at least 3 checkpoints, 3 handoff receipts, 1 pipeline run, 1 review thread, and 2 review comments?
- Does `cargo test -p seatloom-core -- --include-ignored` pass against this exact commit?
- Does the packet remain infrastructure-only with no UI/product widening?
- Is there any hidden operational flaw in schema 004, seed 003, or the verification script that should block acceptance?

## Required Evidence Output

Write full outputs under:

- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-verification/`

Required evidence files:
- `ssh-probe.txt`
- `git-target.txt`
- `remote-env.txt`
- `cargo-check.txt`
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
- `cargo-test-include-ignored.txt` (if fallback path used or rerun needed)

## Verification Questions You Must Answer

Answer all of these explicitly in the delivery artifact:

1. Was the exact pinned commit `ee0159fa440b910de38b29824574760722a18316` verified, not a later workspace state?
2. Did `bash scripts/verify-postgres-baseline.sh` succeed end to end on a docker-capable seat?
3. Are schema 004 and seed 003 compatible with the existing baseline without manual repair?
4. Do the new DB integration checks for checkpoints / receipts / pipeline runs / review threads pass when included?
5. Do the seeded counts and spot-check rows materially match Lyra's supplement claims?
6. Is the slice still infrastructure-only?
7. Is there any blocker that should prevent Lyra from accepting this packet?

## Done Definition

- [ ] Remote SSH connectivity proven or clearly blocked.
- [ ] Exact commit checkout proven.
- [ ] `cargo check` result captured.
- [ ] `cargo test -p seatloom-core` result captured.
- [ ] `bash scripts/verify-postgres-baseline.sh` executed and captured.
- [ ] Fallback isolation commands executed if the primary script fails.
- [ ] SQL spot-checks executed and captured.
- [ ] Delivery artifact written at `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-verification-delivery-v1.md`.
- [ ] Delivery artifact issues a clear verdict: `PASS`, `HOLD`, or `RE-SCOPED`.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-verification-delivery-v1.md`

Required sections:
1. Verdict (`PASS`, `HOLD`, or `RE-SCOPED`)
2. Execution environment
3. Command results matrix
4. SQL spot-check results
5. Findings / blockers
6. Evidence file paths
7. Recommended next action

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_pg_review_continuity_verification.txt
[Flux -> Lyra] PostgreSQL Review + Continuity Verification
completed:
- ...
validation:
- `git rev-parse HEAD` => `ee0159fa440b910de38b29824574760722a18316`
- `$HOME/.cargo/bin/cargo check` => ...
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
- docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-verification-delivery-v1.md
MSG

tmux load-buffer -b flux_to_lyra_pg_review_continuity_verification /tmp/flux_to_lyra_pg_review_continuity_verification.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_pg_review_continuity_verification
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
