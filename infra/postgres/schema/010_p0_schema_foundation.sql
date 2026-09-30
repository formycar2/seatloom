-- 010_p0_schema_foundation.sql
--
-- P0 build plan §3 step 1 — the joins that make one captured record expressible:
-- "a record in SeatLoom because an agent, on some harness, did something — filed
-- under its project and topic, linked to what it touched, retrievable, and
-- independent of which harness produced it."
--
-- Packet: ONYX-2026-09-30-p0-schema-foundation-v1
-- Gaps closed: G2 (event envelope), G3 (ref roles), G5 (verify-only),
--              G6 (topics + event->topic via the EXISTING ref fabric),
--              G12 (reserve the turn key).
--
-- =============================================================================
-- PLACEMENT: POST-SEED. Registered in scripts/apply-db.sh's POST_SEED_SCHEMA.
-- =============================================================================
-- By the rule apply-db.sh encodes: post-seed if the migration constrains columns
-- the seed does not provide. This adds canonical_events.schema_version NOT NULL
-- and event_object_refs.role NOT NULL; seed 001 inserts into BOTH tables and
-- supplies NEITHER column. Pre-seed would break the seed exactly as 008 would
-- have. An unregistered migration is invisible to CI, which builds from empty
-- every pipeline.
--
-- =============================================================================
-- APPEND-ONLY HISTORY — the backfill rules are load-bearing
-- =============================================================================
-- canonical_events and event_object_refs are append-only history. Backfills here
-- assert the weakest defensible claim, never a convenient one:
--
--   schema_version -> 1       existing rows are the FIRST shape of their type.
--   turn_seq       -> NULL    not recorded. NULL is never "zero".
--   role           -> context weakest, least-wrong role. See the G3 block.
--
-- NOTHING is backfilled to role='subject'. Asserting a subject the data does not
-- support is the same class of error as writing 'Stepcode' onto an April session
-- row (see ONYX-2026-09-28-runtime-triple-migration-v1 §2.3).
--
-- =============================================================================
-- IDEMPOTENCY
-- =============================================================================
-- ADD COLUMN IF NOT EXISTS; every backfill guarded by `WHERE <target> IS NULL`;
-- SET NOT NULL is a no-op when already set; CREATE TABLE/INDEX IF NOT EXISTS;
-- every CHECK and the primary-key swap is wrapped in a catalog-guarded DO block
-- so a second apply performs NO catalog write at all (not even a drop+re-add).
-- Whole file is one transaction. Verified by applying twice.

BEGIN;

-- =============================================================================
-- G2 — event envelope: schema_version, closed event_type vocabulary, turn key
-- =============================================================================

ALTER TABLE canonical_events ADD COLUMN IF NOT EXISTS schema_version INT;

-- G12: reserve the T2 join key. turn_seq is the event's ordinal position within
-- its session's turn stream. T2 capture is P1 — this only reserves the column.
ALTER TABLE canonical_events ADD COLUMN IF NOT EXISTS turn_seq INT;

-- Backfill: existing rows are the first shape of their event_type, so version 1.
UPDATE canonical_events SET schema_version = 1 WHERE schema_version IS NULL;

-- turn_seq is deliberately NOT backfilled. Every existing row predates T2
-- capture, so its turn position was never recorded. NULL means *not recorded*;
-- 0 would be a fabricated ordinal.

ALTER TABLE canonical_events ALTER COLUMN schema_version SET NOT NULL;

-- DEFAULT 1 — REQUIRED so this migration does not break the L1 Supervisor IM
-- write path, which cannot be updated in this packet (.rs is out of scope; the
-- Rust mirror is Nimbus step 2).
--
-- crates/seatloom-core/src/db/repositories.rs:421-424 inserts
--   (id, event_type, occurred_at, actor_ref, payload, project_id)
-- and supplies NO schema_version. Without a default, NOT NULL would make every
-- Supervisor IM message fail at runtime with
--   'null value in column "schema_version" violates not-null constraint'
-- the moment 010 lands -- and CI would NOT catch it: no test exercises that path
-- against the database, and reconcile emits no events.
--
-- 1 is the honest value, not a convenience: it is the same claim the legacy
-- backfill above makes -- an unversioned writer produces the first shape of its
-- type. Nimbus's write API (step 2) must set schema_version EXPLICITLY; dropping
-- this default once no writer relies on it is a candidate follow-up.
DO $$
BEGIN
  IF (SELECT column_default FROM information_schema.columns
       WHERE table_name = 'canonical_events' AND column_name = 'schema_version') IS NULL THEN
    ALTER TABLE canonical_events ALTER COLUMN schema_version SET DEFAULT 1;
  END IF;
END $$;

DO $$
BEGIN
  -- schema_version >= 1. Versioning is PER event_type: each event_type carries
  -- its own independent version sequence starting at 1. Which (type, version)
  -- pairs are valid, and what payload shape each implies, is the write API's
  -- contract (G1/A1, Nimbus step 2) — not expressible as a column constraint.
  IF NOT EXISTS (SELECT 1 FROM pg_constraint
                  WHERE conname = 'canonical_events_schema_version_positive') THEN
    ALTER TABLE canonical_events
      ADD CONSTRAINT canonical_events_schema_version_positive
      CHECK (schema_version >= 1);
  END IF;

  -- turn_seq >= 0 when recorded (packet §4 domain: integer >= 0, nullable).
  IF NOT EXISTS (SELECT 1 FROM pg_constraint
                  WHERE conname = 'canonical_events_turn_seq_nonneg') THEN
    ALTER TABLE canonical_events
      ADD CONSTRAINT canonical_events_turn_seq_nonneg
      CHECK (turn_seq IS NULL OR turn_seq >= 0);
  END IF;

  -- Closed event_type vocabulary, PascalCase (packet §4 binding rule 1: match
  -- what is already in PostgreSQL — existing values are PascalCase).
  --
  -- Extensible by migration, deliberately: cheap to widen, expensive to get
  -- wrong in an append-only table. PENDING LYRA SIGN-OFF (packet §2.1).
  --
  -- Membership is justified per value, in three groups:
  --
  --  (a) PRESENT IN COMMITTED DATA — 10 values, all of seed 001's
  --      canonical_events rows. Measured at b6de097 and cross-checked against
  --      the live database (identical set, 10/10).
  --  (b) INSERTED BY LIVE CODE, PRESENT IN NO ROW YET — 2 values. The L1
  --      Supervisor IM write path defaults event_type to 'SupervisorMessage'
  --      (src-tauri/src/commands/supervisor_cmds.rs:45) and the frontend
  --      distinguishes 'SupervisorMessage' from 'SeatResponse'
  --      (ui/src/app-v2/hooks/useSupervisorData.ts:181-182). Omitting these
  --      would make this CHECK break Supervisor IM at runtime the first time a
  --      message is sent. They appear in NO seed row, so a vocabulary derived
  --      from data alone would have missed them.
  --
  -- There is deliberately NO third group for a P0-first-slice value. An earlier
  -- draft of this migration added 'ArtifactProduced'; it was dropped on Lyra's
  -- sign-off (AEGIS-2026-09-30-p0-step1-signoff-v1, ratified by Mr. Zhang). The
  -- first slice REUSES the existing 'ArtifactCreated' rather than introducing a
  -- near-synonym into an append-only table, where two words for one meaning can
  -- never be withdrawn once rows carry both. 'ArtifactCreated' is already in
  -- committed data (seed 001, rows ev-050 through ev-055), so capture needs no
  -- new value. Do not re-add one without the same sign-off.
  IF NOT EXISTS (SELECT 1 FROM pg_constraint
                  WHERE conname = 'canonical_events_event_type_vocab') THEN
    ALTER TABLE canonical_events
      ADD CONSTRAINT canonical_events_event_type_vocab
      CHECK (event_type IN (
        -- (a) present in committed data
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
        -- (b) inserted by live code, in no row yet
        'SupervisorMessage',
        'SeatResponse'
      ));
  END IF;
END $$;

-- payload: DELIBERATELY LEFT NULLABLE by this migration.
--
-- It could be tightened — zero existing rows have a NULL payload (verified on
-- both the seed build and the live database). It is not tightened because:
--   1. NULL (not recorded) and '{}' (recorded, empty) are a real distinction,
--      and this codebase's NULL discipline treats collapsing them as a defect.
--   2. G2 asks for payload VALIDATED against (event_type, schema_version).
--      NOT NULL does not validate shape; it would give false assurance while
--      the actual validation lives in the write API (Nimbus step 2).
--   3. No P0 join depends on payload being present.
--   4. Tightening a column in append-only history is not cheaply reversible.
-- If the write API later guarantees a payload for every type, a follow-up
-- migration can tighten it against a table that is then known to conform.

-- =============================================================================
-- G3 — ref roles: role, constrained ref_type, primary key, one-subject rule
-- =============================================================================

ALTER TABLE event_object_refs ADD COLUMN IF NOT EXISTS role TEXT;

-- Backfill: 'context' for every existing ref.
--
-- Evidence and reasoning: platform design A3 measured 21 'workitem' refs for 6
-- workitems (confirmed here at b6de097: 21 workitem, 7 handoff, 6 artifact,
-- 5 session, 2 delegation). Existing refs carry no role distinction and cannot
-- be mechanically separated into subject/actor/context. 'context' is the
-- weakest, least-wrong claim: it says "this event relates to this object"
-- without asserting agency or aboutness.
--
-- NOT backfilled to 'subject' — see the header.
--
-- NOT backfilled to 'actor' either, although packet §3 permits establishing an
-- actor subset from canonical_events.actor_ref. That subset is empty here: every
-- actor_ref in committed data is of the form 'seat:seat-<name>-001', and
-- event_object_refs contains ZERO rows with ref_type='seat' (measured: the five
-- ref_types present are artifact/delegation/handoff/session/workitem). So no
-- existing ref row is the actor named by its event, and an actor backfill would
-- have nothing to match. ref_type='seat' refs ARE written at runtime
-- (crates/seatloom-core/src/db/repositories.rs:430-431, the Supervisor IM path)
-- but none has been persisted into the seeded history this migration backfills.
UPDATE event_object_refs SET role = 'context' WHERE role IS NULL;

ALTER TABLE event_object_refs ALTER COLUMN role SET NOT NULL;

-- DEFAULT 'context' — REQUIRED for the same reason, with the same caveat.
--
-- crates/seatloom-core/src/db/repositories.rs:430-431 inserts
--   (event_id, ref_type, ref_id)
-- and supplies NO role, so NOT NULL without a default would break the
-- Supervisor IM seat-targeting ref at runtime.
--
-- 'context' is the weakest claim, matching the legacy backfill rule: a writer
-- that does not state a role gets "related to this event" and nothing stronger.
-- The failure mode is therefore UNDER-specification, never a fabricated subject
-- or actor -- and a defaulted row can never trip the one-subject index.
--
-- Deliberately a transitional affordance. The write API (G1/A1, Nimbus step 2)
-- MUST set role explicitly; a defaulted role in new code is a bug, and dropping
-- this default after step 2 lands is a candidate follow-up. Flagged to Lyra and
-- Nimbus in the delivery doc.
DO $$
BEGIN
  IF (SELECT column_default FROM information_schema.columns
       WHERE table_name = 'event_object_refs' AND column_name = 'role') IS NULL THEN
    ALTER TABLE event_object_refs ALTER COLUMN role SET DEFAULT 'context';
  END IF;
END $$;

DO $$
BEGIN
  -- role domain (packet §4), snake_case to match existing lowercase ref_type
  -- values. Flat TEXT so a hand-written FromStr/Display is possible on the Rust
  -- side (ENGINEERING_STANDARDS; objects/session.rs::parse_custom_tagged).
  IF NOT EXISTS (SELECT 1 FROM pg_constraint
                  WHERE conname = 'event_object_refs_role_vocab') THEN
    ALTER TABLE event_object_refs
      ADD CONSTRAINT event_object_refs_role_vocab
      CHECK (role IN ('subject', 'actor', 'context'));
  END IF;

  -- ref_type domain: the object vocabulary.
  --
  -- DEVIATION FROM THE PACKET §4 CONTRACT — 'delegation' is added, and this is
  -- flagged for Lyra's sign-off in the delivery doc (§4 requires telling Nimbus
  -- and posting before landing; both done).
  --
  -- Reason: packet §4 lists {workitem, session, handoff, artifact, document,
  -- seat, topic, project} and omits 'delegation' — but 'delegation' EXISTS in
  -- committed seed data (2 rows, ev-020 and ev-021 -> del-flux-acting-mira-001,
  -- measured at b6de097). Constraining to §4 as written would fail immediately
  -- on existing history, which packet §2.2 explicitly rules unacceptable. It is
  -- a real object type: seat_delegations is a table (schema 001, AD-009 Layer 3)
  -- with its own SeatDelegation object. Its omission reads as an oversight in
  -- the contract, not a deliberate exclusion, so the least-wrong action is to
  -- include it and surface the contract gap rather than rewrite history or ship
  -- a constraint that cannot be validated.
  --
  -- The other three §4 values not yet in data (document, topic, project) are
  -- kept: they are forward-looking and cost nothing, and 'topic' is required by
  -- G6's event->topic path in this very migration.
  IF NOT EXISTS (SELECT 1 FROM pg_constraint
                  WHERE conname = 'event_object_refs_ref_type_vocab') THEN
    ALTER TABLE event_object_refs
      ADD CONSTRAINT event_object_refs_ref_type_vocab
      CHECK (ref_type IN (
        'workitem', 'session', 'handoff', 'artifact',
        'document', 'seat', 'topic', 'project',
        'delegation'   -- present in committed data; see above
      ));
  END IF;
END $$;

-- Primary key: (event_id, ref_type, ref_id) -> (event_id, ref_type, ref_id, role)
--
-- DECISION: role JOINS the key. Reasoning, recorded because this is structural.
--
-- The old key makes it impossible for one event to reference the same object
-- under two roles. That forbids self-referential events, which are ordinary in a
-- multi-agent system: if aegis pauses mira the actor and subject differ, but if
-- mira pauses her own session, closes her own topic, or emits an artifact she
-- also owns, then the SAME object is both 'actor' and 'subject' of ONE event.
-- Under the old key that second ref collides and is silently dropped by the
-- ON CONFLICT DO NOTHING the write path uses — losing exactly the role
-- distinction G3 exists to record.
--
-- Widening is migration-safe: the old key already guaranteed (event_id,
-- ref_type, ref_id) unique, and the backfill sets one uniform role, so every
-- existing row is trivially unique under the wider key. No row can violate it.
--
-- Cost accepted: the wider key permits the same (event, type, id) tagged under
-- two different roles, so it no longer catches a double-tag as a key violation.
-- For 'subject' that is caught by the partial unique index below. For 'actor'
-- and 'context' a duplicate is benign — the same object genuinely can be
-- context for an event for more than one reason.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1
      FROM pg_constraint c
      JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = ANY (c.conkey)
     WHERE c.conrelid = 'event_object_refs'::regclass
       AND c.contype  = 'p'
       AND a.attname  = 'role'
  ) THEN
    ALTER TABLE event_object_refs DROP CONSTRAINT IF EXISTS event_object_refs_pkey;
    ALTER TABLE event_object_refs
      ADD CONSTRAINT event_object_refs_pkey
      PRIMARY KEY (event_id, ref_type, ref_id, role);
  END IF;
END $$;

-- One-subject rule — AT MOST ONE subject per event, enforced by a PARTIAL
-- UNIQUE INDEX. This is the form chosen (packet §2.2 asks which).
--
-- Why partial: after the backfill above, EVERY legacy ref is role='context', so
-- no legacy row falls inside the index predicate and none can violate it. A
-- blanket constraint would fail on existing history immediately.
--
-- HONEST LIMIT, stated because the gap matters: this enforces AT MOST one
-- subject, not EXACTLY one. "Exactly one" requires asserting PRESENCE, which no
-- unique index can do, and which every legacy event would violate since none
-- has a subject at all. So:
--   * the database guarantees no event has two subjects — forever, including
--     for legacy rows;
--   * "every NEW event has exactly one subject" is the write API's obligation
--     (G1/A1, Nimbus step 2), where it can be enforced per-append without
--     retroactively condemning history.
CREATE UNIQUE INDEX IF NOT EXISTS uq_event_object_refs_one_subject_per_event
  ON event_object_refs (event_id)
  WHERE role = 'subject';

-- Lookup path for "which events touched this object, in what role".
CREATE INDEX IF NOT EXISTS idx_event_object_refs_target_role
  ON event_object_refs (ref_type, ref_id, role);

-- =============================================================================
-- G5 — VERIFY, DO NOT ADD: sessions.project_id
-- =============================================================================
-- sessions.project_id ALREADY EXISTS and is already NOT NULL. Measured at
-- b6de097:
--   008_project_isolation.sql:51   ALTER TABLE sessions ADD COLUMN IF NOT EXISTS
--                                  project_id TEXT REFERENCES projects(id)
--                                  DEFAULT 'seatloom';
--   008_project_isolation.sql:106  ALTER TABLE sessions ALTER COLUMN project_id
--                                  DROP DEFAULT;
--   008_project_isolation.sql:107  ALTER TABLE sessions ALTER COLUMN project_id
--                                  SET NOT NULL;
--
-- This migration adds NO column for it and issues NO DDL against sessions. The
-- column is wired into the value-domain contract and asserted by the shipped
-- verification query below. Re-adding it would be a no-op at best and a
-- constraint conflict at worst.
--
-- Guard: fail loudly if that premise is ever false, rather than silently
-- building P0's project filing on a column that is not there.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
     WHERE table_name = 'sessions' AND column_name = 'project_id'
  ) THEN
    RAISE EXCEPTION
      'G5 premise violated: sessions.project_id does not exist. 010 expects 008 to have added it (008:51) and tightened it (008:106-107). Apply 008 first; do NOT add the column here.';
  END IF;
  IF EXISTS (
    SELECT 1 FROM information_schema.columns
     WHERE table_name = 'sessions' AND column_name = 'project_id'
       AND is_nullable = 'YES'
  ) THEN
    RAISE EXCEPTION
      'G5 premise violated: sessions.project_id exists but is NULLABLE. 008:107 should have set it NOT NULL.';
  END IF;
END $$;

-- =============================================================================
-- G6 — topics, and event->topic over the EXISTING ref fabric
-- =============================================================================
-- Columns are exactly those the packet specifies, deliberately: no created_at,
-- no lifecycle CHECK beyond the status domain. G6 says reserve the link and
-- "nothing that decides what a topic is" — auto-segmentation is P1.
--
-- Event->topic needs NO NEW MECHANISM (object model §3.3, "zero new primitive"):
-- it is event_object_refs with ref_type='topic' and role='subject', both of
-- which are now in the constrained domains above. There is deliberately no
-- foreign key from event_object_refs.ref_id to topics.id — ref_id is
-- polymorphic across eight object types and a per-type FK is exactly the new
-- primitive G6 forbids.
CREATE TABLE IF NOT EXISTS topics (
    id               TEXT        PRIMARY KEY,
    project_id       TEXT        NOT NULL REFERENCES projects(id),
    title            TEXT        NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'open',
    opened_by        TEXT        NOT NULL,
    opened_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at        TIMESTAMPTZ,
    parent_topic_id  TEXT        REFERENCES topics(id)
);

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'topics_status_vocab') THEN
    ALTER TABLE topics
      ADD CONSTRAINT topics_status_vocab
      CHECK (status IN ('open', 'closed', 'merged'));
  END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_topics_project_status ON topics (project_id, status);

COMMENT ON TABLE  topics IS
  'Topic under which captured events are filed (P0 G6, ONYX-2026-09-30-p0-schema-foundation-v1). Events link to a topic via event_object_refs (ref_type=''topic'', role=''subject'') — no dedicated join table. Auto-segmentation is P1; this table only reserves the object.';
COMMENT ON COLUMN topics.opened_by IS
  'Actor ref that opened the topic, same shape as canonical_events.actor_ref (e.g. ''seat:seat-onyx-001'', ''human:user'').';
COMMENT ON COLUMN topics.parent_topic_id IS
  'Optional parent for topic nesting. Reserved; no traversal logic in P0.';

COMMENT ON COLUMN canonical_events.schema_version IS
  'Payload shape version, per event_type, starting at 1 (P0 G2). Existing rows backfilled to 1 as the first shape of their type. Which (event_type, schema_version) pairs are valid and what payload each implies is the write API''s contract, not a column constraint.';
COMMENT ON COLUMN canonical_events.turn_seq IS
  'Ordinal position of this event within its session''s turn stream — the T2 join key (session_id, turn_seq) (P0 G12). The session half is expressed via event_object_refs (ref_type=''session''), NOT a column here, so the existing ref fabric stays the only linkage mechanism. NULL means NOT RECORDED, never zero; T2 capture is P1.';
COMMENT ON COLUMN event_object_refs.role IS
  'How this object relates to the event: subject (what the event is about), actor (who did it), context (related, without asserting agency or aboutness). P0 G3. Legacy rows are ''context'' — they predate role capture and cannot be mechanically separated. At most one ''subject'' per event is enforced by the partial unique index uq_event_object_refs_one_subject_per_event; EXACTLY one for new events is the write API''s obligation.';

COMMIT;

-- =============================================================================
-- VERIFICATION QUERY (ONYX.md: a migration ships with one)
-- Run after apply. The two counts MUST be 0; every DISTINCT list must sit inside
-- the packet §4 value-domain contract.
-- =============================================================================
--
-- SELECT count(*) AS events_missing_schema_version FROM canonical_events WHERE schema_version IS NULL;
-- SELECT count(*) AS refs_missing_role             FROM event_object_refs WHERE role IS NULL;
--
-- SELECT DISTINCT role     FROM event_object_refs ORDER BY 1;
-- SELECT DISTINCT ref_type FROM event_object_refs ORDER BY 1;
-- SELECT DISTINCT event_type, schema_version FROM canonical_events ORDER BY 1, 2;
-- SELECT DISTINCT status   FROM topics ORDER BY 1;
--
-- -- G5: verified, not added.
-- SELECT column_name, is_nullable FROM information_schema.columns
--  WHERE table_name = 'sessions' AND column_name = 'project_id';
--
-- -- G12 / §3: turn_seq must be entirely unrecorded, never fabricated.
-- SELECT count(*) AS events_with_turn_seq FROM canonical_events WHERE turn_seq IS NOT NULL;
--
-- -- Transitional defaults that keep the pre-step-2 Supervisor IM writers alive.
-- SELECT table_name, column_name, column_default FROM information_schema.columns
--  WHERE (table_name='canonical_events'  AND column_name='schema_version')
--     OR (table_name='event_object_refs' AND column_name='role');
--
-- -- §3: no legacy row may claim a subject.
-- SELECT count(*) AS refs_claiming_subject FROM event_object_refs WHERE role = 'subject';
--
-- -- G3: the one-subject rule is enforced and the PK includes role.
-- SELECT indexdef FROM pg_indexes WHERE indexname = 'uq_event_object_refs_one_subject_per_event';
-- SELECT a.attname FROM pg_constraint c JOIN pg_attribute a
--          ON a.attrelid = c.conrelid AND a.attnum = ANY (c.conkey)
--  WHERE c.conrelid = 'event_object_refs'::regclass AND c.contype = 'p'
--  ORDER BY a.attname;
