# Task: Nimbus Foundation Quality Automation

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-foundation-quality-automation-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-env-001-tauri-icon-fix-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | nimbus, rust, infrastructure, ci, toolchain, verification |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded engineering packet. You are not alone in the codebase; do not revert others' work, do not widen into product/business logic, and do not resume `NIMBUS-2026-04-29-artifact-read-models-v1` unless Lyra explicitly reactivates it. |

## Objective

Strengthen SeatLoom's Rust engineering foundation without adding business behavior.

The active engineering lane is now infrastructure-only. The immediate need is reproducible toolchain + verification automation so future backend work can proceed on a stable base while product value / usability review continues separately.

This packet supersedes active execution of `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-artifact-read-models-v1.md`, which is now paused.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/architecture-decisions.md`
3. `docs/architecture-design.md`
4. `docs/coordination/acceptance/2026-04-29-lyra-nimbus-env-001-tauri-icon-fix-acceptance.md`
5. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
6. `docs/coordination/COORDINATION_RULES.md`
7. this packet

## Need-to-Know Scope

Implement infrastructure automation only.

In scope:

- pin the Rust toolchain at the repo root so verification is reproducible across seats / CI
- add one deterministic local verification script for the Rust workspace foundation
- add one CI workflow for the same Rust foundation checks
- clean one existing workspace-manifest warning if it is purely infrastructure (example: ignored non-root profile stanza)
- document exact commands and residual warnings / non-goals in the delivery artifact

Out of scope:

- artifact read models
- subtype validation
- Tauri command behavior changes
- storage/repository feature expansion
- retrieval engine
- runtime / PTY wiring
- route/inbox/timeline logic
- any frontend or design-system work
- any product/business feature implementation

## Write Boundary

Primary targets:

- `Cargo.toml`
- `src-tauri/Cargo.toml`
- `rust-toolchain.toml` (new, if needed)
- `.github/workflows/rust-foundation.yml` (new)
- `scripts/verify-rust-foundation.sh` (new)

You may add one small supporting doc note if needed, but keep the packet infrastructure-only and bounded.

## Required Outcome

### 1. Pin the verified Rust toolchain

Add a root-level toolchain declaration that matches the known compile-capable baseline used to close `ENV-001`, unless you find a tighter existing project convention that is clearly better and still deterministic.

The goal is: a new seat or CI runner can discover the intended toolchain without guesswork.

### 2. Add one local verification entrypoint

Provide a single repo-local script that runs the bounded Rust foundation checks from repo root.

Minimum checks:

- `$HOME/.cargo/bin/cargo check`
- `$HOME/.cargo/bin/cargo test -p seatloom-core`

You may use `cargo` from `PATH` internally if you make fallback behavior explicit, but the script must remain deterministic and simple.

### 3. Add one CI workflow

Add one GitHub Actions workflow that mirrors the same bounded Rust checks.

Workflow requirements:

- runs on push + pull_request
- installs the pinned toolchain
- runs the same validation sequence as the local script
- stays Rust-foundation only

### 4. Remove one known infrastructure warning if safely in scope

Today `cargo check` emits the workspace warning that `[profile.release]` under `src-tauri/Cargo.toml` is ignored because profiles must live at workspace root.

If you can remove that warning with a bounded manifest-only change, do it.

Do not widen into unrelated warning cleanup.

## Acceptance Criteria

1. The repo has a deterministic Rust toolchain declaration at root.
2. The repo has one local Rust-foundation verification script.
3. The repo has one CI workflow that mirrors the bounded Rust foundation checks.
4. If the ignored-profile warning can be fixed by moving config only, it is fixed without widening scope.
5. `$HOME/.cargo/bin/cargo check` passes.
6. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
7. No business/product behavior changes are introduced.
8. `NIMBUS-2026-04-29-artifact-read-models-v1` remains paused.

## Required Validation

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
scripts/verify-rust-foundation.sh
```

If CI workflow lint or dry-run is not possible locally, state that explicitly and keep scope bounded.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Toolchain pinning
4. Local verification script behavior
5. CI workflow behavior
6. Manifest / warning cleanup summary
7. Validation commands and results
8. Residual notes / explicit non-goals kept out
9. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Root toolchain pinning is present.
- [ ] Local Rust-foundation verification script exists and runs.
- [ ] CI workflow exists and matches the bounded checks.
- [ ] `$HOME/.cargo/bin/cargo check` passes.
- [ ] `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_foundation_quality_automation.txt
[Nimbus -> Lyra] Foundation Quality Automation
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `scripts/verify-rust-foundation.sh` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_foundation_quality_automation /tmp/nimbus_to_lyra_foundation_quality_automation.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_foundation_quality_automation
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
