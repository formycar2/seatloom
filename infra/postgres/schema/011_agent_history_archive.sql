-- 011_agent_history_archive.sql
-- =============================================================================
-- Agent-history archive index (P2 historical ingestion, pulled forward 2026-10-08
-- for the Claude-Code→PI compliance migration). Registers the harness-agnostic
-- session corpus archived under ~/Documents/GitHub/AgentHistory into SeatLoom's
-- record so it is queryable + traceable WITHOUT depending on any harness.
--
-- Self-contained by design: NO FK to seats (archived sessions have no seat) and
-- NO FK to projects (the corpus spans ~21 projects not in the projects table).
-- The (harness, id) key is the external_ref shape — (harness, native_id) → object —
-- so these rows link cleanly once G4 external_refs lands.
--
-- Additive, post-seed, idempotent (CREATE ... IF NOT EXISTS; re-apply is a no-op).
-- Raw bytes live in the content-addressed archive on disk (sha256); only the
-- INDEX/metadata lives here. Never store transcript bodies (may contain secrets).
-- =============================================================================

CREATE TABLE IF NOT EXISTS archived_sessions (
    id            TEXT        NOT NULL,          -- native session id (per harness)
    harness       TEXT        NOT NULL,          -- codex | claude-code | stepcode | cursor | gemini | opencode
    project       TEXT,                          -- project slug (no FK; archive-only projects)
    cwd           TEXT,                          -- original working path
    session_date  TEXT,                          -- last-activity (manifest 'date')
    bytes         BIGINT,
    lines         INTEGER,
    sha256        TEXT,                          -- integrity digest; NULL for dir-copies (cursor/gemini/opencode)
    raw_path      TEXT        NOT NULL,          -- path within AgentHistory/
    src_path      TEXT,                          -- original source path on disk
    archived_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (harness, id)
);

CREATE INDEX IF NOT EXISTS idx_archived_sessions_project ON archived_sessions (project);
CREATE INDEX IF NOT EXISTS idx_archived_sessions_sha     ON archived_sessions (sha256);
CREATE INDEX IF NOT EXISTS idx_archived_sessions_harness ON archived_sessions (harness);
