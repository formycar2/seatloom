# Review: Data Structure and Flow Readiness

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | LYRA-2026-04-29-data-structure-and-flow-readiness-review-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md`, `docs/coordination/DOCUMENT_TEMPLATES.md`, `docs/coordination/COLLABORATION_PROTOCOL.md`, `docs/coordination/reviews/2026-04-28-process-mapping-review.md`, `docs/architecture-design.md`, `docs/architecture-decisions.md`, `infra/postgres/schema/001_seatloom_core.sql`, `infra/postgres/seed/001_real_collaboration_baseline.sql`, `.seatloom/bootstrap/source-map.yaml` |
| tags | lyra, review, infrastructure, postgres, data-model, data-flow, artifacts, retrieval |

## Verdict

**Conditional Accept for infrastructure baseline. Hold for product-complete collaboration truth.**

Nimbus has produced a valid first PostgreSQL foundation: the repo now has a real database schema, a deterministic seed, read repositories, CI-quality Rust checks, and commit-pinned verification. That work is strong enough to count as a P0 infrastructure baseline.

However, the current data model is **not yet sufficient** to serve as the full product truth layer required by `prd-v0.5`. The gap is not small polish. It affects the core product promises around typed artifacts, review loops, retrieval order, prompt handling, continuity, mobile approval parity, and concurrent collaboration.

The correct reading is:

1. **Foundation status**: good and worth keeping.
2. **Truth-model completeness**: not yet sufficient.
3. **Seed accuracy**: directionally truthful and well-sourced, but intentionally partial and not replay-complete.
4. **Next owner expectation**: Nimbus should continue on infrastructure only, expanding the authority schema and projections before more UI consumes it.

## 1. What Is Already Good Enough

### 1.1 Infrastructure baseline quality

The following baseline work is solid and should be retained:

- PostgreSQL is now present as a real repo-managed dependency (`infra/postgres/docker-compose.yml`).
- A concrete schema exists for the first core object families (`infra/postgres/schema/001_seatloom_core.sql`).
- Seed data is tied to evidence and approximation notes (`infra/postgres/seed/001_real_collaboration_baseline.sql`, `.seatloom/bootstrap/source-map.yaml`).
- Rust quality gates are real and verified (`cargo check`, `cargo test`, `fmt`, `clippy`, plus commit-pinned verification).
- A typed read repository layer exists in `crates/seatloom-core/src/db/repositories.rs`.
- The schema already recognizes the dual-key artifact direction (`template` + `subtype`) and an append-only event family (`canonical_events`).
- Seat identity / role binding / delegation already have a meaningful first relational shape.

### 1.2 Why this foundation matters

This baseline is valuable because it proves three important things:

1. SeatLoom can move beyond file-only mock state.
2. Our real collaboration can already be represented as structured objects.
3. Nimbus can now build the data authority layer incrementally with verification discipline.

## 2. Where The Current Baseline Is Not Yet Sufficient

### 2.1 Authority drift: PostgreSQL vs file-first / SQLite-first contracts

There is a live architecture conflict:

- `docs/coordination/DOCUMENT_TEMPLATES.md` says PostgreSQL is the single storage engine for structured, full-text, and semantic retrieval.
- The accepted repo baseline also now introduces PostgreSQL as the real structured store.
- But `docs/architecture-design.md` and `docs/architecture-decisions.md` still describe `.seatloom/` file-first authority and SQLite FTS5 as the P0 retrieval authority.
- The Tauri app-facing commands still read from file stores, not from PostgreSQL (`src-tauri/src/commands/*_cmds.rs`).

This means the current stack still has **two competing sources of truth**:

1. PostgreSQL read repositories,
2. `.seatloom/` YAML/JSONL stores consumed by the app surface.

That is acceptable only as a temporary migration state. It is not acceptable as the steady-state product truth architecture.

### 2.2 The seed is truthful but intentionally partial

The seed baseline is not false, but it is not a full replay of our actual collaboration history.

Evidence:

- Process mapping reconstructs **5 Seats, 11 WorkItems, 6 Sessions, 7 Handoffs, 27 Artifacts**.
- The current PostgreSQL seed contains **5 Seats, 6 WorkItems, 5 Sessions, 3 Handoffs, 14 Artifacts**.
- `.seatloom/bootstrap/source-map.yaml` explicitly calls the seed a curated reconstruction and explicitly marks some families as deferred.

Therefore the seed should be treated as:

- **accurate within declared scope**,
- **insufficient as a full collaboration mirror**,
- **not yet strong enough for replay-grade product demos or retrieval evaluation**.

### 2.3 Several core product objects still have no durable schema

The current PostgreSQL schema does not yet cover multiple P0/P1 product-critical objects or relations.

Missing or structurally absent today:

- review comment objects,
- comment threads and replies,
- resolve / dispute / reopen state,
- artifact anchor targets (document-level, heading-level, range-level),
- `change_tier_record`,
- prompt state persistence,
- prompt decision / approval records,
- checkpoint objects,
- continuity packs / supervisor continuity packs,
- artifact versions / diffs,
- deterministic retrieval projections,
- mobile action / interrupt projections,
- inbox projection objects,
- timeline projection queries over DB truth,
- document body persistence in PostgreSQL.

## 3. Accuracy Assessment Of The Seeded Collaboration Data

## 3.1 What is accurate

The seed is credible in these ways:

- Seat identities and roles align with the process mapping review.
- The delegation scenario “Flux acting for Mira” is captured.
- The main Nimbus infrastructure work items are captured.
- The seed includes active T1-T7 artifact examples and valid `template+subtype` coverage.
- Event chronology broadly matches the known sequence from the collaboration records.
- Approximation notes are honestly documented.

## 3.2 What is incomplete or lossy

The seed loses important product truth in at least six ways:

1. **Session compression**: multiple real sessions are collapsed into one representative session per seat.
2. **WorkItem compression**: important real work items are omitted, including intermediate review / alignment / value-definition work.
3. **Handoff compression**: only 3 of the 7 reconstructed handoffs are present.
4. **Artifact compression**: only 14 of the 27 reconstructed artifacts are present.
5. **Event compression**: the event ledger is selected, not exhaustive; it preserves shape, not replay completeness.
6. **State collapse**: current tables store mostly final-state snapshots; the richer lifecycle depends on event payload interpretation.

## 3.3 One concrete consistency issue to correct

Nimbus’s delivery note describes the seed as containing “70+ events,” but the current seed file inserts **34 `canonical_events` rows**. The larger number may be counting object references or total structured inserts, but as written it is misleading. That delivery language should be normalized to the actual row count or explicitly clarified.

## 4. Product-Flow Support Assessment Against PRD v0.5

## 4.1 Artifact objectization

**Current state: Partial**

Supported now:

- artifact identity,
- `template`,
- `subtype`,
- `subtype_valid`,
- title / summary / source links / storage path.

Not supported yet but required by PRD:

- universal header metadata as structured columns (`id`, `status`, `author`, `date`, `version`, `depends_on`, `supersedes`, `tags`),
- document body persistence in PostgreSQL,
- multi-WorkItem linkage,
- project-scope linkage,
- open/read state,
- version history,
- diff review,
- anchor map,
- section comment counts.

Assessment: artifact metadata is enough for a first typed list, but not enough for the PRD artifact system.

## 4.2 Open / read / inspect flows for markdown documents

**Current state: Partial**

The current model supports `storage_path`, which is enough to locate a file. It does **not** yet support the richer product flow implied by `INT-07`, `UX-07`, and `P-11`:

- typed artifact reader driven by metadata plus body,
- deterministic section index,
- open-in-editor plus in-product read state,
- quick-view projections,
- version-aware navigation.

If the product direction is now “persistent in PostgreSQL,” artifact bodies should no longer be authoritative only on disk.

## 4.3 Comments, review threads, and review-to-work routing

**Current state: Missing**

PRD 6.5 requires:

- manual comments,
- Supervisor-assisted comments using the same comment object family,
- reply / resolve / dispute,
- section badges with comment counts,
- routing comments back into Inbox and Timeline,
- escalating a comment into a structured follow-up.

No current schema family supports this. `canonical_events` alone is insufficient because comments are first-class durable collaboration objects, not only audit side effects.

## 4.4 Reissue and `change_tier_record`

**Current state: Missing**

`prd-v0.5`, `interaction-spec-v1.1`, and `acceptance-spec-v1.1` require a structured `change_tier_record` with fields including:

- `tier`,
- `reason`,
- `changed_clauses`,
- `impact_level`,
- `executor`,
- `reviewer`,
- `ack_mode`,
- `evidence_refs`.

There is no table or JSONB contract for this today. Without it, review-driven reissue loops are not durably represented.

## 4.5 Prompt-blocked / approval / mobile continuation flows

**Current state: Missing at persistence layer**

The Rust object model includes `PromptState`, but PostgreSQL does not yet persist:

- the active prompt state,
- classification outcome,
- policy outcome,
- bounded preview reference,
- allowed actions,
- action history,
- approval decision channel (`desktop` / `mobile` / `supervisor-assist`),
- assist budget usage.

This means the product can show UI mocks for prompt handling, but the canonical truth layer cannot yet drive those flows.

## 4.6 Continuity, checkpoints, LaunchPack, and SupervisorPack

**Current state: Missing at persistence layer**

Sessions only store:

- `launch_pack_ref`,
- `last_checkpoint_id`.

But there is no durable schema for:

- checkpoints,
- checkpoint summaries,
- context tiers,
- pack composition entries,
- seat-skill matches,
- playbook matches,
- budget estimate records,
- supervisor continuity deltas.

This is below the threshold required for `US-P0-09`, `INT-06`, `INT-09`, `UX-06`, and acceptance checks around inspectable continuity.

## 4.7 Retrieval and search

**Current state: Missing as a real persisted engine**

The product contract requires fixed retrieval order:

1. structured index,
2. full-text,
3. semantic,
4. LLM explanation.

Current problems:

- architecture docs still freeze SQLite FTS5,
- PostgreSQL schema has no retrieval projection tables or search vectors,
- no `documents` authority table or equivalent header/body index exists,
- no `tsvector` fields exist,
- no chunk table exists,
- no `pgvector` layer exists,
- no query router projection exists.

So the current baseline stores core objects, but it does **not** yet implement the PRD retrieval contract.

## 4.8 Concurrent collaboration and write safety

**Current state: Insufficient**

The current schema is not yet safe enough for multi-seat concurrent writes because it lacks most of these controls:

- `project_id` on all major object families,
- `updated_at` on most mutable tables,
- monotonically increasing `revision` or row version,
- idempotency keys for command writes,
- optimistic concurrency checks,
- conflict detection for document header/body sync,
- edit leases or draft ownership where needed,
- explicit append-safe thread/comment model.

This matters because SeatLoom is fundamentally a concurrent collaboration product. Without these controls, “same canonical object across seats and channels” is not durable.

## 4.9 Projection support for Inbox / Timeline / Dashboard / Mobile

**Current state: Missing or stubbed**

PRD and UX require deterministic projections for:

- Morning Digest,
- Inbox next-action queue,
- Timeline replay and drill-through,
- dashboard metrics,
- mobile monitor cards,
- mobile approval queues,
- interrupt triage.

Current app command status shows the gap clearly:

- `get_inbox()` returns an empty vector stub,
- `query_timeline()` returns an empty vector stub,
- `get_prompt_state()` is a stub,
- `reconcile()` is a stub,
- artifact/seat/session/workitem/handoff commands are still file-backed.

This is expected for an infra baseline, but it means frontend-facing product truth is not yet connected.

## 5. Concrete Schema Drift Already Visible Inside The Codebase

There are several mismatches between accepted object models and the current PostgreSQL schema.

### 5.1 WorkItem drift

Rust object includes:

- `depends_on`,
- `parent_id`.

PostgreSQL table does not.

### 5.2 Handoff drift

Rust object includes:

- `artifact_ids`.

PostgreSQL table does not.

### 5.3 Session drift

Rust object includes `PromptState`, but PostgreSQL has no prompt-state persistence.

### 5.4 Event drift

Rust `CanonicalEvent` includes:

- `object_refs`,
- `evidence_refs`.

PostgreSQL stores object refs only in a side table and does not persist `evidence_refs` in normalized form.

### 5.5 Artifact authority drift

User direction has already shifted toward PostgreSQL persistence, but artifact bodies are still described as disk-authoritative in schema comments and implementation.

These drifts should be resolved before the schema is treated as stable.

## 6. Required Infrastructure Requirements From PRD v0.5

The next Nimbus packet should stay infrastructure-only, but it must expand the authority model materially.

## 6.1 Authority and storage requirements

### DS-01 Single authority direction

SeatLoom must choose and document one authoritative storage path for product truth.

Required outcome:

- PostgreSQL becomes the canonical structured authority.
- File-backed stores become either migration inputs, cache/export artifacts, or backward-compatibility shims.
- `architecture-design.md` and `architecture-decisions.md` must be realigned to this direction.

### DS-02 Project scoping

All major persisted families must be project-addressable.

Required outcome:

- add `project_id` or equivalent normalized ownership to sessions, workitems, handoffs, artifacts, comments, checkpoints, and canonical events,
- guarantee deterministic multi-project filtering and project switching.

### DS-03 Document body persistence

If the product requirement is “persist in PostgreSQL, not only files,” artifact body truth must be stored in PostgreSQL.

Required outcome:

- artifact metadata table plus document body table or unified document authority table,
- stored markdown body,
- digest/hash, content length, parse status, and updated timestamp,
- optional disk-export path may remain as derived output, not sole authority.

## 6.2 Artifact model requirements

### DS-04 Structured universal header projection

The universal document header fields from `DOCUMENT_TEMPLATES.md` must be stored structurally:

- `template`,
- `subtype`,
- `document_id`,
- `status`,
- `author`,
- `date`,
- `version`,
- `depends_on`,
- `supersedes`,
- `tags`.

### DS-05 Artifact association model

Artifacts must support:

- many-to-many WorkItem association,
- handoff association,
- session association,
- project-global association,
- provenance / source-of-truth marker,
- subtype validation result with explicit failure reason.

### DS-06 Artifact versioning

Add version-aware storage:

- artifact revisions,
- supersedes chain,
- diffable body snapshots,
- current revision pointer.

## 6.3 Review and comment requirements

### DS-07 First-class comment model

Introduce durable review comment families that support both desktop and mobile feedback.

Minimum fields:

- comment id,
- thread id,
- project id,
- target object type/id,
- artifact anchor type and anchor ref,
- author seat or human,
- body,
- status (`open`, `resolved`, `disputed`, `superseded`),
- created / updated / resolved timestamps,
- linked evidence refs,
- source channel (`desktop`, `mobile`, `supervisor_draft`).

### DS-08 Thread and routing model

Comments must support:

- replies,
- resolve / dispute,
- escalation into `change_tier_record`,
- routing into Inbox/Timeline projections.

## 6.4 Review / reissue requirements

### DS-09 `change_tier_record` persistence

Persist a structured change-tier object, not only an event payload.

Minimum fields:

- target workitem id,
- tier (`L1`, `L2`, `L3`),
- reason,
- changed clauses,
- impact level,
- executor,
- reviewer,
- ack mode,
- evidence refs,
- created_at,
- closed_at,
- resulting workitem or handoff ref if follow-up is created.

## 6.5 Prompt and interrupt requirements

### DS-10 Prompt-state persistence

Add a prompt-state authority family for `Prompt blocked` flows.

Minimum fields:

- session id,
- prompt kind,
- policy,
- bounded preview ref or stored tail,
- allowed actions,
- current state,
- sensitivity flag,
- assist step/token budget,
- desktop/mobile eligibility,
- resolved action and actor.

### DS-11 Interrupt action journal

Mobile and desktop interrupt actions must write the same canonical records with channel metadata.

## 6.6 Continuity and checkpoint requirements

### DS-12 Checkpoint authority model

Persist checkpoint objects structurally, including:

- summary,
- branch / commit,
- artifact set,
- open questions,
- trigger,
- transcript-tail reference.

### DS-13 Pack composition model

Continuity packs must be inspectable as structured compositions, not only generated markdown.

Minimum persisted components:

- pack id,
- session id,
- pack type (`worker`, `supervisor`),
- tier entries (T0/T1/T2),
- selected artifacts,
- selected events,
- selected comments,
- seat-skill matches,
- playbook matches,
- budget estimate,
- overflow reason,
- final compiled artifact ref.

## 6.7 Retrieval requirements

### DS-14 PostgreSQL retrieval authority

Realign retrieval to PostgreSQL if that is now the approved storage direction.

Minimum P0 structure:

- structured document/object index,
- full-text index using `tsvector`,
- exact filters on `template`, `subtype`, `status`, `tags`, `object_refs`,
- deterministic ranking rules for exact-reference search.

### DS-15 Searchable document-body projection

Add normalized searchable body storage with:

- parsed text,
- section headings / anchor index,
- token count or size estimate,
- `search_vector` or equivalent.

### DS-16 Retrieval query journal

Record retrieval requests and result provenance for auditable “why this result” behavior.

## 6.8 Concurrency and integrity requirements

### DS-17 Optimistic concurrency

Every mutable authority family should include:

- `created_at`,
- `updated_at`,
- `revision` or row version.

### DS-18 Idempotent write contract

Write-side commands should carry idempotency keys so repeated mobile/desktop retries do not fork canonical truth.

### DS-19 Conflict policy

Define how SeatLoom handles:

- concurrent comment resolution,
- concurrent document metadata edits,
- document body changed after anchor creation,
- mobile feedback arriving while desktop edit is open.

## 6.9 Projection requirements

### DS-20 Deterministic read projections

Add infrastructure support for deterministic projections or materialized views for:

- Inbox,
- Timeline,
- dashboard health cards,
- mobile action queue,
- artifact reader summary header,
- seat capability truth summary.

These may begin as SQL views or projector tables, but they must be defined now as product-facing read models.

## 7. Recommendation To Nimbus

Nimbus should continue, but only on infrastructure scope. The next useful packets are:

1. **Authority realignment packet**: PostgreSQL as truth, architecture docs aligned, app read path migration plan explicit.
2. **Artifact authority packet**: document headers + body + relations + versioning.
3. **Review object packet**: comments, threads, anchors, routing, `change_tier_record`.
4. **Continuity and prompt packet**: checkpoints, packs, prompt state, interrupt journal.
5. **Retrieval packet**: Postgres structured + FTS projections aligned with PRD order.
6. **Projection packet**: inbox/timeline/dashboard/mobile deterministic read models.

## 8. Final Assessment Answer

### Is Nimbus’s current data-structure work “good enough”?

**As a first infrastructure baseline: yes.**

**As the product-complete collaboration truth layer required by PRD v0.5: no.**

### Is the seeded collaboration data accurate?

**It is accurate within declared scope, but it is curated, partial, and not replay-complete.**

### Does the current structure support future data operations around markdown artifacts, opening, storage, concurrent collaboration, comments, and review routing?

**Only partially.**

It supports the first layer of object persistence and typed artifact listing, but it does not yet support the full PRD flows for comment threads, review routing, continuity inspection, prompt handling, retrieval, or concurrent collaboration safety.

## 9. Go / Hold Summary

- PostgreSQL foundation: **ACCEPT**
- Current schema as final authority model: **HOLD**
- Current seed as demo-grade baseline: **ACCEPT WITH PARTIAL-SCOPE WARNING**
- Current support for artifact review/comment/concurrency flows: **HOLD**
- Nimbus next lane: **CONTINUE ON INFRASTRUCTURE ONLY**
