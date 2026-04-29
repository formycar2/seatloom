# NIMBUS-2026-04-28-Foundation-Scaffold-v1

| Field | Value |
|---|---|
| Owner | Nimbus |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-29 14:00 |
| Acceptance owner | Lyra |
| Scope | Foundation-only engineering scaffold |
| Execution mode | Need-to-know / token-efficient |

## 1. Micro-brief

1. The architecture-definition baseline is accepted; this packet starts the **code-facing foundation scaffold** only.
2. Scope is non-user-facing foundation: workspace skeleton, core object types, IDs, enums, event types, and minimal compile-safe module wiring.
3. Do not reinterpret product meaning and do not expand into full runtime behavior.
4. Keep implementation aligned to the accepted architecture docs and the active v0.5 contract set.
5. Persist the delivery artifact in English; keep terminal/tmux progress summaries in Chinese.

## 2. Decisions already frozen

- `docs/PRODUCT_TRUTH.md` is the only entrypoint.
- Architecture authority:
  - `docs/architecture-decisions.md`
  - `docs/architecture-design.md`
- Accepted constraints already locked:
  - `SQLite FTS5` is the P0 retrieval backend for L1/L2
  - review/reissue stays event-first
  - `template+subtype` is the canonical Artifact classification key
  - prompt handling is first-class in architecture, but behavior implementation is out of scope unless required for compile-safe types

## 3. Contract pack

Mandatory reads, in this order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`
7. `docs/coordination/acceptance/2026-04-28-lyra-architecture-prompt-retrieval-delta-acceptance.md`
8. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
9. `docs/coordination/COORDINATION_RULES.md`

## 4. Need-to-know scope

Read only:

- this packet
- the contract pack above
- the exact Rust/Tauri files you create or modify

Do not consume by default:

- archived product docs as primary authority
- unrelated UI review threads
- broad frontend implementation details

If blocked, escalate one compact blocker with the exact file path and missing truth.

## 5. Required work

| Priority | Requirement | Expected boundary |
|---|---|---|
| P0 | Scaffold the Rust/Tauri workspace described by the accepted architecture | workspace root, `crates/seatloom-core`, `src-tauri`, compile-safe module wiring |
| P0 | Add core object type modules | IDs, SeatIdentity / ProjectRoleBind / SeatDelegation, Session, WorkItem, Artifact, Handoff, Checkpoint, Project config types |
| P0 | Add canonical enums / event families required by the accepted architecture | `ArtifactTemplate`, subtype-aware Artifact fields, event-first review/reissue events, prompt-state enums, delegation events |
| P0 | Add module skeletons for the accepted subsystem layout | `objects/`, `ledger/`, `data_engine/`, `adapter/`, `storage/`, minimal stubs only where needed to compile |
| P0 | Keep retrieval / prompt / route / gate logic at interface or placeholder level unless compile requires more | no full engine behavior in this packet |
| P1 | Add minimal comments where the architecture intent would otherwise be hard to read | comments should explain boundaries, not restate syntax |

## 6. Non-goals

- No UI work
- No end-to-end runtime integrations
- No full storage implementation beyond what is needed for compile-safe scaffolding
- No retrieval engine behavior beyond interfaces / placeholders
- No Tauri command behavior beyond minimal compile-safe stubs

## 7. Done definition

All items below must be true:

- The scaffold exists in code, not only in docs.
- Core type names and fields align with the accepted architecture docs.
- The workspace can run a local compile check (`cargo check` or equivalent) and the result is recorded.
- The delivery artifact lists changed files, compile result, and any intentionally deferred stubs.
- No user-facing behavior is claimed complete by this packet.

## 8. Required delivery artifact

Publish completion as:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-28-foundation-scaffold-delivery-v1.md`

Required sections:

1. Micro-brief
2. Changed files / scaffold tree
3. Type coverage matrix
4. Compile command + result
5. Deferred stubs
6. Blockers / next owner

## 9. Reporting rules

- Durable artifact: English only.
- Terminal/tmux progress summary: Chinese only.
- Keep the delivery delta-based and file-first.
- Use exact file paths and compile commands.
