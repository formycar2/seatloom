-- 013_dispatch_states.sql — SCRATCH COPY for the Nimbus D2 lane (NOT FOR COMMIT).
-- Source: docs/coordination/tasks/onyx/ONYX-2026-10-09-dispatch-states-design.md §4
-- (verbatim SQL), Lyra value-domain sign-off A (2026-10-09). Onyx owns the real
-- migration; this copy exists only to build/smoke-test the Rust half against a
-- scratch DB before Onyx's file reaches track.

BEGIN;

-- Widen dispatches_status_vocab: 4 -> 6 (+failed, +cancelled).
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

-- Widen canonical_events_event_type_vocab: 15 -> 18 (+3 dispatch lifecycle).
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
        'SessionStarted', 'WorkItemCreated', 'WorkItemStatusChanged',
        'ArtifactCreated', 'HandoffSent', 'HandoffAccepted', 'HandoffCompleted',
        'ReviewVerdictIssued', 'SeatDelegationIssued', 'SeatDelegationClosed',
        'SupervisorMessage', 'SeatResponse',
        'SeatDispatchIssued', 'SeatDispatchDelivered', 'SeatDispatchCompleted',
        'SeatDispatchFailed',      -- in_flight -> failed (terminal)
        'SeatDispatchRetried',     -- in_flight -> ready  (retry re-open)
        'SeatDispatchCancelled'    -- ready|blocked|in_flight -> cancelled (terminal)
      ));
  END IF;
END $$;

-- Retry budget, per dispatch.
ALTER TABLE dispatches ADD COLUMN IF NOT EXISTS retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE dispatches ADD COLUMN IF NOT EXISTS max_retries INTEGER NOT NULL DEFAULT 0;

COMMENT ON COLUMN dispatches.retry_count IS
  'Failed attempts already re-opened for retry (013). 0 = no attempt has failed yet. Incremented by the in_flight -> ready SeatDispatchRetried transition; never mutated on a terminal row.';
COMMENT ON COLUMN dispatches.max_retries IS
  'Retry budget set by the issuer (013). 0 = the first failure is terminal (SeatDispatchFailed). A hard ceiling, never NULL: an unbounded retry on a permanently-failing task would loop forever, the stall the contract forbids.';

-- Reverse lookup for the blocked->ready trigger.
CREATE INDEX IF NOT EXISTS idx_dispatch_dependencies_by_referent
    ON dispatch_dependencies (dep_kind, dep_value);

-- blocked -> ready recompute + three referent triggers.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
     WHERE p.proname = 'fn_dispatch_recompute_ready' AND n.nspname = current_schema()
  ) THEN
    CREATE FUNCTION fn_dispatch_recompute_ready() RETURNS trigger
    LANGUAGE plpgsql AS $fn$
    DECLARE
      ref_kind TEXT;
      cand      RECORD;
      total     INT;
      satisfied INT;
    BEGIN
      IF TG_TABLE_NAME = 'canonical_events' THEN ref_kind := 'event';
      ELSIF TG_TABLE_NAME = 'artifacts'  THEN ref_kind := 'artifact';
      ELSIF TG_TABLE_NAME = 'dispatches' THEN ref_kind := 'dispatch';
      ELSE RETURN NEW; END IF;

      FOR cand IN
        SELECT d.id, d.target_seat_id
          FROM dispatches d
          JOIN dispatch_dependencies dd
            ON dd.dispatch_id = d.id
           AND dd.dep_kind     = ref_kind
           AND dd.dep_value    = NEW.id
         WHERE d.status = 'blocked'
      LOOP
        -- Shipped form per the design's own note: existence booleans computed
        -- directly over dd2 via LATERAL (the inline subquery form was marked
        -- illustrative in the design doc).
        SELECT count(*), count(*) FILTER (WHERE p.present)
          INTO total, satisfied
          FROM dispatch_dependencies dd2,
               LATERAL (SELECT CASE
                          WHEN dd2.dep_kind = 'event'    THEN EXISTS(SELECT 1 FROM canonical_events e WHERE e.id = dd2.dep_value)
                          WHEN dd2.dep_kind = 'artifact' THEN EXISTS(SELECT 1 FROM artifacts a       WHERE a.id = dd2.dep_value)
                          WHEN dd2.dep_kind = 'dispatch' THEN EXISTS(SELECT 1 FROM dispatches x      WHERE x.id = dd2.dep_value)
                        END AS present) p
         WHERE dd2.dispatch_id = cand.id;

        IF total > 0 AND satisfied = total THEN
          UPDATE dispatches SET status = 'ready'
           WHERE id = cand.id AND status = 'blocked';
          PERFORM pg_notify('dispatch_ready', cand.target_seat_id);
        END IF;
      END LOOP;

      RETURN NEW;
    END;
    $fn$;
  END IF;
END $$;

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

COMMIT;
