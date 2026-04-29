# Task: Nimbus Storage + Ledger Foundation

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-storage-ledger-foundation-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/acceptance/2026-04-28-lyra-nimbus-foundation-scaffold-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | architecture, foundation, storage, ledger, rust, nimbus |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One active engineering packet only. Do not branch into runtime wiring, adapter behavior, or UI work inside this packet. |

## Objective

Build the first real deterministic persistence foundation on top of the accepted scaffold.

This packet exists to turn the closed type scaffold into auditable project IO primitives without expanding into runtime orchestration. The target is durable config and ledger read/write behavior that later retrieval, IPC, and mobile/desktop truth can rely on.

## Input Files

Mandatory reads, in order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`
7. `docs/coordination/acceptance/2026-04-28-lyra-nimbus-foundation-scaffold-acceptance.md`
8. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
9. `docs/coordination/COORDINATION_RULES.md`

## Need-to-Know Scope

Read only:

- this packet
- the input files above
- the exact Rust files you edit

Do not pull in by default:

- unrelated frontend implementation files
- archived product docs as authority
- broad tmux history or old review chatter

If blocked, report one compact blocker with the exact file path and missing truth.

## Write Boundary

Primary targets:

- `crates/seatloom-core/src/storage/project.rs`
- `crates/seatloom-core/src/storage/yaml_io.rs`
- `crates/seatloom-core/src/storage/jsonl_io.rs`
- `crates/seatloom-core/src/ledger/writer.rs`
- `crates/seatloom-core/src/ledger/reader.rs`
- `crates/seatloom-core/src/ledger/index.rs`

Allowed support targets only if required for compile-safe integration:

- `crates/seatloom-core/src/lib.rs`
- `crates/seatloom-core/src/ledger/mod.rs`
- `crates/seatloom-core/src/storage/mod.rs`
- `crates/seatloom-core/Cargo.toml`
- `Cargo.toml`

## Required Outcome

### 1. Real storage helpers for authoritative project files

Implement real file IO helpers for the project-local authority files described by the architecture:

- YAML read/write for project config and seat profile style records
- JSONL append/read helpers for Ledger-style event persistence
- create-parent-directory behavior where needed
- deterministic error propagation; do not hide IO failures behind generic booleans

### 2. Real Ledger append/read primitives

Replace stub behavior with the smallest useful deterministic implementation for:

- append one canonical event per line
- sequential read of stored events
- basic index rebuild in memory from persisted events

Boundary:

- in-memory index is acceptable here as a rebuild/view cache
- do not implement SQLite retrieval yet in this packet
- do not add semantic or LLM-facing behavior

### 3. Keep the packet audit-safe

- preserve event-first review/reissue assumptions
- preserve `template+subtype` fields in persisted artifacts/events where applicable
- do not invent new object states or product semantics

### 4. Unit-test what is local and deterministic

If the local environment supports it, add focused Rust tests for:

- YAML round-trip for one config type
- JSONL append + read round-trip
- Ledger index rebuild from a small seeded event list

If the seat environment still lacks Rust tooling, keep the tests in code if feasible and record the exact verification blocker without widening scope.

## Non-goals

- No Tauri IPC wiring
- No adapter implementation
- No runtime/session launch behavior
- No retrieval engine implementation beyond what local Ledger read/index needs
- No UI work

## Done Definition

- [ ] `storage/yaml_io.rs` performs real structured file IO.
- [ ] `storage/jsonl_io.rs` performs real append/read behavior for JSONL.
- [ ] `ledger/writer.rs`, `ledger/reader.rs`, and `ledger/index.rs` are no longer stub-only.
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

If the seat still lacks Rust tooling, run and record the exact failure:

```bash
source "$HOME/.cargo/env" && cargo check -p seatloom-core
which cargo
```

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-storage-ledger-foundation-delivery-v1.md`

Required sections:

1. Scope completed
2. File-by-file change summary
3. IO behavior implemented
4. Ledger behavior implemented
5. Validation result
6. Blockers
7. Evidence paths

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_storage_ledger.txt
[Nimbus -> Lyra] Storage + Ledger Foundation
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
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-storage-ledger-foundation-delivery-v1.md
- ...
MSG

tmux load-buffer -b nimbus_to_lyra_storage_ledger /tmp/nimbus_to_lyra_storage_ledger.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_storage_ledger
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
