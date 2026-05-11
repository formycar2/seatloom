# Flux Verify: 008 — Multi-Project Isolation Schema Migration

[Lyra -> Flux] 008 delivered at commit `526d25a`. 12 files / +308/-22. Run commit-pinned Layer A and write your verify delivery doc with the static-invariant matrix below.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-05-09-schema-008-project-isolation-verify-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-05-11 |
| to | flux |
| priority | P0 (parallel with B1 verify; B2 onward gated on 008 acceptance) |
| target_commit | `526d25a` (Nimbus 008) |
| target_branch | `track/infra-foundation` |
| delivery_doc | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-schema-008-project-isolation-delivery-v1.md` (commit `f681251`) |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-schema-008-project-isolation-v1.md` |
| flux_delivery_path | `docs/coordination/tasks/flux/FLUX-2026-05-09-schema-008-project-isolation-verify-delivery-v1.md` |
| tags | flux, verify, 008, schema, multi-project, project-isolation, commit-pinned |

---

## §1 — Summary

Nimbus's 008 lands `project_id TEXT NOT NULL REFERENCES projects(id)` on the four operational tables (workitems / sessions / handoffs / canonical_events) plus indexes, plus 4 new project-mode list commands while preserving existing un-scoped commands for AD-013 v2 Global mode. Migration applied + idempotent re-run proven against `seatloom-postgres` (PG 16): 6/5/3/35 backfilled, all NULL-counts zero, all GROUP BYs scoped to `seatloom`.

Three pre-impl corrections from his §10 (commit `068f67b`) were applied during implementation:
- 10.1 `event_object_refs` columns are `(event_id, ref_type, ref_id)` not the `(event_id, object_id, object_type)` my packet §2.1 sketched.
- 10.2 only `ref_type=seat` emitted today; `canonical_events` lineage joins are forward-compatible.
- 10.3 `ADD COLUMN IF NOT EXISTS` is supported since PG 9.6 (we run 16); no `DO` block needed for idempotency.

## §2 — Verify scope

### Layer A — automated checks

```bash
git fetch && git checkout 526d25a
git rev-parse HEAD                                # must equal 526d25a (full sha)
git show 526d25a --stat                           # expect ~12 files / +308 / -22

# Backend
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test -p seatloom-core --lib                 # must be 60/60 pass per delivery
cargo clippy -p seatloom-core -- -D warnings      # see §3 — clippy nuance same as B1

# Schema apply against the standing podman seatloom-postgres
podman ps                                         # confirm seatloom-postgres up
podman cp infra/postgres/schema/008_project_isolation.sql seatloom-postgres:/tmp/008.sql
podman exec seatloom-postgres psql -U seatloom -d seatloom -f /tmp/008.sql

# Backfill verification (per delivery §X — must reproduce zeros + 'seatloom' grouping)
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM workitems WHERE project_id IS NULL;"
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM sessions WHERE project_id IS NULL;"
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM handoffs WHERE project_id IS NULL;"
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT COUNT(*) FROM canonical_events WHERE project_id IS NULL;"
# All four must return 0.

podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT project_id, COUNT(*) FROM workitems GROUP BY project_id;"
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT project_id, COUNT(*) FROM sessions GROUP BY project_id;"
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT project_id, COUNT(*) FROM handoffs GROUP BY project_id;"
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT project_id, COUNT(*) FROM canonical_events GROUP BY project_id;"
# All four must show only 'seatloom' (the only project today).

# Idempotent re-run (Nimbus claims this works; verify independently)
podman exec seatloom-postgres psql -U seatloom -d seatloom -f /tmp/008.sql
# Expected: NOTICE skipping ADD COLUMN messages + same UPDATE counts (or 0 since values unchanged) + COMMIT clean. NO error.

# Frontend (DTO additions)
cd ui && pnpm exec tsc --noEmit                   # zero errors after DTO project_id additions
cd ui && pnpm build                               # exit 0
```

### Static invariant matrix (15 rows — extends to schema-specific checks)

| # | Invariant | Where to look |
|---|-----------|---------------|
| 1 | Scope held to declared files (~12) | `git show --stat 526d25a` |
| 2 | New file present: `infra/postgres/schema/008_project_isolation.sql` | `git show 526d25a -- 'infra/postgres/schema/008_project_isolation.sql'` non-empty |
| 3 | No B1 file overlap | `git show 526d25a --name-only \| grep -E 'pty/mod\.rs\|tmux_mirror_smoke\|SessionTerminal\.tsx\|SessionsWorkspace\.tsx'` empty (B1 5-file scope untouched). `session_cmds.rs` may show as touched only for the new `cmd_list_sessions_for_project` addition. |
| 4 | i18n.ts unchanged | `git diff 4477ab6..526d25a -- ui/src/i18n.ts` empty |
| 5 | All 4 ALTER TABLE ADD COLUMN with `IF NOT EXISTS` (per Nimbus §10.3 PG 16 support) | grep `ADD COLUMN IF NOT EXISTS project_id` 4× in 008.sql |
| 6 | All 4 SET NOT NULL after backfill | grep `ALTER COLUMN project_id SET NOT NULL` 4× |
| 7 | All 4 indexes created | grep `CREATE INDEX IF NOT EXISTS idx_(workitems\|sessions\|handoffs\|canonical_events)_project` 4× |
| 8 | Backfill uses `event_object_refs.ref_type='seat'` (per Nimbus §10.2 forward-compat) | grep `ref_type` in 008.sql canonical_events backfill block |
| 9 | Backfill defaults to `'seatloom'` for ambiguous joins | grep `'seatloom'` in COALESCE |
| 10 | New project-mode commands present (4): `cmd_list_sessions_for_project` / `cmd_list_workitems_for_project` / `cmd_list_handoffs_for_project` / `cmd_list_events_for_project` | grep in `src-tauri/src/commands/*.rs` |
| 11 | Existing un-scoped commands preserved (AD-013 v2 Global mode unaffected) | grep `cmd_list_workitems\b` (without `_for_project`) still present |
| 12 | DTO `projectId` added to relevant types (workItem/session/handoff/canonicalEvent) | grep `project_id\|projectId` in `src-tauri/src/dto.rs` and `ui/src/lib/types-dto.ts` |
| 13 | Repository inserts take `project_id` parameter | grep `project_id: &str` in workitem/session/handoff/event repo `insert*` fns |
| 14 | Frontend tsc zero errors after DTO additions | `pnpm exec tsc --noEmit` exit 0 |
| 15 | Backfill counts match delivery §X claims (6/5/3/35) | reproduce the four NULL-count + GROUP BY queries; counts may differ slightly if seed has changed since Nimbus's run, but NULL-count must be 0 and grouping must show only `'seatloom'` |

### R-rule matrix (4 rows)

| Rule | Verify-by |
|------|-----------|
| AD-013 v2 backend enforcement (Project mode = `WHERE project_id = $1`; Global mode preserved) | grep both `_for_project` and bare `cmd_list_*` versions present |
| US-P0-03 multi-project | new commands accept `project_id` parameter; existing schema FK enforced |
| Migration safety (transactional, idempotent) | re-run produces no error, NOTICE skip, idempotent UPDATE |
| B1 scope frozen | 008 commit MUST NOT touch B1's 5 files (other than the additive `cmd_list_sessions_for_project` in session_cmds.rs) |

## §3 — Pre-existing clippy lints — same as B1

`cargo clippy -p seatloom-core -- -D warnings` may still fail on the 3 `doc_lazy_continuation` errors in `db/repositories.rs` (same lints as B1 verify). Validate non-regression via:

```bash
git stash
git checkout 4477ab6  # parent of B1 + 008
cargo clippy -p seatloom-core -- -D warnings 2>&1 | grep doc_lazy_continuation | wc -l
git checkout 526d25a
git stash pop
```

If the count at `4477ab6` matches what's at `526d25a`, treat as **non-blocker** (already a recommended sibling cleanup packet — Nimbus §7 flag-up b confirms).

If 008 introduces NEW clippy errors beyond the 3 pre-existing, that IS an 008 regression — HOLD.

## §4 — Layer B — runtime (Mr. Zhang via Aegis)

Out of your scope; rolls into Lyra acceptance window. The runtime checklist will be:

1. `pnpm tauri dev`, attach to a real seat tmux session, confirm Inbox + WorkItems + Sessions + Handoffs surfaces still render correctly post-migration (no DTO mismatch).
2. Verify Project mode renders only seatloom-scoped data.
3. Verify Global mode renders cross-project aggregate (today only seatloom; placeholder for future projects).
4. (Optional) Insert a test project row, attach a workitem to it via SQL, confirm Project mode filters correctly.

Same observation pattern as A4-β / B1.

## §5 — Delivery doc requirements

Write to `docs/coordination/tasks/flux/FLUX-2026-05-09-schema-008-project-isolation-verify-delivery-v1.md`. Same structure as your B1 verify delivery doc:

1. T3 frontmatter (template, subtype=verification_delivery, status=delivered, target_commit=526d25a, packet_ref=this packet).
2. §1 verbatim outputs of all §2 commands.
3. §2 15-row static invariant matrix with PASS/FAIL + line evidence per row.
4. §3 4-row R-rule matrix with PASS/FAIL.
5. §4 known non-blockers: clippy lints (same disposition as B1).
6. §5 verdict: `Layer A PASS` / `Layer A HOLD` / `Layer A FAIL` with one-paragraph rationale.
7. §6 next-action pointer (→ Lyra acceptance + Mr. Zhang Layer B).

## §6 — Concurrency note

You're running B1 verify in parallel. Order doesn't matter — both deliveries are independent; choose your preferred sequencing. Don't bundle the verify deliveries.

## §7 — Timeline

ASAP. 008 acceptance unblocks B2 onward (anything that writes new `canonical_events` rows). The faster 008 closes, the cleaner the v0.0.2 milestone.

---

*Dispatched by Lyra · 2026-05-11 · Same three-layer protocol as A4-β + B1 · 15-row static + 4-row R-rule matrices · Migration apply + idempotent re-run + backfill counts*
