-- 005_adopted_seats_and_external_projects.sql
-- =============================================================================
-- The seats, projects, role bindings and workitems that exist in the LIVE
-- database but were declared by NO seed file. This file closes that gap.
--
-- WHY THIS FILE EXISTS. Aegis measured on 2026-10-28 23:00 (dispatch
-- disp-bc20UzOznJQ8hCgbHw876, verdict event ev-v0zcUVJ-oWefYiXC-9x5C):
--   select id from seats order by id   -> 10 rows
--   grep -oE 'seat-[a-z]+-001' infra/postgres/seed/*.sql | sort -u -> 6 ids
-- The seed described a fleet that does not run. Anyone rebuilding from seed
-- got six seats and one project; the live database has ten seats, forty
-- projects and ten role bindings. A baseline that does not describe the
-- system it baselines is not a baseline — it is a snapshot of a system that
-- stopped existing in April.
--
-- PROVENANCE OF EVERY ROW (all four were written to the live database by an
-- ops script and never mirrored back into the seed; confirmed from git
-- history, not from Aegis's guess):
--
--   seat-vela-001                 scripts/bind-vela-to-seatloom.sh
--                                  commit 1143995 2026-10-16, Xiaolong Zhang
--   seat-ql-001                   scripts/adopt-ql.sh
--                                  commits 9b4d726 / 9ecd57c 2026-10-16
--   seat-networkorchestrator-001  scripts/adopt-network-orchestrator.sh
--                                  commits 4403715 / 6b10242 2026-10-26
--   seat-stepaicloud-001          scripts/adopt-stepaicloud.sh
--                                  commits 88cdee9 / a72cfc7 2026-10-27
--
--   The three external PROJECTS (project_delivery/QL, /networkOrchestrator,
--   /stepaicloud) come from the same adopt scripts. `project_delivery/QL` is
--   ALSO created by migration 021 as one of its 38 corpus homes, so it is NOT
--   declared here — 021 owns it. Only the two projects 021 does not know are
--   declared below.
--
-- IDEMPOTENT AND ADDITIVE. Every statement is guarded so a second apply
-- writes nothing: seats and workitems use `ON CONFLICT (id) DO NOTHING`,
-- projects uses `ON CONFLICT (id) DO NOTHING`, project_role_bindings uses
-- `ON CONFLICT DO NOTHING` on its natural key. No existing row is updated,
-- deleted or rewritten. This is a new file, not an edit to 001-004.
--
-- ONE STRUCTURAL GAP THIS FILE CANNOT CLOSE (recorded, not hidden):
-- `projects.home_path` is ADDED BY MIGRATION 021 (post-seed), so the seed
-- runs before the column exists. The two projects below therefore carry
-- NULL home_path on a fresh rebuild, while live has real paths. Filling them
-- is a migration's job (a guarded UPDATE in the 021 pattern), not the seed's;
-- it is listed in docs/coordination/tasks/onyx/2026-10-28-seed-drift-report-v1.md
-- as the one follow-up this file deliberately leaves open.
-- =============================================================================

-- -----------------------------------------------------------------------------
-- Seats. Every value is the LIVE row, read-only, on 2026-10-28 23:05.
-- The triple (default_harness / default_agent / default_model) is the
-- operator-ruled 2026-10-17 shape: default_harness = 'Pi' for every seat
-- (commit 9d8cb57, the ruling whose cost — "跑在什么包装层下" is no longer in
-- the table — was written into that script's header rather than hidden), and
-- default_model = the pi provider/model form, NOT the Claude Code-era
-- `claude-opus-5[1m]`. See the 001 note at the bottom of this file.
-- -----------------------------------------------------------------------------
INSERT INTO seats (id, name, default_runtime, default_harness, default_agent, default_model, capability_tags, status, created_at) VALUES
  -- seat-vela-001 — the washing seat. Registered by bind-vela-to-seatloom.sh
  -- (1143995) AFTER it had already been running as an unregistered ad-hoc
  -- session; the script inserts only the role binding, the seat row itself
  -- predates it (created_at 2026-10-10). Evidence: that commit plus the
  -- capability tags it declares (corpus-ingest/extraction/redaction/
  -- data-washing), which are vela's actual S0-S5 lane.
  ('seat-vela-001', 'vela', 'Pi', 'Pi', 'Pi', 'stepplan-wash-anthropic/step-5-preview',
   ARRAY['corpus-ingest','extraction','redaction','data-washing'], 'active', '2026-10-10T13:34:08.038279Z'),

  -- seat-ql-001 — the adopted external QL project's worker. Registered by
  -- adopt-ql.sh (9b4d726, renamed by 9ecd57c to ql_delivery per the
  -- operator's correction of an unasked default). Evidence: that commit,
  -- which inserts the seat row, the project row and the binding together as
  -- one transaction. capability_tags are what that script declared.
  ('seat-ql-001', 'ql', 'Pi', 'Pi', 'Pi', 'stepplan-wash-anthropic/step-5-preview',
   ARRAY['ql','delivery','huawei-storage'], 'active', '2026-10-16T10:49:03.570477Z'),

  -- seat-networkorchestrator-001 — adopted external project, registered by
  -- adopt-network-orchestrator.sh (4403715). capability_tags is the EMPTY
  -- array that script declares, not a value I chose: the adopt scripts write
  -- '{}' and the seat has no capability list in the product yet.
  ('seat-networkorchestrator-001', 'networkorchestrator', 'Pi', 'Pi', 'Pi', 'stepplan-wash-anthropic/step-5-preview',
   ARRAY[]::text[], 'active', '2026-10-26T11:36:02.857666Z'),

  -- seat-stepaicloud-001 — derived from the networkOrchestrator script but
  -- started fresh (88cdee9), because a new empty directory has no historical
  -- session to resume. Two defects fixed by a72cfc7 (PATH had no saih; the
  -- script waited without verifying). capability_tags '{}' as declared.
  ('seat-stepaicloud-001', 'stepaicloud', 'Pi', 'Pi', 'Pi', 'stepplan-wash-anthropic/step-5-preview',
   ARRAY[]::text[], 'active', '2026-10-27T05:16:04.759716Z')
ON CONFLICT (id) DO NOTHING;

-- -----------------------------------------------------------------------------
-- Projects. ONLY the two that migration 021 does not create.
--
-- project_delivery/QL is deliberately absent: 021 creates it as one of its
-- 38 corpus homes (021_history_sessions_corpus.sql line 201), and declaring
-- it here too would mean two files owning one row.
--
-- home_path is left NULL on purpose — see the structural-gap note at the top
-- of this file. The live values are recorded in the drift report instead.
-- -----------------------------------------------------------------------------
INSERT INTO projects (id, name, created_at) VALUES
  -- Evidence: adopt-network-orchestrator.sh:140 (4403715), which creates the
  -- project row, the seat row and the role binding together. live name is
  -- 'networkOrchestrator'; live id is 'project_delivery/networkOrchestrator'.
  ('project_delivery/networkOrchestrator', 'networkOrchestrator', '2026-10-26T11:36:02.856614Z'),

  -- Evidence: adopt-stepaicloud.sh:137 (88cdee9), the derived script.
  ('project_delivery/stepaicloud', 'stepaicloud', '2026-10-27T05:16:04.758710Z')
ON CONFLICT (id) DO NOTHING;

-- -----------------------------------------------------------------------------
-- Project role bindings. The four that exist in live and in no seed.
--
-- The three 'custom(worker)' roles are not a placeholder: the adopt scripts
-- declare exactly that string, and 'custom(worker)' is the value that the
-- SeatRole domain accepts for a non-SeatLoom worker (commit 575d85d moved
-- ql_delivery's role to custom(worker) for the same reason). vela's
-- data_engineer is the ROLE bind-vela-to-seatloom.sh was called with
-- (its --role default, line 23).
--
-- authority_doc_refs / constraints / collaboration_template_ref are the empty
-- arrays these scripts write; active_delegation_id is NULL (no delegation).
-- -----------------------------------------------------------------------------
INSERT INTO project_role_bindings (seat_id, project_id, role, authority_doc_refs, constraints, collaboration_template_ref, active_delegation_id) VALUES
  -- bind-vela-to-seatloom.sh:36-37 (1143995) — the only statement that script runs.
  ('seat-vela-001', 'seatloom', 'data_engineer', ARRAY[]::text[], ARRAY[]::text[], NULL, NULL),

  -- adopt-ql.sh:51 (9b4d726).
  ('seat-ql-001', 'project_delivery/QL', 'custom(worker)', ARRAY[]::text[], ARRAY[]::text[], NULL, NULL),

  -- adopt-network-orchestrator.sh:151 (4403715).
  ('seat-networkorchestrator-001', 'project_delivery/networkOrchestrator', 'custom(worker)', ARRAY[]::text[], ARRAY[]::text[], NULL, NULL),

  -- adopt-stepaicloud.sh:148 (88cdee9).
  ('seat-stepaicloud-001', 'project_delivery/stepaicloud', 'custom(worker)', ARRAY[]::text[], ARRAY[]::text[], NULL, NULL)
ON CONFLICT DO NOTHING;

-- -----------------------------------------------------------------------------
-- Workitems. One row in live and in no seed:
--   wi-p0-step6-governed-dispatch — owner seat-nimbus-001, status done.
--
-- WHY IT WAS NEVER SEEDED: it is the P0 step-6 workitem the governed-dispatch
-- slice itself was delivered under (commits 9b96ee8 / 4938545). The seed 001
-- block was written on 2026-04-27 and has not been touched since 009; every
-- workitem added after that date was written straight to live by the write
-- API (append(event) -> projection) and never mirrored back. The seed is not
-- "missing" it by mistake — it is missing the whole category of workitems
-- created after April. This row is declared so a rebuild reproduces the
-- six-item baseline plus this one; the remaining live workitems are the same
-- category and are listed in the drift report rather than invented here.
--
-- acceptance_criteria is a TEXT ARRAY (workitems.acceptance_criteria is
-- text[]), not jsonb — the first draft of this file got that wrong and the
-- scratch-DB apply caught it before commit. Values verbatim from live.
--
-- project_id = 'seatloom' is REQUIRED and was also absent from the first
-- draft: migration 008 made workitems.project_id NOT NULL, while seed 001's
-- workitems INSERT (written before 008) still omits the column. That is the
-- same pre-existing gap that makes seed 001 fail on a fresh database — see
-- the drift report. This file supplies it because live carries 'seatloom'.
-- -----------------------------------------------------------------------------
INSERT INTO workitems (id, title, goal, acceptance_criteria, owner_seat_id, status, priority, created_at, updated_at, project_id) VALUES
  ('wi-p0-step6-governed-dispatch',
   'P0 step 6 — governed dispatch (G19)',
   'Per-seat inbox + ready-aware scheduler so the loop self-runs (no dropped hops)',
   ARRAY['enqueue/get_next/complete wired','13-point scratch-DB proof green','one-in-flight DB-enforced'],
   'seat-nimbus-001', 'done', 'high',
   '2026-10-08T17:28:57.596980Z', '2026-10-08T17:28:57.739644Z', 'seatloom')
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- NOTE ON SEED 001's EXISTING SIX SEAT ROWS — a second drift this file does
-- not touch, recorded here so the next reader does not have to rediscover it.
--
-- Seed 001 upserts the three 009 columns with:
--     default_harness = 'Stepcode', default_agent = 'ClaudeCode',
--     default_model  = 'claude-opus-5[1m]'
-- LIVE has, for all six:
--     default_harness = 'Pi', default_agent = 'ClaudeCode',
--     default_model  = 'stepplan-wash-anthropic/step-5-preview'
--
-- Because 001 ends in `ON CONFLICT (id) DO UPDATE SET` on exactly those three
-- columns, the NEXT SEED APPLY OVERWRITES LIVE. That is not a cosmetic
-- mismatch: `claude-opus-5[1m]` is the Claude Code-era model name, not pi's
-- provider/model form, and scripts/seat-launch.sh's model_for() carries a
-- shape guard specifically to refuse it ("必须校验形状:seats.default_model
-- 里有历史遗留值"). So the stale value is defended against at the launch
-- path but not at the seed path, and the seed still wins on apply.
--
-- Provenance: 629c68e (009) wrote Stepcode/claude-opus-5[1m] into the seed;
-- af0275a set live to Stepcode; then the operator's 2026-10-17 ruling
-- (9d8cb57, fix-seat-harness-to-pi.sh) changed live's harness to Pi — a live
-- UPDATE, never mirrored into the seed. The model divergence came later via
-- the adopt scripts and set-seat-default-model.sh.
--
-- Fixing it means editing seed 001's VALUES block (and its ON CONFLICT body),
-- which is a change to an existing historical file rather than an additive
-- new one, so it is a separate decision for the operator/Aegis. It is the
-- highest-priority item in the drift report.
-- =============================================================================
