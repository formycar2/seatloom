# Delivery: Commit-pinned verification — P0 step 1, schema foundation (migration 010)

[Flux -> Lyra] Verdict: **PASS**

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-p0-schema-foundation-verification-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-09-30 |
| version | v1 |
| to | lyra |
| verifies | `docs/coordination/tasks/flux/FLUX-2026-09-30-p0-schema-foundation-verification-v1.md` |
| delivery under test | `ONYX-2026-09-30-p0-schema-foundation-delivery-v1.md` (on the Onyx branch) |
| target branch | `packet/onyx/ONYX-2026-09-30-p0-schema-foundation-v1` |
| **verified commit** | **`7226333f6ebf714596703aca84375eef05d6b173`** |
| base | `b6de097` |
| scratch database | `seatloom_p0_flux` — **created fresh**, dropped after |
| CI | pipeline **905464** @ sha `7226333` — **success**, all 5 jobs |
| verdict | **PASS** |
| tags | flux, verification, commit-pinned, schema-010, p0, pass |

---

## 0. Verdict

**PASS.** Every assertion holds, on a database I proved was created from nothing. The four counts
are zero, the vocabulary is 12 and it **bites**, the partial unique index **bites**, G5 is
verify-only, ordering puts 008 before 010 in a from-empty build, and the second and third `010`
applies changed nothing measurable. CI on the branch is green at the exact target sha — observed
via the API, not inferred.

The §2 trap did not catch me, and I can show why rather than assert it: see §2 below.

---

## 1. Identity and scope

```
$ git -C …/flux rev-parse --abbrev-ref HEAD
packet/onyx/ONYX-2026-09-30-p0-schema-foundation-v1
$ git -C …/flux rev-parse HEAD
7226333f6ebf714596703aca84375eef05d6b173     <- MATCHES target
$ git -C …/flux status --short
(empty — clean)
```

Clean before and after. Main checkout stayed on `track/infra-foundation` throughout.

```
$ git diff --stat b6de097..7226333
 ...-2026-09-30-p0-schema-foundation-delivery-v1.md | 736 +++++++++++++++++++
 infra/postgres/schema/010_p0_schema_foundation.sql | 462 ++++++++++++
 scripts/apply-db.sh                                |   5 +
 3 files changed, 1203 insertions(+)

$ git diff --name-only b6de097..7226333 | grep -E '\.(rs|tsx)$'   -> (empty)
$ git diff --name-only b6de097..7226333 | grep '^ui/'             -> (empty)
```

**Zero `.rs`, zero `.tsx`, zero `ui/`.** Three files, all in scope. Packet §5's boundary holds.

```
$ ls -1 infra/postgres/schema/
001_… 002_… 003_… 004_… 005_…
008_project_isolation.sql
009_runtime_triple.sql
010_p0_schema_foundation.sql        <- added; 006 and 007 still reserved/absent
```

---

## 2. The trap — I used a genuinely fresh database, and here is the proof

Before doing anything I enumerated the existing databases:

```
$ psql -tAc "SELECT datname FROM pg_database WHERE datname LIKE 'seatloom%'"
seatloom                            <- the LIVE database, and nothing else
```

**No scratch database existed at all**, so there was nothing from an earlier run to reuse. I then
dropped-if-exists and created anyway, and PostgreSQL's own notice confirms the name was previously
unused:

```
$ psql -c 'DROP DATABASE IF EXISTS seatloom_p0_flux;' -c 'CREATE DATABASE seatloom_p0_flux;'
NOTICE:  database "seatloom_p0_flux" does not exist, skipping
DROP DATABASE
CREATE DATABASE
```

**Independent confirmation that the new constraint is the one under test.** The strongest evidence
is not the procedure but the result: the constraint read back out of this database carries **12**
values with no `ArtifactProduced` (§3.3). Had I reused a database from the earlier 13-value run,
the `IF NOT EXISTS (SELECT 1 FROM pg_constraint …)` guard would have left the old constraint in
place and that read would have shown 13. It shows 12. The trap is excluded by observation, not by
my say-so.

I did not write to the live `seatloom` database at any point — confirmed in §6.

---

## 3. Assertions

### 3.1 Ordering and registration — PASS

`010` is registered in the **post-seed** group with a justification string, and the group carries
an explicit ordering comment:

```bash
POST_SEED_SCHEMA=(
  # Order within this group is significant: 008 BEFORE 010.
  # 010's G5 guard RAISES if sessions.project_id is missing or still nullable,
  # and 008:51 / 008:106-107 is what creates and tightens it. Reversing these
  # two makes 010 abort by design rather than silently proceed.
  "008_project_isolation.sql|backfills project_id … then SET NOT NULL …"
  "010_p0_schema_foundation.sql|adds canonical_events.schema_version NOT NULL and
   event_object_refs.role NOT NULL, then backfills both from existing rows. Seed 001 inserts into
   BOTH tables and supplies NEITHER column … Also widens the event_object_refs primary key to
   include role and adds a partial unique index scoped so legacy rows cannot violate it …"
)
```

The from-empty build **honoured it in fact**, not just in configuration:

```
--- pre-seed schema (seed writes columns these create) ---
  001 … 002 … 003 … 004 … 005 … 009_runtime_triple.sql
--- seed (001-004) ---
  001 … 002 … 003 … 004 …
--- post-seed schema (constrains columns the seed does not provide) ---
  008_project_isolation.sql          <- first
  010_p0_schema_foundation.sql       <- second
apply-db: done.

apply-db.sh RUN 1 exit: 0
```

Worth noting as good design: `apply-db.sh` now carries a **completeness guard** — a migration
present on disk but absent from both groups fails loudly instead of being silently skipped.

### 3.2 Envelope, roles, topics, turn key — PASS, all four counts ZERO

```
 events_missing_schema_version -> 0
 refs_missing_role             -> 0
 legacy_subject_rows           -> 0
 turn_seq_backfilled           -> 0
```

`turn_seq` is `NULL` on every row, never `0` — the distinction the packet cares about, since `0`
would be a fabricated ordinal. Domains:

```
  role   |  ref_type              event_type            | schema_version      status
---------+------------         -----------------------+----------------   --------
 context | artifact             ArtifactCreated        |  1                (0 rows)
 context | delegation           HandoffAccepted        |  1
 context | handoff              HandoffCompleted       |  1
 context | session              HandoffSent            |  1
 context | workitem             ReviewVerdictIssued    |  1
(5 rows)                        SeatDelegationClosed   |  1
                                SeatDelegationIssued   |  1
 every legacy ref is 'context'  SessionStarted         |  1
 none is 'subject'              WorkItemCreated        |  1
                                WorkItemStatusChanged  |  1
                               (10 rows, all schema_version 1)
```

`topics` exists and is empty (zero rows, so no `status` values yet — correct for a new table; no
fabricated rows).

**Event→topic uses the existing ref fabric, with no new join mechanism.** Verified structurally
rather than taken on trust:

```
$ tables matching '%topic%' or 'event_%'  ->  event_object_refs, topics
```

There is no `event_topics` or equivalent join table. `topic` is simply one of the nine `ref_type`
values in `event_object_refs_ref_type_vocab` (`workitem, session, handoff, artifact, document,
seat, topic, project, delegation`).

**G5 — verify-only, column NOT re-added.** `010` contains no `ADD COLUMN` for
`sessions.project_id`. Lines 339-373 are a `DO` block that raises if the premise fails:

```
G5 premise violated: sessions.project_id does not exist. 010 expects 008 to have added it (008:51)
and tightened it (008:106-107). Apply 008 first; do NOT add the column here.
G5 premise violated: sessions.project_id exists but is NULLABLE. 008:107 should have set it NOT NULL.
```

Live state: `sessions.project_id text nullable=NO` — present and NOT NULL, from `008`. The only
other `project_id` in the migration is `topics.project_id`, a column of the new table, which is
unrelated.

### 3.3 The vocabulary — 12 values, and it bites

Read straight out of the scratch database:

```
canonical_events_event_type_vocab | CHECK (event_type = ANY (ARRAY[
  'SessionStarted','WorkItemCreated','WorkItemStatusChanged','ArtifactCreated','HandoffSent',
  'HandoffAccepted','HandoffCompleted','ReviewVerdictIssued','SeatDelegationIssued',
  'SeatDelegationClosed','SupervisorMessage','SeatResponse']))
```

Exactly **12** — the ten in committed data plus `SupervisorMessage` and `SeatResponse`. Two further
constraints ship alongside: `schema_version >= 1`, and `turn_seq IS NULL OR turn_seq >= 0`.

**`ArtifactProduced` appears exactly once in the whole of `infra/`**, at `010:133`, and it is a
comment:

```
133:  -- draft of this migration added 'ArtifactProduced'; it was dropped on Lyra's
```

**It accepts all real data** — the seed loaded in the from-empty build, and all ten committed
event types are present with `schema_version = 1`.

**It rejects out-of-vocabulary values.** I ran both directions, in rolled-back transactions, and I
deliberately chose `ArtifactProduced` as the negative probe rather than a random string — so the
rejection simultaneously proves the constraint bites **and** that the 12-value version is the one
in force:

```
=== POSITIVE: a NEW vocab value must be ACCEPTED ===
BEGIN
INSERT 0 1                                      <- 'SupervisorMessage' accepted
ROLLBACK

=== NEGATIVE: an out-of-vocab value must be REJECTED ===
BEGIN
ERROR:  new row for relation "canonical_events" violates check constraint
        "canonical_events_event_type_vocab"
DETAIL:  Failing row contains (flux-probe-bad, ArtifactProduced, …).
ROLLBACK
```

The constraint discriminates — it is not rejecting everything, and it is not accepting everything.

**Partial unique index — present, and it also bites.** `legacy_subject_rows = 0` means no legacy
row can violate it, which I state explicitly as asked. But I also proved the index is live rather
than merely declared:

```
uq_event_object_refs_one_subject_per_event
  :: CREATE UNIQUE INDEX … ON event_object_refs USING btree (event_id) WHERE (role = 'subject')

BEGIN
INSERT 0 1                                      <- first subject for ev-001 accepted
ERROR:  duplicate key value violates unique constraint
        "uq_event_object_refs_one_subject_per_event"
DETAIL:  Key (event_id)=(ev-001) already exists.   <- second subject rejected
ROLLBACK
```

At-most-one-subject is enforced by SQL, as designed. PK confirmed widened:
`PRIMARY KEY (event_id, ref_type, ref_id, role)`, plus
`idx_event_object_refs_target_role (ref_type, ref_id, role)`. Role vocab is
`{subject, actor, context}`. Exactly-one-subject remains the step-2 write API's job, per Aegis.

### 3.4 Idempotency of the second `010` apply — PASS, measured

The packet's real idempotency requirement. I snapshotted **43 schema facts** (every constraint
definition, every index definition, every column of `canonical_events`/`event_object_refs`/`topics`
with nullability, plus row counts for all three tables and the `turn_seq` non-null count), then
applied `010` twice more:

```
SECOND 010 apply -> COMMIT, only "already exists, skipping" notices
THIRD  010 apply -> exit 0

$ diff before after
IDENTICAL — 010 re-apply changed nothing (43 schema facts + row counts)
```

### 3.5 CI on the branch — observed, not inferred

The branch is pushed to GitLab at exactly the target commit
(`7226333f6ebf714596703aca84375eef05d6b173`). Queried via the API:

```
pipeline 905464  sha=7226333  status=success  created=2026-09-30T13:42:08Z

docs:verify      stage=docs    status=success    5.8s
frontend:lint    stage=lint    status=success   62.2s
frontend:test    stage=test    status=success   55.1s
rust:lint        stage=lint    status=success  141.4s
rust:test        stage=test    status=success  149.4s
total jobs: 5
```

**Both anchors are green**: `rust:test` (the one that runs real Postgres, `apply-db.sh` and the DB
integration tests) and `docs:verify`. All five jobs pass. Consistent with Aegis §4, there are no
`schema` or `contract` stages in the pipeline — I did not look for them beyond confirming the job
list is complete at five.

---

## 4. The expected non-defect

A second full `apply-db.sh` run fails, as you said it would:

```
SECOND FULL apply-db.sh exit: 3
ERROR:  null value in column "project_id" of relation "sessions" violates not-null constraint
DETAIL:  Failing row contains (ses-aegis-supervisor-001, …)
failed at: infra/postgres/seed/001_real_collaboration_baseline.sql
```

Identical relation, identical row, identical file to what I proved pre-existing on `track` in the
v3 delivery's control experiment. **`010` changes nothing about it** — the failure is still seed
001 supplying no `project_id` against `008`'s NOT NULL. Noted, not counted against this packet, and
it remains the follow-up packet's business.

---

## 5. Per-assertion table

| § | Assertion | Expected | Actual | Verdict |
|---|---|---|---|---|
| 1 | worktree HEAD == target | `7226333` | `7226333f6ebf714…` | PASS |
| 1 | worktree clean before/after | empty | empty both | PASS |
| 1 | zero `.rs` / `.tsx` | 0 | 0 | PASS |
| 1 | zero `ui/` | 0 | 0 | PASS |
| 1 | 010 added, 006/007 reserved | yes | yes | PASS |
| 2 | genuinely fresh database | fresh | no scratch DB existed; NOTICE confirms | PASS |
| 2 | new (12-value) constraint under test | 12 | 12, read from the DB | PASS |
| 3.1 | 010 post-seed with justification | yes | yes, with ordering comment | PASS |
| 3.1 | 008 ordered before 010 in a real build | yes | 008 then 010 in transcript | PASS |
| 3.1 | from-empty build | exit 0 | exit 0 | PASS |
| 3.2 | `events_missing_schema_version` | 0 | 0 | PASS |
| 3.2 | `refs_missing_role` | 0 | 0 | PASS |
| 3.2 | `legacy_subject_rows` | 0 | 0 | PASS |
| 3.2 | `turn_seq_backfilled` (NULL not 0) | 0 | 0 | PASS |
| 3.2 | every legacy ref is `context` | all | all 5 ref_types context | PASS |
| 3.2 | topics exists, no new join mechanism | ref fabric | `topic` is a ref_type; no join table | PASS |
| 3.2 | G5 verify-only, column not re-added | verify-only | DO/RAISE only; no ADD COLUMN | PASS |
| 3.3 | vocabulary is 12 | 12 | 12 | PASS |
| 3.3 | `ArtifactProduced` only in a comment | comment only | `010:133`, sole occurrence in `infra/` | PASS |
| 3.3 | accepts real data | yes | seed loaded; 10 types present | PASS |
| 3.3 | **rejects out-of-vocab** | reject | rejected by constraint name | PASS |
| 3.3 | partial unique index exists | yes | yes | PASS |
| 3.3 | legacy rows do not violate it | none | 0 subject rows | PASS |
| 3.3 | index actually enforces | enforces | 2nd subject rejected | PASS |
| 3.4 | second 010 apply changes nothing | no change | 43 facts identical after 2 applies | PASS |
| 3.5 | CI `rust:test` | green | success 149.4s | PASS |
| 3.5 | CI `docs:verify` | green | success 5.8s | PASS |
| 4 | 2nd full apply-db fails on project_id | expected | exit 3, as documented | expected non-defect |
| — | scratch DB dropped | yes | dropped | PASS |
| — | live DB untouched | untouched | no `topics` table on live | PASS |
| — | main checkout not switched | on `track` | on `track` | PASS |

---

## 6. Closing state

```
$ psql -tAc "SELECT datname FROM pg_database WHERE datname LIKE 'seatloom%'"
seatloom                                    <- scratch dropped

$ psql -d seatloom -tAc "…"
seats=6 sessions=5 topics_table_exists=0    <- live DB intact AND has no 010:
                                               proof I never applied the migration to it

$ git -C …/flux rev-parse HEAD   -> 7226333f6ebf714596703aca84375eef05d6b173
$ git -C …/flux status --short   -> (empty)
$ git -C …/seatloom rev-parse --abbrev-ref HEAD -> track/infra-foundation
$ git -C …/seatloom status --short -> (empty)
```

`topics_table_exists=0` on the live database is the cleanest available proof that this gate never
wrote to it. Scratch database `seatloom_p0_flux` dropped; the temporary `psql` shim removed. I did
**not** run `scripts/verify-postgres-baseline.sh`, did not edit any `.sql`, did not fix anything,
did not switch the main checkout's branch, and created no stash. Nothing was merged.

Tooling caveat unchanged from prior gates: there is still **no `psql` client on this host**, so all
SQL ran through a temporary shim forwarding to `podman exec -i seatloom-postgres psql`.
`apply-db.sh`'s own host-`psql` invocation path therefore remains unexercised locally — though
note that **CI's `rust:test` does exercise it against a real Postgres service**, which is the first
time that gap has been covered by something in this project.

---

## 7. Recommendation

**Merge.** Step 1 of the P0 build is sound: the boundary held, the guards bite, the migration is
idempotent, and CI is green at the pinned sha.

One observation for step 2, not a finding: the PK `(event_id, ref_type, ref_id, role)` permits the
same object to be attached to one event under two different roles — e.g. both `actor` and
`context`. SQL now guarantees *at most one subject*; nothing prevents role duplication of that
kind. Aegis endorsed the PK as delivered and exactly-one-subject is explicitly the step-2 write
API's job, so this is simply a note that the write API owns a little more than the subject rule
alone.

---

*Flux · 2026-09-30 · PASS · verified commit `7226333f6ebf714596703aca84375eef05d6b173` ·
fresh scratch database proven fresh by observation · vocabulary and partial index both proven to
bite · CI pipeline 905464 observed green at the target sha*
