# Task: Nimbus Deterministic Read-Model Repositories

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-read-model-repositories-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-storage-ledger-foundation-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-seat-registry-delegation-storage-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | nimbus, rust, repository, tauri, sessions, workitems, handoffs, delegations |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded engineering packet. You are not alone in the codebase; do not revert others' work, and do not widen into runtime, PTY, route-engine, or UI work. |

## Objective

Implement the first real deterministic local read-model slice for the Tauri app.

The goal is to replace the current empty list stubs for project objects with actual filesystem-backed repositories and read-only Tauri commands for:

- seats
- sessions
- workitems
- handoffs
- delegations

This packet is intentionally read-heavy and mutation-light: no attach/create transitions yet.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`
7. `docs/coordination/acceptance/2026-04-29-lyra-nimbus-storage-ledger-foundation-acceptance.md`
8. `docs/coordination/acceptance/2026-04-29-lyra-nimbus-seat-registry-delegation-storage-acceptance.md`
9. this packet

## Need-to-Know Scope

Implement only deterministic local listing / loading for the existing object model.

In scope:

- path helpers under `.seatloom/`
- typed read repositories for YAML-backed objects
- stable empty-directory behavior
- deterministic list ordering
- Tauri read-only list commands wired to real repositories
- focused tests for path resolution and repository behavior

Out of scope:

- attach session
- create workitem
- handoff mutations
- route-engine / inbox projection
- timeline query
- prompt engine
- PTY runtime wiring
- any frontend changes

## Write Boundary

Primary targets:

- `crates/seatloom-core/src/storage/project.rs`
- `crates/seatloom-core/src/storage/mod.rs`
- `crates/seatloom-core/src/storage/seat_registry.rs`
- `src-tauri/src/commands/seat_cmds.rs`
- `src-tauri/src/commands/session_cmds.rs`
- `src-tauri/src/commands/workitem_cmds.rs`
- `src-tauri/src/commands/handoff_cmds.rs`
- `src-tauri/src/commands/delegation_cmds.rs`

You may add new bounded repository files under `crates/seatloom-core/src/storage/` if that makes the design cleaner, for example:

- `session_store.rs`
- `workitem_store.rs`
- `handoff_store.rs`
- `read_models.rs`

If you add files, export them in `crates/seatloom-core/src/storage/mod.rs` and keep the scope repository-only.

## Required Outcome

### 1. Add deterministic path helpers for remaining local object families

Extend `ProjectPaths` so the repository layer can resolve, at minimum:

- sessions root and per-session `meta.yaml`
- workitems root and per-workitem file path
- handoffs root and per-handoff file path
- artifacts root if needed for repository completeness

Do not invent a second path convention; use the v0.5 / architecture contract as the authority.

### 2. Implement real read-only repositories

Provide typed repository helpers that can:

- list seat identities from `.seatloom/seats/*/identity.yaml`
- list sessions from `.seatloom/sessions/*/meta.yaml`
- list workitems from `.seatloom/workitems/*.yaml`
- list handoffs from `.seatloom/handoffs/*.yaml`
- list delegations from `.seatloom/delegations/*.yaml`

Repository rules:

- missing directories return empty collections, not errors;
- malformed YAML returns typed errors with path context;
- list order must be deterministic and documented in code.

Recommended sort rules:

- seats: by `name` ascending
- sessions: by `created_at` descending
- workitems: by `updated_at` descending
- handoffs: by `created_at` descending
- delegations: by `issued_at` descending

### 3. Wire read-only Tauri commands to real data

Replace the current empty-command stubs for:

- `list_seats()`
- `list_sessions()`
- `list_workitems()`
- `list_handoffs()`
- `list_delegations()`

The commands may use a bounded project-root assumption if needed for this packet, but document it clearly in the delivery artifact.

### 4. Add focused tests

At minimum, add tests for:

- deterministic path resolution for any new path helpers;
- missing-directory => empty-list behavior;
- one real round-trip / seeded-read case for each new repository;
- deterministic ordering where meaningful.

## Acceptance Criteria

1. No list command above returns a hardcoded empty vector anymore when matching files exist on disk.
2. Missing `.seatloom/<family>/` directories return empty vectors rather than panics or opaque errors.
3. Repository read failures include typed path context.
4. Read ordering is deterministic and stated in code or delivery notes.
5. `$HOME/.cargo/bin/cargo check` passes.
6. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
7. Scope stays read-model only; no mutation or route-engine work is widened into this packet.

## Required Validation

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
```

If workspace-level `cargo check` produces pre-existing warnings outside this packet, record them but do not widen scope to clean them up unless they block the read-model work.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-read-model-repositories-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Path helper additions
4. Repository behavior by object family
5. Command wiring summary
6. Validation commands and results
7. Residual notes / explicit non-goals kept out
8. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] Real repositories exist for the scoped object families.
- [ ] Read-only Tauri list commands no longer return stubbed empty vectors for populated directories.
- [ ] `$HOME/.cargo/bin/cargo check` passes.
- [ ] `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_read_model_repositories.txt
[Nimbus -> Lyra] Read-Model Repositories
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-read-model-repositories-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_read_model_repositories /tmp/nimbus_to_lyra_read_model_repositories.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_read_model_repositories
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
