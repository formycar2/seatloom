# SeatLoom PRD v0.5

| Field | Value |
|---|---|
| Product | SeatLoom |
| Document | PRD v0.5 |
| Status | Draft - Pending Lyra, Aegis, and Mr. Zhang review |
| Updated | 2026-04-28 |
| Language | English |
| Canonical entrypoint | `docs/PRODUCT_TRUTH.md` |
| Previous contract | `docs/archive/product-history/prd-v0.4.md` |
| Companion specs | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` |
| Support docs | `docs/architecture-decisions.md`, `docs/architecture-design.md` |

## 0. Why v0.5 exists

This revision turns the Apr 28 core-value deep dive into an executable product contract and then extends it with two additional core-design inputs:

- `docs/coordination/reviews/2026-04-28-playbook-vs-skills-and-ecosystem.md`
- `docs/coordination/reviews/2026-04-28-context-and-retrieval-design.md`

The key rule for this cycle is strict:

> No feature enters implementation unless it is present in all three layers:
> 1. a user story,
> 2. an interaction flow,
> 3. a screen or UI surface definition.

A second rule is now equally strict:

> No feature remains in scope unless the PRD explicitly states what pain it solves, what happens if it is omitted, and how user behavior changes after it exists.

v0.5 expands the contract around five modules that now form the product core:

1. Data Engine
2. Seat three-layer architecture
3. Playbook system
4. Supervisor Layer
5. Artifact review system

The goal is not to add abstract capability language. The goal is to define what the user sees, what the user can do, how the product behaves across P0, P1, and P2, and why each feature is worth building.

## 1. Product thesis

SeatLoom is a local-first continuity system for human-plus-agent project work.

Its core value is not "more AI agents." Its core value is higher output per cost through:

- deterministic routing, validation, retrieval, and audit work wherever LLM reasoning is unnecessary,
- a built-in supervisor that turns user intent into structured operations,
- reusable seat, skill, and playbook systems so teams do not pay the same context cost twice,
- auditable artifact review and handoff loops that keep work recoverable and governable,
- hard budget and isolation controls so token ROI is enforced by product behavior, not left as team etiquette.

## 2. Product architecture

SeatLoom operates as five connected product modules:

| Module | Purpose | Primary user-visible surfaces |
|---|---|---|
| Data Engine | Deterministically assemble, isolate, budget, validate, retrieve, and audit structured project data across runtimes | Inbox summaries, Timeline labels, context-tier pack previews, budget meters, evidence-search results, gate reasons, audit links |
| Seat three-layer architecture | Separate durable seat identity from project role and collaboration mode, while making seat capability truth visible | Seat Registry, Seat Card panel, Project Role Drawer, Collaboration Template sheet, Execution Template inspector, delegation badges |
| Playbook system | Separate learned reuse from authored seat guidance and inject both progressively where they help | LaunchPack playbook slot, seat-skill slot, Playbook Library, publish flow, savings counters |
| Supervisor Layer | Let the user drive the system by intent, not form-filling, and ask for just-enough evidence on demand | Command Bar, suggestion cards, routing rationale, decision-answer cards, continuity delta cards |
| Artifact review system | Let the team consume, search, review, annotate, and resolve artifacts inside SeatLoom, including dual-key typed coordination documents | Markdown viewer, typed metadata header, family/subtype-aware detail pane, review rail, in-artifact search, artifact family/subtype filters, resolve/dispute controls, version list |

These modules are not independent add-ons. They are the product core. Each one must appear in stories, interactions, and UX definitions before implementation begins.

## 3. Priority boundaries

| Priority | User promise | Included product scope | Not yet included |
|---|---|---|---|
| P0 | The user can start the day, create and route work, inspect seat capability truth, switch runtime, recover continuity, review outputs, find exact evidence without re-explaining the project, and keep urgent approvals moving from mobile when away from the desk. | Data Engine core, Supervisor command bar, seat identity plus project role binding, collaboration template selection, scoped delegation overlay, Seat Card visibility, seat-skill slot visibility, context tiers 0-2, structured checkpoints, LaunchPack preview, Layer 1 structured retrieval, Layer 2 full-text retrieval, budget enforcement, gate visibility, reissue flow, in-app artifact review, dual-key typed coordination artifact rendering, artifact family/subtype filters, manual comments, Mobile Overview, Mobile Inbox, mobile approval/action cards, mobile short-feedback return flow, mobile interrupt triage for `Prompt blocked` / `Gate needed` / `Handoff pending`, and mobile read-only event/artifact quick view. | Autonomous child-work creation, semantic recall, user-editable execution-template forking, cloud-first orchestration, full terminal development workflows on mobile, heavy document editing on mobile. |
| P1 | The user can reuse what the team has already learned, resume work with lower context cost, inspect execution progress with less polling, and review more supporting context from mobile without turning the phone into a second desktop IDE. | Playbook publish/apply flows, collaboration template cloning, supervisor-assisted artifact comments, richer supervisor continuity packs, semantic retrieval layer, session suspend/resume with delta context, Handoff `working` state and optional live activity mode, Execution Template inspector, version history and diff review, token-savings visibility, and richer mobile evidence quick view linked to approvals and feedback loops. | Full autonomy, self-changing governance, always-on background orchestration, full multi-pane review workbench on mobile. |
| P2 | The user can allow bounded automation proposals and compare which patterns actually save time or tokens without losing human approval and auditability. | Draft handoffs/artifacts/work proposals by seats, template recommendations, review-derived action suggestions, advanced ROI dashboards, execution-behavior version history comparisons. | Unbounded agent autonomy, silent state mutation, cloud-first coordination as a requirement. |

### 3.1 Interaction-frequency layers (L1 / L2)

Within each priority tier (P0, P1, P2), features are further classified by **interaction frequency layer**:

| Layer | Definition | Design implication |
|---|---|---|
| **L1** | Highest-frequency, shortest-path interactions — the supervisor's primary daily interface. Currently: **Supervisor IM** (intent → structured proposal → instant confirmation, no view-switching). | L1 surfaces receive investment priority within the same P-tier. Every sprint should strengthen L1 before expanding L2. |
| **L2** | Necessary but lower-frequency data views (Inbox, WorkItems, Artifacts, Sessions, Handoffs). Usage is 10–30× lower than L1 on a per-session basis. | L2 views are built after L1 is stable. They serve lookup / audit / batch-review needs and should link back to L1 for action. |

**Ordering rule**: within the same P-tier, L1 items are implemented before L2 items unless L2 is a hard prerequisite for L1 functionality.

### 3.2 Required mobile companion boundary

Mobile support is required in the active v0.5 contract, but its job is narrow and value-driven:

- Mobile exists so the user can monitor project health, clear urgent approvals, triage interrupts, and send short feedback back into the canonical work loop while away from the desktop.
- Mobile must support the P0/P1 monitoring-plus-feedback loop for `Prompt blocked`, `Gate needed`, `Handoff pending`, returned reviews, and other high-priority supervision events.
- Mobile is not a full development workbench: no long terminal sessions, no heavy markdown editing, no full artifact-review rail, and no expectation that the user authors large work packets from a phone.
- The Data Engine provides the sync, state, audit, retrieval, and budget foundation that mobile relies on, but the mobile requirement itself lives in product scope, interaction, and UX contracts rather than Data Engine rules.

## 4. Contract rule for implementation entry

A feature is not implementation-ready until all five conditions are true:

1. it has a user-facing story with a visible outcome,
2. it has a trigger-success-failure interaction definition,
3. it has a screen or UI container where the behavior is visible,
4. it has a priority assignment with explicit non-goals,
5. it passes the feature value test below.

### Feature value test

Every feature listed in this PRD must answer all three questions in writing:

1. What user pain does this solve?
2. What happens if we do not build it?
3. How does user behavior change after it exists?

If the team cannot answer those questions clearly, the feature is removed from delivery scope and does not enter implementation. This is a hard gate for P0, P1, and P2 alike. P1 and P2 are not allowed to remain as vague capability buckets.

## 5. Story map by priority

| Story ID | Priority | Modules | User outcome | Pain solved | If omitted | Behavior change | Interaction ref | UX ref |
|---|---|---|---|---|---|---|---|---|
| US-P0-01 | P0 | Supervisor, Data Engine | I open SeatLoom and immediately see what requires action, why it matters, and what SeatLoom recommends next. | Start-of-day supervision is slow because status is fragmented across chats, terminals, and memory. | The user keeps polling seats manually, misses blockers, and spends time reconstructing urgency. | The user starts from Overview and Inbox, trusts ranked next actions, and intervenes only where guidance says it matters. | `INT-01`, `INT-04` | `UX-01`, `UX-03` |
| US-P0-02 | P0 | Supervisor, Seat, Data Engine | I tell SeatLoom to create and route a task in natural language, review the structured proposal, and confirm it without filling a blank form. | Turning intent into a clean task packet is repetitive, easy to underspecify, and expensive in chat back-and-forth. | Task creation stays out-of-band, structure drifts, and seat execution starts from incomplete briefs. | The user issues intent once, reviews a proposal, edits only exceptions, and confirms structured routing in-product. | `INT-02` | `UX-02`, `UX-05` |
| US-P0-03 | P0 | Seat | I use the same seat identity across projects, but each project shows the correct role, authority docs, and active collaboration mode. | Teams lose continuity when seats must be recreated or re-explained for every project. | Capability truth drifts, setup is duplicated, and assignments happen without visible authority boundaries. | The user reuses known seats, checks project-specific role bindings, and assigns work with explicit authority context. | `INT-03` | `UX-04` |
| US-P0-04 | P0 | Seat, Supervisor | I can temporarily delegate a scoped piece of work from one seat to another and see that delegation clearly in the UI and timeline. | Temporary seat substitution is common, but proxy work is usually hidden or rewrites ownership history. | Acting-seat work remains ambiguous, audit trails break, and review feedback is sent to the wrong owner. | The user issues a scoped delegation overlay, tracks it visibly, and closes it without mutating original seat identity. | `INT-03`, `INT-04` | `UX-04`, `UX-05`, `UX-11` |
| US-P0-05 | P0 | Data Engine, Supervisor | When a delivery fails review, SeatLoom shows the failure reason, linked evidence, and reissue path without forcing me to reconstruct context manually. | Review failure currently creates vague redo loops and forces manual context restatement. | Rework becomes slow, blame-heavy, and inconsistent because the failure reason is not tied to evidence. | The user rejects or reissues from an evidence-linked review state instead of writing a fresh corrective brief from scratch. | `INT-05` | `UX-05`, `UX-07` |
| US-P0-06 | P0 | Data Engine | When I switch runtime or recover an interrupted session, SeatLoom generates the right continuity pack, shows what is inside it, and gives me a deterministic fallback. | Runtime changes and interruptions currently force expensive manual rehydration. | The user re-explains the project, loses continuity, and recovery quality depends on memory rather than system truth. | The user relaunches through visible pack previews and fallback steps instead of reconstructing context manually in the terminal. | `INT-06` | `UX-06` |
| US-P0-07 | P0 | Artifact review | I can open a full artifact inside the app, see the right template-family and subtype fields for its document kind, comment on a section, and route that comment back into the work loop. | Coordination artifacts lose value when every document looks like raw markdown and reviewers must infer whether it is a task, review, acceptance record, or governance rule by scanning the whole file. | Task packets, reviews, acceptances, and memory logs become slow to parse, hard to compare, and easy to route incorrectly. | The user opens typed artifact detail, reads the key family/subtype metadata first, comments in place, and routes feedback without leaving SeatLoom. | `INT-07` | `UX-07` |
| US-P0-08 | P0 | Seat, Data Engine, Supervisor | Before I assign or launch work, I can inspect a seat's capabilities, accepted inputs, constraints, and budget so routing is based on capability truth rather than role folklore. | Role labels alone are too coarse, so work is misrouted and seats are over-trusted. | Users keep assigning work by memory, seat mismatch remains invisible, and token budgets are broken after the fact. | The user checks a Seat Card, sees why a seat is eligible, and confirms routing with explicit constraints and budgets. | `INT-02`, `INT-03` | `UX-02`, `UX-04`, `UX-05` |
| US-P0-09 | P0 | Data Engine, Playbook | When SeatLoom builds continuity, I see context tiers, matched seat skills, matched playbooks, and budget consumption before launch. | Full-pack injection is expensive and opaque, so users cannot trust or tune context cost. | Runtime switching keeps wasting tokens, and continuity quality stays unpredictable. | The user launches, trims, or expands continuity based on a visible tiered preview instead of accepting a black-box pack. | `INT-06` | `UX-06` |
| US-P0-10 | P0 | Data Engine, Artifact review, Supervisor | I can search exact project evidence, filter by artifact family or subtype, and open the right source without asking a model to reread the entire project history. | Finding a known file, blocker, checkpoint, or specific review/acceptance record is slow when history is only readable as chat or raw files. | Humans and the Supervisor waste time and tokens scanning noisy results for exact references. | The user filters by object plus artifact family/subtype, opens the exact source faster, and loads deeper evidence only when needed. | `INT-13` | `UX-07`, `UX-10` |
| US-P0-11 | P0 | Data Engine, Supervisor | When a wrapped runtime hits an interactive prompt (menus, multi-step wizards, or freeform input), SeatLoom classifies it, applies policy, and either asks me to approve, lets me take over, or safely lets the Supervisor help complete the prompt without rereading long thinking logs. | Interactive CLI prompts stall work and force humans to watch scrolling output; naive automation burns tokens by re-parsing logs. | Work stops when humans miss prompts, or costs explode when tools rely on LLMs to interpret raw terminal output. | The user sees a clear prompt state, chooses human takeover or supervised assist, and keeps token cost bounded by deterministic classification and strict budgets. | `INT-16` | `UX-12` |
| US-P0-12 | P0 | Data Engine, Supervisor | From my phone, I can check project health, progress risk, waiting approvals, and blockers without opening the full desktop workspace. | Supervisors are often away from the desk when a project drifts, stalls, or needs a quick decision. | The user notices issues too late, falls back to noisy side-channel updates, or keeps polling seats manually until they return to the desktop. | The user treats mobile as a continuous monitoring surface, checks health in seconds, and decides whether a desktop intervention is necessary. | `INT-17` | `UX-13` |
| US-P0-13 | P0 | Supervisor, Data Engine, Artifact review | From my phone, I can approve, reject, or escalate a high-priority prompt, gate, or handoff decision with the same state change and audit trail that desktop would create. | Urgent approvals often block work while the decision-maker is away from the primary workstation. | Wrapped sessions stall, handoffs wait, and teams move decisions into unaudited chat messages or verbal instructions. | The user clears or escalates time-sensitive decisions from an action card, while SeatLoom keeps one canonical history across mobile and desktop. | `INT-18` | `UX-14`, `UX-15` |
| US-P0-14 | P0 | Supervisor, Artifact review | From my phone, I can send short feedback on a WorkItem, Handoff, or linked artifact and have it return as an auditable in-product object instead of a detached chat message. | High-context work often needs fast feedback when the reviewer is mobile, but phone-based feedback usually escapes the system of record. | Review intent fragments across chats, execution context drifts, and the receiving seat must manually copy or reinterpret what was said. | The user submits bounded feedback in SeatLoom, sees it linked to the target object, and expects the owner's Inbox and Timeline to reflect it immediately. | `INT-19` | `UX-14`, `UX-15` |
| US-P0-15 | P0 | Supervisor, Data Engine, Seat | When SeatLoom raises `Prompt blocked`, `Gate needed`, or `Handoff pending`, I can triage it from mobile and either continue, escalate, or reserve it for desktop takeover with context intact. | The most expensive interruptions happen while the human is not sitting in front of the full workstation. | Sessions idle, approvals age, and handoffs decay into status chasing because there is no governed interrupt path away from the desktop. | The user handles urgent interrupts from a phone as a controlled continuation surface rather than a passive notification sink. | `INT-20` | `UX-13`, `UX-14`, `UX-15` |
| US-P1-01 | P1 | Playbook, Data Engine | After resolving a blocker once, I can publish that lesson as a playbook and have it suggested automatically in the next relevant context. | The same runtime and workflow problems are solved repeatedly with fresh token spend. | The team keeps paying to rediscover known fixes and cannot compound operational learning. | The user publishes proven fixes once and expects future continuity packs to surface reusable playbooks automatically. | `INT-09` | `UX-06`, `UX-08` |
| US-P1-02 | P1 | Seat | I can clone a collaboration template from another project so a new project starts with known routing and governance rules. | New projects waste setup time by rebuilding seat rules and governance from zero. | Each project starts with inconsistent routing, hidden assumptions, and repeated coordination churn. | The user boots new projects from a proven template instead of reconstructing collaboration rules manually. | `INT-10` | `UX-04` |
| US-P1-03 | P1 | Supervisor, Artifact review | I can ask the Supervisor to turn spoken or loose review feedback into structured comments before I confirm them. | High-context review feedback is often faster to say than to rewrite into formal comments. | Valuable review insight stays trapped in ad hoc notes or is skipped because transcription is too expensive. | The user speaks or free-types review intent, inspects a structured draft, and confirms comments without manual reformulation. | `INT-08` | `UX-02`, `UX-07` |
| US-P1-04 | P1 | Supervisor, Data Engine | When the supervisor returns after a gap, SeatLoom rebuilds a role-appropriate continuity pack instead of a worker-only LaunchPack. | Supervisors and sponsors need different continuity than worker seats, especially after a break. | Leadership resumes blind, rereads raw history, or reuses worker context that omits decision-grade summaries. | The user re-enters through a supervisor-specific continuity pack and makes decisions from condensed role-appropriate evidence. | `INT-09` | `UX-03`, `UX-06` |
| US-P1-05 | P1 | Data Engine | I can suspend a session and later resume it with delta-only context instead of paying for a full relaunch. | Repeated runtime switching and recovery burn tokens on context that the runtime has already seen. | Context reconstruction stays expensive, repetitive, and lossy even when only a small delta changed. | The user pauses and resumes sessions as first-class operations instead of closing and rebuilding every time. | `INT-06` | `UX-06`, `UX-11` |
| US-P1-06 | P1 | Seat, Data Engine | I can see a handoff move into `working` and follow lightweight live activity without polling the terminal constantly. | After a handoff is accepted, progress becomes invisible until completion or failure. | Supervisors keep polling seats manually and lose confidence in long-running work. | The user monitors an optional live activity layer and a visible handoff state strip while preserving replay semantics. | `INT-12` | `UX-05`, `UX-11` |
| US-P1-07 | P1 | Seat, Data Engine | I can inspect a seat's Execution Template before trusting it with recurring work. | Seat behavior feels like a black box, so users cannot audit where tokens are spent or how work is staged. | Teams keep using seats without understanding whether the flow is deterministic, LLM-heavy, or verifiable. | The user reviews execution steps, budgets, and verification points before choosing the seat for recurring work. | `INT-03`, `INT-12` | `UX-04`, `UX-11` |
| US-P1-08 | P1 | Data Engine, Supervisor, Playbook | I can ask why a past decision happened and get retrieved evidence instead of raw history dumps. | Decision rationale is hard to recover when the relevant evidence is spread across checkpoints, artifacts, and prior sessions. | Users keep rereading large history slices or asking the model broad questions with poor grounding. | The user asks focused why-questions and receives a concise answer backed by linked evidence and playbooks. | `INT-14` | `UX-02`, `UX-10` |
| US-P2-01 | P2 | Supervisor, Seat | A seat may draft a handoff or artifact proposal for me, but nothing becomes active until I approve it. | Humans still spend time packaging obvious next-step proposals that the system can prepare safely. | Low-value orchestration work remains manual, slowing throughput even when the next action is predictable. | The user reviews queued draft proposals and approves or edits them instead of authoring every handoff from scratch. | `INT-11` | `UX-09` |
| US-P2-02 | P2 | Playbook, Data Engine, Seat | I can compare which playbooks, seats, and flows actually save time or tokens across projects. | Teams cannot improve reliably if they cannot see what actually reduces time, token cost, or rework. | Optimization stays opinion-based, and successful patterns are hard to justify or scale. | The user tunes seats, playbooks, and routing based on evidence dashboards rather than anecdote. | `INT-15` | `UX-08`, `UX-11` |
| US-P3-01 | P3 | Seat, Data Engine, Supervisor | In a complex project, I can run module workstreams in parallel using session groups and scoped delegation overlays before I duplicate core PO/architect seats. | One central PO/architect lane can become a throughput bottleneck, while early role replication often creates consistency drift and token overhead. | Teams either remain blocked on one lane or fragment into conflicting role copies with expensive coordination. | The user parallelizes by module, assigns temporary module leads with explicit scope/expiry, and keeps final gate truth centralized until replication is evidence-justified. | `INT-P3-01` | `UX-P3-01` |

## 6. Module contracts

### 6.1 Data Engine

#### User promise

SeatLoom does not waste model tokens on formatting, routing, validation, retrieval, or audit work that can be done deterministically.

#### Subsystems

| Subsystem | P0 contract | P1 extension | P2 extension |
|---|---|---|---|
| Template Engine | Generate human-readable Inbox and Timeline labels from event templates with zero LLM dependency | Add richer multi-object templates and localized phrasing rules | Add operator-tunable template packs |
| Pack Engine | Build continuity packs from context tiers 0-2, structured checkpoints, exact evidence hits, seat skills, and playbook matches | Add supervisor continuity packs, delta-only resume packs, and optional deeper evidence loading | Add proposal packs for bounded autonomy flows |
| Gate Engine | Validate state transitions, required fields, role constraints, relation integrity, and budget thresholds before commit | Add richer rule explanations and step-level verification summaries | Add predictive preflight warnings |
| Route Engine | Project canonical object changes into Inbox items, priority bands, and capability-aware seat suggestions | Add live progress projection and playbook-triggered route hints | Add proposal routing for autonomous drafts |
| Retrieval Engine | Query Layer 1 structured index plus Layer 2 full-text search before any semantic or LLM step | Add Layer 3 semantic retrieval for why-questions and similarity matching | Add cross-project ranking and recall tuning |
| Budget Enforcer | Enforce hard input/output budgets at session, handoff, and pipeline scope with graceful stop behavior | Add warnings, delta-cost estimates, and savings counters | Add policy tuning and comparative budget optimization |
| Isolation Layer | Enforce need-to-know access so worker seats see only pack-included or explicitly loaded evidence | Add external-tool access mapping and role-based retrieval expansion | Add operator inspection and policy simulation |
| Audit Engine | Bind every state-changing action to durable evidence and ledger events | Add reuse counters, token-savings estimates, and review-resolution links | Add cross-project analytics exports |

#### What the user sees

- readable Inbox rows instead of raw event names,
- readable Timeline rows instead of raw ledger codes,
- visible gate-failure reasons on WorkItems, Handoffs, comments, and launches,
- continuity previews split into visible context tiers,
- exact evidence search results before any LLM summary,
- artifact hits with human-readable family badges, subtype chips, status, and metadata chips extracted from document headers,
- budget meters and over-budget warnings before launch,
- deterministic fallback instructions when an injection or route fails.

#### User story

As a user, when SeatLoom creates, routes, validates, retrieves, or resumes structured work, I see ready-to-use summaries, reasons, exact evidence, and budget consequences without waiting for an LLM to restate facts the system already knows.

#### Interaction contract coverage

- `INT-01`, `INT-04`, `INT-05`, `INT-06`, `INT-12`, `INT-13`, `INT-14`, `INT-15`, `INT-17`, `INT-18`, `INT-19`, `INT-20`

#### UX surface coverage

- `UX-01`, `UX-03`, `UX-05`, `UX-06`, `UX-10`, `UX-11`, `UX-13`, `UX-14`, `UX-15`

#### Rules

1. If structured data already exists, the Data Engine must assemble and render it before any LLM call is considered.
2. Checkpoints must be structured first-class objects, not summary-only free text.
3. Retrieval order is fixed: structured index first, full-text second, semantic retrieval third, LLM explanation last.
4. Worker seats may access only pack-included or explicitly loaded evidence within policy limits.
5. Budget exhaustion must end in a structured stop, escalation, or partial-result handoff rather than silent truncation.
6. Every engine action that changes project state must emit an audit record and linked evidence.
7. Coordination artifacts that follow the template taxonomy must expose `template`, `subtype`, `id`, `status`, `author`, `date`, `version`, `depends_on`, `supersedes`, and `tags` as structured index fields before markdown-body fallback.
8. Artifact metadata is trusted for rendering, routing, gating, and retrieval only when `subtype` validates against the allow-list in `docs/coordination/DOCUMENT_TEMPLATES.md` Section 11.1.
9. Consent and interactive-prompt handling must be deterministic-first: approval triggers must never depend on a second LLM reading long terminal thinking logs. Prompt classification must use bounded windows and structured signals; LLM assistance (if used) must be budgeted, step-bounded, and policy-gated.
10. For wrapped sessions, interactive prompts must be modeled as a first-class state with a visible user choice: `Approve`, `Human takeover`, `Supervisor assist`, or `Stop`, and every choice must be auditable.

### 6.2 Seat three-layer architecture

#### User promise

A seat is not just a project-local role label. It has a stable identity, a project-specific role, a collaboration mode, and a visible operating contract.

#### Layers

| Layer | Scope | User-visible fields | P0/P1/P2 boundary |
|---|---|---|---|
| Seat Identity | Global | name, default runtime, capability tags, preferred budgets | P0 editable registry |
| Project Role Binding | Per project | role, authority docs, constraints, current owner, delegation badge | P0 required |
| Collaboration Template | Per project | template name, routing rules summary, prohibited paths summary | P0 visible and selectable; P1 clone/edit; P2 recommend |

#### Attached operating contracts

| Contract | Purpose | P0/P1/P2 boundary |
|---|---|---|
| Seat Card | Declare capabilities, accepted input types, output types, budget limits, and constraints for routing | P0 required for trusted assignment |
| Seat Skill | Human-authored guidance package that teaches a seat how to work in this environment | P0 attachable and visible in pack preview |
| Execution Template | Inspectable step sequence showing engine steps, LLM steps, and verification points | P1 inspectable; P2 version-compare and fork suggestions |

#### What the user sees

- a global Seat Registry where the same seat can exist once and be reused across projects,
- a Seat Card panel with capabilities, input/output types, budgets, constraints, and attached seat skills,
- a Project Role Drawer showing the active role for the current project,
- authority documents and constraints visible before work is assigned,
- delegation badges such as `Flux acting for Mira on WI-009` without rewriting Flux's underlying identity,
- an Execution Template inspector that reveals how a seat stages deterministic work, LLM work, and verification.

#### User story

As a user, I can move between projects without recreating seats, while still seeing the correct role, authority, skill guidance, budgets, and execution guardrails for each seat in the current project.

#### Interaction contract coverage

- `INT-02`, `INT-03`, `INT-10`, `INT-12`

#### UX surface coverage

- `UX-02`, `UX-04`, `UX-05`, `UX-11`

#### Rules

1. Seat identity remains stable across projects.
2. Project role binding may change per project without mutating the global identity.
3. Delegation is an overlay with scope, issuer, and expiry; it must not rewrite historical ownership.
4. Routing must use Seat Card truth, not only role labels.
5. Seat Skill is authored guidance and must remain distinct from Playbook, which is learned reuse.
6. Collaboration templates are visible in P0, cloneable/editable in P1, and recommendable in P2.

### 6.3 Playbook system

#### User promise

When the team solves a runtime problem, workflow problem, or repeatable design problem once, SeatLoom can reuse that lesson later instead of paying the same token cost again.

#### Playbook vs Seat Skill contract

| Concept | Nature | Author | Trigger | User-visible value |
|---|---|---|---|---|
| Seat Skill | Authored guidance about how a seat should work | Human | Explicit attachment to a seat or runtime context | Makes seat behavior and expectations legible before execution |
| Playbook | Learned reuse extracted from real project execution | Auto or semi-auto from evidence | Context match during continuity, routing, or review | Prevents repeated mistakes and repeated token spend |

#### P0/P1/P2 boundary

| Priority | Contract |
|---|---|
| P0 | Reserve separate seat-skill and playbook attachment points in continuity previews, and explain why either one is included. |
| P1 | Publish playbooks from resolved blockers or successful recipes, store them at project or global scope, match them by runtime/role/context, and show estimated savings and reuse counts. |
| P2 | Rank and recommend playbooks across projects, suggest candidate reuse automatically, and compare ROI over time. |

#### What the user sees

- a `Seat skills` block in continuity previews,
- a `Relevant playbooks` block that is separate from seat guidance,
- a Playbook Library with type, trigger, scope, saved-effort estimate, reuse count, and source evidence,
- publish and accept screens for playbook candidates,
- cross-project visibility for global playbooks.

#### User story

As a user, I can tell the difference between authored guidance for how a seat works and learned reuse from what the team already proved, and I can expect both to reduce context cost in different ways.

#### Interaction contract coverage

- `INT-06`, `INT-09`, `INT-14`, `INT-15`

#### UX surface coverage

- `UX-06`, `UX-08`, `UX-10`

#### Rules

1. A playbook must link back to source evidence: session, work item, artifact, or blocker resolution.
2. Playbooks may be project-local or global.
3. Playbook application is deterministic matching first, optional Supervisor explanation second.
4. Saved-effort reporting is approximate and must be labeled as an estimate.
5. Seat Skill and Playbook must never be merged into a single opaque "guidance" object.

### 6.4 Supervisor Layer

#### User promise

The primary way to operate SeatLoom is by expressing intent, then confirming a structured proposal or evidence-backed answer, not by starting from empty forms.

#### What the Supervisor is

The Supervisor is a built-in orchestration layer, not a user-created seat.

It is responsible for:

- intent parsing,
- proposal generation,
- structured action decomposition,
- seat routing suggestions,
- evidence-backed answer drafting,
- confirmation before state mutation.

#### P0/P1/P2 boundary

| Priority | Contract |
|---|---|
| P0 | Command Bar, suggestion cards, capability-aware creation of WorkItems/Handoffs/comments, Inbox quick-action suggestions, exact-evidence assist, and explicit confirmation before write actions. |
| P1 | Batch proposals, supervisor continuity packs, richer edit-before-confirm flows, review-assist drafting, and why-decision answers grounded in layered retrieval. |
| P2 | Policy-tuned orchestration proposals, multi-step draft plans, and bounded automation proposals requiring approval. |

#### What the user sees

- a prominent Command Bar in the app shell,
- suggestion cards with proposed objects, owners, seat rationale, budgets, and constraints,
- inline edit controls before confirmation,
- exact-evidence and why-decision answer cards with source links,
- quick-action suggestions on Inbox rows and artifact review threads,
- separate Supervisor context and history, not mixed with worker seat identity.

#### User story

As a user, I can say `Create a task for Nimbus to implement OAuth with Google and GitHub` or `Why did we abandon passport.js?` and SeatLoom returns a reviewable, evidence-backed response before anything important changes.

#### Interaction contract coverage

- `INT-01`, `INT-02`, `INT-05`, `INT-08`, `INT-09`, `INT-11`, `INT-13`, `INT-14`, `INT-17`, `INT-18`, `INT-19`, `INT-20`

#### UX surface coverage

- `UX-02`, `UX-03`, `UX-05`, `UX-06`, `UX-07`, `UX-09`, `UX-10`, `UX-13`, `UX-14`, `UX-15`

#### Rules

1. The Supervisor may draft structured actions or answers, but the user confirms all state-changing operations in P0 and P1.
2. Supervisor token budget is separate from worker seat budgets.
3. Supervisor input must be context-minimized, role-appropriate, and retrieval-grounded.
4. Supervisor output should be structured data first and free text second.
5. When the Supervisor answers a why-question, it must show which evidence sources were used.
6. Review follow-up edits must use a three-tier routing policy to optimize token ROI:
   - `L1` (micro, no semantic impact): reviewer may patch directly without dispatching a worker seat.
   - `L2` (clear semantic edit, implementation-relevant): dispatch to worker with compact acknowledgment only.
   - `L3` (scope/priority/gate semantics): dispatch to worker and require full Supervisor gate review.
7. Every review-follow-up edit must persist a structured `change_tier_record` for retrieval and audit with fields:
   `tier`, `reason`, `changed_clauses`, `impact_level`, `executor`, `reviewer`, `ack_mode`, and `evidence_refs`.
8. L2 acknowledgments must stay in compact format (TaskRef, changed files, changed clauses, impact, evidence, status) to prevent review-loop token expansion.

### 6.5 Artifact review system

#### User promise

SeatLoom is not done when an artifact is produced. The user must be able to read it, search it, comment on it, resolve feedback, and link the review back into work execution.

#### P0/P1/P2 boundary

| Priority | Contract |
|---|---|
| P0 | Open full artifact content inside the app, render coordination artifacts by `template+subtype`, expose dual-key metadata in detail, support manual comments, search exact text, filter by artifact family or subtype, show comment counts on sections, allow resolve/reply/dispute, link comments to Inbox and Timeline, and support project-scope or multi-WorkItem artifact association. |
| P1 | Add Supervisor-assisted comment drafting, version history, artifact diff view, and comment-to-work-draft suggestions. |
| P2 | Add review analytics, batch review bundles, and trend views across artifacts and teams. |

#### What the user sees

- a full-content reader in the main panel,
- a human-readable family badge plus subtype chip and metadata strip showing extracted header fields before the document body,
- a family/subtype-aware summary block that changes for task packets, reviews, acceptances, memory logs, and governance docs,
- typed artifact references in Timeline, Inbox, and related Detail sections so linked evidence is shown as an openable object, not a raw path string,
- a review side rail with comment threads,
- inline anchors and section badges showing comment counts,
- artifact-level search with exact matches and artifact family/subtype filters,
- resolve, reply, and dispute controls on each thread,
- linked WorkItems and version history in artifact detail.

#### Coordination artifact type system

SeatLoom uses `template+subtype` as the canonical coordination-artifact classification key, not raw markdown shape alone. `template` selects the broad document family badge and base layout. `subtype` selects the exact summary block, review actions, Route/Gate automation hooks, and precise retrieval filters. The subtype allow-list is owned by `docs/coordination/DOCUMENT_TEMPLATES.md` Section 11.1 and must be enforced to prevent taxonomy drift.

| Template family | Allowed subtypes | User-visible treatment | Automation and retrieval value |
|---|---|---|---|
| `T1` Authority Doc | `prd`, `ux_spec`, `interaction_spec`, `acceptance_spec`, `architecture_design`, `architecture_decisions` | Show `Authority Doc` badge plus subtype chip such as `PRD` or `UX Spec`; emphasize version, status, supersedes, depends_on, and linked companion specs | Lets the user isolate the active contract layer and lets Route/Gate logic distinguish product contracts from architecture evidence |
| `T2` Role Profile | `seat_role` | Show `Role Profile` badge plus `Seat Role` chip; emphasize mission, capability tags, primary inputs/outputs, and collaboration boundaries | Lets routing and seat review depend on explicit seat-role truth rather than prose inference |
| `T3` Task Packet | `task`, `fix`, `integration`, `verification` | Show `Task Packet` badge plus subtype chip; emphasize owner, priority, deadline, done definition, acceptance reference, and delivery state | Lets the user filter executable assignments by packet kind and lets routing/review automation treat verification packets differently from implementation packets |
| `T4` Review | `gap_review`, `benchmark`, `process_mapping`, `design_proposal` | Show `Review` badge plus subtype chip; emphasize reviewer, scope, severity counts, priority fix list, and next steps | Lets the user separate benchmark evidence from gap or process reviews and lets follow-up routing key off review intent |
| `T5` Acceptance | `acceptance_review`, `gate_decision` | Show `Acceptance` badge plus subtype chip; emphasize verdict, target, evidence package, and immutable issuance marker | Lets Gate surfaces distinguish delivery acceptance from formal stage-gate decisions |
| `T6` Daily Memory | `daily_log` | Show `Daily Memory` badge plus `Daily Log` chip; emphasize date, owner, key decisions, blockers, expected handoffs, and append-only state | Lets continuity search isolate operational history deterministically by day |
| `T7` Governance Doc | `coordination_rules`, `workflow_principles`, `collaboration_protocol`, `document_templates` | Show `Governance Doc` badge plus subtype chip; emphasize status, scope, superseded rules, and linked enforcement docs | Lets the user distinguish operating-policy families and lets gate reasoning cite the exact rule source |

The universal header fields from `docs/coordination/DOCUMENT_TEMPLATES.md` become structured artifact metadata in SeatLoom: `template`, `subtype`, `id`, `status`, `author`, `date`, `version`, `depends_on`, `supersedes`, and `tags`. Detail Pane rendering, Route/Gate automation, and retrieval filters must all use this same dual key instead of inferring document meaning from body text alone.

#### User story

As a user, I can open a task packet, review, acceptance note, or governance document inside SeatLoom, see its family badge and subtype-specific metadata first, find the relevant section quickly, comment on it, and see that feedback return to the responsible seat as an actionable, auditable work signal.

#### Interaction contract coverage

- `INT-05`, `INT-07`, `INT-08`, `INT-13`, `INT-14`, `INT-18`, `INT-19`, `INT-20`

#### UX surface coverage

- `UX-05`, `UX-07`, `UX-10`, `UX-14`, `UX-15`

#### Rules

1. Artifact review is part of the main work loop, not an external-editor-only escape hatch.
2. Manual and Supervisor-assisted comments produce the same underlying comment object and audit trail.
3. Comment resolution must be visible in Inbox, Timeline, and artifact detail.
4. Artifact association may be single WorkItem, multi-WorkItem, or project-scope.
5. Exact artifact search must happen without LLM dependency.
6. Coordination artifacts must preserve `template`, `subtype`, and universal header metadata as structured fields even when the body renders as markdown.
7. Detail Pane and Artifact Reader must switch summary blocks, badges, primary actions, and warnings based on the validated `template+subtype` key rather than forcing one generic layout.
8. Artifact search must filter by artifact family and subtype and use `template`, `subtype`, and other header metadata as structured retrieval inputs before full-text fallback.
9. Route and Gate automation may consume artifact subtype only after allow-list validation, so `gate_decision`, `verification`, or `process_mapping` documents never drift into ambiguous behavior.
10. The minimum baseline objectization surface must cover `T1` (`prd`, `ux_spec`, `interaction_spec`), `T3` (`task`, `fix`, `verification`), `T4` (`gap_review`, `benchmark`), `T5` (`acceptance_review`, `gate_decision`), `T6` (`daily_log`), and `T7` governance documents in Timeline, Inbox evidence entry points, and Detail rendering before broader implementation starts.

### 6.6 Module team topology (P2/P3 follow-up)

#### Why this is deferred but important

Complex projects often need parallel module delivery. Two naive extremes both fail in practice:

- putting all modules on one core PO/architect lane (throughput bottleneck),
- cloning many core seats too early (consistency drift + token overhead).

#### Proposed direction

SeatLoom should scale team topology in this order:

1. **Default path**: keep core seats limited and increase parallelism through **module session groups**.
2. **Next step**: add **delegation overlays** (temporary module leads with explicit scope, issuer, expiry, and authority limits).
3. **Later step**: permit selective core-seat replication only when throughput and scope-divergence evidence justify it.

#### P2/P3 contract intent

| Priority | Contract intent |
|---|---|
| P2 | Define and expose Module Session Groups: per-module grouping of sessions, work items, handoffs, artifacts, and progress panels while keeping final gate centralized. |
| P3 | Add Delegation Overlay orchestration: temporary module-lead assignment with scope/expiry/authority, visible in Timeline and Detail, plus metrics to determine whether core-seat replication is warranted. |

#### Value test

- **Pain solved**: multi-module delivery stalls when one central lane owns all active orchestration.
- **If omitted**: teams either stay bottlenecked or prematurely replicate core seats and pay large consistency/token costs.
- **Behavior change**: teams parallelize by module with explicit scoped delegation while preserving one source of gate truth.

## 7. Integrated P0 journey

The baseline user journey for implementation is:

1. The user opens SeatLoom and sees Overview plus Inbox guidance.
2. The user asks the Supervisor to create or route work.
3. Seat identity, Seat Card, project role binding, collaboration mode, and budget limits determine who can receive that work.
4. The Data Engine generates summaries, gates, routes, retrieval hits, and continuity previews deterministically.
5. A worker seat executes in an external runtime or wrapped session.
6. The user reviews the resulting artifact inside SeatLoom, comments if needed, and either accepts or reissues work.
7. If runtime changes or interruptions occur, SeatLoom rebuilds context through structured checkpoints, context tiers, exact evidence retrieval, and deterministic fallbacks.
8. If a wrapped runtime blocks on an interactive prompt, SeatLoom exposes that prompt state and routes it through deterministic policy plus bounded assistance rather than forcing constant human attention or expensive context rereads.

If any step in this journey lacks a story, interaction contract, UX definition, or value-test answer, implementation must stop until the gap is closed.

## 8. Ecosystem alignment decisions

SeatLoom adopts the following ecosystem learnings into the product contract now:

| Source input | Adopted product decision | Scope |
|---|---|---|
| Agent Skills | Use progressive loading logic and a distinct authored `Seat Skill` concept instead of treating all reuse as one blob | P0 |
| MCP | Use capability declaration, pack-only isolation, and layered loading inside the Data Engine | P0 |
| A2A | Use Seat Card structure and explicit Handoff progress states instead of informal seat descriptions | P0/P1 |
| Harness engineering | Treat budgets and lightweight verification as product-enforced controls, not optional discipline | P0 |
| Context/retrieval design | Use structured checkpoints, context tiers, layered retrieval, and delta resume to maximize token ROI | P0/P1 |

SeatLoom recognizes the following ideas but does not place them in current delivery scope yet:

| Deferred idea | Reason for deferral |
|---|---|
| Exposing SeatLoom as a general MCP Server for third-party runtimes | Important architectural option, but it needs a separate value and security decision before it becomes a delivery commitment |
| User-editable forking of seat Execution Templates | Useful later, but the current priority is inspection and trust, not editable behavior branching |
| Always-on Live Timeline mode by default | Live mode is useful, but replay clarity remains primary and noise must stay optional |

## 9. Non-goals

The v0.5 contract still excludes:

- silent autonomous work promotion,
- unrestricted cross-seat data access,
- cloud-first coordination as a requirement,
- replacing IDEs or model-native chat tools,
- generic issue-board project management positioning,
- unrestricted agent-to-agent orchestration without human approval,
- shipping ecosystem-facing protocol exposure before core user value is contract-clear.

## 10. Implementation entry checklist

Implementation may begin only when all of the following are true:

- `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, and `docs/ux-spec-v1.1.md` have no unresolved P0 conflicts,
- every P0 feature is mapped to at least one user story, one interaction flow, and one UX surface,
- every P1 and P2 feature is explicitly scoped and visible in story form,
- every in-scope feature explicitly states the user pain solved, the consequence of omission, and the user-behavior change after delivery,
- Aegis stage-gate review marks the contract set as `GO`.
