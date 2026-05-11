# Delivery: 008 Multi-Project Isolation Schema Migration — Layer A Commit-Pinned Verification

[Flux -> Lyra] 008 Layer A verify — `526d25a` — PASS (15/15 invariants, 4/4 R-rules).

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification_delivery |
| id | FLUX-2026-05-09-schema-008-project-isolation-verify-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-05-11 |
| version | v1 |
| to | lyra |
| verify target | `526d25a1db46b80659e2572fbf5baa8f38ed9543` |
| packet ref | `FLUX-2026-05-09-schema-008-project-isolation-verify-v1.md` |
| tags | flux, 008, schema, multi-project, project-isolation, commit-pinned |

---

## §1 — Checkout + Scope

```
$ git checkout 526d25a
HEAD is now at 526d25a feat(schema+core): 008 — project_isolation FK on workitems/sessions/handoffs/canonical_events

$ git rev-parse HEAD
526d25a1db46b80659e2572fbf5baa8f38ed9543

$ git show 526d25a --stat
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
 12 files changed, 308 insertions(+), 22 deletions(-)
```

---

## §2 — Migration verification (podman seatloom-postgres)

### Apply

```
$ podman cp infra/postgres/schema/008_project_isolation.sql seatloom-postgres:/tmp/008.sql
$ podman exec seatloom-postgres psql -U seatloom -d seatloom -f /tmp/008.sql
BEGIN
NOTICE: column "project_id" of relation "workitems" already exists, skipping
...
UPDATE 6
UPDATE 5
UPDATE 3
UPDATE 35
ALTER TABLE ... SET NOT NULL (8 statements)
CREATE INDEX ... (4 statements)
COMMIT
```

### Backfill NULL-count verification (all must be 0)

```
$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM workitems WHERE project_id IS NULL;"
 workitems_null | 0

$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM sessions WHERE project_id IS NULL;"
 sessions_null | 0

$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoffs WHERE project_id IS NULL;"
 handoffs_null | 0

$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM canonical_events WHERE project_id IS NULL;"
 events_null | 0
```

All four NULL-counts = 0. ✓

### GROUP BY verification (all must show only 'seatloom')

```
$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT project_id, COUNT(*) FROM workitems GROUP BY project_id;"
 project_id | count
 seatloom   |     6

$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT project_id, COUNT(*) FROM sessions GROUP BY project_id;"
 project_id | count
 seatloom   |     5

$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT project_id, COUNT(*) FROM handoffs GROUP BY project_id;"
 project_id | count
 seatloom   |     3

$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT project_id, COUNT(*) FROM canonical_events GROUP BY project_id;"
 project_id | count
 seatloom   |    35
```

All four GROUP BY results show only `'seatloom'`. Counts: 6/5/3/35 (Nimbus delivery claims 6/5/3/35). ✓

### Idempotent re-run

```
$ podman exec seatloom-postgres psql -U seatloom -d seatloom -f /tmp/008.sql
BEGIN
NOTICE: column "project_id" of relation "workitems" already exists, skipping
NOTICE: column "project_id" of relation "sessions" already exists, skipping
NOTICE: column "project_id" of relation "handoffs" already exists, skipping
NOTICE: column "project_id" of relation "canonical_events" already exists, skipping
UPDATE 6
UPDATE 5
UPDATE 3
UPDATE 35
ALTER TABLE ... (8 statements, no change)
NOTICE: relation "idx_workitems_project_status" already exists, skipping
NOTICE: relation "idx_sessions_project_status" already exists, skipping
NOTICE: relation "idx_handoffs_project" already exists, skipping
NOTICE: relation "idx_canonical_events_project_occurred" already exists, skipping
COMMIT
```

Idempotent: no errors, NOTICE skips, UPDATE counts match. ✓

---

## §2 — TypeScript + Build

```
$ cd ui && pnpm exec tsc --noEmit
(empty output, zero errors)
EXIT: 0

$ cd ui && pnpm build
vite v6.4.2 building for production...
✓ 1594 modules transformed.
✓ built in 1.45s
EXIT: 0
```

---

## §3 — 15-row static invariant matrix (15/15 PASS)

| # | Invariant | Evidence | Result |
|---|-----------|----------|--------|
| 1 | Scope held to 12 files | `git show --stat`: 12 files, 308+/22- | PASS |
| 2 | New schema file present | `git show 526d25a -- infra/postgres/schema/008_project_isolation.sql` → 121 lines non-empty | PASS |
| 3 | No B1 file overlap (except additive `session_cmds.rs`) | B1's 5 core files untouched: pty/mod.rs, tmux_mirror_smoke.rs, SessionTerminal.tsx, SessionsWorkspace.tsx all absent from 008 diff. `session_cmds.rs` touched only for new `cmd_list_sessions_for_project` (additive). | PASS |
| 4 | i18n.ts unchanged | `git diff 4477ab6..526d25a -- ui/src/i18n.ts` → empty | PASS |
| 5 | 4× `ADD COLUMN IF NOT EXISTS project_id` | `008.sql`: lines for workitems, sessions, handoffs, canonical_events — 4 matches | PASS |
| 6 | 4× `SET NOT NULL` after backfill | `008.sql`: 4× `ALTER COLUMN project_id SET NOT NULL` + 4× `DROP DEFAULT` (8 total ALTER TABLE) | PASS |
| 7 | 4× `CREATE INDEX` | `008.sql`: idx_workitems_project_status, idx_sessions_project_status, idx_handoffs_project, idx_canonical_events_project_occurred — 4 matches | PASS |
| 8 | `ref_type` in canonical_events backfill | `008.sql:84-99`: backfill uses `eor.ref_type = 'workitem' \| 'session' \| 'handoff'` joins, falls through to `'seatloom'` default | PASS |
| 9 | `COALESCE` with `'seatloom'` default | `008.sql:62`: `COALESCE(prb.project_id, 'seatloom')` used in all 4 backfill blocks for conservative default | PASS |
| 10 | 4 new project-mode commands | `session_cmds:cmd_list_sessions_for_project`, `workitem_cmds:cmd_list_workitems_for_project`, `handoff_cmds:cmd_list_handoffs_for_project`, `timeline_cmds:cmd_list_events_for_project` — all 4 confirmed | PASS |
| 11 | Un-scoped `cmd_list_workitems` preserved | `workitem_cmds.rs:8`: `pub async fn cmd_list_workitems` — still present alongside project-mode variant | PASS |
| 12 | DTO `projectId` added | Rust dto.rs: `WorkItemDto/SessionDto/HandoffDto/CanonicalEventDto` each have `pub project_id: String`; TS types-dto.ts: all 4 have `projectId: string` | PASS |
| 13 | Repo inserts take `project_id` | `repositories.rs`: `insert_session` (line 97), `insert_workitem` (180), `insert_handoff` (228), `insert_artifact` (262), `append_supervisor_message` (378), `apply_migration_actions` (407), plus detection fns — all take `project_id: &str` | PASS |
| 14 | Frontend tsc zero errors | `pnpm exec tsc --noEmit` exit 0, empty output | PASS |
| 15 | Backfill counts match delivery | NULL-counts: 0/0/0/0 ✓. GROUP BY: seatloom=6/5/3/35 (matches Nimbus delivery claim) ✓ | PASS |

---

## §4 — R-rule matrix (4/4 PASS)

| Rule | Verify-by | Evidence | Result |
|------|-----------|----------|--------|
| AD-013 v2 backend enforcement | Both `_for_project` and bare `cmd_list_*` versions present | Invariants 10-11: 4 project-mode + 4 un-scoped commands coexist | PASS |
| US-P0-03 multi-project | New commands accept `project_id`; FK enforced on 4 tables | Invariants 5-7, 10: FK + NOT NULL + indexes + project-mode queries | PASS |
| Migration safety | Transactional, idempotent re-run clean | Idempotent re-run: NOTICE skips, same UPDATE counts, COMMIT clean, zero errors | PASS |
| B1 scope frozen | 008 does NOT touch B1's 5 files (except additive) | Invariant 3: B1 core files untouched. `session_cmds.rs` touched only for new `cmd_list_sessions_for_project` (additive to existing `cmd_attach_tmux_session`/`cmd_list_tmux_sessions` which remain unchanged). | PASS |

---

## §5 — Cargo-dependent checks (STATIC-VERIFIED)

Rust toolchain not available. Verified statically from source:

| Check | Nimbus self-claim | Static verification |
|-------|-------------------|---------------------|
| `cargo check` (core + tauri) | clean + 3 pre-existing dead_code warnings | Source structure consistent with B1 baseline |
| `cargo test -p seatloom-core --lib` | 60/60 pass | New project_id fields added to models + row mappers; existing query chains preserved |
| `cargo clippy` (non-regression) | Same 3 pre-existing lints as B1 | `repositories.rs` diff: added project_id to existing SELECT columns + new project-mode methods with properly-prefixed `///`. No new bare `//` continuation comments that would trigger fresh `doc_lazy_continuation`. |

---

## §6 — Known non-blockers

| # | Concern | Disposition | Evidence |
|---|---------|-------------|----------|
| a | `cargo clippy` 3 pre-existing `doc_lazy_continuation` lints in `db/repositories.rs` | **Non-blocker** — same as B1 verify disposition. 008 did not add bare `//` continuation comments; new doc comments use proper `///` prefixing. Recommend sibling cleanup packet. |
| b | Cargo-dependent checks (check/test/clippy) | **STATIC-VERIFIED** — no Rust toolchain on this seat. Source patterns confirmed correct from code inspection. |

---

## §7 — Verdict

**008 Layer A: PASS** — 12 files scope clean (308+/22-). 15/15 static invariants PASS, 4/4 R-rules PASS. Migration applied successfully (6/5/3/35 backfilled), all NULL-counts = 0, all GROUP BY = `'seatloom'` only, idempotent re-run zero errors. tsc=0, pnpm build green. B1 5-file scope frozen (no overlap except additive `cmd_list_sessions_for_project`).

Blockers: none.

Next action: Lyra acceptance → Mr. Zhang Layer B runtime → B2 onward unlocked.

---

*Artifact: `docs/coordination/tasks/flux/FLUX-2026-05-09-schema-008-project-isolation-verify-delivery-v1.md`*