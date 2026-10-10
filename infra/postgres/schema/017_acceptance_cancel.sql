-- 017_acceptance_cancel.sql
-- =============================================================================
-- Acceptance cancel path — the schema mirror of Nimbus's cancel_acceptance tool
-- (D2 gate follow-on, Aegis dispatch disp-E8BP2vKlOwMHLomm0eX8J). The tool
-- shipped in the binary, but the migration that supports it existed NOWHERE:
-- no branch or migration carried 'cancelled' in acceptances_status_vocab or
-- 'AcceptanceCancelled' in canonical_events_event_type_vocab (Nimbus drafted
-- the DDL in a .scratch copy explicitly NOT FOR COMMIT). Consequence: Aegis's
-- void of the old empty pending card acc-disp-me6Nm8e9D6ZRe4fKyVokY — and any
-- seat withdrawing its own pending request — was DB-CHECK-blocked. 017 closes
-- that surfaced gap. Raised by Lyra to Aegis; dispatched to Onyx by
-- disp-acclose-onyx (acceptance-cards closeout).
--
-- Authority (value domain, verbatim-pinned both sides):
--   * Rust mirror — packet/nimbus/d5-mcp-recall crates/seatloom-core/src/db/
--     event_domain.rs: EventType::ALL: [EventType; 26] (cross-checked against
--     the closeout packet's verbatim 26-value list — identical set AND order)
--     and RefType::ALL: [RefType; 12].
--   * Cancel semantics — docs/coordination/tasks/nimbus/
--     NIMBUS-2026-10-10-d2-acceptance-base-and-cancel-v1 §2: the REQUESTING
--     SEAT withdraws its OWN pending request: pending -> cancelled (terminal)
--     + AcceptanceCancelled in ONE transaction, mandatory reason. No merge, no
--     rework — the card closes as a recorded void, NOT a rejection ("this card
--     was satisfied by other means, do not merge it" is not "this delivery is
--     not good enough, redo it" — an append-only ledger cannot split one word
--     carrying both meanings later).
--
-- Slot 017: next after 016 (015/016 are the parallel same-seat lanes, delivered
-- on their own branches and applied to live out-of-band; 006/007 stay
-- reserved). POST-SEED — it widens two vocab CHECKs that exist only post-seed
-- and adds no column the seed writes. Canonical order 014 -> ... -> 017
-- (apply-db.sh).
--
-- Additive and idempotent, 013/015 house style: guarded DO blocks writing the
-- COMPLETE lists, so a SECOND apply performs NO catalog write (verified in the
-- delivery doc). The full lists are supersets of BOTH the track state (status
-- 3 values / event_type 21) and the live state (3 / 25), so 017 applies
-- cleanly from either.
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- Status domain now 4 values (incl. cancelled); event vocabulary now 26 (incl. AcceptanceCancelled):
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'acceptances_status_vocab';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- -- Positive: a cancelled acceptance row and an AcceptanceCancelled event both land
-- -- (both were CHECK-rejected before 017).
-- -- Idempotency: a second `psql -f 017_acceptance_cancel.sql` performs NO catalog
-- -- write (both DO-blocks guard-skip).
-- =============================================================================

BEGIN;

-- =============================================================================
-- Vocabulary widening #1 — acceptances_status_vocab: 3 -> 4 (+cancelled)
-- =============================================================================
-- cancelled = the REQUESTING SEAT withdrew its own pending request (terminal,
-- immutable — same discipline as accepted/rejected: the acceptances row is a
-- projection of the append-only ledger). Its actor is the requesting seat,
-- NOT the operator: accepted/rejected are the operator's verdict on the
-- delivery, cancelled is the seat's withdrawal of the request itself. The
-- one-pending partial unique index (uq_acceptances_one_pending_per_dispatch)
-- is scoped to status='pending', so cancelled sits OUTSIDE it and a dispatch
-- may be re-requested after a cancel. Guard checks for the new value, so a
-- re-apply finds it present and does nothing.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'acceptances_status_vocab'
       AND pg_get_constraintdef(oid) LIKE '%cancelled%'
  ) THEN
    ALTER TABLE acceptances
      DROP CONSTRAINT IF EXISTS acceptances_status_vocab;
    ALTER TABLE acceptances
      ADD CONSTRAINT acceptances_status_vocab
      CHECK (status IN ('pending', 'accepted', 'rejected', 'cancelled'));
  END IF;
END $$;

-- =============================================================================
-- Vocabulary widening #2 — canonical_events_event_type_vocab: -> 26 (+AcceptanceCancelled)
-- =============================================================================
-- The 26-value COMPLETE list (013's guarded pattern): the live 21 values
-- (010/012/013/014) + the 4 memory lifecycle values (015) + AcceptanceCancelled.
-- Writing the full list — not a delta — is what lets this migration apply
-- cleanly from either the track state (21 values) or the live state (25), and
-- keeps ONE authoritative enumeration that Nimbus's Rust EventType::ALL pins
-- verbatim (packet/nimbus/d5-mcp-recall, cross-checked set AND order against
-- the closeout packet's list). Existing rows can never violate the wider list.
--
-- AcceptanceCancelled maps to EXACTLY ONE transition (pending -> cancelled),
-- so the acceptances projection replays unambiguously (rule 2).
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%AcceptanceCancelled%'
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
        -- (017, D2 follow-on) acceptance withdrawal — the requesting seat's
        -- own exit, distinct from the operator's verdict
        'AcceptanceCancelled',  -- pending -> cancelled     (actor = requesting seat; reason mandatory)
        -- (015, D5) memory lifecycle
        'MemoryCreated', 'MemoryRecalled', 'MemoryPromoted', 'MemoryForgotten'
      ));
  END IF;
END $$;

-- =============================================================================
-- COMMENTs — the reasoning is part of the artifact (010/012 house style).
-- The status comment is refreshed to carry the fourth value; the feedback
-- comment is refreshed because cancel_acceptance records its mandatory reason
-- there (the cancel path writes feedback + decided_by/decided_at, same columns
-- the reject path uses — the event type is what distinguishes the two).
-- =============================================================================
COMMENT ON COLUMN acceptances.status IS
  'pending | accepted | rejected | cancelled (acceptances_status_vocab). accepted/rejected are the OPERATOR''s verdict (terminal, immutable); cancelled is the REQUESTING SEAT withdrawing its own pending request (terminal, immutable) — "this card was satisfied by other means, do not merge it", recorded as a void, NOT a rejection. Same projection discipline: the acceptances row is a projection of the append-only ledger; rolling a terminal row back is a silent rewrite of a recorded fact. At most one pending per dispatch (uq_acceptances_one_pending_per_dispatch); cancelled rows sit outside it, so a dispatch may be re-requested after a cancel.';
COMMENT ON COLUMN acceptances.feedback IS
  'Reject path: the operator''s one-line feedback. Cancel path (017): the requesting seat''s MANDATORY reason for withdrawing its own pending request. Same column, different actor — the AcceptanceRejected vs AcceptanceCancelled event type is what distinguishes the two; both also set decided_by/decided_at.';

COMMIT;
