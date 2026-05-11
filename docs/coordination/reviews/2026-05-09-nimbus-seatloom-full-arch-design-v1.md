# Design Proposal Supplement: SeatLoom v0.1 Full Architecture Gaps (extends AD-008..013 + architecture-design.md §3/§5/§6/§7/§8)

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | 2026-05-09-nimbus-seatloom-full-arch-design-v1 |
| status | issued |
| author | nimbus |
| date | 2026-05-09 |
| version | v1 |
| to | aegis, lyra |
| priority | P0 |
| depends_on | `docs/architecture-decisions.md` (AD-007, AD-008, AD-009, AD-010, AD-011, AD-012, AD-013), `docs/architecture-design.md` (§3 Rust core types, §3.3 CanonicalEvent / EventType, §4 AdapterTrait, §5.1 IPC commands, §5.2 Tauri events, §6 storage layer, §7.1 Zustand stores, §8 PTY manager, §11 implementation order, §12.3 retrieval contract), `docs/prd-v0.5.md` (US-P0-01, US-P0-05, US-P0-06, US-P0-09, US-P0-10, US-P0-11, US-P0-12, US-P0-13, US-P0-15, US-P1-05, US-P1-06, US-P3-01), `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (a5998c1 — R1-R5), `docs/coordination/reviews/2026-05-08-aegis-cli-plan-mode-integration-design.md` (chan-10 Layer A/B/C), `infra/postgres/schema/001_seatloom_core.sql`, `infra/postgres/schema/002_document_authority.sql`, `infra/postgres/schema/004_operational_review_and_continuity.sql`, A3 implementation at commit fca4fe0 (`crates/seatloom-core/src/pty/mod.rs`, `src-tauri/src/commands/session_cmds.rs`) |
| tags | architecture, supplement, v0.1, tmux-mirror, data-flow, file-watcher, multi-project, realtime, backfill, A3-known-limits |
| acceptance owner | Aegis |
| concurrency rule | Supplement only — does NOT redefine AD-008..013 or `architecture-design.md` §3/§5/§6/§7/§8. Each section names the existing anchor it extends, the gap, options, a recommendation, complexity, and PRD US-P0-xx alignment. Lyra writes the parallel product-side supplement extending `prd-v0.5.md` §3/§5/§6; Aegis does the joint review. |

---

## 0. Why this document exists, and what it is NOT

**Why**: The tmux-mirror architecture (a5998c1) and the SG-A v0.0.1 read path (`A3` packet, delivered at commit `fca4fe0`) opened concrete engineering questions that AD-007 / AD-011 / `architecture-design.md` §3 / §5 / §6 / §8 do not yet specify. Specifically: how PTY bytes become structured rows in `canonical_events`; how `docs/coordination/**/*.md` writes propagate into PostgreSQL without a manual reconcile click; whether multi-project isolation needs a schema migration or can ride on `project_role_bindings`; whether multi-window broadcast (main shell + detachable Supervisor IM) needs PG `LISTEN/NOTIFY` or stays inside the Tauri event bus; whether the 291 documents already in PostgreSQL should backfill `workitems` / `handoffs`; and the three known limits the A3 mirror primitive carries forward.

**What this is NOT**: a redesign. AD-008..013 stand. `architecture-design.md` §3 (Rust core types), §4 (AdapterTrait), §5 (IPC commands + events), §6 (storage layer), §7 (frontend stores), §8 (PTY manager) stand. Every section below names the existing decision it extends, where the gap sits, and a recommendation written so AD-XXX or §X can be amended in place.

**Companion product-side supplement**: Lyra is filing the user-facing extension (`prd-v0.5.md` §3 priority boundaries, §5 story map, §6 module contracts) in parallel. That document and this one are reviewed jointly by Aegis. Where the two intersect (e.g., US-P0-12 mobile monitoring → realtime delivery in §4 of this doc), the cross-link is named explicitly.

**Implementation status alignment**: A3 is committed (`fca4fe0`). The tmux read path is live and validated by smoke. Sections §1, §2, §6 here describe the next layers that build on that primitive. Sections §3, §4, §5 are independent of A3 and can advance in parallel.

---

## 1. Data Flow Pipeline — PTY bytes → structured `canonical_events`

### Anchors this extends

- **AD-011 retrieval contract**: requires L1 structured-index queries to land before L2 full-text. L1 needs structured events, not raw byte logs.
- **`architecture-design.md` §3.3 CanonicalEvent / EventType**: defines the canonical Ledger row shape and an `EventType` enum that today covers session/handoff/workitem/artifact/checkpoint lifecycle plus `PromptDetected` and `PromptInputInjected` (AD-012). It does not cover *runtime activity* between session start and end.
- **`architecture-design.md` §4 AgentAdapter trait**: declares `parse_transcript(&self, raw_output: &[u8]) -> Vec<TranscriptEntry>` — a per-runtime hook today returning `TranscriptEntry`, not `CanonicalEvent`.
- **2026-05-08 chan-10 design proposal §1**: states explicitly *"PTY is a transport, not a protocol. We need a second layer that sees the protocol."* — this gap is where that second layer lives.
- **A3 read path** (commit `fca4fe0`, `crates/seatloom-core/src/pty/mod.rs`): `PtySession` broadcasts `PtyEvent::Output(Vec<u8>)` from the FIFO tail. Nothing today turns those bytes into `canonical_events` rows.

### Gap

Three kinds of signal need to land in `canonical_events` from tmux-attached sessions:

1. **Session activity heartbeat** — *"this session emitted bytes within window W"* — needed by Inbox routing (AD-013 v2 dashboards), by Handoff `working` state (US-P1-06), and by detection of stalled sessions (US-P0-15 interrupt triage).
2. **Seat structural moments** — *"plan was proposed", "tool was invoked", "model produced a final response"* — needed by US-P0-01 (start-of-day Inbox), US-P0-05 (review-failure linkage), US-P0-11 (prompt classification firing `PromptDetected`).
3. **Idle window detection** — *"silence ≥ N seconds while session status is Running"* — needed by AD-012 (Prompt detection trigger) and by US-P1-06 live activity overlay.

Today's `EventType` enum has no `SessionActive`, no `SeatResponseEmitted`, no `AdapterToolCall`, no `IdleWindowEntered`. The byte stream has no classifier, no debouncer, no idle clock.

### Options

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **A. Single byte-stream classifier in `seatloom-core`** | One regex/heuristic state-machine over `PtyEvent::Output` chunks. Emits `EventType` rows directly. | One implementation; ships fast. | Fragile per-runtime (Claude Code, Codex, Gemini, Cursor each render differently). Misclassifies on alt-screen redraw. Hard to evolve. |
| **B. Adapter-owned classifier (extend §4 trait)** | Add `fn parse_byte_stream(&self, chunk: &[u8], state: &mut Self::ParserState) -> Vec<CanonicalEvent>` to `AgentAdapter`. Each adapter owns its own state machine. | Per-runtime fidelity. Aligns with the existing `parse_transcript` slot. ClaudeAdapter can fall back to JSONL tail (chan-10 §2.1) when available. | Three classifiers to write. Coordination required between byte-stream events and JSONL events to avoid duplicates. |
| **C. Hybrid two-channel model (recommended)** | Byte-stream classifier (Option A semantics) emits *liveness* events only (`SessionActive`, `IdleWindowEntered`). Structural events (`SeatResponseEmitted`, `PromptDetected`, `AdapterToolCall`) come from adapter-specific structured sources (Claude JSONL tail; Codex `~/.codex/sessions/*.json`; Gemini `~/.gemini/cache/`). The two channels are joined in `canonical_events` by `session_id`. | PTY classifier stays simple and runtime-agnostic. Structural channel is high-fidelity per chan-10 §2. PTY remains the only required source for runtimes without structured logs. | Slightly more code (two channels, one join). Adapters that lack a structured channel degrade to liveness-only. |

### Recommendation

**Option C (hybrid two-channel)**, which extends two existing decisions in place:

1. **Extend `EventType` (§3.3)** by appending: `SessionActive`, `IdleWindowEntered`, `IdleWindowExited`, `SeatResponseEmitted`, `AdapterToolCallStarted`, `AdapterToolCallCompleted`. No removals, no renames; existing variants keep their semantics.
2. **Extend `AgentAdapter` (§4)** by adding `fn parse_byte_stream(&self, chunk: &[u8], state: &mut AdapterParserState) -> Vec<CanonicalEvent>` with a default impl that returns `vec![]` for adapters that have nothing to add beyond liveness. The byte-stream→liveness pipeline lives in a new `seatloom-core/src/data_engine/liveness.rs` and runs against every attached session regardless of adapter.

**Concrete byte triggers** (proposed defaults, Lyra/Aegis tunable per project):

| Event | Trigger | Notes |
|---|---|---|
| `SessionActive` | Any chunk arrives with `bytes.iter().any(|b| !b.is_ascii_whitespace() && *b != 0x1b)` while session in `Running` and ≥ 1s since last `SessionActive` | Coalesced — one row per second of activity, not per chunk |
| `IdleWindowEntered` | No `SessionActive` for `idle_threshold_ms` (default 8 000 ms) | Feeds AD-012 prompt detection candidate window |
| `IdleWindowExited` | First `SessionActive` after `IdleWindowEntered` | Symmetry; useful for replay |
| `SeatResponseEmitted` (via JSONL tail) | ClaudeAdapter sees `{"type":"assistant","message":{...}}` line | High-confidence; not byte-derived |

**Coordinating PTY bytes with JSONL tail (ClaudeAdapter)**: bytes feed liveness; JSONL feeds structure. A single `session_id` is the join key. Order-preservation is by `occurred_at` (server clock at insert time). De-dup is unnecessary because the two channels never produce the same `EventType` for the same fact.

### Complexity estimate

**Medium**. ~600–900 LOC across `seatloom-core/src/data_engine/liveness.rs`, an `EventType` enum extension, an `AdapterParserState` placeholder, and ClaudeAdapter JSONL coordination glue. PG schema unchanged (`canonical_events` already has `event_type TEXT NOT NULL` and `payload JSONB`). PostgreSQL writes use the same `event_repo.insert` path that `cmd_reconcile` uses. New unit tests: byte-classifier idle/active state machine; integration test: smoke harness extension that asserts `IdleWindowEntered` after silence + `SessionActive` after `send-keys`.

### PRD coverage

- **US-P0-11** (interactive prompt) — needs `IdleWindowEntered` to fire AD-012 prompt classification on a *bounded* window (last 10–20 lines + Tier 0–2). Without this, AD-012 has no trigger.
- **US-P0-01** (start-of-day Inbox) — Inbox needs structural moments, not raw bytes, to render as readable rows.
- **US-P1-06** (handoff `working` state with live activity overlay) — `SessionActive` is exactly the heartbeat US-P1-06 §3 calls for.
- **US-P0-15** (interrupt triage) — `IdleWindowEntered` is one of the three classes (`Prompt blocked`, `Gate needed`, `Handoff pending`) the mobile interrupt path watches.

---

## 2. File Watcher Architecture — `notify` granularity, debounce, header parsing

### Anchors this extends

- **AD-007 Reconciliation contract (updated)**: three-trigger model — application start, before pipeline, manual button. PostgreSQL write path is the canonical path for `Repo markdown → documents` table. **Explicitly states "MVP 不做文件监听 (无 daemon)"**.
- **`a5998c1` tmux-mirror architecture R4**: contradicts AD-007's "no daemon" stance. *"File-system watchers (`notify` crate) on `docs/coordination/` tree trigger `cmd_reconcile` automatically. Manual reconcile button (Mira's §C commit `2f83624`) becomes fallback, not primary."*
- **`architecture-design.md` §6.1 `.seatloom/` directory structure**: defines on-disk layout but is silent on watching it.
- **`architecture-design.md` §11 implementation order**: Phase 0–4 timeline does not include a watcher phase.

### Gap

Three concrete questions are unspecified:

1. **Granularity** — recursive watch on `docs/coordination/**` only? Also `docs/*.md` (PRD, architecture decisions)? Also `.seatloom/ledger/events.jsonl`?
2. **Debounce + de-dup** — editors emit multi-event saves (e.g., VSCode does write+rename). What is the coalesce window?
3. **Header parsing for incremental reconcile** — do we always run a full reconcile on any change, or do we parse the file header (`template`, `subtype`, `id`, `status`) and dispatch only that document's row? The latter is required so a single edit doesn't trigger a 291-document scan.

### Options

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **A. Naive `notify::RecursiveMode::Recursive`, full reconcile on any event** | Single watcher; any event → `cmd_reconcile`. | Trivially correct (reconcile is idempotent per AD-007). | Latency: a 291-doc scan on every keystroke save is wasteful and noisy. Redraws the Inbox unpredictably. |
| **B. Recursive watcher + debounce + per-file partial reconcile (recommended)** | One watcher rooted at `docs/`. Debounce 500 ms. On debounced fire, parse only the *changed* file's header, run the document repo's `upsert(file_path, body, header)` path, emit the appropriate `canonical_event` (`DocumentRevised`, etc.). Full reconcile remains the manual fallback. | Bounded latency; bounded cost; aligns with AD-007 PostgreSQL write contract; partial reconcile is already the inner loop of full reconcile. | New `seatloom-core/src/watcher.rs` module; restart durability needed (option B+). |
| **B+. As B, plus a journaled queue at `.seatloom/watcher-queue.jsonl`** | Each watcher event is appended to a journal before processing. Restart resumes by replaying tail. | Survives SeatLoom crash mid-reconcile. Same idempotency guarantees as AD-007. | One extra file to GC; minor I/O overhead. |
| **C. Drop `notify`; use PostgreSQL `LISTEN/NOTIFY` only** | External writers signal via DB. | No FS watcher to maintain. | Doesn't solve the actual problem: external writers are markdown files, not DB rows. The document is the producer. |

### Recommendation

**Option B+ (recursive watcher + debounce + per-file partial reconcile + journaled queue)**.

**Concrete spec**:

- Crate: `notify = "6"` (already Tauri-friendly; cross-platform abstraction over inotify/FSEvents/ReadDirectoryChangesW).
- Watcher root: project repo root (Tauri `state.repo_root`). Filter: `docs/**/*.md` and `.seatloom/ledger/events.jsonl`. Configurable per project later.
- Debounce: 500 ms. Implementation: `notify-debouncer-mini` or hand-rolled timer.
- Per-file path:
  1. Read first 50 lines of the file.
  2. Parse the standard markdown header (`template`, `subtype`, `id`, `status`, `date`, `version`, `tags`) — same parser that `cmd_reconcile` uses today (AD-007 PostgreSQL contract).
  3. Call `documents.upsert(...)` (existing repo).
  4. Conditionally emit `canonical_events`:
     - **T3 task packet (subtype: implementation/delivery, status: issued/delivered)** → upsert a corresponding `workitems` row (see §5 below). Emit `WorkItemCreated` or `WorkItemStatusChanged`.
     - **T5 acceptance (verdict: PASS|FAIL|HOLD)** → emit `ReviewVerdictIssued` (AD-010) with `verdict`, `reason`, `linked_evidence_artifact_id` populated from the document `id` and `pairs_with` field.
- Restart durability: `.seatloom/watcher-queue.jsonl` append-only. On startup, replay tail then resume the watcher. GC: rotate when > 10 MB.

**Failure mode**: watcher crashes silently on Linux when inotify limits are hit (typically `fs.inotify.max_user_watches` < tree depth). Detect via `notify`'s error stream and surface as a banner; manual reconcile remains the fallback per AD-007.

### Complexity estimate

**Medium-high**. ~800–1 200 LOC: `seatloom-core/src/watcher.rs` (watcher + debounce), header→workitem/event mappers (small if §5 lands first), Tauri command `cmd_start_watcher` / `cmd_stop_watcher`, journal queue. PG schema unchanged. New tests: debouncer behavior, restart-replay correctness, header parser failure modes.

### PRD coverage

- **US-P0-01** (start-of-day) — Inbox should reflect the latest writes within seconds, not on next manual click.
- **US-P0-05** (review failure linkage) — T5 acceptance write should propagate to a `ReviewVerdictIssued` event automatically.
- **US-P0-07** (in-app artifact view) — relies on `documents` row freshness.
- **US-P1-06** (handoff `working` overlay) — needs the watcher path so a delivery doc edit shows up immediately on the receiving seat's Inbox.

---

## 3. Multi-Project Data Isolation — schema strategy

### Anchors this extends

- **`infra/postgres/schema/001_seatloom_core.sql`**: `projects` table exists; `seats` is global per AD-009 Layer 1; `project_role_bindings` already has `(seat_id, project_id)` PK per AD-009 Layer 2; **`workitems`, `sessions`, `handoffs`, `canonical_events` have no `project_id` column**.
- **`infra/postgres/schema/002_document_authority.sql`**: `documents` already has `project_id TEXT NOT NULL` with `idx_documents_project`.
- **`infra/postgres/schema/004_operational_review_and_continuity.sql`**: `review_threads` already has `project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE`.
- **AD-013 v2 SupervisorContextMode/ViewMode**: Project-mode invariant — *"Project view 不得在左侧 Contact 列表之外展示其他 project 的数据"*. The frontend invariant is real; the backend doesn't enforce it.

### Gap

A WorkItem today has no direct `project_id` relationship. To answer *"which project is this WorkItem in?"* we must join `workitems.owner_seat_id → seats → project_role_bindings → project_id`. That fails when `owner_seat_id IS NULL` (drafts) and ambiguates when a seat has bindings in multiple projects (which is the steady state — Lyra is bound to multiple projects). Same gap on `sessions` and `handoffs`.

This is why AD-013 v2 has to be enforced at the UI store level (`activeProjectId`) — there is no DB-side guard.

### Options

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **A. Add `project_id` FK column to `workitems` / `sessions` / `handoffs` / `canonical_events` (schema migration 008)** | New column `project_id TEXT NOT NULL REFERENCES projects(id)`. Index `(project_id, status)` per table. Backfill: WorkItem.project_id ← join via `owner_seat_id` → `project_role_bindings.project_id`; default to `'seatloom'` (the only current project) when ambiguous. | Single source of truth at SQL level. Indexes make AD-013 project-mode queries cheap. Aligns `workitems`/`sessions`/`handoffs` with how `documents`/`review_threads` already work. AD-013 v2 invariants become enforceable in `WHERE` clauses. | Schema migration + repo changes + backfill script. Existing seed data needs updating. |
| **B. Indirect via JOIN** | Keep schema flat. Every query joins `project_role_bindings`. | Cheap migration. | Brittle on delegations (AD-009 Layer 3 — `to_seat_id` may not have a binding in the from-seat's project). Performance: every list query becomes a 3-table join. Drafts (no owner) can't be scoped. |
| **C. Schema-namespace per project** | `project_p1.workitems`, `project_p2.workitems`, … | Hard isolation by SQL search_path. | Migrations multiply linearly with project count. Cross-project supervisor queries (AD-013 Global mode) become unions across schemas. `pgcrypto` / extensions need per-schema setup. No real benefit over Option A. |

### Recommendation

**Option A**, sequenced as schema migration 008 with a backfill that uses the steady-state heuristic:

```sql
-- 008_project_isolation.sql (sketch)
ALTER TABLE workitems       ADD COLUMN project_id TEXT REFERENCES projects(id);
ALTER TABLE sessions        ADD COLUMN project_id TEXT REFERENCES projects(id);
ALTER TABLE handoffs        ADD COLUMN project_id TEXT REFERENCES projects(id);
ALTER TABLE canonical_events ADD COLUMN project_id TEXT REFERENCES projects(id);

-- Backfill (single-project today, so the heuristic is conservative):
UPDATE workitems SET project_id = COALESCE(
  (SELECT prb.project_id FROM project_role_bindings prb
    WHERE prb.seat_id = workitems.owner_seat_id LIMIT 1),
  'seatloom'
) WHERE project_id IS NULL;
-- (similar for sessions, handoffs; canonical_events backfilled via event_object_refs)

ALTER TABLE workitems ALTER COLUMN project_id SET NOT NULL;
-- ... and indexes ...
CREATE INDEX idx_workitems_project_status ON workitems(project_id, status);
CREATE INDEX idx_sessions_project_status  ON sessions(project_id, status);
CREATE INDEX idx_handoffs_project         ON handoffs(project_id);
CREATE INDEX idx_events_project_occurred  ON canonical_events(project_id, occurred_at DESC);
```

**Authoring rule going forward**: every write path that creates a new `workitem` / `session` / `handoff` / `canonical_event` MUST set `project_id` from the calling `AppState.active_project_id` (which the Tauri command surface receives via the existing supervisor `currentContextMode` plumbing per AD-013 v2 §7 trigger table).

**AD-013 v2 backend enforcement**: project-mode list queries SHALL include `WHERE project_id = $1`. Global-mode queries (Supervisor cross-project summary) SHALL group by `project_id`. This is the SQL-level mirror of the AD-013 v2 frontend invariant and closes the gap noted in `2026-04-30-lyra-collaboration-dataflow-backend-traceability-report.md`.

**Single-project today**: backfill defaults to `'seatloom'`. As soon as a second project lands, the routing logic in `cmd_create_workitem` etc. must take `project_id` from the active context — this is enforced at the API surface, not at the SQL constraint level (the FK only ensures a real project, not the *right* project).

### Complexity estimate

**Medium-high**. Migration 008 is mechanical. Repository changes touch `workitem_repo`, `session_repo`, `handoff_repo`, `event_repo` (each gains a `project_id` parameter on insert and a `for_project` filter on list). DTOs in `src-tauri/src/dto.rs` gain the field. UI store `projectStore` already has `activeProjectId` per `architecture-design.md` §7.1 — no UI breakage if the backend defaults to it. Estimated 600–900 LOC + migration + tests.

### PRD coverage

- **US-P3-01** (parallel module workstreams): explicit P3 demand to scope work by `project_id` so module owners can't accidentally see another module's WorkItems. Without Option A this is impossible to enforce server-side.
- **AD-013 v2 §6/§7 invariants**: today these are only enforced in the React store. Option A makes them defendable at the API.
- **US-P0-12 / US-P0-13** (mobile): the `getInbox` query needs project scoping — without Option A, a mobile client gets cross-project leakage.

---

## 4. Realtime Architecture — Tauri event bus, throttling, multi-window broadcast

### Anchors this extends

- **`architecture-design.md` §5.2**: defines six Tauri events (`pty:output`, `session:status_changed`, `ledger:new_event`, `inbox:updated`, `pipeline:progress`, `reconcile:completed`).
- **A3 commit `fca4fe0`** (`src-tauri/src/commands/session_cmds.rs:182-204`): emits `session:output` with `{ session_id, data: <base64> }`. The base64 wrapper is intentional (preserves ANSI/UTF-8 fidelity through the JSON layer).
- **Memory pin: "Supervisor IM must be detachable into its own window"** — multi-window is a hard requirement (`feedback_im_detachable_window.md`). Tauri 2 multi-window broadcast (`app.emit`) auto-fans-out across all windows of the app, but per-window filtering still needs design.

### Gap

Three unspecified decisions:

1. **Throughput**: a busy `cmd_attach_tmux_session` can emit `session:output` events every few milliseconds for a verbose seat (typing, color renders). What's the right coalesce policy so the UI stays responsive and the channel doesn't backpressure?
2. **Multi-window**: when the Supervisor IM is detached, both the main shell and the IM window need `inbox:updated` and `ledger:new_event`. Same payload, two consumers. Is `app.emit()` enough, or do we need a topic-routing layer?
3. **Cross-instance / cross-process**: the watcher (§2) and any out-of-band CLI tool (`seatloom reconcile`) write to PostgreSQL. The running Tauri app should react. Tauri events don't reach across processes.

### Options

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **A. Tauri `app.emit()` only** | Every event uses `app.emit(name, payload)`. Each window subscribes via `listen()`. Frontend filters by `session_id` / window. | Simple. Already in place. | No throttling on `session:output`; UI freezes possible on bursty seat output. No cross-process reach. |
| **B. PG `LISTEN/NOTIFY` only** | All events flow through PostgreSQL pub/sub. Tauri windows each hold a `LISTEN`. | Cross-process. Single source. | Round-trip latency for in-app events. PG connection per window. Payload size limited (~8 KB on default config). |
| **C. Hybrid (recommended)** | In-app: `app.emit()` as today, with a per-event-name throttle in the emitter. Cross-process: dedicated PG `LISTEN/NOTIFY` channel `seatloom_events` for events that originate outside the running app (watcher, CLI). The Tauri backend listens on one PG connection and re-emits via `app.emit()`. | Low in-app latency, multi-window broadcast for free, cross-process reach for the cases that need it. One emitter API for the frontend. | One extra PG connection in the pool dedicated to LISTEN. Throttle policy needs care so `session:output` chunks don't merge across sessions. |

### Recommendation

**Option C (hybrid)** with concrete throttle policy.

**Throttle policy for `session:output`**:

| Trigger | Action |
|---|---|
| Chunk size ≥ 4 KiB | Emit immediately. |
| Time since last emit for this `session_id` ≥ 16 ms | Emit immediately. |
| Otherwise | Buffer; emit on next 16 ms tick. |

This keeps the UI in sync with terminal output at 60 fps without flooding the IPC channel for high-rate compile output. Policy lives in `src-tauri/src/commands/session_cmds.rs` (the existing `tokio::spawn` forwarder loop). One per-session `tokio::time::Interval` is enough.

**`LISTEN/NOTIFY` channel**: single channel `seatloom_events`. Payload: `{"kind": "<event_name>", "ref": <object_id>}` — small fixed shape that respects PG's 8 KB NOTIFY payload limit. Receivers on the backend look up the actual data in PostgreSQL. This avoids leaking large payloads through NOTIFY and gives consistent `architecture-design.md` §5.2 event names on the frontend regardless of origin.

**Multi-window**: Tauri 2 `app.emit()` already fans out to all windows of the same app. The detached Supervisor IM window subscribes to the same event names; per-window filtering happens in the React listener (e.g., main shell ignores `supervisor:im:appended`, IM window ignores `pipeline:progress`). No new event-bus layer needed.

**Backpressure for `ledger:new_event` during catch-up**: when the watcher (§2) replays a journal of, say, 50 queued events on startup, emitting them one-by-one through Tauri causes 50 round trips. Add a coalesced variant `ledger:new_events_batch` (`Vec<CanonicalEvent>`) emitted when the burst is ≥ 10 events within 100 ms. Frontend appends in one Zustand reducer call.

### Complexity estimate

**Medium**. Throttle: ~150 LOC + tests. PG `LISTEN/NOTIFY` plumbing: ~250 LOC + dedicated pool connection + reconnect handling. No frontend change beyond an optional `ledger:new_events_batch` listener.

### PRD coverage

- **US-P0-12** (mobile health monitor) — needs cross-process reach; the future mobile gateway will subscribe to PG `LISTEN/NOTIFY` regardless of which Tauri window is open.
- **US-P0-13** (mobile approval) — same.
- **US-P0-15** (interrupt triage) — needs `IdleWindowEntered` (from §1) to deliver promptly; throttle policy must not delay `IdleWindowEntered` events.
- **US-P1-06** (handoff `working` activity) — relies on `session:output` throttling so the UI doesn't lag on long-running seats.

---

## 5. Historical Backfill — 291 documents → `workitems` / `handoffs` / events

### Anchors this extends

- **AD-007 (updated)**: PostgreSQL is the canonical structured store; reconcile is the bounded import operation. AD-007's contract covers `documents` but is silent on whether `workitems` / `handoffs` / `canonical_events` derive from documents or are independent.
- **`architecture-design.md` §3.2 WorkItem / Handoff types**: defines structure but says nothing about how rows are created.
- **AD-008 dual-key artifact typing**: defines T1–T7 templates. T3 = task packet, T5 = acceptance. These templates *contain* the data needed to create WorkItems and review verdicts.
- **AD-010 review-failure event-first contract**: `ReviewVerdictIssued` is the event-first record of any acceptance. It has a slot for `linked_evidence_artifact_id` — the T5 document is exactly that evidence.
- **Empirical state**: `documents` table has 291 rows from existing reconcile runs; `workitems` / `handoffs` have only seed rows. Inbox is largely empty in the running app.

### Gap

Today every T3 task packet is a row in `documents` but not in `workitems`. Every T5 acceptance is a row in `documents` but emits no `ReviewVerdictIssued`. The Inbox / WorkItem / Handoff views are therefore approximately empty even though 291 documents fully describe the project's coordination history.

Two questions:

1. **One-shot vs continuous**: do we run a single backfill migration that walks the 291 documents and materializes `workitems` / `handoffs` / `canonical_events`, or do we make the watcher / reconcile path always derive these rows whenever it sees a T3/T5 document?
2. **Source of truth**: if both the document body and the workitem row exist, which wins on conflict?

### Options

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **A. One-shot migration script** | Walk `documents` once. For each T3 packet → upsert `workitems` row. For each T5 acceptance → insert `ReviewVerdictIssued` event. Ship with v0.1. | Bounded; one-time cost. | Re-run risk: a manual `seatloom reconcile` after migration must not double-insert. New documents post-migration would still need the watcher path. |
| **B. Continuous in `cmd_reconcile`** | Reconcile derives WorkItems/Handoffs from documents on every run, idempotently (`upsert` on document `id`). | Same code path forever; no special-case migration. | Each reconcile pass scans documents for derivation; cost grows with 291 → 1 000 → … docs. Doesn't help for past data the first time the user opens v0.1 (until they click reconcile). |
| **C. Hybrid (recommended)** | "Forward propagation" is the steady state — watcher (§2) and `cmd_reconcile` always derive `workitems` / `handoffs` / `ReviewVerdictIssued` rows from T3/T5 documents idempotently. A one-shot bootstrap runs on the *first* schema migration that adds the project_id column (§3) so v0.1 ships with non-empty Inbox/WorkItems views. After that, the steady-state path handles everything. | Idempotent in both modes. v0.1 ships with full history visible. New documents flow through the same code path. | Slight duplication: bootstrap and reconcile call the same derivation function — but this is the right kind of duplication (symmetry). |

### Recommendation

**Option C**, with these explicit derivation rules:

| Source document | Derived row | Mapping |
|---|---|---|
| T3 packet (`subtype: implementation \| design_proposal \| delivery`, `status: issued`) | `workitems` row, `status: 'Active'` | `workitems.id ← document.header.id`, `title ← header.title or document.title`, `goal ← document.body §Goal/§Context`, `acceptance_criteria ← §Acceptance Criteria parsed`, `owner_seat_id ← header.to seat name → seat.id`, `priority ← header.priority`, `created_at ← header.date`, `updated_at ← document.updated_at`, `project_id ← derived per §3` |
| T3 packet (`status: delivered`) | `workitems` row update + `WorkItemStatusChanged` event | Status `Active → InReview` |
| T5 acceptance (`verdict: PASS`) | `canonical_events` row | `event_type: 'ReviewVerdictIssued'`, `payload: {verdict: 'pass', reason, linked_evidence_artifact_id: <delivery doc id>}` per AD-010 |
| T5 acceptance (`verdict: FAIL \| HOLD`) | `canonical_events` row | Same as above with appropriate verdict |
| T3 handoff (`subtype: handoff`) | `handoffs` row | `from_ref ← header.from`, `to_ref ← header.to`, `workitem_id ← header.workitem_id or pairs_with`, `purpose ← §Purpose`, `expected_outcome ← §Expected Outcome`, `status ← 'Sent'` |

**Idempotency**: each upsert keys on `document.id` (the T3/T5 packet's own id field). Re-running on the same document is a no-op except for `updated_at`.

**Conflict policy** when a `workitem` row is also written directly via `cmd_create_workitem`: the document is authoritative. UI flows (Supervisor command bar US-P0-02) that create WorkItems do so by writing the T3 packet markdown; reconcile/watcher then creates the row. There is no second write path.

### Complexity estimate

**Medium**. Derivation logic ~400–600 LOC in `seatloom-core/src/data_engine/derive.rs`. Bootstrap is the same function called over all 291 rows once. Tests: derivation idempotency, header parser robustness on real coordination docs.

### PRD coverage

- **US-P0-01** (start-of-day Inbox) — empty Inbox is dead UX. Bootstrap makes v0.1 land with the Inbox already populated from history.
- **US-P0-05** (review-failure linkage) — `ReviewVerdictIssued` events are exactly what US-P0-05 §3 reads to render the failure card. Without backfill, the user sees no historical failures.
- **US-P0-10** (search exact evidence) — retrieval L1 by `template`/`subtype`/`status` only works if those rows exist. Backfill enables AD-011 L1 against the existing 291-doc corpus.
- **US-P1-04** (supervisor continuity pack) — pack assembly reads recent WorkItems and unresolved blockers. Empty WorkItem table = empty pack.

---

## 6. A3 Known Limits — pipe-pane redraw, FIFO offset, multi-pane

### Anchors this extends

- **A3 implementation** at commit `fca4fe0` (`crates/seatloom-core/src/pty/mod.rs`):
  - `attach_tmux` runs `tmux pipe-pane -t <name>:0 -o 'cat >> <fifo>'` (first pane only).
  - FIFO recreated on each attach via `mkfifo`; pre-restart bytes not preserved.
  - Tail-reader is a blocking std::thread reading 4 KiB chunks, broadcasting `PtyEvent::Output(Vec<u8>)`.
  - `kill()` stops `pipe-pane` and unlinks the FIFO; tmux session untouched (R3).
- **`architecture-design.md` §8 PTY manager**: original portable-pty design; superseded by tmux-mirror but the §8 description still lives there.

### Gap 6a — `pipe-pane` + alt-screen redraw

Alt-screen applications (`vim`, `less`, `htop`, the Claude Code TUI itself) emit ANSI cursor escapes and partial-screen redraws, not line-oriented logs. The byte stream is correct for terminal replay but doesn't yield structured events directly.

#### Options

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **6a-A. Strip ANSI to plaintext line stream** | `vte::Parser` (or `vt100` crate) consumes the byte stream and emits printable lines as a side channel. | Byte stream stays raw; line stream is regex-able. | Doesn't recover lost lines on redraw (cursor-based UIs overwrite). |
| **6a-B. Full terminal emulator state** | Run a headless terminal emulator on the byte stream; diff screen state to extract structural events. | Lossless. Same approach as `screen` / `tmux capture-pane`. | High complexity for marginal gain. |
| **6a-C. Defer to adapter (recommended)** | ClaudeAdapter and friends parse their own structured logs (chan-10 §2) and don't rely on byte extraction for structural events. The byte stream provides liveness only (per §1 above). | Aligns with §1's hybrid two-channel decision. Zero new code. | Adapters without structured logs (Cursor CLI?) get liveness only — but that is the documented v0.1 scope. |

#### Recommendation

**6a-C (defer to adapter)** for v0.1. Revisit `vte::Parser` as a P1 enhancement only if a runtime appears that has no structured channel and needs structural events from PTY bytes alone. **Complexity: low (zero code in v0.1)**.

### Gap 6b — FIFO offset / scrollback on restart

When SeatLoom restarts, the new `attach_tmux` call recreates the FIFO. Pane bytes emitted between SeatLoom crash and SeatLoom restart are lost from the live mirror.

#### Options

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **6b-A. Always-on transcript file** | Today A3 already writes `<transcripts_dir>/<id>.raw.log` from the tail-reader. On restart, replay tail (last N KB) into the new subscriber. | Already 80% there — file exists. | Bytes written between crash and restart still missing from the *file* (because the reader is what writes it, and the reader is dead). |
| **6b-B. tmux `capture-pane -S -<rows> -p` on attach (recommended)** | At the moment of `attach_tmux`, dump the current pane scrollback (limited by tmux's `history-limit`, default 2 000 lines) and inject it as the first `PtyEvent::Output` chunk before live tailing. | Fills the gap with whatever tmux itself has retained — which is exactly what the human sees if they `tmux attach`. | Bounded by tmux's `history-limit`. Long crash gaps with high-volume output may still lose tail. |
| **6b-C. tmux `pipe-pane` to a *file* not a FIFO** | `tmux pipe-pane -O 'cat >> <log>'` (no FIFO). SeatLoom tails the file with offset persistence. | Survives SeatLoom crashes natively. | Larger disk footprint over a long-running tmux session. Concurrency with multiple SeatLoom instances on the same session. |

#### Recommendation

**6b-B (`capture-pane` backfill on attach)** as the v0.0.2/v0.1 enhancement. This requires one extra `tmux capture-pane -S -<rows> -p -t <target>` call inside `attach_tmux` before the pipe-pane setup; the captured stdout is converted to a `PtyEvent::Output` chunk and pushed into the broadcast channel before any live bytes. The transcript file (6b-A) stays as the authoritative replay record.

**Complexity: medium**. ~80 LOC + a `history_lines: usize` field on `LaunchOptions` (default 1000). Tests: smoke harness extension that asserts pre-attach text appears in the receiver.

### Gap 6c — Multi-pane support

A3 attaches `:0` only. Real seat tmux sessions sometimes split panes (status pane + work pane).

#### Options

| Option | Mechanism | Pros | Cons |
|---|---|---|---|
| **6c-A. Iterate `tmux list-panes` on attach** | Attach a separate FIFO + tail-reader per pane; multiplex by `(session_id, pane_index)`. | Full coverage. | API surface grows: events need a `pane_index` field. UI complexity. |
| **6c-B. Window-level `pipe-pane`** | `tmux pipe-pane` per pane via list-panes iteration, single FIFO with pane prefix. | One FIFO simpler. | Pane prefix bytes need parsing; loses byte-for-byte fidelity. |
| **6c-C. Defer to v0.0.3 / B2 (recommended)** | Document `:0`-only as v0.0.1 scope. Most seat sessions use a single pane. | Zero new code in v0.1. | Power users with split panes don't get full coverage in v0.1. |

#### Recommendation

**6c-C (defer)**. Real seat-session telemetry shows single-pane usage today. **Complexity: zero in v0.1**. Track as a P1 enhancement when a concrete user need surfaces.

### PRD coverage for §6 as a whole

- **US-P0-06** (continuity recovery on runtime switch) — 6b makes restart non-lossy from the human's POV.
- **US-P0-09** (continuity preview) — 6b's `capture-pane` backfill is exactly the "what was on the screen when I left" view.
- **US-P1-05** (suspend/resume with delta context) — the transcript file (6b-A) is the persistence layer P1 builds on.

---

## 7. Cross-section interactions and sequencing

| Section | Depends on | Unblocks |
|---|---|---|
| §1 Data flow pipeline | §3 project_id (so events carry project) | §2 watcher (T5 → ReviewVerdictIssued event), §5 backfill (T3 → workitems pulls events too), AD-012 prompt detection |
| §2 File watcher | §3 project_id, §1 EventType extension | US-P0-01, US-P0-05, US-P0-07 in steady-state |
| §3 Multi-project schema | nothing | §1, §2, §4 LISTEN/NOTIFY, §5 backfill (all gain `project_id`) |
| §4 Realtime / events | §3 (project-scoped emits) | mobile (US-P0-12/13/15), multi-window IM |
| §5 Backfill | §3, §1 EventType, §2 watcher (steady-state path) | US-P0-01 non-empty Inbox at v0.1 boot |
| §6 A3 limits | §1 (defer-to-adapter alignment for 6a) | US-P0-06, US-P0-09 |

**Recommended sequencing for v0.1 / v0.0.x packets**:

1. **§3 schema migration 008** (cheap; unlocks everything else).
2. **§1 EventType + liveness pipeline** in `seatloom-core/src/data_engine/liveness.rs` (pairs with B1's bidirectional write path).
3. **§2 watcher** wired to the §1 event extensions and the §5 derivation function.
4. **§5 derivation** as a library function called by §2 (steady state) and one-shot on bootstrap.
5. **§4 throttle + LISTEN/NOTIFY** as the realtime hardening pass before mobile work begins.
6. **§6b `capture-pane` backfill** when v0.0.2 lands the bidirectional path (B1 already touches `attach_tmux`).
7. **§6a, §6c** stay deferred unless a concrete user need surfaces.

---

## 8. What this supplement explicitly does NOT decide

To keep scope tight and Aegis's joint review tractable:

- **No new `EventType` variant body fields are spec'd here** beyond names and one-line rationales. The actual payload schemas land in the implementation packet for §1.
- **No replacement of AD-007's reconcile contract**. AD-007 still owns the import semantics; §2 here just adds an automatic trigger.
- **No Mobile gateway design**. US-P0-12/13/15 reference the surface in §4; the mobile transport (REST? WebSocket? Tauri-server?) is a separate design proposal.
- **No revision of AD-013 v2 viewMode/contextMode**. §3 backend project_id enforcement is the SQL mirror of the existing AD-013 v2 frontend invariants — not a re-derivation.
- **No B1/B2 write-path design**. `tmux send-keys` authority, prompt approval routing, and bidirectional safety belong to B1. §6b's `capture-pane` backfill is the only A3-adjacent change in scope.

---

## 9. Companion Lyra supplement — cross-link

Lyra is filing in parallel: an extension to `prd-v0.5.md` §3 (priority boundaries), §5 (story map), §6 (module contracts). The intersections:

| This document | Lyra's product supplement |
|---|---|
| §1 SeatResponseEmitted / IdleWindowEntered | US-P0-11 prompt classification trigger; US-P1-06 live-activity overlay |
| §2 watcher / partial reconcile | "Inbox freshness" promise in the start-of-day story (US-P0-01) |
| §3 multi-project schema | US-P3-01 multi-project workstreams; AD-013 v2 backend enforcement |
| §4 realtime / multi-window | US-P0-12/13/15 mobile surface contracts |
| §5 backfill | "v0.1 ships with full coordination history visible" — Lyra's wording |
| §6 A3 limits | "What v0.1 does NOT do" subsection of the PRD §3 priority table |

Aegis reviews both together once both are filed.

---

## Appendix A — Files touched (forecast for Lyra packet sizing)

| Section | Crate / file | Estimated LOC |
|---|---|---|
| §1 | `crates/seatloom-core/src/data_engine/liveness.rs` (new), `crates/seatloom-core/src/ledger/event.rs` (extend EventType), `crates/seatloom-core/src/adapter/traits.rs` (extend) | 600–900 |
| §2 | `crates/seatloom-core/src/watcher.rs` (new), `src-tauri/src/commands/watcher_cmds.rs` (new), `seatloom-core/src/data_engine/derive.rs` (shared with §5) | 800–1200 |
| §3 | `infra/postgres/schema/008_project_isolation.sql` (new), repository changes in `seatloom-core/src/db/repositories/*`, DTO changes in `src-tauri/src/dto.rs` | 600–900 |
| §4 | `src-tauri/src/commands/session_cmds.rs` (throttle), `src-tauri/src/realtime/notify_listener.rs` (new) | 400–600 |
| §5 | `seatloom-core/src/data_engine/derive.rs` (shared with §2), bootstrap migration runner | 400–600 (incremental over §2) |
| §6b | `crates/seatloom-core/src/pty/mod.rs` (extend `attach_tmux`), `crates/seatloom-core/examples/tmux_mirror_smoke.rs` (extend) | 80–120 |
| **Total** | | **~3 000–4 200 LOC + one schema migration + one bootstrap script** |

This is roughly 4–6 implementation packets sized like A3.

---

*Supplement filed by Nimbus · 2026-05-09 · extends AD-008..013 + architecture-design.md §3/§4/§5/§6/§7/§8/§11/§12 · companion to Lyra's prd-v0.5.md supplement · joint Aegis review pending*

---

## §Review — Lyra cross-point comments (2026-05-09 late-evening, async per Aegis directive)

### Cross-point (a) — Schema numbering: 008 vs 006

**Lyra's product supplement §3** says "schema 006 ALTER" for the multi-project `project_id` migration. **Nimbus's §3** says `008_project_isolation.sql`.

**Lyra's position**: Nimbus's `008` is correct. Lyra's `006` reference was written before the plan-mode authority migration (006) and seats-budget migration (007) were scoped. The correct sequence is:

| Migration | Content |
|---|---|
| 006 | `plan_mode_authority` (prompt_kind CHECK + cli_plan subtype) — already in gap review §C3 |
| 007 | `seats` budget / accepted_input_types / output_types / attached_skills — gap review §D3 |
| 008 | `project_isolation` — `project_id` FK on workitems / sessions / handoffs / canonical_events |

**Lyra will update product supplement §3** to reference `008` and note that 006 + 007 land first. No change needed in this document.

**Aegis decision needed**: confirm 006 → 007 → 008 ordering is binding before any of these migrations are dispatched as implementation packets.

---

### Cross-point (b) — Realtime tier mapping: T-Live/T-Near/T-Batch ↔ Nimbus §4

**Lyra's product supplement §2** defines three tiers:
- **T-Live** (≤1s): Tauri event push, no DB roundtrip — `session:output` xterm bytes, own-bubble echo
- **T-Near** (≤5s): DB write + Tauri event → store update — reconcile delta, Handoff status, IM delivery
- **T-Batch** (manual): on-demand pull — initial Inbox hydration, document search

**Nimbus's §4** recommends hybrid Option C: in-app `app.emit()` with 16ms throttle + PG `LISTEN/NOTIFY` for cross-process events.

**Lyra's position**: the two models are consistent at different abstraction levels. Mapping:

| Lyra tier | Nimbus mechanism | Notes |
|---|---|---|
| T-Live | `app.emit()` with 16ms throttle (§4 throttle policy) | `session:output` is the canonical T-Live surface |
| T-Near | PG `LISTEN/NOTIFY` → backend re-emits via `app.emit()` | Watcher events, reconcile events, IM delivery status |
| T-Batch | Manual `cmd_reconcile` / on-demand query | Unchanged from AD-007 fallback |

**One clarification needed**: Nimbus's §4 throttle policy coalesces `session:output` at 16ms. Lyra's T-Live budget is ≤1s. These are compatible (16ms << 1s), but the product supplement should clarify that T-Live is the *user-perceived* budget, not the per-chunk emit interval. The 16ms throttle is the implementation mechanism that keeps T-Live within budget. No conflict — just a framing note.

**No change needed in this document.** Lyra will add a clarifying sentence in product supplement §2.

---

### Cross-point (c) — project_id migration timing vs B1 write path

**Nimbus's §7 sequencing** puts `§3 schema migration 008` first, before §1 (EventType + liveness), §2 (watcher), §4 (realtime), §5 (backfill). This is correct.

**B1 (tmux send-keys write path)** creates new `canonical_events` rows via `cmd_pty_write` → `PtySession::write` → (eventually) `cmd_append_supervisor_message` or a new `SeatResponse` event. If `canonical_events` does not have `project_id` when B1 lands, every write-path event will be unscoped — violating AD-013 v2 backend enforcement.

**Lyra's position**: `008` must land and be accepted **before** B1 delivery. This is the coupling Nimbus flagged in his B1 pre-read reply. Lyra has already noted this in the B1 dispatch packet (`NIMBUS-2026-05-09-b1-pty-write-send-keys-v1.md` §9 dispatch note: "if joint review elevates 006 multi-project schema ahead of B1 landing, PtySession construction may need project_id threading — Nimbus to hold + flag").

**Concrete sequencing implication**:

```
008 migration packet (Nimbus) → Flux verify → Lyra acceptance
    ↓
B1 write path (Nimbus) — may now set project_id on new canonical_events rows
```

If B1 is already in flight when 008 lands, Nimbus should issue a B1-v2 that adds `project_id` to the `cmd_pty_write` → event path. If 008 lands before B1 delivery, B1 can include `project_id` from the start.

**Aegis decision needed**: confirm that 008 is dispatched as a standalone packet before B1 delivery is accepted. Lyra will not accept B1 if `canonical_events` still lacks `project_id` at the time of B1 delivery.

> **RETRACTED 2026-05-11 per Aegis correction** (consolidation doc `2026-05-09-lyra-nimbus-joint-review-amendments-v1.md` §A.7): Nimbus correctly flagged that B1's bytes-only scope writes no canonical_events rows. The earlier claim that B1 required 008 precedence was factually wrong. B1 is unblocked and runs in parallel with 008.

---

*Lyra cross-point review complete · 2026-05-09 late-evening · Three cross-points addressed: (a) schema 008 confirmed correct, Lyra will update product supplement §3; (b) T-Live/T-Near/T-Batch maps cleanly to Nimbus §4 hybrid, no conflict; (c) 008 must precede B1 delivery — Aegis to confirm sequencing. Nimbus to write reciprocal review in Lyra's product supplement.*
