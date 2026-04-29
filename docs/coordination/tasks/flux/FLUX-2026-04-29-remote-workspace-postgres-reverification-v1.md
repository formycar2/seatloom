# Task: Remote Workspace PostgreSQL Re-Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-remote-workspace-postgres-reverification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-v2.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-delivery-v1.md` |
| tags | flux, verification, postgres, remote-workspace, ssh, docker, commit-pinned, reverification |
| owner | Flux |
| acceptance owner | Lyra |
| supersedes | `FLUX-2026-04-29-remote-workspace-postgres-verification-v2` |
| concurrency rule | Execute this packet alone. Verification only. No code changes unless Lyra explicitly reissues a bounded `T3/fix` packet. |

## Objective

Re-run the sponsor-workspace PostgreSQL verification after Nimbus fixed the PostgreSQL 16 seed blocker.

This is a strict commit-pinned re-verification. Use the exact updated branch head commit, not a floating branch tip.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `a658086b54323259fda2ad2a958d097701f1fbbd`
- `compare_base_commit`: `d007721bd36e9fdd0e145e68a04e32c1bc4d9cf2`
- `code_payload_commit`: `de3aefc33f1905aea27b0078be6bb0e2324604fd`

Interpretation note:

- `a658086...` is the current published branch head and the verification target.
- `de3aefc...` is the actual code fix commit (`ARRAY[]` -> `ARRAY[]::text[]`).
- `a658086...` adds the delivery-artifact closeout on top of that payload.
- Do **not** silently substitute any other `HEAD`.

## Verified Prior Blocker

From the previous remote verification delivery:

- exact prior verified `HEAD`: `d007721bd36e9fdd0e145e68a04e32c1bc4d9cf2`
- Docker path was available and executed successfully
- blocker: `infra/postgres/seed/001_real_collaboration_baseline.sql` line 33 used untyped `ARRAY[]`, which PostgreSQL 16 rejected

Nimbus fix delivery now states:

- payload commit: `de3aefc33f1905aea27b0078be6bb0e2324604fd`
- branch head containing that payload: `a658086b54323259fda2ad2a958d097701f1fbbd`
- zero other untyped `ARRAY[]` remain in the seed file

## Remote Login

Use the sponsor-provided workspace:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

## Read Scope

Read only:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/COORDINATION_RULES.md`
3. `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-v2.md`
4. `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`
5. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-v1.md`
6. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-delivery-v1.md`
7. this packet

Read in code only what is needed:

- `infra/postgres/**`
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- directly referenced Cargo manifests

## Write Boundary

Allowed local write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-reverification-delivery-v1.md`
- `.local/evidence/2026-04-29-remote-workspace-postgres-reverification/**`

Remote workspace writes are allowed only inside:

- `/data/seatloom-verify/`
- `/data/seatloom-verify/repo/`
- `/data/seatloom-verify/scratch/`

Do not edit the local product repo code in this packet.

## Execution Steps

### 1. Local evidence directory

Create and use:

```bash
mkdir -p .local/evidence/2026-04-29-remote-workspace-postgres-reverification/
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
  git checkout a658086b54323259fda2ad2a958d097701f1fbbd && \
  git rev-parse --abbrev-ref HEAD && \
  git rev-parse HEAD && \
  git status --short \
'
```

Rules:

- do not inspect or reuse any non-SeatLoom project on the workspace;
- if Git auth or network fails, stop and report `HOLD`;
- do not switch to `scp` snapshot mode.

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
which pg_isready || true
pg_isready || true
which cargo || true
cargo --version || true
```

### 5. Commit identity gate

Inside `/data/seatloom-verify/repo`, capture and save:

```bash
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
git log --oneline -n 4
```

Hard rule:

- if `git rev-parse HEAD` is not exactly `a658086b54323259fda2ad2a958d097701f1fbbd`, stop and return `HOLD`.

### 6. Core Rust sanity check

Run:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo check -p seatloom-core || cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core --lib || cargo test -p seatloom-core --lib
```

Do not fail the packet on known full-workspace GUI dependency gaps if `seatloom-core` passes.

### 7. PostgreSQL verification path

Because Docker existed in the prior run, start with Path A.

#### Path A — Docker

Run:

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh
```

If Path A is unavailable unexpectedly, document why and only then consider Path B.

#### Path B — native PostgreSQL fallback

Run only if Docker is unexpectedly unavailable but native PostgreSQL is usable:

```bash
cd /data/seatloom-verify/repo
createdb seatloom_verify || true
psql -d seatloom_verify -f infra/postgres/schema/001_seatloom_core.sql
psql -d seatloom_verify -f infra/postgres/seed/001_real_collaboration_baseline.sql
export DATABASE_URL=postgresql://localhost/seatloom_verify
$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored || cargo test -p seatloom-core -- --include-ignored
psql -d seatloom_verify -c "SELECT COUNT(*) FROM seats;"
psql -d seatloom_verify -c "SELECT COUNT(*) FROM artifacts;"
psql -d seatloom_verify -c "SELECT COUNT(*) FROM canonical_events;"
```

#### Path C — neither Docker nor usable native PostgreSQL exists

Stop and report the exact blocker. Do not improvise another bootstrap method.

### 8. Coverage spot-checks

If a live PostgreSQL path succeeds, confirm these truths with SQL or test-backed evidence:

- the five seats exist: `aegis`, `lyra`, `mira`, `nimbus`, `flux`
- at least one delegation exists for Flux acting for Mira
- artifacts exist for every family `T1` through `T7`
- active contract artifacts exist in the seeded data

## Required Evidence Files

Write outputs under:

- `.local/evidence/2026-04-29-remote-workspace-postgres-reverification/`

Required file set:

- `ssh-probe.txt`
- `remote-env.txt`
- `remote-git-identity.txt`
- `remote-cargo-check-core.txt`
- `remote-cargo-test-core-lib.txt`
- `remote-verify-postgres-baseline.txt` if Path A runs
- `remote-native-postgres-flow.txt` if Path B runs
- `remote-cargo-test-include-ignored.txt` if a DB path runs
- `remote-sql-seat-count.txt` if a DB path runs
- `remote-sql-artifact-count.txt` if a DB path runs
- `remote-sql-event-count.txt` if a DB path runs

## Delivery Contract

Your delivery artifact must include:

- exact `target_branch`
- exact `target_commit`
- exact verified `HEAD`
- whether the workspace was clean or dirty
- which verification path ran (`A`, `B`, or `C`)
- exact commands executed
- exact PASS / HOLD / FAIL verdict with blocker reason if not PASS
- whether the prior PG16 blocker is now closed
- whether any new blocker appeared

## Deliverable

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-reverification-delivery-v1.md`

Do not patch product code in this packet.
