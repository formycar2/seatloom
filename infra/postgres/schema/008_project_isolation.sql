-- 008_project_isolation.sql
--
-- Multi-project isolation: add `project_id TEXT NOT NULL REFERENCES projects(id)`
-- to the four core operational tables that currently lack it. Mirrors the SQL-level
-- backing for AD-013 v2's "project mode does not leak across projects" frontend
-- invariant. Operationalizes Nimbus arch supplement §3 + Lyra product supplement
-- §3.3.1 (Aegis joint-review approved 2026-05-09 late-evening).
--
-- Tables touched (currently without project_id):
--   workitems, sessions, handoffs, canonical_events
--
-- Tables already carrying project_id (out of scope, untouched):
--   documents (002), review_threads/review_comments (004),
--   prompt_instances/channel_action_receipts (005),
--   project_role_bindings (001 composite PK)
--
-- Idempotency: PostgreSQL ≥ 9.6 supports ADD COLUMN IF NOT EXISTS on ALTER TABLE.
-- We run PG 16. UPDATE backfill is idempotent against
-- (project_id IS NULL OR project_id = 'seatloom'). SET NOT NULL on an already-
-- NOT NULL column is a no-op. Whole transaction is safely re-runnable.
--
-- Backfill heuristic (per arch §3, finalized in packet §2.1 with corrected
-- event_object_refs columns ref_type / ref_id):
--   workitems.project_id ← join through owner_seat_id → project_role_bindings;
--                          default 'seatloom' when no binding (drafts).
--   sessions.project_id  ← join through seat_id → project_role_bindings;
--                          default 'seatloom'.
--   handoffs.project_id  ← join through workitem_id → workitems → owner_seat_id
--                          → project_role_bindings; default 'seatloom'.
--   canonical_events.project_id ← inherited from event_object_refs lineage if
--                                 ref_type IN ('workitem','session','handoff');
--                                 default 'seatloom' otherwise.
--                                 Today only ref_type='seat' rows are emitted
--                                 (see crates/seatloom-core/src/db/repositories.rs:350),
--                                 so all existing canonical_events fall through
--                                 to 'seatloom' default — the lineage joins are
--                                 forward-compatible for when future writers
--                                 emit workitem/session/handoff refs.
--
-- After migration: AD-013 v2 backend project-mode list queries SHALL include
-- `WHERE project_id = $1`; Global-mode SHALL group by `project_id`. The four
-- new indexes back this access pattern.

BEGIN;

-- =============================================================================
-- 1. Add project_id column with DEFAULT 'seatloom' so existing rows fill in.
--    DEFAULT is dropped in step 3 after backfill.
-- =============================================================================
ALTER TABLE workitems        ADD COLUMN IF NOT EXISTS project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom';
ALTER TABLE sessions         ADD COLUMN IF NOT EXISTS project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom';
ALTER TABLE handoffs         ADD COLUMN IF NOT EXISTS project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom';
ALTER TABLE canonical_events ADD COLUMN IF NOT EXISTS project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom';

-- =============================================================================
-- 2. Backfill from steady-state heuristic (idempotent — only touches default rows).
-- =============================================================================

-- workitems.project_id ← owner_seat_id → project_role_bindings.project_id; default 'seatloom'.
UPDATE workitems SET project_id = COALESCE(
    (SELECT prb.project_id FROM project_role_bindings prb
      WHERE prb.seat_id = workitems.owner_seat_id LIMIT 1),
    'seatloom'
) WHERE project_id IS NULL OR project_id = 'seatloom';

-- sessions.project_id ← seat_id → project_role_bindings.project_id; default 'seatloom'.
UPDATE sessions SET project_id = COALESCE(
    (SELECT prb.project_id FROM project_role_bindings prb
      WHERE prb.seat_id = sessions.seat_id LIMIT 1),
    'seatloom'
) WHERE project_id IS NULL OR project_id = 'seatloom';

-- handoffs.project_id ← workitem_id → workitems → owner_seat_id → bindings; default 'seatloom'.
UPDATE handoffs SET project_id = COALESCE(
    (SELECT prb.project_id FROM project_role_bindings prb
      WHERE prb.seat_id IN (
        SELECT owner_seat_id FROM workitems WHERE id = handoffs.workitem_id
      ) LIMIT 1),
    'seatloom'
) WHERE project_id IS NULL OR project_id = 'seatloom';

-- canonical_events.project_id ← event_object_refs lineage (forward-compat;
-- today every row falls through to 'seatloom' since only ref_type='seat' is emitted).
-- Schema 001 columns: (event_id, ref_type, ref_id) — NOT (event_id, object_type, object_id).
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

-- =============================================================================
-- 3. Drop DEFAULT and tighten NOT NULL on all four (idempotent on re-run).
-- =============================================================================
ALTER TABLE workitems        ALTER COLUMN project_id DROP DEFAULT;
ALTER TABLE workitems        ALTER COLUMN project_id SET NOT NULL;
ALTER TABLE sessions         ALTER COLUMN project_id DROP DEFAULT;
ALTER TABLE sessions         ALTER COLUMN project_id SET NOT NULL;
ALTER TABLE handoffs         ALTER COLUMN project_id DROP DEFAULT;
ALTER TABLE handoffs         ALTER COLUMN project_id SET NOT NULL;
ALTER TABLE canonical_events ALTER COLUMN project_id DROP DEFAULT;
ALTER TABLE canonical_events ALTER COLUMN project_id SET NOT NULL;

-- =============================================================================
-- 4. Indexes for AD-013 v2 project-mode list queries.
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_workitems_project_status          ON workitems(project_id, status);
CREATE INDEX IF NOT EXISTS idx_sessions_project_status           ON sessions(project_id, status);
CREATE INDEX IF NOT EXISTS idx_handoffs_project                  ON handoffs(project_id);
CREATE INDEX IF NOT EXISTS idx_canonical_events_project_occurred ON canonical_events(project_id, occurred_at DESC);

COMMIT;
