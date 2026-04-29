# Delivery: Nimbus Rust Foundation Hardening

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-foundation-hardening-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| task_ref | NIMBUS-2026-04-29-foundation-hardening-v1 |
| from | Nimbus |
| to | Lyra |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-v1.md` |
| tags | nimbus, rust, infrastructure, toolchain, ci, lint, formatting |

## 1. Scope Completed

Hardened SeatLoom's Rust foundation into a repeatable, low-noise engineering baseline:

- Froze the Rust toolchain to exact version 1.95.0 with `rustfmt` and `clippy` components.
- Hardened the local verification script with portable cargo/rustc resolution and the full 4-step gate.
- Updated the CI workflow to mirror the same 4-step bounded gate on the exact toolchain.
- Closed all 28 `seatloom-core` clippy failures without changing product behavior.
- Eliminated all 13 `seatloom-tauri` dead-code scaffold warnings via targeted `#[allow(dead_code)]`.
- Normalized formatting across the entire workspace with `cargo fmt --all`.
- No product/business behavior changes introduced.
- `NIMBUS-2026-04-29-artifact-read-models-v1` remains paused.

## 2. Files Changed

| File | Action | Purpose |
|---|---|---|
| `rust-toolchain.toml` | Modified | Froze channel to `1.95.0`, added `rustfmt` + `clippy` components |
| `scripts/verify-rust-foundation.sh` | Modified | Added fmt/clippy steps, portable cargo resolution with `$HOME/.cargo/env` sourcing |
| `.github/workflows/rust-foundation.yml` | Modified | Switched to `dtolnay/rust-toolchain@master` with `1.95.0`, added fmt + clippy steps |
| `crates/seatloom-core/src/objects/id.rs` | Modified | Added `Default` impl to `define_id!` macro (closes clippy `new_without_default` for all 10 ID types) |
| `crates/seatloom-core/src/git/ops.rs` | Modified | Added `#[derive(Default)]` to `GitOps` |
| `crates/seatloom-core/src/pipeline/runner.rs` | Modified | Added `#[derive(Default)]` to `PipelineRunner` |
| `crates/seatloom-core/src/reconcile/engine.rs` | Modified | Added `#[derive(Default)]` to `ReconcileEngine` |
| `crates/seatloom-core/src/views/inbox.rs` | Modified | Added `#[derive(Default)]` to `InboxView` |
| `crates/seatloom-core/src/views/timeline.rs` | Modified | Added `#[derive(Default)]` to `TimelineView` |
| `crates/seatloom-core/src/data_engine/audit.rs` | Modified | Added `#[derive(Default)]` to `AuditBinder` |
| `crates/seatloom-core/src/data_engine/budget_enforcer.rs` | Modified | Added `#[derive(Default)]` to `BudgetEnforcer` |
| `crates/seatloom-core/src/data_engine/gate_engine.rs` | Modified | Added `#[derive(Default)]` to `GateEngine` |
| `crates/seatloom-core/src/data_engine/isolation.rs` | Modified | Added `#[derive(Default)]` to `IsolationLayer` |
| `crates/seatloom-core/src/data_engine/retrieval.rs` | Modified | Added `#[derive(Default)]` to `RetrievalEngine` |
| `crates/seatloom-core/src/data_engine/route_engine.rs` | Modified | Added `#[derive(Default)]` to `RouteEngine` |
| `crates/seatloom-core/src/storage/seat_registry.rs` | Modified | `map_or` → `is_some_and`, `sort_by` → `sort_by_key` |
| `crates/seatloom-core/src/storage/session_store.rs` | Modified | `sort_by` → `sort_by_key` |
| `crates/seatloom-core/src/storage/workitem_store.rs` | Modified | `map_or` → `is_some_and`, `sort_by` → `sort_by_key` |
| `crates/seatloom-core/src/storage/handoff_store.rs` | Modified | `map_or` → `is_some_and`, `sort_by` → `sort_by_key` |
| `src-tauri/src/commands/seat_cmds.rs` | Modified | Added `#[allow(dead_code)]` |
| `src-tauri/src/commands/session_cmds.rs` | Modified | Added `#[allow(dead_code)]` on 2 functions |
| `src-tauri/src/commands/workitem_cmds.rs` | Modified | Added `#[allow(dead_code)]` on 2 functions |
| `src-tauri/src/commands/handoff_cmds.rs` | Modified | Added `#[allow(dead_code)]` |
| `src-tauri/src/commands/artifact_cmds.rs` | Modified | Added `#[allow(dead_code)]` |
| `src-tauri/src/commands/delegation_cmds.rs` | Modified | Added `#[allow(dead_code)]` |
| `src-tauri/src/commands/prompt_cmds.rs` | Modified | Added `#[allow(dead_code)]` |
| `src-tauri/src/commands/timeline_cmds.rs` | Modified | Added `#[allow(dead_code)]` |
| `src-tauri/src/commands/inbox_cmds.rs` | Modified | Added `#[allow(dead_code)]` |
| `src-tauri/src/commands/reconcile_cmds.rs` | Modified | Added `#[allow(dead_code)]` |
| `src-tauri/src/state.rs` | Modified | Added `#[allow(dead_code)]` |
| 23 files total | Formatted | `cargo fmt --all` normalization (import order, brace style, etc.) |

## 3. Exact Toolchain Freeze

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.95.0"
components = ["rustfmt", "clippy"]
```

Matches the ENV-001 verified baseline: `rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`. Time-stable — a new seat or CI runner gets exactly this version.

## 4. Local Verification Script Hardening

Updated `scripts/verify-rust-foundation.sh`:

- Sources `$HOME/.cargo/env` when present (handles seats where rustup cargo/rustc are not in default `PATH`)
- Resolves `cargo` via `command -v` after env sourcing (no bare `rustc` assumption)
- Prints cargo, rustc, clippy, and rustfmt versions for full traceability
- Runs the bounded 4-step gate:
  1. `cargo check`
  2. `cargo test -p seatloom-core`
  3. `cargo fmt --all --check`
  4. `cargo clippy -p seatloom-core --all-targets -- -D warnings`
- `set -euo pipefail` — exits on first failure

## 5. CI Parity Update

Updated `.github/workflows/rust-foundation.yml`:

- Switched from `dtolnay/rust-toolchain@stable` to `dtolnay/rust-toolchain@master` with `toolchain: "1.95.0"` and explicit `components: rustfmt, clippy`
- Added two new steps: `cargo fmt --all --check` and `cargo clippy -p seatloom-core --all-targets -- -D warnings`
- CI now mirrors the exact same 4-step bounded gate as the local script
- CI workflow lint or dry-run is not possible locally; stated explicitly per packet instructions

## 6. Clippy / Formatting Closure Summary

### Clippy Fixes (28 errors → 0)

| Category | Count | Fix |
|---|---|---|
| `new_without_default` | 21 | Added `Default` impl to `define_id!` macro (10 ID types) + `#[derive(Default)]` on 11 scaffold structs |
| `unnecessary_map_or` | 3 | `.map_or(false, \|ext\| ext == "yaml")` → `.is_some_and(\|ext\| ext == "yaml")` |
| `unnecessary_sort_by` | 4 | `.sort_by(\|a, b\| b.field.cmp(&a.field))` → `.sort_by_key(\|x\| std::cmp::Reverse(x.field))` |

All fixes are lint/idiom level — no product behavior changed.

### Formatting Normalization

`cargo fmt --all` normalized 23 files across the workspace: import ordering, single-line body expansion, and brace style adjustments. Pure formatting — no semantic changes.

## 7. Warning-Noise Cleanup Summary

### Before

`cargo check` output contained:
- 1 profile warning (eliminated in previous packet)
- 13 `dead_code` warnings from unregistered Tauri scaffold functions and `AppState`

### After

`cargo check` output is **completely clean** — zero warnings from any crate.

**Method**: targeted `#[allow(dead_code)]` annotations with comment `// scaffold: not yet registered with Tauri invoke handler` on each intentionally unwired function. No command registration or behavior wiring was introduced.

## 8. Validation Commands and Results

| # | Command | Result |
|---|---|---|
| 1 | `$HOME/.cargo/bin/cargo check` | exit 0, **0 warnings** |
| 2 | `$HOME/.cargo/bin/cargo test -p seatloom-core` | 22 passed, 0 failed |
| 3 | `$HOME/.cargo/bin/cargo fmt --all --check` | exit 0, no diffs |
| 4 | `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings` | exit 0, 0 errors, 0 warnings |
| 5 | `scripts/verify-rust-foundation.sh` | exit 0, all checks passed |

## 9. Residual Non-Goals Kept Out

- **No product/business logic**: zero changes to repository behavior, object models, or command semantics.
- **No artifact read models**: `NIMBUS-2026-04-29-artifact-read-models-v1` remains paused.
- **No Tauri invoke wiring**: scaffold functions have `#[allow(dead_code)]` only; not registered with Tauri's invoke handler.
- **No `seatloom-tauri` clippy**: clippy gate is scoped to `seatloom-core` only per packet. `seatloom-tauri` clippy cleanup is a separate scope.
- **No Cargo.lock tracking**: workspace does not currently commit `Cargo.lock`; separate decision.
- **No CI matrix expansion**: CI runs on `ubuntu-latest` only; macOS/Windows matrix is a separate scope.

## 10. Recommended Next Owner

- **Lyra**: acceptance review of this delivery.
- **Flux**: verification of the hardened foundation gate (run `scripts/verify-rust-foundation.sh` on a separate seat).
- **Nimbus**: next bounded packet per Lyra direction.
