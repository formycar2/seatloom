# Delivery: backend filters by project — and the store stopped copying one unfiltered set into every project

[Nimbus -> Aegis] Mr Zhang reported that the project list shows many projects, but
switching projects changes nothing below it — the live-seats / supervision-overview
/ dispatch / workitem / session / handoff modules all look identical under SeatLoom
and under `ql_delivery`. Fixed at both layers, because there were two.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-17-project-scoped-reads-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-17 |
| version | v1 |
| to | aegis |
| dispatch | `disp-smUiJkscwc__D1g2DhD2R` |
| branch | `packet/nimbus/project-scoped-reads` (new, off clean `track/infra-foundation` @ `5c2b96f`) |
| tags | project-scoping, seats, join, dispatches, ui, store, artifacts, schema |

---

## 1. The measured root cause — two layers, not one

**Layer 1, the commands.** Four took **no** `project_id` at all and read their
tables unfiltered. A fifth took one and **silently substituted the app's default
project** when it was absent:

| command | before | why it mattered |
|---|---|---|
| `cmd_list_seats` | no `project_id` | every project got every seat |
| `cmd_list_workitems` | no `project_id` | idem |
| `cmd_list_handoffs` | no `project_id` | idem |
| `cmd_list_sessions` | no `project_id` | idem |
| `cmd_list_dispatches` | `Option<String>` → **default project** | 🔴 the sneakiest: it *looked* scoped and returned another project's rows |

`cmd_list_dispatches` is the one worth naming: the UI never passed the id, so the
dispatch list under `ql_delivery` was `seatloom`'s dispatches — a populated,
plausible, **wrong** list. A fallback that silently substitutes another project's
rows is worse than an error: the error says something is missing, the fallback says
nothing at all.

**Layer 2, the store.** `hydrateFromBackend` read those eight unfiltered lists
**once** and then copied the **same arrays** into every project's `ProjectData`
inside a `for (const p of projectsDto)` loop, under a comment saying *"v0.1: all
seeded data belongs to the single 'seatloom' project"*. So even a correctly-scoped
command would not have moved the numbers. Fixing only the commands would have
produced a green test suite and an unchanged product.

## 2. The schema asymmetry, measured before writing any code

| claim | measured |
|---|---|
| 13 tables have `project_id` | ✅ 13 — `dispatches canonical_events workitems sessions handoffs topics memories documents acceptances project_role_bindings prompt_instances review_threads channel_action_receipts` |
| `seats` has no `project_id` | ✅ 0 — its columns are `id name default_runtime capability_tags status created_at default_harness default_agent default_model` |
| `artifacts` has no `project_id` | ✅ 0 |
| no table has `project_code` | ✅ 0 columns named `project_code` anywhere |

**Consequence for `seats`, and it is not a detail.** Because `seats` has no
`project_id`, "the seats under this project" is **not a predicate on `seats`** — it
is a predicate on the binding table. `list_seats_for_project` therefore **joins**:

```sql
SELECT s.* FROM seats s
JOIN project_role_bindings b ON b.seat_id = s.id
WHERE b.project_id = $1 ORDER BY s.name ASC
```

A `WHERE project_id = $1` on `seats` would be a **syntax error, not a filter**. The
same runtime-triple validation runs on every row, so an out-of-domain
`default_harness` fails this read exactly as it fails the global one — a domain
violation is not made invisible by scoping.

## 3. No silent default — an error, and here is why not an empty list

`require_project_id` is one helper used by every project-scoped read. An **absent**
`project_id` is now an error naming the command and the reason.

**Why an error rather than an empty list**, which the dispatch allowed either of:
an empty list cannot be told apart from *"this project genuinely has no rows"* —
and that is a **real state**, because the historical-corpus projects bind no seat
and correctly return zero. Returning empty for a missing parameter would collide
with a legitimate answer and still read as data. An error cannot be mistaken for
data, and it surfaces in the caller's `catch`.

Applied to: **seats, workitems, handoffs, sessions, dispatches, events, documents,
acceptances.** The last three are beyond the four the dispatch named; they carry
the identical silent-default fallback and feed modules Mr Zhang named (timeline,
Artifacts tab, the pending-acceptance section). Stated as an addition, not
smuggled in.

**The UI now enforces it at build time.** All eight `api.ts` signatures require
`projectId`. On landing, `tsc` failed on **eight call sites** — every one of them a
place that had been silently unscoped. The compiler found the bug.

## 4. 🔴 Artifacts cannot be partitioned — measured, not asserted

| measurement | value |
|---|---|
| artifacts on live | 15 |
| with `source_session_id` | **0** |
| with `source_workitem_id` | **4** |
| with **neither** | **11** |

A derived join would attribute **4 of 15** and silently drop **11**. That is
precisely *"looks filtered but isn't"*: the operator would see a short list under
one project and a different short list under another, and neither would be the
whole truth. So the command **refuses** — it takes no `project_id`, and both the
command's doc comment and the store say why.

**Chosen: state it cannot be partitioned.** The operator's alternative is on the
branch as `infra/postgres/migrations/015_artifacts_project_id.sql` — nullable
column, **no backfill**, clearly marked as his act. A backfill that guessed would
be a lie with a convenient shape; null means *not recorded*, which is the honest
state for the 11 rows that cannot be derived.

**Worth knowing:** the bulk of what the Artifacts tab renders is **not** this table.
The UI merges `documents` (which **do** have `project_id`) with these 15 rows. The
document half is project-scoped; this half is not.

## 5. The supervision overview, traced command by command

`SupervisionDashboard` makes **no** `api.*` calls — it reads only
`projectData[activeProjectId]`. So its four cards are computed entirely from that
project's rows:

| card | source |
|---|---|
| pending decisions (`pendingDecisions`) | `inboxItems`, derived in the store from **this** project's workItems + handoffs |
| in progress (`inProgress`) | `workItems` where `status === 'Active'` |
| blocked | `workItems` Blocked **+** `sessions` InputRequired |
| in review | `workItems` where `status === 'InReview'` |

So fixing workitems / handoffs / sessions **automatically** fixes the overview —
there is no separate command to change. The timeline, by contrast, reads
`canonical_events` directly and needed its own scoping.

## 6. Measured

**Against live, per project:**

```
project=seatloom      seats=7  workitems=7  handoffs=3  sessions=5  events=400
  seat_ids=seat-aegis-001,seat-flux-001,seat-lyra-001,seat-mira-001,
            seat-nimbus-001,seat-onyx-001,seat-vela-001      ← NO seat-ql-001
project=ql_delivery   seats=1  workitems=0  handoffs=0  sessions=0  events=0
  seat_ids=seat-ql-001                                    ← ONLY ql
ColossalAI: bindings=0 seats=0      sf-code: bindings=0 seats=0
project_delivery/QL: bindings=0 seats=0      (root): bindings=0 seats=0
```

All five scoped reads **differ** between the two projects. The corpus projects are
empty **because they have no binding rows** — counted directly, not assumed.

**The overview's numbers move** — measured on a controlled scratch, because on live
they are all zero (live holds 7 workitems, all `done`, and 5 sessions, all
`completed`). Proving "the numbers change" against a database where they are all
zero would have been proving nothing:

```
proj_a: seats=1 workitems=2 overview_blocked=1 overview_in_review=1
proj_b: seats=1 workitems=0 overview_blocked=0 overview_in_review=0
```

Note `seats=1` on **both** sides — the scoping is per-project, not "A has data and
B has none".

## 7. Gates run

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-tauri --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 168 passed / 0 failed (2 ignored, need Postgres) |
| `cargo test -p seatloom-mcp-emit` | 81 passed / 0 failed (22 ignored) |
| `cargo test -p seatloom-waker` | 16 passed / 0 failed |
| `cargo test -p seatloom-tauri` | 10 passed / 0 failed — 3 of them new, pinning the no-silent-default rule |
| `cargo check --workspace --all-targets` | clean |
| `tsc --noEmit` (ui) | clean |
| `vitest run` (ui) | **194 passed / 0 failed** |
| `proof/project_scope_proof.sh` | ALL POINTS PASS |
| `verify-docs.sh` | the violations it lists are pre-existing on clean `track`; this delivery adds none |

## 8. Known limits, stated rather than hidden

- 🔴 **The live acceptance is not met.** Steps 1–3 (merge, app rebuild, reopening
  the project page) have not happened. The backend is measured against live; the
  **product** has not been looked at. I am not claiming the operator's checklist.
- **Rebased onto track @ `5c2b96f`**, which landed *while I was working* (Aegis's
  mira merge). Two files overlapped and were merged by hand: track's
  seat-attribution-by-binding filter is superseded by the backend join, but its
  `unboundSeats` feature is **kept** (see §9).
- **The same latent fallback remains in five commands the dispatch did not name** —
  `cmd_list_role_bindings`, `cmd_list_prompt_*`, `cmd_list_knowledge_*`,
  `cmd_list_reconcile`, `cmd_list_supervisor_*`. Reported, not silently fixed: each
  has call sites I have not measured. Listed verbatim by the proof.
- **`cmd_list_events_by_type` is still unscoped.** It is called by nothing in the
  UI today, so it is latent rather than live.
- **A fact in a merged comment is wrong.** Track's `useDataStore` comment says
  *"workItems / handoffs / events have project_id and could be; sessions and
  artifacts do not"* — **`sessions` does have `project_id`** (measured, and
  `list_sessions_for_project` has existed since schema 008). Harmless today because
  the comment describes a decision, not a rule, but it will mislead whoever reads
  it next.
- **The two QL projects.** `ql_delivery` and `project_delivery/QL` share a
  `home_path`; only `ql_delivery` carries the binding, so only it shows a seat. Not
  resolved here — Lyra flagged it as §1 #6.

## 9. What I kept from track's merge, and why

Track's merge had already fixed the **seat** half of this defect store-side: filter
`seatsDto` by `boundIds` inside the per-project loop, and carry `unboundSeats` so an
empty list can say why. That was a real fix and I kept its **second** half:

- `cmd_list_all_seats` — one deliberately unscoped read, whose only job is naming
  seats bound in **no** project (measured: `seat-vela-001`). A read whose whole
  purpose is "every seat, regardless of project" cannot be scoped, and pretending
  otherwise would break the one thing it is for.
- `unboundSeats` is still populated and still optional, so the common case is
  unchanged and no consumer learns a field it does not read.

I **replaced** its first half, because a client-side filter is a second, weaker copy
of a rule the backend now enforces — and because filtering only the seats left
workItems / handoffs / sessions / events as the same arrays under every project,
which is four of the five modules Mr Zhang named.

## 10. Out of scope, unchanged

The live database, any migration applied to live, gate events, the merge, and the
app rebuild.

## 11. Forward recording (D5 Phase 2 Lane B)

One memory on topic `topic-project-scoped-reads`, `source_kind=canonical_event`,
`source_ref` = `ev-project-scoped-reads-delivered`, `confidence=quoted`:

| Memory | Kind | Why it passes the three-way test |
|---|---|---|
| `mem-QIhDBMSScBZOgxmPBW48R` | decision | A project-scoped read must refuse an absent `project_id` with an ERROR rather than return an empty list, because "this project has no rows" is a legitimate answer and an empty list cannot be told apart from a missing parameter. Changes how the next scoped read is written; the reasoning is not written down anywhere; evidenced by the measured fact that the corpus projects correctly return 0 seats. |

No second memory: the join-not-WHERE point is a schema fact recorded in the code's
own doc comment, and the artifacts measurement is in the proof's POINT 9. No
preference was touched — Mr Zhang's report is a bug report, not a correction of me,
and his corrections are Aegis's to record.

## 12. Addendum — the report event

The report to Aegis is `ev-T5eYLIm9TO-Jt7fkIhYT8` (ArtifactCreated, subject
`disp-smUiJkscwc__D1g2DhD2R`).

Final state at delivery: branch `packet/nimbus/project-scoped-reads`, one commit
ahead of `track/infra-foundation` @ `5c2b96f`, tree clean, all gates green.
