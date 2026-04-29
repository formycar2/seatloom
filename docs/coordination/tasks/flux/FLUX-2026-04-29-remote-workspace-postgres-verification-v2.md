# Task: Remote Workspace PostgreSQL Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-remote-workspace-postgres-verification-v2 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v2 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-commit-pinned-infra-baseline-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-commit-pinned-infra-baseline-delivery-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-real-collaboration-postgres-baseline-acceptance.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | flux, verification, postgres, remote-workspace, ssh, docker, native-postgres, commit-pinned |
| owner | Flux |
| acceptance owner | Lyra |
| supersedes | `FLUX-2026-04-29-remote-workspace-postgres-verification-v1` |
| concurrency rule | Execute this packet alone. Verification only. No code changes unless Lyra explicitly reissues a bounded `T3/fix` packet. |

## Objective

Use the sponsor-provided remote workspace as a new verification environment for the PostgreSQL baseline, and close or re-confirm the current `HOLD` using an exact Git branch and exact Git commit.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `d007721bd36e9fdd0e145e68a04e32c1bc4d9cf2`
- `compare_base_commit`: `1bf60be9c946e498db031e8f08928051339c1d29`
- `code_payload_commit`: `2f6c41a4d4c5acae2f7e8f103610e7d45482d15e`

Interpretation note:

- `d007721...` is the current published branch head and is the verification target.
- `2f6c41a...` is the preceding infra payload commit called out by Nimbus; `d007721...` adds the delivery-artifact closeout on top of that payload.
- Do **not** silently substitute any other `HEAD`.

## Remote Login

Use the sponsor-provided workspace:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

## Verification Questions

You must answer these five questions in the delivery artifact:

1. Does SSH access to the sponsor-provided remote workspace succeed?
2. Can the exact target branch and exact target commit be fetched and checked out from GitHub?
3. Does the remote workspace have Docker, and if yes, does `bash scripts/verify-postgres-baseline.sh` pass end to end?
4. If Docker is absent, does native PostgreSQL exist and can it validate the same schema/seed/test path?
5. Do the live PostgreSQL integration tests pass against the exact target commit?

## Read Scope

Read only:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/COORDINATION_RULES.md`
3. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
4. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`
5. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-commit-pinned-infra-baseline-v1.md`
6. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-commit-pinned-infra-baseline-delivery-v1.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-nimbus-real-collaboration-postgres-baseline-acceptance.md`
8. this packet

Read in code only what is needed:

- `infra/postgres/**`
- `scripts/verify-postgres-baseline.sh`
- `scripts/verify-rust-foundation.sh`
- `crates/seatloom-core/src/db/**`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- directly referenced Cargo manifests

## Write Boundary

Allowed local write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`
- `.local/evidence/2026-04-29-remote-workspace-postgres-verification/**`

Remote workspace writes are allowed only inside:

- `/data/seatloom-verify/`
- `/data/seatloom-verify/repo/`
- `/data/seatloom-verify/scratch/`

Do not edit the local product repo code in this packet.

## Execution Steps

### 1. Local evidence directory

Create and use:

```bash
mkdir -p .local/evidence/2026-04-29-remote-workspace-postgres-verification/
```

### 2. SSH connectivity probe

Run and save exact output:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com 'uname -a && hostname'
```

If batch auth fails, retry interactively and record the blocker if login still fails.

### 3. Prepare remote workspace with Git first

Operate only inside `/data/seatloom-verify/`.

Preferred flow:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com '\
  mkdir -p /data/seatloom-verify /data/seatloom-verify/scratch && \
  if [ ! -d /data/seatloom-verify/repo/.git ]; then \
    git clone git@github.com:formycar2/seatloom.git /data/seatloom-verify/repo; \
  fi && \
  cd /data/seatloom-verify/repo && \
  git fetch origin track/infra-foundation && \
  git checkout d007721bd36e9fdd0e145e68a04e32c1bc4d9cf2 && \
  git rev-parse --abbrev-ref HEAD && \
  git rev-parse HEAD && \
  git status --short\
'
```

Rules:

- do not inspect or reuse any non-SeatLoom project on the workspace;
- if Git auth or network fails, stop and report `HOLD`;
- do not switch to `scp` snapshot mode unless Lyra explicitly reissues this as provisional verification.

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
git log --oneline -n 3
```

Hard rule:

- if `git rev-parse HEAD` is not exactly `d007721bd36e9fdd0e145e68a04e32c1bc4d9cf2`, stop and return `HOLD`.

### 6. Remote Rust sanity check

Run:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo check || cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core || cargo test -p seatloom-core
bash scripts/verify-rust-foundation.sh
```

### 7. PostgreSQL verification path selection

#### Path A — Docker exists

Run:

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh
```

#### Path B — Docker absent, native PostgreSQL available

Run this bounded equivalent flow:

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

If any live PostgreSQL path succeeds, confirm these truths with SQL or test-backed evidence:

- the five seats exist: `aegis`, `lyra`, `mira`, `nimbus`, `flux`
- at least one delegation exists for Flux acting for Mira
- artifacts exist for every family `T1` through `T7`
- active contract artifacts exist in the seeded data

## Required Evidence Files

Write outputs under:

- `.local/evidence/2026-04-29-remote-workspace-postgres-verification/`

Required file set:

- `ssh-probe.txt`
- `remote-env.txt`
- `remote-git-identity.txt`
- `remote-cargo-check.txt`
- `remote-cargo-test-unit.txt`
- `remote-verify-rust-foundation.txt`
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

Do not patch product code in this packet.
