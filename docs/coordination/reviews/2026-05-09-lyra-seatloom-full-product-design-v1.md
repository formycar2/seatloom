# Product Design Supplement: Data Lifecycle, Real-Time Tiers, Multi-Project, Historical Migration, and L1 Concrete Flows

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | 2026-05-09-lyra-seatloom-full-product-design-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-09 |
| to | aegis (review), nimbus (technical architecture follow-up) |
| priority | P0 |
| relationship | **Extends `docs/prd-v0.5.md`** — does not supersede. PRD remains the product-thesis and scope authority. This document adds operational layers the PRD does not yet specify. |
| depends_on | `docs/prd-v0.5.md` (active product contract), `docs/PRODUCT_TRUTH.md` §1.2 (canonical contract set), `docs/architecture-decisions.md` (AD-008 dual-key artifact typing, AD-009 seat three-layer, AD-010 review-failure event-first, AD-011 retrieval order, AD-012 prompt architecture, AD-013 v2 Supervisor context model), `docs/architecture-design.md` (§3 Rust core types, §6 storage layer, §7 frontend state, §12 Data Engine), `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (tmux-mirror R1-R5, design authority for v0.0.1/0.0.2/0.1), `docs/coordination/reviews/2026-05-09-aegis-mvp-gap-to-tmux-replacement.md` (gap review, SG-A..E packet list), `infra/postgres/schema/001-005_*.sql` (active schema state) |
| tags | product-design, supplement, prd-extension, data-lifecycle, real-time, multi-project, historical-migration, L1, supervisor-im |
| acceptance owner | aegis |
| concurrency rule | Lyra produces; Aegis reviews; Nimbus then produces matching technical-architecture supplement (`crates/seatloom-core/` + `infra/postgres/schema/` design) before any v0.1 implementation packet lands. |

---

## 0. Why this document exists

Mr. Zhang's directive (2026-05-09):

> "我不是在做 MVP，需要把任务布置给 Lyra 来完成产品设计、Nimbus 来做技术架构设计，进一步完善功能。"

And, after the first draft outline:

> "请在现有的 PRD 基础上来完善和补充。"

PRD v0.5 settles **what SeatLoom is** — five module contracts, fifteen P0 stories, L1/L2 layering, dual-key artifact typing, retrieval order, budget enforcement. It does not yet settle five operational questions that block v0.1 from running on real data:

1. **Data lifecycle** — when a coordination markdown file is written in tmux, what is the explicit step sequence by which it becomes a `workitem` / `handoff` / `canonical_event` row, and which step gates which UI surface?
2. **Real-time tiers** — what is the latency budget for each surface? PRD says "deterministic"; it does not say "≤1 s vs ≤5 s vs batch".
3. **Multi-project** — `project_role_bindings` carries `project_id`, but `workitems`, `handoffs`, `canonical_events`, `artifacts` do not. PRD US-P0-03 implies multi-project is required; the schema does not yet support it.
4. **Historical migration** — ~300 coordination markdown files dated 2026-04-30 onward exist on disk and reconcile into `documents`, but the runtime objects (`workitems`, `handoffs`, `canonical_events`) are still on the 2026-04-27..29 seed. The PRD is silent on whether and how to back-fill.
5. **L1 concrete flows** — PRD §6.4 describes Supervisor abstractly. The tmux-mirror architecture (`a5998c1`) gives the data path (tmux → fifo → backend → IM). The user-visible semantics — *what bubble appears, when, with what link* — need product decisions.

This supplement answers each in the same form: cite PRD anchor → state gap → 2–3 options → recommendation with rationale → reference US-P0-xx + tmux-mirror requirement (R1–R5) it satisfies.

It produces zero new feature scope. Every option below is implementation-shape choice within scope already present in PRD v0.5 §5.

---

## 1. Data lifecycle

### 1.1 Context from PRD v0.5

- §6.1 Data Engine: "If structured data already exists, the Data Engine must assemble and render it before any LLM call is considered." (Rule 1)
- §6.5 Artifact review: dual-key `template + subtype` is canonical classification; subtype allow-list owned by `DOCUMENT_TEMPLATES.md` §11.1.
- §6.5 Rule 6: "Coordination artifacts must preserve `template`, `subtype`, and universal header metadata as structured fields even when the body renders as markdown."
- AD-007 (architecture-decisions.md): reconciliation has three trigger points (app launch, pre-pipeline, manual). AD-007's original "no file watcher in MVP" is **superseded for v0.1** by tmux-mirror R4 (file watcher is now P0 for v0.1).

### 1.2 Gap

PRD does not specify the **state machine** by which a markdown file becomes a runtime object. Today:

- markdown lands in `docs/coordination/`,
- reconcile inserts a row into `documents`,
- but no row appears in `workitems` / `handoffs` / `canonical_events` even though the document semantics imply such state changes (e.g. a `T3 task` packet means a workitem was issued; a `T5 acceptance_review` means a workitem moved to `accepted`).

The result: WorkflowPanorama and Inbox show seed-era data, not the live coordination state. Users lose trust in SeatLoom's projection.

### 1.3 Core principle (proposed)

**File system is the source of truth for the *artifact*. PostgreSQL is the projection authority for the *runtime object*.** Reconcile is the **deterministic mapping** from artifact (markdown) to runtime object (row) using the dual-key `template + subtype` as the discriminator.

This principle does not contradict PRD §6.1 Rule 6 — it operationalizes it.

### 1.4 The lifecycle of a T3 task packet

#### Question

A coordination author (Lyra in tmux, Aegis in another tmux) writes a `T3 task` packet markdown file. By what explicit steps does it become a `workitem` row and a `canonical_events` row that drive the Inbox and WorkflowPanorama?

#### Option A — Reconcile-only projection (file → documents → workitems)

Reconcile reads the markdown, validates `template + subtype`, then in the same transaction writes:
1. `documents` row (already done today);
2. one or more `workitems` rows derived from the document header (`id`, `owner`, `priority`, `status`);
3. one `canonical_events` row of `event_type='WorkItemIssued'` with `evidence_ref` pointing to the document.

UI reads from `workitems` and `canonical_events` only; never re-parses markdown.

**Pros**: single authority, deterministic, easy to test, no dual-write race.
**Cons**: every author edit triggers a re-projection; large file edits could cause workitems to be re-derived and over-write any in-product status updates.

#### Option B — Reconcile + in-product mutation, with markdown as outbox

Reconcile creates the *initial* row on first ingest. Subsequent state changes (e.g. supervisor accepts a workitem) write to `workitems` directly via a Tauri command, then emit a `canonical_events` row, then *optionally* write a sibling acceptance markdown file as evidence. The next reconcile run treats the existing workitem row as authoritative and skips re-derivation.

**Pros**: in-product UX writes are first-class, not detoured through file edits.
**Cons**: dual authority — when a markdown is hand-edited *and* a UI mutation happened, who wins? Requires conflict resolution.

#### Option C — Reconcile-only with append-only event log; workitems are a view

Reconcile writes only `canonical_events`. `workitems`, `handoffs`, `inbox` are materialized views over the event log derived deterministically. State changes always come through events (whether from reconcile or from UI Tauri commands).

**Pros**: aligns with AD-010 "review-failure event-first"; single mutation primitive; rebuildable; full audit. Matches `Data Engine Rule 6` ("Every engine action that changes project state must emit an audit record and linked evidence").
**Cons**: more upfront engineering; view materialization needs careful indexing; SELECT performance must be measured.

#### Recommendation: **Option C — event-first with materialized views**

**Rationale**:

1. **AD-010 already commits to event-first for review-failure**. Generalizing this rule to all object lifecycles avoids two write paths (one for review failure events, one for everything else).
2. **PRD §6.1 Rule 6** requires every state-changing engine action to emit an audit record. A pure event-first design makes this structural, not policy.
3. **R3 (failure isolation) survives upgrades better**: if SeatLoom v0.1 has a `workitems` projection bug, v0.1.1 can rebuild the view from the event log without re-reading markdown files. The events table is the durable contract.
4. **Historical back-fill (§4 below) becomes simpler**: replay reconcile against existing markdown to generate retroactive events; the same pipeline serves both first-ingest and migration.

**Trade-off accepted**: extra one-time engineering cost in Nimbus's technical-architecture phase. This is the right place to pay it.

**Maps to**: US-P0-01 (action queue derived from real state), US-P0-05 (review-fail evidence chain), US-P0-07 (typed artifact rendering keyed off event), and the tmux-mirror R4 requirement for lossless data flow.

### 1.5 The lifecycle of a Supervisor IM message

#### Question

A user types "ask Lyra to review the schema" in the Supervisor IM. By what steps does this become a `canonical_events` row and reach Lyra's tmux session?

#### Option A — IM → events → tmux send-keys (synchronous)

IM submit handler calls `cmd_append_supervisor_message` → INSERT `canonical_events(event_type='SupervisorMessage', actor_ref='supervisor:zhang', target_seat_id='lyra', content=...)`. Same Tauri command then runs `tmux send-keys -t Lyra-po-seatloom <content>` (per tmux-mirror R2 write path). Failure of either step rolls back the other.

**Pros**: atomic; user sees both effects simultaneously.
**Cons**: tmux send-keys can fail (session not present, send-keys quoting bug) — coupling event persistence to tmux availability is fragile.

#### Option B — Two-phase: persist event first, then dispatch to tmux

`cmd_append_supervisor_message` writes the event row immediately and returns. A separate worker (tokio task or follow-up command) reads the event and runs `tmux send-keys`. If the send fails, the event row is updated with `delivery_status='failed'` + retry policy.

**Pros**: event capture is durable even if tmux is down; matches R3 failure-isolation spirit (event log decoupled from runtime side effect).
**Cons**: user might see the IM bubble before the seat actually receives the keystrokes — a 50-200ms delay. The IM bubble must indicate delivery status.

#### Option C — Three-phase: persist → dispatch → confirm via mirror

Same as Option B, but in addition the tmux mirror fifo tail watches for the injected text to echo back (since tmux send-keys results in the seat's CLI seeing the input, which the mirror reads). When the echo is observed, the event row is marked `delivery_confirmed`. The IM bubble shows three states: pending → sent → confirmed.

**Pros**: highest fidelity; confirms the seat actually received the keystrokes.
**Cons**: complex; echo detection is timing-sensitive and depends on the seat's CLI rendering; over-engineering for v0.1.

#### Recommendation: **Option B — two-phase, with delivery_status on the event row**

**Rationale**:

1. **Persist-first is the only behavior consistent with R3** (failure isolation: SeatLoom must never lose data because tmux is down).
2. **Option C is correct in spirit but premature** — echo detection is brittle until the mirror parsing pipeline is mature (v0.1 ships read-only mirror in v0.0.1; bidirectional in v0.0.2; hardening in v0.1).
3. **`delivery_status` on the event row** gives the IM bubble enough to distinguish "we recorded your intent" from "the seat received it" without inventing a new table.

**IM bubble UX implication**:

| State | Visual |
|---|---|
| `pending` | grey bubble, spinner |
| `sent` | normal bubble, single check |
| `failed` | normal bubble, red retry icon, click to retry |

`delivery_confirmed` is a v0.2 concern — leave it out of v0.1 to avoid scope creep.

**Maps to**: US-P0-02 (intent → structured proposal → confirm), US-P0-11 (prompt-state surface — the same `delivery_status` mechanism applies to plan-mode approval keystrokes), tmux-mirror R2 (bidirectional) + R3 (failure isolation).

### 1.6 The lifecycle of a Handoff

#### Question

Lyra writes `MIRA-2026-05-09-v02-...-v1.md` (a T3 handoff) in her tmux. How does this surface as a Handoff state-machine row in SeatLoom?

#### Option A — Reconcile-only, status driven by header field

Reconcile reads `status: issued | delivered | accepted | held`, INSERTs a `handoffs` row with that status, emits `HandoffIssued | HandoffDelivered | ...` event.

#### Option B — Reconcile-only, status driven by sibling-document presence

`*-v1.md` (issuance) ⇒ status=`issued`. Presence of sibling `*-delivery-v1.md` ⇒ status=`delivered`. Presence of sibling acceptance under `acceptance/` ⇒ status=`accepted`. Status transitions are inferred from the file system state, not from `status:` headers.

#### Option C — Hybrid: header preferred, sibling-document as fallback

Use the `status` header when present and valid against the allow-list. Fall back to sibling-document inference when the header is missing. Prefer the more advanced state if they disagree (i.e. acceptance file present implies `accepted` even if the issuance header still says `issued`).

#### Recommendation: **Option C — Hybrid with sibling-document priority on disagreement**

**Rationale**:

1. **Authors are not always consistent** — historical packets have `status: issued` in the header but a sibling acceptance doc proving they were accepted. The system must read the union signal.
2. **DOCUMENT_TEMPLATES.md §11.1 (referenced by PRD §6.5) already specifies the universal header includes `status`**. Honor it when present.
3. **The fallback is deterministic** — sibling-document presence is a structural fact, not LLM-inferred.
4. **Disagreement bias toward the more advanced state** matches user mental model: an acceptance document proves the acceptance happened; an unupdated header is just stale metadata.

**Maps to**: US-P0-04 (delegation visibility includes Handoff state), US-P0-05 (review-fail + reissue: the `held` → `delivered` → `accepted` cycle must be visible), AD-010 (event-first review failure).

---

## 2. Real-time tiers

### 2.1 Context from PRD v0.5

- §6.1 Pack Engine + Route Engine: "deterministic" but unquantified.
- §6.4 Supervisor Layer: "explicit confirmation before write actions" — implies user perceives near-instant feedback.
- L1/L2 framework (§3.1): L1 is *highest-frequency, shortest-path*; L2 is *necessary but lower-frequency*.

Tmux-mirror R2 implies "real-time" but does not put a number on it. R4 says "lossless"; it does not say "synchronous".

### 2.2 Gap

Without a tier definition, every backend optimization debate becomes "should this be sync or async?" with no shared answer. Worse: UI components subscribe to live event streams when polling-once-on-mount would suffice, burning CPU and risking event-storm bugs.

### 2.3 Proposal — Three real-time tiers

| Tier | Latency budget | Mechanism | Surfaces |
|---|---|---|---|
| **T-Live** | ≤ 1 s end-to-end (typed character → screen pixel) | Tauri event → component subscriber, no DB roundtrip | tmux mirror output → SessionTerminal xterm; IM message echo into own bubble |
| **T-Near** | ≤ 5 s | DB write + Tauri event → store update → UI re-render | Supervisor IM message arrival on receiver side; reconcile delta after a single file change; Handoff status transition |
| **T-Batch** | manual or scheduled | reconcile run; explicit user trigger (button, ⌘R) | Initial Inbox hydration on app launch; full-document re-ingest after broad coordination directory edits |

#### Where each tier applies

| Surface | Tier | Why |
|---|---|---|
| SessionTerminal xterm output | T-Live | User is watching the seat type — anything > 500 ms feels broken |
| Supervisor IM own-bubble echo | T-Live | Confirms the user's submit reached the system |
| Supervisor IM target-seat keystroke delivery | T-Near | The seat's reply rendering is T-Live; the user knows the keystroke was sent |
| Plan-mode approval card appearance | T-Near | Detection runs against the JSONL transcript tail; 1-3 s window is acceptable |
| WorkflowPanorama event-stream update | T-Near | New events should land within 5 s of the underlying file write |
| Inbox row insertion after reconcile of a new T3 packet | T-Near | File watcher with 2 s debounce + reconcile + UI store update fits in 5 s |
| WorkflowPanorama 30-day historical view | T-Batch | Loaded once on tab open; refreshed only on explicit user action |
| Document search results | T-Batch | User typed query → run search → render |

#### Polling vs. push

- **T-Live and T-Near**: Tauri event push only. No polling.
- **T-Batch**: pull on demand (component mount or user gesture).

#### Watcher granularity

File system watchers (per tmux-mirror R4 + AD-007 supersession) watch only `docs/coordination/**/*.md`. **Not** the entire `docs/` tree, **not** code, **not** binary assets. Rationale: these are the only file kinds that drive runtime objects. Watching the whole repo causes false reconcile triggers on every code save.

### 2.4 Recommendation — adopt the three-tier table verbatim

**Rationale**:

1. **Solves the under-specified "real-time" problem**: every future "should this be live?" question gets a tier answer.
2. **L1/L2 alignment** (PRD §3.1): L1 surfaces (Supervisor IM, action queue) sit in T-Live or T-Near; L2 surfaces (Sessions, Documents) can be T-Near or T-Batch.
3. **Bounded engineering surface**: only two transport mechanisms (Tauri push, on-demand pull). No third path.

**Maps to**: tmux-mirror R2 (bidirectional with reasonable latency), R4 (lossless via T-Near reconcile), all P0 user stories (each surface gets a tier).

---

## 3. Multi-project support

### 3.1 Context from PRD v0.5

- US-P0-03: "I use the same seat identity across projects, but each project shows the correct role, authority docs, and active collaboration mode."
- §6.2 Seat three-layer: Seat Identity (global) + Project Role Binding (per-project) + Collaboration Template (per-project).
- AD-013 v2: Supervisor `contextMode = 'global' | 'project'`; Global vs Project context are mutually exclusive UI modes. The data model behind this UI mode currently assumes one project.

### 3.2 Gap

Schema state (`infra/postgres/schema/001_seatloom_core.sql`):

- `projects` table: PK `id`. Today contains a single seed row.
- `project_role_bindings`: composite PK `(seat_id, project_id)` — multi-project capable.
- `workitems`, `handoffs`, `artifacts`, `canonical_events`, `sessions`, `checkpoints`, `pipeline_runs`, `review_threads`, `prompt_instances`, `documents`: **no `project_id` column**.

Mr. Zhang has multiple active projects in tmux today: `seatloom`, plus `Zephyr-cn-gpu-infer-fabric` (a separate project) and `Onyx-data-seatloom` (extending seatloom but a distinct workstream). PRD US-P0-03 implies these should be browsable as distinct contexts.

### 3.3 Decisions needed

#### 3.3.1 Schema scope of `project_id`

##### Option A — Add `project_id` only to `canonical_events`; derive everything else

Workitems, handoffs, artifacts, sessions become materialized views filtered by their owning event's `project_id`. Single ALTER TABLE.

**Pros**: minimal migration; aligns with §1.4 Option C (event-first).
**Cons**: every join query must traverse events; query plans get more complex; some views may need denormalized `project_id` for index efficiency anyway.

##### Option B — Add `project_id NOT NULL` to all top-level state tables (workitems, handoffs, artifacts, sessions, canonical_events, checkpoints, pipeline_runs, review_threads, prompt_instances, documents)

Each table carries its own `project_id` FK to `projects.id`. Reconcile sets it on insert based on the file path (`docs/coordination/...` ⇒ seatloom project; future `docs/projects/<other>/coordination/...` ⇒ other projects).

**Pros**: simple queries; index-friendly; project switch is `WHERE project_id = $1` everywhere.
**Cons**: ~10 ALTER TABLE; back-fill required for existing rows (set all to `seatloom`); future tables must remember to include the column.

##### Option C — Add `project_id NULLABLE` for now; tighten to NOT NULL later

Same as B but allows NULL during migration. Existing seed and reconciled-from-`docs/coordination/` rows get `project_id='seatloom'`; future projects inherit their own ID; NULL means "unscoped".

**Pros**: gradual rollout; doesn't break existing pre-`project_id` data.
**Cons**: NULL semantics are confusing; "unscoped" might leak across projects in queries that forget the filter.

##### Recommendation: **Option B — `project_id NOT NULL` everywhere, with explicit migration**

**Rationale**:

1. **Multi-project is a stated P0 promise** (US-P0-03). NULL semantics weaken that promise.
2. **One ALTER TABLE per packet of related tables**: schema migration `006_multiproject_scope.sql` can add the column with `DEFAULT 'seatloom'` and then drop the default after back-fill. Single migration, no foot-guns.
3. **Aligns with Aegis's 2026-04-30 chan-04 review** which already noted `project_id` should be a first-class FK.

**Implementation note for Nimbus** (technical-architecture follow-up): the migration adds `project_id TEXT NOT NULL REFERENCES projects(id) DEFAULT 'seatloom'`, runs on a quiet window, then `ALTER COLUMN ... DROP DEFAULT`.

#### 3.3.2 Project switching UX

##### Option A — Project switcher in NavRail, AppShell-wide context

A single project is "active" at a time. All views (Inbox, WorkflowPanorama, Sessions, Documents) filter by the active project. Switching projects re-hydrates from backend.

##### Option B — Per-view project filter, no global active project

Each view has its own project filter chip. User can have Inbox showing seatloom while WorkflowPanorama shows another project.

##### Option C — Supervisor contextMode integration (per AD-013 v2)

`contextMode='global'` (multi-project digest) vs `contextMode='project'` (single project). Reuse AD-013 v2's existing UI mode for project selection; the existing breadcrumb shows current project.

##### Recommendation: **Option C — extend AD-013 v2 contextMode**

**Rationale**:

1. AD-013 v2 already specifies `contextMode = 'global' | 'project'` with a Supervisor breadcrumb. This is the only existing UI mode in the app for "single project vs cross-project view".
2. Building a parallel project switcher in NavRail (Option A) duplicates AD-013 v2's mechanism.
3. Per-view filters (Option B) violate AD-013 v2's mutual-exclusion invariant ("data isolation rules: Project view does not render other projects' data").
4. **The work needed**: Global mode shows aggregate digests across all projects (currently Mock data — see chan-03 acceptance); Project mode filters all queries by `activeProjectId`. The contextMode state machine is already implemented at `f5b8423` / `cadf36e`. Backend wiring is the gap.

**Maps to**: US-P0-03 (per-project role binding visible), US-P0-08 (Seat Card capability per project), AD-013 v2.

#### 3.3.3 New project initialization flow

##### Option A — `cmd_create_project(id, root_path)` Tauri command

User invokes from a "+" menu in NavRail; provides project ID and root path. Backend creates `projects` row, initializes `docs/coordination/` skeleton if absent, registers file watcher for that root.

##### Option B — Discover via tmux session pattern

Any tmux session matching `<seat>-*-<project>` introduces project `<project>` automatically. Backend creates a `projects` row on first attach.

##### Option C — Discover via filesystem pattern

Any directory under a configured `~/projects/` containing `docs/coordination/` is auto-registered as a project on app startup.

##### Recommendation: **Option A primarily; Option B as enrichment**

**Rationale**:

1. Explicit creation (Option A) makes project boundaries auditable. Aligns with PRD §9 non-goal "no silent autonomous work promotion".
2. Option B alone is too permissive — a one-off tmux session for an experiment shouldn't create a tracked project.
3. Option B as enrichment: when SeatLoom sees an unknown tmux session matching the seat naming pattern, surface a dismissable "Create project '<name>'?" toast in the Supervisor IM. User confirms; goes through Option A.
4. Option C is too invasive — directory scanning at startup is a surprise.

**Maps to**: US-P0-03 (multi-project, but with explicit boundary), tmux-mirror R3 (project creation does not break existing tmux state).

---

## 4. Historical migration

### 4.1 Context from PRD v0.5

The PRD does not address migration — it assumes greenfield. Since 2026-04-30, ~300 markdown files (T3 / T4 / T5 / T6 / T7) have accumulated in `docs/coordination/`. Reconcile already inserts them into `documents`, so they're searchable. They are **not** projected into `workitems` / `handoffs` / `canonical_events`, so WorkflowPanorama and Inbox don't see them.

### 4.2 Gap

If we adopt §1.4 Option C (event-first), the question becomes: should reconcile retroactively emit synthetic `WorkItemIssued` / `HandoffDelivered` / `AcceptanceIssued` events for all 300 historical documents, or only project forward from 2026-05-09?

### 4.3 Options

#### Option A — Forward-only

Reconcile emits events only for documents created after a "v0.1 cutover" timestamp. Historical documents remain searchable via the `documents` table but do not appear in WorkflowPanorama or Inbox.

**Pros**: zero risk of mis-ordered event timeline; clean cut over.
**Cons**: WorkflowPanorama and Inbox start empty on v0.1 launch even though 10 days of real coordination history exists; trust regression vs. seed mode.

#### Option B — Full retroactive back-fill

Run a one-time reconcile pass over the entire `docs/coordination/` tree, treating each document's header `date:` field as the synthetic event timestamp. Generate retroactive `WorkItemIssued` / `HandoffDelivered` / `AcceptanceIssued` rows in time order. Mark these events with `synthetic=true` flag.

**Pros**: WorkflowPanorama and Inbox become immediately useful on launch; users see their actual coordination history; tests exercise the pipeline end-to-end against real data.
**Cons**: requires careful handling of the "what status is this document's owning workitem now?" question (a single workitem may have multiple documents — issuance + delivery + acceptance — so the latest event in the chain wins). Synthetic events with stale `actor_ref` (we don't always know who the typing supervisor was historically) need a fallback.

#### Option C — Selective back-fill: T5 acceptances and current-week T3 only

Back-fill only the events that matter for the **current** state (acceptances are terminal, so the latest one defines current status; this-week T3 issuances matter for in-flight work). Pre-2026-05-02 issuances that already have an acceptance are summarized as a single `WorkItemAcceptedRetroactive` event.

**Pros**: smaller event set; cleaner WorkflowPanorama; less synthetic noise.
**Cons**: lose granular history for older work; users may want to drill into "what happened with chan-03?" and find a single retroactive event instead of the full sequence.

### 4.4 Recommendation: **Option B — Full retroactive back-fill, with `synthetic=true` flag**

**Rationale**:

1. **Trust matters more than tidiness**. Mr. Zhang's stated frustration ("数据是 4-27 到 4-29 的 seed 快照") makes clear that empty post-launch panorama is unacceptable.
2. **Synthetic-flagged events are reversible**: if a back-fill heuristic mis-classified a document, future reconcile passes can correct by replacing the synthetic row.
3. **Event timeline integrity is preserved**: each synthetic event uses the document's `date:` header (or git committed-at as fallback) as the canonical timestamp. The timeline is honest about when the work happened, even if the events themselves were materialized today.
4. **One-time cost, durable benefit**: the back-fill runs once per project; future projects inherit the same pipeline.

**Mapping rules** (for Nimbus to operationalize):

| Document template | Synthetic event emitted | actor_ref derived from |
|---|---|---|
| T3 `task` (issuance) | `WorkItemIssued` | `author:` field; default to `supervisor:zhang` |
| T3 `task` `*-delivery-v1.md` | `WorkItemDelivered` | sibling delivery doc `author:` |
| T3 `verification` | `VerificationDelivered` | sibling delivery `author:` |
| T5 `acceptance_review` | `WorkItemAccepted` | `author:` |
| T5 `gate_decision` | `StageGateDecided` | `author:` |
| T4 `gap_review` / `design_proposal` | `ReviewIssued` (no workitem mutation) | `author:` |
| T6 `daily_log` | `MemoryLogged` (no workitem mutation) | `author:` |
| T7 governance | `GovernanceIssued` (no workitem mutation) | `author:` |

**Idempotency**: synthetic events are keyed by `(document_id, event_type)`. Re-running the back-fill produces no duplicate rows.

**Maps to**: US-P0-01 (action queue based on real state), US-P0-05 (review-fail visible from acceptance docs), US-P0-10 (search yields historical evidence already projected as objects).

---

## 5. L1 concrete user flows

### 5.1 Context from PRD v0.5 + tmux-mirror

- §6.4 Supervisor Layer (PRD): Command Bar, suggestion cards, inline edit, confirmation.
- §3.1 (PRD): Supervisor IM is L1 — every sprint should strengthen it.
- AD-012: Interactive prompts are first-class state with `Approve / Human takeover / Supervisor assist / Stop`.
- AD-013 v2: viewMode `chat | dashboard` orthogonal to contextMode.
- Tmux-mirror R2: bidirectional read+write; UI input → `tmux send-keys`.

### 5.2 Three concrete flows that v0.1 must answer

#### 5.2.1 Supervisor sends a message to Lyra; Lyra's reply arrives

##### Step sequence (proposed)

1. User types `Lyra please review the new schema migration` in Supervisor IM (chat mode, `activeContact='lyra'`).
2. UI calls `cmd_append_supervisor_message(actor='supervisor:zhang', target='seat:lyra', content=..., projectId='seatloom')`.
3. Backend writes `canonical_events` row with `delivery_status='pending'` (per §1.5 Option B) — **T-Live for own-bubble echo**.
4. Backend dispatches `tmux send-keys -t Lyra-po-seatloom <content>\n` — **T-Near**.
5. Lyra's CLI receives keystrokes, processes, types reply.
6. Reply bytes flow `Lyra-po-seatloom pane → tmux pipe-pane fifo → SeatLoom backend tail-read → mirror parser`.
7. Mirror parser: silence-window heuristic (3 s of silence + non-ANSI content) ⇒ emit `cmd_append_supervisor_message(actor='seat:lyra', target=null, event_type='SeatResponse', content=<flushed chunk>)`.
8. UI subscribes to canonical_events for the active conversation; new SeatResponse event renders as Lyra's bubble — **T-Near**.

##### Open product decision: what is shown to user during steps 5-7?

Options:

- **A — Nothing**: bubble appears only when SeatResponse is materialized.
- **B — Typing indicator**: while the mirror sees output activity but hasn't crossed the silence-window threshold, show "Lyra is typing..." in IM.
- **C — Live xterm sidebar**: a collapsible xterm pane next to IM shows live raw output; the IM bubble materializes on flush.

##### Recommendation: **Option B — typing indicator only**

**Rationale**: Option C duplicates SessionTerminal (which exists separately); Option A makes 3-second pauses feel broken; Option B is the minimum signal that conveys "the seat is working" without exposing raw terminal noise. The L1 surface stays clean.

**Maps to**: US-P0-02 (intent → confirmation cycle perception), US-P0-11 (prompt-state visibility), tmux-mirror R2.

#### 5.2.2 User approves a plan-mode prompt from IM

##### Step sequence

1. Lyra's CLI hits `ExitPlanMode`.
2. Nimbus's ClaudeAdapter Layer A (chan-10 §6.1) reads the JSONL transcript tail, detects the tool call.
3. Backend INSERT `prompt_instances(kind='plan_approval', evidence_ref=<plan_body>, ...)`, emits Tauri event `prompt:detected`.
4. UI renders a Plan card *inline in the IM* (per AD-013 v2 chat mode) — **T-Near**.
5. User clicks `Approve`.
6. UI calls `cmd_append_prompt_action(prompt_id, kind='approve')`.
7. Backend INSERTS `prompt_actions` row, then runs `tmux send-keys '1\n'` into Lyra's session (per AD-012 + R2 write).
8. Mirror sees Lyra's CLI resume; emits subsequent `SeatResponse` events normally.

##### Open product decision: where does the Plan card live?

Options:

- **A — Inline IM bubble** (rendered as a card-style message in the chat).
- **B — Toast notification** (floating top-right, click to action).
- **C — Dedicated PromptInbox view** (persistent list of unresolved prompts in the sidebar).

##### Recommendation: **Option A primary + Option C as fallback list**

**Rationale**:

1. **Option A respects L1 primacy**: the user is already looking at the IM when their seat hits a prompt; the approval should land where their eyes are.
2. **Option C as a back-pocket**: when the user dismissed the IM panel (Supervisor detached, panel collapsed), they need a way back to outstanding prompts. PromptInbox is a small L2 view that lists active `prompt_instances`.
3. **Option B (toast) is rejected**: AD-012 requires "first-class state"; toasts are ephemeral and contradict that.

**Maps to**: US-P0-11 (prompt-state surface), AD-012, AD-013 v2 chat mode.

#### 5.2.3 WorkflowPanorama as a real-time view

##### Question

WorkflowPanorama (the 5-card / DAG visualization on the Dashboard) currently reads seed data. After §1.4 Option C + §4.4 back-fill, what powers it and how live is it?

##### Step sequence (proposed)

1. WorkflowPanorama component subscribes to a Tauri event `canonical_events:appended` with `projectId` filter.
2. Initial mount: fetch last 30 days of events via `cmd_list_canonical_events(project_id, since=now-30d)`.
3. New event arrives ⇒ store update ⇒ DAG re-derivation runs in the background (T-Near, ≤ 5 s).
4. Card counts (Active workitems, Blockers, In review, Done today) read from materialized views over the event log; views refresh on each event append.

##### Open product decision: panorama scope

Options:

- **A — Project-only**: filter by `activeProjectId`; mirror what the user is currently looking at.
- **B — Global aggregate**: all projects; counts roll up.
- **C — Switches with `contextMode`**: project mode shows project-only; global mode shows aggregate.

##### Recommendation: **Option C — switch with contextMode**

**Rationale**:

1. AD-013 v2 already establishes `contextMode='global'` with a `GlobalDashboard` and `contextMode='project'` with `ProjectDashboard`. WorkflowPanorama lives inside ProjectDashboard.
2. GlobalDashboard gets a *separate* PanoramaSummary that shows per-project counts (active project digest cards), not the DAG.
3. Each panorama variant has a clear scope — no cross-mode ambiguity.

**Maps to**: AD-013 v2, US-P0-01 (action queue at start of day), tmux-mirror R4 (auto-reconcile keeps the panorama live).

---

## 6. Cross-cutting decisions and non-goals (this supplement)

### 6.1 What this supplement does NOT decide

- **Mobile companion** (US-P0-12..15): out of v0.1 scope per gap review §1.4. Not addressed here.
- **Playbook system back-fill** (PRD §6.3): playbooks are P1; their migration is a future supplement.
- **Pgvector / semantic retrieval** (AD-011 layer 3): out of v0.1.
- **Cross-project authoring** (a workitem owned by two projects): not allowed; one project per workitem.
- **Implementation detail**: this is a product-design supplement. Nimbus's technical architecture supplement (forthcoming) decides Rust crate boundaries, table indexing, watcher debounce internals, etc.

### 6.2 Non-goals shared with PRD v0.5 §9 (still in force)

- No silent autonomous work promotion.
- No unrestricted cross-seat data access.
- No cloud-first coordination requirement.
- No replacement of IDEs.
- No ecosystem-protocol exposure before core value is proven.

### 6.3 What changes in PRD v0.5 going forward

This supplement does **not** rewrite the PRD. After Aegis review and Mr. Zhang sign-off, the following PRD anchors should be referenced from this supplement:

- §3.1 (L1/L2): no change. Real-time tiers (§2 here) are an implementation refinement under §3.1.
- §5 (story map): no change. The 15 P0 stories remain the contract.
- §6.1 (Data Engine): add a forward-reference: "for projection lifecycle, see `2026-05-09-lyra-seatloom-full-product-design-v1.md` §1".
- §6.4 (Supervisor): add a forward-reference to §5 here for concrete L1 flows.
- §6.5 (Artifact review): no change; this supplement aligns with dual-key typing.
- AD-007 (reconciliation triggers): explicitly **superseded** by tmux-mirror R4 + this supplement §1 + §2 (file watcher is now P0; manual is fallback).

Aegis's stage-gate decision after review should record the AD-007 supersession explicitly.

---

## 7. Open questions for Aegis review

1. **Event log backbone (§1.4 Option C)**: is event-first projection acceptable as the v0.1 architecture commitment, or is Option A (reconcile-only direct projection) preferred for ship-velocity?
2. **Real-time tier boundaries (§2.3)**: are 1 s / 5 s budgets the right cutoff numbers, or should we adopt 500 ms / 3 s? (Implications: T-Live tier shrinks to local UI only; T-Near becomes the workhorse.)
3. **Multi-project schema migration timing (§3.3.1)**: ship `006_multiproject_scope.sql` in v0.0.1, v0.0.2, or v0.1? My recommendation is v0.0.1 because every subsequent table interaction assumes `project_id`, but the SG-A packet list (gap review §4) does not currently include this migration.
4. **Historical back-fill timing (§4.4)**: run the back-fill as part of v0.1 launch (same release), or as a v0.0.2 utility command (`cmd_backfill_historical_events`) that the user invokes once?
5. **Plan card placement (§5.2.2)**: does AD-012 §"first-class state" formally exclude toasts? My reading is yes, but Aegis may want to amend AD-012 to clarify.

These five questions should reach the Nimbus technical-architecture supplement as constraints.

---

## 8. Hand-off to Nimbus

After Aegis review, this supplement plus the existing PRD v0.5 + tmux-mirror architecture (`a5998c1`) form the **product-side input** for Nimbus's technical-architecture supplement. Nimbus produces:

`docs/coordination/reviews/2026-05-09-nimbus-seatloom-technical-architecture-v1.md` (T4 design_proposal)

covering:

- Rust crate boundary changes (event projector, file watcher, materialized view machinery)
- PostgreSQL schema 006 (multi-project) and 007 (event-first projection if Option C accepted)
- Tauri command surface deltas
- File watcher implementation (notify crate, debounce, path filtering per §2.3)
- Back-fill command implementation per §4.4
- Adapter integration with §1.5 message lifecycle

Nimbus should answer the five open questions in §7 before any v0.1 implementation packet lands.

---

## 9. Done definition for this supplement

- [x] Each of Mr. Zhang's five core questions has 2-3 options, a recommendation, and a US-P0-xx anchor reference.
- [x] All recommendations are consistent with existing PRD v0.5 contracts and AD constraints (AD-008–013 v2).
- [x] AD-007 supersession is explicitly named.
- [x] Open questions for Aegis review are listed (§7).
- [x] Hand-off scope to Nimbus is defined (§8).
- [x] No implementation code in this document.

---

*Supplement issued by Lyra · 2026-05-09 · Extends PRD v0.5; awaits Aegis review. After Aegis sign-off → Nimbus technical-architecture supplement → v0.0.1/0.0.2/0.1 implementation packets.*
