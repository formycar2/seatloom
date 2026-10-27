-- 018_memory_hardening.sql
-- =============================================================================
-- Memory-base hardening (D5 Phase 2, Lane A1) — closes the five measured gaps
-- in 015's memories table (each verified at ZERO occurrences in the shipped
-- table), so the washing pipeline (S3 LLM extraction) pours value into a
-- container that can hold it. HARD PREREQUISITE: no S3 extraction may start
-- before 018 lands — filling the current container and re-washing after the
-- fact would pay for the tokens twice.
--
-- Authority: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md
-- (Lyra sign-off 2026-10-10), carried verbatim by both Phase-2 lane packets;
-- dispatched to Onyx by disp-mem018-onyx. Design parent:
-- AEGIS-2026-10-10-washing-pipeline-design-v2.md (operator-approved; scope
-- tightened to the seatloom project only — 16 sessions / 168MB / 58,015 lines).
--
-- The five gaps (measured against 015's memories):
--   G1 no kind          — six record types had nowhere to live but an
--                         unconstrained tags[]; "every failure in this project"
--                         was unaskable.
--   G2 no supersede/relations — "this decision was later overturned" was
--                         inexpressible; a stale decision served as current is
--                         worse than no memory (today's instance: base='track'
--                         was once true, then wrong).
--   G3 unresolved had no closing state — the most executable record class could
--                         never be closed.
--   G4 no confidence    — an inferred decision looked exactly like a quoted one.
--   G5 no L2 line range — "the evidence is somewhere in this 134MB file" is NOT
--                         evidence; the convergence point of all loss risk and
--                         the load-bearing wall (pruning may shrink the LLM
--                         input, NEVER the L2 pointer).
--
-- Slot 018: next after 017 (006/007 stay reserved). POST-SEED — it ALTERs and
-- constrains tables the seed never writes, widens a CHECK 015 created, and
-- widens the event vocabulary; memories itself exists only after 015
-- (POST-SEED), so 018 cannot precede its prerequisite (this branch stacks on
-- the 015 branch; apply-db.sh canonical order 015 -> 018). The event_type
-- widening writes the COMPLETE 27-value list (013's guarded pattern), so 018
-- applies cleanly from the track state (21 values), the post-017 state (26), or
-- any intermediate — order-independent vs 017 by construction.
--
-- Additive and idempotent, 010/012/013 house style: every object is
-- CREATE ... IF NOT EXISTS / catalog-guarded DO block / ADD COLUMN IF NOT
-- EXISTS, so a SECOND apply performs NO catalog write (verified in the
-- delivery doc). Zero data rows are inserted or rewritten: the additive
-- DEFAULTs backfill existing rows (kind='fact', confidence='inferred') and no
-- row is touched again (the live apply is Aegis's act).
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- New columns on memories:
-- SELECT column_name, data_type, is_nullable, column_default
--   FROM information_schema.columns WHERE table_name='memories'
--    AND column_name IN ('kind','supersedes','confidence','resolved_by','resolved_at','l2_line_start','l2_line_end');
-- -- The three new CHECKs + the widened status vocab:
-- SELECT conname, pg_get_constraintdef(oid) FROM pg_constraint
--  WHERE conname IN ('memories_kind_vocab','memories_resolved_pairing','memories_confidence_vocab',
--                    'memories_l2_lines_required','memories_status_vocab');
-- -- memory_relations: PK + rel vocab:
-- SELECT conname, pg_get_constraintdef(oid) FROM pg_constraint WHERE conrelid = 'memory_relations'::regclass;
-- -- Event vocabulary now 27 (incl. MemoryResolved; AcceptanceCancelled preserved):
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- -- Negative cases (all must be REJECTED): kind out of vocab; status='resolved'
-- -- without resolved_by/resolved_at (and status='active' WITH them);
-- -- l2_ref_kind='file' AND source_kind='archived_session' without line range;
-- -- l2_line_end < l2_line_start; rel out of vocab.
-- -- Idempotency: a second `psql -f 018_memory_hardening.sql` performs NO catalog
-- -- write (all guards skip; IF NOT EXISTS no-op).
-- =============================================================================

BEGIN;

-- =============================================================================
-- G1 KIND — the record type (6 values). Additive DEFAULT 'fact' is a BACKFILL
-- ONLY: the remember tool REQUIRES kind explicitly (typed rejection when
-- absent) — the default is never a write path (contract §G1).
-- =============================================================================
ALTER TABLE memories ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'fact';

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'memories_kind_vocab') THEN
    ALTER TABLE memories
      ADD CONSTRAINT memories_kind_vocab
      CHECK (kind IN ('decision', 'failure', 'fact', 'preference', 'artifact', 'unresolved'));
  END IF;
END $$;

-- =============================================================================
-- G2 SUPERSESSION — memories.supersedes (documents.supersedes style:
-- create-time only, immutable after). Same edge as
-- memory_relations.rel='supersedes' below — two spellings of ONE edge; the
-- write path keeps them consistent in ONE transaction (contract §G2).
-- =============================================================================
ALTER TABLE memories ADD COLUMN IF NOT EXISTS supersedes TEXT NULL REFERENCES memories(id);

-- =============================================================================
-- G2 RELATIONS — the typed edge table (contract §G2 verbatim: columns, PK,
-- rel vocab).
-- =============================================================================
-- from_id/to_id both reference memories(id): an edge between two memories of
-- the same project+topic scope (enforced by the write path, not invented here
-- as a cross-column CHECK the contract does not specify). PK
-- (from_id, to_id, rel) makes the edge set duplicate-free and idempotent to
-- re-assert.
CREATE TABLE IF NOT EXISTS memory_relations (
    from_id TEXT        NOT NULL REFERENCES memories(id),
    to_id   TEXT        NOT NULL REFERENCES memories(id),
    rel     TEXT        NOT NULL,
    PRIMARY KEY (from_id, to_id, rel),
    CONSTRAINT memory_relations_rel_vocab
      CHECK (rel IN ('supersedes', 'caused_by', 'fixed_by', 'refines'))
);

-- =============================================================================
-- G3 RESOLVED — the closing state for 'unresolved' records. Closure is an
-- EXPLICIT tool act (resolve), NEVER a background sweep (contract §G3; D2
-- ruling: no daemon, no automatic anything). forgotten soft-tombstone
-- semantics UNCHANGED.
-- =============================================================================
ALTER TABLE memories ADD COLUMN IF NOT EXISTS resolved_by TEXT NULL;
ALTER TABLE memories ADD COLUMN IF NOT EXISTS resolved_at TIMESTAMPTZ NULL;

-- Status domain widened 2 -> 3 (+resolved). Guard checks the new value; a
-- re-apply finds it present and does nothing. Named + guarded, 012/013 style.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'memories_status_vocab'
       AND pg_get_constraintdef(oid) LIKE '%resolved%'
  ) THEN
    ALTER TABLE memories
      DROP CONSTRAINT IF EXISTS memories_status_vocab;
    ALTER TABLE memories
      ADD CONSTRAINT memories_status_vocab
      CHECK (status IN ('active', 'forgotten', 'resolved'));
  END IF;
END $$;

-- The pairing: resolved IFF (resolved_by AND resolved_at) — a boolean equality,
-- so BOTH directions are enforced: 'resolved' without the who/when is rejected,
-- and an active/forgotten row carrying a resolver is rejected (a half-open
-- state in either direction).
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'memories_resolved_pairing') THEN
    ALTER TABLE memories
      ADD CONSTRAINT memories_resolved_pairing
      CHECK ((status = 'resolved') = (resolved_by IS NOT NULL AND resolved_at IS NOT NULL));
  END IF;
END $$;

-- =============================================================================
-- G4 CONFIDENCE — BINARY, never a score (LLM self-scored confidence is not
-- trustworthy). 'quoted' = the raw text says it; 'inferred' = derived.
-- Additive DEFAULT 'inferred' backfills existing rows (they predate the
-- distinction); no row is rewritten (contract §G4).
-- =============================================================================
ALTER TABLE memories ADD COLUMN IF NOT EXISTS confidence TEXT NOT NULL DEFAULT 'inferred';

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'memories_confidence_vocab') THEN
    ALTER TABLE memories
      ADD CONSTRAINT memories_confidence_vocab
      CHECK (confidence IN ('quoted', 'inferred'));
  END IF;
END $$;

-- =============================================================================
-- G5 L2 LINE RANGE — the load-bearing wall. When the L2 pointer is a repo file
-- distilled from an archived_session, the line range is REQUIRED (and must be
-- well-formed): "the evidence is somewhere in this 134MB file" is not evidence.
-- Pruning (pipeline S2.5) may shrink what the LLM sees; it must NEVER shrink
-- the L2 pointer (contract §G5).
-- =============================================================================
ALTER TABLE memories ADD COLUMN IF NOT EXISTS l2_line_start INTEGER NULL;
ALTER TABLE memories ADD COLUMN IF NOT EXISTS l2_line_end   INTEGER NULL;

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'memories_l2_lines_required') THEN
    ALTER TABLE memories
      ADD CONSTRAINT memories_l2_lines_required
      CHECK (NOT (l2_ref_kind = 'file' AND source_kind = 'archived_session')
             OR (l2_line_start IS NOT NULL AND l2_line_end IS NOT NULL
                 AND l2_line_end >= l2_line_start));
  END IF;
END $$;

-- =============================================================================
-- EVENT VOCABULARY +1 — MemoryResolved: active -> resolved (the G3 closure
-- transition; exactly one transition, replayable). The COMPLETE 27-value list
-- (013's guarded pattern): the 26 values Nimbus's Rust pins today (22
-- acceptance/dispatch + 4 memory + AcceptanceCancelled) + MemoryResolved.
-- Writing the full list — not a delta — is what makes 018 order-independent vs
-- 017: from the track state (21), the post-017 state (26), or any intermediate,
-- the guard fires and installs the same 27. AcceptanceCancelled survives
-- whichever order the two migrations land in.
-- =============================================================================
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%MemoryResolved%'
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
        'MemoryResolved'        -- active -> resolved; payload carries resolved_by/resolved_at
      ));
  END IF;
END $$;

-- =============================================================================
-- Indexes along the recall path (packet §3.8 — seat's choice):
--   * (project_id, topic_id, kind, status) — recall's filter shape: scope,
--     record type, then status (recall DEFAULTs to excluding superseded rows,
--     which the partial supersedes index below serves).
--   * memory_relations (to_id, rel) — the reverse lookup: "what supersedes /
--     caused / fixed / refines THIS memory".
--   * memories (supersedes) WHERE NOT NULL — partial: most rows have no
--     predecessor; serves both the exclusion filter and the forward chain.
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_memories_project_topic_kind_status
    ON memories (project_id, topic_id, kind, status);
CREATE INDEX IF NOT EXISTS idx_memory_relations_to ON memory_relations (to_id, rel);
CREATE INDEX IF NOT EXISTS idx_memories_supersedes ON memories (supersedes)
    WHERE supersedes IS NOT NULL;

-- =============================================================================
-- COMMENTs — the reasoning is part of the artifact (010/012/015 density).
-- =============================================================================
COMMENT ON TABLE memory_relations IS
  'Typed edges between memories (D5 Phase 2, contract §G2). rel vocab: supersedes | caused_by | fixed_by | refines (memory_relations_rel_vocab). supersedes and memory_relations.rel=''supersedes'' are TWO SPELLINGS OF THE SAME EDGE — the write path keeps them consistent in ONE transaction. PK (from_id, to_id, rel) makes the edge set duplicate-free and idempotent to re-assert. Recall DEFAULT excludes superseded rows: a stale decision served as current is worse than no memory.';
COMMENT ON COLUMN memory_relations.from_id IS
  'The source memory (FK memories(id)) — e.g. the newer decision for rel=''supersedes'', the cause for ''caused_by'', the fix for ''fixed_by'', the refinement for ''refines''.';
COMMENT ON COLUMN memory_relations.to_id IS
  'The target memory (FK memories(id)) — e.g. the overturned decision for rel=''supersedes''. Reverse lookups ("what supersedes THIS memory") use idx_memory_relations_to.';
COMMENT ON COLUMN memory_relations.rel IS
  'supersedes | caused_by | fixed_by | refines (memory_relations_rel_vocab). The four edge kinds the contract defines; a fifth is a follow-up contract widening, not a silent addition.';

COMMENT ON COLUMN memories.kind IS
  'Record type (D5 Phase 2, contract §G1): decision | failure | fact | preference | artifact | unresolved (memories_kind_vocab). Without it the six record types had nowhere to live but an unconstrained tags[] — "every failure in this project" was unaskable. The additive DEFAULT ''fact'' is a BACKFILL ONLY: the remember tool REQUIRES kind explicitly (typed rejection when absent) — the default is never a write path.';
COMMENT ON COLUMN memories.supersedes IS
  'The memory this one supersedes (self-FK, documents.supersedes style; create-time only, immutable after). Same edge as memory_relations.rel=''supersedes'' — two spellings, ONE edge, kept consistent by the write path in one transaction. Recall DEFAULT excludes superseded rows (a stale decision served as current is worse than no memory).';
COMMENT ON COLUMN memories.confidence IS
  'quoted | inferred (memories_confidence_vocab) — BINARY, never a score (LLM self-scored confidence is not trustworthy). quoted = the raw text says it; inferred = derived. Additive DEFAULT ''inferred'' backfills pre-018 rows (they predate the distinction); no row is rewritten.';
COMMENT ON COLUMN memories.status IS
  'active | forgotten | resolved (memories_status_vocab). forgotten = soft tombstone (reason required, rows never hard-deleted). resolved = the G3 closing state for ''unresolved'' records: an EXPLICIT tool act (resolve), NEVER a background sweep — closure pairs with resolved_by/resolved_at (memories_resolved_pairing enforces both directions). D2 ruling stands: no daemon, no automatic anything.';
COMMENT ON COLUMN memories.resolved_by IS
  'Who closed the unresolved record (set by the resolve tool, carried in the MemoryResolved event payload). memories_resolved_pairing: status=''resolved'' IFF resolved_by IS NOT NULL AND resolved_at IS NOT NULL — a half-open state is rejected in EITHER direction.';
COMMENT ON COLUMN memories.resolved_at IS
  'When the unresolved record was closed (resolve tool). NULL unless status=''resolved'' (memories_resolved_pairing).';
COMMENT ON COLUMN memories.l2_line_start IS
  'First line of the L2 evidence inside l2_file_path (1-based). REQUIRED when l2_ref_kind=''file'' AND source_kind=''archived_session'' (memories_l2_lines_required): "the evidence is somewhere in this 134MB file" is NOT evidence. Pruning may shrink the LLM input; it must NEVER shrink the L2 pointer.';
COMMENT ON COLUMN memories.l2_line_end IS
  'Last line of the L2 evidence (>= l2_line_start, memories_l2_lines_required). The load-bearing wall — the convergence point of all loss risk.';

COMMIT;
