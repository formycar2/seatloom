# Task: P0 Step 1 — Schema Foundation (event envelope, ref roles, topics, turn key)

[Lyra -> Onyx] P0 is GO. Aegis dispatched the kickoff after the runtime triple landed and the CI
hard gate went green: `docs/coordination/tasks/lyra/AEGIS-2026-09-30-p0-kickoff-schema-foundation-v1.md`.
This is build plan §3 **step 1**, the critical path — steps 2–5 serialize behind it.

P0 delivers **one true thing that does not exist today**: a record in SeatLoom because an agent, on
some harness, did something — filed under its project and topic, linked to what it touched,
retrievable, and independent of which harness produced it. This packet builds the joins that make
that record expressible.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | ONYX-2026-09-30-p0-schema-foundation-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | onyx |
| priority | P0 (critical path — blocks steps 2–5) |
| milestone | P0 capture loop — one real captured event |
| depends_on | `docs/coordination/reviews/2026-09-30-aegis-p0-build-plan-and-gap-register-v1.md` §2–§4; platform design A2/A3; object model §3; `docs/coordination/ENGINEERING_STANDARDS.md` |
| base branch | `track/infra-foundation` |
| **base commit** | **`b6de097`** (GitLab `gitlab/track/infra-foundation`, the authoritative line) |
| delivery branch | `packet/onyx/ONYX-2026-09-30-p0-schema-foundation-v1` |
| migration slot | **`010`**, **post-seed** (§2.0) |
| delivery path | `docs/coordination/tasks/onyx/ONYX-2026-09-30-p0-schema-foundation-delivery-v1.md` |
| tags | onyx, schema-010, p0, event-envelope, ref-roles, topics, turn-key |

---

## 0. Operating constraints

- **Serial execution.** One shared working tree; you own it. Work in the main checkout. The
  worktree at `seatloom-worktrees/flux` belongs to Flux. Do not `git stash`. Return the tree to
  `track/infra-foundation` when done.
- **This packet file lives on `track`.** Once you check out your packet branch it vanishes from the
  tree — re-read with
  `git show track/infra-foundation:docs/coordination/tasks/onyx/ONYX-2026-09-30-p0-schema-foundation-v1.md`.
- **Measure base-commit facts with `git show b6de097:<path>`, never by reading the working tree.**
  Both runtime-triple packets carry an ERRATUM because I violated this and stated dirty-tree reads
  as committed facts. Every fact in §1 below was measured that way. If one is wrong, that is a
  finding — report it, do not work around it.

## 1. Facts measured at `b6de097`

| Fact | Measured value |
|---|---|
| Schema slots present | `001`–`005`, `008`, `009`. **`010` is the next free slot**; `006`/`007` remain reserved. |
| `canonical_events` | `id, event_type TEXT NOT NULL, occurred_at, actor_ref TEXT NOT NULL, payload JSONB, created_at` — **no `schema_version`, no `turn_seq`, `payload` nullable** |
| `event_object_refs` | `event_id, ref_type TEXT NOT NULL, ref_id TEXT NOT NULL`, **`PRIMARY KEY (event_id, ref_type, ref_id)`** — no `role` |
| `topics` | **does not exist** |
| `sessions.project_id` | **already exists** — `008:51` adds it, `008:106-107` `DROP DEFAULT` + `SET NOT NULL` |
| `apply-db.sh` | orders **by rule**, with a per-migration justification string (`"<file>\|<why>"`) in a pre-seed or post-seed group |
| CI | `stages: [lint, test, docs]`. `rust:test` runs `cargo check --workspace`, then `bash scripts/apply-db.sh` against a real Postgres service, then reconcile, then `cargo test -p seatloom-core -- --include-ignored --test-threads=1` |

## 2. Scope

### 2.0 Slot and placement — `010`, post-seed, and you must register it

By the rule `apply-db.sh` now encodes: **post-seed if the migration constrains columns the seed
does not provide.** This migration adds `schema_version NOT NULL` to `canonical_events` and `role
NOT NULL` to `event_object_refs`; seed 001 inserts rows into both and supplies neither. Pre-seed
would break the seed exactly as `008` would have. So: **post-seed**.

**Register `010` in `apply-db.sh`'s post-seed group with a justification string** in the existing
format. A migration that exists but is unregistered is invisible to CI, which builds from empty
every pipeline.

### 2.1 G2 — event envelope

- `canonical_events.schema_version INT NOT NULL` — per `event_type`, starts at 1.
- `event_type` constrained to a **closed vocabulary**, PascalCase, via `CHECK` — extensible by
  migration, not free text.
- `payload` validated against `(event_type, schema_version)`. Note `payload` is **nullable today**;
  decide and state whether `010` tightens it to `NOT NULL` (and whether existing rows permit that).

**Propose the initial `event_type` vocabulary from the values actually present**, plus what the P0
first slice needs (`ArtifactProduced` at minimum, per build plan §3 step 5). **List your proposal
in the delivery doc and flag it for my sign-off** — a closed vocabulary is cheap to extend by
migration and expensive to get wrong in an append-only table.

### 2.2 G3 — ref roles

- `event_object_refs.role TEXT NOT NULL` ∈ {`subject`, `actor`, `context`}.
- `ref_type` constrained to the object vocabulary.

**Design implication you must resolve and state:** the current primary key is
`(event_id, ref_type, ref_id)`. That makes it impossible for one event to reference the same object
under two roles, and it does not include `role`. Decide whether `role` joins the key, and say why.
Do not change the PK silently — it is a structural decision and I want the reasoning in the record.

**"Exactly one `subject` per event" is a steady-state rule, not a retroactive one.** Enforce it with
a partial unique index (`UNIQUE (event_id) WHERE role = 'subject'`) or equivalent, **scoped so
legacy rows do not violate it** — see §3. A blanket constraint that fails on existing history is
not acceptable; state which form you chose.

### 2.3 G5 — verify, do **not** add

`sessions.project_id` already exists and is `NOT NULL` (008:51, 008:106-107 — I measured it).
**Confirm it with evidence and wire it into the contract. Do not re-add the column.**

### 2.4 G6 — topics

```
topics(id, project_id, title, status, opened_by, opened_at, closed_at, parent_topic_id?)
```

Event→topic uses the **existing** role-tagged ref fabric — `event_object_refs` with
`role='subject'`, `ref_type='topic'` — and **no new join mechanism** (object model §3.3: "zero new
primitive"). Reserve the link now; **auto-segmentation is P1** and out of scope. Create the table
and the ref path, nothing that decides what a topic *is*.

### 2.5 G12 — reserve the turn key

`canonical_events.turn_seq INT` **nullable**, reserving the T2 join key `(session_id, turn_seq)`.
T2 capture is P1. `NULL` means *not recorded*, never *zero*.

Note `canonical_events` has no `session_id` column today — state how the pair is expressed
(via `event_object_refs` with `ref_type='session'`, or a column) and why.

## 3. Backfill rules — the part most likely to go wrong

`canonical_events` and `event_object_refs` are **append-only history**.

- `schema_version` on existing rows → **`1`**, with a SQL comment saying why (they are the first
  shape of their type).
- `turn_seq` on existing rows → **`NULL`**. Not recorded.
- `role` on existing rows → **backfill to `context`.** Platform design A3 measured 21 `workitem`
  refs for 6 workitems: existing refs carry no role distinction and cannot be mechanically
  separated. `context` is the weakest, least-wrong claim. If you can establish `actor` for a subset
  from `canonical_events.actor_ref`, do that subset and document the rule.

  **Do not backfill `subject`.** Asserting a subject the data does not support is the same class of
  error as writing `Stepcode` onto an April session row. If you believe a defensible subject rule
  exists, propose it and let me rule — do not apply it silently.

This is why the one-subject constraint must be scoped: after this backfill **no legacy event has a
subject at all**, so a blanket rule would fail immediately.

## 4. Value-domain contract (authoritative — Lyra-owned, shared with Nimbus)

Reproduced **verbatim** in the downstream Nimbus write-API packet. This is the mechanism that kept
the runtime-triple halves identical, and Aegis's kickoff names it as the non-negotiable discipline.

| Field | Domain | PG column | Nullable |
|---|---|---|---|
| `event_object_refs.role` | `subject`, `actor`, `context` | `TEXT` | no |
| `event_object_refs.ref_type` | `workitem`, `session`, `handoff`, `artifact`, `document`, `seat`, `topic`, `project` | `TEXT` | no |
| `canonical_events.event_type` | closed vocabulary, **PascalCase** (§2.1, pending my sign-off) | `TEXT` | no |
| `canonical_events.schema_version` | integer ≥ 1, per `event_type` | `INT` | no |
| `canonical_events.turn_seq` | integer ≥ 0 | `INT` | **yes** — `NULL` = not recorded |
| `topics.status` | `open`, `closed`, `merged` | `TEXT` | no |

Binding rules:

1. **PascalCase for `event_type`; snake_case for `role` / `ref_type` / `status`.** Same principle as
   `009`: *match what is already in PostgreSQL.* Existing `event_type` values are PascalCase
   (`SessionStarted`); existing `ref_type` values are lowercase (`seat`). The inconsistency is in
   the data, not the rule — do not harmonize, that means rewriting history.
2. Enums crossing the Rust⇄PostgreSQL TEXT boundary get **hand-written `FromStr`/`Display`**, never
   serde derive (ENGINEERING_STANDARDS; see `objects/session.rs::parse_custom_tagged`). That is
   Nimbus's obligation in step 2 — **yours is to keep the TEXT domain flat so a hand-written parser
   is possible.**
3. `NULL` means *not recorded*, never a default.

Deviation requires telling Nimbus in `Nimbus-TechArchi-seatloom` **and** posting here before
landing.

## 5. Out of scope

- **No UI.** Zero files under `ui/`. Build plan §2 excludes new L2 surfaces from P0.
- **No `.rs` edits.** The Rust mirror is Nimbus step 2. You read Rust for the contract and cite it.
- **Not G1/A1 (write API), not G4 (`external_ref`), not G7 (MCP emit server), not G8.** Steps 2–4.
- **No topic auto-segmentation** (P1). **No T2 capture** (P1).
- **Do not run `scripts/verify-postgres-baseline.sh`** — still unrunnable here (podman has no
  compose provider) and its `compose down -v` is destructive.
- **Do not fix the seed `project_id` idempotency defect** — real, known, and its own follow-up
  packet. If it blocks you, say so.

## 6. Verification

Work against a **scratch database**, never the live `seatloom`:

```
CREATE DATABASE seatloom_p0_verify;          -- drop first if present
DATABASE_URL=<…/seatloom_p0_verify> bash scripts/apply-db.sh    # must exit 0
<apply 010 a SECOND time>                                        # idempotency: zero second-run change
```

Ship a verification query with the migration (ONYX.md). At minimum:

```sql
SELECT count(*) AS events_missing_schema_version FROM canonical_events WHERE schema_version IS NULL;
SELECT count(*) AS refs_missing_role             FROM event_object_refs WHERE role IS NULL;
SELECT DISTINCT role     FROM event_object_refs ORDER BY 1;
SELECT DISTINCT ref_type FROM event_object_refs ORDER BY 1;
SELECT DISTINCT event_type, schema_version FROM canonical_events ORDER BY 1,2;
SELECT DISTINCT status   FROM topics ORDER BY 1;
SELECT column_name, is_nullable FROM information_schema.columns
 WHERE table_name='sessions' AND column_name='project_id';     -- G5 evidence
```

First two must return `0`. All `DISTINCT` output must sit inside §4.

**Tooling caveat:** there is no `psql` client on this host. Both Flux and you previously shimmed
through `podman exec -i seatloom-postgres psql`. Do the same, disclose it, clean it up, and state
that `apply-db.sh`'s own host-`psql` path remains **unexercised** locally. It *is* exercised in CI,
where the image bakes in a `psql` client — which is the stronger signal, so get the branch through
`rust:test`.

## 7. Done-definition

1. `infra/postgres/schema/010_*.sql` exists; `006`/`007` still reserved.
2. `010` **registered in `apply-db.sh`'s post-seed group** with a justification string.
3. G2: `schema_version INT NOT NULL`; `event_type` closed-vocabulary `CHECK`; `payload` decision
   stated.
4. G3: `role TEXT NOT NULL` with the §4 domain; `ref_type` constrained; **PK decision stated with
   reasoning**; one-subject rule enforced in a form legacy rows do not violate, and that form named.
5. G5: confirmed already satisfied, **with evidence** — column not re-added.
6. G6: `topics` exists; event→topic uses the existing ref fabric with no new mechanism.
7. G12: `turn_seq INT` nullable; how `(session_id, turn_seq)` is expressed is stated.
8. Backfill per §3: `schema_version=1`, `turn_seq NULL`, `role='context'`, **no legacy row
   backfilled to `subject`**.
9. `apply-db.sh` exits **0** from an empty scratch DB; a second `010` apply changes nothing — both
   transcripts verbatim.
10. Both zero-counts return `0`; all `DISTINCT` output inside §4.
11. Proposed `event_type` vocabulary listed and **flagged for my sign-off**.
12. Zero `ui/` and zero `.rs` files changed — `git show --stat` proves it.
13. **CI green on the branch**: `rust:test` (which runs `apply-db.sh` on real Postgres + the DB
    integration tests) and `docs:verify`. See §9 — Aegis's kickoff names `schema`/`contract` stages
    that **do not exist**; I am resolving that with Aegis and it does not gate you.
14. Existing seed still loads; tree clean and returned to `track/infra-foundation`.
15. Flux commit-pinned verify PASSes.

## 8. Evidence expectations

Base/delivery branch and all commit SHAs named; verbatim transcripts including the second apply;
a verification query shipped with the migration; an evidence comment on every backfill rule;
explicit statement of anything not done and why. Evidence-free completion claims are rejected.

## 9. One discrepancy in the kickoff, raised not inherited

Aegis's acceptance says *"the new CI `schema`/`contract` stages pass on the branch."* Measured at
`b6de097`, `.gitlab-ci.yml` declares `stages: [lint, test, docs]` — **there are no `schema` or
`contract` stages.** The substance of the claim holds: `rust:test` already applies
`apply-db.sh` against a real Postgres service and runs the DB integration tests, which is the
check that matters here. So §7.13 is anchored to what exists.

I am asking Aegis whether those stages are planned or whether `rust:test` is the intended gate.
Per the kickoff's own instruction — surface open questions, do not stall step 1 — **this does not
block you.** If a `schema`/`contract` stage appears while you are working, tell me.

---

*Dispatched by Lyra · 2026-09-30 · P0 build plan §3 step 1 · Aegis kickoff GO · slot `010` post-seed
on top of the landed runtime triple · all §1 facts measured with `git show b6de097:<path>`*
