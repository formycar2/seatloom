# Task: Commit-pinned verification — P0 step 1, schema foundation (migration 010)

[Lyra -> Flux] First P0 packet. Onyx delivered `010`, Aegis signed off the open decisions, and
Onyx made the one edit that sign-off required (event_type vocabulary 13 → 12). This is the gate
before merge. Steps 2–5 of the P0 build are queued behind it.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-p0-schema-foundation-verification-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| version | v1 |
| to | flux |
| verifies | `docs/coordination/tasks/onyx/ONYX-2026-09-30-p0-schema-foundation-v1.md` |
| delivery under test | `docs/coordination/tasks/onyx/ONYX-2026-09-30-p0-schema-foundation-delivery-v1.md` |
| sign-off | `docs/coordination/tasks/lyra/AEGIS-2026-09-30-p0-step1-signoff-v1.md` |
| target branch | `packet/onyx/ONYX-2026-09-30-p0-schema-foundation-v1` |
| **target commit** | **`7226333`** |
| base | `b6de097` (GitLab `track/infra-foundation`) |
| run location | `/Users/…/seatloom-worktrees/flux` — already on `7226333`, clean |
| delivery path | `docs/coordination/tasks/flux/FLUX-2026-09-30-p0-schema-foundation-verification-delivery-v1.md` |
| tags | flux, verification, commit-pinned, schema-010, p0 |

---

## 0. State, verified by me before dispatch

```
$ git -C …/flux rev-parse --abbrev-ref HEAD
packet/onyx/ONYX-2026-09-30-p0-schema-foundation-v1
$ git -C …/flux rev-parse --short HEAD
7226333
$ git -C …/flux status --short
(empty — clean)
```

Main checkout stays on `track/infra-foundation`; do not switch it. Commit your delivery doc there.
`ui/node_modules` and the `ui/dist` stub from the Nimbus gate are still in the worktree and are
gitignored — this is a DB gate, so you likely need neither.

## 1. Identity and scope

Standard §14.2 capture. Then:

```
git diff --stat b6de097..7226333
git diff --name-only b6de097..7226333 | grep -E '\.(rs|tsx)$'    # must be EMPTY
git diff --name-only b6de097..7226333 | grep '^ui/'              # must be EMPTY
ls -1 infra/postgres/schema/                                      # 010 added; 006/007 still reserved
```

Packet §5 forbade `.rs` and `ui/` changes. Prove it.

## 2. The trap in this one — you must use a genuinely fresh database

`010` creates its constraints under
`IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = …)`. That guard is correct for
idempotency, **but it means applying `010` to a database that already carries the earlier
13-value constraint will silently leave the old one in place.** Onyx's first delivery had 13
values including `ArtifactProduced`; `7226333` drops it to 12.

**So: `DROP DATABASE` and recreate, or use a brand-new scratch name.** If you reuse a scratch DB
from the earlier run you will "verify" the 13-value constraint and report a PASS that is false.
This is the single most likely way this gate goes wrong.

```
DROP DATABASE IF EXISTS seatloom_p0_flux;  CREATE DATABASE seatloom_p0_flux;
DATABASE_URL=<…/seatloom_p0_flux> bash scripts/apply-db.sh      # must exit 0
<apply 010 a SECOND time>                                        # must change nothing
```

Do **not** write to the live `seatloom` database.

## 3. Assertions

### 3.1 Ordering and registration

`010` must be registered in `apply-db.sh`'s **post-seed** group with a justification string.
Onyx also documents that **008 must run before 010**, because `010`'s G5 guard raises if
`sessions.project_id` is missing or still nullable. Confirm the group order actually puts `008`
before `010`, and that a from-empty build honours it.

### 3.2 Envelope, roles, topics, turn key

- `canonical_events.schema_version INT NOT NULL`; existing rows backfilled to `1`.
- `canonical_events.turn_seq INT` **nullable**, and **not** backfilled (`NULL`, never `0`).
- `event_object_refs.role TEXT NOT NULL`, domain `{subject, actor, context}`; **every legacy row
  is `context`** and **none is `subject`**.
- `topics` exists; event→topic uses the existing ref fabric, no new join mechanism.
- G5: `sessions.project_id` **not re-added** — confirm the migration only verifies it.

```sql
SELECT count(*) AS events_missing_schema_version FROM canonical_events WHERE schema_version IS NULL;
SELECT count(*) AS refs_missing_role             FROM event_object_refs WHERE role IS NULL;
SELECT count(*) AS legacy_subject_rows           FROM event_object_refs WHERE role = 'subject';
SELECT count(*) AS turn_seq_backfilled           FROM canonical_events WHERE turn_seq IS NOT NULL;
SELECT DISTINCT role, ref_type FROM event_object_refs ORDER BY 1,2;
SELECT DISTINCT event_type, schema_version FROM canonical_events ORDER BY 1,2;
SELECT DISTINCT status FROM topics ORDER BY 1;
```

All four counts must be `0`.

### 3.3 The vocabulary — 12 values, and it must actually bite

Final list is **12**: the ten present in committed data — `SessionStarted`, `WorkItemCreated`,
`WorkItemStatusChanged`, `ArtifactCreated`, `HandoffSent`, `HandoffAccepted`, `HandoffCompleted`,
`ReviewVerdictIssued`, `SeatDelegationIssued`, `SeatDelegationClosed` — plus `SupervisorMessage`
and `SeatResponse`. **`ArtifactProduced` must appear nowhere except a comment** explaining it was
dropped.

Two checks:

1. **It accepts all real data** — the seed loaded, so this is implied, but state it.
2. **It rejects an out-of-vocab value.** Same principle as the parse-boundary negative path on the
   Nimbus gate: a constraint that never rejects anything is not evidence it works. Attempt an
   insert with a bogus `event_type` in the scratch DB and paste the rejection.

Also confirm the **partial unique index** on `(event_id) WHERE role='subject'` exists and that
legacy rows do not violate it — which follows from §3.2's `legacy_subject_rows = 0`, but say so
explicitly. Note the PK is now `(event_id, ref_type, ref_id, role)`, endorsed by Aegis as
delivered; at-most-one-subject is SQL's job here, exactly-one is the step-2 write API's.

### 3.4 CI on the branch

Acceptance is anchored to **`rust:test`** (real Postgres, `apply-db.sh`, DB integration tests) and
**`docs:verify`**. Aegis §4 confirmed the `schema`/`contract` stages named in its kickoff **do not
exist** — do not look for them. Report the pipeline result for the branch, or say plainly that you
could not trigger/observe it, rather than inferring.

## 4. Out of scope

- Do not run `scripts/verify-postgres-baseline.sh` (still no compose provider; destructive path).
- Do not fix the seed `project_id` idempotency defect — known, pre-existing, its own follow-up.
  If a second full `apply-db.sh` run fails on it, that is **expected and not this packet's
  defect**; note it and move on.
- Do not edit `.sql`. A defect is an Onyx rework finding.
- Do not touch the main checkout's branch.

## 5. Verdict and delivery

PASS / HOLD / FAIL, same rule: partial verification is HOLD with the reason, not PASS with a
caveat.

Delivery doc in the **main checkout** on `track`: identity captures; the §1 scope proof; explicit
confirmation you used a **fresh** database (§2) and how; the from-empty transcript with exit code;
both `010` applies; the §3.2 query results with all four zeros; the §3.3 negative test output; the
§3.4 CI result or an honest statement you could not observe it; and the verdict.

---

*Dispatched by Lyra · 2026-09-30 · first P0 gate · §2 is the likely failure mode: the IF NOT EXISTS
guard means a reused scratch DB silently verifies the old 13-value constraint*
