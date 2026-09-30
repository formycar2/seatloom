# Task: Prompt + Channel Action Authority Remote Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-04-30-flux-prompt-channel-authority-remote-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md`, `docs/coordination/reviews/2026-04-30-lyra-collaboration-dataflow-backend-traceability-report.md`, `docs/infra/ssh-tunnel-workspace.md` |
| tags | flux, verification, postgres, prompt, channel-action, remote, commit-pinned |
| owner | Lyra |
| acceptance owner | Lyra |

## Objective

Verify Nimbus packet `NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md` against the exact pinned commit `68a7b38` on `track/infra-foundation`.

This is a **commit-pinned remote verification** task. The target is not the moving branch head; the target is the exact commit hash above.

## Scope

Read-only verification unless you hit a trivial verifier-only issue that can be fixed safely.

Primary target surfaces:
- `infra/postgres/schema/005_prompt_and_channel_action_authority.sql`
- `infra/postgres/seed/004_prompt_and_channel_action_seed.sql`
- `scripts/verify-postgres-baseline.sh`
- `crates/seatloom-core/src/db/models.rs`
- `crates/seatloom-core/src/db/repositories.rs`
- `.seatloom/bootstrap/source-map.yaml`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md`

## Remote environment

Use the sponsor-provided remote workspace path and login method already documented in:
- `docs/infra/ssh-tunnel-workspace.md`

Do not touch projects outside SeatLoom.

## Verification method

### 0. Commit identity gate

You must prove the exact code under test before anything else.

Required evidence:
- remote clone/fetch succeeds
- checkout detached `HEAD` at `68a7b38`
- `git rev-parse HEAD` returns `68a7b38...`
- `git status --short` is empty before verification starts

If commit identity cannot be proven, stop and return `HOLD`.

### 1. Rust sanity gate

Run exactly:

```bash
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
```

Expected minimum:
- `cargo check -p seatloom-core` => PASS
- `cargo test -p seatloom-core` => PASS

If these fail, stop and return `HOLD` with the first real blocker.

### 2. PostgreSQL baseline verifier double-run

Run the full verifier twice on the same checked-out commit:

```bash
bash scripts/verify-postgres-baseline.sh
bash scripts/verify-postgres-baseline.sh
```

This is the critical proof point. We need to prove:
- schema 005 is included in the verifier path
- seed 004 is included in the verifier path
- the prompt/channel-action packet does not break the prior accepted baseline
- the verifier is deterministic on repeat runs without manual cleanup between run 1 and run 2

Expected result:
- run 1 => PASS, exit 0
- run 2 => PASS, exit 0
- verifier output includes all accepted baseline steps and the new schema/seed path

If run 1 fails, stop there and return `HOLD`.
If run 1 passes but run 2 fails, return `HOLD` and name the repeat-run blocker explicitly.

### 3. Schema-005 presence checks

After the verifier passes, run SQL spot-checks against the seeded local verification DB.

You must prove the three new families exist:
- `prompt_instances`
- `prompt_actions`
- `channel_action_receipts`

You must also prove the idempotency constraint exists on channel receipts.

Suggested checks:

```sql
SELECT to_regclass('public.prompt_instances');
SELECT to_regclass('public.prompt_actions');
SELECT to_regclass('public.channel_action_receipts');

SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'channel_action_receipts'
ORDER BY indexname;
```

Acceptance expectation:
- all three `to_regclass(...)` queries return non-null
- there is a unique index or unique constraint proving `idempotency_key` uniqueness

### 4. Seed-004 behavior checks

This seed is intentionally zero-row / contract-first. Do not fail it for being sparse.

You must verify:
- schema 005 loads cleanly
- seed 004 loads cleanly
- zero-row seed rationale is consistent with source-map v4 / delivery doc
- no prior baseline tables regress to zero unexpectedly

Required spot-checks:

```sql
SELECT COUNT(*) FROM prompt_instances;
SELECT COUNT(*) FROM prompt_actions;
SELECT COUNT(*) FROM channel_action_receipts;
SELECT COUNT(*) FROM documents WHERE body_text IS NOT NULL;
SELECT COUNT(*) FROM review_threads;
SELECT COUNT(*) FROM checkpoints;
```

Acceptance expectation:
- prompt/channel tables may legitimately be `0`
- existing accepted baseline proof tables remain non-zero where prior acceptance expected non-zero

### 5. Repository-surface proof

Confirm, by code inspection plus test/log evidence, that repository support exists for the new families.

Minimum questions to close:
- Are prompt instance write/read methods present?
- Are prompt action append/list methods present?
- Are channel action receipt write/list/get methods present?
- Is duplicate `idempotency_key` rejection covered by an integration test path?

You do not need to invent new tests. You do need to confirm the claimed repository surface matches the commit under verification.

### 6. Outcome contract

Return one of exactly three verdicts:
- `PASS` — commit `68a7b38` is verified remotely and repeat-run safe
- `HOLD` — a real blocker exists; name the first real blocker precisely
- `FAIL` — only if the packet is fundamentally wrong rather than environment-blocked

## If you patch a trivial verifier issue

Default is read-only verification.

Only if you hit a **small, local, verifier-only defect** that is safe to patch directly, you may patch it, but then you must:
1. commit the fix on the current branch,
2. report the new commit hash explicitly,
3. state exactly why the patch was verifier-safe and low-risk,
4. separate Nimbus's original target commit from your follow-up verification commit.

Do not make broad product changes.

## Done definition

- exact target commit `68a7b38` proven on remote seat
- `cargo check -p seatloom-core` PASS
- `cargo test -p seatloom-core` PASS
- `bash scripts/verify-postgres-baseline.sh` run 1 PASS
- `bash scripts/verify-postgres-baseline.sh` run 2 PASS
- schema-005 families proven present
- `channel_action_receipts.idempotency_key` uniqueness proven
- zero-row seed rationale evaluated correctly
- delivery artifact written with logs/evidence paths

## Reporting format

Return in this format:

```text
[Flux -> Lyra] Prompt + Channel Action Authority Remote Verification
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
- none OR ...
verdict:
- PASS / HOLD / FAIL
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-prompt-channel-authority-remote-verification-delivery-v1.md
- .local/evidence/.../
```
