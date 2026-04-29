-- SeatLoom PostgreSQL Schema 002: Document Authority Layer
-- Extends 001_seatloom_core.sql with typed markdown document persistence.
-- PostgreSQL is the canonical structured truth store for all SeatLoom project data.
-- File-backed stores (.seatloom/) are cache/export artifacts, not peer authority.
--
-- Authority direction: PostgreSQL → FTS → semantic (L1 → L2 → L3 → LLM explanation).
-- SQLite FTS5 is superseded by PostgreSQL structured + full-text retrieval.

-- Enable pg_trgm for lightweight full-text search if available
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- =============================================================================
-- documents — typed markdown coordination documents (universal header + body)
-- DOCUMENT_TEMPLATES.md §2 universal header fields persisted structurally.
-- body_text contains the full markdown body; authoritative in PostgreSQL.
-- =============================================================================
CREATE TABLE IF NOT EXISTS documents (
    id              TEXT        PRIMARY KEY,     -- unique row id (e.g. 'doc-prd-v05')
    project_id      TEXT        NOT NULL,        -- owning project
    artifact_id     TEXT        REFERENCES artifacts(id) ON DELETE SET NULL,
    -- Universal header fields (DOCUMENT_TEMPLATES.md §2)
    template        TEXT,                        -- T1AuthorityDoc ... T7GovernanceDoc
    subtype         TEXT,                        -- prd | task | fix | acceptance_review | ...
    subtype_valid   BOOL,                        -- NULL=unvalidated; false=degraded
    doc_id          TEXT,                        -- id field from document header
    title           TEXT        NOT NULL,
    status          TEXT,                        -- draft | approved | active | superseded
    author          TEXT,                        -- seat name or 'human'
    doc_date        DATE,
    version         TEXT,
    depends_on      TEXT[]      NOT NULL DEFAULT '{}',
    supersedes      TEXT,
    tags            TEXT[]      NOT NULL DEFAULT '{}',
    -- Storage + body
    file_path       TEXT        NOT NULL,        -- repo-relative path to source file
    body_text       TEXT,                        -- full markdown body (PostgreSQL-authoritative)
    body_digest     TEXT,                        -- SHA-256 hex of body_text for change detection
    body_length     INT,
    parse_status    TEXT        NOT NULL DEFAULT 'parsed',  -- parsed | partial | failed
    -- Optimistic concurrency
    revision        INT         NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- document_sections — deterministic heading / anchor projection
-- Enables future artifact review, search result grouping, and comment anchoring.
-- =============================================================================
CREATE TABLE IF NOT EXISTS document_sections (
    id              TEXT        PRIMARY KEY,     -- 'sec-<doc_id>-<ordinal>'
    document_id     TEXT        NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    ordinal         INT         NOT NULL,        -- 1-based heading order in document
    heading_text    TEXT        NOT NULL,        -- verbatim heading text
    heading_level   INT         NOT NULL,        -- 1=H1, 2=H2, ...
    anchor_slug     TEXT        NOT NULL,        -- slugified anchor for in-doc linking
    body_excerpt    TEXT,                        -- first 512 chars of section body
    search_text     TEXT,                        -- normalized searchable text (heading + excerpt)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (document_id, ordinal)
);

-- =============================================================================
-- document_associations — many-to-many: document ↔ project objects
-- Supports WorkItem / Session / Handoff / project-scope association.
-- =============================================================================
CREATE TABLE IF NOT EXISTS document_associations (
    id              TEXT        PRIMARY KEY,
    document_id     TEXT        NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    assoc_type      TEXT        NOT NULL,    -- 'workitem' | 'session' | 'handoff' | 'project'
    assoc_id        TEXT        NOT NULL,    -- id of the associated object
    is_primary      BOOL        NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (document_id, assoc_type, assoc_id)
);

-- =============================================================================
-- Indexes for document authority tables
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_documents_project    ON documents(project_id);
CREATE INDEX IF NOT EXISTS idx_documents_template   ON documents(template);
CREATE INDEX IF NOT EXISTS idx_documents_subtype    ON documents(subtype);
CREATE INDEX IF NOT EXISTS idx_documents_status     ON documents(status);
CREATE INDEX IF NOT EXISTS idx_documents_artifact   ON documents(artifact_id);
CREATE INDEX IF NOT EXISTS idx_documents_updated    ON documents(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_doc_sections_doc     ON document_sections(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_sections_slug    ON document_sections(anchor_slug);
CREATE INDEX IF NOT EXISTS idx_doc_assoc_doc        ON document_associations(document_id);
CREATE INDEX IF NOT EXISTS idx_doc_assoc_target     ON document_associations(assoc_type, assoc_id);

-- Full-text search index on document body and title (PostgreSQL FTS, not SQLite)
CREATE INDEX IF NOT EXISTS idx_documents_fts ON documents
    USING gin(to_tsvector('english', coalesce(title, '') || ' ' || coalesce(body_text, '')));

CREATE INDEX IF NOT EXISTS idx_doc_sections_fts ON document_sections
    USING gin(to_tsvector('english', coalesce(search_text, '')));
