# Task: Nimbus PostgreSQL Prompt + Channel Action Authority

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-05-01 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-design.md`, `docs/architecture-decisions.md`, `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`, `docs/coordination/reviews/2026-04-30-lyra-postgres-runtime-authority-gap-review.md`, `infra/postgres/schema/001_seatloom_core.sql`, `infra/postgres/schema/002_document_authority.sql`, `infra/postgres/schema/003_write_ingest_reconcile.sql`, `infra/postgres/schema/004_operational_review_and_continuity.sql` |
| tags | nimbus, infrastructure, postgres, prompt, mobile, audit, runtime-authority, concurrency |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded infrastructure packet only. You are not alone in the codebase; do not revert others' work, do not widen into UI or product/business logic, and do not touch frontend files. |

## Objective

Close the next P0 authority gap in PostgreSQL by making two operational families first-class:

1. interactive prompt state + prompt-resolution history,
2. generic channel-action receipts for state-changing desktop / mobile / supervisor decisions.

This packet must stay infrastructure-only. Do not implement UI, Tauri commands, API handlers, or workflow automation.

## Why This Packet Exists

The current PostgreSQL baseline is now strong enough for seeded project truth, typed documents, review threads, continuity checkpoints, and deterministic bootstrap.

But three contract-critical gaps still remain:
- prompt-blocked state is not yet a durable PostgreSQL object,
- mobile and desktop state-changing actions do not yet have a canonical receipt family,
- SeatLoom-originated live writes still lack an explicit storage boundary with idempotency and optimistic-guard fields.

This packet addresses those gaps at the schema + repository + verification layer only.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-design.md`
6. `docs/architecture-decisions.md`
7. `docs/coordination/reviews/2026-04-29-lyra-data-structure-and-flow-review.md`
8. `docs/coordination/reviews/2026-04-30-lyra-postgres-runtime-authority-gap-review.md`
9. `infra/postgres/schema/001_seatloom_core.sql`
10. `infra/postgres/schema/002_document_authority.sql`
11. `infra/postgres/schema/003_write_ingest_reconcile.sql`
12. `infra/postgres/schema/004_operational_review_and_continuity.sql`
13. this packet

## Need-to-Know Scope

In scope:
- extend PostgreSQL authority with prompt lifecycle tables,
- extend PostgreSQL authority with generic channel-action receipt tables,
- define optimistic-guard / idempotency fields at the storage edge,
- add typed Rust models and repository methods,
- add focused tests,
- update the PG baseline verifier if schema ordering changes,
- add real seed rows only when directly evidence-backed,
- otherwise keep baseline seed honest and document the zero-row state.

Out of scope:
- UI/frontend work,
- Tauri command wiring,
- API or IPC mutation surfaces,
- runtime automation behavior,
- prompt classifier implementation,
- mobile screens,
- notification delivery,
- semantic retrieval,
- business logic beyond storage validation,
- background daemons or watchers.

## Branch and Commit Discipline

Stay on:
- `track/infra-foundation`

When done:
1. commit only the bounded infrastructure changes from this packet,
2. push the branch,
3. report the exact commit hash in the delivery artifact and tmux reply,
4. keep the result Flux-verifiable by exact commit.

## Write Boundary

Primary targets:
- `infra/postgres/schema/005_prompt_and_channel_action_authority.sql`
- `infra/postgres/seed/004_prompt_and_channel_action_seed.sql` (only if directly evidence-backed; otherwise do not fabricate)
- `.seatloom/bootstrap/source-map.yaml`
- `crates/seatloom-core/src/db/models.rs`
- `crates/seatloom-core/src/db/repositories.rs`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `scripts/verify-postgres-baseline.sh`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md`

Allowed supporting files if they keep the scope cleaner:
- `crates/seatloom-core/src/objects/prompt.rs`
- `crates/seatloom-core/src/db/prompt_validation.rs`
- `crates/seatloom-core/tests/postgres_prompt_channel_action_consistency.rs`

Disallowed in this packet:
- files under `ui/`
- `src-tauri/src/commands/**`
- workflow automation logic
- prompt classifier logic
- non-infra docs unrelated to authority design

## Required Outcome

### 1. Add first-class prompt authority tables

Create a bounded PostgreSQL family that persists the active and historical prompt lifecycle.

Minimum fields required for the prompt instance family:
- `id`
- `project_id`
- `session_id`
- `status` (`active`, `resolved`, `stopped`, `expired`, `superseded` or equivalent bounded set)
- `prompt_kind` (`deterministic`, `wizard_menu`, `freeform`, `sensitive`)
- `prompt_policy` (`auto_allowed`, `needs_approval`, `human_required`)
- bounded evidence reference and/or bounded preview payload
- `available_actions`
- assist budget fields (`max_steps`, `max_tokens`, `steps_used`, `tokens_used`) or equivalent JSONB with typed access
- `expected_next_pattern` if available
- `detected_at`
- `resolved_at`
- `resolved_by`
- optional `result_event_id`
- `created_at`
- `updated_at`

Requirements:
- one row must be enough to answer "what prompt is blocking this session right now?"
- history must not depend on rereading terminal logs,
- no raw unbounded transcript blob may become the authority surface.

### 2. Add prompt action history

Persist the resolution trail for prompt actions.

Minimum fields required:
- `id`
- `prompt_id`
- `action_kind` (`approve`, `human_takeover`, `supervisor_assist`, `stop`, `input_injected`, `auto_completed` or equivalent bounded set)
- `actor_ref`
- `source_channel` (`desktop`, `mobile`, `supervisor`, `system`)
- bounded note or injected input preview/reference
- assist budget usage snapshot if applicable
- `result_status` (`applied`, `rejected`, `stopped`, `conflict` or equivalent)
- optional `result_event_id`
- `created_at`

Requirements:
- this family must support `INT-16` audit expectations,
- supervisor-assist budget usage must be durable,
- sensitive prompts must remain distinguishable at the storage layer.

### 3. Add generic channel-action receipts

Create a canonical receipt family for state-changing desktop / mobile / supervisor actions.

This is the storage-edge contract that future SeatLoom UI actions will write into before or alongside domain-state mutation.

Minimum fields required:
- `id`
- `project_id`
- `target_kind` (`workitem`, `handoff`, `review_thread`, `prompt`, `session`, `artifact`, `document`, `project`)
- `target_id`
- `action_kind` (`approve`, `reject`, `escalate`, `reserve_desktop_takeover`, `return`, `comment_submit`, `stop`, or equivalent bounded set)
- `actor_ref`
- `source_channel` (`desktop`, `mobile`, `supervisor`, `system`)
- optional bounded note
- `evidence_refs`
- optional `policy_summary`
- `idempotency_key`
- optional optimistic-guard fields such as `expected_revision` and `applied_revision` (or equivalent bounded version/status guard)
- `receipt_status` (`applied`, `rejected`, `conflicted`, `noop` or equivalent)
- optional `result_event_id`
- `created_at`

Requirements:
- this family must be general enough to cover the mobile approval / interrupt / feedback contract without inventing mobile-only state branches,
- it must also cover desktop and supervisor actions so the audit surface stays unified,
- keep the contract explicit and typed rather than hiding everything inside one opaque JSON payload.

### 4. Add repository contracts

Add typed Rust DB models and repository methods for:
- create/list/get prompt instances,
- append/list prompt actions,
- create/list/get channel-action receipts,
- filter by `session_id`, `target_kind`, `target_id`, `source_channel`, and `status`.

Even if full write APIs are not wired yet, the repository boundary must be real and testable.

### 5. Keep seed truth honest

Do not fabricate prompt/mobile operational seed rows if the current collaboration record does not directly support them.

Allowed:
- zero-row baseline for the new families with explicit documentation in `.seatloom/bootstrap/source-map.yaml`,
- isolated test fixtures created inside integration tests,
- seed rows only when you can cite concrete repo evidence.

This honesty rule is mandatory.

### 6. Extend verification cleanly

If schema `005` is added, update `scripts/verify-postgres-baseline.sh` so the verifier applies it in the accepted schema order.

If no real seed rows are added, the verifier should still pass and the tests should prove the new families structurally work.

## Acceptance Criteria

1. PostgreSQL includes a first-class prompt authority family.
2. PostgreSQL includes a prompt action history family.
3. PostgreSQL includes a generic channel-action receipt family for desktop/mobile/supervisor actions.
4. The new storage edge exposes idempotency and optimistic-guard fields suitable for future live SeatLoom writes.
5. Typed Rust DB models and repositories exist for the new families.
6. Seed policy stays honest: no fabricated prompt/mobile runtime baseline.
7. `scripts/verify-postgres-baseline.sh` remains deterministic if schema ordering changes.
8. No UI, no Tauri command wiring, no classifier logic, and no business automation is added.
9. `$HOME/.cargo/bin/cargo check -p seatloom-core` passes.
10. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
11. The resulting commit is Flux-verifiable by exact hash.

## Required Validation

At minimum:

```bash
$HOME/.cargo/bin/cargo check -p seatloom-core
$HOME/.cargo/bin/cargo test -p seatloom-core
$HOME/.cargo/bin/cargo fmt --all --check
$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings
bash -n scripts/verify-postgres-baseline.sh
```

If Docker is available on your seat, also run:

```bash
bash scripts/verify-postgres-baseline.sh
```

If Docker is unavailable, record the exact limitation and keep the packet Flux-ready by exact commit.

## Required Delivery Artifact

Write:
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md`

Required sections:
1. Scope completed
2. Contract requirements covered
3. Schema design chosen
4. Seed policy and evidence basis
5. Files changed
6. Validation commands and results
7. Residual non-goals kept out
8. Exact branch and commit
9. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Schema 005 is added or the equivalent bounded family is created.
- [ ] Prompt authority tables exist.
- [ ] Channel-action receipt tables exist.
- [ ] Repository models/methods exist.
- [ ] Validation commands/results are recorded.
- [ ] Exact commit hash is reported.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```text
[Nimbus -> Lyra] PostgreSQL Prompt + Channel Action Authority
branch:
- track/infra-foundation
commit:
- <exact commit>
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo fmt --all --check` => ...
- `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings` => ...
- `bash scripts/verify-postgres-baseline.sh` => ... / not run because ...
blockers:
- none / ...
next action:
- wait for Flux commit-pinned verification and Lyra acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md
```
