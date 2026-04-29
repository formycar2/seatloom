# NIMBUS-2026-04-29-seat-registry-delegation-storage-delivery-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-seat-registry-delegation-storage-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-seat-registry-delegation-storage-v1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-storage-ledger-foundation-acceptance.md` |
| supersedes | - |
| tags | architecture, seat, delegation, storage, rust, nimbus |

## 1. Scope completed

Implemented the deterministic seat-layer persistence slice inside the declared packet boundary only.

Completed scope:

- extended `.seatloom` path helpers for seat identities, project role bindings, and delegations,
- added a small `SeatRegistry` repository surface for `SeatIdentity`, `ProjectRoleBind`, and `SeatDelegation`,
- added typed delegation validation at the storage boundary,
- added focused tests for path resolution, identity round-trip, role-binding round-trip, delegation round-trip, and validation failure.

Not expanded:

- no retrieval work,
- no Tauri IPC wiring,
- no route/gate logic,
- no UI work,
- no runtime orchestration.

## 2. File-by-file change summary

- `crates/seatloom-core/src/storage/project.rs`
  - extended `ProjectPaths` with deterministic seat-layer paths:
    - `seats/<seat-name>/identity.yaml`
    - `seats/<seat-name>/role-bindings/<project>.yaml`
    - `delegations/<delegation-id>.yaml`
- `crates/seatloom-core/src/storage/mod.rs`
  - exported the new `seat_registry` module.
- `crates/seatloom-core/src/storage/seat_registry.rs`
  - added `SeatRegistry` repository helpers for save/load of `SeatIdentity`, `ProjectRoleBind`, and `SeatDelegation`;
  - added `SeatRegistryError` and `SeatRegistryValidationError`;
  - added deterministic delegation validation and focused tests.

## 3. Path and repository behavior implemented

### Path helpers

`ProjectPaths` now resolves the storage contract required by `AD-009` and the architecture design:

- `seat_identity_path(seat_name)` → `.seatloom/seats/<seat-name>/identity.yaml`
- `seat_role_binding_path(seat_name, project_id)` → `.seatloom/seats/<seat-name>/role-bindings/<project>.yaml`
- `delegation_path(delegation_id)` → `.seatloom/delegations/<delegation-id>.yaml`

### Repository surface

`SeatRegistry` centralizes seat-layer persistence so downstream code no longer needs to hand-build seat paths:

- `save_identity()` / `load_identity()`
- `save_role_binding()` / `load_role_binding()`
- `save_delegation()` / `load_delegation()`

All persistence uses the existing typed YAML helpers and therefore keeps parent-directory creation, atomic rename writes, and typed IO failures at the file boundary.

## 4. Validation rules implemented

The storage boundary now rejects the required invalid delegation shapes with typed errors:

- `MissingScope`
- `MissingIssuer`
- `MissingTargetSeat`
- `ActiveDelegationMissingExpiry`

Validation stays deterministic and local to storage/repository rules only:

- scope must not be blank,
- issuer seat id must not be blank,
- target seat id must not be blank,
- an `Active` delegation must carry an expiry.

This does not widen into route, policy, or runtime logic.

## 5. Validation result

Focused tests were added in code for:

- deterministic path resolution,
- seat identity round-trip,
- role binding round-trip,
- delegation round-trip,
- delegation validation failure.

Local execution is still blocked on the Nimbus seat.

Attempted commands and exact result:

```bash
$ cargo test -p seatloom-core
bash: cargo: command not found

$ cargo check -p seatloom-core
bash: cargo: command not found

$ source "$HOME/.cargo/env" && cargo check -p seatloom-core
error: rustup could not choose a version of cargo to run, because one wasn't specified explicitly, and no default is configured.
help: run 'rustup default stable' to download the latest stable release of Rust and set it as your default toolchain.

$ which cargo
/Users/jyxc-dz-0100609/.cargo/bin/cargo
```

Interpretation: the seat exposes a rustup cargo shim, but no usable default toolchain is configured, so compile/test execution still cannot run locally.

## 6. Blockers

- `ENV-001` — Nimbus seat still cannot execute `cargo test` / `cargo check` successfully because no usable default Rust toolchain is configured. The packet is structurally complete; compile-capable verification must happen on a Rust-capable seat or CI.

## 7. Evidence paths

- Packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-seat-registry-delegation-storage-v1.md`
- Product truth entrypoint: `docs/PRODUCT_TRUTH.md`
- PRD seat stories: `docs/prd-v0.5.md` (`US-P0-03`, `US-P0-04`)
- Interaction contract: `docs/interaction-spec-v1.1.md` (`INT-03`, `INT-04`)
- Architecture decision: `docs/architecture-decisions.md` (`AD-009`)
- Architecture storage model: `docs/architecture-design.md` §6.1-§6.3
- Accepted prior packet: `docs/coordination/acceptance/2026-04-29-lyra-nimbus-storage-ledger-foundation-acceptance.md`
- Deterministic path helpers: `crates/seatloom-core/src/storage/project.rs`
- Storage module export: `crates/seatloom-core/src/storage/mod.rs`
- Seat registry repository and validation: `crates/seatloom-core/src/storage/seat_registry.rs`
