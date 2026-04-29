# Delivery: Nimbus Deterministic Read-Model Repositories

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-read-model-repositories-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| task_ref | NIMBUS-2026-04-29-read-model-repositories-v1 |
| from | Nimbus |
| to | Lyra |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-read-model-repositories-v1.md` |
| tags | nimbus, rust, repository, tauri, sessions, workitems, handoffs, delegations |

## 1. Scope Completed

Implemented the first real deterministic local read-model slice for the Tauri app:

- Deterministic filesystem-backed path helpers for sessions, workitems, handoffs, and artifacts.
- Typed read-only repositories for seats, sessions, workitems, handoffs, and delegations.
- All five Tauri list commands wired to real filesystem-backed repositories.
- 13 new focused tests (22 total in seatloom-core, up from 9).

## 2. Files Changed

| File | Action | Purpose |
|---|---|---|
| `crates/seatloom-core/src/storage/project.rs` | Modified | Added path helpers for sessions, workitems, handoffs, artifacts |
| `crates/seatloom-core/src/storage/mod.rs` | Modified | Exported new store modules |
| `crates/seatloom-core/src/storage/seat_registry.rs` | Modified | Added `list_seat_identities()`, `list_delegations()`, `Io` error variant, 4 new tests |
| `crates/seatloom-core/src/storage/session_store.rs` | Created | `SessionStore` with `list_sessions()`, `load_session()`, 3 tests |
| `crates/seatloom-core/src/storage/workitem_store.rs` | Created | `WorkItemStore` with `list_workitems()`, `load_workitem()`, `save_workitem()`, 3 tests |
| `crates/seatloom-core/src/storage/handoff_store.rs` | Created | `HandoffStore` with `list_handoffs()`, `load_handoff()`, `save_handoff()`, 3 tests |
| `src-tauri/src/commands/seat_cmds.rs` | Modified | Wired to `SeatRegistry::list_seat_identities()` |
| `src-tauri/src/commands/session_cmds.rs` | Modified | Wired to `SessionStore::list_sessions()` |
| `src-tauri/src/commands/workitem_cmds.rs` | Modified | Wired to `WorkItemStore::list_workitems()` |
| `src-tauri/src/commands/handoff_cmds.rs` | Modified | Wired to `HandoffStore::list_handoffs()` |
| `src-tauri/src/commands/delegation_cmds.rs` | Modified | Wired to `SeatRegistry::list_delegations()` |

## 3. Path Helper Additions

Extended `ProjectPaths` in `project.rs` with:

| Helper | Path |
|---|---|
| `sessions_dir()` | `.seatloom/sessions/` |
| `session_dir(session_id)` | `.seatloom/sessions/<id>/` |
| `session_meta_path(session_id)` | `.seatloom/sessions/<id>/meta.yaml` |
| `workitems_dir()` | `.seatloom/workitems/` |
| `workitem_path(workitem_id)` | `.seatloom/workitems/<id>.yaml` |
| `handoffs_dir()` | `.seatloom/handoffs/` |
| `handoff_path(handoff_id)` | `.seatloom/handoffs/<id>.yaml` |
| `artifacts_dir()` | `.seatloom/artifacts/` |

All paths follow the v0.5 architecture contract (AD-003, architecture-design.md Section 6.1).

## 4. Repository Behavior by Object Family

### Seats (via `SeatRegistry`)

- **Source**: `.seatloom/seats/*/identity.yaml`
- **Method**: `list_seat_identities()`
- **Sort**: by `name` ascending (deterministic)
- **Missing directory**: returns empty `Vec`
- **Malformed YAML**: returns `SeatRegistryError::Yaml` with file path context

### Sessions (via `SessionStore`)

- **Source**: `.seatloom/sessions/*/meta.yaml`
- **Method**: `list_sessions()`
- **Sort**: by `created_at` descending (most recent first)
- **Missing directory**: returns empty `Vec`
- **Malformed YAML**: returns `SessionStoreError::Yaml` with file path context

### WorkItems (via `WorkItemStore`)

- **Source**: `.seatloom/workitems/*.yaml`
- **Method**: `list_workitems()`
- **Sort**: by `updated_at` descending (most recently updated first)
- **Missing directory**: returns empty `Vec`
- **Malformed YAML**: returns `WorkItemStoreError::Yaml` with file path context

### Handoffs (via `HandoffStore`)

- **Source**: `.seatloom/handoffs/*.yaml`
- **Method**: `list_handoffs()`
- **Sort**: by `created_at` descending (most recent first)
- **Missing directory**: returns empty `Vec`
- **Malformed YAML**: returns `HandoffStoreError::Yaml` with file path context

### Delegations (via `SeatRegistry`)

- **Source**: `.seatloom/delegations/*.yaml`
- **Method**: `list_delegations()`
- **Sort**: by `issued_at` descending (most recent first)
- **Missing directory**: returns empty `Vec`
- **Malformed YAML**: returns `SeatRegistryError::Yaml` with file path context

## 5. Command Wiring Summary

All five Tauri list commands now call real repositories instead of returning hardcoded empty vectors.

| Command | Repository | Sort |
|---|---|---|
| `list_seats()` | `SeatRegistry::list_seat_identities()` | name asc |
| `list_sessions()` | `SessionStore::list_sessions()` | created_at desc |
| `list_workitems()` | `WorkItemStore::list_workitems()` | updated_at desc |
| `list_handoffs()` | `HandoffStore::list_handoffs()` | created_at desc |
| `list_delegations()` | `SeatRegistry::list_delegations()` | issued_at desc |

**Bounded project-root assumption**: All Tauri commands use `std::env::current_dir()` as the project root. This is a bounded assumption documented here; it will be replaced by Tauri managed state in a future runtime-engine packet.

## 6. Validation Commands and Results

### `$HOME/.cargo/bin/cargo check` (workspace)

| Crate | Exit Code | Warnings | Result |
|---|---|---|---|
| seatloom-core | 0 | 0 | PASS |
| seatloom-tauri | 0 | 13 (pre-existing dead_code, out of scope) | PASS |
| seatloom-cli | 0 | 0 | PASS |

**Workspace cargo check: exit 0.**

### `$HOME/.cargo/bin/cargo test -p seatloom-core`

| Test | Result |
|---|---|
| `ledger::index::tests::rebuilds_lookup_maps_from_seeded_events` | ok |
| `storage::seat_registry::tests::resolves_seat_layer_paths_deterministically` | ok |
| `storage::seat_registry::tests::seat_identity_round_trips` | ok |
| `storage::seat_registry::tests::role_binding_round_trips` | ok |
| `storage::seat_registry::tests::delegation_round_trips` | ok |
| `storage::seat_registry::tests::delegation_validation_rejects_required_boundary_gaps` | ok |
| `storage::seat_registry::tests::save_delegation_returns_typed_validation_error` | ok |
| `storage::seat_registry::tests::missing_seats_dir_returns_empty_vec` | ok |
| `storage::seat_registry::tests::list_seat_identities_sorted_by_name` | ok |
| `storage::seat_registry::tests::missing_delegations_dir_returns_empty_vec` | ok |
| `storage::seat_registry::tests::list_delegations_sorted_by_issued_at_descending` | ok |
| `storage::session_store::tests::missing_sessions_dir_returns_empty_vec` | ok |
| `storage::session_store::tests::session_round_trip_and_list` | ok |
| `storage::session_store::tests::resolves_session_paths_deterministically` | ok |
| `storage::workitem_store::tests::missing_workitems_dir_returns_empty_vec` | ok |
| `storage::workitem_store::tests::workitem_round_trip_and_list` | ok |
| `storage::workitem_store::tests::resolves_workitem_paths_deterministically` | ok |
| `storage::handoff_store::tests::missing_handoffs_dir_returns_empty_vec` | ok |
| `storage::handoff_store::tests::handoff_round_trip_and_list` | ok |
| `storage::handoff_store::tests::resolves_handoff_paths_deterministically` | ok |
| `storage::yaml_io::tests::project_config_round_trips_through_yaml` | ok |
| `storage::jsonl_io::tests::jsonl_append_and_read_round_trip` | ok |

**22 passed, 0 failed, 0 ignored. No regression from prior 9 tests.**

## 7. Residual Notes / Explicit Non-Goals Kept Out

- **No mutation commands**: `attach_session`, `create_workitem`, handoff mutations remain as stubs. Out of scope per packet.
- **No route-engine / inbox projection**: not touched.
- **No frontend changes**: not touched.
- **No PTY / runtime wiring**: not touched.
- **No Tauri invoke_handler registration**: commands exist as library functions; actual `#[tauri::command]` registration and managed state injection are deferred to a future runtime-engine packet.
- **The `seat_id` and `active_only` filter parameters on `list_delegations()`** are accepted but not yet implemented (post-filtering). The underlying `list_delegations()` returns all delegations sorted by `issued_at` desc.
- **seatloom-tauri dead_code warnings**: 13 pre-existing warnings for unused command functions. These will resolve when commands are registered with Tauri's invoke handler in a future packet.

## 8. Recommended Next Owner

- **Lyra**: acceptance review of this delivery.
- **Nimbus**: next bounded packet per Lyra direction (runtime-engine wiring, Tauri command registration, or further repository work).
- **Mira**: frontend integration with these read-model commands when Tauri IPC is wired.
