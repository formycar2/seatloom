# Delivery: Runtime Triple Migration — schema 009, seed realignment, drift report

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | ONYX-2026-09-28-runtime-triple-migration-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-09-28 |
| version | v1 |
| packet_ref | `docs/coordination/tasks/onyx/ONYX-2026-09-28-runtime-triple-migration-v1.md` |
| base branch | `track/infra-foundation` |
| base commit | `9599136d615db0c4cdb15811ccaca5532820870d` |
| delivery branch | `packet/onyx/ONYX-2026-09-28-runtime-triple-migration-v1` |
| **delivery commit SHA** | **`629c68e0d64f62f818de472ed9735c65a720132b`** |
| preparatory commit | `18a69fb` — pre-existing uncommitted seed change, NOT authored by this packet (see §7) |
| drift report | `docs/coordination/reviews/2026-09-28-onyx-runtime-triple-drift-report-v1.md` |
| tags | onyx, schema-009, migration, seed, drift-report, runtime-triple, option-B, no-ui |

**Verify this delivery against `629c68e0d64f62f818de472ed9735c65a720132b`.**

---

## 0. Summary

Schema 009 lands the harness / agent / model triple additively; the seed carries the triple for the
six converged seats and deliberately leaves it `NULL` for the five historical April sessions; the
§2.5 drift report is delivered with seven findings.

**Two packet §5 commands could not be executed as written, and one §7 criterion is unmet.** Both are
pre-existing environment/repo defects, proved at base commit before any edit of mine. They are
stated plainly in §9 and §10 rather than worked around — nothing below is a claim without a
transcript.

---

## 1. `git show --stat` (verbatim)

```
$ git show --stat 629c68e0d64f62f818de472ed9735c65a720132b
commit 629c68e0d64f62f818de472ed9735c65a720132b
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Mon Sep 28 21:53:47 2026 +0800

    feat(schema): 009 runtime triple — split Runtime into harness/agent/model
    [full message in git; abridged here]

 ...26-09-28-onyx-runtime-triple-drift-report-v1.md | 376 +++++++++++++++++++++
 infra/postgres/schema/009_runtime_triple.sql       | 176 ++++++++++
 .../seed/001_real_collaboration_baseline.sql       | 109 +++++-
 scripts/bootstrap.sh                               |   3 +-
 scripts/verify-postgres-baseline.sh                |  14 +-
 5 files changed, 661 insertions(+), 17 deletions(-)
```

Scope proof (§7.13), verbatim:

```
--- .tsx files: ---
ZERO
--- files under ui/: ---
ZERO
--- files under src-tauri/: ---
ZERO
--- .rs files: ---
ZERO
--- files under .seatloom/: ---
ZERO
--- files under infra/postgres/ (expected: 2): ---
infra/postgres/schema/009_runtime_triple.sql
infra/postgres/seed/001_real_collaboration_baseline.sql
```

Two files outside the packet's named deliverables are in this commit — `scripts/bootstrap.sh` and
`scripts/verify-postgres-baseline.sh`. Disclosed and justified in §8. Neither is under `ui/`,
`src-tauri/`, `crates/`, or `.seatloom/`.

---

## 2. §2.1 value-domain contract — restated

Reproduced verbatim from the packet. **No deviation. Nothing to notify Nimbus about.**

| Field | Rust type | Serialized domain | PG column | Nullable |
|---|---|---|---|---|
| harness | `Option<Harness>` | `Direct`, `Stepcode`, `Custom(String)` | `TEXT` | yes — `NULL` means *not recorded*, never *none* |
| agent | `Agent` | `ClaudeCode`, `Codex`, `CursorCli`, `GeminiCli`, `Opencode`, `Custom(String)` | `TEXT` | no, on `sessions`; yes on `seats.default_*` |
| model | `Option<ModelId>` (newtype over `String`) | free-form, e.g. `claude-opus-5[1m]` | `TEXT` | yes — `NULL` means *not recorded* |

Compliance per binding rule:

1. **PascalCase kept verbatim.** `Stepcode`, `ClaudeCode`, `GeminiCli`, `Custom` are written
   exactly as-is. Nothing lowercased or snake_cased; this is a pure column split. Proof: the
   `SELECT DISTINCT` outputs in §4.
2. **No `CHECK` constraint on `model`** — and none on `harness`/`agent` either, since `Custom` is an
   open escape hatch on both and the authoritative domain check is Nimbus's fallible parse boundary.
3. **No guessed backfill.** The migration writes `NULL` for every `harness` and `model`; the seed
   writes them only for seats, where the Addendum supplies evidence. §4 proves
   `count(harness) = 0` and `count(model) = 0` across all five sessions.
4. **No role value changed.** `project_role_bindings.role` is untouched by both migration and seed;
   §4's role query is a read-only confirmation.

**Deviation notification date: N/A — no deviation occurred.**

---

## 3. Migration applies — both transcripts (§7.5 idempotency proof)

Setup was deliberately the **realistic upgrade path**: schema 001-005 plus the *pre-009* seed, so
the backfill had real rows to act on. (A fresh-database run is additionally shown in §6.)

Pre-009 state — triple columns provably absent:

```
$ podman exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT name, default_runtime FROM seats ORDER BY name;"
  name  | default_runtime
--------+-----------------
 aegis  | ClaudeCode
 flux   | ClaudeCode
 lyra   | ClaudeCode
 mira   | ClaudeCode
 nimbus | ClaudeCode
 onyx   | ClaudeCode
(6 rows)

$ ... "SELECT table_name, column_name FROM information_schema.columns WHERE (table_name='seats' AND column_name LIKE 'default_%') OR (table_name='sessions' AND column_name IN ('runtime','harness','agent','model')) ORDER BY table_name, column_name;"
 table_name |   column_name
------------+-----------------
 seats      | default_runtime
 sessions   | runtime
(2 rows)
```

### 3.1 FIRST apply — verbatim

```
############ FIRST APPLY of 009_runtime_triple.sql ############
BEGIN
ALTER TABLE
ALTER TABLE
ALTER TABLE
ALTER TABLE
ALTER TABLE
ALTER TABLE
UPDATE 5
UPDATE 6
ALTER TABLE
COMMENT
COMMENT
COMMENT
COMMENT
COMMENT
COMMENT
COMMENT
COMMENT
COMMIT
EXIT=0
```

`UPDATE 5` = five session rows backfilled `agent <- runtime`. `UPDATE 6` = six seat rows backfilled
`default_agent <- default_runtime`. The backfill demonstrably did work.

### 3.2 SECOND apply — verbatim (idempotency)

```
############ SECOND APPLY of 009_runtime_triple.sql (idempotency proof) ############
BEGIN
psql:/tmp/009_runtime_triple.sql:72: NOTICE:  column "default_harness" of relation "seats" already exists, skipping
ALTER TABLE
ALTER TABLE
psql:/tmp/009_runtime_triple.sql:73: NOTICE:  column "default_agent" of relation "seats" already exists, skipping
psql:ALTER TABLE
/tmp/009_runtime_triple.sql:74: NOTICE:  column "default_model" of relation "seats" already exists, skipping
ALTER TABLE
psql:/tmp/009_runtime_triple.sql:80: NOTICE:  column "harness" of relation "sessions" already exists, skipping
psql:/tmp/009_runtime_triple.sql:81: NOTICE:  column "agent" of relation "sessions" already exists, skipping
ALTER TABLE
psql:/tmp/009_runtime_triple.sql:82: ALTER TABLE
NOTICE:  column "model" of relation "sessions" already exists, skipping
UPDATE 0
UPDATE 0
ALTER TABLE
COMMENT
COMMENT
COMMENT
COMMENT
COMMENT
COMMENT
COMMENT
COMMENT
COMMIT
EXIT=0
```

`UPDATE 0` / `UPDATE 0` and six `already exists, skipping` notices: **the second apply is a
no-op.** (The interleaved `psql:` prefixes are stdout/stderr interleaving, not corruption.)

**Idempotency strategy** (as stated in the migration header): `ADD COLUMN IF NOT EXISTS`; every
backfill `UPDATE` guarded by `WHERE <target> IS NULL`; `SET NOT NULL` is a no-op when already set;
`COMMENT ON COLUMN` is an unconditional overwrite; whole file in one transaction. The guard is
deliberately `IS NULL` rather than `IS DISTINCT FROM <source>` — once a writer legitimately sets an
`agent` that differs from the legacy `runtime`, a re-run must not clobber it.

---

## 4. §2.2 verification queries — verbatim

Canonical run, post-migration and post-seed. (Identical results were obtained post-migration
*pre-seed*, except that `default_harness`/`default_model` were `NULL` then — which is exactly the
point: the migration writes no guess, the seed supplies the evidence-backed values.)

```
 sessions_missing_agent
------------------------
                      0
(1 row)

 sessions_agent_runtime_mismatch
---------------------------------
                               0
(1 row)

 seats_agent_runtime_mismatch
------------------------------
                            0
(1 row)

   agent
------------
 ClaudeCode
 Custom
 GeminiCli
(3 rows)

 harness
---------

(1 row)

 default_agent
---------------
 ClaudeCode
(1 row)

 default_harness |   default_model
-----------------+-------------------
 Stepcode        | claude-opus-5[1m]
(1 row)

     role
---------------
 architect
 data_engineer
 designer
 product_owner
 supervisor
 verifier
(6 rows)
```

**All three zero-counts are actual `0`** (§7.6). Domain conformance (§7.7):

| Query | Values returned | Inside §2.1 domain? |
|---|---|---|
| `DISTINCT agent` (sessions) | `ClaudeCode`, `Custom`, `GeminiCli` | yes — all three in the agent domain |
| `DISTINCT harness` (sessions) | `NULL` only | yes — `NULL` permitted, means *not recorded* |
| `DISTINCT default_agent` (seats) | `ClaudeCode` | yes |
| `DISTINCT default_harness, default_model` | `Stepcode`, `claude-opus-5[1m]` | yes — `Stepcode` in domain; `model` free-form |
| `DISTINCT role` | 6 snake_case values | yes — see §4.1 |

Post-migration/pre-seed `harness`+`model` NULL-ness, separately proved:

```
$ ... "SELECT count(*) AS sessions_total, count(harness) AS sessions_with_harness, count(model) AS sessions_with_model FROM sessions;"
 sessions_total | sessions_with_harness | sessions_with_model
----------------+-----------------------+---------------------
              5 |                     0 |                   0
(1 row)
```

### 4.1 Role-binding verification (§2.3 "verify, do not change")

All six values map onto a `SeatRole` variant once Nimbus lands `Supervisor`, `DataEngineer`, and
`rename_all = "snake_case"`:

| Seed value | `SeatRole` variant | Status |
|---|---|---|
| `architect` | `Architect` | pre-existing |
| `data_engineer` | `DataEngineer` | **new in Nimbus's packet** (§2.3.3) |
| `designer` | `Designer` | pre-existing |
| `product_owner` | `ProductOwner` | pre-existing |
| `supervisor` | `Supervisor` | **new in Nimbus's packet** (§2.3.2) |
| `verifier` | `Verifier` | pre-existing |

**Zero values fall through to `Custom(String)`. No finding.** Confirms the packet's premise that
`DataEngineer` was needed: without it `data_engineer` would have been the one unparseable value.
No role value was changed by this packet.

### 4.2 Structural criteria (§7.2, §7.3, §7.4)

```
$ ... information_schema.columns
 table_name |   column_name   | data_type | is_nullable
------------+-----------------+-----------+-------------
 seats      | default_agent   | text      | YES
 seats      | default_harness | text      | YES
 seats      | default_model   | text      | YES
 seats      | default_runtime | text      | YES
 sessions   | agent           | text      | NO
 sessions   | harness         | text      | YES
 sessions   | model           | text      | YES
 sessions   | runtime         | text      | NO
(8 rows)
```

All six new columns `TEXT` (§7.2 ✔). `sessions.agent` `NOT NULL`, `seats.default_agent` nullable
(§7.3 ✔). Both legacy columns still present (§7.4 ✔) and carry the deprecation note:

```
 table_name |   column_name   |  comment
------------+-----------------+----------
 seats      | default_runtime | DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1. Superseded by default_harness / default_agent / default_model. Retained, not dropped: column retirement is deferred to a follow-up packet after Flux confirms no reader references this column. Read default_agent instead.
 sessions   | runtime         | DEPRECATED by ONYX-2026-09-28-runtime-triple-migration-v1. Superseded by harness / agent / model. Retained, not dropped: column retirement is deferred to a follow-up packet after Flux confirms no reader references this column. Read agent instead.
(2 rows)
```

### 4.3 Slot discipline (§7.1)

```
$ ls -1 infra/postgres/schema/
001_seatloom_core.sql
002_document_authority.sql
003_write_ingest_reconcile.sql
004_operational_review_and_continuity.sql
005_prompt_and_channel_action_authority.sql
008_project_isolation.sql
009_runtime_triple.sql
```

009 added; **006 and 007 remain free** per the Aegis 2026-05-09 numbering ruling.

---

## 5. Seed-path decision (§5 flag, §6 item 8)

**Chosen: `ON CONFLICT (id) DO UPDATE SET`, scoped strictly to the new columns.** Rejected
alternatives: a one-shot `UPDATE` in the seed (a second source of truth for the same values,
divergent the moment one is edited), and fresh-database-only re-seed (would make the seed unusable
as a convergence tool on any existing database — exactly the silent-no-op trap the packet flags).

- seats: `SET default_harness, default_agent, default_model` — **only** the three new columns.
- sessions: `SET agent` — **only** `agent`. `harness`/`model` are deliberately *excluded* from the
  `SET` list so that if a later evidence-backed packet fills them in, re-running the seed does not
  wipe that value back to `NULL`.

Every pre-existing column (`name`, `default_runtime`, `capability_tags`, `status`, `created_at`,
`workspace_path`, `branch`, …) is absent from both `SET` lists, so no historical value can be
rewritten by a re-apply. This is what keeps the change additive (§7.11).

Seed apply onto rows that already existed — verbatim:

```
########## SEED RE-APPLY #1 (onto rows that already exist) ##########
BEGIN
INSERT 0 0
INSERT 0 6     <- seats: DO UPDATE fired on all six existing rows
INSERT 0 0
INSERT 0 0
INSERT 0 5     <- sessions: DO UPDATE fired on all five existing rows
INSERT 0 0
[... 17 further "INSERT 0 0" — every other block is DO NOTHING and no-ops ...]
COMMIT
EXIT=0
```

**Post-seed `SELECT` — the §7.8 / §8 proof that rows actually changed, not the diff:**

```
  name  | default_runtime | default_harness | default_agent |   default_model
--------+-----------------+-----------------+---------------+-------------------
 aegis  | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 flux   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 lyra   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 mira   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 nimbus | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
 onyx   | ClaudeCode      | Stepcode        | ClaudeCode    | claude-opus-5[1m]
(6 rows)

            id            |  runtime   | harness |   agent    | model
--------------------------+------------+---------+------------+-------
 ses-aegis-supervisor-001 | ClaudeCode |         | ClaudeCode |
 ses-flux-verify-001      | Custom     |         | Custom     |
 ses-lyra-coord-001       | GeminiCli  |         | GeminiCli  |
 ses-mira-ui-001          | GeminiCli  |         | GeminiCli  |
 ses-nimbus-infra-001     | ClaudeCode |         | ClaudeCode |
(5 rows)
```

All six seats carry `Stepcode` / `ClaudeCode` / `claude-opus-5[1m]` (§7.8 ✔). All five historical
sessions carry `agent` copied from `runtime` with `harness` and `model` blank = `NULL` (§7.9 ✔).

A second seed apply reproduced `INSERT 0 6` / `INSERT 0 5` with byte-identical row state — the
`DO UPDATE` is convergent, not accumulating.

### 5.1 Per-row evidence comments (§7.10)

`seed/001_real_collaboration_baseline.sql` carries two evidence blocks, both citing
`docs/coordination/memory/2026-09-28.md` by path and section name:

- **Seats** — one shared block for all six rows, with the packet's required explicit statement that
  it is shared because the evidence is identical for all six. Cites Addendum — Seat Harness
  Convergence, section **"Seat state after convergence"** for `Stepcode` and `ClaudeCode`, and
  section **"Launch contract"** for `claude-opus-5[1m]` including the probe evidence (banner
  `Opus 5 (1M context)`, status line `0k/1M`) and why the `[1m]` suffix is not optional.
- **Sessions** — states that `agent` is a rename-in-place of an existing value (no new fact
  asserted), and that `harness`/`model` are `NULL` because these are April sessions predating the
  September convergence, with the explicit reasoning that writing `'Stepcode'` there would be a
  fabrication corrupting the attribution layer review §3.2 step 3 depends on. It also names the
  deliberate asymmetry with the seat rows.

---

## 6. Additional evidence: clean-room (fresh-database) run

The §3 run proved the *upgrade* path. This proves the *fresh install* path, where 009 runs against
**empty** tables (so `SET NOT NULL` is applied before any row exists) and the seed takes its plain
`INSERT` branch rather than `DO UPDATE`:

```
applying schema/001_seatloom_core.sql               OK
applying schema/002_document_authority.sql          OK
applying schema/003_write_ingest_reconcile.sql      OK
applying schema/004_operational_review_and_continuity.sql OK
applying schema/005_prompt_and_channel_action_authority.sql OK
applying schema/009_runtime_triple.sql              OK

BEGIN
INSERT 0 1     <- projects
INSERT 0 6     <- seats (true inserts)
INSERT 0 6     <- project_role_bindings
INSERT 0 1     <- seat_delegations
INSERT 0 5     <- sessions
```

Resulting rows were **identical** to the upgrade path in §5 (six seats with the full triple; five
sessions with `agent` set and `harness`/`model` `NULL`), and all §2.2 queries returned the §4
values. Both paths converge on the same state.

Also verified: **009 coexists with 008.** Applying 008 *after* the seed and then 009 again:

```
ALTER TABLE
CREATE INDEX x4
COMMIT
008 EXIT=0
UPDATE 0
UPDATE 0
COMMIT
009-after-008 EXIT=0
```

Container is `postgres:16-alpine`, confirming the packet's PG-16 premise:

```
PostgreSQL 16.14 on aarch64-unknown-linux-musl, compiled by gcc (Alpine 15.2.0) 15.2.0, 64-bit
```

---

## 7. §2.4 Rust-side pre-check — which state I verified against

**I verified against the PRE-Nimbus-landing state, and I landed first.**

At my base commit `9599136` the Rust side is exactly as the packet describes — a string-keyed DB
path that never parses into the enum:

- `crates/seatloom-core/src/db/models.rs:19` — `default_runtime: Option<String>`, passthrough
- `crates/seatloom-core/src/db/models.rs:52` — `runtime: String`, passthrough
- `crates/seatloom-core/src/db/repositories.rs:1276`, `:1323` — `r.get(2)`, raw text
- `crates/seatloom-core/src/objects/seat.rs:16` — `Option<Runtime>`, **typed**
- `crates/seatloom-core/src/objects/session.rs:10` — `Runtime`, **typed**

**Nimbus's delivery SHA: none at time of writing.** Nimbus's refactor was briefly visible as
uncommitted modifications in this shared working tree during my run and was then withdrawn to
Nimbus's own branch; it is not committed on `track/infra-foundation` and has no delivery SHA I can
pin. So the "grace period" the packet describes — a value valid for the DB path but not yet for the
enum path — was still open when I landed.

This is the expected and permitted ordering under the parallel sequencing (packet §9: "Either may
land first — just state which happened"). Stated, not inferred.

**Consequence for Nimbus, worth an explicit word:** because the value domains in §2 were kept
byte-identical, every value now in PostgreSQL is already parseable by the `Harness`/`Agent`/
`SeatRole` variants Nimbus's packet specifies. The §4 `SELECT DISTINCT` outputs are the full set of
strings Nimbus's parse boundary will encounter on this database: agents `{ClaudeCode, Custom,
GeminiCli}`, harnesses `{NULL, Stepcode}`, roles the six snake_case values in §4.1. Nothing in the
database requires a `Custom(...)` fallback, and nothing should make the boundary fail loudly.

### 7.1 Preparatory commit `18a69fb` — pre-existing change, not mine

Packet §1.1 lists as "measured at base commit" that the seed has six seats all `ClaudeCode` and a
`data_engineer` role value. **Both are false at `9599136`** — they existed only as an uncommitted
working-tree modification. Base commit has five seats with mixed runtimes and no `data_engineer`.

To keep this packet's diff auditable I committed that pre-existing change **separately and first**,
as `18a69fb`, attributed in its message as not authored by this packet. Its content, verbatim
(`diff` of base commit vs the committed pre-existing state):

```
+  ('seat-lyra-001',  'lyra',   'ClaudeCode',  ...      (was 'GeminiCli')
+  ('seat-mira-001',  'mira',   'ClaudeCode',  ...      (was 'GeminiCli')
+  ('seat-flux-001',  'flux',   'ClaudeCode',  ...      (was 'Custom')
+  ('seat-onyx-001',  'onyx',   'ClaudeCode',  ...      (new sixth seat)
+  ('seat-onyx-001',  'seatloom', 'data_engineer', ...  (new role binding)
+ the 2026-09-28 runtime-convergence comment block
```

No triple column appears in it — confirming it is the Addendum decision 3/5 realignment, not my
work. Recorded as drift finding **D-1**; Lyra owns correcting the §1.1 fact table.

---

## 8. Files changed beyond the packet's named deliverables

Disclosed rather than left to be discovered:

| File | Change | Why it was necessary |
|---|---|---|
| `scripts/verify-postgres-baseline.sh` | added `009_runtime_triple.sql` to the schema list (+ a comment recording D-3/D-4/D-5) | **Required.** Seed 001 now writes the six triple columns. Applying the seed against a database without 009 fails with `column ... does not exist`, so without this edit the packet would have broken the verifier outright. 009 is independent of 008 (touches only columns created in 001), so adding it alone is safe. |
| `scripts/bootstrap.sh` | added `009_runtime_triple.sql` after `008` | Same reason — otherwise the documented bring-up path breaks. |

Neither script's runtime detection, ordering, or destructive semantics were altered. I deliberately
did **not** add `008` to the verifier — see §10 and drift finding D-4.

---

## 9. §5 commands that could not be executed as written

### 9.1 `/docker-entrypoint-initdb.d/` does not exist — packet §5 path is stale

Packet §5 specifies applying the migration via
`podman exec seatloom-postgres psql ... -f /docker-entrypoint-initdb.d/009_runtime_triple.sql`.
**That directory is never populated.** `infra/postgres/docker-compose.yml` mounts only the data
volume:

```
$ grep -n 'initdb' infra/postgres/docker-compose.yml
NO initdb mount in compose
```

`verify-postgres-baseline.sh:24-26` says so explicitly: *"docker-compose.yml does NOT mount schema
files into /docker-entrypoint-initdb.d, so there is no hidden auto-apply path during container
init."*

**Substitution used:** `podman cp <file> seatloom-postgres:/tmp/<file>` followed by
`podman exec ... psql -v ON_ERROR_STOP=1 -f /tmp/<file>` — the same mechanism both repo scripts use.
Semantically identical, and it additionally proves there is no hidden auto-apply masking the result.
Recommend Lyra correct the §5 command text.

### 9.2 `podman ps` did not show the container; the podman VM was stopped

`podman ps` initially returned a connection error (VM stopped) and, once started, no
`seatloom-postgres` container existed at all — so the packet's "confirm seatloom-postgres is up"
step had nothing to confirm. I started the VM (`podman machine start`) and created the container
with the same `podman run` invocation `bootstrap.sh:106-114` uses. Every transcript above comes from
that container.

---

## 10. §7.12 NOT MET — `verify-postgres-baseline.sh` cannot run on this seat

**This is the one done-definition criterion I did not satisfy, and I am not claiming otherwise.**

```
$ bash scripts/verify-postgres-baseline.sh
EXIT CODE = 127
...
--- Step 0: Static cross-seed consistency ---
running 3 tests
test postgres_seed_consistency_ingest_paths_exist_in_repo ... ok
test postgres_seed_consistency_document_artifact_refs_exist_in_baseline ... ok
test postgres_seed_consistency_ingest_doc_ids_match_seeded_documents ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

--- Step 1: Teardown stale volume + start clean container ---
(destructive reset — ensures deterministic schema + seed state)
scripts/verify-postgres-baseline.sh: line 59: docker: command not found
scripts/verify-postgres-baseline.sh: line 60: docker: command not found
```

**Cause:** the script hardcodes `docker` (`:59-60` `docker compose down -v` / `up -d`), unlike
`bootstrap.sh`, which auto-detects the runtime. This seat has podman only and **no compose provider
of any kind** (`docker` absent; `podman compose` finds no provider; no `Docker.app`).

**Proved pre-existing, not caused by this packet:** the identical failure was captured by running
the script at base commit `9599136` **before any edit of mine**. Full evidence in drift report D-5.

**What did pass:** step 0, the pure-Rust static seed-consistency gate — the only step that runs
without a container — passes **with my edited seed**, 3/3 tests. That is the one part of the
verifier my change could have broken, and it does not.

**Not remediated** because porting the verifier to runtime-agnostic container handling changes what
a Flux PASS means, and Flux owns that harness. Recommended owner and the concrete fix (lift
`bootstrap.sh:44-71`) are in D-5.

**Request to Flux:** run §7.12 on a seat that has Docker. Everything else in §7 is evidenced above.

---

## 11. §4 invariant compliance

| Rule | Required behavior | Evidence |
|---|---|---|
| Option B — no L2 surface | No rendered surface added | §1 scope proof: `git show --stat 629c68e` → `ZERO` files under `ui/`, `ZERO` under `src-tauri/`, `ZERO` `.tsx`. The 5 changed files are 2× `infra/postgres/`, 2× `scripts/`, 1× `docs/`. |
| AD-011 — PG canonical, `.seatloom/` is cache | Drift reported, not "fixed" by writing YAML | Drift report exists at `docs/coordination/reviews/2026-09-28-onyx-runtime-triple-drift-report-v1.md` (376 lines, 7 findings). §1 scope proof: `ZERO` files under `.seatloom/`. `find .seatloom -name '*.yaml'` still returns only the pre-existing `bootstrap/source-map.yaml`. |
| Review §2 — attribution revisable | Historical harness/model stay `NULL` | §4: `SELECT DISTINCT harness FROM sessions` → `NULL` only; `count(harness)=0, count(model)=0` of 5 rows; §5 post-seed `SELECT` shows blank `harness`/`model` on all five April rows. Migration contains no `unwrap_or`-equivalent default — `009_runtime_triple.sql` steps 3-4 write only `agent`. |
| ONYX.md — evidence per seeded row | Every changed/added seed value carries an evidence comment | §5.1; seed diff in `git show 629c68e -- infra/postgres/seed/001_real_collaboration_baseline.sql`. Two evidence blocks citing `docs/coordination/memory/2026-09-28.md` Addendum sections "Seat state after convergence" and "Launch contract" by name. |
| ONYX.md — no historical rewrite | No pre-existing seed row deleted or semantically rewritten | Seed data-line diff (§11.1 below): every pre-existing column value byte-identical; only new columns and the `ON CONFLICT` clause change. Both `DO UPDATE SET` lists contain **only** new columns. Row counts unchanged (6 seats, 5 sessions, 6 bindings, 1 delegation). |
| §2.1 value-domain contract | PG values match the contract exactly | §4 `SELECT DISTINCT` outputs, mapped row-by-row against the domain table in §2. No deviation. |
| Aegis numbering ruling | 006/007 left free | §4.3 `ls -1 infra/postgres/schema/` — jumps 005 → 008 → 009. |

### 11.1 Seed diff, data lines only (no-historical-rewrite proof)

```
-INSERT INTO seats (id, name, default_runtime, capability_tags, status, created_at) VALUES
+INSERT INTO seats (id, name, default_runtime, default_harness, default_agent, default_model, capability_tags, status, created_at) VALUES
-  ('seat-aegis-001', 'aegis',  'ClaudeCode',  ARRAY['supervision',...], 'active', '2026-04-24T00:00:00Z'),
+  ('seat-aegis-001', 'aegis',  'ClaudeCode', 'Stepcode', 'ClaudeCode', 'claude-opus-5[1m]', ARRAY['supervision',...], 'active', '2026-04-24T00:00:00Z'),
   [... same shape for lyra, mira, nimbus, flux, onyx ...]
-ON CONFLICT (id) DO NOTHING;
+ON CONFLICT (id) DO UPDATE SET
+  default_harness = EXCLUDED.default_harness,
+  default_agent   = EXCLUDED.default_agent,
+  default_model   = EXCLUDED.default_model;

-INSERT INTO sessions (id, seat_id, runtime, workspace_path, branch, status, created_at, ended_at) VALUES
+INSERT INTO sessions (id, seat_id, runtime, agent, harness, model, workspace_path, branch, status, created_at, ended_at) VALUES
-  ('ses-lyra-coord-001', 'seat-lyra-001', 'GeminiCli',  '/Users/.../seatloom', 'main', 'completed', ...),
+  ('ses-lyra-coord-001', 'seat-lyra-001', 'GeminiCli',  'GeminiCli',  NULL, NULL, '/Users/.../seatloom', 'main', 'completed', ...),
   [... same shape for the other four ...]
-ON CONFLICT (id) DO NOTHING;
+ON CONFLICT (id) DO UPDATE SET
+  agent = EXCLUDED.agent;
```

Every `id`, `name`, `default_runtime`, `capability_tags`, `status`, `created_at`, `seat_id`,
`runtime`, `workspace_path`, `branch`, and timestamp is unchanged. Nothing deleted.

---

## 12. Dimension assignment, recorded for future readers

Stated in the migration header so it survives independently of this doc:

- **`stepcode` / `stepgo` is a HARNESS, not an agent.** It is the process wrapper / credential /
  proxy layer. There must never be an `Agent::Stepcode`.
- **`opencode` is an AGENT, not a harness.** It is a CLI being driven, hence `Agent::Opencode` in
  the §2 domain and no `Harness::Opencode`.

One enum could not express that distinction; three columns can. This is the resolution of the
"missing variants" issue Aegis review §4 raised — it is resolved by giving the two values different
*dimensions*, not by adding variants to a single enum.

---

## 13. Anything contradicting the packet's stated facts

| Packet claim | Reality | Drift finding |
|---|---|---|
| §1.1 "Seed seat rows — six seats, all `default_runtime = 'ClaudeCode'`" | **False at base commit** — five seats, mixed runtimes, no `onyx`. True only of the uncommitted working tree. | D-1 |
| §1.1 "Seed role values — …, `data_engineer`" | **False at base commit** — five bindings, no `data_engineer`. | D-1 |
| §5 apply via `/docker-entrypoint-initdb.d/…` | Directory never populated; compose mounts no schema. | §9.1 |
| §5 `podman ps` "confirm seatloom-postgres is up" | Podman VM was stopped and the container did not exist. | §9.2 |
| §5 / §7.12 `verify-postgres-baseline.sh` "must still pass" | Cannot run on this seat — hardcodes `docker`, exit 127. Pre-existing at base commit. | D-5 |
| §1.1 `.seatloom/` contains only `bootstrap/` and `transcripts/` | **Held** at start. An untracked `seat-sessions.tsv` appeared later in the session (seat tooling, not this packet). `seats/` and `delegations/` still absent. | D-7 |
| — | Schema 008 and seed 001 are mutually inapplicable (`sessions.project_id` `NOT NULL`, no default, seed omits it) → `bootstrap.sh` cannot succeed on a fresh volume. | **D-3 (P0)** |
| — | `verify-postgres-baseline.sh` omits 008 while `bootstrap.sh` applies it — the two schema authorities disagree. | D-4 |
| — | `infra/README.md` still says 5 seats. | D-6 |

Confirmed unchanged: slots 006/007 reserved; `seats.default_runtime` `TEXT` nullable;
`sessions.runtime` `TEXT NOT NULL`; `project_role_bindings.role` `TEXT NOT NULL`; the five
historical session runtimes `{ClaudeCode, GeminiCli, Custom}`; PG 16.

---

## 14. Done-definition status (§7)

| # | Criterion | Status |
|---|---|---|
| 1 | 009 exists; 006/007 free | ✅ §4.3 |
| 2 | 6 new columns, all `TEXT` | ✅ §4.2 |
| 3 | `sessions.agent` NOT NULL; `seats.default_agent` nullable | ✅ §4.2 |
| 4 | legacy columns retained + `COMMENT` naming packet id | ✅ §4.2 |
| 5 | applies cleanly; 2nd apply is a no-op | ✅ §3.1, §3.2 |
| 6 | three zero-counts return `0` | ✅ §4 |
| 7 | `DISTINCT` queries inside §2.1 domains | ✅ §4 |
| 8 | six seats carry the triple, proven by post-seed `SELECT` | ✅ §5 |
| 9 | five sessions: `agent` copied, `harness`/`model` `NULL` | ✅ §5 |
| 10 | evidence comment for every seeded value | ✅ §5.1 |
| 11 | no pre-existing seed row deleted/rewritten | ✅ §11.1 |
| 12 | `verify-postgres-baseline.sh` passes | ❌ **NOT MET** — §10, pre-existing blocker D-5. Step 0 passes; steps 1+ need Docker. |
| 13 | zero `ui/`, `src-tauri/`, `.rs`, `.seatloom/` files | ✅ §1 |
| 14 | drift report with all three objects compared | ✅ drift report §1, 7 findings |
| 15 | delivery doc contains all nine §6 items | ✅ §1(2), §2(5), §3-6(3,4), §5(8), §7(7), §11(5), §12(8), §13(9), §14 |
| 16 | Flux commit-pinned Layer A PASS | ⏳ pending Flux against `629c68e0d64f62f818de472ed9735c65a720132b` |

**15 of 16 met. One (§7.12) is blocked by a pre-existing environment defect proved to predate this
packet; it needs a Docker-capable seat, not a change to this delivery.**

---

## 15. Coordination notes

- **To Nimbus (`Nimbus-TechArchi-seatloom`):** contract unchanged — **no deviation, nothing to
  reconcile**. I landed first; you have no delivery SHA yet (§7). The complete set of strings your
  parse boundary will meet on this database is in §4: agents `{ClaudeCode, Custom, GeminiCli}`,
  harnesses `{NULL, Stepcode}`, roles the six snake_case values in §4.1 — all parseable by the
  variants your packet specifies, including the `DataEngineer` addition, which §4.1 confirms was
  needed.
- **To Lyra (`Lyra-po-seatloom`):** four packet-text corrections needed — §1.1's two seed facts
  (D-1), the §5 `/docker-entrypoint-initdb.d/` path (§9.1), and §7.12's runnability on a
  podman-only seat (D-5). Plus a decision on D-3 (P0: `bootstrap.sh` broken on a fresh volume),
  which I did not fix because it is outside this packet's scope.
- **To Flux (`Flux-Quality&Ops-seatloom`):** verify against
  `629c68e0d64f62f818de472ed9735c65a720132b`. §7.12 needs a Docker-capable seat. Note D-4 before
  treating a verifier PASS as covering `project_id`.

---

*Onyx · 2026-09-28 · delivery for ONYX-2026-09-28-runtime-triple-migration-v1 · Aegis review §8
item 2 · option B (no UI surface until L1 ships) · Onyx's first packet as a registered seat*
