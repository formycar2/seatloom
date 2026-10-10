-- 014_acceptance_gate.sql
-- =============================================================================
-- Acceptance gate (D2) — the `acceptances` first-class object + its 3 lifecycle
-- events, so a delivery can be verified-and-accepted (or rejected) as a SYSTEM
-- capability rather than a chat promise.
--
-- Authority: docs/coordination/tasks/lyra/LYRA-2026-10-09-d2-acceptance-loop-design-v1.md
-- §2 (data model), §3 (event vocabulary — Lyra sign-off). Dispatched to Onyx by
-- disp-Rlth465QCOr9L5HK_CIu5; the DDL below is the contract, implemented verbatim.
--
-- Acceptance is a FIRST-CLASS object, not a dispatch status: its actor is the
-- user/operator (not a seat), its lifecycle differs, and it carries its own
-- payload (branch, how_to_verify, merge_commit). The dispatch row keeps its
-- "work unit" semantics (012/013 untouched); the acceptance row links to it by
-- dispatch_id. Merge criterion (operational): a delivery merges to track ONLY
-- after Flux technical verify-green + the user's acceptance of a user-perceivable
-- result.
--
-- Slot 014: next after 013. POST-SEED — creates a brand-new table the seed never
-- writes and widens two vocab CHECKs 010/012 created (cannot precede them). It
-- also REFERENCES dispatches(id) (created by 012), so canonical POST-SEED order is
-- 012 -> 013 -> 014 (apply-db.sh). NOTE: 014's event_type DROP/ADD installs the
-- full 21-value list, so it assumes 013 has already run; apply-db.sh's ordered
-- POST_SEED array guarantees that.
--
-- Additive and idempotent, 010/012 house style: every vocab change is a
-- pg_get_constraintdef-guarded DO block; every object is CREATE ... IF NOT
-- EXISTS / guarded, so a SECOND apply performs NO catalog write.
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- acceptances exists with the exact contract columns/domains:
-- SELECT column_name, data_type, is_nullable, column_default
--   FROM information_schema.columns WHERE table_name='acceptances' ORDER BY ordinal_position;
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'acceptances_status_vocab';
-- -- One-pending-per-dispatch partial unique index + the two lookup indexes:
-- SELECT indexdef FROM pg_indexes WHERE tablename='acceptances';
-- -- Vocabularies widened: event_type now 21 (incl. Acceptance*), ref_type now 11 (incl. acceptance):
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'event_object_refs_ref_type_vocab';
-- -- Idempotency: a second `psql -f 014_acceptance_gate.sql` performs NO catalog write.
-- =============================================================================

BEGIN;

-- =============================================================================
-- Vocabulary widening #1 — canonical_events_event_type_vocab: 18 -> 21
-- =============================================================================
-- The three acceptance transitions, each mapping to EXACTLY ONE (from -> to) so
-- the acceptances projection replays unambiguously (rule 2). Lyra sign-off: D2
-- design §3. 013 widened this 15 -> 18; 014 -> 21.
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
        -- (010)
        'SessionStarted', 'WorkItemCreated', 'WorkItemStatusChanged',
        'ArtifactCreated', 'HandoffSent', 'HandoffAccepted', 'HandoffCompleted',
        'ReviewVerdictIssued', 'SeatDelegationIssued', 'SeatDelegationClosed',
        'SupervisorMessage', 'SeatResponse',
        -- (012, G19) dispatch lifecycle
        'SeatDispatchIssued', 'SeatDispatchDelivered', 'SeatDispatchCompleted',
        -- (013, G19) terminal / retry / cancel
        'SeatDispatchFailed', 'SeatDispatchRetried', 'SeatDispatchCancelled',
        -- (014, D2) acceptance lifecycle
        'AcceptanceRequested',  -- -- -> pending            (actor = seat)
        'AcceptanceGranted',    -- pending -> accepted      (actor = operator; payload merge_commit)
        'AcceptanceRejected'    -- pending -> rejected      (actor = operator; payload feedback + rework_dispatch_id)
      ));
  END IF;
END $$;

-- =============================================================================
-- Vocabulary widening #2 — event_object_refs_ref_type_vocab: 10 -> 11 (+acceptance)
-- =============================================================================
-- An acceptance event must be able to NAME the acceptance it is about (subject),
-- exactly the reasoning 010 used for 'delegation' and 012 for 'dispatch'. 012
-- widened this 9 -> 10; 014 -> 11. Guard checks for the new value; none of the
-- existing values contains 'acceptance' as a substring, so the guard is sound.
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
        'delegation',  -- 010: present in committed data
        'dispatch',    -- 012 (G19): dispatches is its own object
        'acceptance'   -- 014 (D2): acceptances is its own object; an event must
                       -- be able to name the acceptance it is about.
      ));
  END IF;
END $$;

-- =============================================================================
-- acceptances — the first-class acceptance object. One row = one delivery
-- awaiting (or having received) a user accept/reject verdict.
-- =============================================================================
-- Columns are EXACTLY the contract's, in its order. Design notes recorded inline
-- and in the delivery doc:
--  * id — app-generated as `acc-<dispatch-id>` (Nimbus); not schema-enforced (a
--    format CHECK would over-constrain and the contract does not ask for it).
--  * dispatch_id — FK to dispatches(id): the lineage to the work being accepted.
--    Safe/additive: no seeded acceptance row exists to dangle it.
--  * branch / base — the delivery branch (packet/<seat>/<id>) and its base
--    ('track' by default). The merge target.
--  * how_to_verify — the one-step "do A, see B" the user performs.
--  * flux_verdict_ref — NO FK (see below): polymorphic reference to the Flux
--    verify-green verdict (a ReviewVerdictIssued event id or an artifact id).
--  * status — pending | accepted | rejected; domain CHECK below. accepted and
--    rejected are TERMINAL and immutable (same reason as 013: the acceptances row
--    is a projection of the append-only ledger; rolling a terminal row back is a
--    silent rewrite of a recorded fact).
--  * feedback / rework_dispatch_id — reject path: the operator's feedback and a
--    soft lineage ref to the auto-re-enqueued rework dispatch (NO FK — see below).
--  * merge_commit — the merge commit sha recorded on accept.
--  * decided_by / decided_at — the operator actor_ref and decision time.
--    decided_by is polymorphic (a configured operator ref, ref_type='seat' at the
--    event layer); NO FK — the operator gains a first-class identity later, not
--    invented here.
CREATE TABLE IF NOT EXISTS acceptances (
    id                 TEXT        PRIMARY KEY,               -- acc-<dispatch-id>
    dispatch_id        TEXT        NOT NULL REFERENCES dispatches(id),
    project_id         TEXT        NOT NULL REFERENCES projects(id),
    branch             TEXT        NOT NULL,                  -- packet/<seat>/<id> under acceptance
    base               TEXT        NOT NULL DEFAULT 'track',   -- merge base
    how_to_verify      TEXT        NOT NULL,                  -- one-step "do A, see B"
    flux_verdict_ref   TEXT        NOT NULL,                  -- Flux verify-green verdict ref (no FK)
    status             TEXT        NOT NULL,                  -- pending | accepted | rejected
    feedback           TEXT,                                   -- reject feedback (operator's words)
    rework_dispatch_id TEXT,                                   -- auto-re-enqueued rework dispatch (lineage, no FK)
    merge_commit       TEXT,                                   -- merge commit sha on accept
    decided_by         TEXT,                                   -- operator actor_ref (no FK)
    decided_at         TIMESTAMPTZ,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Status domain CHECK (contract: 3 values). Named + guarded, matching 012/013's
-- vocab style so the verification query can reference it by name.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'acceptances_status_vocab') THEN
    ALTER TABLE acceptances
      ADD CONSTRAINT acceptances_status_vocab
      CHECK (status IN ('pending', 'accepted', 'rejected'));
  END IF;
END $$;

-- =============================================================================
-- At most ONE pending acceptance per dispatch (partial unique index) — prevents
-- a duplicate request_acceptance for the same delivery. accepted/rejected rows
-- sit OUTSIDE the predicate, so a dispatch may be re-requested after a reject.
-- =============================================================================
CREATE UNIQUE INDEX IF NOT EXISTS uq_acceptances_one_pending_per_dispatch
    ON acceptances (dispatch_id)
    WHERE status = 'pending';

-- The two lookup indexes the contract asks for:
--  * (dispatch_id) — all acceptances for a dispatch (incl. accepted/rejected).
--  * (status)      — the pending list = WHERE status='pending' (the UI read model).
CREATE INDEX IF NOT EXISTS idx_acceptances_dispatch ON acceptances (dispatch_id);
CREATE INDEX IF NOT EXISTS idx_acceptances_status   ON acceptances (status);

-- =============================================================================
-- COMMENTs — the reasoning is part of the artifact (010/012 house style).
-- =============================================================================
COMMENT ON TABLE acceptances IS
  'Acceptance gate (D2, ONYX packet disp-Rlth465QCOr9L5HK_CIu5). One row = one delivery awaiting / having received a user accept-or-reject verdict. A FIRST-CLASS object, not a dispatch status: its actor is the user/operator, its lifecycle differs, and it carries branch/how_to_verify/merge_commit. This row is a PROJECTION of canonical_events (AcceptanceRequested/Granted/Rejected); the ledger is authoritative. accepted/rejected are terminal and immutable. Merge criterion: Flux verify-green + user acceptance before merge to track. Value-domain + event contract: docs/coordination/tasks/lyra/LYRA-2026-10-09-d2-acceptance-loop-design-v1.md §2-§3.';
COMMENT ON COLUMN acceptances.id IS
  'App-generated as acc-<dispatch-id> (Nimbus). Not schema-enforced: a format CHECK would over-constrain and the contract does not ask for one.';
COMMENT ON COLUMN acceptances.dispatch_id IS
  'FK to dispatches(id) — lineage to the work unit being accepted. Safe/additive: no seeded acceptance row exists to dangle it.';
COMMENT ON COLUMN acceptances.branch IS
  'The delivery branch under acceptance, packet/<seat>/<id>. cmd_accept_delivery merges this into base.';
COMMENT ON COLUMN acceptances.base IS
  'Merge base, default ''track''. A delivery merges to its base only after accept.';
COMMENT ON COLUMN acceptances.how_to_verify IS
  'The one-step "do A, see B" the user performs to verify a user-perceivable result before accepting.';
COMMENT ON COLUMN acceptances.flux_verdict_ref IS
  'NO FK — polymorphic reference to the Flux verify-green verdict: a ReviewVerdictIssued event id OR an artifact id. Existence is validated by the APPLICATION before accept (the first half of the merge criterion; "not verified-green may not merge"). Same no-FK-polymorphic reasoning as dispatch_dependencies.dep_value.';
COMMENT ON COLUMN acceptances.status IS
  'pending | accepted | rejected (acceptances_status_vocab). accepted/rejected are TERMINAL and immutable — the acceptances row is a ledger projection; rolling a terminal row back is a silent rewrite of a recorded fact. At most one pending per dispatch (uq_acceptances_one_pending_per_dispatch).';
COMMENT ON COLUMN acceptances.feedback IS
  'Reject path: the operator''s one-line feedback (issuer''s words), carried into the AcceptanceRejected event payload and the rework dispatch.';
COMMENT ON COLUMN acceptances.rework_dispatch_id IS
  'NO FK — soft lineage to the auto-re-enqueued rework dispatch created on reject (target = original seat, priority+10). Existence enforced by the reject flow; a FK was deliberately not added because the contract DDL omits it (reported, not deviated).';
COMMENT ON COLUMN acceptances.merge_commit IS
  'The merge commit sha recorded on accept (cmd_accept_delivery), also carried in the AcceptanceGranted event payload.';
COMMENT ON COLUMN acceptances.decided_by IS
  'The operator actor_ref who accepted/rejected. NO FK — polymorphic configured operator ref (ref_type=''seat'' at the event layer); the operator gains a first-class identity later, not invented here.';
COMMENT ON COLUMN acceptances.decided_at IS
  'When the operator accepted/rejected. NULL while pending.';

COMMIT;
