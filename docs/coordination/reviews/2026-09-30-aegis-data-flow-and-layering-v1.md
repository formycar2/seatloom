# SeatLoom Data Flow and Layering

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | review-2026-09-30-data-flow-and-layering |
| status | draft — pending Mr. Zhang and Lyra review |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| depends_on | `docs/coordination/reviews/2026-09-30-aegis-p0-scope-takeover-and-capture-v1.md`, `docs/coordination/reviews/2026-09-29-aegis-platform-design-macro-to-micro-v1.md` |
| tags | data-flow, data-model, layering, mcp, event-schema, reasoning-traces, model-training |

## 0. What this document does

Designs the data flow, then tests it against real data: does the proposed
structure actually hold the information a live agent produces? It also introduces
the **layering** the work requires — the governed work graph is a thin spine on
top of a large body of intermediate process data, and the two must not be
conflated.

## 1. The write mechanism — capture by emission, not by parsing

The agent emits governed data; SeatLoom does not reverse-engineer it from bytes.
An LLM's true output surface into SeatLoom is **one event API**, exposed as an
MCP server (the current harness, stepcode Claude Code, already supports
`--mcp-config`; the repo has no MCP scaffolding yet — greenfield).

```
[human] configures Registry (project / seat / role)
   │
[agent, any harness] does a meaningful thing
   │  calls SeatLoom MCP tool in the governed format
   ▼
emit_event({ type, schema_version, actor, subject, refs, payload })
   │
[SeatLoom MCP server]  1. validate against the event schema  ← governance is enforced here
   │                   2. append to canonical_events + event_object_refs
   │                   3. update projections in the same transaction
   ▼
data is now: filed under project · linked (lineage via refs) · retrievable
   ├─► [human] reads projections, searches, traverses lineage
   └─► [agent] subscribe()/query() relevant events → collaboration flows as data
```

Governance lives at step 1: `type` must be in the closed vocabulary, `payload`
must match that type's versioned schema (platform design A2), or the event is
rejected. This is what "generate data in the governed format" means concretely.

## 2. The three data tiers (the layering)

Captured agent activity is not one thing. Measured against a real transcript
(§3), it splits by fidelity and volume into three tiers that must be stored and
governed differently.

| Tier | What | Volume | Governance | Priority |
|---|---|---|---|---|
| **T1 — Canonical events** | The work graph: work begun, artifact produced, handoff sent, verdict issued | ~1–2% of turns | Typed, versioned, schema-validated at emit | **P0** |
| **T2 — Process traces** | Reasoning output, search queries + hits, RAG retrievals, tool calls — the *how* behind each event | ~98% of turns | Captured but **not** governed; own schema | **TODO** |
| **T3 — Raw transcript** | The full harness byte stream | the whole file | Archived as-is, per harness | **P2** |

The critical rule: **T1 is a thin spine, not the whole record.** Forcing T2
(reasoning, search) into the event envelope would turn the clean work graph into
a flood. They are different layers on purpose.

Note a structural asymmetry: **T1 is harness-agnostic for free** (the agent emits
the same event shape regardless of harness), while **T2 and T3 are per-harness**
(each harness's transcript format differs). This is a second reason T2 is
deferred — it is the harder, format-specific work.

## 3. Empirical validation — does the structure hold?

Sampled one seat's session: `~/.claude/projects/.../<uuid>.jsonl`, 8.8 MB.

| Record / block | Count | Tier |
|---|---|---|
| assistant turns | 927 | — |
| user turns | 480 | — |
| `thinking` blocks (reasoning output) | 326 | **T2** |
| `tool_use` / `tool_result` | 409 / 409 | **T2** |
| of which `Bash` (incl. grep/search) | 307 | T2 (search) |
| of which `Read` | 7 | T2 (retrieval) |
| `Agent` (subagent fan-out) | 2 | T2 (RAG-like) |
| `AskUserQuestion` (decision point) | 13 | candidate T1 |
| `Write` / `Edit` (file production) | 22 / 40 | some T1, most T2 |
| `text` blocks (assistant messages) | 295 | T2/T3 |

**Verdict on T1 sufficiency.** The canonical event envelope
(`type · schema_version · actor · subject · refs · payload`) *is* sufficient to
carry every work-graph event in this transcript: "produced artifact X for
workitem Y", "handed off to seat Z", "issued verdict V" all map cleanly to
`subject` + `refs` + `payload`. The schema holds.

**Verdict on T2.** It is real, it is the bulk (98%), and it does not fit T1 —
correctly. 326 reasoning traces and 409 tool calls per session are not work
events; they are the process that produced the work events. They need their own
tier.

## 4. The four data types (write/read rules)

Orthogonal to tiers: by role in the system, data is one of four kinds. This is
the governance boundary for *who may write what*.

| Type | Written by | Nature | Examples |
|---|---|---|---|
| **Registry** | human / supervisor | declared, slow-changing | `projects`, `seats`, `project_role_bindings` |
| **Events (T1)** | **LLM, via MCP emit** | append-only; the only write interface for agents | `canonical_events` (+ `event_object_refs`) |
| **Projections** | machine, derived from events | current-state read model; **never written directly** | `workitems`, `handoffs`, `artifacts`, `sessions` |
| **Content** | LLM / files | bodies and versions, for retrieval | `documents`, artifact files |

The single hard rule: **an agent's only write path is emitting T1 events (and
attaching content). It never mutates a projection directly** (platform design
A1). Registry is human. This keeps the work graph derivable and auditable.

## 5. T2 process traces — TODO, and why they matter

Recorded now as a deferred layer, not built in P0. But the join must be designed
now so they can attach later without reworking T1.

### 5.1 What they are

- **Reasoning output** — the agent's `thinking` blocks: how it decided.
- **Search content** — queries and hits (grep, index lookups): what it looked for.
- **RAG / retrieval content** — documents and context pulled in, subagent
  fan-out results: what it read to decide.
- **Tool-call detail** — the fine-grained action stream under each event.

### 5.2 Why capture them (future purpose)

1. **Workflow optimization.** With T2 linked to T1 outcomes, questions become
   answerable: how many tool calls and reasoning steps does a seat spend to
   produce one accepted artifact? Where is effort wasted, where do retries
   cluster, which retrievals never get used? Optimization stops being anecdotal.
2. **Training vertical models.** T2 reasoning traces paired with T1 outcomes are
   `(context, reasoning, result)` triples — a distillation / fine-tuning dataset
   drawn from the team's own accepted work. This is the compounding asset: the
   record captures not just *what* was decided but *how it was reasoned*, which
   is exactly what a domain model needs to learn. Capturing it early, even
   ungoverned, means the dataset accrues from day one rather than starting at
   zero when the need becomes urgent.

### 5.3 The one thing to design now: the join key

T2 can be deferred, but its **attachment point cannot**. Every T1 event must
carry enough identity that a T2 record can be linked to it later:

- `session_id` — which session produced it (via `external_ref`, harness-agnostic).
- `turn_seq` — monotonic position within the session.
- the event's own `id` and `subject` ref.

Lay these on T1 events now (they are nearly free), and T2 becomes an additive
layer keyed on `(session_id, turn_seq)` whenever it is built — never a T1
redesign. This is the same lesson as `external_ref`: reserve the join fabric
before it is needed.

## 6. Consequence for P0

P0 builds **T1 only, via the MCP emit loop** (§1), with the join key (§5.3)
reserved on every event. T2 is a documented TODO layer; T3 is the existing
corpus (P2, findable-under-project only). This keeps P0 small and the abstraction
future-proof.

The P0 event vocabulary starts from the ten types already present in seed:
`WorkItemCreated`, `WorkItemStatusChanged`, `ArtifactCreated`/`ArtifactProduced`,
`SessionStarted`/`SessionEnded`, `HandoffSent`/`HandoffAccepted`/`HandoffCompleted`,
`ReviewVerdictIssued`, `SeatDelegationIssued`/`SeatDelegationClosed`. The skill
(P0 scope §7) instructs the agent to emit at exactly these points.

## 7. Open questions for Mr. Zhang and Lyra

1. **T2 storage substrate.** When T2 is built, does it live in PostgreSQL
   (queryable, joins to T1 cheaply) or in object/file storage indexed by the
   join key (cheaper for 98%-volume data, JOINs are lookups)? The choice can be
   deferred, but the join key (§5.3) cannot.
2. **Emit trust.** The agent self-reports T1 events. Do we cross-check emitted
   events against the raw transcript (T3) to catch a seat that forgot to emit, or
   trust the skill's discipline in P0 and reconcile later?
3. **Content addressing.** Artifacts are files today. Should T1's `subject` for a
   produced artifact carry a content digest so the same artifact is identifiable
   across harnesses and re-emissions (dedup, integrity), rather than by path?
