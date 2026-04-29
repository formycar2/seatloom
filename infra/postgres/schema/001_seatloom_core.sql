-- SeatLoom PostgreSQL Schema v1
-- Authoritative structured collaboration truth store.
-- Evidence payloads (markdown docs) remain on disk.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- =============================================================================
-- projects
-- =============================================================================
CREATE TABLE IF NOT EXISTS projects (
    id                        TEXT        PRIMARY KEY,
    name                      TEXT        NOT NULL,
    created_at                TIMESTAMPTZ NOT NULL,
    worker_budget_tokens      INT         NOT NULL DEFAULT 8192,
    supervisor_budget_tokens  INT         NOT NULL DEFAULT 32768
);

-- =============================================================================
-- seats (global identity layer — AD-009 Layer 1)
-- =============================================================================
CREATE TABLE IF NOT EXISTS seats (
    id                TEXT        PRIMARY KEY,
    name              TEXT        NOT NULL UNIQUE,
    default_runtime   TEXT,
    capability_tags   TEXT[]      NOT NULL DEFAULT '{}',
    status            TEXT        NOT NULL DEFAULT 'active',
    created_at        TIMESTAMPTZ NOT NULL
);

-- =============================================================================
-- project_role_bindings (per-project layer — AD-009 Layer 2)
-- =============================================================================
CREATE TABLE IF NOT EXISTS project_role_bindings (
    seat_id                    TEXT   NOT NULL REFERENCES seats(id),
    project_id                 TEXT   NOT NULL,
    role                       TEXT   NOT NULL,
    authority_doc_refs         TEXT[] NOT NULL DEFAULT '{}',
    constraints                TEXT[] NOT NULL DEFAULT '{}',
    collaboration_template_ref TEXT,
    active_delegation_id       TEXT,
    PRIMARY KEY (seat_id, project_id)
);

-- =============================================================================
-- seat_delegations (scoped overlay — AD-009 Layer 3)
-- =============================================================================
CREATE TABLE IF NOT EXISTS seat_delegations (
    id                 TEXT        PRIMARY KEY,
    issuer_seat_id     TEXT        NOT NULL REFERENCES seats(id),
    from_seat_id       TEXT        NOT NULL REFERENCES seats(id),
    to_seat_id         TEXT        NOT NULL REFERENCES seats(id),
    workitem_id        TEXT,
    scope_description  TEXT        NOT NULL,
    issued_at          TIMESTAMPTZ NOT NULL,
    expires_at         TIMESTAMPTZ,
    status             TEXT        NOT NULL DEFAULT 'active'
);

-- =============================================================================
-- sessions
-- =============================================================================
CREATE TABLE IF NOT EXISTS sessions (
    id                   TEXT        PRIMARY KEY,
    seat_id              TEXT        NOT NULL REFERENCES seats(id),
    runtime              TEXT        NOT NULL,
    native_session_id    TEXT,
    workspace_path       TEXT        NOT NULL,
    branch               TEXT,
    status               TEXT        NOT NULL,
    launch_pack_ref      TEXT,
    last_checkpoint_id   TEXT,
    pid                  INT,
    created_at           TIMESTAMPTZ NOT NULL,
    ended_at             TIMESTAMPTZ
);

-- =============================================================================
-- workitems
-- =============================================================================
CREATE TABLE IF NOT EXISTS workitems (
    id                   TEXT        PRIMARY KEY,
    title                TEXT        NOT NULL,
    goal                 TEXT,
    acceptance_criteria  TEXT[]      NOT NULL DEFAULT '{}',
    owner_seat_id        TEXT        REFERENCES seats(id),
    status               TEXT        NOT NULL,
    priority             TEXT        NOT NULL DEFAULT 'medium',
    created_at           TIMESTAMPTZ NOT NULL,
    updated_at           TIMESTAMPTZ NOT NULL
);

-- =============================================================================
-- handoffs
-- =============================================================================
CREATE TABLE IF NOT EXISTS handoffs (
    id               TEXT        PRIMARY KEY,
    from_ref         TEXT        NOT NULL,
    to_ref           TEXT        NOT NULL,
    workitem_id      TEXT        NOT NULL REFERENCES workitems(id),
    purpose          TEXT        NOT NULL,
    expected_outcome TEXT        NOT NULL,
    required_receipt BOOL        NOT NULL DEFAULT false,
    status           TEXT        NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL,
    sent_at          TIMESTAMPTZ
);

-- =============================================================================
-- artifacts (metadata only — body/payload stays on disk)
-- template + subtype follow DOCUMENT_TEMPLATES.md §11.1 (AD-008)
-- =============================================================================
CREATE TABLE IF NOT EXISTS artifacts (
    id                  TEXT        PRIMARY KEY,
    template            TEXT,
    subtype             TEXT,
    subtype_valid       BOOL,
    system_kind         TEXT,
    title               TEXT        NOT NULL,
    summary             TEXT,
    source_session_id   TEXT        REFERENCES sessions(id),
    source_workitem_id  TEXT        REFERENCES workitems(id),
    storage_path        TEXT        NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL
);

-- =============================================================================
-- canonical_events (append-only ledger — AD-005)
-- =============================================================================
CREATE TABLE IF NOT EXISTS canonical_events (
    id           TEXT        PRIMARY KEY,
    event_type   TEXT        NOT NULL,
    occurred_at  TIMESTAMPTZ NOT NULL,
    actor_ref    TEXT        NOT NULL,
    payload      JSONB,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS event_object_refs (
    event_id  TEXT NOT NULL REFERENCES canonical_events(id),
    ref_type  TEXT NOT NULL,
    ref_id    TEXT NOT NULL,
    PRIMARY KEY (event_id, ref_type, ref_id)
);

-- =============================================================================
-- Indexes for common query patterns
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_sessions_seat      ON sessions(seat_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status    ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_workitems_owner    ON workitems(owner_seat_id);
CREATE INDEX IF NOT EXISTS idx_workitems_status   ON workitems(status);
CREATE INDEX IF NOT EXISTS idx_handoffs_workitem  ON handoffs(workitem_id);
CREATE INDEX IF NOT EXISTS idx_artifacts_template ON artifacts(template);
CREATE INDEX IF NOT EXISTS idx_artifacts_workitem ON artifacts(source_workitem_id);
CREATE INDEX IF NOT EXISTS idx_events_type        ON canonical_events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_occurred    ON canonical_events(occurred_at DESC);
