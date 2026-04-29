-- SeatLoom PostgreSQL Schema 003: Write/Ingest/Reconcile Bookkeeping
-- Implements the MVP reconcile contract from AD-007:
--   triggers = { startup, pre_pipeline, manual }
--   no file watcher, no background daemon
--   markdown is an authoring input surface; PostgreSQL is the canonical truth store
--
-- This schema closes the steady-state write path:
--   markdown authored in repo → seatloom reconcile → PostgreSQL authority

-- =============================================================================
-- reconcile_runs — one row per reconcile invocation
-- Records trigger type, run status, and aggregate outcome counts.
-- =============================================================================
CREATE TABLE IF NOT EXISTS reconcile_runs (
    id              TEXT        PRIMARY KEY,    -- 'run-<nanoid>'
    trigger         TEXT        NOT NULL,       -- 'startup' | 'pre_pipeline' | 'manual'
    status          TEXT        NOT NULL        -- 'running' | 'completed' | 'failed'
                    DEFAULT 'running',
    scanned         INT         NOT NULL DEFAULT 0,
    inserted        INT         NOT NULL DEFAULT 0,
    updated         INT         NOT NULL DEFAULT 0,
    unchanged       INT         NOT NULL DEFAULT 0,
    failed          INT         NOT NULL DEFAULT 0,
    conflicted      INT         NOT NULL DEFAULT 0,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ
);

-- =============================================================================
-- reconcile_items — per-file result within a reconcile run
-- Records individual file outcomes: inserted / updated / unchanged / failed / conflict.
-- =============================================================================
CREATE TABLE IF NOT EXISTS reconcile_items (
    id              TEXT        PRIMARY KEY,    -- 'ri-<nanoid>'
    run_id          TEXT        NOT NULL REFERENCES reconcile_runs(id) ON DELETE CASCADE,
    file_path       TEXT        NOT NULL,       -- repo-relative path
    document_id     TEXT        REFERENCES documents(id) ON DELETE SET NULL,
    outcome         TEXT        NOT NULL,       -- 'inserted'|'updated'|'unchanged'|'failed'|'conflict'
    digest_before   TEXT,                       -- body_digest before this run (NULL if new)
    digest_after    TEXT,                       -- body_digest after this run
    revision_before INT,                        -- document.revision before (NULL if new)
    revision_after  INT,                        -- document.revision after
    parse_status    TEXT,
    failure_reason  TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- document_versions — durable revision snapshots for changed documents
-- Created only when document body/header actually changes between ingest runs.
-- Unchanged content never creates a new version row.
-- =============================================================================
CREATE TABLE IF NOT EXISTS document_versions (
    id              TEXT        PRIMARY KEY,    -- 'dv-<nanoid>'
    document_id     TEXT        NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    revision        INT         NOT NULL,       -- matches document.revision at time of snapshot
    body_digest     TEXT        NOT NULL,
    body_text       TEXT,                       -- full body at this revision
    header_snapshot JSONB,                      -- parsed header fields at this revision
    run_id          TEXT        REFERENCES reconcile_runs(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (document_id, revision)
);

-- =============================================================================
-- Indexes for reconcile tables
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_reconcile_runs_started   ON reconcile_runs(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_reconcile_runs_trigger   ON reconcile_runs(trigger);
CREATE INDEX IF NOT EXISTS idx_reconcile_items_run      ON reconcile_items(run_id);
CREATE INDEX IF NOT EXISTS idx_reconcile_items_doc      ON reconcile_items(document_id);
CREATE INDEX IF NOT EXISTS idx_reconcile_items_outcome  ON reconcile_items(outcome);
CREATE INDEX IF NOT EXISTS idx_doc_versions_doc         ON document_versions(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_versions_run         ON document_versions(run_id);
