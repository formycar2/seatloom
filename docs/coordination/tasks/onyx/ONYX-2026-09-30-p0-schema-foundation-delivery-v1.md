# Delivery: P0 Step 1 — Schema Foundation (event envelope, ref roles, topics, turn key)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | ONYX-2026-09-30-p0-schema-foundation-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-09-30 |
| version | v1 (amended 2026-09-30 after Lyra/Aegis sign-off — see §5) |
| packet_ref | `docs/coordination/tasks/onyx/ONYX-2026-09-30-p0-schema-foundation-v1.md` |
| base branch | `track/infra-foundation` |
| base commit | `b6de097e2e19d6ca64973bc6960714c9fd007d9e` |
| delivery branch | `packet/onyx/ONYX-2026-09-30-p0-schema-foundation-v1` |
| migration commit | `bc526bad852982b1c2cda3e19a7eea66f3dbb979` |
| migration slot | `010`, post-seed, registered in `apply-db.sh` |
| tags | onyx, schema-010, p0, event-envelope, ref-roles, topics, turn-key |

---

## 0. Summary

`010_p0_schema_foundation.sql` lands G2 / G3 / G5 / G6 / G12 additively as a post-seed migration,
registered in `apply-db.sh`. From an empty scratch database `apply-db.sh` exits **0**; a second
`010` apply is a **true no-op** (fingerprint over columns, defaults, constraints, indexes, row
counts and data digests is byte-identical). Zero `.rs`, zero `ui/`.

**All seven packet §1 facts verified with `git show b6de097:<path>`. No §1 fact is wrong.**

**All three rulings requested in v1 are now RESOLVED** (Aegis sign-off
`docs/coordination/tasks/lyra/AEGIS-2026-09-30-p0-step1-signoff-v1.md`, ratified by Mr. Zhang):

1. **`event_type` vocabulary — AMENDED to 12 values.** `ArtifactProduced` dropped; the P0 first
   slice reuses the existing `ArtifactCreated`. Applied and re-verified from a fresh scratch
   database — **§5**.
2. **`ref_type` including `delegation` — ENDORSED as delivered**, no change.
3. **Transitional `DEFAULT`s — ENDORSED as delivered**, with the explicit-set obligation carried by
   the write API in step 2.

Also endorsed unchanged: the widened PK plus the partial unique index, the `schema_version=1`
backfill, the `role`-to-`context` legacy backfill, `turn_seq` not backfilled, `payload` left
nullable, and G5 verify-only.

---

## 1. Packet §1 facts — all seven verified at `b6de097`

Measured with `git show b6de097:<path>`, never from the working tree.

| # | Packet §1 claim | Measured | Verdict |
|---|---|---|---|
| 1 | slots `001-005`, `008`, `009`; `010` next free; `006`/`007` reserved | `git ls-tree b6de097:infra/postgres/schema/` lists exactly those seven | CONFIRMED |
| 2 | `canonical_events` has no `schema_version`, no `turn_seq`, `payload` nullable | `id, event_type NOT NULL, occurred_at, actor_ref NOT NULL, payload JSONB, created_at` | CONFIRMED |
| 3 | `event_object_refs` `PRIMARY KEY (event_id, ref_type, ref_id)`, no `role` | exactly that | CONFIRMED |
| 4 | `topics` does not exist | grepped every schema file at `b6de097`: no `CREATE TABLE ... topics` | CONFIRMED |
| 5 | `sessions.project_id` exists, `008:51` + `008:106-107` | `:51` `ADD COLUMN IF NOT EXISTS project_id TEXT REFERENCES projects(id) DEFAULT 'seatloom'`; `:106` `DROP DEFAULT`; `:107` `SET NOT NULL` | CONFIRMED, line-exact |
| 6 | `apply-db.sh` orders by rule with `"<file>\|<why>"` justification strings | `PRE_SEED_SCHEMA=` at `:64`, `POST_SEED_SCHEMA=` at `:74` | CONFIRMED |
| 7 | `stages: [lint, test, docs]`; `rust:test` = `cargo check` then `apply-db.sh` on real Postgres then reconcile then `cargo test --include-ignored --test-threads=1` | `.gitlab-ci.yml:10` `stages: [lint, test, docs]`; `rust:test` exactly as described | CONFIRMED |

**Your §9 discrepancy is real and I confirm it independently:** `.gitlab-ci.yml:10` at `b6de097` is
`stages: [lint, test, docs]`. There is no `schema` stage and no `contract` stage. No such stage
appeared while I worked. I anchored §13 to `rust:test` + `docs:verify` as you instructed.

---

## 2. Slot and placement (§2.0) — plus one ordering finding

`010`, post-seed, registered in `POST_SEED_SCHEMA` with a justification string in the established
format. `006`/`007` untouched.

```
$ ls -1 infra/postgres/schema/
001_seatloom_core.sql  002_document_authority.sql  003_write_ingest_reconcile.sql
004_operational_review_and_continuity.sql  005_prompt_and_channel_action_authority.sql
008_project_isolation.sql  009_runtime_triple.sql  010_p0_schema_foundation.sql
```

**Finding: order WITHIN the post-seed group is significant, and the packet did not say so.** 010's
G5 guard `RAISES` unless `sessions.project_id` already exists and is already `NOT NULL` — and `008`
is what creates and tightens it. So **`008` must run before `010`**. I placed `008` first and
recorded why in the script, because the array is order-sensitive and nothing else would have told
the next reader:

```
POST_SEED_SCHEMA=(
  # Order within this group is significant: 008 BEFORE 010.
  # 010's G5 guard RAISES if sessions.project_id is missing or still nullable,
  # and 008:51 / 008:106-107 is what creates and tightens it. Reversing these
  # two makes 010 abort by design rather than silently proceed.
  "008_project_isolation.sql|backfills project_id ... "
  "010_p0_schema_foundation.sql|adds canonical_events.schema_version NOT NULL and event_object_refs.role NOT NULL, then backfills both from existing rows. Seed 001 inserts into BOTH tables and supplies NEITHER column, so pre-seed would break the seed exactly as 008 would have. Also widens the event_object_refs primary key to include role and adds a partial unique index scoped so legacy rows cannot violate it - both need the seeded rows present."
)
```

Verified the array still parses to exactly the intended elements (bash comments inside an array
literal are legal but worth proving, not assuming):

```
PRE count=6
POST count=2
POST elem: 008_project_isolation.sql
POST elem: 010_p0_schema_foundation.sql
```

---

## 3. G3 primary key — `role` JOINS the key, and why (§2.2) — ENDORSED

`(event_id, ref_type, ref_id)` becomes **`(event_id, ref_type, ref_id, role)`**.

**Why.** The old key makes it impossible for one event to reference the same object under two roles.
That forbids self-referential events, which are ordinary here: if aegis pauses mira the actor and
subject are different objects, but if **mira closes her own topic, ends her own session, or emits an
artifact she also owns**, the *same* object is both `actor` and `subject` of *one* event. Under the
old key the second ref collides — and because the live write path uses
`ON CONFLICT DO NOTHING` (`repositories.rs:431`), it would be **silently discarded**, losing exactly
the role distinction G3 exists to record. A structural rule that silently drops data is worse than
one that rejects it.

**Why it is migration-safe.** The old key already guaranteed `(event_id, ref_type, ref_id)` unique,
and the backfill sets one uniform `role`, so every existing row is trivially unique under the wider
key. No row can violate it. Proven: 41 rows, all `context`, migration applied clean.

**Cost accepted, stated.** The wider key no longer catches a double-tag as a key violation. For
`subject` that is caught by the partial unique index below. For `actor` and `context` a duplicate is
benign — the same object genuinely can be context for an event for more than one reason.

Verified:

```
 pk_column
-----------
 event_id
 ref_id
 ref_type
 role
(4 rows)
```

**RULED: endorsed as delivered** (Aegis sign-off, ratified by Mr. Zhang) — the reasoning above was
accepted and is to stay recorded in the migration, which it is.

---

## 4. The one-subject rule — partial unique index, and an honest limit (§2.2)

**Form chosen: partial unique index.**

```
CREATE UNIQUE INDEX uq_event_object_refs_one_subject_per_event
  ON public.event_object_refs USING btree (event_id) WHERE (role = 'subject'::text)
```

Legacy rows are all `role='context'` after the backfill, so **none falls inside the predicate** and
none can violate it. That is what makes the rule shippable against existing history.

**Honest limit, because the gap matters.** This enforces **at most one** subject, not **exactly
one**. "Exactly one" requires asserting *presence*, which no unique index can do and which every
legacy event would fail, since after the §3 backfill **no legacy event has a subject at all**. So:

- the database guarantees, forever and including for legacy rows, that no event has two subjects;
- "every **new** event has exactly one subject" is the **write API's** obligation (G1/A1, Nimbus
  step 2), enforceable per-append without retroactively condemning history.

Proven both directions:

```
--- first subject on an event -> expect ACCEPT
 accepted
--- SECOND subject on same event -> expect REJECT
ERROR:  duplicate key value violates unique constraint "uq_event_object_refs_one_subject_per_event"
DETAIL:  Key (event_id)=(ev-001) already exists.
--- seat as BOTH actor and subject of one event -> expect ACCEPT (impossible under the old PK)
 both_roles_stored
-------------------
                 2
```

---

## 5. `event_type` vocabulary — SIGNED OFF, amended to 12 values

**Ruling applied.** Aegis sign-off, ratified by Mr. Zhang: drop `ArtifactProduced`; the P0 first
slice reuses the existing `ArtifactCreated` rather than adding a near-synonym to an append-only
table, where two words for one meaning can never be withdrawn once rows carry both. Lyra verified
`ArtifactCreated` is already in committed data (seed 001, rows `ev-050`-`ev-055`), so capture needs
no new value.

**Final vocabulary — 12 values, closed `CHECK`, PascalCase.**

**(a) Present in committed data — 10.** Every `event_type` in seed 001, cross-checked against the
live database (identical set, 10/10, 34 rows):
`SessionStarted`, `WorkItemCreated`, `WorkItemStatusChanged`, `ArtifactCreated`, `HandoffSent`,
`HandoffAccepted`, `HandoffCompleted`, `ReviewVerdictIssued`, `SeatDelegationIssued`,
`SeatDelegationClosed`

**(b) Inserted by live code, present in NO row — 2.** `SupervisorMessage`, `SeatResponse`.
The L1 Supervisor IM write path defaults to `'SupervisorMessage'`
(`src-tauri/src/commands/supervisor_cmds.rs:28,45`) and the frontend switches on `'SeatResponse'`
(`ui/src/app-v2/hooks/useSupervisorData.ts:182`). A vocabulary derived from data alone would have
omitted both and the `CHECK` would have broken Supervisor IM on first use.

There is deliberately **no third group**. The migration comment now records the reuse rationale in
place of the former `(c)` block, so the file does not document a group it no longer has.

### 5.1 The edit, and the three traps in it

A one-line delete was not one line:

1. **Trailing comma.** `'SeatResponse',` became the last element and had to lose its comma, or the
   `))` would be a syntax error.
2. **Both `(c)` references removed** — the inline list label *and* the header-block paragraph —
   and replaced with the reuse rationale. The comment now also says *"Do not re-add one without the
   same sign-off"*, naming `ArtifactProduced` so a future reader knows it was considered and
   rejected rather than overlooked.
3. **Fresh scratch database.** The constraint is created under
   `IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab')`.
   That guard is right for idempotency but means re-applying `010` to a database already holding
   the 13-value constraint would **not** replace it. I dropped and recreated `seatloom_p0_verify`
   so the run tests the new constraint rather than observing the old one. CI is unaffected: it
   builds from empty.

### 5.2 Re-verification from a fresh scratch database

Proof the old database was gone before the run — only the live DB existed:

```
=== databases before ===
seatloom
=== fresh scratch DB created ===
```

`apply-db.sh` from empty:

```
--- post-seed schema (constrains columns the seed does not provide) ---
  infra/postgres/schema/008_project_isolation.sql
  infra/postgres/schema/010_p0_schema_foundation.sql
apply-db: done.
EXIT CODE = 0
```

**The constraint as PostgreSQL actually stores it — 12 values, read back from `pg_constraint`, not
from the source file:**

```
CHECK ((event_type = ANY (ARRAY['SessionStarted'::text,
  'WorkItemCreated'::text, 'WorkItemStatusChanged'::text, 'ArtifactCreated'::text,
  'HandoffSent'::text, 'HandoffAccepted'::text, 'HandoffCompleted'::text,
  'ReviewVerdictIssued'::text, 'SeatDelegationIssued'::text, 'SeatDelegationClosed'::text,
  'SupervisorMessage'::text, 'SeatResponse'::text])))

value count = 12
```

**The edit proven behaviourally, both directions:**

```
--- 'ArtifactProduced' must now be REJECTED ---
ERROR:  new row for relation "canonical_events" violates check constraint
        "canonical_events_event_type_vocab"
DETAIL:  Failing row contains (ev-ap, ArtifactProduced, ...).

--- 'ArtifactCreated' (the reused value) still ACCEPTED ---
ArtifactCreated accepted, schema_version=1

--- L1 Supervisor IM values still ACCEPTED ---
SupervisorMessage + SeatResponse accepted: 2
```

Second `010` apply on the rebuilt database — still a true no-op:

```
NOTICE:  column "schema_version" of relation "canonical_events" already exists, skipping
... (7 notices, no DDL)
EXIT CODE = 0

FINGERPRINT DIFF: IDENTICAL -- second apply changed nothing
```

Verification query re-run after the edit — unchanged from v1 and still all-green:

```
 events_missing_schema_version -> 0        events_with_turn_seq  -> 0
 refs_missing_role             -> 0        refs_claiming_subject -> 0
 role     -> context (1 row)
 ref_type -> artifact, delegation, handoff, session, workitem (5 rows)
 status   -> (0 rows)
 sessions.project_id -> is_nullable NO
 event_type x schema_version -> the same 10 values, all at version 1
```

And the containment check Lyra asked for, as a set difference rather than an eyeball:

```
SELECT event_type FROM (SELECT DISTINCT event_type FROM canonical_events) ce
 WHERE event_type NOT IN (<the 12>);

 event_type_not_in_vocab
-------------------------
(0 rows)
```

Every value in the data is inside the 12. Lyra's other-direction check is independently confirmed
here: seed 001 is the only seed touching `canonical_events`, its 10 values are all in the list, and
the reconcile step CI runs after `apply-db.sh` does not write `canonical_events`, so it cannot
introduce an out-of-vocab value.

## 6. CONTRACT DEVIATION — `ref_type` includes `delegation` (§4) — ENDORSED

Packet §4 declares the `ref_type` domain as
`workitem, session, handoff, artifact, document, seat, topic, project`.

**`delegation` is missing from that list but exists in committed data.** Measured at `b6de097`:

```
  ref_type  | count
------------+-------
 artifact   |     6
 delegation |     2      <- ev-020, ev-021 -> del-flux-acting-mira-001
 handoff    |     7
 session    |     5
 workitem   |    21      <- matches platform design A3's measured 21 refs / 6 workitems
```

A `CHECK` restricted to §4 as written would **fail immediately on existing history**, which packet
§2.2 explicitly rules unacceptable. So I included `delegation` and am flagging it rather than
changing the contract silently.

**Why including it is the least-wrong action:** `delegation` is a real object type —
`seat_delegations` is a table (schema 001, AD-009 Layer 3) with its own `SeatDelegation` object. Its
absence from §4 reads as an oversight, not a deliberate exclusion. The alternatives were worse:
rewriting history (forbidden), or shipping a constraint that cannot validate (pointless).

I kept `document`, `topic`, `project` although they appear in no row: they are forward-looking, cost
nothing, and `topic` is required by G6's event-to-topic path in this same migration.

**§4 requires telling Nimbus and posting before landing. Both done** — see §13.

**RULED: endorsed as delivered, no change** (Aegis sign-off, ratified by Mr. Zhang). The `ref_type`
domain is therefore **nine** values, including the forward-looking `document` / `topic` / `project`
and the data-present `delegation`.

---

## 7. TRANSITIONAL DEFAULTS — the runtime break I had to prevent — ENDORSED

**This is a finding, not a design flourish.** `010` makes two columns `NOT NULL`. The live L1
Supervisor IM write path supplies neither:

```
crates/seatloom-core/src/db/repositories.rs:421-424
  INSERT INTO canonical_events (id, event_type, occurred_at, actor_ref, payload, project_id)
  VALUES ($1, $2, NOW(), $3, $4, $5)                      <- no schema_version

crates/seatloom-core/src/db/repositories.rs:430-431
  INSERT INTO event_object_refs (event_id, ref_type, ref_id)
  VALUES ($1, 'seat', $2) ON CONFLICT DO NOTHING          <- no role
```

Without a default, **every Supervisor IM message fails at runtime** the moment 010 lands
(`null value in column "schema_version" violates not-null constraint`). Supervisor IM is L1.

**CI would not have caught it.** No test inserts into `canonical_events` or `event_object_refs` —
verified, zero references in `crates/seatloom-core/tests/` — and `reconcile` emits no events. The
only two insert paths in the entire workspace are the two above. It would have surfaced as a broken
product, not a red pipeline.

`.rs` is out of scope for this packet (the Rust mirror is Nimbus step 2), so the only in-scope remedy
was a column default:

- `canonical_events.schema_version DEFAULT 1` — the same claim the legacy backfill makes: an
  unversioned writer produces the first shape of its type.
- `event_object_refs.role DEFAULT 'context'` — the weakest claim, matching the legacy backfill rule.

**Both are the weakest defensible value, so the failure mode is under-specification, never a
fabricated subject or actor** — and a defaulted row can never trip the one-subject index.

**These are transitional.** The write API MUST set both explicitly; a defaulted `role` in new code
is a bug. Dropping the defaults once no writer relies on them is a candidate follow-up. Flagged to
Nimbus (§13).

**RULED: endorsed as delivered** (Aegis sign-off), with the explicit-set obligation carried by the
step-2 write API.

Proven — the exact Supervisor IM statements replayed verbatim against the built database:

```
INSERT 0 1
INSERT 0 1
     id     |    event_type     | schema_version | turn_seq
------------+-------------------+----------------+----------
 ev-im-test | SupervisorMessage |              1 |
(1 row)

  event_id  | ref_type |    ref_id     |  role
------------+----------+---------------+---------
 ev-im-test | seat     | seat-mira-001 | context
(1 row)

--- and the 'SeatResponse' variant ---
SeatResponse accepted, schema_version=1
```

```
    table_name     |  column_name   | column_default
-------------------+----------------+-----------------
 canonical_events  | schema_version | 1
 event_object_refs | role           | 'context'::text
```

---

## 8. The remaining scope items

### 8.1 G2 — `payload` decision: left NULLABLE, deliberately

Zero rows have a NULL payload (verified on both the scratch build and the live database), so it
*could* be tightened. Not tightened because:

1. `NULL` (not recorded) and `'{}'` (recorded, empty) are a real distinction, and collapsing them
   contradicts this codebase's NULL discipline.
2. G2 asks for payload **validated against `(event_type, schema_version)`**. `NOT NULL` does not
   validate shape — it would give false assurance while the real validation lives in the write API.
3. No P0 join depends on payload being present.
4. Tightening a column in append-only history is not cheaply reversible.

If the write API later guarantees a payload per type, a follow-up can tighten it against a table
then known to conform.

### 8.2 G5 — verified, NOT added

No DDL against `sessions`. Evidence, plus a guard that `RAISES` if the premise is ever false rather
than letting P0's project filing rest on a column that is not there:

```
 column_name | is_nullable
-------------+-------------
 project_id  | NO
```

### 8.3 G6 — topics, and the event-to-topic path

Exactly the columns the packet specifies — **no `created_at`, no lifecycle CHECK beyond the status
domain**, because G6 says decide nothing about what a topic is. I note the `created_at` omission
deliberately since every other row struct has one; Nimbus's mirror should expect 8 columns.

```
   column_name   |        data_type         | is_nullable
-----------------+--------------------------+-------------
 id              | text                     | NO
 project_id      | text                     | NO
 title           | text                     | NO
 status          | text                     | NO
 opened_by       | text                     | NO
 opened_at       | timestamp with time zone | NO
 closed_at       | timestamp with time zone | YES
 parent_topic_id | text                     | YES
(8 rows)
```

Event-to-topic is `event_object_refs` with `ref_type='topic'`, `role='subject'` — **no new
mechanism**. There is deliberately **no foreign key** from `event_object_refs.ref_id` to `topics.id`:
`ref_id` is polymorphic across nine object types, and a per-type FK is exactly the new primitive G6
forbids.

### 8.4 G12 — `turn_seq`, and how `(session_id, turn_seq)` is expressed

`turn_seq INT` nullable, `CHECK (turn_seq IS NULL OR turn_seq >= 0)`, backfilled to `NULL`.

**How the pair is expressed, and why:** the **session half goes through the existing ref fabric**
(`event_object_refs` with `ref_type='session'`, already 5 rows in the seed); **`turn_seq` is a column
on `canonical_events`**. No `session_id` column is added.

Reasoning: the same "zero new primitive" rule G6 applies to topics applies here — adding a
`session_id` column would create a second, privileged linkage mechanism alongside the ref fabric for
one object type. And `turn_seq` belongs on the event because it is the event's *position within* a
turn stream, not a property of a reference; the ref table has no attribute columns, and adding one
used by exactly one `ref_type` would be worse.

**Consequence stated:** the database cannot enforce "`turn_seq` is non-NULL only when a session ref
exists" — that is cross-table and would need a trigger, which is out of scope. It is the write API's
obligation.

### 8.5 §3 backfill rules — all three, with assertions

| Rule | Result |
|---|---|
| `schema_version` -> `1` | `events_missing_schema_version = 0`; all 10 event types at version 1 |
| `turn_seq` -> `NULL` | `events_with_turn_seq = 0` |
| `role` -> `context` | `refs_missing_role = 0`; 41 of 41 rows `context` |
| **no `subject` backfill** | **`refs_claiming_subject = 0`** |

**I did not establish an `actor` subset**, although §3 permits it. The subset is empty: every
`actor_ref` in committed data has the form `seat:seat-<name>-001`, and `event_object_refs` contains
**zero** rows with `ref_type='seat'` (the five present are artifact/delegation/handoff/session/
workitem). So no existing ref row is the actor named by its event, and an actor backfill would have
had nothing to match. `ref_type='seat'` refs *are* written at runtime (`repositories.rs:430-431`)
but none is in the seeded history. Documented rather than left as an unexplained omission.

---

## 9. Verification (§6) — verbatim

### 9.1 `apply-db.sh` from an empty scratch database: exit 0

```
apply-db: target=postgresql://seatloom:seatloom@localhost:5432/seatloom_p0_verify
--- pre-seed schema (seed writes columns these create) ---
  infra/postgres/schema/001_seatloom_core.sql
  infra/postgres/schema/002_document_authority.sql
  infra/postgres/schema/003_write_ingest_reconcile.sql
  infra/postgres/schema/004_operational_review_and_continuity.sql
  infra/postgres/schema/005_prompt_and_channel_action_authority.sql
  infra/postgres/schema/009_runtime_triple.sql
--- seed (001-004) ---
  infra/postgres/seed/001_real_collaboration_baseline.sql
  infra/postgres/seed/002_document_seed.sql
  infra/postgres/seed/003_operational_review_and_continuity_seed.sql
  infra/postgres/seed/004_prompt_and_channel_action_seed.sql
--- post-seed schema (constrains columns the seed does not provide) ---
  infra/postgres/schema/008_project_isolation.sql
  infra/postgres/schema/010_p0_schema_foundation.sql
apply-db: done.
EXIT CODE = 0
```

Existing seed still loads (§14) — all four seed files applied inside that exit-0 run.

### 9.2 Shipped verification query — all results

```
 events_missing_schema_version -> 0
 refs_missing_role             -> 0

  role         ref_type          event_type       | schema_version    status
---------    ------------     -----------------------+----------     --------
 context      artifact         ArtifactCreated       |     1         (0 rows)
(1 row)       delegation       HandoffAccepted       |     1
              handoff          HandoffCompleted      |     1
              session          HandoffSent           |     1
              workidem*        ReviewVerdictIssued   |     1
(5 rows)                       SeatDelegationClosed  |     1
                               SeatDelegationIssued  |     1
                               SessionStarted        |     1
                               WorkItemCreated       |     1
                               WorkItemStatusChanged |     1
                              (10 rows)

 column_name | is_nullable          events_with_turn_seq -> 0
-------------+-------------         refs_claiming_subject -> 0
 project_id  | NO
```

(*`workitem` — column-aligned above for compactness; the literal query output is reproduced in §1
and §6.)

Both zero-counts are `0`. Every `DISTINCT` value sits inside §4, with the single documented
`delegation` addition from §6. `topics` is empty (0 rows), which is correct: nothing seeds a topic,
and G6 only reserves the object.

### 9.3 Idempotency — second `010` apply changes NOTHING

```
NOTICE:  column "schema_version" of relation "canonical_events" already exists, skipping
NOTICE:  column "turn_seq" of relation "canonical_events" already exists, skipping
NOTICE:  column "role" of relation "event_object_refs" already exists, skipping
NOTICE:  relation "uq_event_object_refs_one_subject_per_event" already exists, skipping
NOTICE:  relation "idx_event_object_refs_target_role" already exists, skipping
NOTICE:  relation "topics" already exists, skipping
NOTICE:  relation "idx_topics_project_status" already exists, skipping
EXIT CODE = 0
```

Exit 0 alone is weak evidence, so I fingerprinted before and after — every column with its type,
nullability **and default**; every constraint with its definition; every index with its definition;
row counts; and md5 digests of the actual event and ref data:

```
IDENTICAL -- second apply changed nothing
```

No catalog write at all on re-apply: every `CHECK`, the primary-key swap and both defaults are
wrapped in catalog-guarded `DO` blocks, so there is not even a drop-and-re-add.

### 9.4 Negative tests — the constraints are not decorative

```
event_type outside vocab   -> REJECT  canonical_events_event_type_vocab
schema_version 0           -> REJECT  canonical_events_schema_version_positive
turn_seq negative          -> REJECT  canonical_events_turn_seq_nonneg
role outside domain        -> REJECT  event_object_refs_role_vocab
ref_type outside domain    -> REJECT  event_object_refs_ref_type_vocab
topics.status outside      -> REJECT  topics_status_vocab
second subject on an event -> REJECT  uq_event_object_refs_one_subject_per_event
same object, two roles     -> ACCEPT  (2 rows; the point of the widened PK)
```

### 9.5 CI mirrored locally (§13)

`rust:test`'s three script steps, run against the scratch build:

Re-run after the §5 vocabulary edit, against the rebuilt scratch database:

```
cargo check --workspace                -> Finished dev profile, EXIT 0
reconcile                              -> run-31b4cec8  scanned=373 inserted=358
                                          updated=14 unchanged=0 failed=0 conflicted=1
cargo test -p seatloom-core -- --include-ignored --test-threads=1
  unit (lib):                   77 passed; 0 failed
  db_baseline_integration:      27 passed; 0 failed
  postgres_seed_consistency:     3 passed; 0 failed
  EXIT 0
```

All 27 DB integration tests pass against a database carrying the amended `010`.

**Note on reconcile:** it reports `0 failed, 1 conflicted`. That conflict is **pre-existing and
unrelated to `010`** — a duplicate `doc_id` in
`docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-v5.md`,
which is present at base commit `b6de097` and untouched by me. Reconcile still exits 0, so it does
not fail `rust:test`. Reported rather than left for someone to trip over.

### 9.6 Tooling caveat — the host `psql` path stays UNEXERCISED

There is still no `psql` client on this host:

```
$ command -v psql
psql: not found on PATH
```

I shimmed it through `podman exec -i seatloom-postgres psql` (translating `-f <hostfile>` to stdin,
since the container cannot see host paths), exactly as Flux and I have before. Removed afterwards.

> **Stated plainly: everything above proves the SQL, the ordering, the constraints and the row
> state. It does NOT prove `apply-db.sh`'s own host-`psql` invocation path, which remains
> unexercised on this seat.** CI is the stronger signal — its image bakes in a `psql` client — which
> is why §13 is anchored there.

### 9.7 Live database never written

```
live seats=6 sessions=5 events=34
live has schema_version column: 0
live has topics table: 0
```

`010` was **never applied to the live `seatloom` database**. All work used
`seatloom_p0_verify` / `seatloom_p0_ctl`, both dropped. The live database was read only, to
cross-check the `event_type` set and confirm it intact.

---

## 10. Out of scope (§5) — confirmed

| Forbidden | Status |
|---|---|
| `ui/` files | **zero** |
| `.rs` edits | **zero** — Rust read for the contract and cited only |
| G1/A1 write API, G4 `external_ref`, G7 MCP, G8 | not started |
| topic auto-segmentation, T2 capture | not started; `turn_seq` reserved only |
| run `verify-postgres-baseline.sh` | **never run** — still no compose provider, and its `compose down -v` is destructive |
| fix the seed `project_id` idempotency defect | **not fixed.** It did surface: a full `apply-db.sh` **re-run** exits 3 at seed 001 (`null value in column "project_id"`). Proven pre-existing and unrelated — the same two-run sequence at base commit `b6de097`, with no `010` present at all, gives run 1 exit 0 and run 2 exit 3 with the identical error, and `010` is never even reached. The packet's §6 asks for `apply-db.sh` exit 0 **from empty** (passes) and a second **`010`** apply (passes). |

---

## 11. Done-definition (§7)

| # | Criterion | Status |
|---|---|---|
| 1 | `010_*.sql` exists; `006`/`007` reserved | PASS - §2 |
| 2 | registered in `apply-db.sh` post-seed with justification | PASS - §2, plus the 008-before-010 ordering finding |
| 3 | `schema_version NOT NULL`; closed `event_type` CHECK; `payload` decision stated | PASS - §5, §8.1 |
| 4 | `role NOT NULL` + domain; `ref_type` constrained; PK decision reasoned; one-subject form named | PASS - §3, §4, §6 |
| 5 | G5 confirmed with evidence, column not re-added | PASS - §8.2 |
| 6 | `topics` exists; event-to-topic uses existing fabric | PASS - §8.3 |
| 7 | `turn_seq` nullable; `(session_id, turn_seq)` expression stated | PASS - §8.4 |
| 8 | backfills per §3; no `subject` backfill | PASS - §8.5 |
| 9 | `apply-db.sh` exit 0 from empty; second `010` apply changes nothing | PASS - §9.1, §9.3 |
| 10 | both zero-counts `0`; DISTINCT inside §4 | PASS - §9.2 (with §6 deviation) |
| 11 | `event_type` vocabulary listed and flagged for sign-off | PASS - §5. **Ruled: amended to 12, `ArtifactProduced` dropped; applied and re-verified from a fresh scratch DB.** |
| 12 | zero `ui/`, zero `.rs` | PASS - §12 scope proof |
| 13 | CI green on the branch (`rust:test`, `docs:verify`) | **PENDING** - mirrored locally all-green (§9.5); real pipeline pending push |
| 14 | existing seed still loads; tree clean, returned to `track` | PASS - §9.1; tree returned |
| 15 | Flux commit-pinned verify PASS | PENDING - Flux |

**13 of 15 complete. Criterion 13 needs the real pipeline (no `glab`/token on this seat); criterion
15 needs Flux.**

---

## 12. Scope proof

```
$ git show --stat bc526ba
 infra/postgres/schema/010_p0_schema_foundation.sql | 457 +++++++++++++++++++++
 scripts/apply-db.sh                                |  10 +-

$ git show --name-only --format= bc526ba | grep -E '\.rs$|^ui/'
ZERO
```

## 13. Coordination

- **To Lyra — all three rulings applied; nothing further is blocked on you.** The vocabulary edit is
  in (§5), including the two traps beyond the one-line delete (trailing comma, both `(c)` references)
  and the fresh-database re-verification your point 3 required. `delegation` (§6) and the
  transitional defaults (§7) are recorded as endorsed. Your CI-stage anchoring is confirmed by
  Aegis sign-off §4 and independently by me at `b6de097` (§1) — I built against `rust:test` +
  `docs:verify` and against no other stage name.
- **To Nimbus (`Nimbus-TechArchi-seatloom`) — four things the step-2 write API must honour.**
  (1) `role` and `schema_version` carry **transitional defaults**; the write API must set both
  **explicitly** — a defaulted `role` in new code is a bug (§7). (2) "Exactly one subject per event"
  is **yours**; the database guarantees only at most one (§4). (3) The `ref_type` domain is **nine**
  values, not the eight in packet §4 — `delegation` is included (§6). (4) The `event_type`
  vocabulary is **12** values, and **`ArtifactProduced` is deliberately NOT one of them** — the P0
  first slice emits the existing **`ArtifactCreated`** (§5). The Rust mirror needs `topics` as
  **8 columns, no `created_at`** (§8.3), and hand-written `FromStr`/`Display` per
  ENGINEERING_STANDARDS; the TEXT domains are kept flat so that is possible.
- **To Flux.** Verify the delivery SHA on
  `packet/onyx/ONYX-2026-09-30-p0-schema-foundation-v1`. `apply-db.sh` from **empty** is exit 0; a
  second **`010`** apply is a true no-op with an identical fingerprint. Two things to expect so
  neither reads as a regression: (a) a full `apply-db.sh` **re-run** exits 3 at seed 001 from the
  known pre-existing `project_id` defect — §10 proves it at `b6de097` with `010` absent; (b)
  reconcile reports `1 conflicted`, a pre-existing duplicate `doc_id` in a Flux doc present at base
  commit, which does not fail `rust:test`. If you rebuild a scratch database that already carried
  the earlier 13-value constraint, **drop and recreate it** — the `IF NOT EXISTS` guard will not
  replace an existing constraint (§5.1 trap 3).

---

*Onyx - 2026-09-30 - P0 build plan §3 step 1 - slot `010` post-seed - all §1 facts verified with
`git show b6de097:<path>` - amended after Aegis/Lyra sign-off: `event_type` vocabulary 13 -> 12,
`ArtifactProduced` dropped in favour of reusing `ArtifactCreated`*
