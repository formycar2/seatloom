# Task: Nimbus Artifact Read Models + Dual-Key Filters

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-artifact-read-models-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `docs/coordination/acceptance/2026-04-28-lyra-artifact-objectization-baseline-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-read-model-repositories-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | nimbus, rust, artifact, repository, template, subtype, tauri |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded engineering packet. You are not alone in the codebase; do not revert others' work, and do not widen into PTY/runtime, route-engine, prompt engine, or UI work. |

## Objective

Implement the first real artifact read-model slice for SeatLoom.

The current backend still treats artifact commands as stubs even though the product contract requires typed artifact objects with `template+subtype` metadata, deterministic family/subtype filtering, and direct artifact openability.

This packet should make artifact metadata queryable without touching full-text retrieval or UI rendering.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`
7. `docs/coordination/DOCUMENT_TEMPLATES.md`
8. `docs/coordination/acceptance/2026-04-28-lyra-artifact-objectization-baseline-acceptance.md`
9. `docs/coordination/acceptance/2026-04-29-lyra-nimbus-read-model-repositories-acceptance.md`
10. this packet

## Need-to-Know Scope

Implement only deterministic artifact metadata reading and filtering.

In scope:

- deterministic artifact path helpers under `.seatloom/artifacts/`
- typed artifact repository reading `meta.yaml`
- optional `payload.md` / payload-path resolution only as metadata output, not body indexing
- list + load for artifact objects
- deterministic filtering by `template`, `subtype`, and `workitem_id`
- bounded subtype validation against the current template/subtype allow-list contract
- Tauri command library functions for artifact listing/loading/validation
- focused tests for path resolution, missing-dir behavior, round-trip reads, filters, and subtype validation

Out of scope:

- retrieval Layer 2 / full-text indexing
- semantic search / embeddings
- markdown body parsing
- invoke-handler registration
- prompt engine
- route-engine / inbox projection
- frontend changes

## Write Boundary

Primary targets:

- `crates/seatloom-core/src/objects/artifact.rs` only if a small validation result type or helper is needed
- `crates/seatloom-core/src/storage/project.rs`
- `crates/seatloom-core/src/storage/mod.rs`
- `src-tauri/src/commands/artifact_cmds.rs`

You may add new bounded repository files under `crates/seatloom-core/src/storage/`, for example:

- `artifact_store.rs`
- `artifact_validation.rs`

If you add files, export them in `crates/seatloom-core/src/storage/mod.rs` and keep the packet repository-only.

## Required Outcome

### 1. Add deterministic artifact path helpers

Extend `ProjectPaths` to resolve, at minimum:

- artifact root directory
- per-artifact directory
- per-artifact `meta.yaml`
- per-artifact payload path helper (or equivalent bounded payload-path resolution)

Use the architecture contract storage shape:

- `.seatloom/artifacts/<artifact_id>/meta.yaml`
- `.seatloom/artifacts/<artifact_id>/payload.*`

Do not invent a second artifact layout.

### 2. Implement a real ArtifactStore

Provide a typed repository surface that can:

- list artifacts from `.seatloom/artifacts/*/meta.yaml`
- load one artifact by id
- filter listed artifacts by `template`, `subtype`, and optional `source_workitem_id`
- return empty collections when the artifact directory does not exist
- return typed path-aware errors for malformed YAML or unreadable paths

Deterministic ordering rule:

- sort by `created_at` descending
- stabilize directory iteration first so ties remain deterministic

### 3. Implement bounded subtype validation

Artifact subtype handling must follow the current contract:

- `template+subtype` is the canonical typed-artifact key
- subtype allow-list authority is `docs/coordination/DOCUMENT_TEMPLATES.md`
- invalid or unknown subtype must be detectable without reading the markdown body

Implement a bounded validation surface such as `validate_artifact_subtype(id)` that returns an explicit result object or status, for example:

- template present + subtype allowed => valid
- template present + subtype missing/unknown => invalid
- system-generated artifact with `system_kind` and no template => not_applicable

Hard rule: keep the allow-list local and deterministic for this packet. Do not add network/config loading.

### 4. Wire artifact command library functions to real data

Replace current artifact stubs so that, at minimum:

- `list_artifacts(template?, subtype?, workitem_id?)` returns real filtered artifact metadata
- `get_artifact(id)` returns the matching artifact or `None` / typed error per your existing command style
- `validate_artifact_subtype(id)` returns the validation result surface added in this packet

A bounded `std::env::current_dir()` project-root assumption is acceptable for this packet if documented clearly.

## Acceptance Criteria

1. `list_artifacts()` no longer returns a hardcoded empty vector when matching artifact metadata exists on disk.
2. Missing `.seatloom/artifacts/` directories return empty vectors rather than panics or opaque errors.
3. Artifact listing and load errors preserve typed path context.
4. Artifact filtering by `template`, `subtype`, and `workitem_id` is deterministic and covered by focused tests.
5. Subtype validation follows the documented allow-list and does not rely on markdown-body inference.
6. `$HOME/.cargo/bin/cargo check` passes.
7. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
8. Scope stays metadata/read-model only; no retrieval-engine, runtime, or UI widening enters this packet.

## Required Validation

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
```

If there are pre-existing warnings outside this packet, record them and keep scope bounded.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-artifact-read-models-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Path helper additions
4. Artifact repository behavior
5. Filter and subtype-validation behavior
6. Command wiring summary
7. Validation commands and results
8. Residual notes / explicit non-goals kept out
9. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] A real artifact repository exists for the scoped metadata slice.
- [ ] `list_artifacts()` no longer returns a hardcoded empty vector for populated artifact directories.
- [ ] `get_artifact()` and `validate_artifact_subtype()` are implemented in a bounded, deterministic way.
- [ ] `$HOME/.cargo/bin/cargo check` passes.
- [ ] `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_artifact_read_models.txt
[Nimbus -> Lyra] Artifact Read Models + Dual-Key Filters
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
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-artifact-read-models-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_artifact_read_models /tmp/nimbus_to_lyra_artifact_read_models.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_artifact_read_models
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
