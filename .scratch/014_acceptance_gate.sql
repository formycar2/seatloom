-- 014_acceptance_gate.sql — SCRATCH COPY for the Nimbus D2 lane (NOT FOR COMMIT).
-- Source: docs/coordination/tasks/lyra/LYRA-2026-10-09-d2-acceptance-loop-design-v1.md §2/§3
-- (Lyra value-domain sign-off B/C, 2026-10-09), written in the 012 catalog-guarded
-- house style. Onyx owns the real migration; this copy exists only to build and
-- smoke-test the Rust half against a scratch DB before Onyx's file reaches track.

BEGIN;

-- Event vocabulary: +3 acceptance transitions (each exactly one transition).
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%AcceptanceRequested%'
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
        'SeatDispatchFailed', 'SeatDispatchRetried', 'SeatDispatchCancelled',
        'AcceptanceRequested',   -- -- -> pending           (actor = seat)
        'AcceptanceGranted',     -- pending -> accepted      (actor = operator)
        'AcceptanceRejected'     -- pending -> rejected      (actor = operator)
      ));
  END IF;
END $$;

-- ref_type: +1 ('acceptance') — the subject type of acceptance events.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'event_object_refs_ref_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%acceptance%'
  ) THEN
    ALTER TABLE event_object_refs
      DROP CONSTRAINT IF EXISTS event_object_refs_ref_type_vocab;
    ALTER TABLE event_object_refs
      ADD CONSTRAINT event_object_refs_ref_type_vocab
      CHECK (ref_type IN (
        'workitem', 'session', 'handoff', 'artifact',
        'document', 'seat', 'topic', 'project',
        'delegation', 'dispatch',
        'acceptance'   -- 014 (D2): acceptances is its own object; an event must
                       -- be able to name the acceptance it is about.
      ));
  END IF;
END $$;

-- acceptances — the D2 acceptance gate. Its own first-class object: the actor is
-- the operator/issuer (not the seat), the lifecycle differs from a dispatch, and
-- it carries its own payload (branch, how-to-verify, merge commit).
CREATE TABLE IF NOT EXISTS acceptances (
    id                 TEXT PRIMARY KEY,               -- acc-<dispatch-id>
    dispatch_id        TEXT NOT NULL REFERENCES dispatches(id),
    project_id         TEXT NOT NULL REFERENCES projects(id),
    branch             TEXT NOT NULL,                  -- packet/<seat>/<id>
    base               TEXT NOT NULL DEFAULT 'track',
    how_to_verify      TEXT NOT NULL,                  -- "do A, see B"
    flux_verdict_ref   TEXT NOT NULL,                  -- ReviewVerdictIssued event/artifact id
    status             TEXT NOT NULL,                  -- pending | accepted | rejected
    feedback           TEXT,                           -- operator's reject words
    rework_dispatch_id TEXT,                           -- auto-enqueued rework (lineage)
    merge_commit       TEXT,                           -- accept's merge commit
    decided_by         TEXT,                           -- operator actor_ref
    decided_at         TIMESTAMPTZ,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Status domain: accepted/rejected terminal and immutable (the row is a
-- projection of the append-only ledger; reverting a terminal row would be a
-- silent rewrite of a recorded fact — the same reasoning as 013).
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'acceptances_status_vocab') THEN
    ALTER TABLE acceptances
      ADD CONSTRAINT acceptances_status_vocab
      CHECK (status IN ('pending', 'accepted', 'rejected'));
  END IF;
END $$;

-- At most one PENDING acceptance per dispatch (no duplicate requests).
CREATE UNIQUE INDEX IF NOT EXISTS uq_acceptances_one_pending_per_dispatch
    ON acceptances (dispatch_id)
    WHERE status = 'pending';

-- Pending list read model (the UI's 待验收区) and per-dispatch lineage lookup.
CREATE INDEX IF NOT EXISTS idx_acceptances_status ON acceptances (status, created_at);
CREATE INDEX IF NOT EXISTS idx_acceptances_dispatch ON acceptances (dispatch_id);

COMMIT;
