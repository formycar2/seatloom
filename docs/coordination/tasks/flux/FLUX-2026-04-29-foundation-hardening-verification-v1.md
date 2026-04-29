# Task: Flux Verification of Nimbus Rust Foundation Hardening

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-foundation-hardening-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-quality-automation-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | flux, verification, rust, infrastructure, ci, toolchain, clippy, formatting |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | Read-only verification only. Do not patch product code. Do not start until Nimbus has written `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-delivery-v1.md`. |

## Objective

Run one detailed read-only verification pass against Nimbus's Rust foundation hardening packet.

Your job is not to redesign the packet. Your job is to prove whether the hardening gate is actually closed in the current repo state.

Because this is infrastructure work, be strict about command results, scope discipline, and whether the changes stayed out of product/business logic.

## Start Condition

Do not begin until this file exists:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-delivery-v1.md`

If it does not exist yet, wait.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-quality-automation-acceptance.md`
3. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-v1.md`
4. `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-delivery-v1.md`
5. this packet

## Need-to-Know Scope

Review only the hardening packet and the files it declares.

Expected primary files:

- `rust-toolchain.toml`
- `scripts/verify-rust-foundation.sh`
- `.github/workflows/rust-foundation.yml`
- `Cargo.toml`
- `src-tauri/Cargo.toml`
- any Rust files touched strictly for `cargo fmt`, `seatloom-core` clippy closure, or scaffold warning suppression

Do not reopen:

- frontend/UI work
- product contract docs
- artifact read models
- retrieval engine implementation
- repository/business feature work
- runtime command behavior

## Verification Questions

Answer every question explicitly:

1. Is the Rust toolchain frozen to the exact verified version rather than a floating channel?
2. Does the local verification script resolve Cargo/Rustc portably and complete successfully from repo root?
3. Does CI mirror the same bounded validation sequence as the local script?
4. Do these commands pass exactly as claimed?
   - `$HOME/.cargo/bin/cargo check`
   - `$HOME/.cargo/bin/cargo test -p seatloom-core`
   - `$HOME/.cargo/bin/cargo fmt --all --check`
   - `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings`
   - `scripts/verify-rust-foundation.sh`
5. Is the old ignored-profile warning gone?
6. Is the old `seatloom-tauri` scaffold dead-code warning noise gone, or at least reduced by the bounded method Nimbus documented?
7. Did Nimbus keep the changes infrastructure-only, with no product/business behavior expansion?
8. Should the hardening packet be accepted at `PASS`, or held?

## Required Validation

Run exactly these commands from repo root and record the raw result summary:

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
$HOME/.cargo/bin/cargo fmt --all --check
$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings
scripts/verify-rust-foundation.sh
```

Also inspect the diff or touched files to ensure the packet did not widen into product/business behavior.

## Required Evidence Folder

Write any logs or notes under:

- `.local/evidence/2026-04-29-foundation-hardening-verification/`

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-foundation-hardening-verification-delivery-v1.md`

Required sections:

1. Scope reviewed
2. Verification method
3. Result by acceptance criterion
4. Command results
5. Scope-discipline check
6. Verdict (`PASS` / `HOLD`)
7. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] All required commands are re-run and recorded.
- [ ] Toolchain/script/CI parity is explicitly verified or disputed.
- [ ] Scope discipline is explicitly checked.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/flux_to_lyra_foundation_hardening_verification.txt
[Flux -> Lyra] Rust Foundation Hardening Verification
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
verdict:
- PASS / HOLD
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-29-foundation-hardening-verification-delivery-v1.md
MSG

tmux load-buffer -b flux_to_lyra_foundation_hardening_verification /tmp/flux_to_lyra_foundation_hardening_verification.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b flux_to_lyra_foundation_hardening_verification
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
