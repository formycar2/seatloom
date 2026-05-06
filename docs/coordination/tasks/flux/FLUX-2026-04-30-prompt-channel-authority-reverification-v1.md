# Task: Flux Prompt + Channel Action Authority Re-Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-30-prompt-channel-authority-reverification-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-06 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-05-06 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-flux-prompt-channel-authority-remote-verification-v1.md`, `docs/infra/ssh-tunnel-workspace.md` |
| tags | flux, verification, postgres, prompt, channel-action, remote, commit-pinned, repeat-run, schema005, test-isolation |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No code edits. No parallel subtasks. Verify only the bounded infra surface and stop on the first real blocker. |

## Objective

Prove or disprove that Nimbus's schema-005 test isolation fix closes the HOLD you issued on commit `68a7b38`.

This is a narrow, commit-pinned remote verification packet. Do not widen into UI, Tauri, or general repo review.

## What Changed Since the HOLD

Earlier HOLD, issued by you on commit `68a7b38ed13c4c5dc3d68aeecd1d970030b66451`:

Flux's HOLD identified that the schema-005 integration tests were not safe under the accepted verifier contract (`cargo test -p seatloom-core -- --include-ignored`):

- **Defect A** — `prompt_instance_tables_exist_and_empty_at_baseline()` asserted global zero-row state; that state is only valid before any mutating schema-005 test inserts fixture rows.
- **Defect B** — `prompt_action_append_and_list()` referenced `pi-test-001`, a row created by another test. Cross-test side-effect dependency.
- **Defect C** — tests were neither self-contained nor split into isolated phases, making the proof path order-dependent under `--include-ignored`.

Nimbus now claims this is fixed by commit `1bb561ab7872ed69b5dd7d40335ea8c689a5563a`:

- `prompt_instance_tables_exist_and_empty_at_baseline()` removed from Rust tests; zero-row baseline is now proven by a SQL `DO $$ ... $$` block in verifier Step 4 (runs before any mutating test).
- `prompt_action_append_and_list()` now creates its own parent `PromptInstanceRow` with unique fixture ID `pi-action-fixture-001`, independent of any sibling test.
- All schema-005 tests now use unique fixture IDs (`pi-test-001`, `pi-action-fixture-001`, `car-test-001`, `car-test-002`) and are order-independent.
- Verifier step numbers updated: old Step 4 → new Step 5, Step 5 → Step 6, Step 6 → Step 7; new Step 4 is the SQL zero-row baseline proof.

Your job is to verify that exact claim against the exact commit.

## Target Identity (Mandatory)

Verify exactly this Git target:

- `target_remote`: `git@github.com:formycar2/seatloom.git`
- `target_branch`: `track/infra-foundation`
- `target_commit`: `1bb561ab7872ed69b5dd7d40335ea8c689a5563a`
- `compare_base_commit`: `68a7b38ed13c4c5dc3d68aeecd1d970030b66451`

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
1. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md`
2. `docs/coordination/tasks/lyra/LYRA-2026-04-30-flux-prompt-channel-authority-remote-verification-v1.md`
3. `docs/infra/ssh-tunnel-workspace.md`
4. this packet

Inspect code/files only as needed:
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `infra/postgres/schema/005_prompt_and_channel_action_authority.sql`
- `infra/postgres/seed/004_prompt_and_channel_action_seed.sql`
- `infra/postgres/docker-compose.yml`

## Write Boundary

Allowed local write locations only:
- `docs/coordination/tasks/flux/FLUX-2026-04-30-prompt-channel-authority-reverification-delivery-v1.md`
- `.local/evidence/2026-04-30-prompt-channel-authority-reverification-v1/**`

No code edits.
No doc edits outside the delivery artifact.
If you discover a real implementation defect, return `HOLD` with the exact failing command, exact failing step, and the narrowest real blocker only.

## Execution Steps

### 1. Local evidence directory

```bash
mkdir -p .local/evidence/2026-04-30-prompt-channel-authority-reverification-v1/
```

### 2. SSH connectivity probe

```bash
ssh -o BatchMode=yes -o ConnectTimeout=10 -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com 'uname -a && hostname'
```

### 3. Refresh exact remote commit in a clean verification repo

Use a fresh path to avoid dirty-state confusion from earlier verification runs.

```bash
ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com '\
  rm -rf /data/seatloom-verify-schema005-isolation && \
  mkdir -p /data/seatloom-verify-schema005-isolation && \
  git clone git@github.com:formycar2/seatloom.git /data/seatloom-verify-schema005-isolation/repo && \
  cd /data/seatloom-verify-schema005-isolation/repo && \
  git fetch origin track/infra-foundation && \
  git checkout 1bb561ab7872ed69b5dd7d40335ea8c689a5563a && \
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
cd /data/seatloom-verify-schema005-isolation/repo
git rev-parse HEAD
git status --short
git log --oneline -n 5
```

Stop and return `HOLD` if either condition is true:
- `git rev-parse HEAD` is not exactly `1bb561ab7872ed69b5dd7d40335ea8c689a5563a`
- `git status --short` is non-empty before verification starts

### 6. Narrow Rust sanity checks

Run exactly these commands, in this order:

```bash
cd /data/seatloom-verify-schema005-isolation/repo
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
```

Expected:
- `cargo check -p seatloom-core` => PASS
- `cargo test -p seatloom-core` => 54 unit tests pass, 3 seed-consistency tests pass, 27 ignored (not 28 — the global zero-row test was removed)

Do not run repo-root `cargo check`.

### 7. Primary proof point: first verifier run

```bash
cd /data/seatloom-verify-schema005-isolation/repo
bash scripts/verify-postgres-baseline.sh | tee .local/evidence/2026-04-30-prompt-channel-authority-reverification-v1/run-1.txt
```

Confirm that:
- Step 4 (`Schema-005 zero-row baseline proof`) appears in the output and passes
- Step 7 (`cargo test -p seatloom-core -- --include-ignored`) passes — this is the critical new proof point

### 8. Primary proof point: second verifier run immediately after the first

Do not manually run `docker compose down -v`, `docker compose up`, or any cleanup between runs.

```bash
cd /data/seatloom-verify-schema005-isolation/repo
bash scripts/verify-postgres-baseline.sh | tee .local/evidence/2026-04-30-prompt-channel-authority-reverification-v1/run-2.txt
```

### 9. SQL spot checks after the second passing run

Run and capture these exact commands:

```bash
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT to_regclass('public.prompt_instances');"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT to_regclass('public.prompt_actions');"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT to_regclass('public.channel_action_receipts');"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT indexname, indexdef FROM pg_indexes WHERE tablename = 'channel_action_receipts' ORDER BY indexname;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM reconcile_runs;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM document_versions;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE body_text IS NOT NULL;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE template IS NULL;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM checkpoints;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoff_receipts;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_threads;"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM review_comments;"
```

Expected minimum acceptance signals:
- both verifier runs exit `0`
- both verifier runs end with `=== All checks passed ===`
- both verifier runs include Step 4 `schema-005 zero-row baseline confirmed`
- `to_regclass(...)` returns non-null for all three schema-005 families
- a unique index or unique constraint exists on `channel_action_receipts.idempotency_key`
- `documents WHERE template IS NULL` returns `0`
- `reconcile_runs` is non-zero
- `document_versions` is non-zero
- `documents WHERE body_text IS NOT NULL` is non-zero
- `checkpoints`, `handoff_receipts`, `review_threads`, and `review_comments` are all non-zero

### 10. If any verifier run fails, isolate the narrowest real blocker

Use this fallback sequence and capture all output:

```bash
cd /data/seatloom-verify-schema005-isolation/repo/infra/postgres && docker compose ps
cd /data/seatloom-verify-schema005-isolation/repo/infra/postgres && docker compose logs --tail 120

docker exec seatloom-postgres pg_isready -U seatloom -d seatloom || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM reconcile_runs;" || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM document_versions;" || true
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM documents WHERE template IS NULL;" || true
```

Do not patch. Return `HOLD`.

## Delivery Format

Return exactly this structure in the delivery markdown and tmux summary:

```text
[Flux -> Lyra] Prompt + Channel Action Authority Re-Verification
completed:
- ...
validation:
- `git rev-parse HEAD` => ...
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `bash scripts/verify-postgres-baseline.sh` run 1 => ...
- `bash scripts/verify-postgres-baseline.sh` run 2 => ...
- Step 4 zero-row baseline proof => ...
- SQL spot-checks => ...
blockers:
- none / ...
verdict:
- PASS | HOLD
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-prompt-channel-authority-reverification-delivery-v1.md
- .local/evidence/2026-04-30-prompt-channel-authority-reverification-v1/
```
