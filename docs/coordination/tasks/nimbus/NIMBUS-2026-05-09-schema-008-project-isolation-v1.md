# Task: Schema 008 — Multi-Project Isolation (project_id FK on operational tables)

[Lyra -> Nimbus] 008 dispatched in parallel with B1 (not blocking). Both target v0.0.2. B1 bytes-only writes no `canonical_events`, so 008's `project_id` column is not a B1 dependency — but B2 onward will need it. Land 008 in v0.0.2 alongside or after B1 at your sequencing call.

| Field | Value |
|---|---|
| template | T4 |
| subtype | implementation |
| id | NIMBUS-2026-05-09-schema-008-project-isolation-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-05-09 (late evening) |
| to | nimbus |
| priority | P0 (parallel with B1; both target v0.0.2 milestone) |
| milestone | v0.0.2 |
| dispatched_at | 2026-05-09 late-evening (after Aegis joint-review ruling) |
| depends_on | A4-β PASS @ `ab672e5`; Nimbus arch supplement §3 (commit `dd8758c`); Aegis joint-review ruling (006→007→008 binding) |
| relationship to B1 | **Parallel, not blocking.** Aegis correction 2026-05-09 late-evening: B1 bytes-only scope writes zero `canonical_events` rows, so 008's `project_id` column is not required for B1 acceptance. 008 still lands as soon as practical because every *subsequent* write-path packet (B2 onward, Inbox flows, supervisor-message commands) will create `canonical_events` rows that need `project_id`. |
| delivery path | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-schema-008-project-isolation-delivery-v1.md` |
| tags | nimbus, schema, 008, project-isolation, multi-project, AD-013-v2-backend-enforcement, parallel-with-B1 |

---

## 1. Context

**Aegis joint-review ruling (2026-05-09 late-evening, corrected)**:
- Migration numbering binding: **006 plan_mode_authority → 007 seats_budget → 008 project_isolation**.
- 008 dispatches as a standalone packet **in parallel with B1**, not blocking it. (Lyra's earlier cross-point (c) claim that B1 required 008 precedence was factually wrong: Nimbus correctly flagged B1 bytes-only scope writes no `canonical_events` rows, so 008's `project_id` column doesn't affect B1 acceptance.)

**Why 008 still lands in v0.0.2**: every *subsequent* write-path packet — B2 `cmd_append_supervisor_message` paths, Inbox flows, Plan-mode approval injection events, watcher-derived `WorkItemCreated` events from §2 — will create `canonical_events` rows that need `project_id` to satisfy AD-013 v2 backend enforcement and US-P0-03 multi-project guarantee. Doing 008 before that surface lights up avoids a costly retrofit.

**Why 008 must follow 006 + 007**: Aegis numbering is binding. 006 (`plan_mode_authority` — gap review §C3) and 007 (`seats_budget` — gap review §D3) own 006/007 slots respectively.

**Source of truth**: Nimbus arch supplement §3 (`docs/coordination/reviews/2026-05-09-nimbus-seatloom-full-arch-design-v1.md` lines 151-216). The packet below operationalizes that section's recommendation.

## 2. Scope (3 areas)

### 2.1 Schema migration `infra/postgres/schema/008_project_isolation.sql`

Add `project_id TEXT NOT NULL REFERENCES projects(id)` + indexes to the four core operational tables that currently lack it:

- `workitems`
- `sessions`
- `handoffs`
- `canonical_events`

The existing 6 tables that already carry `project_id` are out of scope (`documents` schema 002, `review_threads` schema 004, `prompt_instances` + `channel_action_receipts` schema 005, `project_role_bindings` schema 001).

**Concrete migration body** (per Nimbus arch supplement §3 sketch, finalized here):

```sql
-- 008_project_isolation.sql

BEGIN;

-- 1. Add column with DEFAULT to allow backfill without locking writes long.
ALTER TABLE workitems        ADD COLUMN project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom';
ALTER TABLE sessions         ADD COLUMN project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom';
ALTER TABLE handoffs         ADD COLUMN project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom';
ALTER TABLE canonical_events ADD COLUMN project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom';

-- 2. Backfill existing rows using the steady-state heuristic from arch §3:
--    workitem.project_id ← join via owner_seat_id → project_role_bindings; default 'seatloom'.
UPDATE workitems SET project_id = COALESCE(
  (SELECT prb.project_id FROM project_role_bindings prb
    WHERE prb.seat_id = workitems.owner_seat_id LIMIT 1),
  'seatloom'
) WHERE project_id IS NULL OR project_id = 'seatloom';

UPDATE sessions SET project_id = COALESCE(
  (SELECT prb.project_id FROM project_role_bindings prb
    WHERE prb.seat_id = sessions.seat_id LIMIT 1),
  'seatloom'
) WHERE project_id IS NULL OR project_id = 'seatloom';

UPDATE handoffs SET project_id = COALESCE(
  (SELECT prb.project_id FROM project_role_bindings prb
    WHERE prb.seat_id IN (
      SELECT owner_seat_id FROM workitems WHERE id = handoffs.workitem_id
    ) LIMIT 1),
  'seatloom'
) WHERE project_id IS NULL OR project_id = 'seatloom';

-- canonical_events: backfill via event_object_refs (the existing row that links events to objects).
-- Conservative: any event whose object_refs match a workitem/session/handoff with non-default project_id
-- inherits that project_id; otherwise stays 'seatloom'.
UPDATE canonical_events ce SET project_id = COALESCE(
  (SELECT w.project_id FROM event_object_refs eor
    JOIN workitems w ON w.id = eor.object_id
    WHERE eor.event_id = ce.id AND eor.object_type = 'workitem'
    LIMIT 1),
  (SELECT s.project_id FROM event_object_refs eor
    JOIN sessions s ON s.id = eor.object_id
    WHERE eor.event_id = ce.id AND eor.object_type = 'session'
    LIMIT 1),
  (SELECT h.project_id FROM event_object_refs eor
    JOIN handoffs h ON h.id = eor.object_id
    WHERE eor.event_id = ce.id AND eor.object_type = 'handoff'
    LIMIT 1),
  'seatloom'
) WHERE project_id IS NULL OR project_id = 'seatloom';

-- 3. Drop DEFAULT and tighten NOT NULL on all four.
ALTER TABLE workitems        ALTER COLUMN project_id DROP DEFAULT;
ALTER TABLE workitems        ALTER COLUMN project_id SET NOT NULL;
ALTER TABLE sessions         ALTER COLUMN project_id DROP DEFAULT;
ALTER TABLE sessions         ALTER COLUMN project_id SET NOT NULL;
ALTER TABLE handoffs         ALTER COLUMN project_id DROP DEFAULT;
ALTER TABLE handoffs         ALTER COLUMN project_id SET NOT NULL;
ALTER TABLE canonical_events ALTER COLUMN project_id DROP DEFAULT;
ALTER TABLE canonical_events ALTER COLUMN project_id SET NOT NULL;

-- 4. Indexes for AD-013 v2 project-mode list queries.
CREATE INDEX IF NOT EXISTS idx_workitems_project_status         ON workitems(project_id, status);
CREATE INDEX IF NOT EXISTS idx_sessions_project_status          ON sessions(project_id, status);
CREATE INDEX IF NOT EXISTS idx_handoffs_project                 ON handoffs(project_id);
CREATE INDEX IF NOT EXISTS idx_canonical_events_project_occurred ON canonical_events(project_id, occurred_at DESC);

COMMIT;
```

**Idempotency**: Use `ADD COLUMN IF NOT EXISTS` style? PostgreSQL doesn't support `ADD COLUMN IF NOT EXISTS` on `ALTER TABLE` directly, so wrap in a `DO` block if you want re-runnability — or rely on the runner's "skip if migration version recorded" semantics (your current convention). State your choice in the delivery doc; either works.

### 2.2 Repository changes

Each of the four core repos gains a `project_id` parameter on insert and a `for_project` filter on list:

| Repo | Location | Changes |
|---|---|---|
| `workitem_repo` | `crates/seatloom-core/src/db/repositories/workitem_repo.rs` (or wherever `cmd_create_workitem` lands) | `insert(...)` takes `project_id: &str`; `list_for_project(project_id: &str, ...) -> Vec<...>`; existing `list(...)` either deleted or kept as `list_all()` for Global mode |
| `session_repo` | similar | same pattern |
| `handoff_repo` | similar | same pattern |
| `event_repo` | for `canonical_events` writes | `append(..., project_id: &str)`; `list_for_project(project_id: &str, ...)`; existing `list_recent(...)` either deleted or kept as `list_all_recent()` for Global mode |

**DTO changes** in `src-tauri/src/dto.rs`:

- `WorkItemDto`, `SessionDto`, `HandoffDto`, `CanonicalEventDto` each gain `project_id: String` in their serialize shape (camelCase wire: `projectId`).

**Tauri command surface** in `src-tauri/src/commands/`:

- `cmd_list_workitems` etc. take `project_id: String` parameter and filter accordingly. Default to `state.default_project_id` for compatibility, but emit a warning log if the param is missing — frontend should be passing it explicitly post-008.
- `cmd_create_workitem`, `cmd_create_handoff`, `cmd_append_supervisor_message`, etc. all take `project_id` from `AppState.active_project_id` (which is already set per AD-013 v2 currentContextMode plumbing). For commands that don't yet receive an active project ID via state, plumb it in.

### 2.3 Backfill verification

The backfill heuristic is conservative (defaults to `'seatloom'` for ambiguous joins). After the migration runs:

```sql
-- Verification queries — must return zero rows:
SELECT 'workitems'        AS table_name, COUNT(*) FROM workitems        WHERE project_id IS NULL;
SELECT 'sessions'         AS table_name, COUNT(*) FROM sessions         WHERE project_id IS NULL;
SELECT 'handoffs'         AS table_name, COUNT(*) FROM handoffs         WHERE project_id IS NULL;
SELECT 'canonical_events' AS table_name, COUNT(*) FROM canonical_events WHERE project_id IS NULL;

-- Sanity counts — must show all rows scoped to 'seatloom' (the only project today):
SELECT project_id, COUNT(*) FROM workitems         GROUP BY project_id;
SELECT project_id, COUNT(*) FROM sessions          GROUP BY project_id;
SELECT project_id, COUNT(*) FROM handoffs          GROUP BY project_id;
SELECT project_id, COUNT(*) FROM canonical_events  GROUP BY project_id;
```

Capture these in delivery doc verbatim.

## 3. Out of scope (do NOT do)

- **Do not touch B1 files**: `crates/seatloom-core/src/pty/mod.rs`, `src-tauri/src/commands/session_cmds.rs`'s `cmd_pty_write` / `cmd_pty_write_bytes`, `crates/seatloom-core/examples/tmux_mirror_smoke.rs`, `ui/src/app-v2/panel/SessionTerminal.tsx`, `ui/src/app-v2/panel/SessionsWorkspace.tsx`. These are B1's territory — wait until 008 lands and Lyra issues "B1 GO v2".
- **Do not modify migration 006 (`plan_mode_authority`) or 007 (`seats_budget`)**: those are owned by the Stage Gate C / D packets and dispatched separately.
- **Do not introduce a new project**: backfill defaults to `'seatloom'` because that's the only project today. Multi-project routing logic at the API surface (cmd_create_workitem etc.) is the policy enforcement point; the FK only ensures a real project, not the *right* project.
- **Do not change AD-013 v2 frontend invariants**: backend project_id enforcement is the SQL mirror, not a re-derivation of the React store rules.

## 4. R-rule / invariant compliance

Copy verbatim into delivery doc §Compliance:

| Rule | Required behavior | Verification approach |
|---|---|---|
| R1 attach-only (still in force) | No spawn / no PTY changes | Inspect: `git show <commit> -- crates/seatloom-core/src/pty/mod.rs` must be empty |
| R3 failure-isolation (still in force) | Migration runs in transaction; tmux session ownership unaffected | Inspect: `BEGIN; ... COMMIT;` wrapper; no tmux subprocess calls |
| AD-013 v2 backend enforcement | After 008, project-mode list queries SHALL include `WHERE project_id = $1`; Global-mode SHALL group by `project_id` | Inspect repo signatures + at least one example query in delivery doc |
| US-P0-03 multi-project | `project_id` is the routing primitive across workitems/sessions/handoffs/canonical_events | Inspect schema diff + one DTO example |
| Single-project today | Backfill heuristic defaults to `'seatloom'` for ambiguous joins | Sanity-count queries above must show all rows in `'seatloom'` |

## 5. Verification plan

### 5.1 Layer A (Nimbus runs, Flux verifies)

```
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test -p seatloom-core --lib   # all existing tests must still pass
cargo clippy -p seatloom-core -- -D warnings

# Migration runner against the verifier postgres
podman ps                            # confirm seatloom-postgres is up
podman exec seatloom-postgres psql -U seatloom -d seatloom -f /docker-entrypoint-initdb.d/008_project_isolation.sql
# OR via the existing `bash scripts/verify-postgres-baseline.sh` if it picks up 008 automatically

# Verification queries (capture verbatim per §2.3)
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "<each query from §2.3>"

cd ui && pnpm exec tsc --noEmit       # zero errors after DTO changes
cd ui && pnpm build                   # exit 0
```

Paste all output verbatim into delivery doc.

### 5.2 Layer B (Mr. Zhang runtime — short)

After Lyra acceptance and a `pnpm tauri dev` restart:

1. Open Supervisor IM → confirm Inbox renders normally (no broken queries).
2. Open SessionsWorkspace → attach to a tmux session → confirm `cmd_attach_tmux_session` still works with project_id propagation.
3. (If Aegis schedules cmd_create_workitem flow) — create a WorkItem via Supervisor IM → DB row should land with `project_id='seatloom'`.

Layer B for 008 is intentionally minimal because the migration is structural; the heavy runtime test is B1's job.

## 6. Delivery doc requirements

Write to `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-schema-008-project-isolation-delivery-v1.md`. Must contain:

1. T3 frontmatter (template, subtype=implementation_delivery, status=delivered, commit=<sha>, packet_ref=this packet).
2. `git show --stat <sha>` verbatim. Expected scope: `infra/postgres/schema/008_project_isolation.sql` (new); repo source files (workitem_repo / session_repo / handoff_repo / event_repo); DTOs in `src-tauri/src/dto.rs`; Tauri command surface in `src-tauri/src/commands/*`. Estimated 600-900 LOC per arch supplement §3.
3. All §5.1 Layer A outputs verbatim (cargo check/test/clippy/build, migration apply, verification queries, pnpm tsc/build).
4. §4 compliance checklist with line-pinned evidence.
5. **Backfill verification table** — paste the four NULL-count queries (must be zero) and the four GROUP BY queries (must show all rows in `'seatloom'`).
6. **Idempotency note** — describe how re-running the migration behaves (transaction abort? skip-if-version-recorded? `DO` block?).
7. **DTO wire-shape table** — show the camelCase serialization for the new `projectId` field on each of the four DTOs (so Mira / Lyra / Aegis can verify TS type alignment when B1 v2 lands).

## 7. Acceptance criteria (Lyra)

1. Schema migration runs cleanly against current podman seatloom-postgres baseline.
2. All four backfill verification NULL-count queries return zero rows.
3. All four GROUP BY queries return only `'seatloom'` (since that's the only project today).
4. `cargo check` + `cargo test` + `cargo clippy -D warnings` all green.
5. `pnpm exec tsc --noEmit` zero errors after DTO changes.
6. `pnpm build` exit 0.
7. R1, R3, AD-013 v2 invariants line-pinned via grep evidence in delivery doc.
8. No B1-scope files touched.
9. Idempotency strategy stated and tested (re-run migration → no-op or clean abort).
10. Flux Layer A commit-pinned verify PASSes.

On Flux Layer A PASS → Lyra acceptance → Lyra issues "B1 GO v2" to Nimbus with `project_id` parameter wiring added to B1 scope.

## 8. Scope boundaries (do not expand)

- Do **not** add a "create new project" UI affordance — the `projects` table already has the seatloom row; future projects are a separate packet.
- Do **not** modify `documents`, `review_threads`, `prompt_instances`, `channel_action_receipts`, `project_role_bindings` — they already carry `project_id`.
- Do **not** introduce schema-namespace per project (Option C from arch §3) — Aegis ruled in favor of Option A (FK column).
- Do **not** wire historical back-fill (T3/T5 → workitems derivation) — that's arch supplement §5, separate packet.
- Do **not** introduce file watcher (arch §2) or realtime LISTEN/NOTIFY (arch §4) — those are separate packets sequenced after 008 per arch §7.

## 9. Timing

**Dispatched 2026-05-09 late-evening** per Aegis joint-review ruling. Target: Nimbus delivery within next work session (08-09 hours of focused work — 600-900 LOC, no design ambiguity, schema sketch already in arch §3). On delivery → Flux Layer A verify → Lyra acceptance → "B1 GO v2".

If anything in the migration sketch turns out wrong against current schema (e.g., `event_object_refs` column names differ from arch §3 assumption), post a comment on this packet rather than guessing.

---

## 10. Nimbus pre-implementation comments (2026-05-11)

Per §9 directive, posting before any code touches the schema.

### 10.1 §2.1 backfill SQL — `event_object_refs` column names

Actual on-disk schema (`infra/postgres/schema/001_seatloom_core.sql:138-143`):

```sql
CREATE TABLE IF NOT EXISTS event_object_refs (
    event_id  TEXT NOT NULL REFERENCES canonical_events(id),
    ref_type  TEXT NOT NULL,
    ref_id    TEXT NOT NULL,
    PRIMARY KEY (event_id, ref_type, ref_id)
);
```

Packet §2.1 backfill UPDATE references `eor.object_id` and `eor.object_type`. Real columns are `eor.ref_id` and `eor.ref_type`. The `canonical_events` backfill block needs:

```sql
UPDATE canonical_events ce SET project_id = COALESCE(
  (SELECT w.project_id FROM event_object_refs eor
    JOIN workitems w ON w.id = eor.ref_id
    WHERE eor.event_id = ce.id AND eor.ref_type = 'workitem'
    LIMIT 1),
  (SELECT s.project_id FROM event_object_refs eor
    JOIN sessions s ON s.id = eor.ref_id
    WHERE eor.event_id = ce.id AND eor.ref_type = 'session'
    LIMIT 1),
  (SELECT h.project_id FROM event_object_refs eor
    JOIN handoffs h ON h.id = eor.ref_id
    WHERE eor.event_id = ce.id AND eor.ref_type = 'handoff'
    LIMIT 1),
  'seatloom'
) WHERE project_id IS NULL OR project_id = 'seatloom';
```

I'll use this corrected form in the migration. Flagging here (not silently fixing the packet) per your "don't guess" rule.

### 10.2 §2.1 backfill — current data fact

Today the only `event_object_refs` rows the codebase emits are `ref_type='seat'` (see `crates/seatloom-core/src/db/repositories.rs:350` — the `cmd_append_supervisor_message` path). Zero rows have `ref_type IN ('workitem', 'session', 'handoff')`. So the corrected `canonical_events` backfill above will fall through every COALESCE branch and default everything to `'seatloom'`.

That's the right outcome for today (single project), but I want to record it explicitly so a future reader doesn't assume the backfill is doing real lineage work. I'll add a one-line comment in the migration SQL noting "lineage joins are forward-compatible; today they all fall through to 'seatloom' default."

### 10.3 §2.1 idempotency — PG does support `ADD COLUMN IF NOT EXISTS`

Packet §2.1 says PostgreSQL doesn't support `ADD COLUMN IF NOT EXISTS` on `ALTER TABLE`. Actually PostgreSQL has supported it since 9.6 (2016) — the SeatLoom container runs PG 16. Confirmed via PG docs: `ALTER TABLE … ADD COLUMN IF NOT EXISTS …` is valid syntax.

I'll use `ADD COLUMN IF NOT EXISTS` directly (no `DO` block needed) for cleanest re-runnability. The idempotency note in the delivery doc will state: "Re-running 008 against a tree that already has `project_id` is a no-op for the column adds; the UPDATE statements are idempotent against `WHERE project_id IS NULL OR project_id = 'seatloom'`; the `ALTER ... DROP DEFAULT / SET NOT NULL` are idempotent (already-dropped DEFAULT and already-NOT NULL state are no-ops). The whole transaction can be re-run safely."

### 10.4 No other deviations from the packet

§2.2 repo signature changes, §2.3 verification queries, §4 R-rule compliance, §5 Layer A plan, §6 delivery doc shape, §7 acceptance criteria, §8 scope boundaries — all read clean against the current code/schema. I'll implement to the packet.

### 10.5 Status of B1

For Lyra's awareness in case the message ordering crossed: B1 was delivered at commit `0c0f425` + delivery doc `b1d4b4b` before this comment was written. The 008 packet correctly notes parallelism, so 008 implementation proceeds independently.

---

*Packet comments by Nimbus · 2026-05-11 · Three corrections to §2.1 (column names, current-data fact, idempotency syntax) flagged before implementation. Will use corrected SQL in the migration. No scope changes proposed.*

---

*Dispatched by Lyra · 2026-05-09 late-evening · Aegis joint-review ruling: 008 standalone before B1 · gates B1 dispatch · operationalizes Nimbus arch supplement §3 verbatim*
