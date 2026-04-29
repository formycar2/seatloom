# Task: Remote Workspace PostgreSQL Authority + Typed Documents Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-remote-workspace-postgres-authority-documents-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-authority-and-typed-documents-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-authority-and-typed-documents-delivery-v1.md`, `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md` |
| tags | flux, verification, postgres, typed-documents, remote-workspace, commit-pinned |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Execute this packet alone. Verification only. No product code patches. If you discover an implementation flaw, report it by exact commit and failing command rather than widening scope. |

## Objective

Verify Nimbus's PostgreSQL authority + typed documents packet on a docker-capable remote workspace, pinned to the exact published commit.

This packet proves three things:

1. the schema 002 document authority layer boots correctly on PostgreSQL,
2. the typed document seed and ingest path work against real repo markdown,
3. the ignored DB integration tests for documents pass on the exact commit Nimbus reported.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `a696f8e17dde4b9be95a7e99a1cf2517237cd871`
- `compare_base_commit`: `a658086b54323259fda2ad2a958d097701f1fbbd`
- `code_payload_commit`: `85c8a04d3c0cb66a972363ad4d23e648cb68a329`

Interpretation note:

- `85c8a04...` is the code payload for PostgreSQL authority + typed documents.
- `a696f8e...` adds the delivery artifact closeout and is the exact verification target.
- Do **not** silently substitute any other `HEAD`.

## Remote Login

Use the sponsor-provided workspace:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

## Read Scope

Read only:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/COORDINATION_RULES.md`
3. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-authority-and-typed-documents-v1.md`
4. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-authority-and-typed-documents-delivery-v1.md`
5. `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`
6. this packet

Read in code only what is needed:

- `infra/postgres/schema/002_document_authority.sql`
- `infra/postgres/seed/002_document_seed.sql`
- `scripts/ingest-documents.sh`
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/src/db/**`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `docs/architecture-design.md`
- `docs/architecture-decisions.md`

## Write Boundary

Allowed local write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-authority-documents-verification-delivery-v1.md`
- `.local/evidence/2026-04-29-remote-workspace-postgres-authority-documents-verification/**`

Remote workspace writes are allowed only inside:

- `/data/seatloom-verify/`
- `/data/seatloom-verify/repo/`
- `/data/seatloom-verify/scratch/`

Do not edit local product code in this packet.

## Execution Steps

### 1. Local evidence directory

Create and use:

```bash
mkdir -p .local/evidence/2026-04-29-remote-workspace-postgres-authority-documents-verification/
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
  git checkout a696f8e17dde4b9be95a7e99a1cf2517237cd871 && \
  git rev-parse --abbrev-ref HEAD && \
  git rev-parse HEAD && \
  git status --short \
'
```

Hard rules:

- do not inspect or reuse any non-SeatLoom project on the workspace;
- if Git auth or network fails, stop and return `HOLD`;
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

- if `git rev-parse HEAD` is not exactly `a696f8e17dde4b9be95a7e99a1cf2517237cd871`, stop and return `HOLD`.

### 6. Rust sanity checks

Run:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo check -p seatloom-core || cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core --lib || cargo test -p seatloom-core --lib
```

### 7. Reset and start PostgreSQL cleanly

Run:

```bash
cd /data/seatloom-verify/repo/infra/postgres
docker compose down -v || true
docker compose up -d --wait
```

### 8. Apply schema 001 + schema 002 and seeds 001 + 002

Run in this order:

```bash
cd /data/seatloom-verify/repo
docker exec seatloom-postgres psql -U seatloom -d seatloom -f /docker-entrypoint-initdb.d/001_seatloom_core.sql

docker cp infra/postgres/schema/002_document_authority.sql seatloom-postgres:/tmp/002_document_authority.sql
docker exec seatloom-postgres psql -U seatloom -d seatloom -f /tmp/002_document_authority.sql

docker cp infra/postgres/seed/001_real_collaboration_baseline.sql seatloom-postgres:/tmp/001_real_collaboration_baseline.sql
docker exec seatloom-postgres psql -U seatloom -d seatloom -f /tmp/001_real_collaboration_baseline.sql

docker cp infra/postgres/seed/002_document_seed.sql seatloom-postgres:/tmp/002_document_seed.sql
docker exec seatloom-postgres psql -U seatloom -d seatloom -f /tmp/002_document_seed.sql
```

### 9. Run the real markdown ingest path

Run:

```bash
cd /data/seatloom-verify/repo
bash scripts/ingest-documents.sh
```

### 10. Run ignored DB integration tests

Run:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored || cargo test -p seatloom-core -- --include-ignored
```

### 11. Coverage and truth spot-checks

Run deterministic SQL checks and capture outputs:

```bash
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM document_sections;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE body_text IS NOT NULL;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT template, COUNT(*) FROM documents GROUP BY template ORDER BY template;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT id, template, subtype, revision FROM documents ORDER BY id LIMIT 20;"
```

Then confirm these verification points explicitly:

- at least 11 typed documents are present;
- T1 through T7 are all represented;
- active contract docs are present as document rows;
- ingest populated `body_text` for the seeded repo docs;
- the packet remains infrastructure-only and does not widen UI or Tauri behavior.

## Required Evidence Output

Write full outputs under:

- `.local/evidence/2026-04-29-remote-workspace-postgres-authority-documents-verification/`

Required evidence files:

- `ssh-probe.txt`
- `git-target.txt`
- `remote-env.txt`
- `cargo-check-seatloom-core.txt`
- `cargo-test-lib.txt`
- `docker-compose-reset-up.txt`
- `apply-schema-001.txt`
- `apply-schema-002.txt`
- `apply-seed-001.txt`
- `apply-seed-002.txt`
- `ingest-documents.txt`
- `cargo-test-include-ignored.txt`
- `sql-document-count.txt`
- `sql-section-count.txt`
- `sql-body-count.txt`
- `sql-template-coverage.txt`
- `sql-document-sample.txt`

## Verification Questions You Must Answer

Answer all of these explicitly in the delivery artifact:

1. Does schema 002 apply cleanly on PostgreSQL 16 at the exact target commit?
2. Does the typed document seed load without manual repair?
3. Does `bash scripts/ingest-documents.sh` succeed against the seeded repo docs?
4. Do the ignored document-related DB integration tests pass when included?
5. Is the exact commit identity proven end to end?
6. Does the packet remain infrastructure-only?
7. Is there any remaining blocker that should prevent Lyra from accepting this packet?

## Done Definition

- [ ] Exact commit fetched and checked out.
- [ ] Remote environment captured.
- [ ] Rust sanity checks captured.
- [ ] PostgreSQL reset / boot captured.
- [ ] Schema 001 + 002 applied and captured.
- [ ] Seed 001 + 002 applied and captured.
- [ ] `bash scripts/ingest-documents.sh` executed and captured.
- [ ] `cargo test -p seatloom-core -- --include-ignored` executed and captured.
- [ ] SQL spot-checks captured.
- [ ] Delivery artifact written at `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-authority-documents-verification-delivery-v1.md`.
- [ ] Delivery verdict is explicit: `PASS`, `HOLD`, or `RE-SCOPED`.

## Pass / Hold / Re-Scoped Rules

### PASS

Use `PASS` only if all of the following are true:

- exact commit identity is proven;
- schemas 001 + 002 apply without manual repair;
- seeds 001 + 002 apply without manual repair;
- `bash scripts/ingest-documents.sh` succeeds;
- ignored DB tests pass;
- SQL spot-checks materially match Nimbus's delivery claims;
- no blocking infra flaw is found.

### HOLD

Use `HOLD` if the implementation looks correct but the remote environment cannot satisfy the required path (for example Docker, Git auth, or workspace availability issues). Separate environment blockers from implementation findings.

### RE-SCOPED

Use `RE-SCOPED` if a real implementation flaw is found. List:

- exact failing command
- exact error
- probable root cause
- recommended owner
- bounded fix scope

## Non-Goals

- No UI work.
- No Tauri surface widening.
- No schema redesign beyond verification evidence.
- No background sync design work.
- No steady-state reconcile implementation here; that belongs to Nimbus's next packet.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-authority-documents-verification-delivery-v1.md`
