# SeatLoom Object Model and Traceability

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | review-2026-09-30-object-model-and-traceability |
| status | draft — pending Mr. Zhang and Lyra review |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| depends_on | `docs/coordination/reviews/2026-09-30-aegis-data-flow-and-layering-v1.md`, `docs/coordination/reviews/2026-09-30-aegis-p0-scope-takeover-and-capture-v1.md` |
| tags | object-model, topic, semantic-layer, replay, traceability, lineage |

## 0. Purpose

The data-flow doc (dependency above) settled *how data is captured* (emit /
observe / interpose) and *at what fidelity* (tiers T1/T2/T3). This document
settles *how the objects compose* — the entity hierarchy, the missing semantic
layer (topic), and the acid-test use case that validates the whole model:
replaying and tracing a project. It consolidates the design conversation into a
form an implementer can build from.

## 1. Three kinds of data — and what the tiers actually tier

A correction that the model needs stated explicitly: **T1/T2/T3 tiers *activity*
only. Not everything is activity.** Any datum is one of three kinds, and only the
first is tiered:

| Kind | What it is | Tiered? | Home |
|---|---|---|---|
| **Activity** | something that happened (an agent did / thought / said; a human directed) | **yes — T1/T2/T3** | the event ledger (+ traces + raw) |
| **Object** | something that exists (code, document, screenshot, artifact) | no | its authoritative store (git, doc store, blob store); **referenced by** events |
| **Context** | the setting activity happens in (host, cwd, branch, harness, model) | no | attributes on the `session` (and `external_ref`) |

### 1.1 The unified promotion rule

Within *activity*, the three tiers relate by one rule:

> **T3 is everything (the raw record). T1 is the consequential subset promoted to
> typed facts. T2 is the connective process tissue between them.**

The single promotion criterion is **"does it have a work-graph consequence?"** —
and it is applied identically across every activity form:

- Every shell command is in T3; a consequential one (a deploy) is also a T1
  event; routine ones (`ls`, `grep`) are T2.
- Every message is in T3; a consequential one (a directive, a decision) is also a
  T1 event; conversational filler is T2.
- Every edit is in T3; a shaped result (a commit) is also a T1 event; the
  intermediate edits are T2.

### 1.2 Placing the common data forms

| Form | Kind | Where it lands |
|---|---|---|
| source code | object | git owns it; a change = T1 event referencing `(commit, digest)`; edit stream = T2/T3 |
| human → AI instruction | activity (supervision) | all in T3; consequential intent/decision = T1 event (already: `cmd_append_supervisor_message`); filler = T2 |
| AI → human reply | activity (supervision) | all in T3; substantive decision/answer = T1 event; filler = T2 |
| machine / operating environment | context | `session` attributes (host, cwd, branch, harness, model); **not** a tier |
| screenshot | object (evidence) | content-addressed store; referenced by a T1 observation event |
| a deploy / ssh command | activity (world-action) | mechanics = T2, effect = T1 event, evidence = object |

## 2. The object hierarchy

Five levels, from durable container to atomic fact:

```
project            durable container; one per real body of work
  └─ topic         semantic thread — a coherent subject over a stretch of activity   ← the missing layer
       └─ workitem  a governed unit of work (owner, lifecycle, acceptance) — optional
            └─ event    the atomic fact (T1)
session            a continuous harness run; CROSS-CUTS topics (carries context)
```

`session` sits to the side deliberately: it is mechanical (one harness run) and
cross-cuts the semantic hierarchy — one session spans many topics, and one topic
spans many sessions. Every event belongs to a project, is attributed to a
session (hence a context), usually to a topic, and sometimes to a workitem.

## 3. The topic semantic layer

### 3.1 What a topic is, and what it is not

A **topic** is a semantic thread: a coherent subject or goal over a stretch of
activity. It is distinct from the two entities it is easily confused with:

- **not a session** — a session is one continuous harness run; a single
  conversation-session routinely covers several topics, and a topic can resume
  across several sessions.
- **not a workitem** — a workitem is a governed unit with an owner, a lifecycle,
  and acceptance. Some topics spawn workitems ("investigate project X's bug");
  many do not ("AI industry news"). A topic is looser and broader.

Topic is the **mid-level unit between project (too coarse) and event (too fine)**
— and it is the granularity at which replay and search become useful to a human.

### 3.2 Why it is needed — the real problem

One continuous conversation commonly covers unrelated threads. A real pattern:
the human, in one sitting, discusses AI-industry news, then asks to investigate a
project problem, then requests a product-manual revision, then scopes a personal
project. The human did **not** segment these. Without a topic layer, that
session is one flat stream: replay returns an undifferentiated wall, and search
cannot say "the part about the project problem." Topic is the index that turns
the flat stream into "here are the four threads; this is the one you want."

### 3.3 How it fits the model — zero new primitive

Topic reuses the existing fabric. It is **an object that events reference** via
the role-tagged refs from data-flow A3:

```
topics(id, project_id, title, status[open|closed|merged],
       opened_by, opened_at, closed_at, parent_topic_id?)

event → topic:  event_object_refs(event_id, role='subject',
                                   ref_type='topic', ref_id=<topic_id>)
```

No new mechanism: `topics` is a table like `workitems`; the event-to-topic link
is the same join fabric already used for every other object.

### 3.4 Assignment is derived, revisable, and provenance-carrying

The load-bearing design decision. Which topic an event belongs to is a
**labelling** — and labels follow the same rule as attribution and cross-harness
identity elsewhere in this design: **derived and revisable, never baked in as
ground truth at write time.** Topic boundaries are often only clear in hindsight,
and a better model can re-segment later.

So an assignment records its provenance and can be overridden:

```
topic_assignment: event_id, topic_id, assigned_by, method, confidence, evidence, created_at
   method ∈ { human, supervisor_ai, agent_self }
```

The consequence that answers the original question directly: **"human segments"
vs "AI segments" is not an architecture choice — both are just producers of the
same revisable label.** The data layer must accept a topic label from any
producer with provenance and allow revision; the interaction form is a separate,
later decision that changes no schema.

### 3.5 The producer model (recommended)

Three producers, one revisable dataset — isomorphic to the capture strategy
(who produces the signal, at what reliability):

| Producer | Character | Role |
|---|---|---|
| **Supervisor AI (e.g. Aegis)** | low-friction, automatic, inferential | **default** — scans the event stream, detects topic shifts, opens/attaches topics |
| **Human** | high authority, low frequency | corrects: confirm / split / merge / rename (overrides the AI) |
| **Acting agent** | high confidence when certain | self-tags on emit (a seat working on wi-X knows its topic) |

Do **not** force manual human segmentation — the four-topics-in-one-conversation
pattern proves humans will not do it reliably. Default to supervisor-AI proposal,
human correction, agent self-tag where certain.

### 3.6 Worked example

One continuous session with the supervisor; Aegis segments four topics:

| topic | kind | spawns |
|---|---|---|
| topic-1 AI industry news | discussion | — |
| topic-2 investigate project X problem | investigation | wi-investigate |
| topic-3 product-manual revision | work | wi-manual-edit + artifact |
| topic-4 personal project Y | work | workitems |

The session is one (continuous); it is segmented into four topic threads.
Replay-by-topic yields the four threads separately. If Aegis mis-segments (e.g.
merges 3 and 4), the human splits them — because the assignment is revisable.

### 3.7 Priority — reserve the join now, add the logic later

Same "reserve the join fabric before you need it" lesson as `external_ref` and
the T2 join key:

- **P0** — the `topics` table and the event→topic ref exist from the start
  (cheap; a join key you do not want to retrofit). Even a trivial default topic
  per session is enough to reserve the link.
- **P1** — Aegis auto-segmentation (real-time shift detection) + the human
  correction controls.
- **P2** — retrospective re-segmentation of the historical corpus (and it
  improves as models improve).

## 4. Replay and traceability — the validating use case

This is the acid test: can human, AI, environment, action, and object be strung
together into one record that replays, searches, and migrates?

### 4.1 The event ledger is the spine

Every one of the five is either an event or is referenced by one. An event is a
hub with five spokes:

```
                 ┌── actor_ref ──────► who (human / which AI seat)
                 ├── session_id ─────► environment (host/cwd/branch/harness/model)
   EVENT ────────┤                      + that session's T2 process
                 ├── type + payload ─► the action (what happened)
                 ├── object refs ────► objects (topic / workitem / commit / artifact), role-tagged
                 └── content ref ────► the bytes (git commit + digest)
```

**Replaying a project = walking its events in time and expanding each along those
spokes.** Time orders them; session binds environment and process; refs link
objects and topic; actor names human or AI.

### 4.2 Concrete chain (real object ids from this project)

The runtime-triple change, as an ordered event chain:

| # | event | actor | session → environment | refs |
|---|---|---|---|---|
| 1 | `SupervisorMessage` "converge seats, design platform" | human | aegis session | project |
| 2 | `WorkItemCreated` wi-runtime-triple | seat:lyra | lyra session | subject=wi, topic |
| 3 | `HandoffSent` → nimbus | seat:lyra | lyra session | wi |
| 4 | `SessionStarted` | seat:nimbus | branch `packet/nimbus/…`, harness stepcode-claude, model opus-5 | — |
| 5 | *(T2: reasoning + greps + edits, joined by session_id+turn_seq)* | seat:nimbus | same | — |
| 6 | `ArtifactChanged` | seat:nimbus | same | subject=commit `b10280d` + digest, ctx=wi |
| 7 | `ReviewVerdictIssued` PASS | seat:flux | | wi, evidence=pipeline `#901696` |
| 8 | `WorkItemStatusChanged` → done | seat:lyra | | wi |

### 4.3 The three operations

- **Replay** — the ordered event stream for a project (or, at human scale, for a
  *topic*). It is **zoomable**: read at T1 for the headline narrative, drop to T2
  for the reasoning behind a step, drop to T3 for the verbatim bytes — the tiers
  are linked by `session_id + turn_seq`.
- **Search** — deterministic retrieval along any spoke (by object, by actor, by
  action type, by content digest, by time window, by topic), via the structured
  index + full-text layers (AD-011). Finding the exact record is a query, not an
  LLM reread (US-P0-10).
- **Migration** — the record is portable because nothing in the spine depends on
  the harness that produced it: seats and hosts are referenced by `external_ref`,
  object content by `digest + commit`, and the whole ledger lives in PostgreSQL +
  git — never in `~/.claude` or `~/.codex`. Switch a seat from Claude Code to
  Codex, or move the project to another machine, and the events, refs, objects,
  and environment snapshots all still resolve.

### 4.4 Point-in-time fidelity

Faithful replay requires the ledger, not the projections. **The ledger is
"then"; the projections are "now."** Because events are append-only and reference
an immutable session context, replay shows what was true at each step even after
branches are deleted or models are switched. The environment is snapshotted at
the session/event, not looked up live.

### 4.5 Before and after (why this is not yet possible)

Today, to replay this episode the data is scattered: human intent in one seat's
`~/.claude` jsonl, Nimbus's work in another, environment in ephemeral tmux, code
in git, and the linkage exists only in coordination markdown and human memory.
After: one ordered event stream in PostgreSQL, refs resolving to git objects,
zoomable, searchable, portable. That gap is the P0 work.

## 5. Open questions for Mr. Zhang and Lyra

1. **Default topic.** In P0, before auto-segmentation exists, is one default
   topic per session acceptable (reserving the link), or should the supervisor
   assign a topic at session start even manually?
2. **Topic vs workitem overlap.** When a topic spawns exactly one workitem, are
   they distinct rows (topic = the thread, workitem = the governed unit) or does
   the workitem subsume the topic? Recommendation: keep them distinct — the topic
   outlives the workitem and can carry discussion that is not itself work.
3. **Cross-project topics.** Can a topic span projects (e.g. a decision that
   affects two projects), or is a topic strictly within one project?
   Recommendation: within one project in P0; cross-project links via refs later.
