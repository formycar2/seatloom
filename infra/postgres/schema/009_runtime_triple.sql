-- 009_runtime_triple.sql
--
-- Split the collapsed `Runtime` dimension into the harness / agent / model triple.
-- Operationalizes the PostgreSQL side of
--   docs/coordination/reviews/2026-09-28-aegis-multi-harness-session-ingestion-architecture-v1.md §4
-- under packet ONYX-2026-09-28-runtime-triple-migration-v1 (Aegis review §8 item 2),
-- paired with NIMBUS-2026-09-28-runtime-triple-refactor-v1 on the Rust side.
--
-- WHY: `Runtime` collapsed three orthogonal dimensions into one column —
--   harness  = process wrapper / credential / proxy layer   (e.g. `stepcode`)
--   agent    = the CLI or protocol being driven             (e.g. Claude Code)
--   model    = the model id the agent is pinned to          (e.g. `claude-opus-5[1m]`)
-- As of 2026-09-28 all six seats run stepcode → Claude Code → `claude-opus-5[1m]`
-- (docs/coordination/memory/2026-09-28.md Addendum, "Seat state after convergence"),
-- a state `default_runtime = 'ClaudeCode'` cannot express.
--
-- Dimension assignment, stated explicitly so a future reader does not re-add either
-- value on the wrong axis: `stepcode` / `stepgo` is a HARNESS, not an agent.
-- `opencode` is an AGENT, not a harness. One enum could not express that; three can.
--
-- VALUE-DOMAIN CONTRACT (Lyra-owned; packet §2.1, verbatim in both paired packets):
--
--   | Field   | Serialized domain                                              | PG column | Nullable |
--   |---------|----------------------------------------------------------------|-----------|----------|
--   | harness | Direct, Stepcode, Custom                                       | TEXT      | yes      |
--   | agent   | ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Custom      | TEXT      | no  (sessions) / yes (seats.default_*) |
--   | model   | free-form, e.g. `claude-opus-5[1m]`                            | TEXT      | yes      |
--
-- Contract rules honoured here:
--   1. Values stay PascalCase verbatim. Existing `sessions.runtime` values are already
--      PascalCase (ClaudeCode, GeminiCli, Custom), so this migration is a pure column
--      split, NOT a value re-encoding. Nothing is lowercased or snake_cased.
--   2. No CHECK constraint enumerating models — model ids churn faster than releases.
--      (Also no CHECK on harness/agent: `Custom` is an open escape hatch on both, and
--      the authoritative domain check lives in Nimbus's fallible parse boundary at row
--      hydration, which fails loudly on unknown values per review §5.)
--   3. NULL harness / NULL model mean *the value was not recorded at the time*. They do
--      NOT mean "direct" or "default". This migration backfills NO guess: a NULL you can
--      later fill is recoverable, a fabricated 'Direct' is indistinguishable from a real
--      one forever (review §2 — attribution is derived and revisable, never baked in).
--   4. SeatRole values are untouched; they stay snake_case. Nimbus adds
--      rename_all = "snake_case" plus Supervisor and DataEngineer variants so the
--      existing seed values become parseable. This migration changes no role value.
--
-- ADDITIVE ONLY. Nothing is dropped.
--   `seats.default_runtime` and `sessions.runtime` are RETAINED and marked deprecated
--   via COMMENT ON COLUMN. Dropping them in the same migration that adds their
--   replacements would make the rollback path a data-loss event. Retirement is deferred
--   to a follow-up packet, after Flux confirms no reader references them.
--
-- Slot choice: 009. Slots 006 (`plan_mode_authority`) and 007 (`seats_budget`) remain
-- RESERVED and unlanded under the Aegis 2026-05-09 numbering ruling; occupying either
-- would break that ruling.
--
-- IDEMPOTENCY STRATEGY (proven by applying this file twice; both transcripts are in the
-- delivery doc):
--   * ADD COLUMN IF NOT EXISTS — supported on PG 9.6+; this container runs PG 16
--     (postgres:16-alpine, infra/postgres/docker-compose.yml).
--   * Every backfill UPDATE is guarded by `WHERE <target> IS NULL`, so a second apply
--     matches zero rows (UPDATE 0). The guard is deliberately `IS NULL` rather than
--     `IS DISTINCT FROM <source>`: once a writer legitimately sets an agent that differs
--     from the legacy `runtime` value, re-running this migration must not clobber it.
--   * SET NOT NULL on an already-NOT NULL column is a no-op.
--   * COMMENT ON COLUMN is unconditional overwrite — inherently idempotent.
--   * Whole file is one transaction.

BEGIN;

-- =============================================================================
-- 1. seats — add the default triple (all nullable, matching `default_runtime TEXT`).
-- =============================================================================
ALTER TABLE seats ADD COLUMN IF NOT EXISTS default_harness TEXT;
ALTER TABLE seats ADD COLUMN IF NOT EXISTS default_agent   TEXT;
ALTER TABLE seats ADD COLUMN IF NOT EXISTS default_model   TEXT;

-- =============================================================================
-- 2. sessions — add the triple. `agent` is tightened to NOT NULL in step 5, after
--    backfill, to match the existing `runtime TEXT NOT NULL`.
-- =============================================================================
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS harness TEXT;
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS agent   TEXT;
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS model   TEXT;

-- =============================================================================
-- 3. Backfill sessions.agent from sessions.runtime — a straight copy.
--    Contract rule 1 keeps the value domain identical, so this is a rename-in-place
--    of the one dimension the legacy column actually carried.
--    sessions.harness and sessions.model are left NULL on purpose (rule 3): they were
--    never recorded for any existing row. No guess is written here.
-- =============================================================================
UPDATE sessions SET agent = runtime WHERE agent IS NULL;

-- =============================================================================
-- 4. Backfill seats.default_agent from seats.default_runtime — straight copy.
--    seats.default_harness and seats.default_model are left NULL by THIS MIGRATION.
--    The seed sets them for the six known seats, because that is where the evidence
--    lives (docs/coordination/memory/2026-09-28.md Addendum) — a schema migration is
--    the wrong place to assert a fact about which harness a particular seat runs.
-- =============================================================================
UPDATE seats SET default_agent = default_runtime
 WHERE default_agent IS NULL
   AND default_runtime IS NOT NULL;

-- =============================================================================
-- 5. Tighten sessions.agent to NOT NULL, matching `runtime TEXT NOT NULL`.
--    Safe: step 3 populated every row from a NOT NULL source column.
--    seats.default_agent stays NULLABLE, matching `default_runtime TEXT` nullable.
-- =============================================================================
ALTER TABLE sessions ALTER COLUMN agent SET NOT NULL;

-- =============================================================================
-- 6. Deprecate — but RETAIN — the collapsed legacy columns.
--    Retirement is deliberately deferred to a follow-up packet (packet §2.2.6 /
--    §3), gated on Flux confirming no reader references them.
-- =============================================================================
COMMENT ON COLUMN seats.default_runtime IS
  'DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1. Superseded by '
  'default_harness / default_agent / default_model. Retained, not dropped: column '
  'retirement is deferred to a follow-up packet after Flux confirms no reader '
  'references this column. Read default_agent instead.';

COMMENT ON COLUMN sessions.runtime IS
  'DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1. Superseded by '
  'harness / agent / model. Retained, not dropped: column retirement is deferred to a '
  'follow-up packet after Flux confirms no reader references this column. Read agent '
  'instead.';

-- =============================================================================
-- 7. Document the new columns, including the load-bearing NULL semantics.
-- =============================================================================
COMMENT ON COLUMN seats.default_harness IS
  'Process wrapper / credential / proxy layer this seat launches under. Domain: '
  'Direct | Stepcode | Custom (PascalCase, per the packet §2.1 value-domain contract). '
  'NULL means NOT RECORDED — never "direct" and never "default".';
COMMENT ON COLUMN seats.default_agent IS
  'CLI / protocol this seat drives. Domain: ClaudeCode | Codex | CursorCli | GeminiCli '
  '| Opencode | Custom (PascalCase). Nullable, matching the legacy default_runtime.';
COMMENT ON COLUMN seats.default_model IS
  'Model id this seat pins its agent to, e.g. claude-opus-5[1m]. Free-form TEXT by '
  'design — no CHECK constraint, because model ids churn faster than releases. '
  'NULL means NOT RECORDED.';

COMMENT ON COLUMN sessions.harness IS
  'Process wrapper / credential / proxy layer this session ran under. Domain: '
  'Direct | Stepcode | Custom (PascalCase). NULL means NOT RECORDED — never "direct". '
  'Historical rows predating harness capture are legitimately NULL and must not be '
  'backfilled with a guess (review §2: attribution is derived and revisable).';
COMMENT ON COLUMN sessions.agent IS
  'CLI / protocol this session drove. Domain: ClaudeCode | Codex | CursorCli | '
  'GeminiCli | Opencode | Custom (PascalCase). NOT NULL, matching the legacy runtime.';
COMMENT ON COLUMN sessions.model IS
  'Model id this session was pinned to, e.g. claude-opus-5[1m]. Free-form TEXT by '
  'design — no CHECK constraint. NULL means NOT RECORDED.';

COMMIT;

-- =============================================================================
-- VERIFICATION QUERY (ONYX.md: a migration ships with one).
-- Run after apply. The three counts MUST be 0; the DISTINCT lists must contain only
-- values inside the §2.1 domains, plus NULL.
-- =============================================================================
--
-- -- Every session must have an agent; none may have been lost in the split.
-- SELECT count(*) AS sessions_missing_agent          FROM sessions WHERE agent IS NULL;
-- SELECT count(*) AS sessions_agent_runtime_mismatch FROM sessions WHERE agent IS DISTINCT FROM runtime;
--
-- -- Seats: agent copied, harness/model untouched by the migration itself.
-- SELECT count(*) AS seats_agent_runtime_mismatch
--   FROM seats WHERE default_agent IS DISTINCT FROM default_runtime;
--
-- -- Value-domain conformance against §2.1.
-- SELECT DISTINCT agent          FROM sessions ORDER BY 1;
-- SELECT DISTINCT harness        FROM sessions ORDER BY 1;
-- SELECT DISTINCT default_agent  FROM seats    ORDER BY 1;
-- SELECT DISTINCT default_harness, default_model FROM seats ORDER BY 1, 2;
-- SELECT DISTINCT role           FROM project_role_bindings ORDER BY 1;
