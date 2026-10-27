-- 017_acceptance_cancel.sql — SCRATCH COPY for the Nimbus D2 lane (NOT FOR COMMIT).
-- Source: Aegis governed dispatch disp-E8BP2vKlOwMHLomm0eX8J (defect 2, "the
-- ledger cannot be corrected through the product"), implemented on the Nimbus
-- side by that same dispatch. ONYX OWNS THE REAL FILE — this copy exists so the
-- Rust half can be built and smoke-tested against a scratch DB, exactly as
-- .scratch/013_dispatch_states.sql and .scratch/014_acceptance_gate.sql did.
--
-- WHY A NEW TRANSITION AND NOT "reject WITHOUT rework"
-- -----------------------------------------------------
-- The dispatch offered two shapes. The one NOT taken: let `reject` skip the
-- rework enqueue. That needs no DDL, but it records a void as a REJECTION — in
-- an append-only ledger, one word for two meanings can never be split once rows
-- carry both (the same reasoning that kept `ArtifactProduced` out of the event
-- vocabulary: two words for one meaning are equally unrecoverable). "The card
-- was satisfied by other means, do not merge it" is not "this delivery is not
-- good enough, redo it".
--
-- The shape taken mirrors what 013 already signed for dispatches:
-- `SeatDispatchCancelled` (issuer-side withdrawal, terminal). The acceptance
-- analogue is `AcceptanceCancelled` (the requesting SEAT pulls back its own
-- pending request, terminal). Same object, same terminal-immutability rule,
-- same one-event-per-transition rule (contract rule 2).
--
-- SLOT: 017 (was drafted as 016 — RENAMED when Onyx's 016 landed first).
-- 015 is Onyx's D5 knowledge-layer schema (LYRA-2026-10-10-d5-knowledge-layer-
-- milestone, Lane A); 016 is Onyx's acceptance_plan migration
-- (disp-accplan-lane-c-onyx, commit a6f2ac8, whose gate event
-- ev-016-acceptance-plan-landed is already in the ledger). This withdrawal
-- migration therefore takes the next free slot, 017, and must be applied AFTER
-- 016 (it only widens vocabularies, so the order is immaterial in practice, but
-- the numbering records the real sequence). Registering it in
-- scripts/apply-db.sh is Onyx's call together with that slot.
--
-- ORDERING: POST-SEED. It widens two vocab CHECKs that 010/012/013/014 created
-- and adds no column the seed writes. It REQUIRES 014 (the `acceptances` table
-- and `acceptances_status_vocab`); applied before 014 it fails loudly, which is
-- the intended signal.
--
-- Additive and idempotent, 010/012/013/014 house style: every vocab change is a
-- pg_get_constraintdef-guarded DO block, so a SECOND apply performs NO catalog
-- write. The only other statement is a COMMENT (documentation, not data).
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- status domain is now 4 values, cancelled included:
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'acceptances_status_vocab';
-- -- event vocabulary is now 22 values, AcceptanceCancelled included:
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- -- a cancelled row round-trips through the write path (see the Rust tests):
-- -- INSERT ... status='cancelled' ; UPDATE ... status='pending' must REFUSE (nothing rewrites a terminal row)
-- -- Idempotency: a second `psql -f 017_acceptance_cancel.sql` performs NO catalog write.
-- =============================================================================

BEGIN;

-- =============================================================================
-- Vocabulary widening #1 — acceptances_status_vocab: 3 -> 4 (+cancelled)
-- =============================================================================
-- `cancelled` is TERMINAL, like accepted/rejected: the row is a projection of
-- the append-only ledger, and rolling a terminal row back would be a silent
-- rewrite of a recorded fact (the 013/014 reasoning, unchanged). It sits
-- OUTSIDE the uq_acceptances_one_pending_per_dispatch predicate, so a dispatch
-- whose card was cancelled may be requested again — the same freedom a rejected
-- dispatch has.
--
-- The guard checks for the NEW value in the live constraint definition, so the
-- block is a no-op on a database that already carries it (and on a fresh 014
-- install it simply installs the 4-value domain).
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
-- Vocabulary widening #2 — canonical_events_event_type_vocab: +AcceptanceCancelled
-- =============================================================================
-- The fourth acceptance transition: pending -> cancelled, actor = the
-- requesting seat (NOT the operator — a void is the seat withdrawing its own
-- hand-over, mirroring 013's issuer-side SeatDispatchCancelled). Payload:
-- {dispatch_id, reason}. Exactly one transition, so the acceptances projection
-- replays unambiguously.
--
-- 014 installed the full 21-value list, so this block re-installs the full list
-- WITH the new value. THE LIST BELOW IS THE POST-015 LIST: it must include
-- 015's four memory types, because a guarded widening DROPs and re-ADDs the
-- whole CHECK — writing a stale list here would silently REVERT 015 (exactly
-- the bug this file had in its first draft, caught by the scratch-DB proof:
-- applying 015 then this file left the memory types out of the vocabulary).
-- It therefore assumes 014 AND 015 have already run (apply-db.sh's ordered
-- POST_SEED array guarantees that, and 017 is appended after 014/015/016).
DO $$
BEGIN
  -- The guard checks for the NEW value in the live definition: present -> the
  -- widening already happened, so the block performs no catalog write. (It does
  -- NOT need to check for the memory types — those are part of the list this
  -- block installs, and 015's own guard put them there.)
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
        'AcceptanceRequested',  -- -- -> pending   (actor = seat)
        'AcceptanceGranted',    -- pending -> accepted (actor = operator)
        'AcceptanceRejected',   -- pending -> rejected (actor = operator)
        -- (015, D5) memory lifecycle — carried verbatim so this widening
        -- re-installs them rather than dropping them
        'MemoryCreated', 'MemoryRecalled', 'MemoryPromoted', 'MemoryForgotten',
        -- (017, D2 follow-on) the withdrawal
        'AcceptanceCancelled'   -- pending -> cancelled (actor = the requesting seat)
      ));
  END IF;
END $$;

-- =============================================================================
-- COMMENT refresh — `feedback` now also carries the cancel reason.
-- =============================================================================
-- No column is added and no row is touched: the reason of a void lives in the
-- same free-text column as a reject's feedback, so the 待验收 read model can
-- render it without a second query. The application enforces "a cancel must
-- carry a reason" (the write path refuses an empty one); it is deliberately NOT
-- a CHECK constraint, keeping this migration to the two vocabulary widenings
-- that need Lyra's sign-off.
COMMENT ON COLUMN acceptances.feedback IS
  'Operator''s free-text reason on a reject, or the requesting seat''s reason on a cancel (017). NULL while pending.';

COMMENT ON COLUMN acceptances.status IS
  'pending | accepted | rejected | cancelled (acceptances_status_vocab). accepted/rejected/cancelled are TERMINAL and immutable — the acceptances row is a ledger projection; rolling a terminal row back is a silent rewrite of a recorded fact. At most one pending per dispatch (uq_acceptances_one_pending_per_dispatch).';

COMMIT;
