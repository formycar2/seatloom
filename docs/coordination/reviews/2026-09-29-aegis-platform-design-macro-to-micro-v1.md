# SeatLoom Platform Design: Macro to Micro

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | review-2026-09-29-platform-design-macro-to-micro |
| status | draft — pending Mr. Zhang and Lyra review |
| author | aegis |
| date | 2026-09-29 |
| version | v1 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/architecture-decisions.md`, `docs/coordination/reviews/2026-09-28-aegis-multi-harness-session-ingestion-architecture-v1.md`, `docs/coordination/ENGINEERING_STANDARDS.md` |
| tags | design, platform, data-model, event-sourcing, abstraction, roadmap |

## 0. Purpose and method

The workbench is built: quality gates are mechanical, CI is green, the six seats
run one harness. This document answers the next question — what the platform is
*for*, what real work it carries, and whether the data abstraction can hold that
weight.

Method note: every claim below is measured against the running system (local
PostgreSQL, the Rust crates, the Tauri command surface), not inferred from the
specs. Where the implementation and the contract disagree, that disagreement is
the finding.

---

# PART I — MACRO: what business this platform actually carries

## 1. The reference customer is this project itself

SeatLoom's first and clearest customer is the operation that built it:

| Measured reality | Value |
|---|---|
| Active AI seats under one supervisor | 6 (aegis, lyra, mira, nimbus, flux, onyx) |
| Distinct project working directories in harness history | ~30 |
| Accumulated harness session corpus | ~16 GB across 5 harnesses, 8 months |
| Coordination artifacts produced | 351 markdown documents |
| Harness migrations survived | all 6 seats re-hosted in one afternoon (2026-09-28) |

This is not a hypothetical persona. It is a working pattern: **one human
supervising a standing team of AI seats across many projects, over months.**

## 2. What is actually scarce

The PRD's thesis is already correct and worth restating in operational terms:
the bottleneck is not agent capability. Two things are scarce.

**Supervisor attention.** One human cannot poll six seats. Today's session
proved it: seat progress was invisible until a purpose-built observation script
existed. Attention spent reconstructing state is attention not spent deciding.

**Token budget.** Context is repurchased every time continuity is lost. A seat
relaunched without its history pays again for what the system already knows.

Everything the platform does should reduce one of these two costs. A feature that
reduces neither is decoration, regardless of how interesting it is.

## 3. The durable moat: SeatLoom outlives the harnesses

The strongest strategic fact surfaced this week: **harnesses are volatile and
the record must not be.**

In a single afternoon all six seats moved from a mix of Gemini CLI, bare shells,
and stale resumes onto one Claude Code harness on a pinned model. Nothing about
the work changed. If the seats' identity, history, and obligations had lived
inside the harnesses, that migration would have destroyed them.

This is the defensible position. Not "orchestrate agents" — many tools will do
that. Rather: **be the layer that holds project truth while the agent layer
churns underneath.** The corpus in §1 is the asset; the harnesses are
replaceable tenants.

Two consequences follow, and they should be treated as product law:

1. **SeatLoom is local-first because its subject matter is local.** The seats are
   local tmux sessions; the 16 GB corpus is on local disk. A remote deployment
   cannot see either. This is why remote hosting was ruled out — not a limitation
   but a definition.
2. **SeatLoom observes; it does not own.** The tmux mirror already follows this
   (kill SeatLoom, tmux is untouched). Ingestion must follow it too: read-only
   against every harness store.

## 4. What the platform sells, stated plainly

> A standing AI team keeps its memory, its obligations, and its cost discipline
> across every harness, model, and project it touches — and one human can
> supervise it without polling.

The three nouns are the product: **memory** (continuity), **obligations**
(work graph and governance), **cost discipline** (budget and determinism).

---

# PART II — MESO: how to build the platform

## 5. The central diagnosis: the write side does not exist

This is the most important finding in this document.

Measured across `src-tauri/src/commands/`: **49 Tauri commands. 38 are `list_*`
or `get_*`.** The eleven non-read commands are:

| Command | What it actually mutates |
|---|---|
| `cmd_append_supervisor_message` | appends one row to `canonical_events` |
| `cmd_reconcile` | ingests markdown files into `documents` |
| `cmd_launch_session`, `cmd_kill_session`, `cmd_attach_tmux_session`, `cmd_pty_write`, `cmd_pty_write_bytes`, `cmd_pty_resize` | runtime/tmux, not domain state |
| `cmd_open_supervisor_window`, `cmd_close_supervisor_window`, `cmd_supervisor_window_status` | window management |

**There is no command that creates a WorkItem, sends a Handoff, issues a
Delegation, records a review verdict, or advances any object's status.**

Compare against the PRD's P0 promises: `US-P0-02` (create and route a task),
`US-P0-04` (scoped delegation), `US-P0-05` (reissue after review failure),
`US-P0-13` (approve from mobile). Every one of these is a write. None has a path.

So the honest description of SeatLoom today is: **a well-built read-only viewer
over seeded fixture data, plus a faithful tmux mirror, plus a supervisor message
log.** The viewer is real and the mirror is real. The work loop is not.

### 5.1 Why this matters more than any missing surface

Two failure modes follow directly, and both are already visible:

- **Every new L2 surface is a viewer over fixtures.** Building Sessions,
  Documents, WorkItems views does not move the product closer to its promise,
  because there is nothing to view that a human created through the product.
- **The L1 rule is right but currently toothless.** "Supervisor IM is L1" is a
  correct prioritization. But IM today can only *append a message*. Intent
  cannot become a state change, so the L1 surface cannot deliver the L1 promise
  (`US-P0-02`: intent → structured proposal → confirmation).

The L1/L2 ordering rule should therefore be sharpened:

> L1 is not a surface. L1 is **one closed write loop**: intent in, state change
> out, visible consequence. A surface that cannot close the loop is not L1 no
> matter how central it looks.

## 6. Proposed next milestone: one vertical write slice

Not a new module. One object type, all the way through.

**Slice: "create and route one WorkItem from the Supervisor IM."**

```
supervisor types intent
  → Supervisor parses to a structured proposal (deterministic, no LLM for routing)
  → Gate Engine validates (required fields, role constraints, budget)
  → confirm
  → ONE transaction: append WorkItemCreated + WorkItemRouted events
                     + update the workitems projection
  → Route Engine projects into the owner seat's Inbox
  → Timeline row renders from the event via Template Engine
  → dispatched to the seat's tmux session
  → seat's delivery returns as an Artifact linked to the same WorkItem
```

Why this slice, specifically:

- It touches **all five PRD modules** (Data Engine, Seat, Supervisor, Artifact
  review, and Playbook by way of the launch pack) with the smallest possible
  object surface. It is the thinnest cut that proves the architecture.
- It is the loop the team already performs by hand every day — today Lyra wrote
  two task packets and dispatched them via `tmux send-keys`. The product would be
  replacing a real, observed, repeated manual workflow, not a guessed one.
- It forces the event-ledger question (§7) to be answered for real rather than
  deferred.

Acceptance should be behavioural, not structural: **a WorkItem that exists in
PostgreSQL because a human typed intent into SeatLoom, with a complete event
chain explaining how it got there.** Today zero objects satisfy that.

## 7. The ledger is decorative, and that is the root abstraction problem

`canonical_events` is designed as the audit spine. AD-010 makes review failure
event-first; AD-009 requires delegation to never mutate ownership history. Both
depend on the ledger being real.

Measured in the live database:

| Fact | Value |
|---|---|
| Rows in `canonical_events` | 34 |
| Origin of those rows | **all from seed SQL** |
| Distinct event types present | 10 (`WorkItemCreated`, `HandoffSent`, `SeatDelegationIssued`, …) |
| Code paths that write to the ledger | **1** (`append_supervisor_message`) |
| Event types the code can emit | 1 |

So the vocabulary exists in fixtures, the schema exists, and the writer does not.
When domain writes are eventually added the default path is the obvious one:
`UPDATE workitems SET status=...` with an event appended beside it — **dual
write**, which is exactly the drift AD-009/AD-010 were written to prevent.

The fix must land *before* the write side is built, because afterwards every
mutation site has to be rewritten.

---

# PART III — MICRO: seven concrete abstraction changes

Ordered by leverage. Each states the measured problem, the change, and why it
matters.

## A1. Make the event the only write primitive

**Problem.** Projections (`workitems`, `handoffs`, `sessions`, …) are plain
mutable tables. Nothing structurally prevents mutating them without an event.

**Change.** One write API: `append(event) -> projects`. Every domain mutation is
an event append plus its projection update **in a single transaction**. No
repository method mutates a projection table directly. Reads stay on the
projections (fast, indexed, unchanged) — this is CQRS-lite, not full event
sourcing, and deliberately so: the projections are already well-shaped.

**Why.** It is the only way AD-009's "never mutate ownership history" and
AD-010's "review failure is event-first" become structurally true rather than
aspirational. It also gives the supervisor a real answer to "how did this get
this way", which is §2's scarce resource.

**Test of success.** You can drop every projection table and rebuild it from
`canonical_events` alone. If you cannot, the ledger is still decorative.

## A2. Type and version the event envelope

**Problem.** `canonical_events` is `event_type TEXT` + `payload JSONB` with **no
version column**. The moment a payload shape changes, old and new rows are
indistinguishable — in an append-only table that can never be migrated away.

**Change.**

```
event_type      TEXT NOT NULL      -- closed vocabulary, PascalCase
schema_version  INT  NOT NULL      -- per event_type, starts at 1
payload         JSONB NOT NULL     -- validated against (event_type, schema_version)
```

Mirror it in Rust as a typed enum with **hand-written `FromStr`/`Display`**, per
the serialization-boundary rule codified in ENGINEERING_STANDARDS: serde's tagged
representation does not match a flat TEXT column. The runtime-triple work already
proved this pattern (`parse_custom_tagged` in `objects/session.rs`).

**Why.** An append-only log with an untyped, unversioned payload is a liability
that grows monotonically. Fixing it costs one migration now and is impossible
later.

## A3. Promote `event_object_refs` to the real join fabric

**Problem.** The table is `(event_id, ref_type, ref_id)` with `ref_type` as free
text. Measured: **21 `workitem` refs for 6 workitems** — so refs carry no role
distinction. "Which workitem is this event *about*" and "which workitem is merely
*context*" are indistinguishable.

**Change.** Add a `role` column with a closed domain:

| role | meaning |
|---|---|
| `subject` | the object this event is about (exactly one per event) |
| `actor` | the seat/human that caused it |
| `context` | related objects, for retrieval joins |

Constrain `ref_type` to the object vocabulary.

**Why.** This table is what makes "show me everything about X" a single indexed
query instead of a scan over ten tables. It is also the join fabric the
multi-harness ingestion needs (`§3.3` of the ingestion review). Both features
depend on it, so it should be strengthened once, deliberately.

## A4. Introduce `external_ref` as a first-class abstraction

**Problem.** SeatLoom objects carry SeatLoom ids. Harness sessions carry harness
ids. There is no mapping, so a session in `~/.claude/projects/*.jsonl` and the
`sessions` row describing it have no structural link. Today's workaround was to
pin `--session-id` at launch and record the pairing in a TSV file — a stopgap
outside the data model.

**Change.** One table, applicable to any object:

```
external_refs(
  harness TEXT, native_id TEXT,      -- e.g. ('claude_code', '<uuid>')
  object_type TEXT, object_id TEXT,  -- internal identity
  confidence TEXT,                   -- 'pinned' | 'derived'
  evidence JSONB,                    -- how the link was established
  PRIMARY KEY (harness, native_id)
)
```

**Why.** This is the same lesson as the runtime triple, one level up: identity
must be explicit where two systems meet. It converts the ingestion review's
hardest problem (§2, "there is no shared key") from a guessing exercise into
recorded data — `pinned` for sessions SeatLoom launched, `derived` with evidence
for the 16 GB of history. And it retires the TSV stopgap.

## A5. Resolve the dual-authority split between typed and stringly paths

**Problem.** Two parallel representations of the same objects:

- `crates/seatloom-core/src/storage/seat_registry.rs` — file-backed, **typed**
  (`Runtime`, `SeatRole` enums)
- `crates/seatloom-core/src/db/models.rs` — PostgreSQL-backed, **stringly**
  (`Option<String>` passed straight to the DTO)

AD-011 declares PostgreSQL canonical and `.seatloom/` a cache. But the *cache*
has the stronger types, and the canonical path has none. Today's seed change was
safe **only because** the DB path never parses into the enum — a latent trap:
the seed contains `role = 'supervisor'` (snake_case) and `agent = 'Custom'`
(bare), neither of which matches serde's default derive for those enums.

**Change.** The typed layer sits **over** PostgreSQL, not beside it. Parse at the
DB boundary using the hand-written `FromStr` (A2), delete the parallel file-backed
typed store or demote it to an export-only serializer.

**Why.** Two authorities means no authority. And the current arrangement hides
the risk in the path the product actually reads.

## A6. Collapse the duplicated typing columns

**Problem.** `documents.artifact_id` correctly makes documents a layer over
artifacts (artifact = typed node in the work graph; document = parsed body,
header, versions). But **both tables carry `template`, `subtype`, and
`subtype_valid`.** If they disagree, nothing says which wins.

Compounding it, the subtype allow-list exists in **two Rust copies**
(`db/document_parser.rs` and `storage/artifact_store.rs`) plus the markdown table
in `DOCUMENT_TEMPLATES.md §11.1` — three sources, one unit test, and editing the
document alone silently desyncs the code.

**Change.** Typing lives on `artifacts` (the identity layer); `documents` reads it
through the FK. Generate the allow-list from one source and assert the other
copies against it in a test.

**Why.** Dual-key typing (AD-008) is load-bearing for the artifact review
surface. A load-bearing key stored twice is a latent inconsistency.

## A7. Make budget a first-class object, not a field

**Problem.** The PRD makes budget enforcement a P0 Data Engine contract
("hard input/output budgets at session, handoff, and pipeline scope"). Today
`AssistBudget` exists only inside `objects/session.rs` as prompt-assist scope.
There is no budget table, no ledger of consumption, no per-scope enforcement
point.

**Change.** A `budgets` projection keyed by `(scope_type, scope_id)` with
`limit`, `consumed`, `policy`, fed by `TokensConsumed` events (A1). Enforcement
reads the projection; the ledger explains every charge.

**Why.** §2 names token budget as one of the two scarce resources, and the PRD
promises enforcement "by product behavior, not team etiquette". A budget that is
a struct field inside one subsystem cannot be enforced across sessions, handoffs,
and pipelines, and cannot be *shown* — which is what the user actually needs.

---

## 8. Sequencing

The ordering matters more than the content: A1–A3 must precede the write slice,
because every one of them is cheap now and expensive after write paths exist.

| Step | Work | Owner | Gate |
|---|---|---|---|
| 1 | A2 typed/versioned event envelope | Nimbus + Onyx | migration idempotent, round-trip test |
| 2 | A3 `event_object_refs.role` | Onyx | closed domain enforced |
| 3 | A1 append-only write API | Nimbus | projections rebuildable from ledger |
| 4 | **Write slice (§6)** | Nimbus + Mira | a human-created WorkItem with a full event chain |
| 5 | A4 `external_refs` | Onyx | seat sessions `pinned`; ingestion unblocked |
| 6 | A5 dual-authority resolution | Nimbus | one typed path over PG |
| 7 | A6 typing de-duplication | Onyx | one allow-list source |
| 8 | A7 budget object | Nimbus | budget visible and enforced per scope |

Steps 1–4 are the critical path to a product that does something. Steps 5–8 pay
down abstraction debt that will otherwise compound.

## 9. What this design explicitly does not do

- **No new surfaces.** The L1 rule holds: close the write loop before adding L2
  views. §6 adds no screen that does not already exist.
- **No autonomy.** Every write in the slice is human-confirmed. P2 bounded
  automation stays out of scope.
- **No full event sourcing.** Projections remain the read model. A1 constrains
  *writes*, it does not rebuild reads around a stream.
- **No cloud.** §3 established local-first as a definition, not a phase.

## 10. Open questions for Mr. Zhang and Lyra

1. **Is the write slice (§6) the right first loop**, or should the first closed
   loop be review-verdict-and-reissue (`US-P0-05`) instead? Argument for
   WorkItem-create: it is the loop the team performs most often by hand.
2. **A1 forbids direct projection mutation.** That is a real constraint on every
   future contributor, including the seats. Accept as an engineering standard?
3. **A7 requires token accounting from the harnesses.** Claude Code reports usage
   per turn in its JSONL; other harnesses vary. Is per-harness partial coverage
   acceptable for P0, with `NULL` meaning "not reported" (the rule established
   for the runtime triple)?
