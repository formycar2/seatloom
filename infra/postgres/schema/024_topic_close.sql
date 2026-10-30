-- 024 — the topic close path: `open -> closed`, with `closed_at` written.
--
-- THE DEFECT THIS CLOSES (measured on live 2026-10-29, read-only):
--
--   select pg_get_constraintdef(oid) from pg_constraint
--    where conrelid='topics'::regclass and contype='c';
--   -> CHECK ((status = ANY (ARRAY['open','closed','merged'])))
--
--   select status, count(*) from topics group by 1;
--   -> open | 51      -- fifty-one open, zero closed, zero merged
--
--   select count(*) from topics where closed_at is not null;
--   -> 0
--
-- So the database has enforced a three-value vocabulary since 010, and nothing in
-- the program can reach two of the three values. `closed_at` is a column no
-- writer touches. That is the "the feature exists but there is no path to it"
-- shape — the operator has objected to it repeatedly (dead buttons, states the
-- product claims but cannot reach).
--
-- The user-visible cost was already measured, not theorised: the acceptance card
-- for 019 tells the operator that three of the 51 topics are `open`. With all 51
-- open the word carries no information, and a live scope cannot be told from a
-- finished one.
--
-- THE RULING (dispatch disp-TbGRGr5cvpJFgCeaayu7S). Topics ARE closeable, by the
-- seat that owns the recall scope, and only from `open`. The alternative — "topics
-- are permanent, remove the unreachable states" — was rejected because deleting a
-- lifecycle the shipped schema declares is a bigger act than giving it a path,
-- and because the states are not wrong, merely unreachable.
--
-- WHAT HAPPENS TO A CLOSED TOPIC'S MEMORIES: nothing. No query in the codebase
-- filters on topic status (measured — no `t.status =` predicate exists in any
-- repository query), so a closed scope keeps answering `get_topic_memories` and
-- the tree reads exactly as before. A recall scope is a finding aid, not an
-- access control; hiding finished work's evidence behind a status change would
-- make the record less auditable, not more.
--
-- `merged` IS NOT A TRANSITION. Merging two recall scopes has to decide what
-- happens to every memory under both, which is a product decision rather than a
-- state change. The CHECK keeps `merged` legal for the day it is designed; this
-- migration does not add a path to it, and the event vocabulary does not
-- advertise it.
--
-- NO BACKFILL. The 51 existing rows stay `open`: which of them is finished is a
-- judgement about the work, not a fact this migration can derive, and closing
-- them here would be a live-database act taken without the operator.

-- The event vocabulary: `TopicClosed` joins `TopicOpened`.
--
-- The DB CHECK on `canonical_events.event_type` is a closed set, so widening it
-- is a DROP + CREATE of the constraint, exactly as 019 did for `TopicOpened`.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab') THEN
    ALTER TABLE canonical_events
      ADD CONSTRAINT canonical_events_event_type_vocab
      CHECK (event_type IN (
        'SessionStarted', 'WorkItemCreated', 'WorkItemStatusChanged', 'ArtifactCreated',
        'HandoffSent', 'HandoffAccepted', 'HandoffCompleted', 'ReviewVerdictIssued',
        'SeatDelegationIssued', 'SeatDelegationClosed', 'SupervisorMessage', 'SeatResponse',
        'DispatchEnqueued', 'DispatchStatusChanged', 'AcceptanceRequested', 'AcceptanceDecided',
        'AcceptanceCancelled', 'MemoryCreated', 'MemoryRecalled', 'MemoryPromoted',
        'MemoryForgotten', 'MemoryResolved', 'OperatorDecisionRequested',
        'OperatorDecisionResolved', 'TopicOpened', 'TopicClosed'
      ));
  ELSE
    ALTER TABLE canonical_events
      DROP CONSTRAINT canonical_events_event_type_vocab;
    ALTER TABLE canonical_events
      ADD CONSTRAINT canonical_events_event_type_vocab
      CHECK (event_type IN (
        'SessionStarted', 'WorkItemCreated', 'WorkItemStatusChanged', 'ArtifactCreated',
        'HandoffSent', 'HandoffAccepted', 'HandoffCompleted', 'ReviewVerdictIssued',
        'SeatDelegationIssued', 'SeatDelegationClosed', 'SupervisorMessage', 'SeatResponse',
        'DispatchEnqueued', 'DispatchStatusChanged', 'AcceptanceRequested', 'AcceptanceDecided',
        'AcceptanceCancelled', 'MemoryCreated', 'MemoryRecalled', 'MemoryPromoted',
        'MemoryForgotten', 'MemoryResolved', 'OperatorDecisionRequested',
        'OperatorDecisionResolved', 'TopicOpened', 'TopicClosed'
      ));
  END IF;
END $$;

-- `closed_at` gains the company its sibling `opened_at` already has: a CHECK that
-- the timestamp is only present when the status says it should be. Without it, a
-- row could carry `closed_at` while still `open`, which is the shape of every
-- other silent inconsistency in this schema family.
--
-- Idempotent by name, so a re-run does not fail on a duplicate constraint.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'topics_closed_at_matches_status') THEN
    ALTER TABLE topics
      ADD CONSTRAINT topics_closed_at_matches_status
      CHECK (
        (status = 'open'    AND closed_at IS NULL)
        OR (status <> 'open' AND closed_at IS NOT NULL)
      );
  END IF;
END $$;

COMMENT ON COLUMN topics.closed_at IS
  'Written by close_topic (024) as NOW(), never passed in by a caller. The topics_closed_at_matches_status CHECK keeps it present exactly when the status is not open. It was a column nothing wrote from 010 until 024, which is why all 51 live rows were open.';
