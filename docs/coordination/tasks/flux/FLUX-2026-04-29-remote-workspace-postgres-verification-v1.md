# Task: Remote Workspace PostgreSQL Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-remote-workspace-postgres-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-real-collaboration-postgres-baseline-acceptance.md` |
| tags | flux, verification, postgres, remote-workspace, ssh, docker, native-postgres |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No code changes. No schema edits. No product-scope widening. |

## Objective

Use the sponsor-provided remote workspace as a new verification environment for the PostgreSQL baseline.

Remote login provided by sponsor:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

Why this packet exists:

- Nimbus implementation is structurally complete.
- Local seats confirmed Rust-only checks pass.
- The open blocker is environment-only: current local seats do not have Docker.
- Sponsor explicitly provided a remote workspace that may have Docker or a previously installed PostgreSQL service.

Your job is to verify the baseline there, without changing product code.


## Commit Pin Gate

This verification must be commit-pinned.

Before running any acceptance-closing verification, Lyra must provide Flux with:

- `target_branch`
- `target_commit`

Flux rules for this packet:

- do not verify a floating remote `HEAD` as if it were the intended delivery;
- if the required target commit is not available on the remote workspace via Git, stop and report `HOLD`;
- if Lyra later authorizes an `scp` snapshot fallback, label that result `PROVISIONAL` and do not treat it as acceptance-closing proof.


## Verification Strategy

Use this exact priority order.

### Path A — preferred

If the remote workspace has Docker available, verify using the intended repo-managed path:

```bash
bash scripts/verify-postgres-baseline.sh
```

### Path B — sponsor-authorized fallback

If Docker is unavailable on the remote workspace, but a usable native PostgreSQL service/client exists there, run a bounded equivalent verification:

1. confirm PostgreSQL service/client availability,
2. prepare a disposable verification database,
3. apply the existing schema,
4. apply the real seed,
5. run the DB integration tests with `--include-ignored`,
6. run a small SQL spot-check set.

Important:

- do not redesign the method;
- do not patch Nimbus code in this packet;
- do not alter schema/seed files;
- if neither Docker nor usable native PostgreSQL is available, stop and report exact blocker.

## Read Scope

Read only:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md`
3. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`
4. `docs/coordination/tasks/flux/FLUX-2026-04-29-postgres-baseline-verification-delivery-v1.md`
5. this packet

Read in code only what is needed:

- `infra/postgres/**`
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/src/db/**`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- directly referenced Cargo manifests

## Write Boundary

Allowed local write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`
- `.local/evidence/2026-04-29-remote-workspace-postgres-verification/**`

Remote workspace writes are allowed only inside this sponsor-approved disposable verification layout:

- `/data/seatloom-verify/`
- `/data/seatloom-verify/repo/`
- `/data/seatloom-verify/scratch/`

Do not edit the local product repo code in this packet.

## Execution Steps

### 1. Local evidence directory

Create and use:

```bash
.local/evidence/2026-04-29-remote-workspace-postgres-verification/
```

### 2. SSH connectivity probe

Try a bounded probe first:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com 'uname -a && hostname'
```

If key-based auth is unavailable, retry interactively and record the exact auth blocker if login fails.

### 3. Prepare remote verification workspace

After successful login:

- detect whether a recent SeatLoom repo copy already exists under `/data/seatloom-verify/repo/`;
- if not, populate the sponsor-approved `/data` area with a fresh SeatLoom-only repo copy at the exact target commit;
- do not touch any non-SeatLoom project already present on the workspace;
- avoid heavyweight local caches.

Use this remote directory layout:

```text
/data/seatloom-verify/
  repo/
  scratch/
```

Preferred checkout method (sponsor clarified the workspace pubkey is already configured in GitHub under `formaycar2`, so the remote workspace may be able to access the SeatLoom repo directly):

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com '\
  mkdir -p /data/seatloom-verify /data/seatloom-verify/scratch && \
  if [ ! -d /data/seatloom-verify/repo/.git ]; then \
    git clone <SEATLOOM_GIT_REMOTE> /data/seatloom-verify/repo; \
  else \
    cd /data/seatloom-verify/repo && git fetch --all --tags && git status --short --branch; \
  fi\
'
```

Important rules for the Git path:

- operate only inside `/data/seatloom-verify/repo/`;
- do not inspect, edit, or reuse any other project on the remote workspace;
- if you reuse an existing SeatLoom checkout, record why it is current enough and show that it contains the target commit;
- if the Git path fails because remote auth, network, or missing target commit prevents exact commit checkout, stop and report `HOLD` unless Lyra explicitly reissues this packet as provisional snapshot verification.

Fallback bounded transfer (sponsor also allowed `scp`):

```bash
mkdir -p /tmp/seatloom-transfer
tar \
  --exclude='.git' \
  --exclude='node_modules' \
  --exclude='target' \
  --exclude='.local/evidence' \
  -czf /tmp/seatloom-transfer/seatloom-verify-src.tgz .

scp /tmp/seatloom-transfer/seatloom-verify-src.tgz \
  buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com:/data/seatloom-verify/

ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com '\
  mkdir -p /data/seatloom-verify/repo /data/seatloom-verify/scratch && \
  tar -xzf /data/seatloom-verify/seatloom-verify-src.tgz -C /data/seatloom-verify/repo && \
  rm -f /data/seatloom-verify/seatloom-verify-src.tgz\
'
```

If a fresh repo copy already exists remotely and is clearly current, you may reuse it and record why.

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

### 5. Commit identity capture

Inside the remote repo copy, capture:

```bash
cd /data/seatloom-verify/repo
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
```

The reported `HEAD` must match the packet's target commit before you continue.

### 6. Remote Rust sanity check

Inside the remote repo copy:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo check || cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core || cargo test -p seatloom-core
```

### 7. Verification path selection

#### If Docker exists

Run:

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh
```

#### If Docker is absent but native PostgreSQL is available

Run a bounded equivalent flow. Preferred disposable DB name:

```bash
seatloom_verify
```

Suggested sequence:

```bash
createdb seatloom_verify || true
psql -d seatloom_verify -f infra/postgres/schema/001_seatloom_core.sql
psql -d seatloom_verify -f infra/postgres/seed/001_real_collaboration_baseline.sql
export DATABASE_URL=postgresql://localhost/seatloom_verify
$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored || cargo test -p seatloom-core -- --include-ignored
psql -d seatloom_verify -c "SELECT COUNT(*) FROM seats;"
psql -d seatloom_verify -c "SELECT COUNT(*) FROM artifacts;"
psql -d seatloom_verify -c "SELECT COUNT(*) FROM canonical_events;"
```

If the remote PostgreSQL auth model differs, adapt only enough to complete verification and document the exact commands used.

### 8. Coverage spot-checks

Confirm these truths in the live DB if any DB path succeeds:

- 5 seats exist: `aegis`, `lyra`, `mira`, `nimbus`, `flux`
- at least 1 delegation captures Flux acting for Mira
- active contract artifacts exist
- at least one artifact exists for each family `T1` through `T7`

Use SQL queries or test output; keep it bounded.

## Required Evidence Output

Write full outputs under:

- `.local/evidence/2026-04-29-remote-workspace-postgres-verification/`

Required evidence files:

- `ssh-probe.txt`
- `remote-env.txt`
- `remote-git-identity.txt`
- `remote-cargo-check.txt`
- `remote-cargo-test-unit.txt`
- `remote-verify-postgres-baseline.txt` if Path A runs
- `remote-native-postgres-flow.txt` if Path B runs
- `remote-sql-seat-count.txt` if a DB path runs
- `remote-sql-artifact-count.txt` if a DB path runs
- `remote-sql-event-count.txt` if a DB path runs
- `remote-cargo-test-include-ignored.txt` if a DB path runs

## Verification Questions You Must Answer

1. Does SSH access to the sponsor-provided remote workspace succeed?
2. Is the exact target branch + target commit available remotely and verified before testing starts?
3. Does that workspace have Docker? If yes, does the repo-managed script pass end to end?
4. If Docker is absent, does native PostgreSQL exist and can it validate the same schema/seed/test path?
5. Do DB integration tests pass against a live PostgreSQL instance?
5. Does seeded live data materially match Nimbus's claimed coverage?
6. Is there any real implementation flaw, or is the blocker purely environment/ops?

## Done Definition

- [ ] Remote workspace access attempted and recorded.
- [ ] Remote environment captured.
- [ ] Remote Rust sanity check captured.
- [ ] Path A or Path B attempted, with exact result captured.
- [ ] If a live DB path succeeds, integration tests and SQL spot-checks captured.
- [ ] Delivery artifact written at `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`.
- [ ] Delivery artifact issues a clear verdict: `PASS`, `HOLD`, or `RE-SCOPED`.

## Verdict Rules

### PASS

Use `PASS` if either of these is true:

- Path A succeeds end to end on the remote workspace; or
- Path B succeeds end to end on a live native PostgreSQL service and no implementation flaw is found.

### HOLD

Use `HOLD` if the implementation still appears correct but remote access or remote environment prerequisites are missing.

### RE-SCOPED

Use `RE-SCOPED` only if a real implementation defect is found. List exact command, exact error, probable root cause, owner, and bounded fix scope.

## Non-Goals

- No code changes.
- No new migrations.
- No schema redesign.
- No UI work.
- No reopening PostgreSQL as the storage authority.
- No product-scope expansion.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`

Required sections:

1. Verdict (`PASS`, `HOLD`, or `RE-SCOPED`)
2. Remote execution environment
3. Path chosen (`A` Docker script or `B` native PostgreSQL fallback)
4. Command results matrix
5. Live DB coverage spot-checks
6. Findings / blockers
7. Evidence file paths
8. Recommended next action

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_remote_workspace_postgres_verification.txt
[Flux -> Lyra] Remote Workspace PostgreSQL Verification
completed:
- ...
validation:
- `ssh ...` => ...
- `docker --version` => ...
- `psql --version` => ...
- `$HOME/.cargo/bin/cargo check` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash scripts/verify-postgres-baseline.sh` => ... / NOT RUN
- `native PostgreSQL fallback` => ... / NOT RUN
blockers:
- none / ...
verdict:
- PASS / HOLD / RE-SCOPED
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md
MSG

tmux load-buffer -b flux_to_lyra_remote_workspace_postgres_verification /tmp/flux_to_lyra_remote_workspace_postgres_verification.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_remote_workspace_postgres_verification
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
