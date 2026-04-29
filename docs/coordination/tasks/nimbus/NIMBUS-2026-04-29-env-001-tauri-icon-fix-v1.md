# Task: ENV-001 Tauri Icon Asset Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | fix |
| id | NIMBUS-2026-04-29-env-001-tauri-icon-fix-v1 |
| status | issued |
| author | aegis |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-29-env-001-compile-verification-delivery-v1.md`, `docs/coordination/acceptance/2026-04-29-aegis-env-001-compile-verification-acceptance.md` |
| tags | nimbus, fix, env-001, tauri, icon |
| owner | Nimbus |
| acceptance owner | Lyra |

## Objective

Provide the missing Tauri icon asset so that workspace-level `cargo check` passes and `ENV-001` can be fully closed.

## Background

The ENV-001 compile verification found that `seatloom-core` compiles and passes all 9 tests, but `seatloom-tauri` fails at `cargo check` because `tauri::generate_context!()` requires `src-tauri/icons/icon.png` which does not exist.

## Scope

1. Add a valid PNG icon file at `src-tauri/icons/icon.png`.
2. Ensure any other icon paths referenced in `src-tauri/tauri.conf.json` also have valid files.
3. Optionally: remove the unused `PathBuf` import from `crates/seatloom-core/src/storage/seat_registry.rs:1`.

## Done Definition

- [ ] `src-tauri/icons/icon.png` exists and is a valid PNG.
- [ ] `cargo check` (workspace-level) exits 0.
- [ ] `cargo test -p seatloom-core` still passes (no regression).

## Non-Goals

- No feature implementation.
- No architecture changes.
- No scope widening beyond the icon asset and optional warning cleanup.

## Acceptance Spec Reference

- `docs/coordination/acceptance/2026-04-29-aegis-env-001-compile-verification-acceptance.md`
