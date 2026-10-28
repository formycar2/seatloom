-- 015_artifacts_project_id.sql — add `project_id` to `artifacts` so the Artifacts
-- tab can be scoped per project like every other module.
--
-- 🔴 THIS IS A LIVE MIGRATION. IT IS THE OPERATOR'S ACT, NOT A SEAT'S.
--    Nimbus measured the need and wrote this file; he did NOT run it and must not.
--    Run it only if you want artifacts partitioned per project.
--
-- WHY IT IS NEEDED. `artifacts` is one of only two tables without `project_id`
-- (the other is `seats`, which is global by design and reaches its project through
-- `project_role_bindings`). Measured on live before this file was written:
--
--   SELECT count(*)                                                    -> 15
--   SELECT count(*) WHERE source_session_id  IS NOT NULL               ->  0
--   SELECT count(*) WHERE source_workitem_id IS NOT NULL               ->  4
--   SELECT count(*) WHERE source_session_id IS NULL
--                      AND source_workitem_id IS NULL                  -> 11
--
-- So a DERIVED join would attribute 4 of 15 rows and silently drop 11. That is why
-- the shipped behaviour refuses to pretend rather than joining: a short list that
-- looks filtered is worse than an honest unscoped one.
--
-- WHAT THIS DOES. Adds the column, nullable, with NO backfill. Null means
-- "not recorded" — the same convention the rest of the schema uses for an absent
-- value, and the honest state for the 11 rows that cannot be derived. A backfill
-- that guessed would be a lie with a convenient shape, so there is none.
--
-- The two FKs are commented out rather than applied: `artifacts` currently has no
-- FK to `sessions` or `workitems` either, and adding one here would change
-- insert-time behaviour for the write path, which is out of this migration's scope.
-- Apply them in a separate migration if you want referential integrity.
--
-- AFTER RUNNING IT. The command layer and the UI still treat artifacts as
-- unscoped until the corresponding code lands. This file only makes the column
-- exist; it does not change what the app shows.

BEGIN;

ALTER TABLE artifacts
    ADD COLUMN IF NOT EXISTS project_id TEXT;

-- Index for the per-project read the command will run once it exists.
CREATE INDEX IF NOT EXISTS idx_artifacts_project_id
    ON artifacts (project_id);

-- Optional, and deliberately NOT applied here — see the note above.
-- ALTER TABLE artifacts
--     ADD CONSTRAINT artifacts_project_fk
--     FOREIGN KEY (project_id) REFERENCES projects (id);
--
-- ALTER TABLE artifacts
--     ADD CONSTRAINT artifacts_session_fk
--     FOREIGN KEY (source_session_id) REFERENCES sessions (id);
--
-- ALTER TABLE artifacts
--     ADD CONSTRAINT artifacts_workitem_fk
--     FOREIGN KEY (source_workitem_id) REFERENCES workitems (id);

COMMENT ON COLUMN artifacts.project_id IS
    'Project this artifact belongs to. NULL = not recorded (11 of the 15 pre-migration rows have neither source_session_id nor source_workitem_id, so no project can be derived for them). Added by 015 so the Artifacts tab can be scoped per project.';

COMMIT;

-- Verify after running:
--   SELECT count(*) AS total,
--          count(project_id) AS with_project,
--          count(*) - count(project_id) AS not_recorded
--   FROM artifacts;
-- Expect: 15 / 0 / 15 immediately after the migration (no backfill), and the
-- `with_project` count rising only as new artifacts are written with a project.
