-- 015_knowledge_layer.sql
-- =============================================================================
-- Knowledge layer (D5, Lane A) — SeatLoom extends from "manage work" to "manage
-- knowledge": the project tree gains a parent field, memories get their base
-- table (L0/L1/L2 progressive disclosure + cold->hot lifecycle), and the
-- archived_sessions free-text project slug gets an executable normalization
-- rule into the tree. Vocabulary widens +4 event types / +1 ref type.
--
-- Authority: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md
-- (Lyra sign-off 2026-10-10) §1-§6, carried verbatim by both D5 lane packets;
-- dispatched to Onyx by disp-d5-lane-a-onyx. The scope framework is UNCHANGED
-- (project + topic is the ONLY scoping framework); agent harness + LLM stay
-- execution entities, never a scope of their own.
--
-- Slot 015: next after 014 (006/007 stay reserved). POST-SEED — it adds a
-- column, creates a brand-new table, creates a view, and widens two vocab
-- CHECKs; the seed writes none of these, and the widened CHECKs exist only
-- post-seed. memories REFERENCES topics(id) (created by 010, POST-SEED) and
-- documents(id) (002), so 015 cannot precede its prerequisites. Canonical
-- order 010 -> 015 (apply-db.sh). 016 (acceptance-plan lane, same seat,
-- serialized) lands on its own branch and does not touch the vocabularies.
--
-- Additive and idempotent, 010/012/013 house style: every object is
-- CREATE ... IF NOT EXISTS / catalog-guarded DO block, so a SECOND apply
-- performs NO catalog write (verified in the delivery doc).
--
-- -----------------------------------------------------------------------------
-- VERIFICATION QUERY (ONYX.md §5.5). Run after apply on a SCRATCH database.
-- Every assertion is checked in the delivery doc.
-- -----------------------------------------------------------------------------
-- -- Vocabularies widened: event_type now 25 (incl. Memory*), ref_type now 12 (incl. memory):
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'event_object_refs_ref_type_vocab';
-- -- Project tree column: nullable, self-FK, not-self CHECK, partial index:
-- SELECT column_name, data_type, is_nullable FROM information_schema.columns
--  WHERE table_name='projects' AND column_name='parent_project_id';
-- SELECT conname, pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'projects_parent_not_self';
-- SELECT indexdef FROM pg_indexes WHERE indexname = 'idx_projects_parent';
-- -- memories shape (contract §4 verbatim) + the three domain CHECKs:
-- SELECT column_name, data_type, is_nullable, column_default
--   FROM information_schema.columns WHERE table_name='memories' ORDER BY ordinal_position;
-- SELECT conname, pg_get_constraintdef(oid) FROM pg_constraint
--  WHERE conname IN ('memories_l2_pairing','memories_tier_vocab','memories_status_vocab');
-- -- Normalization view (contract §2 R1-R5; ZERO row inserts):
-- SELECT map_status, count(*) FROM v_archived_sessions_normalized GROUP BY 1;
-- SELECT raw_slug, normalized_parent, normalized_leaf, map_status
--   FROM v_archived_sessions_normalized WHERE raw_slug IN
--    ('seatloom','sf-code','sf-code/storage','fde/field_evaluation','(root)');
-- -- Idempotency: a second `psql -f 015_knowledge_layer.sql` performs NO catalog
-- -- write (all guards skip; IF NOT EXISTS no-op).
-- =============================================================================

BEGIN;

-- =============================================================================
-- 1. PROJECT TREE — projects.parent_project_id (contract §1)
-- =============================================================================
-- NULL = root project. Self-FK expresses hierarchy without inventing a second
-- framework; history already shows it (sf-code/storage, ai-assistant/sf-package,
-- fde/field_evaluation). CHECK (parent_project_id IS DISTINCT FROM id) keeps a
-- project from being its own parent (a 1-cycle; deeper cycles are the washing
-- pipeline's concern, Phase 2 / Aegis lane). Additive-safe: existing rows
-- backfill to NULL = root, and a NULL check expression passes the CHECK.
ALTER TABLE projects ADD COLUMN IF NOT EXISTS parent_project_id TEXT NULL REFERENCES projects(id);

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'projects_parent_not_self') THEN
    ALTER TABLE projects
      ADD CONSTRAINT projects_parent_not_self
      CHECK (parent_project_id IS DISTINCT FROM id);
  END IF;
END $$;

-- Partial index: only non-root projects have a parent to look up by.
CREATE INDEX IF NOT EXISTS idx_projects_parent ON projects (parent_project_id)
    WHERE parent_project_id IS NOT NULL;

COMMENT ON COLUMN projects.parent_project_id IS
  'Project tree parent (D5, contract §1). NULL = root project. Self-FK to projects(id): the tree expresses the hierarchy history already shows (sf-code/storage, ai-assistant/sf-package, fde/field_evaluation) without a second scoping framework. projects_parent_not_self blocks a 1-cycle (self-parent); deeper cycles are the Phase-2 washing pipeline''s concern (Aegis lane). Partial index idx_projects_parent covers the non-root lookups.';

-- =============================================================================
-- 2. MEMORIES — the small-granularity distilled-knowledge base (contract §4)
-- =============================================================================
-- Columns are EXACTLY the contract's, in its order; the l2 pairing CHECK sits
-- where the contract puts it. Design notes recorded inline and in the delivery
-- doc:
--  * project_id + topic_id — BOTH required: project+topic is the only scoping
--    framework (every knowledge object carries both ids).
--  * L0/L1/L2 — progressive disclosure, NOT three stored copies: l0_summary is
--    the ~100-token default layer, l1_overview the ~2K-token on-demand layer,
--    l2_* the pointer to the full product (document row or repo file + digest).
--    The token budget belongs to the caller.
--  * l2 pairing CHECK — exactly one of (document id | file path) must be present
--    per l2_ref_kind; the same CHECK also pins the 2-value kind domain.
--  * heat / recall_count / last_recalled_at — the recall audit surface: +1 per
--    recall, NO automatic decay, ever (contract §4 TIER note). Tier transitions
--    happen ONLY via the explicit promote tool (up or down); no background
--    daemon, no auto-promotion.
--  * status active|forgotten — forget is a SOFT tombstone (reason required);
--    rows are never hard-deleted (ledger discipline). A content correction is
--    forget + remember, not an in-place edit (no revision machinery).
--  * Write path: rows are written ONLY through the MCP memory tools, each
--    appending its event in the SAME transaction (012 dispatch pattern). No
--    second write path, no direct SQL by seats.
CREATE TABLE IF NOT EXISTS memories (
    id                 TEXT        PRIMARY KEY,
    project_id         TEXT        NOT NULL REFERENCES projects(id),
    topic_id           TEXT        NOT NULL REFERENCES topics(id),
    title              TEXT        NOT NULL,
    tags               TEXT[]      NOT NULL DEFAULT '{}',
    l0_summary         TEXT        NOT NULL,          -- ~100 tokens; the ONLY default layer
    l1_overview        TEXT,                           -- ~2K tokens; fetched on demand
    l2_ref_kind        TEXT        NOT NULL,          -- document | file
    l2_document_id     TEXT        NULL REFERENCES documents(id),
    l2_file_path       TEXT        NULL,              -- repo-relative, documents.file_path style
    l2_body_digest     TEXT        NULL,              -- sha256 hex, documents.body_digest style
    CONSTRAINT memories_l2_pairing CHECK (
        (l2_ref_kind = 'document' AND l2_document_id IS NOT NULL)
        OR (l2_ref_kind = 'file' AND l2_file_path IS NOT NULL)
    ),
    source_kind        TEXT        NOT NULL,          -- archived_session | canonical_event | document | manual
    source_ref         TEXT        NOT NULL,          -- '<harness>:<id>' | event id | doc id | actor note (NO FK)
    heat               INTEGER     NOT NULL DEFAULT 0,    -- +1 per recall; NO automatic decay, ever
    recall_count       INTEGER     NOT NULL DEFAULT 0,
    last_recalled_at   TIMESTAMPTZ NULL,              -- NULL = never recalled
    tier               TEXT        NOT NULL DEFAULT 'cold',  -- cold | warm | hot
    tier_changed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status             TEXT        NOT NULL DEFAULT 'active', -- active | forgotten
    forgotten_reason   TEXT        NULL,              -- required when status='forgotten'
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tier domain (contract §4): cold | warm | hot. Transitions ONLY via the
-- promote tool (explicit target tier, up or down). Named + guarded, matching
-- 012/013's vocab style so the verification query can reference it by name.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'memories_tier_vocab') THEN
    ALTER TABLE memories
      ADD CONSTRAINT memories_tier_vocab
      CHECK (tier IN ('cold', 'warm', 'hot'));
  END IF;
END $$;

-- Status domain (contract §4): active | forgotten. forget is a SOFT tombstone.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'memories_status_vocab') THEN
    ALTER TABLE memories
      ADD CONSTRAINT memories_status_vocab
      CHECK (status IN ('active', 'forgotten'));
  END IF;
END $$;

-- The two scope-first lookup indexes the contract's read pattern needs
-- (recall filters by project+topic, then status or tier).
CREATE INDEX IF NOT EXISTS idx_memories_project_topic_status ON memories (project_id, topic_id, status);
CREATE INDEX IF NOT EXISTS idx_memories_project_topic_tier   ON memories (project_id, topic_id, tier);

-- Full-text search over title + l0_summary (PG FTS, 002's idx_documents_fts
-- pattern) — recall's query is a text filter, NOT semantic retrieval
-- (embeddings are out of D5 scope, contract §11).
CREATE INDEX IF NOT EXISTS idx_memories_fts ON memories
    USING gin(to_tsvector('english', coalesce(title, '') || ' ' || coalesce(l0_summary, '')));

-- =============================================================================
-- COMMENTs — the reasoning is part of the artifact (010/012 house style).
-- =============================================================================
COMMENT ON TABLE memories IS
  'Knowledge-layer memory base (D5, ONYX packet disp-d5-lane-a-onyx). Small-granularity distilled knowledge; large refined products stay in documents (002) — this does not fork that model. L0/L1/L2 = progressive disclosure (l0 ~100 tokens is the only default layer; l1 ~2K on demand; l2 = pointer to the full product), NOT three stored copies. Tier cold|warm|hot transitions ONLY via the explicit promote tool (up or down): no background daemon, no auto-decay, no auto-promotion. heat +1 per recall with NO automatic decay ever. forget is a SOFT tombstone (reason required); rows are never hard-deleted (ledger discipline); a correction = forget + remember. Rows are written ONLY through the MCP memory tools, each appending its event in the SAME transaction (012 pattern) — no second write path. Value-domain contract: docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-knowledge-layer-v1.md §4.';
COMMENT ON COLUMN memories.project_id IS
  'FK to projects(id). project+topic is the ONLY scoping framework — every knowledge object carries BOTH ids; agent harness + LLM are execution entities, never a scope.';
COMMENT ON COLUMN memories.topic_id IS
  'FK to topics(id) (010). The topic unit; hierarchy via topics.parent_topic_id (untouched by 015).';
COMMENT ON COLUMN memories.tags IS
  'Free-form labels, TEXT[] NOT NULL DEFAULT ''{}''. Not a scope — filtering convenience only.';
COMMENT ON COLUMN memories.l0_summary IS
  'The ~100-token default layer — the ONLY layer recall returns without an explicit depth request. Non-empty by contract (NOT NULL).';
COMMENT ON COLUMN memories.l1_overview IS
  'The ~2K-token overview, fetched only on explicit depth=l1. NULL until the authoring tool provides it.';
COMMENT ON COLUMN memories.l2_ref_kind IS
  'document | file — which pointer kind l2 carries. The pairing CHECK (memories_l2_pairing) also pins this 2-value domain: a third value satisfies neither branch.';
COMMENT ON COLUMN memories.l2_document_id IS
  'FK to documents(id) when l2_ref_kind=''document'' — the refined product stays in the 002 document-authority model.';
COMMENT ON COLUMN memories.l2_file_path IS
  'Repo-relative path when l2_ref_kind=''file'' (documents.file_path style). Exactly one of l2_document_id / l2_file_path per row (memories_l2_pairing).';
COMMENT ON COLUMN memories.l2_body_digest IS
  'sha256 hex of the referenced body (documents.body_digest style) — integrity pin for the pointed-to product.';
COMMENT ON COLUMN memories.source_kind IS
  'archived_session | canonical_event | document | manual — what this memory was distilled FROM. Domain enforced by the writing tool (typed rejection), not a DB CHECK: the contract lists the values in prose, and a DB CHECK here would freeze the set before the tools exist.';
COMMENT ON COLUMN memories.source_ref IS
  'NO FK — polymorphic reference: ''<harness>:<id>'' for archived_session, event id for canonical_event, doc id for document, actor note for manual. Same no-FK-polymorphic reasoning as 010''s event_object_refs.ref_id and 012''s dispatch_dependencies.dep_value.';
COMMENT ON COLUMN memories.heat IS
  '+1 per recall. NO automatic decay, ever (contract §4) — heat is an explicit audit counter, not a cache TTL.';
COMMENT ON COLUMN memories.recall_count IS
  'Total recalls (each appends exactly one MemoryRecalled event — contract §9; the ledger grows from "what happened" to "what is known").';
COMMENT ON COLUMN memories.last_recalled_at IS
  'NULL = never recalled. Set by the recall tool''s projection UPDATE (WorkItemStatusChanged pattern), backed by the MemoryRecalled event.';
COMMENT ON COLUMN memories.tier IS
  'cold | warm | hot (memories_tier_vocab). Transitions ONLY via the promote tool with an explicit target tier (up or down); tier_changed_at records the last transition. No background daemon, no auto-promotion, no auto-decay.';
COMMENT ON COLUMN memories.tier_changed_at IS
  'When the tier last changed (promote tool). NOT NULL DEFAULT NOW() at creation.';
COMMENT ON COLUMN memories.status IS
  'active | forgotten (memories_status_vocab). forget is a SOFT tombstone: the row stays, status flips, forgotten_reason required. Rows are never hard-deleted (ledger discipline).';
COMMENT ON COLUMN memories.forgotten_reason IS
  'Required when status=''forgotten'' (the soft tombstone''s reason, carried into the MemoryForgotten event payload). Enforced by the forget tool; the DB does not add a cross-column CHECK the contract does not specify.';
COMMENT ON COLUMN memories.created_at IS
  'Row creation time (the remember tool, same transaction as the MemoryCreated event).';
COMMENT ON COLUMN memories.updated_at IS
  'Last mutation time (recall/promote/forget projections, each event-backed).';

-- =============================================================================
-- 3. VOCABULARY WIDENING — event_type +4 (memory lifecycle), ref_type +1 (memory)
-- =============================================================================
-- 015 writes the COMPLETE lists (013's guarded pattern): the guard checks for
-- the new values and DROP+ADDs the full list only when they are absent, so the
-- migration applies cleanly from EITHER the track state (15 event types / 10
-- ref types, pre-013/014) OR the live state (21 / 11, 013/014 applied
-- out-of-band) — the full list is a superset of both. Existing rows can never
-- violate the wider list.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%MemoryCreated%'
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
        'AcceptanceRequested',  -- -> pending            (actor = seat)
        'AcceptanceGranted',    -- pending -> accepted   (actor = operator)
        'AcceptanceRejected',   -- pending -> rejected   (actor = operator)
        -- (015, D5) memory lifecycle — each maps to EXACTLY ONE transition
        'MemoryCreated',        -- (none) -> active/cold; actor = the remembering seat
        'MemoryRecalled',       -- active: heat+1, recall_count+1, last_recalled_at=now
                                 -- (projection UPDATE backed by the event, WorkItemStatusChanged
                                 -- pattern; payload carries the heat delta)
        'MemoryPromoted',       -- tier change; payload carries from_tier/to_tier (up or down)
        'MemoryForgotten'       -- active -> forgotten; payload carries reason (soft tombstone)
      ));
  END IF;
END $$;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'event_object_refs_ref_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%memory%'
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
        'memory'       -- 015 (D5): memories is its own object; the four memory
                       -- events must be able to name the memory they are about
                       -- (same reasoning as 010 'delegation' / 012 'dispatch' /
                       -- 014 'acceptance').
      ));
  END IF;
END $$;

-- =============================================================================
-- 4. PROJECT SLUG NORMALIZATION — v_archived_sessions_normalized (contract §2)
-- =============================================================================
-- archived_sessions.project is free text (011 design: archive-only projects, no
-- FK). This view is the EXECUTABLE form of the R1-R5 rule — a read-only
-- projection, ZERO row inserts (R3: materializing the tree is the Phase-2
-- washing pipeline, Aegis lane):
--   R1 exact: slug == projects.id -> that project.
--   R2 path:  'a/b' -> leaf project 'b' under parent 'a' ('a/b/c' nests
--       likewise: parent = everything before the last '/', leaf = after it);
--       mapped only when that (leaf, parent) pair really exists in the tree.
--   R4 unmapped: unknown slug -> map_status 'unmapped', reported, never guessed
--       (a path whose leaf is not a project, or whose leaf's parent differs
--       from the slug's prefix, is unmapped — not force-fit).
--   R5 '(root)': unmapped (13 sessions) + ESCALATED to Aegis — no ruling is
--       assumed here; the strategy decision is requested, not taken.
-- NULL project slugs are unmapped too (never guessed).
--
-- Catalog-guarded creation (idempotent): the view is created only when absent.
-- A future definition change is a FOLLOW-UP migration's DROP+CREATE, never a
-- silent rewrite here.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_class c
      JOIN pg_namespace n ON n.oid = c.relnamespace
     WHERE c.relname = 'v_archived_sessions_normalized'
       AND c.relkind = 'v'
       AND n.nspname = current_schema()
  ) THEN
    EXECUTE $view$
      CREATE VIEW v_archived_sessions_normalized AS
      SELECT
          a.harness                                        AS harness,
          a.id                                             AS id,
          a.project                                        AS raw_slug,
          -- normalized_parent / normalized_leaf: the resolved tree position.
          CASE
            WHEN a.project IS NULL OR a.project = '(root)' THEN NULL       -- R4/R5
            WHEN p1.id IS NOT NULL THEN p1.parent_project_id               -- R1 exact
            WHEN a.project LIKE '%/%' THEN
                 substring(a.project from '^(.*)/[^/]+$')                  -- R2 path prefix
            ELSE NULL
          END                                               AS normalized_parent,
          CASE
            WHEN a.project IS NULL OR a.project = '(root)' THEN NULL       -- R4/R5
            WHEN p1.id IS NOT NULL THEN p1.id                              -- R1 exact
            WHEN a.project LIKE '%/%' THEN
                 substring(a.project from '([^/]+)$')                      -- R2 leaf
            ELSE NULL
          END                                               AS normalized_leaf,
          -- map_status: mapped iff the slug resolves into the EXISTING tree.
          CASE
            WHEN a.project IS NULL OR a.project = '(root)' THEN 'unmapped' -- R4/R5
            WHEN p1.id IS NOT NULL THEN 'mapped'                           -- R1
            WHEN p2.id IS NOT NULL THEN 'mapped'                           -- R2
            ELSE 'unmapped'                                                -- R4
          END                                               AS map_status
      FROM archived_sessions a
      -- R1: exact slug == projects.id
      LEFT JOIN projects p1 ON p1.id = a.project
      -- R2: 'a/b[/c...]' -> leaf project under parent prefix (both must exist)
      LEFT JOIN projects p2
             ON a.project LIKE '%/%'
            AND p2.id = substring(a.project from '([^/]+)$')
            AND p2.parent_project_id = substring(a.project from '^(.*)/[^/]+$')
    $view$;
  END IF;
END $$;

COMMENT ON VIEW v_archived_sessions_normalized IS
  'Executable project-slug normalization rule (D5, contract §2 R1-R5). Read-only projection over archived_sessions.project free text: R1 exact slug == projects.id; R2 ''a/b'' -> leaf project ''b'' under parent ''a'' (''a/b/c'' nests likewise; mapped only when that (leaf, parent) pair exists); R4 unknown slug -> unmapped, reported, never guessed; R5 ''(root)'' -> unmapped + ESCALATED to Aegis (13 sessions; strategy decision requested, not assumed). ZERO row inserts — materializing the tree is the Phase-2 washing pipeline (Aegis lane). Definition changes go in a follow-up migration, never a silent rewrite.';

COMMIT;
