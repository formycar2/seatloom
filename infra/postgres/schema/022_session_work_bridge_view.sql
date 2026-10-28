-- 022_session_work_bridge_view.sql
-- =============================================================================
-- The session<->work bridge as ONE read-only view: v_memory_session_join.
--
-- WHY IT EXISTS. A memory's `source_ref` is a recorded provenance string with NO
-- foreign key (015's design, restated by `remember`'s own tool description:
-- "No FK — recorded, never guessed"). That is correct for a write path, but it
-- means the join can break in silence. It already did: before 021's re-key, 3 of
-- 6 codex `source_ref` values resolved against NO row, and nothing in the product
-- reported it — a session looked like it had produced nothing. This view is the
-- executable form of "can this be measured?", and the number it reports is the
-- load-bearing wall of LYRA-2026-10-16-session-work-bridge-contract-v1 §11.2
-- slice 1.
--
-- Authority: docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md
-- (Lyra, packet/lyra/milestone-summary) §11.2 slice 1 "make the join executable
-- in SQL and measurable". Dispatched as disp-XnCxvGI88oAbdGcyeefhl
-- (AEGIS-2026-10-26-two-gates-blocked-v1, task A) because the gate
-- ev-session-key-readmodel-live measured the view ABSENT and three dispatches
-- have been blocked on it since 10-16.
--
-- =============================================================================
-- MEASURED ON THE LIVE DATABASE BEFORE WRITING THIS (2026-10-26)
-- =============================================================================
--   294 memories, of which 90 carry source_kind='archived_session' across
--        10 DISTINCT source_ref values (not 90 sessions — 90 records)
--   272 archived_sessions rows; 5 sessions rows (the seed's)
--
--   A1 (the declared path pair, the contract's PRIMARY key):
--       regexp_replace(l2_file_path,'^~/Documents/GitHub/AgentHistory/','')
--           == archived_sessions.raw_path
--       -> 90 RESOLVED, 0 UNRESOLVED. Every row's l2_file_path carries the
--          '~/Documents/GitHub/AgentHistory/' prefix (0 exceptions measured).
--
--   A2 (source_ref split on the FIRST colon, the contract's SECONDARY key):
--       (harness, id) == (split_part(source_ref,':',1),
--                         substring(source_ref from position(':' in source_ref)+1))
--       -> 87 RESOLVED, 3 UNRESOLVED.
--
--   The 3 that A2 cannot resolve are exactly 3 rows of one codex memory whose
--   source_ref is 'codex:019dda18-07f6-79e0-8008-e4eb16a5448d'. A1 resolves them
--   all: their l2_file_path normalises to a raw_path that exists. So the
--   disagreement between the two keys is REAL and is exactly the class of drift
--   this contract exists to surface — not a rounding artefact.
--
--   This is why the view reports BOTH keys side by side instead of picking one:
--   the contract names A1 primary and A2 secondary, and the reason A2 is
--   secondary is precisely that it has failed once. A view that resolved only
--   through A1 would hide the 3 rows A2 still cannot reach; a view that resolved
--   only through A2 would report them as orphans when they are not. Both columns,
--   plus a resolved-by column, so the disagreement is visible on every read.
--
-- =============================================================================
-- WHAT THIS VIEW IS NOT
-- =============================================================================
--   NOT a table, NOT materialized, ZERO rows inserted (contract slice 1).
--   It changes the behaviour of no existing column on `memories` or
--   `archived_sessions` — it adds a projection, touches no data.
--   It does NOT answer "what ledger events did this session produce": the
--   capture spine did not exist when any corpus session ran (earliest ledger
--   event 2026-10-09, latest corpus session 2026-10-08), so that is an empty set
--   by construction and the UI must say WHY rather than showing an empty list
--   (contract §5, red line 1 — writing those events would be fabrication).
--
-- =============================================================================
-- IDEMPOTENT
-- =============================================================================
-- Catalog-guarded creation, 015's own pattern for v_archived_sessions_normalized:
-- the view is created only when absent. A later definition change is a FOLLOW-UP
-- migration's DROP+CREATE, never a silent rewrite here — a view that quietly
-- changes shape under a reader is the same silent break this migration exists to
-- prevent. Re-running this file therefore writes zero catalog rows.
--
-- It writes NO DATA at all (a view has no rows), so PRE-SEED/POST-SEED
-- classification is decided by what it READS: it reads memories and
-- archived_sessions, which the seed never writes (measured: zero
-- `INSERT INTO memories` in any seed file; the seed's 5 `sessions` rows are
-- seeded, and this view reads `sessions` only for the seat-facing half). Same
-- POST-SEED rule as 015's view.
--
-- Canonical order 021 -> 022.
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
      WITH keyed AS (
        -- One row per memory that claims a session provenance. The three keys are
        -- computed here so the SELECT below reads as the report it is.
        SELECT
            m.id                                            AS memory_id,
            m.project_id                                    AS project_id,
            m.topic_id                                      AS topic_id,
            m.title                                         AS title,
            m.kind                                          AS kind,
            m.confidence                                    AS confidence,
            m.l0_summary                                    AS l0_summary,
            m.source_kind                                   AS source_kind,
            m.source_ref                                    AS source_ref,
            m.l2_file_path                                  AS l2_file_path,
            m.l2_ref_kind                                   AS l2_ref_kind,
            -- A1: the contract's PRIMARY key. Both sides DECLARE the path; neither
            -- derives it, and a path does not change when an id convention does —
            -- which is the property that makes it immune to the re-key that broke
            -- A2 once already. One documented normalization: strip the corpus root.
            regexp_replace(m.l2_file_path, '^~/Documents/GitHub/AgentHistory/', '')
                                                           AS a1_path,
            -- A2: the contract's SECONDARY key. Split on the FIRST colon so a
            -- session id containing a colon cannot corrupt the harness half.
            split_part(m.source_ref, ':', 1)               AS a2_harness,
            substring(m.source_ref from position(':' in m.source_ref) + 1)
                                                           AS a2_id
        FROM memories m
        WHERE m.source_kind = 'archived_session'
          -- An empty source_ref is not a key. Recorded, never guessed: it is
          -- reported as unresolvable rather than matched against nothing.
          AND coalesce(m.source_ref, '') <> ''
      )
      SELECT
          k.memory_id,
          k.project_id,
          k.topic_id,
          k.title,
          k.kind,
          k.confidence,
          k.l0_summary,
          k.source_ref,
          k.l2_file_path,
          k.l2_ref_kind,

          -- The resolved session, or NULL when neither key reaches one.
          coalesce(s1.harness, s2.harness)                 AS session_harness,
          coalesce(s1.id, s2.id)                           AS session_id,
          coalesce(s1.project, s2.project)                 AS session_project,
          coalesce(s1.raw_path, s2.raw_path)               AS session_raw_path,
          coalesce(s1.message_count, s2.message_count)     AS session_message_count,
          coalesce(s1.cwd, s2.cwd)                         AS session_cwd,

          -- Which key did the resolving. Both keys agreeing is the healthy case;
          -- a disagreement is the drift this contract exists to surface.
          CASE
            WHEN s1.id IS NOT NULL AND s2.id IS NOT NULL THEN 'both'
            WHEN s1.id IS NOT NULL THEN 'a1_path'
            WHEN s2.id IS NOT NULL THEN 'a2_source_ref'
            ELSE 'unresolved'
          END                                              AS resolved_by,

          -- What is missing, named. An unresolved row must be actionable, not
          -- merely counted: the UI has to say WHICH pointer is absent (contract
          -- §9 step 4 — "records exist but do not resolve" must look different
          -- from "no records").
          CASE
            WHEN s1.id IS NOT NULL OR s2.id IS NOT NULL THEN NULL
            WHEN coalesce(k.l2_file_path, '') = '' THEN 'l2_file_path'
            ELSE 'session'
          END                                              AS missing,

          -- A1 and A2 verdicts, kept separate so the 3-row disagreement stays
          -- countable: `select resolved_by, count(*) group by 1` is the drift
          -- report, and it must not collapse into one number.
          (s1.id IS NOT NULL)                              AS a1_resolves,
          (s2.id IS NOT NULL)                              AS a2_resolves
      FROM keyed k
      -- LEFT JOIN, never INNER: a memory that reaches no session is the finding,
      -- and an inner join would delete the evidence.
      LEFT JOIN archived_sessions s1 ON s1.raw_path = k.a1_path
      LEFT JOIN archived_sessions s2 ON s2.harness = k.a2_harness AND s2.id = k.a2_id
    $view$;
  END IF;
END $$;

-- =============================================================================
-- VERIFICATION — the numbers this view must reproduce on the live database.
-- Run after applying; they are the contract's load-bearing wall.
-- =============================================================================
-- SELECT resolved_by, count(*) FROM v_memory_session_join GROUP BY 1 ORDER BY 2 DESC;
--   both         | 87     <- the healthy case
--   a1_path      |  3     <- the codex rows A2 cannot reach: A1 is primary and holds
--   a2_source_ref|  0
--   unresolved   |  0
-- SELECT count(*) FROM v_memory_session_join WHERE NOT a1_resolves;   -- 0
-- SELECT count(*) FROM v_memory_session_join WHERE NOT a2_resolves;   -- 3  <- the drift number
-- SELECT count(*) FROM v_memory_session_join WHERE missing IS NOT NULL; -- 0
--
-- The load-bearing assertion is NOT "everything resolves". It is that the number
-- is MEASURABLE. Before 021, 3 codex references resolved against nothing and
-- nothing reported it; today the same 3 rows are visible, named, and separated
-- from the 87 that both keys agree on.
