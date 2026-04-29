# Acceptance: Nimbus Foundation Quality Automation

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-29-nimbus-foundation-quality-automation-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| target | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-delivery-v1.md` |
| verdict | CONDITIONAL PASS |
| tags | acceptance, nimbus, rust, infrastructure, ci, toolchain, verification |

## Verdict

**CONDITIONAL PASS**

Nimbus landed the intended infrastructure-only foundation slice: repo-root toolchain declaration, a local verification entrypoint, a CI workflow, and the non-root Cargo profile cleanup all exist in the expected places.

However, Lyra's local recheck found one real acceptance gap and one reproducibility gap:

1. `scripts/verify-rust-foundation.sh` fails on Lyra's seat because it shells out to bare `rustc --version` even when only the rustup-managed Cargo shim is discoverable.
2. `rust-toolchain.toml` uses a floating `stable` channel instead of freezing the verified `1.95.0` baseline.

Because the packet is materially correct but not yet fully closure-safe across seats, it is accepted only as **CONDITIONAL PASS**. A bounded Nimbus hardening follow-up is required before the Rust foundation lane can be considered fully hardened.

## Scope Reviewed

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-delivery-v1.md`
- `rust-toolchain.toml`
- `scripts/verify-rust-foundation.sh`
- `.github/workflows/rust-foundation.yml`
- `Cargo.toml`
- `src-tauri/Cargo.toml`
- `$HOME/.cargo/bin/cargo check`
- `$HOME/.cargo/bin/cargo test -p seatloom-core`
- `scripts/verify-rust-foundation.sh`

## Coverage Matrix

| Requirement slice | Evidence paths | Result | Notes |
|---|---|---|---|
| Repo-root Rust toolchain declaration exists | `rust-toolchain.toml` | CONDITIONAL PASS | The file exists, but `channel = "stable"` is not a durable freeze of the verified baseline. |
| One deterministic local verification script exists | `scripts/verify-rust-foundation.sh` | HOLD | The script exists and is mostly aligned, but it fails on Lyra's seat because `rustc` is invoked directly from `PATH`. |
| One CI workflow mirrors the bounded Rust foundation checks | `.github/workflows/rust-foundation.yml` | PASS | Push/PR workflow exists and runs `cargo check` plus `cargo test -p seatloom-core`. |
| Non-root profile warning is removed by manifest-only cleanup | `Cargo.toml`; `src-tauri/Cargo.toml`; `$HOME/.cargo/bin/cargo check` | PASS | `[profile.release]` now lives at workspace root and the ignored-profile warning is gone. |
| Workspace `cargo check` passes | `$HOME/.cargo/bin/cargo check` | PASS | Command exits `0`; only pre-existing `seatloom-tauri` dead-code warnings remain. |
| `seatloom-core` tests pass | `$HOME/.cargo/bin/cargo test -p seatloom-core` | PASS | `22` tests pass locally on Lyra's seat. |
| Packet stayed infrastructure-only and did not resume business-facing backend work | Delivery artifact; touched files above | PASS | No artifact read-model, retrieval, runtime wiring, or UI/product logic changes were introduced. |
| Paused artifact-read-model packet remains paused | Delivery artifact | PASS | `NIMBUS-2026-04-29-artifact-read-models-v1` remains inactive as directed. |

## Findings

| ID | Severity | Finding | Why it matters | Status |
|---|---|---|---|---|
| NFQA-01 | High | `scripts/verify-rust-foundation.sh` invokes bare `rustc --version`, which fails when `rustc` is not directly on `PATH` even though Cargo is available through rustup shims. | The packet's required local verification entrypoint is not actually portable across seats, so the acceptance criterion "script exists and runs" is not yet met. | Must fix |
| NFQA-02 | Medium | `rust-toolchain.toml` uses `channel = "stable"` instead of the verified toolchain version. | The foundation lane is supposed to be reproducible; a floating stable channel can drift over time and invalidate later CI/local comparisons. | Must fix |
| NFQA-03 | Low | Workspace `cargo check` still emits 13 pre-existing `seatloom-tauri` dead-code warnings. | The packet did not promise warning-zero output, so this is not a blocker here, but it is a good next infra hardening target. | Deferred into next packet |

## Required Fixes for Nimbus

1. Update `rust-toolchain.toml` to freeze the verified Rust toolchain version rather than a floating channel.
2. Repair `scripts/verify-rust-foundation.sh` so it resolves both Cargo and Rustc robustly from the active toolchain and succeeds on Lyra's seat.
3. Use the next bounded infrastructure packet to tighten foundation quality gates rather than reopening product/business work.

## Go / No-Go Recommendation

- **Close this packet at full PASS now:** **NO-GO**
- **Treat the landed scope as materially useful and continue via one bounded hardening follow-up:** **GO**
- **Resume paused business-facing backend packets before hardening closes:** **NO-GO**

## Evidence Paths

- Packet issued: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-v1.md`
- Delivery reviewed: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-quality-automation-delivery-v1.md`
- Toolchain file: `rust-toolchain.toml`
- Local script: `scripts/verify-rust-foundation.sh`
- CI workflow: `.github/workflows/rust-foundation.yml`
- Workspace manifest: `Cargo.toml`
- Tauri manifest: `src-tauri/Cargo.toml`
