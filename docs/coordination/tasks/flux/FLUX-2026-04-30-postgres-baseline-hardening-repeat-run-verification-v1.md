# Task: Flux PostgreSQL Baseline Hardening Repeat-Run Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md`, `docs/coordination/reviews/2026-04-30-lyra-postgres-runtime-authority-gap-review.md`, `docs/infra/ssh-tunnel-workspace.md` |
| tags | flux, verification, postgres, docker, remote-workspace, commit-pinned, repeat-run |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No code edits. No parallel subtasks. Verify only the bounded infra surface and stop on the first real blocker. |

## Objective

Prove or disprove that Nimbus's PostgreSQL baseline verification hardening actually closes the repeat-run verifier gap.

This is not a general repo check. This is a commit-pinned remote verification of one bounded infrastructure claim:

`bash scripts/verify-postgres-baseline.sh` should pass twice in a row on the same Docker-capable seat without any manual cleanup between runs.

## Why This Packet Exists

Earlier remote verification proved the PostgreSQL baseline itself was functionally correct, but it also exposed an operator-gap:
- stale Docker volume state could break repeat runs,
- container health could report early,
- the proof path needed manual cleanup or extra operator judgment.

Nimbus responded with a bounded hardening commit that:
- performs destructive volume cleanup inside the verifier,
- adds explicit `pg_isready` polling,
- preserves the accepted proof sequence,
- changes no schema, seed, Rust product behavior, or UI.

Your job is to verify that exact claim against the exact commit.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `24c820b81de8f8a479e9d229f5c106ed440eb675`
- `compare_base_commit`: `ea79461ddabc9d2ce5830f1a0de05cfdd7354682`

Hard rules:
- do not substitute a later `HEAD`,
- do not verify local working-tree state,
- do not patch code even if you find a defect,
- do not run repo-root `cargo check`,
- do not widen into frontend or GTK/Tauri runtime checks.

## Remote Login

Use the sponsor-provided workspace exactly as documented:

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com
```

## Read Scope

Read only:
1. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-baseline-verification-hardening-delivery-v1.md`
2. `docs/coordination/reviews/2026-04-30-lyra-postgres-runtime-authority-gap-review.md`
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
- `docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-delivery-v1.md`
- `.local/evidence/2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1/**`

No code edits.
No doc edits outside the delivery artifact.
If you discover a real implementation defect, return `HOLD` with the exact failing command, exact failing step, and the narrowest real blocker only.

## Execution Steps

### 1. Local evidence directory

```bash
mkdir -p .local/evidence/2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1/
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
  git checkout 24c820b81de8f8a479e9d229f5c106ed440eb675 && \
  git rev-parse --abbrev-ref HEAD && \
  git rev-parse HEAD && \
  git status --short \
'
```

### 4. Capture bounded remote environment

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

Stop and return `HOLD` if either condition is true:
- `git rev-parse HEAD` is not exactly `24c820b81de8f8a479e9d229f5c106ed440eb675`
- `git status --short` is non-empty before verification starts

### 6. Narrow Rust sanity checks

Run exactly these commands, in this order:

```bash
cd /data/seatloom-verify/repo
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
```

Do not run repo-root `cargo check`.

### 7. Primary proof point: first verifier run

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh | tee .local/evidence/2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1/run-1.txt
```

### 8. Primary proof point: second verifier run immediately after the first

Do not manually run `docker compose down -v` or any cleanup between runs.

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh | tee .local/evidence/2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1/run-2.txt
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
- both verifier runs exit `0`,
- both verifier runs end with `=== All checks passed ===`,
- `documents WHERE template IS NULL` returns `0`,
- `reconcile_runs` is non-zero,
- `document_versions` is non-zero,
- `documents WHERE body_text IS NOT NULL` is non-zero,
- `checkpoints`, `handoff_receipts`, `pipeline_runs`, `review_threads`, and `review_comments` are all non-zero.

### 10. If any verifier run fails, isolate the narrowest real blocker

Use this fallback sequence and capture all output:

```bash
cd /data/seatloom-verify/repo/infra/postgres && docker compose ps
cd /data/seatloom-verify/repo/infra/postgres && docker compose logs --tail 120

docker exec seatloom-postgres pg_isready -U seatloom -d seatloom || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM reconcile_runs;" || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM document_versions;" || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE template IS NULL;" || true
```

Do not patch. Return `HOLD`.

## Delivery Format

Return exactly this structure in the delivery markdown and tmux summary:

```text
[Flux -> Lyra] PostgreSQL Baseline Hardening Repeat-Run Verification
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
- docs/coordination/tasks/flux/FLUX-2026-04-30-postgres-baseline-hardening-repeat-run-verification-delivery-v1.md
- .local/evidence/2026-04-30-postgres-baseline-hardening-repeat-run-verification-v1/
```
