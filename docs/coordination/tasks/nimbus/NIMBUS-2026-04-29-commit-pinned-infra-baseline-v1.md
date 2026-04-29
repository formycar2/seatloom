# Task: Commit-Pinned Infrastructure Baseline

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-commit-pinned-infra-baseline-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/COORDINATION_RULES.md`, `docs/coordination/reviews/2026-04-29-seatloom-branch-management-design-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-delivery-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-delivery-v1.md` |
| tags | nimbus, infrastructure, git, branch-management, postgres, verification |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | Execute this packet alone. No UI work. No product-scope widening. No business logic expansion. |

## Objective

Produce one commit-pinned infrastructure baseline that Flux can verify remotely by exact branch + exact commit.

This packet exists because the PostgreSQL baseline must now close against a reproducible Git commit, not a floating local workspace.

## Model Routing

Preferred reasoning model for this packet: `claude-opus-4-6`.

Transport note: Lyra will not send `/model` switch commands inline in Nimbus seat-chat messages. Treat this as advisory routing context, not a required chat command.

## Scope

You are responsible for the infrastructure-only baseline needed for remote PostgreSQL verification.

Target outcome:

1. create or confirm an infrastructure track branch named `track/infra-foundation`,
2. place the accepted infrastructure baseline on that branch as a real Git commit,
3. push that branch so the sponsor-provided remote workspace can fetch it from GitHub,
4. report the exact branch and exact commit SHA for Flux.

## Allowed Change Surface

Allowed code/config areas for this packet:

- `crates/seatloom-core/**`
- `src-tauri/**` when the change is infrastructure-facing and already part of the accepted foundation scope
- `infra/postgres/**`
- `scripts/verify-rust-foundation.sh`
- `scripts/verify-postgres-baseline.sh`
- `rust-toolchain.toml`
- `.github/workflows/rust-foundation.yml`
- `.seatloom/bootstrap/**`
- infrastructure-bound Nimbus delivery docs needed to keep the branch self-describing

Out of scope:

- `ui/**`
- frontend refactor work
- PRD / UX / interaction contract rewrites
- new product behavior
- new business logic slices beyond the already delivered infrastructure baseline

## Safety Rules

- Do not rewrite or revert unrelated dirty work from other seats.
- Do not merge into `main`.
- Do not force-push shared branches.
- If you cannot isolate the infrastructure baseline safely without touching unrelated work, stop and report the blocker instead of improvising.

## Required Delivery Fields

Your delivery artifact must report these exact fields:

- `base_branch`
- `base_commit`
- `delivery_branch`
- `delivery_commit`
- `pushed_to_origin` (`yes` / `no` + reason)
- `included_paths`
- `excluded_paths`
- `validation_commands`
- `validation_results`

## Validation

Run the standard infrastructure gate on the exact delivery commit:

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
$HOME/.cargo/bin/cargo fmt --all --check
$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings
scripts/verify-rust-foundation.sh
```

If PostgreSQL verification can also be run locally on your seat without widening scope, you may include it, but it is not required for this packet.

## Deliverable

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-commit-pinned-infra-baseline-delivery-v1.md`

Then report to Lyra in tmux with:

- exact branch
- exact commit SHA
- whether the branch is available on origin
- whether Flux can now run commit-pinned remote verification

## Done Definition

This packet is done only when:

1. an infrastructure-only branch exists for verification,
2. the branch contains a real committed baseline,
3. the branch is available to a remote Git-based verifier,
4. the delivery artifact lists the exact commit identity and validation evidence.
