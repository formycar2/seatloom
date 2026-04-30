-- SeatLoom PostgreSQL Schema 005: Prompt + Channel Action Authority
-- Closes the remaining P0 authority gaps identified in the Lyra runtime-authority
-- gap review (LYRA-2026-04-30-postgres-runtime-authority-gap-review-v1):
--
--   1. prompt_instances  — first-class lifecycle row per detected blocked prompt
--   2. prompt_actions    — durable resolution trail for every prompt decision
--   3. channel_action_receipts — canonical receipt for state-changing desktop/
--                                mobile/supervisor actions across all target families
--
-- Contract pressure: US-P0-11, US-P0-13, US-P0-14, US-P0-15, INT-16, INT-18,
--                   INT-19, INT-20, E-10, E-11 (PRD v0.5 / interaction-spec v1.1)
--
-- PostgreSQL is the canonical truth store for these objects.
-- No raw terminal transcript may become the authority surface.
-- Seed 004 is intentionally empty: no real prompt/mobile action rows exist in
-- the collaboration record at baseline. Tests use in-test fixtures only.

-- =============================================================================
-- prompt_instances — one row per detected blocked prompt
-- Supports "what prompt is blocking this session right now?" with one row.
-- History does not depend on rereading terminal logs.
-- =============================================================================
CREATE TABLE IF NOT EXISTS prompt_instances (
    id                    TEXT        PRIMARY KEY,
    project_id            TEXT        NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    session_id            TEXT        NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    status                TEXT        NOT NULL DEFAULT 'active' CHECK (
                              status IN ('active', 'resolved', 'stopped', 'expired', 'superseded')
                          ),
    prompt_kind           TEXT        NOT NULL CHECK (
                              prompt_kind IN ('deterministic', 'wizard_menu', 'freeform', 'sensitive')
                          ),
    prompt_policy         TEXT        NOT NULL CHECK (
                              prompt_policy IN ('auto_allowed', 'needs_approval', 'human_required')
                          ),
    -- Bounded evidence: reference token or short preview only; no raw terminal blob
    evidence_ref          TEXT,
    evidence_preview      TEXT,
    available_actions     TEXT[]      NOT NULL DEFAULT '{}',
    -- Supervisor-assist budget (NULL = not supervisor-assisted)
    assist_max_steps      INT,
    assist_max_tokens     INT,
    assist_steps_used     INT         NOT NULL DEFAULT 0,
    assist_tokens_used    INT         NOT NULL DEFAULT 0,
    -- Expected terminal pattern after resolution (for halt-on-mismatch)
    expected_next_pattern TEXT,
    detected_at           TIMESTAMPTZ NOT NULL,
    resolved_at           TIMESTAMPTZ,
    resolved_by           TEXT,
    result_event_id       TEXT        REFERENCES canonical_events(id) ON DELETE SET NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- prompt_actions — resolution trail for prompt decisions (INT-16 audit surface)
-- Sensitive prompts remain distinguishable; budget usage is durable.
-- =============================================================================
CREATE TABLE IF NOT EXISTS prompt_actions (
    id                  TEXT        PRIMARY KEY,
    prompt_id           TEXT        NOT NULL REFERENCES prompt_instances(id) ON DELETE CASCADE,
    action_kind         TEXT        NOT NULL CHECK (
                            action_kind IN (
                                'approve', 'human_takeover', 'supervisor_assist',
                                'stop', 'input_injected', 'auto_completed'
                            )
                        ),
    actor_ref           TEXT        NOT NULL,
    source_channel      TEXT        NOT NULL DEFAULT 'desktop' CHECK (
                            source_channel IN ('desktop', 'mobile', 'supervisor', 'system')
                        ),
    -- Bounded note or injected input preview; never unbounded transcript
    note                TEXT,
    -- Assist budget snapshot at time of this action
    steps_budget_used   INT,
    tokens_budget_used  INT,
    result_status       TEXT        NOT NULL DEFAULT 'applied' CHECK (
                            result_status IN ('applied', 'rejected', 'stopped', 'conflict')
                        ),
    result_event_id     TEXT        REFERENCES canonical_events(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- channel_action_receipts — canonical receipt for state-changing actions
-- Covers desktop / mobile / supervisor approval, reject, escalate, takeover, etc.
-- This is the storage-edge contract future UI actions write before domain mutation.
-- Unified across all target families so the audit surface stays single-canonical.
-- =============================================================================
CREATE TABLE IF NOT EXISTS channel_action_receipts (
    id                TEXT        PRIMARY KEY,
    project_id        TEXT        NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    target_kind       TEXT        NOT NULL CHECK (
                          target_kind IN (
                              'workitem', 'handoff', 'review_thread', 'prompt',
                              'session', 'artifact', 'document', 'project'
                          )
                      ),
    target_id         TEXT        NOT NULL,
    action_kind       TEXT        NOT NULL CHECK (
                          action_kind IN (
                              'approve', 'reject', 'escalate',
                              'reserve_desktop_takeover', 'return',
                              'comment_submit', 'stop'
                          )
                      ),
    actor_ref         TEXT        NOT NULL,
    source_channel    TEXT        NOT NULL DEFAULT 'desktop' CHECK (
                          source_channel IN ('desktop', 'mobile', 'supervisor', 'system')
                      ),
    note              TEXT,
    evidence_refs     TEXT[]      NOT NULL DEFAULT '{}',
    policy_summary    TEXT,
    -- Idempotency: prevents duplicate action delivery
    idempotency_key   TEXT        NOT NULL UNIQUE,
    -- Optimistic write guard: caller declares expected revision; DB records applied
    expected_revision INT,
    applied_revision  INT,
    receipt_status    TEXT        NOT NULL DEFAULT 'applied' CHECK (
                          receipt_status IN ('applied', 'rejected', 'conflicted', 'noop')
                      ),
    result_event_id   TEXT        REFERENCES canonical_events(id) ON DELETE SET NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- Indexes
-- =============================================================================
CREATE INDEX IF NOT EXISTS idx_prompt_instances_session  ON prompt_instances(session_id, detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_prompt_instances_status   ON prompt_instances(status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_prompt_instances_project  ON prompt_instances(project_id, detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_prompt_instances_event    ON prompt_instances(result_event_id);
CREATE INDEX IF NOT EXISTS idx_prompt_actions_prompt     ON prompt_actions(prompt_id, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_prompt_actions_channel    ON prompt_actions(source_channel);
CREATE INDEX IF NOT EXISTS idx_prompt_actions_actor      ON prompt_actions(actor_ref);
CREATE INDEX IF NOT EXISTS idx_channel_action_target     ON channel_action_receipts(target_kind, target_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_channel_action_project    ON channel_action_receipts(project_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_channel_action_channel    ON channel_action_receipts(source_channel);
CREATE INDEX IF NOT EXISTS idx_channel_action_status     ON channel_action_receipts(receipt_status);
CREATE INDEX IF NOT EXISTS idx_channel_action_actor      ON channel_action_receipts(actor_ref);
