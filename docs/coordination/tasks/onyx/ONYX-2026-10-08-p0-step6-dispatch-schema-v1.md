# Task: P0 step 6 (Onyx half) — `dispatches` inbox migration + `depends_on`

[Lyra -> Onyx] Aegis dispatch `AEGIS-2026-10-08-p0-step6-governed-dispatch-v1`: the schema half of governed dispatch (G19). Nimbus builds the scheduler against your migration in parallel. **One contract governs both packets** — `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md`, carried verbatim into both. Do not re-derive it.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | ONYX-2026-10-08-p0-step6-dispatch-schema-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| to | onyx |
| priority | P0 |
| milestone | P0 — governed dispatch (no dropped hops) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-p0-step6-governed-dispatch-v1.md` |
| shared contract | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` (verbatim, both packets) |
| parallel packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1.md` |
| depends_on | migration `010` (vocab / role / schema_version); write API A1 (merged); step 4 MCP emit (`af3a0f7`, merged) |
| base branch | `track/infra-foundation` @ `b200cd3` |
| delivery branch | `packet/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-v1` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/onyx` (currently on `onyx/pi-workspace`) |
| delivery path | `docs/coordination/tasks/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-delivery-v1.md` |
| tags | onyx, p0, g19, governed-dispatch, schema, inbox |

---

## 0. A blocking defect I found before dispatching — fix this FIRST

**`infra/postgres/schema/011_agent_history_archive.sql` is on disk and on `track/infra-foundation` (commit `6b2addb`) but is registered in NEITHER `PRE_SEED_SCHEMA` nor `POST_SEED_SCHEMA` in `scripts/apply-db.sh`.** I verified this:

```
$ grep -n '011' scripts/apply-db.sh
(no output)
```

`apply-db.sh`'s own completeness guard therefore fires:

```
ERROR: unclassified migration(s) in infra/postgres/schema — refusing to apply a partial schema.
  unclassified: 011_agent_history_archive.sql
```

So **the script currently refuses to build a database at all.** Every scratch-DB build since `6b2addb` has had to be done by hand — which is exactly what Flux reported doing for the step-4 gate ("migrations 001–010 are actually 8 files: 001-005, 008, 009, 010"). This is a P0 prerequisite for your own verification **and** for Flux's step-6 acceptance, which must build a scratch DB.

Register `011` in the correct group **with a reason**, per the script's own rule: pre-seed if the migration constrains columns the seed does not provide, post-seed otherwise. Read `011` and decide; state which group and why in your delivery. Do not simply append it to make the guard pass.

## 1. Scope

### 1.1 The migration — `infra/postgres/schema/012_governed_dispatch.sql`

Slot `012` is next: 001–005, 008, 009, 010, 011 exist. (Note `006`/`007` were never created — the numbering has gaps by history, not by error. Do not renumber anything.)

1. **Widen `canonical_events_event_type_vocab`** with the three new values from the contract: `SeatDispatchIssued`, `SeatDispatchDelivered`, `SeatDispatchCompleted`. Use `010`'s catalog-guarded `DO $$ … IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = …) $$` pattern — additive, idempotent, one transaction, verified by applying twice.
2. **Widen `event_object_refs_ref_type_vocab`** with `dispatch`, same pattern.
3. **Create `dispatches`** exactly as the contract specifies: `id`, `target_seat_id`, `issued_by`, `project_id` (REFERENCES projects), `title`, `payload` (JSONB), `status`, `priority`, `issued_at`, `started_at`, `completed_at`.
4. **Create `dispatch_dependencies`** exactly as the contract specifies: `dispatch_id` REFERENCES dispatches(id), `dep_kind` IN ('event','artifact','dispatch'), `dep_value` TEXT. Primary key `(dispatch_id, dep_kind, dep_value)` so the same referent cannot be listed twice.
5. **Status domain CHECK** on `dispatches.status` IN ('ready','blocked','in_flight','done').
6. **`dep_kind` domain CHECK** IN ('event','artifact','dispatch').
7. **One-in-flight partial unique index** on `dispatches (target_seat_id) WHERE status = 'in_flight'` — contract rule 1. This is the load-bearing constraint: it makes one-in-flight a database fact, not a scheduler promise.
8. **Indexes the scheduler needs**: at minimum `(target_seat_id, status, priority DESC, issued_at)` for the pull query, and `(dispatch_id)` on dependencies (covered by the PK).
9. **Register the migration in `scripts/apply-db.sh`** in the correct group with a reason, alongside the `011` fix. A migration on disk but unregistered is invisible to CI — `010`'s header says so and this packet has just proved it.
10. **COMMENT ON** the new tables/columns the way `010` does — the reasoning is part of the artifact.
11. **Ship a verification query** in the file header comment (ONYX.md §5.5 requires one), covering: the two widened vocabularies list the new values; `dispatches` and `dispatch_dependencies` exist with the right domains; the one-in-flight index exists; idempotency (a second apply performs no catalog write).

### 1.2 What is NOT yours

- **No Rust.** Nimbus owns `crates/seatloom-mcp-emit` and the scheduler. Your packet is `.sql` + `scripts/`.
- **No UI.** No `.tsx`.
- **No changes to `write_api.rs` / `event_domain.rs`.** A1's accepted files are read-only for you. If the migration needs a matching Rust enum value to compile, that is Nimbus's lane — flag it, do not edit it.
- **No `dispatches` seeding.** The seed is not in scope; Flux builds his own fixtures on a scratch DB.

## 2. Design decisions you own and must state

1. **`dep_value` referential integrity.** There is deliberately **no FK** from `dispatch_dependencies.dep_value` to any table — `dep_kind` is polymorphic across three object types, which is exactly why a per-kind FK is the wrong primitive (the same reasoning `010` used for `event_object_refs.ref_id`). State the consequence plainly: a dangling dependency is possible at the schema level, so **readiness must be evaluated by existence query, not by FK**. Say how you verified that a dangling dep reads as `blocked` rather than erroring.
2. **`target_seat_id` typing.** Match whatever the existing `seats` table uses (read schema `001` and the seed). If a FK to `seats(id)` is safe and additive, add it and say why; if it would break the seed or an existing path, do not, and say why instead. Do not guess.
3. **`payload` JSONB vs JSON.** `010` left `canonical_events.payload` nullable deliberately and documented why. State your choice for `dispatches.payload` and the reason, in the same spirit.
4. **Pre- or post-seed** for `012`, with the reason, applying the same rule you applied to `011`.

## 3. Verification

```
bash scripts/apply-db.sh            # or the equivalent dry-run the script supports
```

Plus, on a **scratch** database (never the live `seatloom`):

1. Apply `012` **twice** — the second apply must perform no catalog write at all (same standard `010` met).
2. Run the verification query from the file header; every assertion must hold.
3. Insert one `ready` dispatch and one `blocked` dispatch (with a dependency on an event id that does not exist); confirm the second reads as not-ready by existence query, with no error.
4. Insert a second `in_flight` dispatch for the **same** seat and confirm the partial unique index **rejects** it. Then insert an `in_flight` dispatch for a **different** seat and confirm it is accepted. Paste both.
5. Insert a `done` dispatch for the same seat alongside an `in_flight` one and confirm it is accepted (only `in_flight` is unique-scoped).

## 4. Done-definition

1. `011` registered in `scripts/apply-db.sh` with a stated reason, and the completeness guard passes.
2. `012_governed_dispatch.sql` exists, is additive and idempotent, widens both vocabularies with exactly the contract's new values, creates both tables with the contract's exact columns and domains, and carries the one-in-flight partial unique index.
3. `012` is registered in `scripts/apply-db.sh` in the correct group with a reason.
4. Verification query shipped; all five §3 checks pass with verbatim output.
5. Design decisions §2 stated with reasons, not assumed.
6. Zero Rust, zero `.tsx`, no A1 files touched, no live database touched.
7. Delivery artifact at the header path with the exact delivery commit.

## 5. Blockers

- None at dispatch time.
- **Report rather than work around** if a column, constraint, or line reference has moved, or if the contract's shape turns out to be inexpressible without a Rust change. A wrong assumption here lands in an append-only table and in a scheduler Nimbus is building against it right now.

---

*Dispatched by Lyra · 2026-10-08 · P0 step 6, schema half · joined to Nimbus's scheduler packet by one verbatim value-domain contract · §0 is a blocking defect I found and verified before dispatching*
