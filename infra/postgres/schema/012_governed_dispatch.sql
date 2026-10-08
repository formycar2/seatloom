-- 012_governed_dispatch.sql
-- =============================================================================
-- Governed dispatch (G19, P0 step 6) — the per-seat inbox (`dispatches`) and the
-- dependency graph AS DATA (`dispatch_dependencies`), plus the three dispatch
-- lifecycle event-types and the `dispatch` ref-type that make a dispatches row a
-- replayable PROJECTION of canonical_events rather than free-floating state.
--
-- Authority: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md
-- (the single value-domain contract, carried verbatim into both the Onyx schema
-- packet and the Nimbus scheduler packet). This migration implements that
-- contract's schema half; it does not re-derive it. Dispatched by
-- docs/coordination/tasks/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-v1.md.
--
-- Slot 012: 001-005, 008, 009, 010, 011 exist (006/007 are historical gaps, not
-- errors — do NOT renumber). POST-SEED: the seed writes none of the new columns,
-- and the two vocab CHECKs this widens exist only after 010 (also POST-SEED) has
-- created them, so 012 cannot precede its own prerequisite. See scripts/apply-db.sh.
--
-- Additive and idempotent, in 010's house style: every CHECK/vocab change is
-- wrapped in a catalog-guarded DO block, so a second apply performs NO catalog
-- write. The two vocab widenings DROP-and-re-ADD the existing CHECK (010 only
-- had to CREATE fresh ones); the guard compares the live constraint definition
-- via pg_get_constraintdef so the DROP/ADD only fires when the new value is
-- actually missing — a re-apply is a true no-op.
-- =============================================================================

BEGIN;

-- =============================================================================
-- Vocabulary widening #1 — canonical_events.event_type: +3 dispatch lifecycle
-- =============================================================================
-- Contract: SeatDispatchIssued / SeatDispatchDelivered / SeatDispatchCompleted.
-- Rationale (verbatim from the contract): the dispatches row is a PROJECTION,
-- rebuildable from the ledger ONLY if all three transitions are events. Omitting
-- any one makes the projection unreplayable — the exact defect step 2 (A1)
-- removed. Escalation adds NO value: a cycle/stall is emitted as the existing
-- 'SupervisorMessage' (actor_ref = the scheduler). Minimal widening, reused value.
--
-- 010 created this CHECK with 12 values; 012 must widen it to 15. Because the
-- constraint already exists, the idempotency guard cannot be 010's bare
-- "IF NOT EXISTS (conname ...)" (that would skip the widening forever). Instead
-- it checks whether the CURRENT definition already contains the new values; only
-- if not does it DROP and re-ADD the full 15-value list in one transaction.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%SeatDispatchIssued%'
  ) THEN
    ALTER TABLE canonical_events
      DROP CONSTRAINT IF EXISTS canonical_events_event_type_vocab;
    ALTER TABLE canonical_events
      ADD CONSTRAINT canonical_events_event_type_vocab
      CHECK (event_type IN (
        -- (010) present in committed data
        'SessionStarted',
        'WorkItemCreated',
        'WorkItemStatusChanged',
        'ArtifactCreated',
        'HandoffSent',
        'HandoffAccepted',
        'HandoffCompleted',
        'ReviewVerdictIssued',
        'SeatDelegationIssued',
        'SeatDelegationClosed',
        -- (010) inserted by live code, in no row yet
        'SupervisorMessage',
        'SeatResponse',
        -- (012, G19) dispatch lifecycle — the three transitions that make the
        -- dispatches projection replayable. Lyra sign-off: the value-domain
        -- contract 2026-10-08.
        'SeatDispatchIssued',      -- a dispatch entered the inbox
        'SeatDispatchDelivered',   -- a seat pulled it (ready -> in_flight)
        'SeatDispatchCompleted'    -- the seat marked it done (in_flight -> done)
      ));
  END IF;
END $$;

-- =============================================================================
-- Vocabulary widening #2 — event_object_refs.ref_type: +1 ('dispatch')
-- =============================================================================
-- Contract rationale (verbatim): dispatches is a table with its own object,
-- exactly the reasoning 010 used to admit 'delegation'. Without it an event
-- cannot name the dispatch it is about. Same catalog-guarded DROP/ADD pattern.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'event_object_refs_ref_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%dispatch%'
  ) THEN
    ALTER TABLE event_object_refs
      DROP CONSTRAINT IF EXISTS event_object_refs_ref_type_vocab;
    ALTER TABLE event_object_refs
      ADD CONSTRAINT event_object_refs_ref_type_vocab
      CHECK (ref_type IN (
        'workitem', 'session', 'handoff', 'artifact',
        'document', 'seat', 'topic', 'project',
        'delegation',  -- 010: present in committed data
        'dispatch'     -- 012 (G19): dispatches is its own object; an event must
                       -- be able to name the dispatch it is about.
      ));
  END IF;
END $$;

-- =============================================================================
-- dispatches — the per-seat inbox. One row = one thing a seat must do.
-- =============================================================================
-- Columns are EXACTLY the contract's, in its order. Design decisions recorded
-- inline and in the delivery doc:
--
--  * target_seat_id / issued_by typing — TEXT, matching seats.id and the
--    actor_ref shape already used across the schema (001: seats.id TEXT PK;
--    canonical_events.actor_ref is TEXT). target_seat_id carries a FK to
--    seats(id): it is additive and safe because the seed never inserts a
--    dispatches row, so no seeded row can dangle the FK — every live seat the
--    scheduler targets is already a seats row (001 + seed 001). issued_by is
--    deliberately NOT FK'd: the contract types it as an actor_ref (e.g.
--    'human:user', 'seat:seat-lyra-001'), which is a superset of seat ids and
--    is never a bare seats.id — a FK there would reject legitimate human/
--    supervisor issuers.
--  * project_id — FK to projects(id), per the contract (REFERENCES projects(id)).
--    Safe/additive for the same reason: no seeded dispatches row exists to
--    dangle it.
--  * payload — JSONB NOT NULL, per the contract. This diverges from 010's
--    nullable canonical_events.payload ON PURPOSE and the contract makes the
--    call: a dispatch with no payload carries no packet path or instructions and
--    is meaningless, so "not recorded" is not a legitimate state here (unlike an
--    event payload, where NULL vs '{}' is a real distinction 010 preserved).
--    The writer always supplies at least {"packet": "..."}. JSONB over JSON for
--    the usual reasons (binary storage, indexable, dedup of keys); no ordering
--    or whitespace fidelity is needed.
--  * status — TEXT, lowercase snake_case to match every existing status column
--    (001 seats/sessions/workitems/handoffs all use lowercase), domain CHECK below.
--  * priority — INT, higher pulls first; ties broken by issued_at (scheduler's job).
CREATE TABLE IF NOT EXISTS dispatches (
    id              TEXT        PRIMARY KEY,
    target_seat_id  TEXT        NOT NULL REFERENCES seats(id),
    issued_by       TEXT        NOT NULL,                       -- actor_ref of the issuer
    project_id      TEXT        NOT NULL REFERENCES projects(id),
    title           TEXT        NOT NULL,                       -- one-line routing summary
    payload         JSONB       NOT NULL,                       -- packet path + instructions
    status          TEXT        NOT NULL,                       -- ready | blocked | in_flight | done
    priority        INTEGER     NOT NULL,                       -- higher pulls first
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ
);

-- Status domain CHECK (contract: 4 values). Guarded so re-apply is a no-op.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'dispatches_status_vocab') THEN
    ALTER TABLE dispatches
      ADD CONSTRAINT dispatches_status_vocab
      CHECK (status IN ('ready', 'blocked', 'in_flight', 'done'));
  END IF;
END $$;

-- =============================================================================
-- THE LOAD-BEARING CONSTRAINT — one in_flight dispatch per seat, IN THE DATABASE.
-- =============================================================================
-- Contract rule 1: a partial unique index on (target_seat_id) WHERE
-- status='in_flight'. This makes "one in-flight per seat" a database FACT, not a
-- scheduler promise — a second concurrent pull cannot create a second in_flight
-- row for the same seat even if the scheduler has a bug or a race. 'done',
-- 'ready', and 'blocked' rows are OUTSIDE the index predicate, so a seat may
-- have any number of them; only 'in_flight' is unique-scoped.
CREATE UNIQUE INDEX IF NOT EXISTS uq_dispatches_one_in_flight_per_seat
    ON dispatches (target_seat_id)
    WHERE status = 'in_flight';

-- The scheduler's pull query (contract rule 4: highest-priority READY item for a
-- seat, skipping blocked, ties by issued_at). This index serves that ORDER BY.
CREATE INDEX IF NOT EXISTS idx_dispatches_pull
    ON dispatches (target_seat_id, status, priority DESC, issued_at);

-- =============================================================================
-- dispatch_dependencies — the dependency graph AS DATA, machine-checkable.
-- =============================================================================
-- Contract: a dispatch is READY iff every dependency row's referent EXISTS.
--
-- DESIGN DECISION — dep_value has DELIBERATELY NO foreign key.
-- dep_kind is polymorphic across three object types (event | artifact | dispatch),
-- pointing at canonical_events.id / artifacts.id / dispatches.id respectively.
-- A per-kind FK is exactly the wrong primitive here — the same reasoning 010
-- used for event_object_refs.ref_id, which is polymorphic across eight object
-- types and carries no FK. The consequence, stated plainly: a dangling
-- dependency (a dep_value whose referent does not yet, or no longer, exists) is
-- possible at the schema level. That is BY DESIGN: readiness is evaluated by an
-- EXISTENCE QUERY ("does a row with this id exist in the kind's table?"), not by
-- FK. A missing referent must read as 'blocked' — NOT an error, NOT a stall
-- (contract status domain) — which an existence query delivers and an FK would
-- turn into an insert-time rejection. Verified in the delivery doc §3.3.
--
-- Primary key (dispatch_id, dep_kind, dep_value): the same referent cannot be
-- listed twice for one dispatch. This PK also covers the by-dispatch lookup the
-- readiness check needs (leading column dispatch_id), so no separate index is
-- added for it.
CREATE TABLE IF NOT EXISTS dispatch_dependencies (
    dispatch_id  TEXT NOT NULL REFERENCES dispatches(id),
    dep_kind     TEXT NOT NULL,   -- event | artifact | dispatch
    dep_value    TEXT NOT NULL,   -- canonical_events.id | artifacts.id | dispatches.id
    PRIMARY KEY (dispatch_id, dep_kind, dep_value)
);

-- dep_kind domain CHECK (contract). Guarded so re-apply is a no-op.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'dispatch_dependencies_dep_kind_vocab') THEN
    ALTER TABLE dispatch_dependencies
      ADD CONSTRAINT dispatch_dependencies_dep_kind_vocab
      CHECK (dep_kind IN ('event', 'artifact', 'dispatch'));
  END IF;
END $$;

-- =============================================================================
-- COMMENTs — the reasoning is part of the artifact (010 house style).
-- =============================================================================
COMMENT ON TABLE dispatches IS
  'Per-seat inbox for governed dispatch (P0 G19, ONYX-2026-10-08-p0-step6-dispatch-schema-v1). One row = one thing a seat must do. This row is a PROJECTION of canonical_events (SeatDispatchIssued/Delivered/Completed); the ledger is authoritative. Delivery is MCP PULL ONLY (get_next_dispatch when a seat is idle) — never mid-turn tmux injection. Value-domain contract: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md.';
COMMENT ON COLUMN dispatches.target_seat_id IS
  'The seat that must act. FK to seats(id). At most one in_flight row per seat is a DATABASE fact via uq_dispatches_one_in_flight_per_seat, not only a scheduler rule — one-in-flight is what kills context pollution.';
COMMENT ON COLUMN dispatches.issued_by IS
  'actor_ref of the issuer (e.g. ''human:user'', ''seat:seat-lyra-001''), same shape as canonical_events.actor_ref. Deliberately NOT FK''d to seats(id): actor_ref is a superset of seat ids (humans/supervisor issue dispatches too).';
COMMENT ON COLUMN dispatches.payload IS
  'JSONB, NOT NULL by contract: a dispatch with no payload carries no packet path or instructions and is meaningless. Unlike canonical_events.payload (010, nullable, where NULL=not-recorded is a real distinction), a dispatch always records its packet. Typically {"packet": "<path>", ...}.';
COMMENT ON COLUMN dispatches.status IS
  'ready | blocked | in_flight | done (dispatches_status_vocab). blocked = a dependency is not yet present: NOT an error, NOT a stall. Every transition is backed by a canonical_events row in the SAME transaction (contract rule 2); no silent UPDATE.';
COMMENT ON COLUMN dispatches.priority IS
  'Higher pulls first; ties broken by issued_at (contract). Priority tuning/preemption are explicitly OUT of the first slice.';
COMMENT ON TABLE dispatch_dependencies IS
  'Dependency graph AS DATA for governed dispatch (P0 G19). A dispatch is READY iff every row''s referent EXISTS (existence query, never FK). Cross-seat dependencies are expressible by construction, which is what makes a cycle DETECTABLE rather than merely possible (contract: detect + escalate via SupervisorMessage, never hang).';
COMMENT ON COLUMN dispatch_dependencies.dep_value IS
  'The referent id. Polymorphic across dep_kind (canonical_events.id | artifacts.id | dispatches.id), so DELIBERATELY no FK — same reasoning as event_object_refs.ref_id. A dangling referent is possible by design and reads as blocked via existence query, not as an insert-time error.';

COMMIT;

-- =============================================================================
-- VERIFICATION QUERY (ONYX.md §5.5: a migration ships with one). Run after apply
-- on a SCRATCH database. Every assertion is checked in the delivery doc §3.
-- =============================================================================
--
-- -- Both vocabularies now list the new values:
-- SELECT pg_get_constraintdef(oid) LIKE '%SeatDispatchIssued%'
--    AND pg_get_constraintdef(oid) LIKE '%SeatDispatchDelivered%'
--    AND pg_get_constraintdef(oid) LIKE '%SeatDispatchCompleted%' AS event_vocab_widened
--   FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- SELECT pg_get_constraintdef(oid) LIKE '%dispatch%' AS ref_type_has_dispatch
--   FROM pg_constraint WHERE conname = 'event_object_refs_ref_type_vocab';
--
-- -- Both tables exist with the right domains:
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'dispatches_status_vocab';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'dispatch_dependencies_dep_kind_vocab';
-- SELECT column_name, data_type, is_nullable FROM information_schema.columns
--   WHERE table_name = 'dispatches' ORDER BY ordinal_position;
-- SELECT column_name, data_type, is_nullable FROM information_schema.columns
--   WHERE table_name = 'dispatch_dependencies' ORDER BY ordinal_position;
--
-- -- The load-bearing one-in-flight index exists and is partial on status='in_flight':
-- SELECT indexdef FROM pg_indexes WHERE indexname = 'uq_dispatches_one_in_flight_per_seat';
--
-- -- Readiness is by existence, not FK: a dep on a non-existent event id inserts
-- -- cleanly (no error) and the dispatch reads blocked:
-- --   INSERT INTO dispatch_dependencies VALUES ('<d>', 'event', 'ev-does-not-exist');  -- succeeds
-- --   SELECT NOT EXISTS (SELECT 1 FROM canonical_events WHERE id='ev-does-not-exist'); -- t => blocked
--
-- -- The one-in-flight rule is a DB fact:
-- --   two in_flight for the SAME seat  -> second INSERT rejected by the unique index
-- --   in_flight for a DIFFERENT seat   -> accepted
-- --   done + in_flight for the SAME seat -> accepted (only in_flight is unique-scoped)
--
-- -- Idempotency: a second `psql -f 012_governed_dispatch.sql` performs NO catalog
-- -- write (all DO-blocks guard-skip; CREATE ... IF NOT EXISTS are no-ops).
