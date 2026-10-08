# Delivery: P0 step 6 (Onyx half) — `dispatches` inbox migration + `apply-db.sh` fix

[Onyx -> Lyra] Schema half of governed dispatch (G19). Implements the verbatim
value-domain contract; Nimbus builds the scheduler against it in parallel.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | ONYX-2026-10-08-p0-step6-dispatch-schema-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-10-08 |
| version | v1 |
| responds to | `docs/coordination/tasks/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-v1.md` |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-p0-step6-governed-dispatch-v1.md` |
| shared contract | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` (verbatim, both packets) |
| parallel packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1.md` |
| base branch | `track/infra-foundation` |
| base commit | `c5b2397` (see §0.1 — reconciled from packet's stated `b200cd3`) |
| delivery branch | `packet/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-v1` |
| delivery commit | `97cf9bf` |
| files changed | `infra/postgres/schema/012_governed_dispatch.sql` (new), `scripts/apply-db.sh` (+2 lines) |
| rust / tsx touched | none (verified: `git status --short` has no `.rs`/`.tsx`) |
| live db touched | none — all verification on scratch DB `scratch_onyx_step6`, dropped after |

---

## 0. The §0 blocking defect — fixed FIRST

`infra/postgres/schema/011_agent_history_archive.sql` was on disk and on the track
but registered in **neither** `PRE_SEED_SCHEMA` nor `POST_SEED_SCHEMA` in
`scripts/apply-db.sh`. The script's completeness guard therefore refused to build any
database (`ERROR: unclassified migration(s) … refusing to apply a partial schema`).
This is why every scratch build since the 011 commit had to be done by hand.

**Fix:** registered `011` in **POST-SEED**, with a stated reason.

**Classification reasoning (by the script's own rule, not by number):** the rule is
"PRE-SEED if the seed writes columns this migration creates; POST-SEED otherwise."
`011` is `CREATE TABLE IF NOT EXISTS archived_sessions` — a brand-new table with **no
FK to seats/projects** and **no DDL against any seeded table**, and the seed never
writes to it. The seed writes none of its columns, so it is not pre-seed; it falls in
the "otherwise" bucket → POST-SEED. In truth its placement relative to the seed is
immaterial (they share nothing), so the non-pre-seed classification is simply the
honest one — and it matches the migration's own header (`Additive, post-seed,
idempotent`). It was **not** appended blindly to silence the guard.

## 0.1 Base-commit reconciliation (reported, not worked around)

The packet names base `track/infra-foundation @ b200cd3`, and its §0 text references
`6b2addb`. Neither of those commits contains the packet file or the value-domain
contract I was told to implement against — both artifacts first appear one commit
later, at `4e070c3` ("dispatch P0 step 6 …"), and the local `track/infra-foundation`
tip is `c5b2397` (one further writeback commit: MEMORY + daily log, **no schema
change**). History is clean and linear: `6b2addb → b200cd3 → 4e070c3 → c5b2397`, each
an ancestor of the next.

Basing on `b200cd3` would produce a delivery branch that does **not** carry the packet
it fulfils. I therefore based the delivery branch on the current track tip `c5b2397`,
which is strictly ahead of `b200cd3` and contains the packet, the contract, `011`, and
the current `apply-db.sh`. No schema content differs between `4e070c3` and `c5b2397`.
Flagging per packet §5 ("report rather than work around if a … line reference has
moved"); if Lyra wants the branch rebased onto a different base, that is a cheap redo.

The worktree was on `onyx/pi-workspace @ 6b2addb` at dispatch (stale, behind track);
this is the expected PI post-migration state noted in the re-hydration brief.

---

## 1. What shipped — `infra/postgres/schema/012_governed_dispatch.sql`

Additive, idempotent, one transaction (`BEGIN … COMMIT`), 010 house style.

1. **event_type vocab widened** +3: `SeatDispatchIssued`, `SeatDispatchDelivered`,
   `SeatDispatchCompleted` (12 → 15 values).
2. **ref_type vocab widened** +1: `dispatch`.
3. **`dispatches`** created with the contract's exact columns in order: `id`,
   `target_seat_id`, `issued_by`, `project_id`, `title`, `payload` (JSONB NOT NULL),
   `status`, `priority` (INT), `issued_at`, `started_at`, `completed_at`.
4. **`dispatch_dependencies`** created: `dispatch_id` REFERENCES dispatches(id),
   `dep_kind`, `dep_value`, PK `(dispatch_id, dep_kind, dep_value)`.
5. **`dispatches_status_vocab`** CHECK: `ready | blocked | in_flight | done`.
6. **`dispatch_dependencies_dep_kind_vocab`** CHECK: `event | artifact | dispatch`.
7. **The load-bearing constraint** — partial unique index
   `uq_dispatches_one_in_flight_per_seat ON dispatches (target_seat_id) WHERE
   status='in_flight'`. One-in-flight is a **database fact**, not a scheduler promise.
8. **Scheduler pull index** `idx_dispatches_pull (target_seat_id, status,
   priority DESC, issued_at)`; the dependencies lookup is covered by its PK.
9. **Registered in `apply-db.sh`** (POST-SEED, with reason — see §0 and §2.4).
10. **COMMENTs** on both tables and the load-bearing columns.
11. **Verification query** in the file header.

### Idempotency note worth stating
010 only ever **created** fresh CHECKs, so its bare `IF NOT EXISTS (conname …)` guard
was enough. 012 must **widen two CHECKs that already exist**, where that guard would
skip the widening forever. So the two vocab blocks guard on
`pg_get_constraintdef(oid) LIKE '%<new value>%'` and only then `DROP CONSTRAINT IF
EXISTS … ; ADD CONSTRAINT …` with the full value list. A re-apply finds the value
already present and does nothing. Verified by three applies (§3.1).

## 2. Design decisions I own (packet §2) — stated, not assumed

1. **`dep_value` has no FK — by design.** `dep_kind` is polymorphic across three object
   types (event/artifact/dispatch), so a per-kind FK is the wrong primitive — the same
   reasoning 010 used for `event_object_refs.ref_id`. **Consequence, stated plainly:** a
   dangling dependency is possible at the schema level. That is intentional — **readiness
   is an existence query, not an FK.** A missing referent must read as `blocked` (not an
   error, not a stall), which existence delivers and an FK would turn into an insert-time
   rejection. **Verified (§3.3):** a dep on `ev-does-not-exist` inserts cleanly and the
   dispatch computes `is_ready=false`.
2. **`target_seat_id` typing — TEXT, FK to `seats(id)`.** `seats.id` is `TEXT PRIMARY
   KEY` (001). The FK is additive and safe because the seed never inserts a `dispatches`
   row, so no seeded row can dangle it; every live seat the scheduler targets is already
   a `seats` row. **Verified (guards):** an unknown seat is rejected by
   `dispatches_target_seat_id_fkey`. `issued_by` is deliberately **not** FK'd: the
   contract types it as an `actor_ref` (`human:user`, `seat:…`), a superset of seat ids —
   a FK there would reject legitimate human/supervisor issuers.
3. **`payload` JSONB NOT NULL.** 010 left `canonical_events.payload` nullable because
   NULL (not recorded) vs `{}` (recorded, empty) is a real distinction there. Here the
   contract says NOT NULL and I agree: a dispatch with no payload carries no packet path
   or instructions and is meaningless, so "not recorded" is not a legitimate state. JSONB
   (not JSON) for binary storage/indexability; no whitespace/order fidelity is needed.
4. **Pre- or post-seed for 012 → POST-SEED.** Same rule as 011 (the seed writes none of
   the new columns). Additionally the two vocab CHECKs 012 widens exist only after 010
   (itself POST-SEED) has created them, so 012 **cannot** precede its own prerequisite.
   No seeded row references any value 012 adds, so post-seed cannot break the seed.

## 3. Verification — scratch DB only (`scratch_onyx_step6`), live `seatloom` untouched

Environment: no host `psql`; ran inside the running `seatloom-postgres`
(`postgres:16-alpine`) container against a throwaway DB, dropped afterward. Confirmed
live `seatloom` has no `dispatches` table and is still stale (no `sessions.project_id`).

**Full build** — `scripts/apply-db.sh` against the scratch DB applies the whole
`001→012` chain, completeness guard passes (011 and 012 both classified), no errors.

1. **§3.1 idempotency** — applied 012 three times; snapshot of all four vocab/status
   CHECKs, both new indexes, and both tables' columns is **byte-identical** before/after
   the re-applies (`diff` empty). Re-apply = zero catalog write.
2. **§3.2 verification query** — every assertion holds. `canonical_events_event_type_vocab`
   lists all three new values; `event_object_refs_ref_type_vocab` lists `dispatch`;
   `dispatches`/`dispatch_dependencies` exist with the exact contract columns/domains;
   `uq_dispatches_one_in_flight_per_seat` is `… WHERE (status = 'in_flight'::text)`.
3. **§3.3 blocked-by-existence** — inserted a `ready` and a `blocked` dispatch; the
   `blocked` one's dep points at a non-existent event id. The dep **inserted with no
   error** (no FK) and the readiness existence query returns `is_ready=false`.
4. **§3.4 one-in-flight** — first `in_flight` for `seat-aegis-001` accepted; a **second**
   `in_flight` for the same seat **rejected**: `duplicate key value violates unique
   constraint "uq_dispatches_one_in_flight_per_seat"`. An `in_flight` for a **different**
   seat (`seat-flux-001`) **accepted**; final state = one in_flight per seat.
5. **§3.5 done coexists** — a `done` dispatch for `seat-aegis-001` inserted fine
   alongside that seat's `in_flight` row (only `in_flight` is unique-scoped).

**Extra guard checks (all fire):** `target_seat_id` FK rejects unknown seat;
`project_id` FK rejects unknown project; `status` CHECK rejects `queued`; `dep_kind`
CHECK rejects `file`; dependency PK rejects a duplicate `(dispatch_id,dep_kind,dep_value)`.

## 4. Done-definition (packet §4)

1. ✅ 011 registered in `apply-db.sh` with a reason; completeness guard passes.
2. ✅ 012 additive + idempotent; both vocabularies widened with exactly the contract's
   new values; both tables with the contract's exact columns/domains; one-in-flight
   partial unique index present.
3. ✅ 012 registered in `apply-db.sh` (POST-SEED) with a reason.
4. ✅ Verification query shipped; all five §3 checks pass with the output above.
5. ✅ Design decisions §2 stated with reasons.
6. ✅ Zero Rust, zero `.tsx`, no A1 files touched, no live database touched.
7. ✅ Delivery artifact at this path; delivery commit `97cf9bf`.

## 5. Flags for Lyra / Nimbus (report, don't work around)

- **Base-commit reconciliation (§0.1).** Delivery branch based on current track tip
  `c5b2397` rather than the packet's stated `b200cd3`, because only the later commit
  carries the packet and contract. No schema content differs. Rebase on request.
- **Nimbus lane — Rust enum.** If the write path emits the three new event types /
  `dispatch` ref_type through a Rust enum (as `objects/*` parse paths do), Nimbus must
  add the matching variants for the migration's domain to be reachable from code. That
  is Nimbus's lane per packet §1.2; flagged, not touched.
- **Stale local DB (separate known item).** Live `seatloom` still lacks 008/009/010/011/012.
  A rebuild-to-latest via `apply-db.sh` remains an open data-integrity task in my domain;
  not in this packet's scope.

---

*Delivered by Onyx · 2026-10-08 · P0 step 6, schema half · delivery commit `97cf9bf` on
`packet/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-v1`*
