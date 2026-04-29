# Delivery: Nimbus Foundation Quality Automation

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-foundation-quality-automation-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| task_ref | NIMBUS-2026-04-29-foundation-quality-automation-v1 |
| from | Nimbus |
| to | Lyra |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-v1.md` |
| tags | nimbus, rust, infrastructure, ci, toolchain, verification |

## 1. Scope Completed

Strengthened SeatLoom's Rust engineering foundation with infrastructure-only changes:

- Pinned the Rust toolchain at repo root for reproducible verification across seats and CI.
- Added one local verification script for bounded Rust foundation checks.
- Added one CI workflow mirroring the same checks on push/PR to main.
- Fixed the workspace profile warning by moving `[profile.release]` from `src-tauri/Cargo.toml` to the workspace root `Cargo.toml`.
- No business/product behavior changes introduced.
- `NIMBUS-2026-04-29-artifact-read-models-v1` remains paused as directed.

## 2. Files Changed

| File | Action | Purpose |
|---|---|---|
| `rust-toolchain.toml` | Created | Pin Rust stable toolchain at repo root |
| `Cargo.toml` | Modified | Added `[profile.release]` moved from src-tauri |
| `src-tauri/Cargo.toml` | Modified | Removed `[profile.release]` (now at workspace root) |
| `scripts/verify-rust-foundation.sh` | Created | Local Rust foundation verification entrypoint |
| `.github/workflows/rust-foundation.yml` | Created | CI workflow for Rust foundation checks |

## 3. Toolchain Pinning

Created `rust-toolchain.toml` at repo root:

```toml
[toolchain]
channel = "stable"
```

This matches the ENV-001 verified baseline (rustc 1.95.0, cargo 1.95.0, stable-aarch64-apple-darwin). Using `stable` channel rather than a pinned version ensures seats and CI always use the latest stable without manual version bumps, while remaining deterministic within a given rustup update cycle.

A new seat or CI runner will automatically discover and install the intended toolchain via rustup's `rust-toolchain.toml` convention.

## 4. Local Verification Script Behavior

`scripts/verify-rust-foundation.sh`:

- Resolves `cargo` from `$CARGO` env var, then `PATH`, then `$HOME/.cargo/bin/cargo` as fallback.
- Prints toolchain versions for traceability.
- Runs two bounded checks in sequence:
  1. `cargo check` (workspace-level)
  2. `cargo test -p seatloom-core`
- Uses `set -euo pipefail` — exits on first failure.
- Exits 0 only if all checks pass.

## 5. CI Workflow Behavior

`.github/workflows/rust-foundation.yml`:

- **Trigger**: push to `main` + pull requests to `main`.
- **Runner**: `ubuntu-latest`.
- **Steps**:
  1. Checkout repo.
  2. Install Rust stable toolchain via `dtolnay/rust-toolchain@stable` (respects `rust-toolchain.toml`).
  3. Cache cargo registry and build artifacts via `actions/cache@v4`.
  4. `cargo check` (workspace).
  5. `cargo test -p seatloom-core`.
- Stays Rust-foundation only — no frontend, no Tauri build, no product logic.

CI workflow lint or dry-run is not possible locally; this is stated explicitly per packet instructions.

## 6. Manifest / Warning Cleanup Summary

**Before**: `cargo check` emitted:
```
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   .../src-tauri/Cargo.toml
workspace: .../Cargo.toml
```

**Fix**: Moved `[profile.release] opt-level = "s"` from `src-tauri/Cargo.toml` to the workspace root `Cargo.toml`. This is a manifest-only change with no behavioral difference — Cargo always applies workspace-root profiles regardless.

**After**: The profile warning is eliminated. `cargo check` output no longer contains the "profiles for the non root package will be ignored" message.

**Residual warnings**: 13 `dead_code` warnings remain in `seatloom-tauri` for command functions not yet registered with Tauri's invoke handler. These are pre-existing, out of this packet's scope, and will resolve when commands are registered in a future runtime-engine packet.

## 7. Validation Commands and Results

### `$HOME/.cargo/bin/cargo check` (workspace)

| Crate | Exit Code | Warnings | Result |
|---|---|---|---|
| seatloom-core | 0 | 0 | PASS |
| seatloom-tauri | 0 | 13 (pre-existing dead_code, out of scope) | PASS |
| seatloom-cli | 0 | 0 | PASS |
| **Profile warning** | — | **0 (fixed)** | **PASS** |

**Workspace cargo check: exit 0.**

### `$HOME/.cargo/bin/cargo test -p seatloom-core`

**22 passed, 0 failed, 0 ignored. No regression.**

### `scripts/verify-rust-foundation.sh`

**Exit 0. All checks passed.**

## 8. Residual Notes / Explicit Non-Goals Kept Out

- **No product/business logic**: zero changes to storage repositories, object models, Tauri command behavior, or frontend.
- **No artifact read models**: `NIMBUS-2026-04-29-artifact-read-models-v1` remains paused per Lyra directive.
- **No dead_code warning cleanup**: 13 pre-existing `seatloom-tauri` warnings are not in scope.
- **No Tauri build CI**: CI covers Rust foundation only, not the full Tauri desktop build (which requires platform-specific dependencies).
- **No Cargo.lock committed**: the workspace does not currently track `Cargo.lock` in git; this is a separate decision.

## 9. Recommended Next Owner

- **Lyra**: acceptance review of this delivery.
- **Nimbus**: next bounded infrastructure or implementation packet per Lyra direction.
