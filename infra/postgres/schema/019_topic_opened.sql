-- 019_topic_opened.sql
-- =============================================================================
-- TopicOpened (D5 Phase 2 follow-on) — the schema half of the governed topic
-- creation path. Nimbus delivered the `open_topic` MCP tool
-- (packet/nimbus/open-topic @ 29df563, Flux-verified; `EventType::ALL` pinned
-- at 28 on the Rust side) per Aegis ruling 1
-- (docs/coordination/tasks/aegis/AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md):
-- production had NO path to create a topic, while `memories.topic_id` is NOT
-- NULL FK -> topics(id) and topics held 0 rows — every `remember` (and every
-- seat's forward-recording obligation) failed on the foreign key. This
-- migration is the remaining half: until 019 is landed AND applied to live,
-- `open_topic` refuses at the database CHECK with the vocabulary violation
-- (escalated, not worked around).
--
-- Authority: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-11-topic-opened-019-v1.md
-- (Lyra sign-off 2026-10-11) — the signed addendum ratifying Nimbus's ADD
-- decision (no honest reuse candidate: WorkItemCreated would record a topic as
-- a work item, ArtifactCreated as an artifact, SupervisorMessage as an
-- escalation; in an append-only ledger one word for two meanings can never be
-- split once rows carry both — the same reasoning that kept ArtifactProduced
-- out and gave dispatch (012), acceptance (014) and memory (015/018) their own
-- types). Dispatched to Onyx by disp-019-onyx.
--
-- Contract, verbatim: EVENT VOCABULARY +1 TopicOpened — "a recall scope was
-- opened deliberately. Exactly one transition per event (row + event in ONE
-- transaction; refs subject = ref_type 'topic', actor = the seat that opened
-- it, opened_by resolved server-side from SEATLOOM_ACTOR_REF, never a
-- parameter). Idempotent re-open emits NO event. Full PG list after 019 = 28
-- values = the 27 live values + TopicOpened." ref_type 'topic' already exists
-- (010) — NO ref_type widening. No new columns, no new tables, no daemon.
--
-- Slot 019: next after 018 (015-018 landed on track via eceb264; 006/007 stay
-- reserved). POST-SEED — it widens ONE vocab CHECK that 010/012/013/014/017/
-- 015/018 created (those CHECKs exist only post-seed; the seed writes no
-- canonical_events rows). Canonical order 018 -> 019 (apply-db.sh).
--
-- Additive and idempotent, 013's guarded pattern: the widening DROPs and
-- re-ADDs the COMPLETE 28-value list, guarded on 'TopicOpened' being absent,
-- so a SECOND apply performs NO catalog write (verified in the delivery doc).
-- The full list — never a delta — is what lets 019 apply cleanly from the
-- track state (27, post-018) or any intermediate, and what keeps 015/018's
-- values from being silently reverted (a first draft of this file in Nimbus's
-- scratch carried a stale list and would have reverted them; caught there, and
-- the list below is cross-checked value-by-value against the pinned Rust
-- EventType::ALL on packet/nimbus/open-topic before commit).
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- 28 values, incl. 'TopicOpened' (and every pre-019 value preserved):
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- -- Positive: a TopicOpened event lands (previously CHECK-refused):
-- INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, project_id, schema_version, payload)
--   VALUES ('ev-019-demo', 'TopicOpened', NOW(), 'seat:seat-onyx-001', 'seatloom', 1, '{}');
-- -- Negative: an unknown event type is still refused.
-- -- Idempotency: a second `psql -f 019_topic_opened.sql` performs NO catalog write.
-- =============================================================================

BEGIN;

-- =============================================================================
-- Vocabulary widening — canonical_events_event_type_vocab: 27 -> 28 (+TopicOpened)
-- =============================================================================
-- A topic is a first-class object with its own lifecycle (open|closed|merged)
-- and its own actor (topics.opened_by, NOT NULL — the schema itself demands a
-- deliberate opener), so its creation is its own transition — the same shape
-- 012 used for dispatches and 014 for acceptances. ref_type 'topic' already
-- exists (010); no ref_type widening is needed. The opening seat is the actor
-- (opened_by resolved server-side, never a parameter); an idempotent re-open
-- emits NO event (tool-layer ON CONFLICT DO NOTHING semantics).
--
-- The list below is the POST-018 list PLUS TopicOpened: every value the
-- previous migrations installed — 010's 12, 012's 3, 013's 3, 014's 3, the 017
-- withdrawal, 015's 4, 018's 1 — carried forward verbatim, cross-checked
-- against the pinned Rust EventType::ALL ([EventType; 28]) on
-- packet/nimbus/open-topic (set AND order).
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
        'AcceptanceRequested',  -- -- -> pending            (actor = seat)
        'AcceptanceGranted',    -- pending -> accepted      (actor = operator)
        'AcceptanceRejected',   -- pending -> rejected      (actor = operator)
        -- (017, D2 follow-on) acceptance withdrawal
        'AcceptanceCancelled',  -- pending -> cancelled     (actor = requesting seat)
        -- (015, D5) memory lifecycle
        'MemoryCreated',        -- (none) -> active/cold; actor = the remembering seat
        'MemoryRecalled',       -- active: heat+1, recall_count+1, last_recalled_at=now
        'MemoryPromoted',       -- tier change (up OR down, explicit only)
        'MemoryForgotten',      -- active -> forgotten (soft tombstone, reason required)
        -- (018, D5 Phase 2) memory closure
        'MemoryResolved',       -- active -> resolved; payload carries resolved_by/resolved_at
        -- (019, D5 Phase 2 follow-on) topic creation
        'TopicOpened'           -- (none) -> open; actor = the opening seat (server-resolved)
      ));
  END IF;
END $$;

COMMENT ON COLUMN canonical_events.event_type IS
  'canonical_events_event_type_vocab (010, widened by 012/013/014/017/015/018/019). 28 values; each names exactly one transition so the projections replay unambiguously. TopicOpened (019): a recall scope was opened deliberately — row + event in ONE transaction, refs subject = ref_type ''topic'' (exists since 010), actor = the opening seat (opened_by resolved server-side from SEATLOOM_ACTOR_REF, never a parameter); an idempotent re-open emits NO event. Contract: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-11-topic-opened-019-v1.md.';

COMMIT;
