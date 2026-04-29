# Task: ENV-001 Compile-Capable Verification

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-04-29-env-001-compile-verification-v1 |
| status | issued |
| author | aegis |
| date | 2026-04-29 |
| version | v1 |
| to | flux |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/MEMORY.md`, `docs/coordination/memory/2026-04-29.md`, `docs/coordination/reviews/2026-04-29-product-baseline-freeze-stage-review.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-storage-ledger-foundation-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-seat-registry-delegation-storage-acceptance.md` |
| tags | flux, verification, env-001, rust, compile, ci |
| owner | Flux |
| acceptance owner | Lyra |
| concurrency rule | One active verification packet only. No feature implementation. No scope widening beyond compile verification. |

## Objective

Close `ENV-001` by running the first real Rust compile pass on the accepted Nimbus foundation scaffold, storage + ledger foundation, and seat-registry + delegation-storage slices.

The current Nimbus seat lacks `cargo` / `rustc`. This packet routes compile verification to a compile-capable path: CI-first, with a Rust-capable fallback seat if CI is unavailable.

## Background

Nimbus has delivered and Lyra has accepted three bounded Rust engineering packets:

1. Foundation scaffold (workspace, core types, enums, event families, Tauri command stubs)
2. Storage + ledger foundation (deterministic YAML/JSONL project IO, Ledger primitives)
3. Seat registry + delegation storage (durable seat identity, project role binding, scoped delegation persistence)

All three were accepted as scope-complete but remain environment-unverified because the Nimbus seat has no Rust toolchain. Product Baseline Freeze is now `GO`, but downstream implementation packets will accumulate unverifiable Rust assumptions if `ENV-001` is not closed.

## Input Files

Read only these inputs before executing:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/coordination/MEMORY.md`
3. `docs/coordination/memory/2026-04-29.md`
4. `docs/coordination/reviews/2026-04-29-product-baseline-freeze-stage-review.md`
5. this packet

## Need-to-Know Scope

Read only:

- the input files above
- `src-tauri/` workspace: `Cargo.toml`, `Cargo.lock`, and Rust source files needed for compile verification
- exact config or toolchain files needed to run the required commands

Do not read:

- UI source (`ui/`) unless a compile error directly references it
- archived product docs
- old acceptance or task packets beyond those listed in `depends_on`

If a check cannot be completed, mark it explicitly with the exact blocking condition instead of widening scope.

## Write Boundary

Allowed write locations only:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-env-001-compile-verification-delivery-v1.md`
- `.local/evidence/2026-04-29-env-001-compile-verification/**`

## Execution Path

### Primary path: CI

If a CI lane with Rust toolchain is available:

1. Run the required commands (see below) in CI.
2. Capture all output logs under `.local/evidence/2026-04-29-env-001-compile-verification/`.
3. Write the delivery artifact.

### Fallback path: Rust-capable seat

If CI is not available or not configured:

1. Execute on any seat or environment where `rustc` and `cargo` are installed.
2. Run the required commands.
3. Capture all output logs under `.local/evidence/2026-04-29-env-001-compile-verification/`.
4. Write the delivery artifact.

## Required Commands

Execute in this exact order and capture full stdout/stderr for each:

```bash
# 1. Toolchain version capture
rustc --version
cargo --version

# 2. Workspace compile check (type-check only, no binary output)
cd src-tauri && cargo check 2>&1

# 3. Test suite (bounded subset acceptable with explicit scope statement)
cd src-tauri && cargo test 2>&1
```

If `cargo test` cannot run the full suite (e.g., missing test fixtures, external dependencies, or Tauri build-time requirements), run a bounded subset and state the exact scope in the delivery artifact:

```bash
# Bounded alternative if full suite fails
cd src-tauri && cargo test --lib 2>&1
```

## Required Evidence Output

All command output must be captured under:

```
.local/evidence/2026-04-29-env-001-compile-verification/
  rustc-version.txt
  cargo-version.txt
  cargo-check.txt
  cargo-test.txt
```

Each file must contain the full command invocation and complete stdout/stderr output.

## Done Definition

- [ ] `rustc --version` output captured and recorded.
- [ ] `cargo --version` output captured and recorded.
- [ ] `cargo check` result is PASS (exit code 0) or FAIL with exact error output.
- [ ] `cargo test` (or bounded subset with explicit scope statement) result is PASS or FAIL with exact error output.
- [ ] All evidence files are written under `.local/evidence/2026-04-29-env-001-compile-verification/`.
- [ ] Delivery artifact is written at `docs/coordination/tasks/flux/FLUX-2026-04-29-env-001-compile-verification-delivery-v1.md`.
- [ ] Delivery artifact contains a clear `ENV-001` verdict: `CLOSED` (all pass) or `RE-SCOPED` (with concrete failures and required follow-up).

## Pass/Fail Criteria

### PASS (ENV-001 CLOSED)

- `cargo check` exits 0.
- `cargo test` (or bounded subset) exits 0 with no test failures.
- All evidence files exist and contain complete output.

### FAIL (ENV-001 RE-SCOPED)

- Any required command exits non-zero.
- Delivery artifact must list:
  - exact failing command(s)
  - exact error message(s)
  - root cause assessment (missing dependency, type error, test assertion, Tauri build requirement, etc.)
  - recommended owner and bounded fix scope for each failure

A `FAIL` verdict does not invalidate the accepted Nimbus packets. It surfaces concrete compile issues that must be fixed in a separate bounded follow-up packet before downstream implementation can depend on compile-safe assumptions.

## Non-Goals

- No feature implementation of any kind.
- No Rust code changes. If code changes are needed to pass compilation, document the required changes and return them as findings, do not patch.
- No UI work.
- No product-contract or design-doc modifications.
- No scope widening beyond the four required commands.
- No reopening of accepted Nimbus scaffold, storage, or seat-registry packets.
- No architecture decisions or new ADRs.

## Acceptance Spec Reference

- `docs/acceptance-spec-v1.1.md` Section 6 (`E-01` through `E-11`) for engineering acceptance principles
- `docs/coordination/reviews/2026-04-29-product-baseline-freeze-stage-review.md` Section "ENV-001 routing" for the stage-review mandate

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/flux/FLUX-2026-04-29-env-001-compile-verification-delivery-v1.md`

Required sections:

1. ENV-001 verdict (`CLOSED` or `RE-SCOPED`)
2. Execution environment (OS, rustc version, cargo version, execution path used)
3. Command results matrix (command, exit code, pass/fail, evidence path)
4. Failures detail (if any: exact errors, root cause, recommended fix owner)
5. Evidence file paths
6. Recommended next action
