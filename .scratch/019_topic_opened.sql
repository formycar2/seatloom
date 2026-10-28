-- 019_topic_opened.sql — SCRATCH COPY for the Nimbus open-topic lane (NOT FOR COMMIT).
-- Source: Aegis ruling docs/coordination/tasks/aegis/AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md
-- (ruling 1). ONYX OWNS THE REAL FILE — this copy exists so the Rust half can be
-- built and smoke-tested against a scratch DB, exactly as .scratch/013..018 did.
--
-- SLOT: 019, next after 018 (015/016/017/018 are taken: D5 knowledge layer,
-- acceptance_plan, the D2 withdrawal, memory hardening). POST-SEED, additive and
-- idempotent: it widens ONE vocab CHECK that 010/012/013/014/015/018 created.
--
-- WHY A NEW EVENT TYPE AND NOT A REUSE (the ruling asked Nimbus to decide and
-- to write the cost of both options down):
--   * Reuse: there is no honest candidate. `WorkItemCreated` would record a
--     topic as a work item; `ArtifactCreated` as an artifact; `SupervisorMessage`
--     as an escalation. In an append-only ledger one word for two meanings can
--     never be split once rows carry both — the same reasoning that kept
--     `ArtifactProduced` out and that gave `dispatch` (012), `acceptance` (014)
--     and `memory` (015/018) their own types.
--   * Add: one migration, one Lyra value-domain sign-off, and the hand-written
--     FromStr/Display the codebase already uses for every TEXT enum. The cost is
--     a slot and a review; the cost of the alternative is a permanently
--     ambiguous ledger.
--   Chosen: ADD `TopicOpened`. Until this migration lands, `open_topic` refuses
--   at the database CHECK with the vocabulary violation — escalated, not worked
--   around.
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- -----------------------------------------------------------------------------
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
--   -- 28 values, incl. 'TopicOpened'
-- -- Negative: INSERT ... event_type='TopicOpened' is ACCEPTED; an unknown type still refused.
-- -- Idempotency: a second `psql -f 019_topic_opened.sql` performs NO catalog write.
-- =============================================================================

BEGIN;

-- =============================================================================
-- Vocabulary widening — canonical_events_event_type_vocab: 27 -> 28 (+TopicOpened)
-- =============================================================================
-- A topic is a first-class object with its own lifecycle (open|closed|merged)
-- and its own actor (`opened_by`), so its creation is its own transition — the
-- same shape 012 used for dispatches and 014 for acceptances. ref_type 'topic'
-- already exists (010); no ref_type widening is needed.
--
-- The list below is the POST-018 list: a guarded widening DROPs and re-ADDs the
-- whole CHECK, so it must carry every value the previous migrations installed —
-- 010's 12, 012's 3, 013's 3, 014's 3, the 017 withdrawal, 015's 4 and 018's 1.
-- (A first draft of this file carried a stale list and would have silently
-- reverted 015/018; caught by the scratch proof.)
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%TopicOpened%'
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
        'AcceptanceRequested', 'AcceptanceGranted', 'AcceptanceRejected',
        -- (017, D2 follow-on) acceptance withdrawal
        'AcceptanceCancelled',
        -- (015, D5) memory lifecycle
        'MemoryCreated', 'MemoryRecalled', 'MemoryPromoted', 'MemoryForgotten',
        -- (018, D5 Phase 2) memory closure
        'MemoryResolved',
        -- (019, topic creation) a recall scope is opened
        'TopicOpened'          -- (none) -> open; actor = the opening seat
      ));
  END IF;
END $$;

COMMENT ON COLUMN canonical_events.event_type IS
  'canonical_events_event_type_vocab (010, widened by 012/013/014/017/015/018/019). 28 values; each names exactly one transition so the projections replay unambiguously.';

COMMIT;
