# Task: P0 Step 2 — Write API (A1/G1): `append(event) -> projections`

[Lyra -> Nimbus] P0 step 1 (schema `010`) is merged, accepted, and reconciled to GitLab
(`231d0a5`, CI green). Step 2 is the GO, per
`docs/coordination/tasks/lyra/AEGIS-2026-09-30-p0-step2-write-api-v1.md`. This is the
**critical-path write side** — steps 4, 6 and 5 all ride on it. Toward user-milestone **UM1**:
"work leaves a clickable trace."

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | NIMBUS-2026-09-30-p0-write-api-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | nimbus |
| priority | P0 (critical path) |
| milestone | P0 capture loop — write side (UM1) |
| depends_on | `infra/postgres/schema/010_p0_schema_foundation.sql` (merged); build plan §3 step 2; `docs/coordination/ENGINEERING_STANDARDS.md`; `docs/coordination/ROADMAP.md` |
| base branch | `track/infra-foundation` @ `29adefd` (GitLab-reconciled `231d0a5` + dispatch) |
| delivery branch | `packet/nimbus/NIMBUS-2026-09-30-p0-write-api-v1` (**already created**) |
| **build location** | **`/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus`** |
| delivery path | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-p0-write-api-delivery-v1.md` |
| tags | nimbus, p0, write-api, a1, event-sourcing, um1 |

---

## 0. Build location — your own worktree, prepped

**Do not build in the shared checkout.** Standing rule: never build a code packet in the shared
tree while a UI dev server runs there — a watcher regenerates `src-tauri/gen/schemas/capabilities.json`
and dirties the tree under a commit-pinned gate. (For accuracy: no `tauri dev`/`vite` is running
*right now* and the shared tree is clean — I checked. The worktree is the durable pattern
regardless, and removes the question.)

I created and prepped it for you:

```
/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus
  branch packet/nimbus/NIMBUS-2026-09-30-p0-write-api-v1 @ 29adefd, clean
  ui/node_modules installed; ui/dist/index.html stub (both gitignored)
```

`cd` there and work. The shared checkout stays on `track/infra-foundation` — **do not switch its
branch.** Flux's worktree (`…/flux`) is not yours.

**Recovery note:** this packet file lives on `track`. It is present in your worktree at `29adefd`,
but if you ever need it after moving HEAD, read it with
`git show track/infra-foundation:docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-p0-write-api-v1.md`.
Commit your delivery doc on **your packet branch**.

## 1. Scope — one write primitive

**`append(event) -> projections`.** Every domain mutation is an event append **plus** its
projection update **in one transaction**. No repository method mutates a projection table
directly. Reads stay on the projections (CQRS-lite).

The ledger (`canonical_events` + `event_object_refs`) becomes the source of truth for anything
that changes state. Today it is decorative — rows are written *alongside* mutations rather than
*causing* them. Step 2 inverts that.

## 2. Obligations inherited from `010` — all four are verbatim requirements

`010` deliberately left half of its invariants to you. Each has a line-pinned anchor in the
merged migration.

### 2.1 Set `schema_version` **explicitly** on every append

`010:87` sets `DEFAULT 1`. That default is a **legacy-seed crutch only** — it exists because seed
001 inserts rows supplying no version. **New code must set it explicitly.** Relying on the default
is a defect, not a convenience. Dropping the default is a tracked P1 cleanup, so your code must
not depend on it still being there.

### 2.2 Set `event_object_refs.role` **explicitly**

Same shape. `010:220` already states the rule in the migration itself: *"MUST set role explicitly;
a defaulted role in new code is a bug."* Never rely on the `context` default.

### 2.3 **Exactly one `subject` per new event** — this is your half

`010`'s partial unique index (`010:332`, `WHERE role = 'subject'`) guarantees **at most one**.
The API must enforce **presence**: an append carrying **zero** subjects is rejected before it
reaches the database. Together the two halves make "exactly one."

This must be a typed, testable rejection with a negative test — not a comment, not an assumption.
A happy path proves nothing about it.

### 2.4 Closed domains, hand-written parsers

`event_type` ∈ the **12-value** vocabulary; `ref_type` ∈ its domain; `role` ∈
`{subject, actor, context}`. Enums crossing the Rust⇄PostgreSQL TEXT boundary get **hand-written
`FromStr`/`Display`**, never serde derive — ENGINEERING_STANDARDS, and the pattern already in
`crates/seatloom-core/src/objects/session.rs::parse_custom_tagged`.

Note `ArtifactProduced` is **not** in the vocabulary. The first slice reuses `ArtifactCreated`.

## 3. Value-domain contract (authoritative — Lyra-owned)

Reproduce this table **verbatim** in your delivery doc. Same discipline that kept the runtime
triple and `010` aligned across two seats.

| Field | Rust | Serialized domain | PG column | Nullable |
|---|---|---|---|---|
| `event_type` | enum, hand-written `FromStr`/`Display` | `SessionStarted`, `WorkItemCreated`, `WorkItemStatusChanged`, `ArtifactCreated`, `HandoffSent`, `HandoffAccepted`, `HandoffCompleted`, `ReviewVerdictIssued`, `SeatDelegationIssued`, `SeatDelegationClosed`, `SupervisorMessage`, `SeatResponse` | `TEXT` | no |
| `schema_version` | `i32` | integer ≥ 1, per `event_type`, starts at 1 | `INT` | no — **set explicitly** |
| `role` | enum, hand-written | `subject`, `actor`, `context` | `TEXT` | no — **set explicitly** |
| `ref_type` | enum, hand-written | `workitem`, `session`, `handoff`, `artifact`, `document`, `seat`, `topic`, `project`, **`delegation`** | `TEXT` | no |
| `turn_seq` | `Option<i32>` | integer ≥ 0 | `INT` | yes — `NULL` = not recorded, never `0` |
| `payload` | typed per `(event_type, schema_version)` | JSON object | `JSONB` | nullable in SQL; **your API should require it** for new events — state your choice |

PascalCase for `event_type`; snake_case for `role`/`ref_type`. Deviation requires telling me
before landing.

## 4. The A1 success test — and the scoping decision I am making for you

**A projection table must be rebuildable from `canonical_events` alone.** If it cannot, the ledger
is still decorative and step 2 is not done.

**But read this before you design the test, because I measured it and it changes the shape:**

Seeded historical events **cannot** support a rebuild. `WorkItemCreated` payloads in seed 001
carry only `{"workitem_id", "title"}` — e.g. `ev-010`. The `workitems` projection requires
`status NOT NULL`, `priority NOT NULL`, `created_at NOT NULL`, `updated_at NOT NULL`, plus
`goal`, `acceptance_criteria`, `owner_seat_id`. `priority`, `goal` and `acceptance_criteria`
appear in **no** seeded payload. The historical ledger is lossy by construction: those rows were
written for display, not for replay.

**Ruling: the rebuild test operates on events your write API itself produced, not on seeded
history.** Concretely:

1. In a scratch database, write N events through `append()` — enough to exercise create **and**
   update paths (`WorkItemCreated` then `WorkItemStatusChanged` is the natural pair).
2. Truncate or drop the projection rows those events produced.
3. Rebuild **from `canonical_events` + `event_object_refs` alone.**
4. Assert the rebuilt projection equals the pre-truncate state, field by field.

**Do not rewrite seed payloads to make history replayable.** That is forbidden — it fabricates
evidence and it is not this packet's scope. **Do state the limitation plainly in the delivery
doc**: which projections are replayable, from which point forward, and that pre-step-2 history is
not. A future backfill packet can decide what to do about it; hiding it is the only wrong answer.

If you conclude a different projection than `workitems` is the better first target, say why and
pick it — the requirement is that *a* projection round-trips, with reasoning, not that it is
specifically `workitems`.

## 5. Out of scope

- **No MCP emit server** (step 4). **No governed dispatch** (step 6). **No first captured event**
  (step 5, the UM1 slice). This packet is the primitive those ride on.
- **No UI.** Zero `.tsx` files. `ui/src/**/*.ts` type declarations only where `tsc` forces a
  minimal, behaviour-preserving fix that adds no new surface. (Wording deliberately wider than
  "type-only" — the runtime-triple gate showed "type-only" is too narrow for a refactor that
  removes a field the UI reads. Adding surface is still the violation.)
- **No `.sql` changes.** `010` is merged and accepted. If you believe the schema must change,
  stop and tell me — that reopens a closed gate.
- **Do not drop `010`'s defaults.** Tracked P1 cleanup; your code must simply not depend on them.
- **Do not rewrite seed payloads** (§4).
- **Do not run `scripts/verify-postgres-baseline.sh`** — still no compose provider locally, and
  it is destructive.
- **Do not write to the live `seatloom` database.** Scratch databases only.

## 6. Verification (Layer A — you run, Flux gates)

In your worktree:

```
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test  -p seatloom-core --lib
cargo clippy -p seatloom-core -- -D warnings
cd ui && pnpm exec tsc --noEmit
cd ui && pnpm build
git show --stat <sha>
```

Plus, against a **fresh scratch database** (never the live one):

```
DROP DATABASE IF EXISTS seatloom_s2_nimbus; CREATE DATABASE seatloom_s2_nimbus;
DATABASE_URL=<…> bash scripts/apply-db.sh        # must exit 0
```

then the three behavioural proofs:

1. **Atomicity** — a normal append writes event + projection **in one transaction**, and a forced
   failure mid-append rolls **both** back. Prove the rollback, not just the success.
2. **Zero-subject rejection** (§2.3) — typed error, with output.
3. **Rebuild** (§4) — the round-trip, with the equality assertion output.

**Tooling note:** there is no host `psql` client. Prior gates shimmed through
`podman exec -i seatloom-postgres psql`. Do the same, disclose it, and state that
`apply-db.sh`'s host-`psql` path stays unexercised locally — CI exercises it, so get the branch
through `rust:test`.

Expected and **not** your defect: a second full `apply-db.sh` run fails on `sessions.project_id`
NOT NULL. Known pre-existing seed defect, its own follow-up. Note and move on.

## 7. Done-definition

1. One `append(event) -> projections` path; **no repository method mutates a projection directly**
   — demonstrate it (e.g. the projection write is private to the append path).
2. Event + projection commit in **one transaction**; rollback proven.
3. `schema_version` set **explicitly** on every append; no reliance on `DEFAULT 1`.
4. `role` set **explicitly** on every ref; no reliance on the `context` default.
5. Zero-subject append **rejected** with a typed error; negative test output included.
6. `event_type`/`ref_type`/`role` parse via **hand-written `FromStr`/`Display`**; `ArtifactProduced`
   absent.
7. §3 contract reproduced verbatim; any deviation flagged to me before landing.
8. **A projection rebuilds from the ledger alone**, per §4, with the equality assertion shown —
   and the seeded-history limitation stated plainly.
9. All §6 commands green, verbatim output.
10. Zero `.tsx`, zero `.sql` files changed — `git show --stat` proves it.
11. Built in the Nimbus worktree; shared checkout never branch-switched.
12. Delivery doc complete; Flux commit-pinned verify PASSes.

## 8. Evidence expectations

Base/delivery branch and commit SHAs; verbatim transcripts, not summaries; **both** the success
and the failure path for atomicity; the negative test for §2.3; the rebuild equality output; and
an explicit statement of anything not done and why. A passing happy path is not evidence that a
failure mode works — three prior gates turned on exactly that distinction.

---

*Dispatched by Lyra · 2026-09-30 · P0 step 2, critical path · worktree-isolated · §4 scopes the
rebuild test to API-produced events because seeded payloads are provably lossy*

---

## ERRATUM (Lyra, 2026-09-30) — §3 `ref_type` was eight values; it is **nine**

**My error, corrected above.** The §3 contract table omitted **`delegation`**. The merged `010`
CHECK admits nine values (`010:265-271`) because `delegation` is in committed seed data —
`ev-020`/`ev-021` → `del-flux-acting-mira-001`. Onyx added it deliberately during `010`, flagged
it in its delivery, and I accepted that migration.

I then wrote this packet's table with eight by **copying my own earlier draft instead of
re-measuring against the migration that actually landed**. A Rust domain of eight could not read
existing history, so `RefType` correctly carries nine.

Nimbus caught it, notified me before landing as the contract requires, and pinned it with
`serialized_sets_match_schema_010_checks`, which asserts the serialized set equals `010` verbatim —
so a future widening forces the Rust side to follow. **No code change is needed; the table was
wrong.**

Same defect class as the runtime-triple ERRATA: a fact copied forward rather than measured at the
commit it describes. The rule stands and now extends to contract tables — **re-derive a value
domain from the merged schema, never from a previous packet.**
