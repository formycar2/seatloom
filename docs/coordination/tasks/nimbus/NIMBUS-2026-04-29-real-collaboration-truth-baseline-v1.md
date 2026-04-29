# Task: Nimbus Real Collaboration Truth Baseline

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-29-real-collaboration-truth-baseline-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-04-30 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `docs/coordination/reviews/2026-04-28-process-mapping-review.md`, `docs/coordination/MEMORY.md`, `docs/coordination/memory/2026-04-29.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-read-model-repositories-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-seat-registry-delegation-storage-acceptance.md`, `docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md`, `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`, `docs/coordination/COORDINATION_RULES.md` |
| tags | nimbus, infrastructure, seed-data, truth-baseline, seatloom, artifacts, ledger, repositories |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded infrastructure packet only. You are not alone in the codebase; do not revert others' work, do not widen into product/business logic, route engine, prompt engine, PTY/runtime behavior, or UI implementation. |

## Objective

Build the first **repository-native real project truth baseline** for SeatLoom.

The goal is to place our **actual collaboration history and actual coordination artifacts** into `.seatloom/` using the accepted SeatLoom object model, so that a later frontend can read real data from the repo directly instead of demo-only mock data.

This is **not** an API-first packet and **not** a business-feature packet.
It is an infrastructure and data-foundation packet:

1. objectize the current SeatLoom collaboration into real local files,
2. make that data loadable through the existing deterministic repository layer,
3. close the last missing artifact metadata read path needed for later frontend consumption.

## Why this packet exists

Mr. Zhang's direction is explicit:

- the product should reflect the way we are already collaborating to build SeatLoom;
- the repo should contain **real** seats / sessions / workitems / handoffs / artifacts / events;
- frontend work should later read those repo-native objects directly, rather than waiting for a later API or relying on fake sample data.

So this packet is the bridge between:

- accepted contract + storage foundations already landed, and
- future UI / usability work that needs truthful repository data.

## Required Read Order

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/acceptance-spec-v1.1.md`
5. `docs/architecture-decisions.md`
6. `docs/architecture-design.md`
7. `docs/coordination/DOCUMENT_TEMPLATES.md`
8. `docs/coordination/reviews/2026-04-28-process-mapping-review.md`
9. `docs/coordination/MEMORY.md`
10. `docs/coordination/memory/2026-04-29.md`
11. the three accepted Nimbus infra acceptances listed above
12. this packet

## Need-to-Know Scope

Implement only the **truth-data baseline** needed for repo-local loading and future UI consumption.

In scope:

- create a real `.seatloom/` project dataset for the current SeatLoom repo;
- encode real seats, role bindings, scoped delegation, sessions, workitems, handoffs, artifacts, and ledger events;
- use **actual repository evidence** from our collaboration on 2026-04-27 through 2026-04-29;
- add the bounded artifact metadata repository support that is still missing, so typed artifacts can be listed and loaded locally;
- add one deterministic integrity check path proving the seeded baseline can be loaded from the repo root without mock-only assumptions;
- keep all data local, typed, and Git-trackable.

Out of scope:

- no backend API surface expansion beyond bounded local read commands/repositories;
- no route engine / inbox projection work;
- no prompt engine / PTY / runtime work;
- no pipeline automation work;
- no UI / frontend work;
- no generalized auto-ingest framework;
- no semantic retrieval / embeddings / FTS expansion;
- no mutation-heavy flows like create/update/transition commands beyond what is strictly needed to write the baseline files.

## Real Data Rules

Hard rules for this packet:

1. **No fake demo content.** Do not use lorem ipsum, placeholder owners, invented file names, or invented narratives.
2. **Use real repository artifacts.** If an artifact object exists, it must point to a real file already present in this repo or to a payload copy created from a real file in this repo.
3. **Use explicit provenance.** If any timestamp or status had to be approximated from evidence rather than copied from an exact source, note that in the provenance output.
4. **Use contract taxonomy exactly.** Artifact `template + subtype` must follow `docs/coordination/DOCUMENT_TEMPLATES.md`.
5. **Keep it truthful, not exhaustive.** A curated, evidence-backed baseline is acceptable; a speculative complete reconstruction is not.

## Write Boundary

Primary data targets:

- `.seatloom/config/project.yaml`
- `.seatloom/seats/**`
- `.seatloom/delegations/*.yaml`
- `.seatloom/sessions/**`
- `.seatloom/workitems/*.yaml`
- `.seatloom/handoffs/*.yaml`
- `.seatloom/artifacts/**`
- `.seatloom/ledger/events.jsonl`

Primary code targets:

- `crates/seatloom-core/src/storage/project.rs` only if bounded path helpers are still needed
- `crates/seatloom-core/src/storage/mod.rs`
- `src-tauri/src/commands/artifact_cmds.rs`

You may add bounded files under:

- `crates/seatloom-core/src/storage/` such as `artifact_store.rs`
- `crates/seatloom-core/tests/` for baseline integrity verification
- `scripts/` for one local baseline verification entrypoint if helpful

Do not widen into unrelated modules.

## Required Outcome

### 1. Materialize a real `.seatloom/` baseline

Create the first committed project-local baseline under `.seatloom/`.

At minimum, seed:

- the project config;
- seat identities for `aegis`, `lyra`, `mira`, `nimbus`, `flux`;
- one project role binding per seat for the `seatloom` project;
- at least one scoped delegation capturing the real `flux acting for mira` case;
- a session set that can express:
  - Aegis supervisor work,
  - Lyra runtime switch / coordination work,
  - Mira UI work,
  - Nimbus infrastructure work,
  - Flux verification work;
- workitems that reflect real collaboration lanes rather than generic placeholders;
- handoffs that reflect real seat-to-seat transfers already evidenced in repo history;
- artifacts that cover the active contract set plus key coordination evidence;
- ledger events that connect the seeded object graph chronologically.

This baseline may be curated rather than exhaustive, but it must be able to tell a truthful end-to-end story of the current SeatLoom collaboration.

### 2. Cover all core object families with real references

The seeded object graph must include real references across these families:

- `SeatIdentity`
- `ProjectRoleBind`
- `SeatDelegation`
- `Session`
- `WorkItem`
- `Handoff`
- `Artifact`
- `CanonicalEvent`

Minimum relationship requirements:

- every seeded `Handoff` references a real `WorkItem`;
- every seeded `Artifact` has a real title and real source path / payload provenance;
- every seeded `Session` belongs to a real seeded seat;
- ledger events must reference the seeded object IDs they claim to describe;
- at least one event chain must show workitem review/reissue evidence;
- at least one event chain must show delegation issuance;
- at least one event chain must show artifact creation tied to a real coordination file.

### 3. Finish the missing artifact metadata read-model slice

The repo still lacks real artifact metadata loading.

Implement the bounded missing slice needed for the truth baseline:

- deterministic artifact path helpers if still missing;
- an `ArtifactStore` or equivalent bounded repository surface;
- list / load for artifact metadata from `.seatloom/artifacts/*/meta.yaml`;
- deterministic filtering by `template`, `subtype`, and optional `source_workitem_id`;
- bounded subtype validation against the current allow-list;
- real `list_artifacts(...)` and `get_artifact(...)` command behavior rather than hardcoded empty results.

This subsumes only the metadata portion needed for the seed baseline.
Do **not** widen into full retrieval engine work.

### 4. Provide provenance and truth notes

Write one deterministic provenance artifact, for example under:

- `.seatloom/bootstrap/source-map.yaml`
- or `.seatloom/bootstrap/source-map.md`

It must state:

- which repo files were used as evidence for seeded seats / sessions / workitems / handoffs / artifacts / events;
- where approximation was used;
- which object families are intentionally partial in v1.

### 5. Add one bounded integrity verification path

Add one deterministic verification path that proves the repo-root baseline is readable.

Acceptable examples:

- focused Rust tests that load the current repo's `.seatloom/` baseline and assert non-empty, referentially-valid reads;
- or one bounded script that runs those tests / checks;
- or both, if still kept narrow.

The point is not heavy automation; the point is that future seats can trust that the committed baseline is loadable and internally coherent.

## Minimum Artifact Family Coverage

The seeded artifacts must include **real** examples spanning the current taxonomy, with at least:

- one `T1` authority doc,
- one `T2` role profile,
- one `T3` task packet,
- one `T4` review,
- one `T5` acceptance,
- one `T6` daily memory entry,
- one `T7` governance doc.

In addition, the baseline must include the active contract set:

- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/architecture-decisions.md`
- `docs/architecture-design.md`
- `docs/PRODUCT_TRUTH.md`

## Acceptance Criteria

1. A committed `.seatloom/` directory now exists with real, non-placeholder project data.
2. The seeded baseline expresses the current SeatLoom collaboration through connected seat / session / workitem / handoff / artifact / event objects.
3. At least one seeded delegation object truthfully captures the `Flux acting for Mira` case.
4. Artifact objects use valid `template + subtype` pairs or a clear `system_kind` path.
5. `list_artifacts(...)` and `get_artifact(...)` no longer behave as hardcoded empty stubs when artifact metadata exists on disk.
6. The baseline includes at least one real artifact example for each `T1` through `T7` family and includes the active contract set.
7. A deterministic provenance artifact exists and explains evidence sources plus approximations.
8. One deterministic integrity verification path exists and passes on a Rust-capable seat.
9. `$HOME/.cargo/bin/cargo check` passes.
10. `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
11. Scope stays infrastructure/data-foundation only; no route-engine, prompt-engine, or UI widening enters this packet.

## Required Validation

```bash
$HOME/.cargo/bin/cargo check
$HOME/.cargo/bin/cargo test -p seatloom-core
```

If you add a bounded baseline verification script, run it too and record the result.

## Required Delivery Artifact

Write:

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-truth-baseline-delivery-v1.md`

Required sections:

1. Scope completed
2. Files changed
3. Seeded `.seatloom/` structure summary
4. Seeded object coverage by family
5. Artifact taxonomy coverage (`template + subtype`)
6. Provenance sources and approximation notes
7. Artifact repository / command behavior
8. Validation commands and results
9. Residual notes / explicit non-goals kept out
10. Recommended next owner

## Done Definition

- [ ] Delivery artifact is written at the required path.
- [ ] `.seatloom/` baseline is committed with real project data.
- [ ] Artifact metadata repository support exists and is loadable.
- [ ] Baseline provenance is written.
- [ ] Deterministic validation passes on a Rust-capable seat.
- [ ] `$HOME/.cargo/bin/cargo check` passes.
- [ ] `$HOME/.cargo/bin/cargo test -p seatloom-core` passes.
- [ ] tmux reply is sent to Lyra after writeback.

## Direct tmux reply contract

Send this exact format when done:

```bash
cat <<'MSG' >/tmp/nimbus_to_lyra_real_collaboration_truth_baseline.txt
[Nimbus -> Lyra] Real Collaboration Truth Baseline
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => ...
- `...` => ...
blockers:
- none / ...
next action:
- wait for acceptance
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-truth-baseline-delivery-v1.md
MSG

tmux load-buffer -b nimbus_to_lyra_real_collaboration_truth_baseline /tmp/nimbus_to_lyra_real_collaboration_truth_baseline.txt
tmux paste-buffer -t 'Lyra-po-seatloom' -b nimbus_to_lyra_real_collaboration_truth_baseline
tmux send-keys -t 'Lyra-po-seatloom' Enter
```
