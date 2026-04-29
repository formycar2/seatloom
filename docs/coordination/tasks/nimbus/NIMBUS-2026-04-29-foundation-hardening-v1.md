# Task: Nimbus Rust Foundation Hardening

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-foundation-hardening-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-quality-automation-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | nimbus, rust, infrastructure, toolchain, ci, lint, formatting |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded infrastructure packet. You are not alone in the codebase; do not revert others' work, do not widen into product/business logic, and keep `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-artifact-read-models-v1.md` paused. |

## Objective

Use the infrastructure-only lane to harden the Rust foundation into a repeatable, low-noise engineering baseline.

This packet has two jobs:

1. close the remaining acceptance gaps from `foundation-quality-automation`
2. raise the Rust foundation gate so future infrastructure work starts from deterministic toolchain + portable verification + clean quality checks

Do not resume product/business backend work.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/architecture-decisions.md`
3. `docs/architecture-design.md`
4. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-v1.md`
5. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-delivery-v1.md`
6. `docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-quality-automation-acceptance.md`
7. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
8. `docs/coordination/COORDINATION_RULES.md`
9. this packet

## Need-to-Know Scope

Infrastructure only.

In scope:

- freeze the verified Rust toolchain exactly
- make the local verification script portable across seats
- add `rustfmt` + `clippy` into the Rust foundation gate
- normalize Rust formatting where required for the new gate
- close the current `seatloom-core` clippy failures without changing product behavior
- remove the current `seatloom-tauri` dead-code warning noise by a bounded infrastructure-safe method
- mirror the tightened checks in CI
- document exact results and residual non-goals

Out of scope:

- artifact read models
- storage/repository feature expansion
- retrieval engine behavior
- Tauri invoke wiring or runtime behavior changes
- PTY/runtime integration
- UI/frontend work
- product/business logic additions
- schema/IA changes driven by the redesign track

## Write Boundary

Primary targets:

- `rust-toolchain.toml`
- `scripts/verify-rust-foundation.sh`
- `.github/workflows/rust-foundation.yml`
- `Cargo.toml`
- `src-tauri/Cargo.toml` (only if strictly needed)
- Rust source files touched only for formatting, clippy closure, or dead-code warning suppression within the existing foundation scaffold

Allowed Rust code-change categories:

- exact toolchain metadata
- `Default` impls or equivalent lint-safe refactors
- sort/filter idiom cleanup surfaced by clippy
- formatting-only changes from `cargo fmt`
- targeted `#[allow(dead_code)]` or similarly bounded warning suppression on scaffold-only Tauri stubs that are intentionally not wired yet

Disallowed Rust code-change categories:

- new product objects/fields
- new repository behavior
- new IPC behavior
- new runtime command semantics
- anything that resumes the paused artifact-read-model lane

## Required Outcome

### 1. Freeze the verified toolchain exactly

Update `rust-toolchain.toml` to the exact verified baseline:

- channel `1.95.0`
- include the components needed by the new gate (`rustfmt`, `clippy`)

The goal is time-stable reproducibility, not a floating stable channel.

### 2. Repair and harden the local verification script

` scripts/verify-rust-foundation.sh ` must:

- resolve Cargo and Rustc from the active toolchain robustly
- avoid bare `rustc` assumptions that fail on seats where only rustup shims are discoverable
- print the exact toolchain used for traceability
- run the full bounded validation sequence from repo root

Required local sequence:

1. `cargo check`
2. `cargo test -p seatloom-core`
3. `cargo fmt --all --check`
4. `cargo clippy -p seatloom-core --all-targets -- -D warnings`

Keep the script deterministic and simple.

### 3. Tighten CI to match the local gate

Update `.github/workflows/rust-foundation.yml` so CI mirrors the same bounded gate:

- exact toolchain from `rust-toolchain.toml`
- `cargo check`
- `cargo test -p seatloom-core`
- `cargo fmt --all --check`
- `cargo clippy -p seatloom-core --all-targets -- -D warnings`

Do not widen into frontend, Tauri bundle builds, or OS-matrix experimentation.

### 4. Close clippy failures in `seatloom-core`

Make the current `seatloom-core` code pass:

```bash
cargo clippy -p seatloom-core --all-targets -- -D warnings
```

The current failures are lint/hygiene level (`new_without_default`, `unnecessary_map_or`, `unnecessary_sort_by`, formatting-driven idioms).

Fix them without changing product meaning.

### 5. Remove current workspace warning noise from scaffold-only Tauri stubs

`cargo check` is currently still noisy because unregistered Tauri stub commands and `AppState` emit `dead_code` warnings.

Close that warning noise by a bounded infrastructure-safe method only:

- targeted allow annotations on intentionally unwired scaffold functions / state are acceptable
- command registration or behavior wiring is not acceptable in this packet

The goal is a cleaner foundation signal, not earlier runtime implementation.

## Acceptance Criteria

1. `rust-toolchain.toml` freezes the exact verified version and required components.
2. `scripts/verify-rust-foundation.sh` succeeds on Lyra's seat.
3. The script and CI workflow mirror the same bounded validation sequence.
4. `$HOME/.cargo/bin/cargo check` passes without the prior profile warning and without the scaffold dead-code warning noise.
5. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
6. `$HOME/.cargo/bin/cargo fmt --all --check` passes.
7. `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings` passes.
8. No product/business behavior is added.
9. `NIMBUS-2026-04-29-artifact-read-models-v1` remains paused.

## Required Validation

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
$HOME/.cargo/bin/cargo fmt --all --check
$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings
scripts/verify-rust-foundation.sh
```

If CI dry-run is not possible locally, state that explicitly.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Exact toolchain freeze
4. Local verification script hardening
5. CI parity update
6. Clippy / formatting closure summary
7. Warning-noise cleanup summary
8. Validation commands and results
9. Residual non-goals kept out
10. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Exact toolchain freeze is present.
- [ ] Local script succeeds on the full bounded sequence.
- [ ] CI mirrors the same bounded sequence.
- [ ] `cargo fmt --all --check` passes.
- [ ] `cargo clippy -p seatloom-core --all-targets -- -D warnings` passes.
- [ ] `cargo check` is clean from the known scaffold warning noise.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_foundation_hardening.txt
[Nimbus -> Lyra] Rust Foundation Hardening
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo fmt --all --check` => ...
- `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings` => ...
- `scripts/verify-rust-foundation.sh` => ...
blockers:
- none / ...
next action:
- wait for Flux verification and Lyra acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_foundation_hardening /tmp/nimbus_to_lyra_foundation_hardening.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_foundation_hardening
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
