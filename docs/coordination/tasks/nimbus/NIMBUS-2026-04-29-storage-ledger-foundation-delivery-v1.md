# NIMBUS-2026-04-29-storage-ledger-foundation-delivery-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-storage-ledger-foundation-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-29 |
| version | v1 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-storage-ledger-foundation-v1.md`, `docs/PRODUCT_TRUTH.md`, `docs/architecture-design.md`, `docs/architecture-decisions.md` |
| supersedes | - |
| tags | architecture, foundation, storage, ledger, rust, nimbus |

## 1. Scope completed

Implemented the first deterministic persistence slice inside the packet boundary only.

Completed scope:

- real YAML read/write helpers for authoritative project-local files,
- real JSONL append/read helpers for Ledger-style event persistence,
- real `LedgerWriter` append behavior,
- real `LedgerReader` sequential replay behavior,
- real in-memory `LedgerIndex` rebuild behavior,
- focused local unit tests for YAML round-trip, JSONL round-trip, and index rebuild.

Not expanded:

- no Tauri IPC wiring,
- no runtime/session orchestration,
- no adapter behavior,
- no SQLite retrieval implementation,
- no UI work.

## 2. File-by-file change summary

- `crates/seatloom-core/src/storage/project.rs`
  - added `created_at` to `ProjectConfig` so project YAML matches the architecture contract;
  - added default values for pack/pipeline budgets and timeouts;
  - added `ProjectPaths` helpers for deterministic `.seatloom/config/project.yaml` and `.seatloom/ledger/events.jsonl` paths.
- `crates/seatloom-core/src/storage/yaml_io.rs`
  - replaced stubbed read/write functions with real YAML file IO;
  - added explicit `YamlIoError` variants for read/parse/create-parent/serialize/write/sync/persist failures;
  - implemented atomic temp-file rename on write and a focused YAML round-trip test.
- `crates/seatloom-core/src/storage/jsonl_io.rs`
  - replaced stubbed append helper with real append + newline + flush + sync behavior;
  - added sequential JSONL reader with per-line parse errors;
  - added `JsonlIoError` and a JSONL append/read round-trip test.
- `crates/seatloom-core/src/ledger/writer.rs`
  - replaced no-op append with real `CanonicalEvent` append through `append_jsonl`.
- `crates/seatloom-core/src/ledger/reader.rs`
  - replaced empty stub with real sequential replay through `read_jsonl`;
  - treats a missing ledger file as an empty ledger baseline instead of a crash path.
- `crates/seatloom-core/src/ledger/index.rs`
  - replaced vector-only stub with rebuildable in-memory indexes keyed by event id, event type, and object ref;
  - added lookup helpers and a seeded rebuild test.

## 3. IO behavior implemented

### YAML

- `read_yaml<T>` now loads file content and returns typed parse errors with the concrete file path.
- `write_yaml<T>` now:
  1. creates parent directories when needed,
  2. serializes to YAML,
  3. writes to a sibling temp file,
  4. flushes and `sync_all()` the temp file,
  5. renames the temp file into place.

This matches the architecture requirement for atomic YAML writes and deterministic failure propagation.

### JSONL

- `append_jsonl<T>` now:
  1. creates parent directories when needed,
  2. opens the target in append mode,
  3. writes one JSON record,
  4. appends exactly one newline,
  5. flushes and `sync_data()` the file.
- `read_jsonl<T>` now reads line-by-line, skips blank lines, and returns a line-numbered parse error if one record is malformed.

## 4. Ledger behavior implemented

- `LedgerWriter::append()` now persists one `CanonicalEvent` per line to `events.jsonl`.
- `LedgerReader::read_all()` now replays persisted events in stored order.
- `LedgerReader::read_all()` returns `Ok(vec![])` when the ledger file does not exist yet, which gives a deterministic empty-ledger baseline for a new project.
- `LedgerIndex::rebuild_from()` now rebuilds an in-memory projection over persisted events with lookups for:
  - event id,
  - event type,
  - object reference.

This keeps the packet inside the architecture boundary: an in-memory cache/projection only, without implementing SQLite retrieval in this slice.

## 5. Validation result

Focused tests were added in code for:

- YAML round-trip (`storage/yaml_io.rs`),
- JSONL append/read round-trip (`storage/jsonl_io.rs`),
- Ledger index rebuild (`ledger/index.rs`).

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

Interpretation: a Rust shim exists on this seat, but no usable default toolchain is configured, so compile/test execution remains unavailable locally.

## 6. Blockers

- `ENV-001` — Nimbus seat still cannot run `cargo test` / `cargo check` successfully because no usable default Rust toolchain is configured. This packet stays static and auditable; compile verification must happen on a Rust-capable seat or CI.

## 7. Evidence paths

- Packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-storage-ledger-foundation-v1.md`
- Product truth entrypoint: `docs/PRODUCT_TRUTH.md`
- Architecture decisions: `docs/architecture-decisions.md`
- Architecture design: `docs/architecture-design.md`
- Project config paths and schema: `crates/seatloom-core/src/storage/project.rs`
- YAML IO implementation: `crates/seatloom-core/src/storage/yaml_io.rs`
- JSONL IO implementation: `crates/seatloom-core/src/storage/jsonl_io.rs`
- Ledger writer: `crates/seatloom-core/src/ledger/writer.rs`
- Ledger reader: `crates/seatloom-core/src/ledger/reader.rs`
- Ledger index: `crates/seatloom-core/src/ledger/index.rs`
