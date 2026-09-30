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

> **Companion (2026-09-30):** this document settles *capture* (emit/observe/
> interpose) and *fidelity* (tiers T1/T2/T3). How the objects compose — the
> entity hierarchy, the **topic** semantic layer, the three data *kinds*
> (activity/object/context), and the replay/traceability use case — is in
> `docs/coordination/reviews/2026-09-30-aegis-object-model-and-traceability-v1.md`.

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

## 7. File modifications — reference git, do not duplicate it

A file the agent edits via a tool is not one kind of data; it spans three
layers, each with a different home. Getting this wrong (copying every file
version into the database) is a large, avoidable mistake.

| Facet | What it is | Home |
|---|---|---|
| **The act** | each `Edit`/`Write` call — how the file evolved during the session | **T2 process trace** (fine-grained, deferred) |
| **The durable result** | the file's content now, usually fixed by a commit | **git** — the authoritative store; bytes are not copied |
| **The work-graph fact** | "seat X produced/changed artifact Y" | **T1 event** referencing `(commit_sha, path, content_digest)` |

### 7.1 Principles

1. **Do not duplicate git; reference it.** Source file truth lives in git.
   SeatLoom stores a reference — `(path, content_digest, commit_sha)` — not the
   bytes as truth. Same "observe, do not own" as the tmux mirror. Copying every
   source version into PostgreSQL is both enormous and redundant with git.
2. **One existing exception.** Documents that SeatLoom itself renders, reviews,
   or searches (the markdown coordination docs) *do* have their bodies ingested —
   `documents` / `document_versions` already store `body_text` + `body_digest` +
   `revision` + `run_id` provenance. The boundary: **PostgreSQL is canonical for
   the work graph and the document bodies SeatLoom renders; git is canonical for
   source-file content; the two are joined by digest.** This does not contradict
   AD-011, which is about structured project data, not source bytes.
3. **Content addressing is identity.** Reference a file by `content_digest`
   (sha256 of bytes), not by path alone. `body_digest` already exists; extend it
   to every file-bearing event. The same artifact then stays identifiable across
   harnesses and re-emissions, giving dedup and integrity for free.
4. **The commit is the natural T1 granularity.** Measured: one session made 62
   `Edit`/`Write` calls but 23 commits. The edit stream is T2; the commit is the
   durable, linkable unit. One git commit maps cleanly to one `ArtifactChanged`
   event carrying `(commit_sha, changed paths + digests)`.
5. **For files, capture-by-observation is reliable — and this strengthens the
   emit model.** Note the asymmetry: semantic events (handoff, verdict, decision)
   have no structured source and must be self-reported via MCP emit; **file
   changes have a structured source — git itself.** So SeatLoom can *observe*
   git (a post-commit hook or repo watch) and derive `FileCommitted` /
   `ArtifactChanged` events reliably, without trusting a seat to self-report
   them. The highest-volume, most-verifiable change type is thus guaranteed by
   observation; only genuinely semantic events depend on the agent's emit
   discipline. This retires the "emit trust" concern for the file case.

### 7.2 The consistent picture

SeatLoom's relationship to git equals its relationship to tmux: **observe,
reference, do not copy.** tmux is the execution plane for the terminal (mirrored
via `pipe-pane`); git is the durable plane for files (observed via commits).
Both are referenced by identity — a session/pane for tmux, a `(commit_sha,
digest)` for git — never duplicated as a second source of truth.

## 8. Actions on the world — the receipt pattern already exists

Beyond producing files, agents *act on external systems*: open a browser,
authenticate, operate, test, screenshot, SSH to a host, run a command, verify,
observe. This is a distinct category, and most of its schema already exists —
`channel_action_receipts` and the AD-012 prompt tables (schema 005). Do not
reinvent them.

`channel_action_receipts` already carries the right shape: `target_kind`,
`target_id`, `action_kind`, `actor_ref`, `source_channel`, `evidence_refs`,
`policy_summary`, `idempotency_key`, `expected_revision`, `applied_revision`,
`receipt_status`, `result_event_id` — an action on a target, with evidence,
policy, safe-retry, optimistic concurrency, and a link to its canonical event.
It was shaped for supervisor channel-actions; agent world-actions either
generalize it or take a sibling, but the pattern is proven. (As everywhere: it
is seed-only today; nothing live writes to it.)

### 8.1 The decisive difference from files: no external store to observe

| | File modification | World action |
|---|---|---|
| External structured store | **git** | **none** |
| Capture | **observe** (reliable) | **emit / from tool call** only |
| Audit fallback | git history | **T3 raw transcript** — the only record of what the agent actually ran |

"SSH to prod and restart a service" leaves no structured external ledger SeatLoom
can observe. So world-actions depend on capture more than files do, and their
audit stakes are higher. This makes the emit-trust question sharpest here, and
its answer concrete: **reconcile emitted action-events against the raw transcript
(T3)**, which is the ground truth of what tools were actually invoked.

### 8.2 Three facets (parallel to file modifications)

| Facet | What | Home |
|---|---|---|
| **Mechanics** | the exact command / HTTP request / screenshot bytes | T2 tool call + Content (a screenshot is content-addressed like a file) |
| **Effect** | external state changed | **T1 action event / receipt**: `target` + `action_kind` + outcome + `policy` + `idempotency_key` |
| **Observation** | evidence gathered about external state | **evidence linked to a claim** (`evidence_refs`) |

Observation is evidence, and it connects directly to existing product contracts:
acceptance-spec §7 requires evidence packages (screenshots/recordings). `test`,
`verify`, `screenshot`, `observe` all produce evidence attached to a claim (a
WorkItem verdict, an acceptance). High-risk effects (`deploy`, `restart`) use the
`idempotency_key` and `expected/applied_revision` columns already present for
destructive-action safety.

### 8.3 The risk axis and a security rule

World-actions carry a risk dimension nothing else does, and it is already
modeled: read-only observation (low), external effect (high), credential use
(sensitive) flow through `policy_summary` / AD-012 classification / US-P0-11
(classify → policy → approve or take over). One hard rule: **credentials never
enter the data model.** An authentication action records "authenticated to
service X at time T" as an audit fact; the secret is never stored — the same
discipline as gitignored secrets referenced but never committed.

### 8.4 The T1/T2 split still governs

Routine operations (`ls`, `grep`, a read-only check) are T2 mechanics. A
world-action becomes a T1 event only when it has a work-graph consequence: a
deploy, a gating test verdict, evidence captured for an acceptance. The test is
the same as everywhere — does it change the work graph.

### 8.5 The capture-reliability spectrum

Pulling §7 and §8 together, the three change categories differ by how reliably
they can be captured, which dictates the mechanism:

- **Files** — external store is git → **observe** (most reliable).
- **World actions** — no external store, high audit value → **emit + tool-call
  capture + T3 reconciliation** (the hard middle).
- **Pure semantic events** (handoff, verdict, decision) — no source at all →
  **emit only**.

## 9. Capture strategy — emit, observe, or interpose

Where does capture happen? Three positions, in increasing strength and cost:

| Position | Mechanism | Reliability |
|---|---|---|
| **Emit** | agent calls MCP to report what it did | trust / discipline dependent |
| **Observe** | watch the artifacts (git, tmux output) | reliable with a structured store (git), brittle without |
| **Interpose** | SeatLoom *is* the tool (managed tmux, git, ssh proxy); all I/O flows through a choke point | structural — capture cannot be skipped |

Interposition's precise value: **it converts a "must-trust-emit" channel into an
"observe" channel** — wherever you interpose, you manufacture the external
structured store that §8.1 found missing. It is also *control*, not just capture:
a SeatLoom ssh proxy can block destructive commands and force approval on prod
(US-P0-11), so capture and governance become one mechanism. It is the eventual
answer to capture completeness — but "wrap everything" is wrong.

### 9.1 What to interpose — by principle, not by ambition

Interpose only where **(a) capture is otherwise impossible or untrusted AND
(b) the stakes are high.** Applied:

| Tool | Position | Why |
|---|---|---|
| **git** | observe, do not interpose | git-observe is already reliable; a managed git remote adds little |
| **tmux** | managed session (already largely built) | `pipe-pane`/`send-keys`/attach exist; launching through SeatLoom is a small extension that also pins session identity (`external_ref`) |
| **ssh / remote exec** | **interpose — the top candidate** | exactly the "no external store, high audit" gap; captured at the terminal choke point (§9.2), not per-tool |
| **browser** | interpose (foundation exists) | high value (screenshots, web actions); a browser-automation capability already exists |
| **arbitrary CLI / HTTP** | recording shell (§9.2) + T3 | the terminal choke point captures the whole CLI category faithfully; only non-shell tools fall back to emit |

### 9.2 Prefer the lightest interposition that works

Nearly all world-actions go through the shell (ssh, git, curl, tests, deploys),
so **the terminal is the natural single choke point for the whole CLI category**
— one point instead of N tool-proxies, and unlike emit or an MCP broker it does
not depend on the agent choosing a special tool: it captures even a raw-Bash ssh.
For the CLI category this beats both per-tool proxies and the MCP broker on
completeness.

**One precise requirement decides whether it works: capture at the execution
shell / PTY, not the display pane.** The existing tmux mirror (`pipe-pane`)
captures the harness's *UI rendering*, which is lossy — Claude Code's Bash tool
captures command I/O internally and shows a collapsed block, so `pipe-pane` on a
seat's pane never sees the raw ssh command and its output. Faithful capture needs
the agent's commands to run through a **recording shell** (a shell shim as
`$SHELL`, or a recorded PTY). "Managed iTerm + tmux" is the delivery vehicle; the
recording shell is the actual capture point. This is a natural *upgrade* of the
tmux bridge that already exists — from display mirror to execution capture — not
a new foundation.

What it covers, measured on the 8.8 MB sample: `Bash` was 307 of ~409 tool calls,
so the recording shell captures the large majority of tool activity completely.
`Edit`/`Write` (direct harness file ops), `Read`, and reasoning bypass the shell
and are covered by git-observe and emit respectively. So the terminal is the
complete solution for the *CLI-action* category, not for all data.

**It does not replace emit; it makes emit trustworthy.** The recording gives
complete, faithful raw capture (every command + output) = the T3 ground truth.
Turning "ran `systemctl restart x`" into a T1 semantic "restarted service X"
still needs a lift (parse or emit). So: recording = complete, non-repudiable
T2/T3; emit = T1 semantic labelling. And the recording is exactly the ground
truth §8.1 named for reconciling emit against — this mechanizes that
reconciliation rather than leaving it to discipline.

Two lighter/heavier companions remain:

- **MCP action broker.** Still useful for *structured* actions (`run_on_host`
  returns structured results, manages credentials), but completeness is backed by
  the recording shell, not by the agent remembering to use the tool.
- **Egress gateway / sandbox.** The network-level variant for the sandboxed end
  state: one choke point captures all outbound I/O and doubles as isolation and
  budget enforcement, at the cost of sandboxing the agent environment.

### 9.3 Reconciling with "mirror, not replacement"

Interposition (a mandatory choke point) does conflict with "kill SeatLoom and
tmux survives." Resolve it **per channel by risk**, not globally:

- **Common channels** (tmux, git) → observe / fail-open: if SeatLoom is down,
  direct access still works. Non-destructive, as today.
- **High-risk channels** (prod ssh) → interpose / fail-closed is *desirable*: no
  SeatLoom means no prod access. Here control is the point.

"Observe, do not own" is thus not absolute — it is graded by stakes.

### 9.4 It activates schema that already exists

The interposing proxy is the missing *live writer* for `channel_action_receipts`
(§8): that table's `target` / `action_kind` / `evidence_refs` / `policy_summary`
/ `idempotency_key` / `result_event_id` shape was built for exactly this. So
interposition is not a new foundation — it feeds one already laid.

### 9.5 Sequencing

- **P0** — no proxies. Emit (semantic events) + git-observe (files) + managed
  tmux (mostly built). Enough to make the data flow real.
- **P1** — the recording shell delivered via managed tmux/iTerm: the terminal
  choke point that captures the whole CLI-action category faithfully (§9.2),
  upgrading the existing display mirror to execution capture, plus the MCP action
  broker for structured actions; both feed `channel_action_receipts`.
- **P2** — transparent ssh proxy / egress gateway / sandbox: the end state for
  capture completeness, carrying isolation and policy enforcement with it.

The direction is sound and eventually necessary, but it is the roadmap for
*capture completeness*, not the P0 starting point.

## 10. Open questions for Mr. Zhang and Lyra

1. **T2 storage substrate.** When T2 is built, does it live in PostgreSQL
   (queryable, joins to T1 cheaply) or in object/file storage indexed by the
   join key (cheaper for 98%-volume data, JOINs are lookups)? The choice can be
   deferred, but the join key (§5.3) cannot.
2. **Uncommitted deliverables.** The clean P0 path makes a commit the T1
   checkpoint (§7.1.4). For a deliverable a seat produces but does not commit,
   does the agent emit `ArtifactProduced` with a digest and optional body
   snapshot (the documents-ingestion path), or is "commit it" a required
   discipline for P0?
3. **Search over source.** SeatLoom can index git blobs for retrieval (§P0
   capability 5) without storing them as source of truth. Is a git-backed search
   index in P0 scope, or is P0 retrieval limited to the work graph and rendered
   documents?
