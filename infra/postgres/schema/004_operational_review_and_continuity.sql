-- SeatLoom PostgreSQL Schema 004: Operational review + continuity authority
-- Extends schemas 001-003 with the missing first-class operational objects
-- required by the v0.5 contract set:
--   - checkpoints for continuity recovery and context resumption
--   - handoff_receipts for receipt audit and inbox/timeline proof
--   - pipeline_runs for deterministic verification/execution history
--   - review_threads + review_comments for the shared desktop/mobile feedback model
--
-- PostgreSQL remains the canonical truth store for these objects.
-- Markdown files and runtime transcripts remain evidence/input surfaces only.

-- =============================================================================
-- checkpoints — structured continuity snapshots
-- Supports PRD/INT/UX continuity preview, session restore, and delta recovery.
-- Tier 0/1/2 JSON fields are intentionally flexible at the storage edge.
-- =============================================================================
CREATE TABLE IF NOT EXISTS checkpoints (
    id                          TEXT        PRIMARY KEY,
    session_id                  TEXT        NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    trigger                     TEXT        NOT NULL CHECK (
                                    trigger IN ('session_ended', 'artifact_produced', 'manual')
                                ),
    summary_what_was_done       TEXT        NOT NULL,
    summary_current_state       TEXT        NOT NULL,
    summary_open_questions      TEXT[]      NOT NULL DEFAULT '{}',
    summary_quality             TEXT        NOT NULL CHECK (
                                    summary_quality IN ('full', 'minimal')
                                ),
    artifact_ids_at_checkpoint  TEXT[]      NOT NULL DEFAULT '{}',
    branch                      TEXT,
    last_commit                 TEXT,
    transcript_tail_ref         TEXT,
    continuity_tier0            JSONB,
    continuity_tier1            JSONB,
    continuity_tier2            JSONB,
    continuity_budget_tokens    INT,
    delta_context               JSONB,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- handoff_receipts — explicit acknowledgment object for required receipts
-- The receipt object is separate from the handoff row so it can be audited,
-- listed, and eventually projected into Timeline/Inbox/Detail.
-- =============================================================================
CREATE TABLE IF NOT EXISTS handoff_receipts (
    id               TEXT        PRIMARY KEY,
    handoff_id       TEXT        NOT NULL REFERENCES handoffs(id) ON DELETE CASCADE,
    acknowledged_by  TEXT        NOT NULL REFERENCES seats(id) ON DELETE RESTRICT,
    acknowledged_at  TIMESTAMPTZ NOT NULL,
    note             TEXT,
    source_channel   TEXT        NOT NULL DEFAULT 'desktop' CHECK (
                         source_channel IN ('desktop', 'mobile', 'supervisor')
                     ),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (handoff_id, acknowledged_by, acknowledged_at)
);

-- =============================================================================
-- pipeline_runs — deterministic execution / verification run history
-- This is infrastructure-only history for foreground/manual execution today.
-- =============================================================================
CREATE TABLE IF NOT EXISTS pipeline_runs (
    id               TEXT        PRIMARY KEY,
    pipeline_id      TEXT        NOT NULL,
    workitem_id      TEXT        REFERENCES workitems(id) ON DELETE SET NULL,
    status           TEXT        NOT NULL CHECK (
                         status IN ('running', 'completed', 'failed', 'aborted')
                     ),
    current_stage    TEXT,
    stage_index      INT,
    trigger          TEXT        NOT NULL DEFAULT 'manual' CHECK (
                         trigger IN ('manual', 'handoff', 'gate', 'scheduled')
                     ),
    initiated_by     TEXT,
    result_summary   TEXT,
    artifact_ids     TEXT[]      NOT NULL DEFAULT '{}',
    evidence_refs    TEXT[]      NOT NULL DEFAULT '{}',
    run_metadata     JSONB,
    started_at       TIMESTAMPTZ NOT NULL,
    finished_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- review_threads — shared thread object for desktop review and mobile feedback
-- Manual comments, supervisor-assisted comments, and mobile feedback all enter
-- the same canonical family so downstream routing/audit stays unified.
-- =============================================================================
CREATE TABLE IF NOT EXISTS review_threads (
    id                  TEXT        PRIMARY KEY,
    project_id          TEXT        NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    source_channel      TEXT        NOT NULL DEFAULT 'desktop' CHECK (
                            source_channel IN ('desktop', 'mobile', 'supervisor')
                        ),
    mode                TEXT        NOT NULL DEFAULT 'manual' CHECK (
                            mode IN ('manual', 'supervisor_assisted')
                        ),
    target_kind         TEXT        NOT NULL CHECK (
                            target_kind IN ('artifact', 'document', 'workitem', 'handoff', 'session', 'project')
                        ),
    target_id           TEXT        NOT NULL,
    document_id         TEXT        REFERENCES documents(id) ON DELETE SET NULL,
    artifact_id         TEXT        REFERENCES artifacts(id) ON DELETE SET NULL,
    workitem_id         TEXT        REFERENCES workitems(id) ON DELETE SET NULL,
    handoff_id          TEXT        REFERENCES handoffs(id) ON DELETE SET NULL,
    session_id          TEXT        REFERENCES sessions(id) ON DELETE SET NULL,
    anchor_kind         TEXT        NOT NULL DEFAULT 'object' CHECK (
                            anchor_kind IN ('document', 'section', 'text_range', 'object')
                        ),
    anchor_ref          TEXT,
    anchor_label        TEXT,
    title               TEXT,
    status              TEXT        NOT NULL DEFAULT 'open' CHECK (
                            status IN ('open', 'resolved', 'disputed', 'closed')
                        ),
    requires_followup   BOOL        NOT NULL DEFAULT false,
    review_tier         TEXT        CHECK (review_tier IN ('L1', 'L2', 'L3')),
    change_tier_record  JSONB,
    evidence_refs       TEXT[]      NOT NULL DEFAULT '{}',
    created_by          TEXT        NOT NULL,
    assigned_to         TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at         TIMESTAMPTZ,
    resolved_by         TEXT
);

-- =============================================================================
-- review_comments — ordered entries within a review thread
-- Supports replies plus the same canonical object family for short mobile feedback.
-- =============================================================================
CREATE TABLE IF NOT EXISTS review_comments (
    id                TEXT        PRIMARY KEY,
    thread_id         TEXT        NOT NULL REFERENCES review_threads(id) ON DELETE CASCADE,
    parent_comment_id TEXT        REFERENCES review_comments(id) ON DELETE CASCADE,
    author_ref        TEXT        NOT NULL,
    body_text         TEXT        NOT NULL,
    mode              TEXT        NOT NULL DEFAULT 'manual' CHECK (
                          mode IN ('manual', 'supervisor_assisted')
                      ),
    source_channel    TEXT        NOT NULL DEFAULT 'desktop' CHECK (
                          source_channel IN ('desktop', 'mobile', 'supervisor')
                      ),
    state             TEXT        NOT NULL DEFAULT 'active' CHECK (
                          state IN ('active', 'resolved', 'disputed', 'retracted')
                      ),
    evidence_refs     TEXT[]      NOT NULL DEFAULT '{}',
    comment_metadata  JSONB,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- Indexes
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_checkpoints_session       ON checkpoints(session_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_checkpoints_created       ON checkpoints(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_handoff_receipts_handoff  ON handoff_receipts(handoff_id, acknowledged_at DESC);
CREATE INDEX IF NOT EXISTS idx_handoff_receipts_actor    ON handoff_receipts(acknowledged_by);
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_workitem    ON pipeline_runs(workitem_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_pipeline    ON pipeline_runs(pipeline_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_pipeline_runs_status      ON pipeline_runs(status);
CREATE INDEX IF NOT EXISTS idx_review_threads_target     ON review_threads(target_kind, target_id);
CREATE INDEX IF NOT EXISTS idx_review_threads_document   ON review_threads(document_id);
CREATE INDEX IF NOT EXISTS idx_review_threads_artifact   ON review_threads(artifact_id);
CREATE INDEX IF NOT EXISTS idx_review_threads_workitem   ON review_threads(workitem_id);
CREATE INDEX IF NOT EXISTS idx_review_threads_handoff    ON review_threads(handoff_id);
CREATE INDEX IF NOT EXISTS idx_review_threads_session    ON review_threads(session_id);
CREATE INDEX IF NOT EXISTS idx_review_threads_status     ON review_threads(status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_review_threads_project    ON review_threads(project_id, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_review_threads_channel    ON review_threads(source_channel);
CREATE INDEX IF NOT EXISTS idx_review_comments_thread    ON review_comments(thread_id, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_review_comments_parent    ON review_comments(parent_comment_id);
CREATE INDEX IF NOT EXISTS idx_review_comments_author    ON review_comments(author_ref);
CREATE INDEX IF NOT EXISTS idx_review_threads_change_tier ON review_threads USING gin (change_tier_record);
