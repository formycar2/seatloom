# Task: Nimbus Seat Registry + Delegation Storage

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-seat-registry-delegation-storage-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-storage-ledger-foundation-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | architecture, seat, delegation, storage, rust, nimbus |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One active engineering packet only. Do not branch into retrieval, IPC wiring, route/gate engines, or UI work inside this packet. |

## Objective

Build the next deterministic storage slice on top of the accepted YAML/JSONL foundation.

The product contract now depends on three seat-layer truths being durable and separable:

1. global seat identity,
2. project-local role binding,
3. scoped delegation overlay.

This packet turns those architecture objects into real project-local persistence primitives without widening into routing or runtime behavior.

## Input Files

Mandatory reads, in order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md` (`US-P0-03`, `US-P0-04`)
3. `docs/interaction-spec-v1.1.md` (`INT-03`, `INT-04`)
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md` (`AD-009`)
6. `docs/architecture-design.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-nimbus-storage-ledger-foundation-acceptance.md`
8. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
9. `docs/coordination/COORDINATION_RULES.md`
10. this packet

## Need-to-Know Scope

Read only:

- this packet
- the input files above
- the exact Rust files you edit

Do not pull in by default:

- frontend implementation files
- archived product docs as authority
- retrieval design beyond what this packet directly needs

If blocked, report one compact blocker with the exact file path and missing truth.

## Write Boundary

Primary targets:

- `crates/seatloom-core/src/storage/project.rs`
- `crates/seatloom-core/src/storage/mod.rs`
- `crates/seatloom-core/src/storage/seat_registry.rs` *(new file allowed)*

Optional support targets only if required for compile-safe integration:

- `crates/seatloom-core/src/objects/seat.rs`
- `crates/seatloom-core/src/lib.rs`
- `crates/seatloom-core/Cargo.toml`

## Required Outcome

### 1. Deterministic path helpers for seat-layer storage

Add or extend path helpers so the project model can address:

- `seats/<seat-name>/identity.yaml`
- `seats/<seat-name>/role-bindings/<project>.yaml`
- `delegations/<delegation-id>.yaml`

Rules:

- keep path generation deterministic and local to the accepted `.seatloom` structure;
- do not invent alternate storage roots.

### 2. Real persistence helpers for seat-layer objects

Implement real read/write helpers or a small repository surface for:

- `SeatIdentity`
- `ProjectRoleBind`
- `SeatDelegation`

The result should let downstream code persist and reload these objects without hand-writing path logic everywhere.

### 3. Validation at the persistence boundary

Add typed validation errors for the storage/repository boundary where the contract requires them.

Minimum validation coverage:

- delegation cannot be saved without scope,
- delegation cannot be saved without issuer,
- delegation cannot be saved without target seat,
- active delegation cannot be saved without expiry.

Keep this validation deterministic and typed. Do not widen into policy engines.

### 4. Focused local tests

If local Rust tooling is usable, add focused tests for:

- path resolution,
- seat identity round-trip,
- role binding round-trip,
- delegation round-trip / validation failure.

If Rust tooling remains unavailable, keep the tests in code where feasible and record the exact validation blocker without widening scope.

## Non-goals

- No Tauri IPC commands
- No route engine logic
- No retrieval or SQLite work
- No UI work
- No runtime/session orchestration

## Done Definition

- [ ] Seat-layer path helpers exist for identity, role binding, and delegation storage.
- [ ] Real read/write behavior exists for `SeatIdentity`, `ProjectRoleBind`, and `SeatDelegation`.
- [ ] Typed validation errors cover the required delegation boundary rules.
- [ ] Changes stay inside the declared write boundary.
- [ ] Verification command(s) and exact result are recorded, including `ENV-001` if still present.
- [ ] Delivery artifact is written at the required path.
- [ ] tmux reply is sent to Lyra after the artifact is written.

## Validation

Preferred commands if the seat supports Rust:

```bash
cargo test -p seatloom-core 2>&1
cargo check -p seatloom-core 2>&1
```

If the seat still lacks usable Rust tooling, run and record the exact failure:

```bash
source "$HOME/.cargo/env" && cargo check -p seatloom-core
which cargo
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-seat-registry-delegation-storage-delivery-v1.md`

Required sections:

1. Scope completed
2. File-by-file change summary
3. Path and repository behavior implemented
4. Validation rules implemented
5. Validation result
6. Blockers
7. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_seat_registry_delegation.txt
[Nimbus -> Lyra] Seat Registry + Delegation Storage
completed:
- ...
validation:
- `cargo test -p seatloom-core` => ...
- `cargo check -p seatloom-core` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-seat-registry-delegation-storage-delivery-v1.md
- ...
MSG

tmux load-buffer -b nimbus_to_lyra_seat_registry_delegation /tmp/nimbus_to_lyra_seat_registry_delegation.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_seat_registry_delegation
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
