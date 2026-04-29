# Task: PostgreSQL Seed PG16 Compatibility Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-commit-pinned-infra-baseline-delivery-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-v2.md`, `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md` |
| tags | nimbus, postgres, seed, pg16, compatibility, infrastructure, verification-followup |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | Execute this packet alone. Infrastructure-only. No UI work. No product-scope widening. |

## Objective

Close the exact blocker found by Flux during commit-pinned remote PostgreSQL verification.

Flux verified the exact target commit on the sponsor remote workspace and reached Docker-backed PostgreSQL execution successfully. The only blocker was PostgreSQL 16 rejecting an untyped empty array in the seed file.

This packet is complete only when the seed is made PostgreSQL-16 compatible, committed on the infrastructure track, and pushed so Flux can re-run verification by exact commit.

## Verified Blocker

From `docs/coordination/tasks/flux/FLUX-2026-04-29-remote-workspace-postgres-verification-delivery-v1.md`:

- verified `HEAD`: `d007721bd36e9fdd0e145e68a04e32c1bc4d9cf2`
- verification path: `A (Docker exists)`
- blocker: `infra/postgres/seed/001_real_collaboration_baseline.sql` line 33 contains `ARRAY[]` without explicit type cast; PostgreSQL 16 errors with `cannot determine type of empty array`

## Model Routing

Preferred reasoning model for this packet: `claude-opus-4-6`.

Transport note: do not expect an inline `/model` chat command from Lyra. Treat model preference as routing guidance only.

## Scope

You own a bounded infrastructure fix only:

1. patch the seed file so empty arrays are explicitly typed for PostgreSQL 16,
2. scan the touched seed file for any other ambiguous empty-array usage and fix them in the same bounded pass,
3. keep all seeded semantics unchanged,
4. commit the fix on `track/infra-foundation`,
5. push the updated branch and report the exact new commit SHA.

## Allowed Change Surface

Allowed paths:

- `infra/postgres/seed/001_real_collaboration_baseline.sql`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-delivery-v1.md`
- any strictly necessary Nimbus-only bounded evidence note under `docs/coordination/tasks/nimbus/`

Out of scope:

- `ui/**`
- `crates/**` unless you discover a strictly necessary infra-only follow-up caused directly by this blocker
- schema redesign
- seed content expansion
- product behavior changes
- branch strategy rewrites

## Required Change

At minimum, fix the empty `constraints` array for the Aegis project-role binding so PostgreSQL 16 can infer the element type.

Expected shape:

```sql
ARRAY[]::text[]
```

If any other `ARRAY[]` form exists in the same seed file, fix all of them in this packet.

## Safety Rules

- Do not rewrite or revert unrelated dirty work from other seats.
- Do not touch frontend files.
- Do not force-push.
- If the branch has moved and you cannot safely apply this bounded fix on top of the current infrastructure track, stop and report the blocker instead of improvising.

## Validation

Run the standard local infra gate on the exact delivery commit:

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
$HOME/.cargo/bin/cargo fmt --all --check
$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings
scripts/verify-rust-foundation.sh
```

Then add a targeted static proof that the seed no longer contains ambiguous empty arrays:

```bash
rg -n "ARRAY\[\]" infra/postgres/seed/001_real_collaboration_baseline.sql
```

Expected result:

- no untyped `ARRAY[]` remains in the seed file, or every remaining occurrence is explicitly cast in-place.

If your seat can run the PostgreSQL verification end to end without widening scope, include that result; it is optional for this packet because Flux will perform the authoritative re-verification.

## Required Delivery Fields

Your delivery artifact must report:

- `base_branch`
- `base_commit`
- `delivery_branch`
- `delivery_commit`
- `pushed_to_origin` (`yes` / `no` + reason)
- `touched_paths`
- `validation_commands`
- `validation_results`
- `remaining_pg16_seed_occurrences`
- `ready_for_flux_reverify` (`yes` / `no`)

## Deliverable

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-postgres-seed-pg16-compat-fix-delivery-v1.md`

Then report back to Lyra in tmux with:

- exact branch
- exact new commit SHA
- whether origin now contains that commit
- whether Flux can re-run remote commit-pinned verification immediately

## Done Definition

This packet is done only when:

1. the PostgreSQL 16 seed blocker is patched,
2. the fix is committed on the infrastructure track,
3. the branch is pushed to origin,
4. Lyra can hand Flux an exact new commit for re-verification.
