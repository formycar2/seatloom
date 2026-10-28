-- 020_operator_decisions.sql
-- =============================================================================
-- OperatorDecision (2026-10-22) — the structured object for "what only the
-- operator can decide", so it stops existing ONLY as a message.
--
-- Authority: docs/coordination/tasks/aegis/AEGIS-2026-10-22-operator-decision-object-v1.md
-- (Aegis dispatch disp-dwSmYkULRU4HdSDJCDWH9, target seat-nimbus-001). Backed by
-- LYRA-2026-10-19-top-level-value-design-v1 V3 ("只有他能决定的事,要自己来找他")
-- and 张先生's 2026-10-20 authorisation for Aegis to draw the judgement boundary.
--
-- THE MEASURED GAP. Before this migration there was no pending-obligation table
-- at all, and `fleet-status.sh`'s 「等你拍板」 read ONLY `acceptances` — so a
-- product-direction ruling, or a precondition only the operator can resolve
-- ("do we add an exemption to the containment defence?", "which database does
-- the sandbox talk to?", "do we delete that file?") existed ONLY as a
-- SupervisorMessage row. The ledger had the record; nothing structured could
-- answer "how many things are waiting on the human right now, and about what".
-- V3 names that failure mode exactly: 静默遗漏 — silent omission. This table
-- makes it countable.
--
-- THIS IS NOT A MESSAGE TABLE. The judgement rule (Aegis, from 张先生's 10-20
-- authorisation): a message does not constitute a decision obligation. A blocked
-- dispatch, a failed read, a rework request — those belong to a seat or to Aegis.
-- Only what meets the four criteria goes in here (money, shipping, the outside
-- world; changing the instructions given to others; changing what the product
-- shows other people by default). Aegis owns the judgement of WHICH item that
-- is; this migration only supplies the object. That is why there is no seed.
--
-- Slot 020: next after 019 (021 is the parallel same-seat history-corpus
-- migration on its own branch — serialized, never co-edited). POST-SEED — it
-- creates a brand-new table the seed never writes to and widens two vocab
-- CHECKs that 010/012 created (which exist only post-seed), so it cannot
-- precede its prerequisites. Canonical order 018 -> 019 -> 020.
--
-- Additive and idempotent, 013's guarded pattern: both widenings DROP and
-- re-ADD the COMPLETE lists, guarded on the new value being absent, so a SECOND
-- apply performs NO catalog write (verified in the delivery doc). The full
-- lists — never deltas — are what let 020 apply cleanly from the track state
-- (28/12, post-019) or any intermediate, and what keeps 015/017/018/019's values
-- from being silently reverted. Cross-checked value-by-value against the pinned
-- Rust EventType::ALL [EventType; 30] / RefType::ALL [RefType; 13] on this
-- branch.
--
-- NO SEED DATA. A pending "等你决定" that no one created is contamination, not
-- a feature — an operator would be shown an obligation nobody owes him. The
-- table starts empty by construction: nothing inserts a row here except the
-- request path, which requires an operator-or-Aegis actor.
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- operator_decisions exists with the exact contract columns/domains:
-- SELECT column_name, data_type, is_nullable, column_default
--   FROM information_schema.columns WHERE table_name='operator_decisions' ORDER BY ordinal_position;
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'operator_decisions_status_vocab';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'operator_decisions_choices_array';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'operator_decisions_decider_shape';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'operator_decisions_creator_shape';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'operator_decisions_resolved_pairing';
-- SELECT indexdef FROM pg_indexes WHERE tablename='operator_decisions';
-- -- Vocabularies widened: event_type now 30 (incl. OperatorDecision*), ref_type now 13 (incl. decision):
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'event_object_refs_ref_type_vocab';
-- -- Zero rows, and the count question the table exists to answer:
-- SELECT count(*) FROM operator_decisions;
-- SELECT count(*) FROM operator_decisions WHERE status='pending';
-- -- Negative: a blank choice is refused by the TRIGGER (a CHECK cannot scan
-- -- elements). An empty array is refused by the CHECK. A seat other than Aegis
-- -- is refused by operator_decisions_creator_shape.
-- INSERT INTO operator_decisions (id, project_id, target, choices, consequence, status, created_by)
--   VALUES ('dec-blank', 'seatloom', 't', '["a", "  "]'::jsonb, 'c', 'pending', 'seat:seat-aegis-001');
-- -- Idempotency: a second `psql -f 020_operator_decisions.sql` performs NO catalog write.
-- =============================================================================

BEGIN;

-- =============================================================================
-- Vocabulary widening #1 — canonical_events_event_type_vocab: 28 -> 30
-- (+OperatorDecisionRequested, +OperatorDecisionResolved)
-- =============================================================================
-- Two transitions, each mapping to EXACTLY ONE (from -> to) so the
-- operator_decisions projection replays unambiguously (rule 2): the obligation
-- goes IN as a request, the ruling comes back as a resolve. There is no third
-- transition and no edit: a resolved decision is immutable, exactly like a
-- terminal acceptance (014) or a dispatched dispatch (013).
--
-- Why NOT reuse an existing type (the 019 reasoning, restated because it is the
-- same temptation): in an append-only ledger one word for two meanings can never
-- be split once rows carry both. `SeatDispatchIssued` would record a human
-- obligation as a work unit; `SupervisorMessage` as a message; `AcceptanceGranted`
-- as a merge. Each of those has its own actor and its own lifecycle, and this
-- object's differ from all three.
--
-- Guard checks for 'OperatorDecisionRequested'; no pre-020 value contains it as
-- a substring, so the guard is sound.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%OperatorDecisionRequested%'
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
        -- (020, operator decisions) the human-obligation object's two transitions
        'OperatorDecisionRequested',  -- (none) -> pending; actor = the requester
        'OperatorDecisionResolved',   -- pending -> resolved; payload carries chosen + ruling_by
        -- (019, D5 Phase 2 follow-on) topic creation
        'TopicOpened'           -- (none) -> open; actor = the opening seat (server-resolved)
      ));
  END IF;
END $$;

COMMENT ON COLUMN canonical_events.event_type IS
  'canonical_events_event_type_vocab (010, widened by 012/013/014/017/015/018/020/019). 30 values; each names exactly one transition so the projections replay unambiguously. OperatorDecisionRequested/Resolved (020): a decision only the operator can rule on went IN as a request, and the ruling came back — row + event in ONE transaction, refs subject = ref_type ''decision'' (020), actor = the requester on the way in / the operator on the way out. Contract: docs/coordination/tasks/aegis/AEGIS-2026-10-22-operator-decision-object-v1.md.';

-- =============================================================================
-- Vocabulary widening #2 — event_object_refs_ref_type_vocab: 12 -> 13 (+decision)
-- =============================================================================
-- An operator-decision event must be able to NAME the obligation it is about
-- (subject), exactly the reasoning 010 used for 'delegation' and 012 for
-- 'dispatch'. Guard checks for the new value; none of the existing values
-- contains 'decision' as a substring, so the guard is sound.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'event_object_refs_ref_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%decision%'
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
        'acceptance',  -- 014 (D2): acceptances is its own object
        'memory',      -- 015 (D5): memories is its own object
        'decision'     -- 020: operator_decisions is its own object; an event must
                       -- be able to name the obligation it is about.
      ));
  END IF;
END $$;

-- =============================================================================
-- operator_decisions — the first-class "waiting on the human" object.
-- One row = one decision only the operator can rule on, pending or resolved.
-- =============================================================================
-- Columns are the value design §5 six, in the packet's order, plus the lineage
-- the rest of the schema uses (project_id, created_by, created_at). Design notes:
--
--  * id — app-generated as `dec-<slug>` (Nimbus). Not schema-enforced: a format
--    CHECK would over-constrain and the contract does not ask for one.
--  * target — the CONCRETE thing to decide, not a paragraph. This is the whole
--    point of the object: a message can be a paragraph; an obligation has to
--    name what is being decided.
--  * choices — JSONB array of strings, non-empty (CHECK below). ONE option is
--    not a choice; an empty array is not a decision. The array is deliberately
--    NOT a CHECK-enumerated domain: the legal options are domain knowledge the
--    requester states, and freezing them in the schema would mean a migration
--    for every new question.
--  * decider — who is entitled to rule. DEFAULT is the configured operator ref,
--    which is why the packet says "默认是操作者,但不默认成所有事都推给他":
--    Aegis may create a row with decider='seat:seat-aegis-001' for a ruling it
--    owns, and then it does NOT appear in the operator's pending count. The
--    CHECK forbids a decider that is neither an operator ref nor a seat ref —
--    a fabricated decider would let anything be waved through.
--  * consequence — what happens once it is decided. REQUIRED (enforced in the
--    write path, and the schema refuses a blank): a choice offered without its
--    consequence is the operator clicking in the dark.
--  * evidence — JSONB array of references (dispatch id, verdict ref, ruling
--    ref). The support chain, so the operator can check the claim rather than
--    trust the summary.
--  * status — pending | resolved. resolved is TERMINAL and immutable, same
--    reasoning as 013/014: the row is a projection of the append-only ledger,
--    and rolling a resolved row back is a silent rewrite of a recorded fact.
--  * chosen / resolved_by / resolved_at — the ruling. The pairing CHECK enforces
--    all three IFF status='resolved' (both directions, 018's pattern): a
--    resolved row with no chosen value, or a pending row carrying one, is a lie
--    about the ledger.
CREATE TABLE IF NOT EXISTS operator_decisions (
    id           TEXT        PRIMARY KEY,               -- dec-<slug>
    project_id   TEXT        NOT NULL REFERENCES projects(id),
    target       TEXT        NOT NULL,                  -- the concrete thing to decide
    choices      JSONB       NOT NULL,                  -- non-empty array of strings
    decider      TEXT        NOT NULL DEFAULT 'human:supervisor',
    consequence  TEXT        NOT NULL,                  -- what happens once decided
    evidence     JSONB       NOT NULL DEFAULT '[]'::jsonb,
    status       TEXT        NOT NULL,                  -- pending | resolved
    created_by   TEXT        NOT NULL,                  -- operator or seat-aegis-001 (the only creators)
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    chosen       TEXT,                                  -- the ruling (required IFF resolved)
    resolved_by  TEXT,                                  -- who ruled (required IFF resolved)
    resolved_at  TIMESTAMPTZ                             -- when (required IFF resolved)
);

-- Status domain CHECK (contract: 2 values). Named + guarded, matching
-- 012/013's vocab style so the verification query can reference it by name.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'operator_decisions_status_vocab') THEN
    ALTER TABLE operator_decisions
      ADD CONSTRAINT operator_decisions_status_vocab
      CHECK (status IN ('pending', 'resolved'));
  END IF;
END $$;

-- A decider is either the operator or a seat. Nothing else. A fabricated
-- decider ('system', 'auto', 'default') would be an obligation nobody owes —
-- and, worse, a row nobody will ever see waiting.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'operator_decisions_decider_shape') THEN
    ALTER TABLE operator_decisions
      ADD CONSTRAINT operator_decisions_decider_shape
      CHECK (decider LIKE 'human:%' OR decider LIKE 'seat:%');
  END IF;
END $$;

-- The creator is equally constrained. The packet's third rule: only the
-- operator and Aegis may create one of these — a seat must not be able to
-- manufacture a "waiting on the operator" to push its own work.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'operator_decisions_creator_shape') THEN
    ALTER TABLE operator_decisions
      ADD CONSTRAINT operator_decisions_creator_shape
      CHECK (created_by LIKE 'human:%' OR created_by = 'seat:seat-aegis-001');
  END IF;
END $$;

-- =============================================================================
-- The two shape rules that make this an obligation rather than a note.
-- =============================================================================
-- (a) choices is a NON-EMPTY ARRAY of NON-BLANK strings. An empty array is not a
--     choice, and neither is a blank one: "decide between '' and ''" is a
--     request to guess.
--
--     WHY THIS IS TWO MECHANISMS AND NOT ONE CHECK. The first draft put the
--     blank-element rule inside the CHECK with
--     `NOT EXISTS (SELECT 1 FROM jsonb_array_elements_text(choices) ...)`, and
--     PostgreSQL refuses it outright: `cannot use subquery in check constraint`.
--     That is not a formality — a CHECK must be evaluable from the row alone, so
--     the rule holds on every write path including a direct INSERT. A rule that
--     needs to look at the ARRAY'S ELEMENTS belongs in a trigger, which can.
--     Split rather than dropped: the CHECK keeps what a single-row expression
--     can express (it is an array, and it is not empty), and the trigger carries
--     the blank-element half with the SAME message text, so a caller sees one
--     refusal for one rule instead of two shapes of the same complaint.
-- (b) resolved IFF (chosen, resolved_by, resolved_at) all set. 018's
--     boolean-equality pairing pattern, applied to three columns: a resolved row
--     with no ruling is a fabricated verdict, and a pending row carrying one
--     pre-records a decision that has not happened.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'operator_decisions_choices_array') THEN
    ALTER TABLE operator_decisions
      ADD CONSTRAINT operator_decisions_choices_array
      CHECK (
        jsonb_typeof(choices) = 'array'
        AND jsonb_array_length(choices) > 0
      );
  END IF;
END $$;

-- (a') the blank-element half, as a trigger. Idempotent by
--      CREATE OR REPLACE + a guarded DROP/CREATE pair.
CREATE OR REPLACE FUNCTION operator_decisions_choices_are_nonblank()
RETURNS TRIGGER AS $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM jsonb_array_elements_text(NEW.choices) AS c
     WHERE btrim(c) = ''
  ) THEN
    RAISE EXCEPTION 'operator_decisions_choices_nonblank: decision % has a blank choice — "decide between '''' and ''''" is a request to guess',
      NEW.id
      USING ERRCODE = 'check_violation';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_operator_decisions_choices_nonblank ON operator_decisions;
CREATE TRIGGER trg_operator_decisions_choices_nonblank
  BEFORE INSERT OR UPDATE OF choices ON operator_decisions
  FOR EACH ROW
  EXECUTE FUNCTION operator_decisions_choices_are_nonblank();

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'operator_decisions_resolved_pairing') THEN
    ALTER TABLE operator_decisions
      ADD CONSTRAINT operator_decisions_resolved_pairing
      CHECK (
        (status = 'resolved') =
        (chosen IS NOT NULL AND resolved_by IS NOT NULL AND resolved_at IS NOT NULL)
      );
  END IF;
END $$;

-- =============================================================================
-- Indexes
-- =============================================================================
-- The one the object exists for: "how many things are pending, and for whom".
-- Scoped to status, and to the decider — the operator's pending count must not
-- include a row whose decider is a seat.
CREATE INDEX IF NOT EXISTS idx_operator_decisions_status_decider
    ON operator_decisions (status, decider);

-- The per-project read the UI runs (every other table in the schema is
-- project-scoped; this one is too).
CREATE INDEX IF NOT EXISTS idx_operator_decisions_project
    ON operator_decisions (project_id, status);

-- =============================================================================
-- COMMENTs — the reasoning is part of the artifact (010/012 house style).
-- =============================================================================
COMMENT ON TABLE operator_decisions IS
  'OperatorDecision (020, packet AEGIS-2026-10-22-operator-decision-object-v1). One row = one decision only the operator can rule on, pending or resolved. NOT a message table: a message does not constitute a decision obligation (Aegis judgement rule, 张先生 2026-10-20 authorisation) — blocked dispatches, failed reads and rework belong to a seat or to Aegis. This row is a PROJECTION of canonical_events (OperatorDecisionRequested/Resolved); the ledger is authoritative. resolved is TERMINAL and immutable. NO SEED DATA: a pending "等你决定" nobody created is contamination, not a feature. Contract: docs/coordination/tasks/aegis/AEGIS-2026-10-22-operator-decision-object-v1.md.';
COMMENT ON COLUMN operator_decisions.id IS
  'App-generated as dec-<slug> (Nimbus). Not schema-enforced: a format CHECK would over-constrain and the contract does not ask for one.';
COMMENT ON COLUMN operator_decisions.target IS
  'The CONCRETE thing to decide, not a paragraph. This is the whole point of the object: a message can be a paragraph; an obligation must name what is being decided.';
COMMENT ON COLUMN operator_decisions.choices IS
  'Non-empty JSONB array of non-blank strings: the array half is operator_decisions_choices_array (a CHECK can only see the row), the non-blank half is trg_operator_decisions_choices_nonblank (a trigger can scan the elements). One option is not a choice; an empty array is not a decision. Deliberately NOT a CHECK-enumerated domain — the legal options are domain knowledge the requester states, and freezing them in the schema would mean a migration for every new question.';
COMMENT ON COLUMN operator_decisions.decider IS
  'Who is entitled to rule. DEFAULT ''human:supervisor'' — the configured operator. Aegis may set a seat decider for a ruling IT owns, and then the row does NOT appear in the operator''s pending count (operator_decisions_decider_shape admits only human:* and seat:*).';
COMMENT ON COLUMN operator_decisions.consequence IS
  'What happens once it is decided. REQUIRED — a choice offered without its consequence is the operator clicking in the dark, which is worse than not asking.';
COMMENT ON COLUMN operator_decisions.evidence IS
  'JSONB array of references (dispatch id, verdict ref, ruling ref) — the support chain, so the operator can check the claim instead of trusting the summary. Default empty: not every obligation has evidence, but one that claims some must name it.';
COMMENT ON COLUMN operator_decisions.status IS
  'pending | resolved (operator_decisions_status_vocab). resolved is TERMINAL and immutable — the row is a ledger projection; rolling a resolved row back is a silent rewrite of a recorded fact. operator_decisions_resolved_pairing enforces chosen/resolved_by/resolved_at IFF resolved, both directions.';
COMMENT ON COLUMN operator_decisions.created_by IS
  'The requester: the operator, or seat:seat-aegis-001 (operator_decisions_creator_shape admits nothing else). Only these two may create — a seat must not be able to manufacture a "waiting on the operator" to push its own work.';
COMMENT ON COLUMN operator_decisions.chosen IS
  'The ruling — which of `choices` was taken. Required IFF status=''resolved'' (operator_decisions_resolved_pairing).';
COMMENT ON COLUMN operator_decisions.resolved_by IS
  'Who ruled. Required IFF status=''resolved''. Normally the operator; Aegis resolves the rows it created for itself.';
COMMENT ON COLUMN operator_decisions.resolved_at IS
  'When the ruling landed. Required IFF status=''resolved''. NULL while pending.';

COMMIT;
