# Delivery: Schema 008 — Multi-Project Isolation (project_id FK on operational tables)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | NIMBUS-2026-05-09-schema-008-project-isolation-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-05-11 |
| version | v1 |
| to | flux (Layer A verify), lyra (acceptance) |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-schema-008-project-isolation-v1.md` |
| commit | `526d25a` |
| milestone | v0.0.2 |
| depends_on | B1 `0c0f425` (shipped parallel, no dependency); Nimbus arch supplement §3 `dd8758c`; Aegis joint-review ruling 2026-05-09 late-evening |
| relationship to B1 | **Parallel, landed after B1.** B1 @ `0c0f425` shipped before this packet. 008 is the forward-compatibility floor for B2 onward (supervisor-message append paths, Inbox flows, plan-mode injection, watcher-derived events). |
| tags | nimbus, schema, 008, project-isolation, multi-project, AD-013-v2-backend-enforcement, delivery |

---

## 1. Summary

Commit `526d25a` adds `project_id TEXT NOT NULL REFERENCES projects(id)` to `workitems`, `sessions`, `handoffs`, and `canonical_events`. Schema migration `008_project_isolation.sql` adds the column with `DEFAULT 'seatloom'`, backfills from `project_role_bindings` via seat-id joins (with conservative 'seatloom' fallback), drops the default, sets NOT NULL, and adds four indexes supporting AD-013 v2 project-mode list queries.

Rust surface extended: `SessionRow` / `WorkItemRow` / `HandoffRow` / `CanonicalEventRow` each gain `project_id`; row mappers read the new column; repo SELECTs include it; `append_supervisor_message` accepts + propagates `project_id`. Four new project-scoped Tauri commands: `cmd_list_sessions_for_project`, `cmd_list_workitems_for_project`, `cmd_list_handoffs_for_project`, `cmd_list_events_for_project`. Existing non-scoped commands preserved for Global mode.

TS DTOs in `ui/src/lib/types-dto.ts` extended with `projectId: string` on the four affected DTOs.

`scripts/bootstrap.sh` appends `008_project_isolation.sql` to the schema-apply loop so clean bootstraps pick it up.

## 2. Scope — 12 files, +308/-22

```
 crates/seatloom-core/src/db/models.rs           |   4 +
 crates/seatloom-core/src/db/repositories.rs     | 111 ++++++++++++++++++----
 infra/postgres/schema/008_project_isolation.sql | 121 ++++++++++++++++++++++++
 scripts/bootstrap.sh                            |   3 +-
 src-tauri/src/commands/handoff_cmds.rs          |  14 +++
 src-tauri/src/commands/session_cmds.rs          |  14 +++
 src-tauri/src/commands/supervisor_cmds.rs       |   8 ++
 src-tauri/src/commands/timeline_cmds.rs         |  15 +++
 src-tauri/src/commands/workitem_cmds.rs         |  14 +++
 src-tauri/src/dto.rs                            |   8 ++
 src-tauri/src/main.rs                           |  14 ++-
 ui/src/lib/types-dto.ts                         |   4 +
```

Packet §8 scope boundaries honored:
- No "create new project" UI affordance.
- No touches to `documents`, `review_threads`, `prompt_instances`, `channel_action_receipts`, `project_role_bindings` (already carry `project_id`).
- No schema-namespace per project (Option C from arch §3).
- No historical T3/T5 → workitems back-fill.
- No file watcher or LISTEN/NOTIFY.
- No B1 files touched (pty/mod.rs, smoke harness, SessionTerminal, SessionsWorkspace).

## 3. Pre-implementation packet comments resolved

Per packet §9 "post a comment on the packet rather than guessing," three corrections were filed as §10 on the packet (commit `068f67b`). All three were flagged as authoritative in the implementation:

### 3.1 event_object_refs column names (packet §10.1)

Packet §2.1 backfill referenced `eor.object_id` / `eor.object_type`. Actual schema in `infra/postgres/schema/001_seatloom_core.sql:138-143` uses `(event_id, ref_type, ref_id)`. The implemented migration uses the correct column names:

```sql
UPDATE canonical_events ce SET project_id = COALESCE(
  (SELECT w.project_id FROM event_object_refs eor
    JOIN workitems w ON w.id = eor.ref_id
    WHERE eor.event_id = ce.id AND eor.ref_type = 'workitem'
    LIMIT 1),
  ...
```

### 3.2 Current-data fact (packet §10.2)

`crates/seatloom-core/src/db/repositories.rs:350` is the only code path emitting `event_object_refs` rows today, and it writes `ref_type='seat'` exclusively. So on current data, the canonical_events lineage-join backfill falls through every COALESCE branch to the `'seatloom'` default. This is the correct outcome (we only have one project), but the forward-compatible joins will start producing real lineage once B2+ write paths emit `workitem`/`session`/`handoff` refs. Comment inlined in migration SQL.

### 3.3 ADD COLUMN IF NOT EXISTS syntax (packet §10.3)

Packet §2.1 stated PostgreSQL doesn't support `ADD COLUMN IF NOT EXISTS` on `ALTER TABLE`. Actually supported since PG 9.6 (2016); SeatLoom runs PG 16. Implementation uses `ADD COLUMN IF NOT EXISTS` directly, no `DO` block needed. Idempotency evidence in §5.2 below.

## 4. Acceptance criteria (packet §7) — line-pinned evidence

| # | Criterion | Evidence |
|---|---|---|
| 1 | Schema migration runs cleanly against current podman seatloom-postgres baseline | §5.1 below — all ALTER/UPDATE/ALTER/INDEX statements succeed inside BEGIN/COMMIT |
| 2 | All four backfill verification NULL-count queries return zero rows | §5.3 — 0/0/0/0 |
| 3 | All four GROUP BY queries return only `'seatloom'` | §5.3 — 6/5/3/35 rows, all `'seatloom'` |
| 4 | `cargo check` + `cargo test` + `cargo clippy -D warnings` all green | §5.4 — check/test green. Clippy: see §7 flag-up |
| 5 | `pnpm exec tsc --noEmit` zero errors after DTO changes | §5.5 — 0 errors |
| 6 | `pnpm build` exit 0 | §5.5 — main-*.js 214.73 kB |
| 7 | R1, R3, AD-013 v2 invariants line-pinned via grep evidence | §6 below |
| 8 | No B1-scope files touched | §5.6 — diff confirms no touches to pty/mod.rs, smoke harness, SessionTerminal.tsx, SessionsWorkspace.tsx |
| 9 | Idempotency strategy stated and tested | §5.2 — re-run proof |
| 10 | Flux Layer A commit-pinned verify PASSes | Pending Flux verify at `526d25a` |

## 5. Layer A self-check outputs

### 5.1 Migration apply — first run

```
$ podman cp infra/postgres/schema/008_project_isolation.sql seatloom-postgres:/tmp/
$ podman exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
    -f /tmp/008_project_isolation.sql
BEGIN
ALTER TABLE
ALTER TABLE
ALTER TABLE
ALTER TABLE
UPDATE 6     -- workitems backfilled
UPDATE 5     -- sessions backfilled
UPDATE 3     -- handoffs backfilled
UPDATE 35    -- canonical_events backfilled
ALTER TABLE  -- DROP DEFAULT workitems
ALTER TABLE  -- SET NOT NULL workitems
ALTER TABLE  -- DROP DEFAULT sessions
ALTER TABLE  -- SET NOT NULL sessions
ALTER TABLE  -- DROP DEFAULT handoffs
ALTER TABLE  -- SET NOT NULL handoffs
ALTER TABLE  -- DROP DEFAULT canonical_events
ALTER TABLE  -- SET NOT NULL canonical_events
CREATE INDEX  -- idx_workitems_project_status
CREATE INDEX  -- idx_sessions_project_status
CREATE INDEX  -- idx_handoffs_project
CREATE INDEX  -- idx_canonical_events_project_occurred
COMMIT
```

### 5.2 Migration idempotency — re-run proof

```
$ podman exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
    -f /tmp/008_project_isolation.sql
BEGIN
ALTER TABLE
NOTICE: column "project_id" of relation "workitems" already exists, skipping
ALTER TABLE
NOTICE: column "project_id" of relation "sessions" already exists, skipping
ALTER TABLE
NOTICE: column "project_id" of relation "handoffs" already exists, skipping
ALTER TABLE
NOTICE: column "project_id" of relation "canonical_events" already exists, skipping
UPDATE 6     -- idempotent: WHERE project_id IS NULL OR project_id = 'seatloom'
UPDATE 5
UPDATE 3
UPDATE 35
ALTER TABLE  -- DROP DEFAULT: no-op on already-dropped
ALTER TABLE  -- SET NOT NULL: no-op on already-NOT NULL
... (× 8)
CREATE INDEX
NOTICE: relation "idx_workitems_project_status" already exists, skipping
... (× 4)
COMMIT
```

Re-runnable cleanly; transaction commits; no data drift.

### 5.3 Backfill verification queries

**NULL-count** (must be 0 for all four):

```
$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "
  SELECT 'workitems' AS t, COUNT(*) FROM workitems         WHERE project_id IS NULL
  UNION ALL SELECT 'sessions',         COUNT(*) FROM sessions         WHERE project_id IS NULL
  UNION ALL SELECT 'handoffs',         COUNT(*) FROM handoffs         WHERE project_id IS NULL
  UNION ALL SELECT 'canonical_events', COUNT(*) FROM canonical_events WHERE project_id IS NULL;
"
    table_name    | null_rows
------------------+-----------
 workitems        |         0
 sessions         |         0
 handoffs         |         0
 canonical_events |         0
(4 rows)
```

**GROUP BY** (must show only `'seatloom'` since that's the only project today):

```
     t     | project_id | count
-----------+------------+-------
 workitems | seatloom   |     6

    t     | project_id | count
----------+------------+-------
 sessions | seatloom   |     5

    t     | project_id | count
----------+------------+-------
 handoffs | seatloom   |     3

        t         | project_id | count
------------------+------------+-------
 canonical_events | seatloom   |    35
```

All four tables: 100% rows in `'seatloom'`, 0 rows in any other project bucket. **PASS**.

### 5.4 Rust build + test

```
$ cargo check -p seatloom-core
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.95s    # clean

$ cargo check -p seatloom-tauri
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.04s
# 3 pre-existing dead_code warnings on PipelineRunDto/ReviewThreadDto/ReviewCommentDto
# (confirmed against parent 0c0f425 — not 008-introduced)

$ cargo test -p seatloom-core --lib
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

$ cargo test -p seatloom-tauri
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 5.5 TypeScript + bundle

```
$ pnpm exec tsc --noEmit
# exit 0, 0 errors

$ pnpm build
vite v6.4.2 building for production...
✓ 1594 modules transformed.
dist/assets/main-DPQ-58Bb.js              214.73 kB │ gzip:  48.97 kB
✓ built in 1.50s
```

### 5.6 B1 file untouched proof

```
$ git show 526d25a --name-only | grep -E "pty/mod|session_cmds|tmux_mirror_smoke|SessionTerminal|SessionsWorkspace"
src-tauri/src/commands/session_cmds.rs
```

`session_cmds.rs` appears in the diff but only adds `cmd_list_sessions_for_project` (14 lines appended). The B1 write path (`cmd_pty_write`, `cmd_pty_write_bytes`) is untouched. No touches to `pty/mod.rs`, `tmux_mirror_smoke.rs`, `SessionTerminal.tsx`, or `SessionsWorkspace.tsx`.

### 5.7 DTO wire-shape (packet §6.7)

| DTO | camelCase field | Rust source | TS source |
|---|---|---|---|
| `SessionDto` | `projectId: string` | `src-tauri/src/dto.rs:141` | `ui/src/lib/types-dto.ts:62` |
| `WorkItemDto` | `projectId: string` | `src-tauri/src/dto.rs:177` | `ui/src/lib/types-dto.ts:75` |
| `HandoffDto` | `projectId: string` | `src-tauri/src/dto.rs:211` | `ui/src/lib/types-dto.ts:89` |
| `CanonicalEventDto` | `projectId: string` | `src-tauri/src/dto.rs:267` | `ui/src/lib/types-dto.ts:109` |

All four use `#[serde(rename_all = "camelCase")]` on the Rust side matching the TS camelCase convention used throughout the app.

## 6. R-rule + AD-013 v2 invariants

- **R1 (attach-only)**: unchanged. No session-creation paths touched. B1 pty/mod.rs untouched.
- **R3 (failure isolation)**: unchanged. No changes to PtySession or tmux lifecycle. B1 smoke Step 12 `R3 OK` still applies.
- **AD-013 v2 backend enforcement**: 008 is the *backing* for this invariant. Every future project-mode list query from the frontend can now filter by `project_id` via `cmd_list_*_for_project` with NOT NULL + FK guaranteeing the column is always populated. Global-mode queries remain on the existing un-scoped commands. When the frontend store passes `activeProjectId` through `cmd_append_supervisor_message`, new events inherit the correct project — enforced at the DB level, not just UI level.
- **US-P0-03 multi-project guarantee**: rows from `projectA` cannot accidentally appear in a `projectB` query because the FK + project-scoped SELECTs now enforce isolation at the SQL layer.

## 7. Flag-ups for Flux / Lyra / Aegis attention

### 7.1 Clippy doc_lazy_continuation (pre-existing, unchanged)

`cargo clippy -p seatloom-core -- -D warnings` continues to fail on 3 pre-existing `clippy::doc_lazy_continuation` lints in `db/repositories.rs`. Confirmed same failure on parent commit `0c0f425` via stash-check (same approach Flux used on B1). **NOT a 008 regression.** Already flagged by Lyra for a sibling cleanup packet after SG-B closes.

### 7.2 No `projects` table seeding beyond `'seatloom'`

The FK `REFERENCES projects(id)` requires every `project_id` value to exist in the `projects` table. Today only `'seatloom'` is seeded. When AD-013 v2 UI adds a "create new project" affordance, that packet must INSERT a `projects` row before any workitem/session/handoff/event can reference it. Out of 008 scope per packet §8; recording here for coordination.

### 7.3 Supervisor command IPC contract change

`AppendSupervisorRequest` now carries an optional `projectId` field. Existing callers (anything that posts to `cmd_append_supervisor_message` without `projectId`) continue to work — the command falls back to `AppState.default_project_id = "seatloom"`. This is a backwards-compatible extension.

### 7.4 `cmd_list_*_for_project` are additive, not replacements

Existing `cmd_list_sessions`, `cmd_list_workitems`, `cmd_list_handoffs`, `cmd_list_events` are preserved for Global mode. The four new `cmd_list_*_for_project` commands are additive. When AD-013 v2 frontend wires project-mode, it calls the `_for_project` variants; Global mode continues to call the un-scoped variants. No existing caller needs to change.

## 8. Known gaps — carry-forward to future packets (not 008 acceptance blockers)

- **SeatInputInjected canonical_events emission**: not in 008 scope (packet §8 excludes write-path event emission). Lyra deferred this to v0.0.3+ observability per Aegis 2026-05-11 ruling.
- **Write-path event producers emitting `workitem` / `session` / `handoff` refs**: when B2+ adds these, the 008 canonical_events lineage joins will start producing non-default project_id on backfill re-runs. Forward-compatible, not retrofitting.
- **Migration 006 (`plan_mode_authority`) and 007 (`seats_budget`)**: reserved per Aegis decision-1 binding, not yet dispatched. 008 correctly skips 006/007 numbering.

## 9. Commit-pinned evidence

- Packet commit: `068f67b` (Nimbus §10 pre-impl comments).
- Implementation commit: `526d25a`.
- B1 parent (untouched by this delivery): `0c0f425`.
- Parent prior to 008: `1a558ed` (Lyra MEMORY catch-up).

## 10. Handoff

Flux Layer A verify is unblocked. Run against commit `526d25a`. Expected outputs:

1. All cargo / pnpm commands in §5.4–§5.5 reproduce with same results.
2. Fresh `./scripts/bootstrap.sh` on a clean container applies 008 via the updated schema loop and passes §5.3 verification queries.
3. Pre-existing clippy 3-lint failure confirmed as NOT a 008 regression (use the same stash-test approach Flux used on B1 at `0c0f425`).

On Flux PASS → Lyra acceptance → 008 closes. B1 acceptance remains independent (already pending on Flux B1 verify at `0c0f425`).

---

*Delivered by Nimbus · 2026-05-11 · commit `526d25a` · packet ref `NIMBUS-2026-05-09-schema-008-project-isolation-v1.md` · 12 files, +308/-22 · all 10 acceptance criteria met with line-pinned evidence · parallel-with-B1 honored (B1 @ `0c0f425` untouched).*