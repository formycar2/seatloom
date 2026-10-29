# Delivery: Lane C — migration 016 acceptance-plan schema

[Onyx -> Lyra] acceptance-lineage milestone, Lane C (migration 016): the structured
acceptance plan now has a structural carrier (`acceptances.acceptance_plan` JSONB,
shape v1) and a no-plan-no-card DB guard (`acceptances_plan_required`, NOT VALID by
design). Verified on a scratch DB only; the live `seatloom` database was never touched.

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | implementation-delivery |
| id | ONYX-2026-10-10-acceptance-plan-schema-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-10-10 |
| version | v1 |
| responds to | governed dispatch `disp-accplan-lane-c-onyx` (acceptance-lineage Lane C, highest priority — operator blocking feedback) |
| packet | `docs/coordination/tasks/onyx/ONYX-2026-10-10-acceptance-plan-schema-v1.md` (found on `packet/lyra/milestone-summary`; not on my base branch — worked from the payload + that packet verbatim) |
| governing contract | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1.md` §1 (shape v1), §2 (enforcement), §6 (red lines) — carried verbatim per packet §2 |
| base branch | `track/infra-foundation` @ `b6751a1` |
| delivery branch | `packet/onyx/acceptance-plan-schema` |
| files changed | `infra/postgres/schema/016_acceptance_plan.sql` (new), `scripts/apply-db.sh` (+1 POST_SEED entry), this doc |
| rust / tsx touched | none (verified: `git status --short` shows only the two files above) |
| live db touched | none — all verification on scratch DB `seatloom_scratch_016_onyx` (fresh build inside the PG16 container), dropped after |

---

## 0. What shipped

### 0.1 `infra/postgres/schema/016_acceptance_plan.sql` (new)

1. **ONE column**: `ALTER TABLE acceptances ADD COLUMN IF NOT EXISTS acceptance_plan JSONB NOT NULL DEFAULT '{}'::jsonb;` — additive-safe: pre-existing rows backfill harmlessly to `'{}'`.
2. **ONE CHECK**: `acceptances_plan_required` — `CHECK (acceptance_plan <> '{}'::jsonb)` added **NOT VALID**, in a `pg_constraint`-guarded DO block (house style, 010/012/013).
3. **COMMENT ON COLUMN** quoting the contract's shape v1 verbatim (same wording Lanes A/B/D carry), including the `'{}'` is-not-a-plan rule, request-time-set/immutable-after rule, and the tool-layer/DB-layer enforcement split.
4. Header carries the authority chain (value-domain contract §1/§2, dispatch id), the slot rationale (016 = next after 014; 015 is the parallel D5 migration on the same seat — serialized, never co-edited), and the ONYX.md §5.5 verification queries.

### 0.2 `scripts/apply-db.sh` — POST_SEED registration with a rule-based reason

`016_acceptance_plan.sql` appended to `POST_SEED_SCHEMA` after 014, canonical order `014 -> 016`: the seed writes none of `acceptances`' rows and the table exists only after 014 (POST-SEED) creates it. `bash -n` clean; completeness guard passes.

### 0.3 Contract compliance (packet §3.5 red lines)

- **No new event types, no new ref types, no new tables** — the plan rides the existing `AcceptanceRequested` event payload onto the acceptances projection (same pattern as `dispatches.payload` JSONB, 012). Vocabularies untouched (still 21 event types / 11 ref types).
- **No data rows inserted/modified** — the legacy pending row's plan backfill + `VALIDATE CONSTRAINT` is Aegis's live-DB act; not done here.
- **`crates/**` / `ui/**` untouched** (verified above).
- **No historical migration renumbered** (006/007 stay reserved).

---

## 1. Design notes the packet asked me to state

- **NOT VALID is deliberate, and the handoff is documented in the migration itself.** The one legacy pending row (`acc-disp-me6Nm8e9D6ZRe4fKyVokY`, Mira D2 UI) currently carries no plan. NOT VALID exempts existing rows from the check while every INSERT/UPDATE is held to it from the moment it is added. The migration comment states the handoff verbatim: Aegis backfills Lyra's authored plan for that row and runs `ALTER TABLE acceptances VALIDATE CONSTRAINT acceptances_plan_required;` as a live-DB act on the Aegis lane. The next reader of the migration will not mistake it for a forgotten validate.
- **Shape validation lives in the tool layer, not the DB.** `request_acceptance` (MCP, Nimbus lane) rejects a missing/malformed plan with a TYPED error before any DB contact (contract §2, house discipline). The DB enforces only non-emptiness — it does not re-implement the shape validator. This keeps one authority per rule.
- **Immutability**: the plan is set at request time and immutable after, same rule as `how_to_verify`; recorded in the column comment.
- **015 (D5) interaction**: none by construction — 016 only appends to `schema/` and `apply-db.sh`; the two same-seat dispatches serialize, no overlapping edit points.

## 2. Verification — scratch DB only (live `seatloom` untouched)

Environment: no host `psql`; ran through a `postgres:16-alpine` runner container sharing the `seatloom-postgres` network namespace, against a **freshly built** scratch DB `seatloom_scratch_016_onyx` (dropped after). Live `seatloom` confirmed still without `acceptances.acceptance_plan` after the run.

1. **Full build from track state** — `scripts/apply-db.sh` on the empty DB applies the whole `001 → 016` chain + seed; completeness guard passes, exit 0, no SQL error. 016 applied in canonical position after 014.
2. **Column form** (§5.2):
   ```
    column_name     | data_type | is_nullable | column_default
   -----------------+-----------+-------------+----------------
    acceptance_plan | jsonb     | NO          | '{}'::jsonb
   ```
3. **CHECK form** (§5.3):
   ```
          conname          | convalidated |                pg_get_constraintdef
   ------------------------+--------------+---------------------------------------------------
    acceptances_plan_required | f        | CHECK ((acceptance_plan <> '{}'::jsonb)) NOT VALID
   ```
   Present, `convalidated = f` (NOT VALID, as designed), definition contains `<> '{}'`.
4. **Negative case** (§5.4): `INSERT ... acceptance_plan='{}'` → **rejected**:
   `ERROR: new row for relation "acceptances" violates check constraint "acceptances_plan_required"`.
5. **Positive case** (§5.4): a plan with `plan_version/scenario/steps/expected` → **accepted** (`INSERT 0 1`; row reads back `plan_version=1`, 1 step, 1 expected).
6. **Idempotency** (§5.1) — second apply of `016_acceptance_plan.sql`, catalog counts before/after:
   ```
   BEFORE: pg_class=611 pg_constraint=228 pg_attribute=4085
   AFTER:  pg_class=611 pg_constraint=228 pg_attribute=4085   (zero delta; only NOTICE ... already exists, skipping)
   ```
7. **COMMENT present** — `col_description(...) IS NOT NULL` → `t`.

## 3. Finding to flag (pre-existing, NOT fixed — out of this dispatch's lane)

Running `scripts/apply-db.sh` a **second time against an already-built database** fails at seed 001:

```
psql:infra/postgres/seed/001_real_collaboration_baseline.sql:155: ERROR:  null value in
column "project_id" of relation "sessions" violates not-null constraint
```

Cause: seed 001's `INSERT INTO sessions ... ON CONFLICT (id) DO UPDATE` supplies no `project_id`, and `008_project_isolation.sql` (POST-SEED) sets `sessions.project_id NOT NULL`. On a re-run the ON CONFLICT update path does not suppress the NOT NULL check, so the seed aborts (exit 3). This is independent of 016 — it reproduces on any DB that has already been through 008 (also observed on the pre-built `seatloom_scratch_016` DB). It does not affect 016's idempotency, which is measured per-migration (metric 6 above) exactly as the D2 (013/014) delivery did. **Flagged for Aegis/Lyra**: a seed-side fix (e.g. supply `project_id` in the seed insert or make the upsert conflict-scoped) is a separate decision — I did not touch the seed under this dispatch.

## 4. Completion flow

1. Committed to `packet/onyx/acceptance-plan-schema` (commit carries the dispatch id).
2. This delivery doc.
3. Gate event `ev-016-acceptance-plan-landed` (ArtifactCreated) emitted — unblocks Nimbus Lane B via the 013 trigger.
4. `complete_dispatch(disp-accplan-lane-c-onyx, outcome="done")`.
5. Flux verification self-requested (commit-pinned branch/SHA, base `track/infra-foundation`, COORDINATION_RULES §14).
6. After Flux green: `request_acceptance` with the packet §7 how_to_verify verbatim. The seat never merges itself.
