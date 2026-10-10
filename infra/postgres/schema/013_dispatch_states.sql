-- 013_dispatch_states.sql
-- =============================================================================
-- Governed-dispatch terminal / failure states (G19 follow-on to 012).
--
-- Authority: docs/coordination/tasks/onyx/ONYX-2026-10-09-dispatch-states-design.md
-- (the §4 design), carried into the Lyra-owned contract
-- docs/coordination/tasks/lyra/LYRA-2026-10-09-d2-acceptance-loop-design-v1.md §3 —
-- SeatDispatchFailed / SeatDispatchRetried / SeatDispatchCancelled are FORMALLY
-- SIGNED OFF by Lyra (D2 dispatch disp-Rlth465QCOr9L5HK_CIu5). Open-question
-- ruling recorded: a retry re-opens to plain `ready` (any eligible seat may pull
-- it) — NO stickiness, no "preferred seat" surface is invented (fleet resilience:
-- a repeatedly-failing job should be pickable by another seat).
--
-- Slot 013: next after 012 (006/007 stay reserved). POST-SEED — it widens CHECKs
-- 010/012 created and adds triggers, so it cannot precede its prerequisites; the
-- seed writes none of the new columns. See scripts/apply-db.sh.
--
-- Additive and idempotent, 010/012 house style: every vocab change is a
-- pg_get_constraintdef-guarded DO block and every object is CREATE ... IF NOT
-- EXISTS / guarded, so a SECOND apply performs NO catalog write.
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- Status domain now 6 values; event vocabulary now 18:
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'dispatches_status_vocab';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- -- Retry columns exist, NOT NULL, default 0:
-- SELECT column_name, data_type, is_nullable, column_default
--   FROM information_schema.columns WHERE table_name='dispatches'
--    AND column_name IN ('retry_count','max_retries');
-- -- Reverse index + function + three triggers exist:
-- SELECT indexdef FROM pg_indexes WHERE indexname='idx_dispatch_dependencies_by_referent';
-- SELECT proname FROM pg_proc WHERE proname='fn_dispatch_recompute_ready';
-- SELECT tgname, tgrelid::regclass FROM pg_trigger
--  WHERE tgname IN ('trg_dispatch_ready_on_event','trg_dispatch_ready_on_artifact','trg_dispatch_ready_on_dispatch');
-- -- Idempotency: a second `psql -f 013_dispatch_states.sql` performs NO catalog
-- -- write (both DO-blocks guard-skip; ADD COLUMN / CREATE INDEX IF NOT EXISTS no-op).
-- =============================================================================

BEGIN;

-- =============================================================================
-- Vocabulary widening #1 — dispatches_status_vocab: 4 -> 6 (+failed, +cancelled)
-- =============================================================================
-- failed    = a seat attempted and did not succeed (terminal, immutable).
-- cancelled = withdrawn by the issuer before completion (terminal, immutable).
-- Both are TERMINAL: like `done`, a terminal row never leaves that state — the
-- dispatches row is a projection of the append-only ledger, and rolling a
-- terminal row back would be a silent rewrite of a recorded fact (rule 2). The
-- one-in-flight partial unique index is scoped to status='in_flight' only, so
-- both new values sit OUTSIDE it and rule 1 is unchanged. Guarded on the new
-- value being absent, so a re-apply finds it present and does nothing.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'dispatches_status_vocab'
       AND pg_get_constraintdef(oid) LIKE '%failed%'
  ) THEN
    ALTER TABLE dispatches
      DROP CONSTRAINT IF EXISTS dispatches_status_vocab;
    ALTER TABLE dispatches
      ADD CONSTRAINT dispatches_status_vocab
      CHECK (status IN ('ready', 'blocked', 'in_flight', 'done', 'failed', 'cancelled'));
  END IF;
END $$;

-- =============================================================================
-- Vocabulary widening #2 — canonical_events_event_type_vocab: 15 -> 18
-- =============================================================================
-- The three transitions that make the failed/retry/cancel paths replayable
-- (rule 2: omitting any one makes the projection unreplayable). 012 widened this
-- 12 -> 15 with the same guard; each lifecycle event maps to EXACTLY ONE
-- (from -> to) transition so the ledger replays unambiguously. Lyra sign-off:
-- VALUE-DOMAIN §3 / D2 design. Escalation adds no value (stays SupervisorMessage).
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%SeatDispatchFailed%'
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
        -- (012, G19) dispatch lifecycle
        'SeatDispatchIssued',      -- a dispatch entered the inbox
        'SeatDispatchDelivered',   -- a seat pulled it (ready -> in_flight)
        'SeatDispatchCompleted',   -- the seat marked it done (in_flight -> done)
        -- (013, G19) terminal / retry / cancel transitions
        'SeatDispatchFailed',      -- in_flight -> failed (terminal)
        'SeatDispatchRetried',     -- in_flight -> ready  (retry re-open)
        'SeatDispatchCancelled'    -- ready|blocked|in_flight -> cancelled (terminal, actor=issuer)
      ));
  END IF;
END $$;

-- =============================================================================
-- Retry budget columns — retry_count / max_retries
-- =============================================================================
-- NOT NULL DEFAULT 0, not nullable: retry_count=0 ("no attempt has failed yet")
-- and max_retries=0 ("no retry budget; first failure is terminal") are both REAL
-- recorded states, not "not recorded" (the same reasoning 012 used for payload
-- JSONB NOT NULL). max_retries is a HARD CEILING, never NULL-means-unlimited: an
-- unbounded retry on a permanently-failing task would loop forever — the stall
-- the contract forbids. Additive: existing rows backfill to 0 implicitly via the
-- default; no seeded dispatch row exists to break.
ALTER TABLE dispatches ADD COLUMN IF NOT EXISTS retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE dispatches ADD COLUMN IF NOT EXISTS max_retries INTEGER NOT NULL DEFAULT 0;

COMMENT ON COLUMN dispatches.retry_count IS
  'Failed attempts already re-opened for retry (013). 0 = no attempt has failed yet. Incremented by the in_flight -> ready SeatDispatchRetried transition; never mutated on a terminal row.';
COMMENT ON COLUMN dispatches.max_retries IS
  'Retry budget set by the issuer (013). 0 = the first failure is terminal (SeatDispatchFailed). A hard ceiling, never NULL: an unbounded retry on a permanently-failing task would loop forever, the stall the contract forbids.';

-- Reverse lookup for the blocked->ready trigger (and any future "what depends on
-- X?"): the dispatch_dependencies PK leads with dispatch_id, so a dep_kind+
-- dep_value lookup would otherwise seq-scan on every referent insert.
CREATE INDEX IF NOT EXISTS idx_dispatch_dependencies_by_referent
    ON dispatch_dependencies (dep_kind, dep_value);

-- =============================================================================
-- blocked -> ready recompute — one function, three thin triggers
-- =============================================================================
-- Readiness is authoritatively an EXISTENCE query the app runs on every pull
-- (012: no FK on dep_value). What was missing: when a blocked dispatch's
-- dependency referent lands LATER, the stored status column stayed 'blocked'
-- (the projection and the inbox view went stale) and NOTHING woke the seat.
-- These AFTER INSERT triggers close both: for the just-inserted referent, find
-- every CURRENTLY-blocked dispatch that names it, re-check ALL of that dispatch's
-- deps by existence, and — only if the set is now complete — persist blocked ->
-- ready (guarded) and pg_notify('dispatch_ready', seat) on the SAME channel
-- enqueue_dispatch already fires, so the existing seatloom-waker wakes the seat
-- with zero new machinery (no polling).
--
-- Guarded CREATE (no DROP): a re-apply creates nothing; a future body change
-- ships as its own DROP+CREATE migration.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
     WHERE p.proname = 'fn_dispatch_recompute_ready' AND n.nspname = current_schema()
  ) THEN
    CREATE FUNCTION fn_dispatch_recompute_ready() RETURNS trigger
    LANGUAGE plpgsql AS $fn$
    DECLARE
      ref_kind  TEXT;
      cand      RECORD;
      total     INT;
      satisfied INT;
    BEGIN
      -- Which dep_kind does the table this trigger fired on satisfy? Referent id
      -- spaces collide across the three tables (all TEXT), so match kind to the
      -- firing table rather than trusting dep_value alone.
      IF TG_TABLE_NAME = 'canonical_events' THEN ref_kind := 'event';
      ELSIF TG_TABLE_NAME = 'artifacts'  THEN ref_kind := 'artifact';
      ELSIF TG_TABLE_NAME = 'dispatches' THEN ref_kind := 'dispatch';
      ELSE RETURN NEW; END IF;  -- defensive: unknown table, no-op

      -- Every dispatch that is CURRENTLY blocked and names the just-inserted
      -- referent as one of its dependencies is a candidate to unblock.
      FOR cand IN
        SELECT d.id, d.target_seat_id
          FROM dispatches d
          JOIN dispatch_dependencies dd
            ON dd.dispatch_id = d.id
           AND dd.dep_kind     = ref_kind
           AND dd.dep_value    = NEW.id
         WHERE d.status = 'blocked'
      LOOP
        -- Re-evaluate ALL of this dispatch's deps by existence (the same query
        -- the app runs on pull). One referent landing may complete the set, or
        -- may not; count total deps and satisfied deps.
        SELECT count(*) AS total,
               count(*) FILTER (WHERE sat) AS satisfied
          INTO total, satisfied
          FROM (
            SELECT CASE dd2.dep_kind
                     WHEN 'event'    THEN EXISTS(SELECT 1 FROM canonical_events e WHERE e.id = dd2.dep_value)
                     WHEN 'artifact' THEN EXISTS(SELECT 1 FROM artifacts a       WHERE a.id = dd2.dep_value)
                     WHEN 'dispatch' THEN EXISTS(SELECT 1 FROM dispatches x      WHERE x.id = dd2.dep_value)
                   END AS sat
              FROM dispatch_dependencies dd2
             WHERE dd2.dispatch_id = cand.id
          ) depcheck;

        -- total > 0 guards a zero-dep row (which is 'ready' by definition and so
        -- never a blocked candidate anyway); satisfied = total means the set is
        -- now complete.
        IF total > 0 AND satisfied = total THEN
          -- Persist the flip. GUARDED: only blocked -> ready; a row that raced
          -- to in_flight/done/failed/cancelled/ready is left untouched (0 rows),
          -- so this commutes safely with the app's transitions.
          UPDATE dispatches SET status = 'ready'
           WHERE id = cand.id AND status = 'blocked';
          -- Wake the seat through the SAME channel enqueue_dispatch uses.
          PERFORM pg_notify('dispatch_ready', cand.target_seat_id);
        END IF;
      END LOOP;

      RETURN NEW;
    END;
    $fn$;
  END IF;
END $$;

-- Three triggers, each guarded so a re-apply creates nothing. AFTER INSERT only
-- (never INSERT OR UPDATE): the status-flip UPDATE inside the function must not
-- re-fire the trigger, so there is no recursion.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_dispatch_ready_on_event'
                 AND tgrelid = 'canonical_events'::regclass) THEN
    CREATE TRIGGER trg_dispatch_ready_on_event
      AFTER INSERT ON canonical_events
      FOR EACH ROW EXECUTE FUNCTION fn_dispatch_recompute_ready();
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_dispatch_ready_on_artifact'
                 AND tgrelid = 'artifacts'::regclass) THEN
    CREATE TRIGGER trg_dispatch_ready_on_artifact
      AFTER INSERT ON artifacts
      FOR EACH ROW EXECUTE FUNCTION fn_dispatch_recompute_ready();
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_dispatch_ready_on_dispatch'
                 AND tgrelid = 'dispatches'::regclass) THEN
    CREATE TRIGGER trg_dispatch_ready_on_dispatch
      AFTER INSERT ON dispatches
      FOR EACH ROW EXECUTE FUNCTION fn_dispatch_recompute_ready();
  END IF;
END $$;

COMMENT ON FUNCTION fn_dispatch_recompute_ready() IS
  'Governed-dispatch (G19, 013): on insert of an event/artifact/dispatch referent, flip any blocked dispatch whose dep set is now fully satisfied to ready (persisted) and pg_notify its seat on the dispatch_ready channel the waker already LISTENs on. Readiness stays authoritatively an existence query; this is the projection-updater + notifier for the blocked->ready direction.';

COMMIT;
