# SeatLoom P0 Build Plan and Gap Register

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | review-2026-09-30-p0-build-plan-and-gap-register |
| status | draft — pending Mr. Zhang go/review |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| depends_on | `docs/coordination/reviews/2026-09-30-aegis-p0-scope-takeover-and-capture-v1.md`, `docs/coordination/reviews/2026-09-30-aegis-data-flow-and-layering-v1.md`, `docs/coordination/reviews/2026-09-30-aegis-object-model-and-traceability-v1.md`, `docs/coordination/reviews/2026-09-29-aegis-platform-design-macro-to-micro-v1.md` |
| tags | plan, gap-register, p0, implementation, sequencing |

## 0. Purpose

Closes the design phase. Consolidates every gap surfaced across the design
conversation into one prioritized register, defines the P0 build, and specifies
the first executable packet. This is the design-to-build transition artifact.

## 1. Gap register

Every gap found by stress-testing the model against real scenarios. Priority is
by the P0-scope contract: P0 = make new work capturable and connected;
P1 = compounding assets and richer capture; P2 = history and cost.

| # | Gap | What it is | Priority | Source |
|---|---|---|---|---|
| G1 | Event as sole write primitive (**A1**) | every mutation = append event + projection update in one txn; projections rebuildable from ledger | **P0** | platform design §7, A1 |
| G2 | Typed + versioned event envelope (**A2**) | `event_type` closed vocab + `schema_version` + validated `payload`; hand-written FromStr/Display | **P0** | platform design A2 |
| G3 | `event_object_refs.role` (**A3**) | `subject`/`actor`/`context` — the lineage join fabric | **P0** | platform design A3 |
| G4 | `external_ref` (**A4**) | `(harness/system, native_id) → object`, `pinned`/`derived` — harness-agnostic identity | **P0** | p0-scope §8, ingestion review |
| G5 | `sessions.project_id` | session records only `workspace_path`; cannot attribute to a project (project ≠ path) | **P0** | project-vs-path analysis |
| G6 | Topic layer | `topics` table + event→topic ref; assignment derived/revisable | **P0** (reserve ref) / P1 (auto-segment) | object-model §3 |
| G7 | MCP emit server | the write mechanism; agents emit governed events; no scaffolding exists | **P0** | data-flow §1 |
| G8 | File events reference git | change = event with `(commit, digest)`; do not copy bytes | **P0** | data-flow §7 |
| G9 | Dual-authority resolution (**A5**) | typed layer over PostgreSQL; retire parallel file-typed store | P1 | platform design A5 |
| G10 | Typing de-duplication (**A6**) | one subtype allow-list source | P1 | platform design A6 |
| G11 | Capability asset (Playbook / Skill / Tool) | **published** asset: stable id + version chain + application-by-reference events + composition-specialization; general/vertical/custom is a derived view over the usage graph, NOT fork lineage | P1 | test scenario, generalization discussion |
| G12 | T2 process traces | reasoning / search / RAG capture; join key `(session_id, turn_seq)` reserved on T1 now | P1 (reserve key P0) | data-flow §2, §5 |
| G13 | Recording shell / terminal capture | execution-shell (not display-pane) capture of CLI actions; the faithful T3 for world-actions | P1 | data-flow §9 |
| G14 | Standing commitment / recurring task | the "future-tense" object: definition + cadence generating firing-events; = AD-006 deferred daemon | P1 | interval-task scenario |
| G15 | SUT (system-under-test) version dimension | test results are meaningful only per `(case × SUT version × method version)`; not modeled | P1 (Mr. Zhang to confirm) | test scenario |
| G16 | Budget as first-class object (**A7**) | per-scope token accounting + enforcement | P2 | platform design A7 |
| G17 | Historical corpus ingestion | the ~16 GB across harnesses; findable-under-project only in P0/P1, deep relationship reconstruction later | P2 | ingestion review |
| G18 | Interposition proxies / egress gateway | ssh/browser proxy, sandbox; capture completeness end-state | P2 | data-flow §9 |

## 2. P0 build — what the first cut delivers

P0 makes **one true thing** exist that does not today: a record in SeatLoom
because an agent, on some harness, did something — filed under its project and
topic, linked to what it touched, retrievable, and independent of the harness.

P0 scope = G1–G8 (the write side + the reserved joins) plus the first slice.

Explicitly NOT in P0: capability assets, T2, recording shell, recurring tasks,
budget, historical ingestion, new L2 surfaces.

## 3. Sequencing — schema joins first, they are cheap now and expensive later

| Step | Work | Owner | Done when |
|---|---|---|---|
| 1 | **Schema foundation**: G2 event envelope (`schema_version`, closed vocab), G3 `event_object_refs.role`, G5 `sessions.project_id`, G6 `topics` + event→topic ref, G12 join key `(session_id, turn_seq)` reserved on events | Onyx | migration idempotent (applies twice clean); value-domain contract stated; existing seed still loads |
| 2 | **Write API (G1/A1)**: one `append(event) → projects` path; no repository mutates a projection directly | Nimbus | a projection table can be rebuilt from `canonical_events` alone (the A1 success test) |
| 3 | **`external_ref` (G4)**: table + pin seat sessions launched via SeatLoom | Onyx + Nimbus | seat sessions `pinned`; a session resolves to its object harness-agnostically |
| 4 | **MCP emit server (G7)**: minimal `emit_event(type, payload, refs)`, schema-validated | Nimbus | a seat's `--mcp-config` points at it; a malformed event is rejected |
| 5 | **First slice**: one seat emits one real `ArtifactProduced` (with `(commit, digest)` per G8), it lands via the write API, links to a workitem + topic + project, and is retrievable | Nimbus + Flux verify | a human can query the event and traverse to the artifact and back — a record that satisfies the P0 acceptance (§2), which zero records do today |

Steps 1–2 are the critical path; 3–5 make it real and harness-agnostic.

## 4. First packet (ready to specify in full)

**Onyx — schema foundation (step 1).** Bounded, additive, idempotent. Must carry
a verbatim value-domain contract (the discipline that kept the runtime-triple
change aligned): the event `type` vocabulary, the `role` enum
(`subject`/`actor`/`context`), the `schema_version` rule, and `topics`/`external_ref`
column domains — reproduced in both the Onyx and Nimbus packets. Next migration
slot after the current highest; base schema stays pre-seed, this rides as a
post-seed migration per `scripts/apply-db.sh` ordering.

Everything downstream (write API, MCP server, first slice) depends on this
landing, so it is the first thing to build.

## 5. Acceptance for P0

> A record exists in SeatLoom because an agent did something on some harness;
> it is filed under its project and topic, linked to what it produced, findable
> by exact query, and unchanged by which harness produced it.

Verified by Flux with a concrete trace, not a structural check.

## 6. Open items folded into the pool (decide as reached, not now)

Takeover strength (data-flow §9.3), T2 substrate (data-flow §8.1), topic-vs-
workitem distinctness (object-model §5), SUT-version priority (G15). None blocks
steps 1–5; each is decided when its step arrives.
