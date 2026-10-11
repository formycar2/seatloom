-- 020_gate_criterion.sql
-- =============================================================================
-- A delivery announcement is not a gate — the criterion at RELEASE time.
--
-- Authority: Aegis ruling 2026-10-11,
-- docs/coordination/tasks/aegis/AEGIS-2026-10-11-gate-vs-announcement-ruling.md
-- (裁决一 "凭 payload 判定一个事件能不能当门", 裁决四 "校验只能发生在释放时刻").
-- The criterion's SHAPE is Nimbus's tool contract —
-- crates/seatloom-mcp-emit/src/gate.rs on packet/nimbus/gate-vs-announcement
-- (commit 1a4b75e) — and this file is its SQL twin. Both sides answered the
-- four questions; the answers and the evidence are in
-- docs/coordination/tasks/onyx/ONYX-2026-10-12-gate-release-check-delivery-v1.md.
--
-- Slot 020: next after 019 (006/007 stay reserved — do NOT renumber). POST-SEED:
-- the seed writes no dispatch_dependencies rows and none of the objects here
-- touch a seeded table's columns; the widened CHECKs and the function this
-- replaces exist only after 013 (POST-SEED) has created them, so 020 cannot
-- precede its prerequisite. See scripts/apply-db.sh.
--
-- Additive and idempotent, 013/015/017/019 house style: every object is guarded
-- on the catalog, so a SECOND apply performs NO catalog write.
--
-- WHAT WAS WRONG (013, restated because the fix is only legible next to it):
-- fn_dispatch_recompute_ready flipped blocked -> ready on referent EXISTENCE.
-- An existence query cannot tell "the code landed" from "the capability runs",
-- so an edge hung on a delivery announcement was satisfied from the moment it
-- was hung — and disp-onight-writein was released onto a capability that had
-- never once succeeded. One event id carried two meanings.
--
-- WHAT THIS DOES: for an edge whose dep_value is GATE-SHAPED, the referent must
-- exist AND be a measured gate. Every other edge keeps 013's behaviour
-- byte-for-byte. The criterion lives in ONE place per language — gate.rs on the
-- Rust side, fn_gate_row_is_gate() here — so producer, edge and release cannot
-- drift apart. Two copies of one criterion that disagree is the failure mode
-- this ruling exists to prevent.
-- =============================================================================
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- The two criterion helpers exist and are IMMUTABLE (required for a CHECK):
-- SELECT proname, provolatile FROM pg_proc
--  WHERE proname IN ('fn_gate_is_gate_id','fn_gate_row_is_gate');
-- -- The release function now carries the criterion (and still returns trigger):
-- SELECT pg_get_functiondef('fn_dispatch_recompute_ready()'::regprocedure)
--   LIKE '%fn_gate_is_gate_id%';
-- -- A gate-shaped id on a non-event edge is now refused by the DATABASE:
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint
--  WHERE conname = 'dispatch_dependencies_gate_edge_must_be_event';
-- -- The held-by-a-non-qualified-gate set is queryable without trusting Rust:
-- SELECT * FROM v_dispatch_gate_held;
-- -- Idempotency: a second `psql -f 020_gate_criterion.sql` performs NO catalog
-- -- write — all five DO-blocks guard-skip, including the view's and the
-- -- comments' (an unconditional COMMENT ON is a pg_description write).
-- =============================================================================

BEGIN;

-- =============================================================================
-- The criterion, in SQL — one function per predicate, mirroring gate.rs
-- =============================================================================
-- Guarded CREATE (no DROP): a re-apply creates nothing. A future change to the
-- criterion's SHAPE ships as its own migration that DROPs and re-CREATEs these,
-- exactly as 013 anticipated for its own function body.

-- fn_gate_is_gate_id — the SQL twin of gate.rs::is_gate_id.
--
-- A SEGMENT test, not a suffix test, and that is a finding rather than a
-- preference: ruling 2 says new gate ids are uniformly `ev-<capability>-live`,
-- but the first gate actually emitted under it — ev-live-019-applied, Aegis's
-- own — ends in `-applied`. A suffix-only test would have read a real, fully
-- verified gate as an announcement. Both shapes are matched; the test can only
-- over-recognise (one extra payload read), never under-recognise.
--
-- This is a SHAPE test only. It says "the issuer meant this as a gate", never
-- "this event is a gate" — that is fn_gate_row_is_gate's job, applied to the row
-- that eventually lands.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
     WHERE p.proname = 'fn_gate_is_gate_id' AND n.nspname = current_schema()
  ) THEN
    CREATE FUNCTION fn_gate_is_gate_id(event_id TEXT) RETURNS boolean
    LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $fn$
      SELECT coalesce(
               event_id LIKE '%-live-%' OR event_id LIKE '%-live',
               false)
    $fn$;
  END IF;
END $$;

-- fn_gate_row_is_gate — the SQL twin of gate.rs::row_is_gate / gate_payload_shape.
--
-- A gate iff ALL of:
--   * event_type = 'SupervisorMessage'  (the type that already means "a
--     statement from a seat about the state of the system"; ArtifactCreated —
--     the announcement type — is deliberately excluded), and
--   * payload->>'gate' = the event's OWN id (so a row cannot be re-pointed at a
--     different gate by editing its payload), and
--   * statement / verified / does_not_assert are all STATED.
--
-- "Stated" is the load-bearing word and it is where the two languages could
-- have drifted. Rust's gate.rs::stated() treats absent, null, empty AND
-- whitespace-only as "not stated" — a gate that claims to be verified by a
-- blank string is verified by nothing. So this function trims, and it trims
-- EXACTLY Rust's whitespace set (char::is_whitespace = the Unicode White_Space
-- property: U+0009-U+000D, U+0020, U+0085, U+00A0, U+1680, U+2000-U+200A,
-- U+2028, U+2029, U+202F, U+205F, U+3000), enumerated below rather than taken
-- from the locale's [[:space:]], because a SUBSET of Rust's set would be the
-- dangerous direction: the database would release a dispatch the seat then
-- refuses to pull, which is the silent hang this ruling exists to prevent.
--
-- A non-object payload (a JSON scalar or array) yields NULL for every ->> and
-- so reads as "not stated" — the same answer Rust's payload.get() gives.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
     WHERE p.proname = 'fn_gate_row_is_gate' AND n.nspname = current_schema()
  ) THEN
    CREATE FUNCTION fn_gate_row_is_gate(
        event_type TEXT, event_id TEXT, payload JSONB) RETURNS boolean
    LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $fn$
      SELECT coalesce(
               event_type = 'SupervisorMessage'
               AND payload ->> 'gate' = event_id
               AND btrim(coalesce(payload ->> 'statement', ''),      ws) <> ''
               AND btrim(coalesce(payload ->> 'verified', ''),       ws) <> ''
               AND btrim(coalesce(payload ->> 'does_not_assert', ''), ws) <> '',
               false)
        FROM (SELECT (' ' || chr(9) || chr(10) || chr(11) || chr(12) || chr(13)
                     || chr(133) || chr(160) || chr(5760)
                     || chr(8192) || chr(8193) || chr(8194) || chr(8195)
                     || chr(8196) || chr(8197) || chr(8198) || chr(8199)
                     || chr(8200) || chr(8201) || chr(8202)
                     || chr(8232) || chr(8233) || chr(8239) || chr(8287)
                     || chr(12288))::TEXT AS ws) w
    $fn$;
  END IF;
END $$;



-- =============================================================================
-- The release-time check — fn_dispatch_recompute_ready body replacement
-- =============================================================================
-- WHY CREATE OR REPLACE AND NOT DROP + CREATE (013's header anticipated the
-- latter): the three 013 triggers trg_dispatch_ready_on_event / _on_artifact /
-- _on_dispatch are bound to this function. A DROP FUNCTION would take them with
-- it (or fail without CASCADE), so replacing the body in place is the only
-- change that leaves the trigger wiring untouched. Same name, same signature,
-- same return type — the triggers need no re-creation.
--
-- Guarded on the INSTALLED body already calling fn_gate_is_gate_id: that call
-- is the load-bearing difference, so the guard tests semantics rather than
-- formatting, and a re-apply is a true no-op.
DO $$
DECLARE installed TEXT;
BEGIN
  SELECT pg_get_functiondef(p.oid) INTO installed
    FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
   WHERE p.proname = 'fn_dispatch_recompute_ready' AND n.nspname = current_schema();

  IF installed IS NOT NULL AND installed LIKE '%fn_gate_is_gate_id%' THEN
    RAISE NOTICE '020: fn_dispatch_recompute_ready already carries the gate criterion — nothing to do';
  ELSE
    CREATE OR REPLACE FUNCTION fn_dispatch_recompute_ready() RETURNS trigger
    LANGUAGE plpgsql AS $fn$
    DECLARE
      ref_kind  TEXT;
      cand      RECORD;
      total     INT;
      satisfied INT;
    BEGIN
      -- Which dep_kind does the table this trigger fired on satisfy? Referent id
      -- spaces collide across the three tables (all TEXT), so match kind to the
      -- firing table rather than trusting dep_value alone. (013, unchanged.)
      IF TG_TABLE_NAME = 'canonical_events' THEN ref_kind := 'event';
      ELSIF TG_TABLE_NAME = 'artifacts'  THEN ref_kind := 'artifact';
      ELSIF TG_TABLE_NAME = 'dispatches' THEN ref_kind := 'dispatch';
      ELSE RETURN NEW; END IF;  -- defensive: unknown table, no-op

      -- Every dispatch that is CURRENTLY blocked and names the just-inserted
      -- referent as one of its dependencies is a candidate to unblock. (013,
      -- unchanged.)
      FOR cand IN
        SELECT d.id, d.target_seat_id
          FROM dispatches d
          JOIN dispatch_dependencies dd
            ON dd.dispatch_id = d.id
           AND dd.dep_kind     = ref_kind
           AND dd.dep_value    = NEW.id
         WHERE d.status = 'blocked'
      LOOP
        -- Re-evaluate ALL of this dispatch's deps. One referent landing may
        -- complete the set, or may not; count total deps and satisfied deps.
        SELECT count(*) AS total,
               count(*) FILTER (WHERE sat) AS satisfied
          INTO total, satisfied
          FROM (
            SELECT CASE
                     -- GATE EDGE (020): the referent must exist AND be a
                     -- measured gate. This is the ruling's criterion — a
                     -- payload test, not an id test — and it is the same
                     -- predicate dep_satisfied() applies on the pull path. If
                     -- the two ever disagree, the database says `ready` while
                     -- the seat cannot pull it, which is worse than the
                     -- disease: it is a LOUD silent.
                     WHEN dd2.dep_kind = 'event'
                          AND fn_gate_is_gate_id(dd2.dep_value) THEN
                          EXISTS(SELECT 1 FROM canonical_events e
                                  WHERE e.id = dd2.dep_value
                                    AND fn_gate_row_is_gate(
                                          e.event_type, e.id, e.payload))
                     -- ORDINARY EDGES (013, unchanged, matched to the dep_kind's
                     -- own table): referent existence is enough. Ruling 3 keeps
                     -- "wait for the code to land" a legitimate need.
                     WHEN dd2.dep_kind = 'event'    THEN
                          EXISTS(SELECT 1 FROM canonical_events e WHERE e.id = dd2.dep_value)
                     WHEN dd2.dep_kind = 'artifact' THEN
                          EXISTS(SELECT 1 FROM artifacts a       WHERE a.id = dd2.dep_value)
                     WHEN dd2.dep_kind = 'dispatch' THEN
                          EXISTS(SELECT 1 FROM dispatches x      WHERE x.id = dd2.dep_value)
                     -- Any other dep_kind yields NULL, which FILTER (WHERE sat)
                     -- counts as unsatisfied. 013's behaviour, preserved.
                   END AS sat
              FROM dispatch_dependencies dd2
             WHERE dd2.dispatch_id = cand.id
          ) depcheck;

        -- total > 0 guards a zero-dep row (which is 'ready' by definition and so
        -- never a blocked candidate anyway); satisfied = total means the set is
        -- now complete. (013, unchanged.)
        IF total > 0 AND satisfied = total THEN
          -- Persist the flip. GUARDED: only blocked -> ready; a row that raced
          -- to in_flight/done/failed/cancelled/ready is left untouched (0 rows),
          -- so this commutes safely with the app's transitions. (013, unchanged.)
          UPDATE dispatches SET status = 'ready'
           WHERE id = cand.id AND status = 'blocked';
          -- Wake the seat through the SAME channel enqueue_dispatch uses.
          -- (013, unchanged.)
          PERFORM pg_notify('dispatch_ready', cand.target_seat_id);
        END IF;
      END LOOP;

      RETURN NEW;
    END;
    $fn$;
  END IF;
END $$;


-- =============================================================================
-- The naming convention, enforced by the DATABASE
-- =============================================================================
-- A gate is always an EVENT row: the criterion reads event_type and payload,
-- and only canonical_events has them. So a gate-shaped dep_value on a non-event
-- edge is a mistake, not a preference. Nimbus's enqueue_dispatch already refuses
-- it (GateEdgeUndeclared); this is the same refusal one layer down, so a writer
-- that bypasses the tool — a script, a manual psql session, the integration
-- tests that construct the 10-10 ledger state by direct SQL — cannot create the
-- shape either. Reversible: drop the constraint in a later migration if the
-- convention is ever retired.
--
-- Guarded on the constraint's absence, so a re-apply performs no catalog write.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'dispatch_dependencies_gate_edge_must_be_event'
  ) THEN
    ALTER TABLE dispatch_dependencies
      ADD CONSTRAINT dispatch_dependencies_gate_edge_must_be_event
      CHECK (dep_kind = 'event' OR NOT fn_gate_is_gate_id(dep_value));
  END IF;
END $$;


-- =============================================================================
-- "It must not hang silently" — made queryable, not merely asserted
-- =============================================================================
-- Question 3 of the four put to this seat was whether a dispatch that can never
-- get a qualified gate should stay `blocked` plus one escalation from the pull
-- path, or get a reportable terminal state. Answer: blocked + escalation, and NO
-- new terminal state — the reasoning is in the delivery doc §4.3. A
-- `gate_unverified` value would not be terminal in the sense 013's failed/
-- cancelled are (a qualified gate could still land and release it), so it would
-- be a sub-state of blocked with no new release path, bought at the price of a
-- seventh status value, a seventh transition event, and a Lyra re-sign-off.
--
-- What the escalation could NOT do is make the condition checkable from the
-- database: it is emitted by the Rust pull path, which this seat does not own
-- and cannot verify from a migration. This view is that missing half. It is a
-- pure read — no status vocabulary change, no event type, no trigger — and it
-- implements EXACTLY the predicate the pull path escalates on, so the two
-- surfaces cannot disagree about which dispatches are held:
--
--   * the stored status is non-terminal (ready | blocked), AND
--   * EVERY referent exists (existence, matched to the dep_kind's own table), AND
--   * at least one gate-shaped event dep is NOT a qualified gate.
--
-- CREATE OR REPLACE VIEW is naturally idempotent in MEANING but not in the
-- catalog: an unconditional re-apply rewrites the pg_class row even when the
-- definition is byte-identical. Guarded on the installed definition already
-- calling fn_gate_is_gate_id — the load-bearing call — so a re-apply performs
-- no catalog write, the same discipline the three DO-blocks above use.
DO $$
DECLARE vdef TEXT;
BEGIN
  SELECT pg_get_viewdef(c.oid) INTO vdef
    FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
   WHERE c.relname = 'v_dispatch_gate_held' AND c.relkind = 'v'
     AND n.nspname = current_schema();

  IF vdef IS NULL OR vdef NOT LIKE '%fn_gate_is_gate_id%' THEN
    CREATE OR REPLACE VIEW v_dispatch_gate_held AS
    SELECT d.id            AS dispatch_id,
           d.project_id,
           d.target_seat_id,
           d.status,
           held.dep_values AS held_by
      FROM dispatches d
      JOIN LATERAL (
            SELECT string_agg(dd.dep_value, ', ' ORDER BY dd.dep_value) AS dep_values
              FROM dispatch_dependencies dd
             WHERE dd.dispatch_id = d.id
               AND dd.dep_kind = 'event'
               AND fn_gate_is_gate_id(dd.dep_value)
               AND NOT EXISTS (SELECT 1 FROM canonical_events e
                                WHERE e.id = dd.dep_value
                                  AND fn_gate_row_is_gate(e.event_type, e.id, e.payload))
           ) held ON held.dep_values IS NOT NULL
     WHERE d.status IN ('ready', 'blocked')
       AND NOT EXISTS (   -- every referent exists; an unknown dep_kind counts as
                          -- MISSING, which is what referent_exists() answers
             SELECT 1
               FROM dispatch_dependencies dd
              WHERE dd.dispatch_id = d.id
                AND NOT coalesce(CASE dd.dep_kind
                                   WHEN 'event'    THEN EXISTS(SELECT 1 FROM canonical_events e WHERE e.id = dd.dep_value)
                                   WHEN 'artifact' THEN EXISTS(SELECT 1 FROM artifacts a       WHERE a.id = dd.dep_value)
                                   WHEN 'dispatch' THEN EXISTS(SELECT 1 FROM dispatches x      WHERE x.id = dd.dep_value)
                                 END, false)
           );
  ELSE
    RAISE NOTICE '020: v_dispatch_gate_held already carries the gate criterion — nothing to do';
  END IF;
END $$;


-- =============================================================================
-- Comments — guarded, because an unconditional COMMENT ON performs a catalog
-- write to pg_description on every apply, and "a second apply performs NO
-- catalog write" has to be literally true to be worth claiming. format(%L)
-- carries the text so it needs no escaping, and obj_description() is compared
-- with IS DISTINCT FROM so an already-correct comment is left alone.
-- =============================================================================
DO $$
DECLARE c TEXT;
BEGIN
  c := 'Gate/announcement criterion (020, Aegis ruling 2026-10-11): is this event id SHAPED like a gate id? Segment test (-live- or a trailing -live), matching gate.rs::is_gate_id byte-for-byte, because the first gate emitted under ruling 2 (ev-live-019-applied) does not end in -live. Shape only — it never asserts the row is a gate.';
  IF obj_description('fn_gate_is_gate_id(TEXT)'::regprocedure, 'pg_proc') IS DISTINCT FROM c THEN
    EXECUTE format('COMMENT ON FUNCTION fn_gate_is_gate_id(TEXT) IS %L', c);
  END IF;

  c := 'Gate/announcement criterion (020, Aegis ruling 2026-10-11): is this (event_type, id, payload) row a MEASURED gate? SupervisorMessage whose payload.gate names its own id and whose statement/verified/does_not_assert are all stated, whitespace trimmed over Rust char::is_whitespace (Unicode White_Space) enumerated exactly. Matches gate.rs::row_is_gate.';
  IF obj_description('fn_gate_row_is_gate(TEXT, TEXT, JSONB)'::regprocedure, 'pg_proc') IS DISTINCT FROM c THEN
    EXECUTE format('COMMENT ON FUNCTION fn_gate_row_is_gate(TEXT, TEXT, JSONB) IS %L', c);
  END IF;

  c := 'Governed-dispatch (G19, 013): on insert of an event/artifact/dispatch referent, flip any blocked dispatch whose dep set is now fully satisfied to ready (persisted) and pg_notify its seat on the dispatch_ready channel the waker already LISTENs on. 020 tightened the satisfaction test for GATE-SHAPED event deps: such a referent must exist AND be a measured gate (SupervisorMessage whose payload.gate names its own id, with statement/verified/does_not_assert stated), the same criterion gate.rs::row_is_gate applies on the pull path. Ordinary (existence) edges are unchanged.';
  IF obj_description('fn_dispatch_recompute_ready()'::regprocedure, 'pg_proc') IS DISTINCT FROM c THEN
    EXECUTE format('COMMENT ON FUNCTION fn_dispatch_recompute_ready() IS %L', c);
  END IF;

  c := 'A gate is an EVENT row (020): the criterion reads event_type and payload, which only canonical_events carries, so a gate-shaped dep_value (fn_gate_is_gate_id) may only hang on dep_kind=''event''. The database-level twin of enqueue_dispatch''s GateEdgeUndeclared refusal, so a writer that bypasses the tool cannot create the shape either.';
  IF obj_description((SELECT con.oid FROM pg_constraint con
                      WHERE con.conname = 'dispatch_dependencies_gate_edge_must_be_event'
                        AND con.conrelid = 'dispatch_dependencies'::regclass),
                    'pg_constraint') IS DISTINCT FROM c THEN
    EXECUTE format('COMMENT ON CONSTRAINT dispatch_dependencies_gate_edge_must_be_event ON dispatch_dependencies IS %L', c);
  END IF;

  c := '020 (Aegis ruling 2026-10-11, question 3): dispatches that are stuck NOT because a referent is missing but because a gate-shaped dep landed as a delivery announcement instead of a measured gate. Exactly the predicate get_next_dispatch escalates on, expressed as a read so the condition is checkable without trusting the Rust pull path. Empty result = nothing is held this way.';
  IF obj_description('v_dispatch_gate_held'::regclass, 'pg_class') IS DISTINCT FROM c THEN
    EXECUTE format('COMMENT ON VIEW v_dispatch_gate_held IS %L', c);
  END IF;
END $$;

COMMIT;
