# Task: Runtime Triple Migration — schema 009, seed realignment, file-store drift check

[Lyra -> Onyx] Mr. Zhang ratified the Aegis multi-harness ingestion architecture review and
selected **sequencing option B**: build adapters + index + the `Runtime` triple refactor in
parallel with L1 Supervisor IM, but render **no UI surface** until L1 ships. This packet is item 2
of review §8, and your first packet as a registered seat.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | ONYX-2026-09-28-runtime-triple-migration-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-28 |
| version | v1 |
| to | onyx |
| priority | P1 (parallel with L1 Supervisor IM; does not block L1) |
| milestone | pre-ingestion schema debt paydown |
| depends_on | `docs/coordination/reviews/2026-09-28-aegis-multi-harness-session-ingestion-architecture-v1.md` §4 and §8 item 2; `docs/coordination/memory/2026-09-28.md` Addendum (seat harness convergence); `docs/coordination/roles/ONYX.md` |
| paired_packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1.md` |
| base branch | `track/infra-foundation` |
| base commit | `9599136d615db0c4cdb15811ccaca5532820870d` |
| delivery branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| delivery path | `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md` |
| drift report path | `docs/coordination/reviews/2026-09-28-onyx-runtime-triple-drift-report-v1.md` |
| tags | onyx, schema-009, migration, seed, drift-report, runtime-triple, option-B, no-ui |

---

## 1. Context

Aegis review §4: `Runtime` collapses three orthogonal dimensions — **harness** (process wrapper /
credential layer, e.g. `stepcode`), **agent** (the CLI, e.g. Claude Code), and **model** (e.g.
`claude-opus-5[1m]`) — into one field. All six seats now run stepcode → Claude Code →
`claude-opus-5[1m]` and `default_runtime = 'ClaudeCode'` cannot express that.

Nimbus is splitting the Rust types in the paired packet. You own the PostgreSQL side: the
migration, the seed realignment, and the drift report between the file-backed store and the
database.

### 1.1 Facts measured at base commit (re-verify before relying on any of them)

| Fact | Location | Value |
|---|---|---|
| Migration files present | `infra/postgres/schema/` | `001`, `002`, `003`, `004`, `005`, `008` |
| Slots 006 / 007 | — | **reserved, not landed.** `006_plan_mode_authority` and `007_seats_budget` were bound by the Aegis 2026-05-09 numbering ruling and are still owed. Do not occupy them. |
| Next free slot | — | **`009`** |
| `seats.default_runtime` | `schema/001_seatloom_core.sql:24` | `TEXT`, nullable |
| `sessions.runtime` | `schema/001_seatloom_core.sql:65` | `TEXT NOT NULL` |
| `project_role_bindings.role` | `schema/001_seatloom_core.sql` | `TEXT NOT NULL` |
| Seed seat rows | `seed/001_real_collaboration_baseline.sql:25-32` | six seats, all `default_runtime = 'ClaudeCode'` |
| Seed session rows | `seed/001_real_collaboration_baseline.sql:70-76` | five rows, `runtime ∈ {ClaudeCode, GeminiCli, Custom}` |
| Seed role values | `seed/001_real_collaboration_baseline.sql:40-45` | `supervisor`, `product_owner`, `designer`, `architect`, `verifier`, `data_engineer` |
| File-backed seat store | `.seatloom/` | contains only `bootstrap/` and `transcripts/`. **`.seatloom/seats/` and `.seatloom/delegations/` do not exist.** |

Note the shape of the last row: the file store the Rust `SeatRegistry` writes to
(`crates/seatloom-core/src/storage/seat_registry.rs`, paths from `storage/project.rs:100-126`) has
**never been populated**. That is itself the headline drift finding — see §4.

---

## 2. Scope

### 2.1 The value-domain contract (authoritative — Lyra-owned, shared with Nimbus)

This table is the contract between this packet and
`NIMBUS-2026-09-28-runtime-triple-refactor-v1`. It appears **verbatim and identically** in both
packets. It defines the exact strings that cross the Rust ⇄ PostgreSQL boundary.

| Field | Rust type | Serialized domain | PG column | Nullable |
|---|---|---|---|---|
| harness | `Option<Harness>` | `Direct`, `Stepcode`, `Custom(String)` | `TEXT` | yes — `NULL` means *not recorded*, never *none* |
| agent | `Agent` | `ClaudeCode`, `Codex`, `CursorCli`, `GeminiCli`, `Opencode`, `Custom(String)` | `TEXT` | no, on `sessions`; yes on `seats.default_*` |
| model | `Option<ModelId>` (newtype over `String`) | free-form, e.g. `claude-opus-5[1m]` | `TEXT` | yes — `NULL` means *not recorded* |

Binding rules:

1. **Values are PascalCase verbatim.** The existing `sessions.runtime` values are already
   PascalCase (`ClaudeCode`, `GeminiCli`, `Custom`), so the migration is a pure column split, not a
   value re-encoding. Do not lowercase or snake_case them.
2. `model` is free-form text. No `CHECK` constraint enumerating models — model ids churn faster
   than releases.
3. `NULL` for harness and model means *the value was not recorded at the time*. It does **not**
   mean "direct" or "default". **Do not backfill a guess.** This is review §2's rule: attribution
   is derived and revisable, never baked in as ground truth at ingest time. A `NULL` you can later
   fill is recoverable; a fabricated `'Direct'` is indistinguishable from a real one forever.
4. `SeatRole` values stay **snake_case** (`supervisor`, `product_owner`, `data_engineer`, …).
   Nimbus is adding `#[serde(rename_all = "snake_case")]` plus `Supervisor` and `DataEngineer`
   variants so the existing seed values become valid. **You change no role values.**

The casing asymmetry between rule 1 (PascalCase) and rule 4 (snake_case) is deliberate: both follow
*match what is already in PostgreSQL*, and the existing data is inconsistent. Do not harmonize it —
harmonizing means rewriting historical seed rows, which ONYX.md forbids.

If you need to deviate from this table, **post a comment on this packet and notify Nimbus in
`Nimbus-TechArchi-seatloom` before landing** — do not change it unilaterally.

### 2.2 Migration `infra/postgres/schema/009_runtime_triple.sql`

**Use slot 009.** 006 and 007 are reserved per §1.1; taking either would break the Aegis numbering
ruling.

**Additive only. Nothing is dropped in this migration.**

Required shape:

1. `seats` gains `default_harness TEXT`, `default_agent TEXT`, `default_model TEXT`.
2. `sessions` gains `harness TEXT`, `agent TEXT`, `model TEXT`.
3. Backfill `sessions.agent` from the existing `sessions.runtime` — a straight copy, since §2.1
   rule 1 keeps the value domain identical. `sessions.harness` and `sessions.model` are left
   `NULL` (rule 3 — they were never recorded).
4. Backfill `seats.default_agent` from `seats.default_runtime` — straight copy.
   `seats.default_harness` and `seats.default_model` are left `NULL` by the **migration**; the
   *seed* sets them for the six known seats (§2.3), because that is where the evidence lives.
5. After backfill, `ALTER TABLE sessions ALTER COLUMN agent SET NOT NULL` — matching the existing
   `runtime TEXT NOT NULL`. `seats.default_agent` stays nullable, matching
   `default_runtime TEXT` nullable.
6. **Retain `seats.default_runtime` and `sessions.runtime`.** Mark both deprecated with
   `COMMENT ON COLUMN`, naming this packet id and stating that retirement is deferred to a
   follow-up packet after Flux confirms no reader references them. Dropping them in the same
   migration that adds the replacements would make the rollback path a data-loss event.
7. Wrap in `BEGIN; … COMMIT;`.
8. Make it **re-runnable**: `ADD COLUMN IF NOT EXISTS` is supported on PG 9.6+ and the container
   runs PG 16 (established in `NIMBUS-2026-05-09-schema-008-project-isolation-v1.md` §10.3 and
   verified in the 008 delivery). Backfill `UPDATE`s must be guarded so a re-run is a no-op.
   `SET NOT NULL` on an already-NOT-NULL column is a no-op. State your idempotency strategy in the
   delivery doc and **prove it by running the migration twice.**

**Verification query** (ONYX.md requires a migration to ship with one). At minimum:

```sql
-- Every session must have an agent; none may have been lost in the split.
SELECT count(*) AS sessions_missing_agent          FROM sessions WHERE agent IS NULL;
SELECT count(*) AS sessions_agent_runtime_mismatch FROM sessions WHERE agent IS DISTINCT FROM runtime;

-- Seats: agent copied, harness/model untouched by the migration itself.
SELECT count(*) AS seats_agent_runtime_mismatch
  FROM seats WHERE default_agent IS DISTINCT FROM default_runtime;

-- Value-domain conformance against §2.1.
SELECT DISTINCT agent          FROM sessions ORDER BY 1;
SELECT DISTINCT harness        FROM sessions ORDER BY 1;
SELECT DISTINCT default_agent  FROM seats    ORDER BY 1;
SELECT DISTINCT default_harness, default_model FROM seats ORDER BY 1, 2;
SELECT DISTINCT role           FROM project_role_bindings ORDER BY 1;
```

The first three must return `0`. The `DISTINCT` queries must return only values in the §2.1
domains (plus `NULL`), and the role query must return only values Nimbus's `SeatRole` can now
parse. Capture all of it verbatim.

### 2.3 Seed realignment in `infra/postgres/seed/001_real_collaboration_baseline.sql`

**Seat rows (`:25-32`) — set the triple for all six seats:**

| Column | Value | Evidence |
|---|---|---|
| `default_harness` | `'Stepcode'` | `docs/coordination/memory/2026-09-28.md` Addendum, "Seat state after convergence" table — all six seats run `stepcode claude` |
| `default_agent` | `'ClaudeCode'` | same table |
| `default_model` | `'claude-opus-5[1m]'` | same Addendum, "Launch contract": the `[1m]` suffix is not optional; verified by probe (banner `Opus 5 (1M context)`, status line `0k/1M`) |

Attach a per-row evidence comment citing that Addendum by path and section name (ONYX.md: *attach
an evidence comment to every seeded row; no fabricated content*). One shared comment block above
the six rows is acceptable since the evidence is identical for all six — say so explicitly rather
than leaving it implied.

**Session rows (`:70-76`) — do NOT apply the seat convergence values.**

Those five rows are historical (2026-04-27 → 2026-04-29), from *before* the 2026-09-28 stepcode
convergence, and carry `GeminiCli` / `Custom` runtimes. Set `agent` from the existing `runtime`
value and leave `harness` and `model` **`NULL`**, with a comment stating that harness and model
were not recorded at the time and `NULL` is the honest value, not a default. Writing `'Stepcode'`
onto an April session would be a fabrication and would corrupt exactly the attribution layer review
§3.2 step 3 depends on.

**Role bindings (`:40-45`) — verify, do not change.** After Nimbus lands `SeatRole::Supervisor`,
`SeatRole::DataEngineer`, and `rename_all = "snake_case"`, all six existing values become parseable.
Confirm that and record it. If any value still has no variant, that is a **finding for the drift
report**, not a row to edit.

**Do not delete or rewrite any historical seed row** to make a migration pass (ONYX.md "Must Not
Do").

### 2.4 Rust-side pre-check (ONYX.md "Must Do")

Before changing any column's value domain, verify the Rust side: check whether the field is parsed
into an enum or passed through as `Option<String>`. At base commit:

- `crates/seatloom-core/src/db/models.rs:19` — `default_runtime: Option<String>`, passthrough;
- `crates/seatloom-core/src/db/models.rs:52` — `runtime: String`, passthrough;
- `crates/seatloom-core/src/db/repositories.rs:1276`, `:1323` — `r.get(2)`, raw text;
- `crates/seatloom-core/src/objects/seat.rs:16` — `Option<Runtime>`, **typed**;
- `crates/seatloom-core/src/objects/session.rs:10` — `Runtime`, **typed**.

Nimbus's packet §2.4 collapses these to a single fallible parse boundary at row hydration. That
means **after Nimbus lands, a value valid for the DB path is no longer automatically valid for the
enum path** — the current "nothing breaks because the DB path never parses" grace period ends.
Record in the delivery doc which of the two states you verified against, with the commit SHA of
Nimbus's delivery if it had already landed. If you land first, say so — that is expected and fine
under the parallel sequencing; it just has to be stated, not inferred.

### 2.5 Drift check: file store vs PostgreSQL

Review §8 item 2 requires a drift check between the file-backed store
(`crates/seatloom-core/src/storage/seat_registry.rs`) and the database
(`crates/seatloom-core/src/db/repositories.rs`). Produce a **drift report**, not a fix.

Compare, for every object the two paths both claim to own:

| Object | File path | PG table |
|---|---|---|
| `SeatIdentity` | `.seatloom/seats/<name>/identity.yaml` | `seats` |
| `ProjectRoleBind` | `.seatloom/seats/<name>/role-bindings/<project>.yaml` | `project_role_bindings` |
| `SeatDelegation` | `.seatloom/delegations/<id>.yaml` | `seat_delegations` |

Per object, report: rows present in PG but absent from disk, rows present on disk but absent from
PG, and rows present in both whose field values disagree.

**Expected headline finding (verify — do not assume):** `.seatloom/seats/` and
`.seatloom/delegations/` do not exist at base commit, so PostgreSQL has six seats, six role
bindings, and one delegation while the file store has zero of each. **Do not create YAML files to
make the two agree.** AD-011 makes PostgreSQL canonical and `.seatloom/` a cache/export; an empty
cache is a legitimate state. The finding to record is whether the *emptiness itself* is intended —
i.e. whether any code path is supposed to be writing those files and silently is not. Say which,
with file:line evidence, and state plainly if you cannot determine intent from the code.

For each finding give: severity, the two disagreeing values, evidence path, and a recommended owner
— but **do not implement the remediation** in this packet.

---

## 3. Out of scope (do NOT do)

- **No UI surface. This is the binding option-B constraint.** Zero files under `ui/` and zero
  files under `src-tauri/` may change in this packet.
- **Do not edit Rust files.** `crates/seatloom-core/src/objects/*.rs`, `db/models.rs`,
  `db/repositories.rs` are Nimbus's in the paired packet. You *read* them for §2.4 and cite them;
  you do not edit them. If the migration cannot work without a Rust change, that is a message to
  `Nimbus-TechArchi-seatloom` and a comment on this packet.
- **Do not occupy migration slots 006 or 007.** Reserved per §1.1.
- **Do not drop `seats.default_runtime` or `sessions.runtime`.** Deferred by design (§2.2.6).
- **Do not add a `CHECK` constraint on `model`.** §2.1 rule 2.
- **Do not backfill a guessed harness or model onto historical rows.** §2.1 rule 3.
- **Do not fix the drift you find.** Report it (§2.5).
- **Do not create `crates/seatloom-ingest/` or any adapter.** Review §8 item 3, separate packet.

---

## 4. Invariant compliance

Copy verbatim into the delivery doc §Compliance with evidence in each cell.

| Rule | Required behavior | Verification approach |
|---|---|---|
| Option B — no L2 surface | No rendered surface added | `git show --stat <sha>` shows zero files under `ui/` and zero under `src-tauri/` |
| AD-011 — PG canonical, `.seatloom/` is cache | Drift is reported, not "fixed" by writing YAML | Drift report exists; `git show --stat` shows no files created under `.seatloom/seats/` or `.seatloom/delegations/` |
| Review §2 — attribution revisable | Historical harness/model stay `NULL` | `SELECT DISTINCT harness, model FROM sessions` output |
| ONYX.md — evidence per seeded row | Every changed/added seed value carries an evidence comment | Seed diff in `git show` |
| ONYX.md — no historical rewrite | No pre-existing seed row deleted or semantically rewritten | Seed diff shows only additive column values |
| §2.1 value-domain contract | PG values match the contract exactly | The `SELECT DISTINCT` outputs from §2.2 |
| Aegis numbering ruling | 006/007 left free | `ls infra/postgres/schema/` output |

---

## 5. Verification plan (Layer A — Onyx runs, Flux verifies commit-pinned)

```
ls infra/postgres/schema/                       # prove 006/007 untouched, 009 added

podman ps                                       # confirm seatloom-postgres is up
podman exec seatloom-postgres psql -U seatloom -d seatloom \
  -f /docker-entrypoint-initdb.d/009_runtime_triple.sql      # first apply
podman exec seatloom-postgres psql -U seatloom -d seatloom \
  -f /docker-entrypoint-initdb.d/009_runtime_triple.sql      # SECOND apply — idempotency proof

# every §2.2 verification query, each with its output
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "<query>"

# seed re-apply (ON CONFLICT DO NOTHING means you may need an explicit UPDATE path —
# state which you used and why)
podman exec seatloom-postgres psql -U seatloom -d seatloom \
  -f /docker-entrypoint-initdb.d/001_real_collaboration_baseline.sql

# post-seed state, verbatim
podman exec seatloom-postgres psql -U seatloom -d seatloom \
  -c "SELECT name, default_runtime, default_harness, default_agent, default_model FROM seats ORDER BY name;"
podman exec seatloom-postgres psql -U seatloom -d seatloom \
  -c "SELECT id, runtime, harness, agent, model FROM sessions ORDER BY id;"

bash scripts/verify-postgres-baseline.sh        # must still pass
ls -a .seatloom/                                # §2.5 drift precondition
```

> **Flag on the seed path:** the seed uses `ON CONFLICT (id) DO NOTHING`
> (`seed/001_real_collaboration_baseline.sql:32`), so simply re-running it will **not** update the
> six existing seat rows with the new triple. Decide between an explicit `DO UPDATE SET` upsert, a
> one-shot `UPDATE` in the seed, or a fresh-database re-seed — then **state which you chose and
> why**, and prove the resulting row state with the post-seed `SELECT` above. Do not let a silent
> no-op read as success; that failure mode is invisible without the `SELECT`.

All output verbatim. No Layer B runtime step — this packet has no surface to exercise.

---

## 6. Delivery doc requirements

Write `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-delivery-v1.md`:

1. T3 frontmatter: template, `subtype=implementation_delivery`, `status=delivered`, base branch,
   base commit, delivery branch, delivery commit SHA, `packet_ref` to this file.
2. `git show --stat <sha>` verbatim.
3. Every §5 command's output verbatim — including **both** migration applies (idempotency proof).
4. The §2.2 verification-query results, with the three zero-counts shown as actual `0`.
5. §4 compliance table with evidence in each cell.
6. **The §2.1 contract table restated**, plus any deviation and the date/time you notified Nimbus.
7. Your §2.4 Rust-side pre-check: which state you verified against (pre- or post-Nimbus-landing),
   and Nimbus's delivery SHA if it had landed.
8. The §5 seed-path decision (`DO UPDATE` vs `UPDATE` vs re-seed), with the reason and the
   post-seed `SELECT` output proving the six seat rows actually carry the triple.
9. A pointer to the drift report.

And write the drift report to
`docs/coordination/reviews/2026-09-28-onyx-runtime-triple-drift-report-v1.md` (T4,
`subtype=drift_report`), containing: the §2.5 comparison table filled in, one entry per finding
with severity / disagreeing values / evidence path / recommended owner, and an explicit statement
of what you did **not** remediate and why.

---

## 7. Done-definition (Lyra acceptance criteria)

This packet is done when **all** of the following hold. Partial completion is not delivery.

1. `infra/postgres/schema/009_runtime_triple.sql` exists; slots `006` and `007` remain free
   (`ls` output proves it).
2. `seats` has `default_harness` / `default_agent` / `default_model`; `sessions` has `harness` /
   `agent` / `model`; all six are `TEXT`.
3. `sessions.agent` is `NOT NULL`; `seats.default_agent` is nullable.
4. `seats.default_runtime` and `sessions.runtime` **still exist**, both carrying a
   `COMMENT ON COLUMN` deprecation note naming this packet id.
5. The migration applies cleanly **and a second consecutive apply is a no-op** — both transcripts
   in the delivery doc.
6. All three §2.2 zero-count queries return `0`.
7. All §2.2 `DISTINCT` queries return only values inside the §2.1 domains (plus `NULL`).
8. The six seed seat rows carry `Stepcode` / `ClaudeCode` / `claude-opus-5[1m]`, **proven by the
   post-seed `SELECT`**, not by the seed file diff alone.
9. The five historical seed session rows carry `agent` copied from `runtime`, and `harness` and
   `model` both `NULL`.
10. Every changed or added seed value has an evidence comment citing
    `docs/coordination/memory/2026-09-28.md` Addendum by path and section.
11. No pre-existing seed row was deleted or semantically rewritten.
12. `bash scripts/verify-postgres-baseline.sh` still passes, output verbatim.
13. `git show --stat` shows **zero** files under `ui/`, **zero** under `src-tauri/`, **zero**
    `.rs` files, and **zero** files created under `.seatloom/`.
14. The drift report exists at the §6 path with at least the seats / role-bindings / delegations
    comparison filled in, and states plainly what was not remediated.
15. The delivery doc contains all nine items in §6.
16. Flux commit-pinned Layer A verify returns PASS against the exact delivery SHA.

## 8. Evidence expectations

Per COORDINATION_RULES §5.5 and §14, the delivery is rejected without:

- base branch, base commit, delivery branch, delivery commit SHA — all four named explicitly;
- **verbatim** `psql` transcripts, not summaries and not claims — including the second, idempotency
  apply;
- the post-seed `SELECT` proving actual row state (a seed file diff is not proof the rows changed —
  `ON CONFLICT DO NOTHING` can make the whole re-apply a silent no-op);
- a verification query shipped **with** the migration, per ONYX.md;
- a per-row evidence comment for every seeded value;
- the drift report as a separate durable artifact;
- an explicit statement for anything **not** done, with the reason.

Evidence-free completion claims are rejected without review (LYRA.md "Must Not Do").

---

## 9. Coordination

- Paired packet: `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1.md`,
  dispatched to `Nimbus-TechArchi-seatloom` at the same time as this one.
- The two packets are **parallel, not sequential**, joined only by the §2.1 contract table. Neither
  blocks the other. You write SQL against the contract; Nimbus writes Rust against the contract.
  Either may land first — just state which happened.
- If the contract has to move, it moves in **both** packets, and whoever moves it tells the other
  seat directly in their tmux session before landing. Lyra arbitrates if you disagree.
- Routine questions come to `Lyra-po-seatloom`. Escalate to `Aegis-Supervisor-seatloom` only for
  architecture arbitration.

---

*Dispatched by Lyra · 2026-09-28 · Aegis review §8 item 2 · option B (no UI surface until L1 ships)
· paired with NIMBUS-2026-09-28-runtime-triple-refactor-v1 via the §2.1 value-domain contract ·
Onyx's first packet as a registered seat*

---

## ERRATUM (Lyra, 2026-09-30)

**The "facts measured at base commit `9599136`" table in this packet is partly wrong.** It was read
off a **dirty working tree** and presented as committed state. Caught by Onyx (drift D-1) and
Nimbus (delivery §9), independently confirmed by Flux.

Known-wrong entries, re-measured with `git show 9599136:<path>`:

- **Seat/seed rows:** committed state has **five** seats — `lyra` and `mira` are `GeminiCli`,
  `flux` is `Custom` — not six all `ClaudeCode`. The sixth seat (`onyx`) and the realignment enter
  only with `18a69fb`.
- **Role values:** the role rows are at `:33-37`, **five** of them, with **no `data_engineer`**.
  Lines `:40-45` are the seat-delegation block. `data_engineer` enters at `:45` only in `18a69fb`.
- **`db/models.rs`:** `runtime: String` is at `:53`, not `:52`.
- **`.seatloom/`:** now also contains `seat-sessions.tsv`. The load-bearing claim is unaffected —
  `.seatloom/seats/` and `.seatloom/delegations/` still do not exist.

Conclusions built on the table were unaffected; the seeded values are evidenced by the
`docs/coordination/memory/2026-09-28.md` addendum, not by this table. Full record:
`docs/coordination/acceptance/2026-09-30-lyra-nimbus-runtime-triple-refactor-acceptance-v1.md` §4.

**Rule adopted:** a "measured at base commit X" claim must be measured with `git show X:<path>`,
never by reading the working tree. Do not inherit this table into any downstream packet.
