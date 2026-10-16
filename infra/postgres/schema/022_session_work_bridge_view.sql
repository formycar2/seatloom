-- 022_session_work_bridge_view.sql
-- =============================================================================
-- The session↔work bridge, slice 1: ONE read-only view that makes the join
-- executable in SQL and measurable — including the drift number that would have
-- caught the last silent break.
--
-- Authority: docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md
-- (Lyra, packet/lyra/milestone-summary @ 8601167) §3 and §11.2 slice 1,
-- dispatched as disp-session-key-view. Red lines restated from the contract's
-- §5 so this file has to guess at nothing:
--   * No event is ever written for a historical session. This file writes no
--     canonical_events row and no memory row.
--   * No joining by derived filename or by topic id. This view joins ONLY the
--     two Tier A keys of §3 — the declared path pair and the recorded session
--     reference. The topic id is never parsed.
--   * No Tier B link is presented as a recorded one. This view carries no
--     derived attribution of any kind.
--   * No transcript bodies. Pointers and line ranges only.
--   * No touching the live database, no gate events, no merging.
--
-- THE FINDING THAT DECIDES THIS FILE (contract §1): the join already exists.
-- 90 of the 154 live memories carry source_kind='archived_session' with
-- source_ref = '<harness>:<native session id>', and migration 021 re-keyed 182
-- archived_sessions rows to the native session id specifically so that would
-- resolve — its header says so in as many words, and it checked
-- pg_constraint/pg_views/pg_trigger for any FK before re-keying. So this is not
-- a key to build. It is an existing key to pin, make measurable, and stop from
-- being quietly broken again.
--
-- THE LOAD-BEARING REQUIREMENT (contract §11.2 slice 1): the view must answer
-- ONE NUMBER — how many memories carry a session reference that resolves against
-- no row. Before 021, 3 of 6 codex references resolved against nothing and
-- nothing in the product reported it, so those sessions looked like they had
-- produced nothing at all. The number is:
--
--   SELECT count(*) AS memories_with_unresolvable_session_ref
--     FROM v_memory_session_join
--    WHERE NOT a2_resolves;
--
-- Measured on the live database on 2026-10-16 (READ ONLY, no writes): 3. See
-- §"WHAT THE NUMBER IS TODAY" below and the delivery note
-- docs/coordination/tasks/onyx/ONYX-2026-10-16-session-key-view-v1.md.
--
-- =============================================================================
-- WHAT THE NUMBER IS TODAY — and why it is not zero
-- =============================================================================
-- Measured read-only against live on 2026-10-16:
--
--   archived_session memories .............. 90
--   A1 (path pair) resolves ................ 90
--   A2 (session reference) resolves ........ 87
--   NEITHER resolves ....................... 0
--   A1 and A2 resolve to different rows ..... 0
--
-- The 3 that A2 does not resolve all carry source_ref
-- 'codex:019dda18-07f6-79e0-8008-e4eb16a5448d' (3 memories, mem-kyloXn9AY3D_,
-- mem-Gih4q6JxjyyJ, mem-xqtBZmhQd1Iw). Their l2_file_path DOES resolve (A1), to
-- the archived_sessions row whose id is the raw FILE NAME
-- 'rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d'.
--
-- That row is one of the 60 codex rows 021 did NOT re-key, because it is one of
-- the 89 inventory-only rows outside the 183-session corpus (message_count IS
-- NULL — measured: 150 corpus codex rows all carry native-uuid ids, all 60
-- non-corpus codex rows still carry 'rollout-<ts>-<uuid>' file-name ids). So the
-- memory's recorded reference names an id that row has never carried, while its
-- path pointer names the row correctly.
--
-- This is exactly the silent break the contract's number exists to catch, and it
-- is live right now. The view reports it as join_state = 'path_only'. It does NOT
-- repair it: the contract's §4.2 rule 4 is forget + remember, never an in-place
-- UPDATE of source_ref, and no row is rewritten here. Reporting is this file's
-- whole job.
--
-- =============================================================================
-- SCOPE — which memories appear here, and why
-- =============================================================================
-- Only memories with source_kind = 'archived_session'. That is the only
-- source_kind whose source_ref is a session reference; the other three
-- (canonical_event / document / manual) point at an event id, a document id, or
-- an actor note. They have no session reference to resolve, so they are out of
-- this view's scope rather than rows whose reference is null. A memory's
-- l2_line_start/l2_line_end are REQUIRED for exactly this combination by 018's
-- memories_l2_lines_required CHECK, so a row here always carries a line range.
--
-- =============================================================================
-- IDEMPOTENCY — a second apply writes nothing at all
-- =============================================================================
-- Catalog-guarded creation in 015's pattern: the view is created only when
-- absent, and the COMMENT ON VIEW sits INSIDE the same guard, so a second apply
-- performs no catalog write either (a COMMENT ON rewrites pg_description). A
-- future definition change is a FOLLOW-UP migration's DROP+CREATE, never a
-- silent rewrite here.
--
-- Zero rows are inserted, updated or deleted by this file. It adds ONE view and
-- touches no table. No column on memories or archived_sessions changes
-- behaviour, and no FK is added to source_ref — remember's own contract says of
-- that column "No FK — recorded, never guessed", and the contract's §5 forbids
-- adding an FK to make it enforce something it was never designed to enforce.
-- Measure instead.
--
-- Slot 022: next after 021. POST-SEED by apply-db.sh's rule — it creates a view
-- over memories (015, POST-SEED) and archived_sessions (011, POST-SEED) and
-- adds no column any seed writes, so it cannot precede its prerequisites.
-- Canonical order 015 -> 021 -> 022.
-- =============================================================================

BEGIN;

-- =============================================================================
-- 1. THE VIEW — v_memory_session_join
-- =============================================================================
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_class c
      JOIN pg_namespace n ON n.oid = c.relnamespace
     WHERE c.relname = 'v_memory_session_join'
       AND c.relkind = 'v'
       AND n.nspname = current_schema()
  ) THEN
    EXECUTE $view$
      CREATE VIEW v_memory_session_join AS
      SELECT
          -- ---------------------------------------------------------------
          -- THE MEMORY SIDE — what was distilled. Read-only pointers and line
          -- ranges; never a transcript body (011's red line).
          -- ---------------------------------------------------------------
          mem.id            AS memory_id,
          mem.project_id    AS project_id,
          mem.topic_id      AS topic_id,
          mem.title         AS title,
          mem.kind          AS kind,
          mem.confidence    AS confidence,
          mem.status        AS status,
          mem.created_at    AS created_at,
          -- The L2 pointer, verbatim. Never rewritten, never normalized in the
          -- output — a consumer that needs the file on disk gets the exact
          -- string the write-in declared.
          mem.l2_file_path  AS l2_file_path,
          mem.l2_line_start AS l2_line_start,
          mem.l2_line_end   AS l2_line_end,

          -- ---------------------------------------------------------------
          -- TIER A1 (PRIMARY) — the declared path pair.
          -- ---------------------------------------------------------------
          -- l2_file_path with the corpus root stripped: the ONE documented
          -- normalization the contract's §3 allows. Both sides DECLARE this
          -- path; neither derives it, which is why it is primary — it is
          -- immune to id re-keying, the failure that broke its sibling key.
          mem.a1_corpus_path AS a1_corpus_path,
          -- How many archived_sessions rows declare that exact raw_path.
          -- Measured on live: 272 rows, 271 distinct raw_path — ONE cursor
          -- directory is declared by two sessions. A plain join on raw_path
          -- would fan that memory into two rows and double-count it in the
          -- load-bearing number, so the ambiguity is counted here, not
          -- silently resolved.
          a1.path_rows      AS a1_path_rows,
          -- The resolved row's identity, NULL unless exactly one row declares
          -- the path. Reported, never guessed (015's R4 rule, same shape).
          CASE WHEN a1.path_rows = 1 THEN a1.harness END AS a1_harness,
          CASE WHEN a1.path_rows = 1 THEN a1.id      END AS a1_id,
          (a1.path_rows = 1) AS a1_resolves,

          -- ---------------------------------------------------------------
          -- TIER A2 (SECONDARY) — the recorded session reference.
          -- ---------------------------------------------------------------
          -- source_ref split on the FIRST colon: harness before it, id
          -- everything after it (so a wrapped id such as stepcode's
          -- 'claude:<uuid>' survives whole). A reference with no colon at all
          -- is malformed: BOTH halves are NULL rather than one half being the
          -- whole string, so the row is measured unresolved instead of
          -- half-resolved.
          mem.ref_harness AS ref_harness,
          mem.ref_id      AS ref_id,
          -- The row the reference points at. (harness, id) is
          -- archived_sessions' PRIMARY KEY, so this join is at most one row by
          -- construction — unlike A1's.
          a2.harness      AS a2_harness,
          a2.id           AS a2_id,
          (a2.id IS NOT NULL) AS a2_resolves,

          -- ---------------------------------------------------------------
          -- THE JOINED SESSION — A1 primary, A2 secondary.
          -- ---------------------------------------------------------------
          coalesce(CASE WHEN a1.path_rows = 1 THEN a1.harness END, a2.harness)
                          AS session_harness,
          coalesce(CASE WHEN a1.path_rows = 1 THEN a1.id END,      a2.id)
                          AS session_id,
          CASE WHEN a1.path_rows = 1 THEN a1.raw_path       END AS session_raw_path,
          CASE WHEN a1.path_rows = 1 THEN a1.project        END AS session_project,
          -- 021's hand-off note #1: message_count IS NOT NULL is what
          -- distinguishes the 183-row corpus from the 271-row archive
          -- inventory. A read that forgets this double-counts. Carried here
          -- because whether the joined session is a corpus session changes how
          -- the join's result must be read.
          CASE WHEN a1.path_rows = 1 THEN a1.message_count END AS session_message_count,

          -- ---------------------------------------------------------------
          -- join_state — the exhaustive five-value answer to "did this memory
          -- reach its session, and by which key". The contract's §7.2 needs
          -- three states distinguishable, and they are:
          --   both_agree  records exist and resolve, by both keys
          --   path_only   records exist; the PATH resolves but the recorded
          --                SESSION REFERENCE does not — this is the silent
          --                break, and today it is 3 memories
          --   ref_only    records exist; the reference resolves but the path
          --                pointer does not — the other half of the same break
          --   disagree    both resolve, to DIFFERENT rows — the two recorded
          --                keys contradict each other (0 today; the view must
          --                not be able to hide it)
          --   none        neither resolves — the memory is orphaned from the
          --                corpus
          -- "No distilled records at all" is NOT a value here: it is a
          -- SESSION-side state (zero rows for that session), which is what a
          -- session-centric read over this view returns. The contract's §7.2
          -- requires it be distinguishable from "some, but broken", and it is —
          -- zero rows is not join_state='none'.
          CASE
            WHEN a1.path_rows = 1 AND a2.id IS NOT NULL
                 AND (a1.harness, a1.id) IS NOT DISTINCT FROM (a2.harness, a2.id)
              THEN 'both_agree'
            WHEN a1.path_rows = 1 AND a2.id IS NOT NULL
              THEN 'disagree'
            WHEN a1.path_rows = 1
              THEN 'path_only'
            WHEN a2.id IS NOT NULL
              THEN 'ref_only'
            ELSE 'none'
          END AS join_state
      FROM (
          -- The memories that carry a session reference, with BOTH Tier A keys
          -- PARSED (not yet resolved). The outer SELECT names every output
          -- column explicitly, so a column added to `memories` later cannot
          -- silently widen this view's output.
          SELECT
              m.*,
              -- A1: strip the corpus root. tools/wash/s5_writein.py writes
              -- l2_file_path as '~/Documents/GitHub/AgentHistory/' +
              -- <corpus-relative path>; 011/021 write archived_sessions.
              -- raw_path as that same corpus-relative path. Both spellings the
              -- producer has ever written are accepted — the ~-rooted form and
              -- the bare corpus-relative form 011/015 document ("path within
              -- AgentHistory/"). Anything else yields NULL here, and a NULL
              -- pointer is NOT force-matched: it is measured as unresolved.
              CASE
                WHEN m.l2_file_path LIKE '%/Documents/GitHub/AgentHistory/%'
                  THEN substring(m.l2_file_path
                                 from '/Documents/GitHub/AgentHistory/(.*)$')
                WHEN m.l2_file_path LIKE 'raw/%'
                  THEN m.l2_file_path
                ELSE NULL
              END AS a1_corpus_path,
              CASE WHEN position(':' in m.source_ref) > 0
                   THEN split_part(m.source_ref, ':', 1)
              END AS ref_harness,
              CASE WHEN position(':' in m.source_ref) > 0
                   THEN substring(m.source_ref
                                  from position(':' in m.source_ref) + 1)
              END AS ref_id
            FROM memories m
           WHERE m.source_kind = 'archived_session'
      ) mem
      -- A1 resolution: at most one row, plus how many rows claimed the path.
      -- The window is computed over every row matching the WHERE, before
      -- ORDER BY/LIMIT narrow it to one — so path_rows is the true claim count,
      -- and the ORDER BY is only a deterministic choice of WHICH row to show,
      -- never a resolution.
      LEFT JOIN LATERAL (
          SELECT s.harness, s.id, s.raw_path, s.project, s.message_count,
                 count(*) OVER () AS path_rows
            FROM archived_sessions s
           WHERE s.raw_path = mem.a1_corpus_path
           ORDER BY s.harness, s.id
           LIMIT 1
      ) a1 ON true
      -- A2 resolution: the PRIMARY KEY, so at most one row by construction.
      LEFT JOIN archived_sessions a2
             ON a2.harness = mem.ref_harness
            AND a2.id      = mem.ref_id
    $view$;

    COMMENT ON VIEW v_memory_session_join IS
      'The session↔work bridge, slice 1 (contract LYRA-2026-10-16-session-work-bridge-contract-v1 §3, §11.2; dispatch disp-session-key-view). ONE read-only view — no table, no materialization, zero rows inserted — that makes the existing join executable and measurable. Tier A1 (PRIMARY) joins a memory''s l2_file_path with the corpus root stripped against archived_sessions.raw_path: both sides DECLARE the path, neither derives it, and it is immune to id re-keying, which is why it is primary. Tier A2 (SECONDARY) splits source_ref on the FIRST colon and joins (harness, id) — the PRIMARY KEY, so at most one row. A1 resolves ONLY when exactly one row declares the path (a1_path_rows = 1): raw_path is not unique, one cursor directory is declared by two sessions, and a plain join would fan that memory into two rows and double-count it in the load-bearing number. Ambiguity is counted and reported, never guessed (015''s R4 rule). join_state is the exhaustive five-value answer: both_agree | path_only | ref_only | disagree | none. THE LOAD-BEARING NUMBER — how many memories carry a session reference that resolves against no row — is: SELECT count(*) FROM v_memory_session_join WHERE NOT a2_resolves. Measured on live 2026-10-16: 3 (all source_ref ''codex:019dda18-07f6-79e0-8008-e4eb16a5448d''; their path pointer resolves, so they are path_only, and the row they point at is an inventory-only row whose id 021 never re-keyed because it is outside the 183-session corpus). That is the silent break this number exists to catch. It does NOT repair it: a wrong source_ref is forget + remember (015), never an in-place UPDATE, and no row is rewritten here. ''No distilled records at all'' is not a join_state — it is a session-side state, zero rows for that session, which is what makes it distinguishable from ''some, but broken'' (contract §7.2). No topic id is ever parsed (contract §4 anti-pattern 1); no Tier B attribution is carried (§5 red line 3); no transcript body (§5 red line 4); no FK is added to source_ref, which remember''s own contract declares "No FK — recorded, never guessed". Definition changes go in a follow-up migration, never a silent rewrite.';
  END IF;
END $$;

COMMIT;
