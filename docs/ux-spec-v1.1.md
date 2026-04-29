# SeatLoom UX Specification v1.1

| Field | Value |
|---|---|
| Document | UX Specification v1.1 |
| Status | Draft - Pending review |
| Updated | 2026-04-28 |
| Language | English |
| Depends on | `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md` |

## 0. Purpose

This document defines what the user sees on screen for the v0.5 contract.

The design target is not a generic dashboard. The interface must make AI collaboration feel governed, legible, and actionable:

- intent first, not form first,
- deterministic truth before decorative AI text,
- high information density without mixed-script readability problems,
- visible roles, budgets, evidence, and next actions on one screen.

## 1. Design principles

### 1.1 Reading comfort for mixed English/CJK content

SeatLoom must assume mixed scripts in real project data even if persistent design docs are authored in English.

Recommended stacks:

- UI sans: `"SF Pro Text", "PingFang SC", "Noto Sans SC", sans-serif`
- UI mono: `"SF Mono", "JetBrains Mono", "Noto Sans Mono CJK SC", monospace`

Rules:

- body text: 13px / 20px minimum,
- dense table text: 12px / 18px minimum,
- avoid combining Latin monospace and CJK proportional text in the same line unless separated by labels or chips,
- status, ID, and role chips should use consistent height to prevent script imbalance.

### 1.2 Visual direction

- Light-first neutral workspace with blue-green operational accents.
- Strong status semantics: green running, amber needs attention, red failed, slate completed, teal review, violet drift.
- Calm backgrounds with clear panel hierarchy instead of notification-heavy chrome.

### 1.3 Information hierarchy

The shell must answer four questions at all times:

1. What needs action now?
2. Who owns what in this project?
3. What evidence supports the current state?
4. How much context or budget is being spent?

## 2. App shell

### UX-01 Shell layout

```
+-----------------------------------------------------------------------------------+
| Project Switcher | Supervisor Command Bar                          | Health/Seat |
+------------------+-------------------------------------------------+-------------+
| Sidebar          | Main                                             | Detail      |
| 240px            | flexible                                         | 380px       |
|                  |                                                  |             |
+-----------------------------------------------------------------------------------+
| Terminal / Session Panel (collapsed by default, 300px when open)                 |
+-----------------------------------------------------------------------------------+
| Status Bar: project path | reconcile status | template | runtime health           |
+-----------------------------------------------------------------------------------+
```

Required shell elements:

- project switcher always visible in the top left,
- Supervisor Command Bar centered and always reachable,
- health area showing reconcile freshness, current collaboration template, and runtime or budget warnings,
- default Detail state is `Project Overview`, not a blank panel.

### UX-12 Interactive prompt banner and assist controls (wrapped sessions)

Purpose:

- Prevent stalled CLI prompts from silently blocking work when the human is not watching the terminal output.
- Avoid token waste: do not require rereading long terminal logs or thinking streams to detect or resolve prompts.

Surface location:

- Session Panel (terminal area) shows a top banner when a wrapped session is blocked on input.
- Session row in Sidebar/Session list shows a small `Prompt` chip when blocked.

Banner content (P0):

| Element | Content |
|---|---|
| Status | `Prompt blocked` with runtime and session id |
| Classification chip | `deterministic` / `wizard/menu` / `freeform` / `sensitive` |
| Policy chip | `auto allowed` / `needs approval` / `human required` |
| Preview | last 10-20 lines of the prompt window (bounded) |
| Actions | `Approve`, `Human takeover`, `Supervisor assist` (if allowed), `Stop` |

Rules:

- `Supervisor assist` is disabled for `sensitive` prompts.
- `Supervisor assist` must show a confirmation card before any injected input beyond trivial deterministic choices.
- The prompt preview must be bounded and never auto-expand into full terminal history.
- When `Supervisor assist` is used, the UI must show step count and token budget for the assist loop.
- All prompt resolution outcomes must be visible in Timeline as an auditable event.

### Project Overview default content

When nothing is selected, Detail shows:

- current project name and path,
- active collaboration template,
- running and suspended sessions,
- open WorkItems,
- pending Inbox count,
- last reconcile time,
- top unresolved review thread count,
- current budget alerts.

### UX-13 Mobile Overview and Inbox companion

Purpose:

- Keep project supervision alive while the user is away from the desktop.
- Turn mobile into a governed monitor-and-response channel, not a miniature IDE.

Positioning:

- Mobile support is required in the active contract.
- Mobile is for monitoring, approval/takeover, and short feedback return loops.
- Mobile is not for full terminal work, long-form document editing, or broad task authoring from scratch.

Information architecture:

```
+---------------------------------------------+
| Project | Sync freshness | Alert count      |
+---------------------------------------------+
| Overview                                  |
| - health cards                            |
| - blockers / gates / prompts waiting      |
| - sessions / handoffs / budget warnings   |
+---------------------------------------------+
| Mobile Inbox                              |
| - high-priority action rows               |
| - prompt / gate / handoff interrupts      |
+---------------------------------------------+
| Recent activity / quick-view entry        |
+---------------------------------------------+
| Tabs: Overview | Inbox | Activity          |
+---------------------------------------------+
```

Required Mobile Overview content:

- project name and sync freshness,
- unresolved blocker count,
- pending gate / approval count,
- `Prompt blocked` count,
- pending handoff count,
- over-budget or stalled-session warnings,
- one recommended next action.

Required Mobile Inbox row content:

- priority chip,
- action type,
- owner seat,
- object ref,
- one-line summary,
- waiting-time indicator,
- evidence quick-view entry,
- primary action CTA.

Behavior rules:

- default filter is `high priority / action needed`, not full-history browsing,
- sync freshness is always visible,
- cached snapshots must show a read-only banner before any action is allowed,
- mobile cards may drill into Action Card or Quick View, but may not expand into multi-pane desktop layouts.

Non-goals:

- no full terminal transcript browsing beyond bounded previews,
- no heavy markdown editing or full review-rail interaction,
- no expectation that the user manages the whole project from a phone.

### UX-14 Mobile Action Card and Feedback Composer

#### Mobile Action Card

| Element | Content |
|---|---|
| Object | WorkItem / Handoff / Prompt / Gate ref |
| Ownership | owner seat, project role, waiting time |
| Summary | one to two lines explaining what is blocked or awaiting decision |
| Evidence | quick-view link for the most relevant event or artifact |
| State | current gate, handoff, or prompt status plus policy/risk chips when relevant |
| Actions | `Approve`, `Reject`, `Escalate`, `Reserve desktop takeover`, `Open on desktop` (subset varies by object type) |

Behavior rules:

- prompt cards must show classification and policy chips before the CTA row,
- decision notes are optional but bounded,
- after an action succeeds, the card must show a short receipt with actor, time, and updated status.

#### Mobile Feedback Composer

Fields:

- target object ref,
- feedback mode chips: `Needs change`, `Question`, `FYI`,
- bounded text input,
- optional link to the latest event or artifact excerpt,
- actions: `Send feedback`, `Save draft`, `Cancel`.

Behavior rules:

- sending feedback creates the same underlying comment or feedback object family used by desktop work and review surfaces,
- feedback must route back into Inbox and Timeline rather than staying as a device-local note,
- P0 mobile feedback does not include file upload, rich diff editing, or long threaded review.

### UX-15 Mobile Event and Artifact Quick View

Purpose:

- Let the user inspect enough evidence to approve, reject, escalate, or comment without pretending mobile is a full artifact-review workspace.

Surface:

- bottom sheet or push page opened from Mobile Overview, Mobile Inbox, or Mobile Action Card.

For event quick view, show:

- timestamp,
- actor,
- object ref,
- one-paragraph summary,
- linked session / WorkItem / Handoff references.

For artifact quick view, show:

- artifact title or path,
- family badge plus subtype chip,
- status / author / date / version chips when present,
- bounded excerpt or first relevant section,
- `Open on desktop` action.

Behavior rules:

- quick view is read-only by default,
- typed artifact metadata must stay visible above the excerpt,
- feedback from quick view launches Mobile Feedback Composer instead of opening the full desktop review rail.

## 3. Supervisor surfaces

### UX-02 Supervisor Command Bar and suggestion card

Command Bar behavior:

- width: 640px desktop target,
- placeholder example: `Ask SeatLoom to create, route, review, recover, or explain work...`,
- recent commands open in a dropdown on focus,
- supports object mentions such as WorkItems, seats, and artifacts,
- supports an evidence-search mode and a why-decision mode.

Suggestion card layout:

| Area | Content |
|---|---|
| Intent | user request restated in one line |
| Proposed objects | WorkItem, Handoff, comment, playbook, delegation, or template change |
| Ownership | target seat, capability match, role constraints, authority docs |
| Evidence | linked artifacts, sessions, comments, or playbooks |
| Budget | estimated pack size or session impact |
| Impact | what will change after confirm |
| Actions | `Confirm`, `Edit`, `Cancel` |

Edit mode must be inline, not a separate blank form.

### Why-answer card

When the user asks a why-question, the Command Bar returns a card with:

- one concise answer block,
- source evidence chips,
- confidence note,
- `Open sources`, `Pin to continuity`, and `Copy answer` actions.

### UX-03 Morning Digest and next-action area

Morning Digest sits at the top of the Inbox or Overview and includes:

- interruptions since last session,
- new comments or returned handoffs,
- drifted or blocked WorkItems,
- latest gate decision,
- one recommended next action,
- budget or health warnings that affect the next action.

Each digest item links to the underlying object.
When a digest or Inbox item depends on document evidence, the row must show typed artifact chips with family/subtype labels rather than a bare path string.

## 4. Seat surfaces

### UX-04 Seat Registry, Seat Card, Role Drawer, and template sheet

#### Global Seat Registry

Columns:

- seat name,
- default runtime,
- capability tags,
- accepted input types,
- preferred budget,
- projects using this seat.

Primary actions:

- `Create identity`,
- `Bind to project`,
- `Archive`,
- `Open seat card`,
- `Open project roles`.

#### Seat Card panel

Fields:

- seat name,
- default runtime,
- capabilities,
- accepted input types,
- output types,
- input budget,
- output budget,
- constraints,
- attached seat skills,
- project usage summary.

The panel must make it obvious whether the seat is assignable, under-specified, or budget-constrained.

#### Project Role Drawer

Fields:

- current role,
- authority docs,
- constraints,
- current runtime preference for this project,
- active delegation badge if any,
- active collaboration template summary.

Delegation badge format:

- `Acting for Mira on WI-009 until closed`
- includes issuer and scope on hover or in expanded detail.

#### Execution Template inspector (P1)

Read-only fields:

- template name,
- ordered steps,
- step type (`engine` or `llm`),
- per-step budget where applicable,
- verification point labels,
- last-updated marker.

#### Collaboration Template sheet

Must show before confirm:

- template name,
- routing summary,
- prohibited paths,
- escalation path,
- affected seats.

## 5. WorkItems and Handoffs

### UX-05 Work loop surfaces

#### Sidebar rows

Seat row must show:

- name,
- project role badge,
- status dot,
- runtime chip,
- recent activity line,
- delegation marker if active.

WorkItem row must show:

- ID,
- title,
- priority chip,
- status,
- owner,
- gate badge when blocked or in review.

Handoff row or detail entry must show:

- sender and receiver,
- state strip,
- linked WorkItem,
- latest progress time,
- live indicator when active.

Inbox row must show:

- priority chip,
- action type,
- actor,
- object ref,
- summary,
- linked typed artifact chips when document evidence exists.

#### WorkItem detail

Sections:

1. identity: title, ID, owner, priority, state,
2. goal and acceptance criteria,
3. gate status strip with pass/fail requirements,
4. linked artifacts, comments, handoffs, and sessions,
5. actions: `Assign`, `Create handoff`, `Return`, `Reject delivery`, `Reissue`, `Switch runtime`.

If work is reissued after failed review, the detail header shows a banner:

- `Returned with new scope`,
- linked verdict reason,
- linked evidence,
- current assignee or pending assignee.

When the reissue path is tiered, the same header area must also show a `Review change` record strip with:

- tier badge (`L1`, `L2`, `L3`),
- execution-mode chip (`Direct patch`, `Compact ack`, `Full gate`),
- changed-clause links,
- impact-level chip,
- reviewer and executor labels,
- linked evidence refs.

Behavior rules:

- `L1` keeps the strip compact and marks the reviewer as executor,
- `L2` must expose a six-line compact-ack preview only (`TaskRef`, changed files, changed clauses, impact, evidence, status),
- `L3` must keep a visible `Full gate review required` badge until the follow-up delivery is reviewed again.

#### Handoff detail

Must show:

- sender,
- receiver,
- purpose,
- expected outcome,
- state strip (`sent`, `accepted`, `working`, `completed`, `returned`),
- latest artifact or checkpoint,
- `Open live activity` action when available.

## 6. Session and continuity surfaces

### UX-06 Session detail, tiered pack preview, and continuity surfaces

Session detail must show:

- runtime,
- seat,
- start/end state,
- linked WorkItem,
- interruption or suspension status,
- latest artifact,
- budget meter,
- checkpoint freshness,
- actions: `Open terminal`, `Switch runtime`, `Recover session`, `Suspend session`, `Resume session`, `View pack`.

Continuity pack preview layout:

| Block | Content |
|---|---|
| Tier 0 identity | seat, runtime, WorkItem, branch |
| Tier 1 state | AC progress, latest commit, current blocker |
| Tier 2 decisions | key decisions and reasons |
| Seat skills | authored guidance attached to the seat |
| Relevant playbooks | learned reuse matches |
| Budget | estimated injected tokens and remaining allowance |
| Fallback | exact file path and short instruction |

Behavior rules:

- Tier 0-2 are visible by default,
- deeper evidence is hidden behind `Load more evidence`,
- budget overflow must mark the specific tier or evidence block causing it,
- seat skills and playbooks must be displayed separately.

P1 continuity extension adds:

- suspended/resumed state banner,
- delta since last checkpoint,
- unresolved blockers,
- pending handoffs,
- last gate decisions,
- Supervisor-specific continuity preview.

## 7. Artifact review workspace

### UX-07 Artifact reader and review rail

The artifact workspace takes over Main while keeping Detail available.

Reader layout:

```
+-----------------------------------------------------------------------------------+
| Back | Artifact title / path                             | Open in editor | Copy path |
+-----------------------------------------------------------------------------------+
| Markdown / rich content reader                    | Review rail                   |
| section badges with comment counts                | thread list                   |
| inline anchors for selected text                  | active / resolved / disputed  |
+-----------------------------------------------------------------------------------+
| Find in artifact | Add comment | Request change | Ask Supervisor to comment | View versions |
+-----------------------------------------------------------------------------------+
```

Typed document header requirements:

- show a human-readable artifact family badge (`Authority Doc`, `Role Profile`, `Task Packet`, `Review`, `Acceptance`, `Daily Memory`, `Governance Doc`, or generic `Artifact`) plus a subtype chip such as `PRD`, `Verification`, or `Gate Decision`,
- show metadata chips for `id`, `status`, `author`, `date`, and `version` when present,
- show `depends_on` and `supersedes` links as clickable references when present,
- keep the metadata strip visible before the user scrolls into raw markdown content.

Type-specific summary block:

| Template family + subtype | Required summary content in Detail / Reader |
|---|---|
| Authority Doc (`prd`, `ux_spec`, `interaction_spec`, `acceptance_spec`, `architecture_design`, `architecture_decisions`) | version, status, supersedes, depends_on, linked companion specs |
| Role Profile (`seat_role`) | mission, capability tags, primary inputs/outputs, collaboration boundaries |
| Task Packet (`task`, `fix`, `integration`, `verification`) | owner, priority, deadline, done definition, acceptance reference, delivery state |
| Review (`gap_review`, `benchmark`, `process_mapping`, `design_proposal`) | reviewer, scope, severity counts, priority fix list, next steps |
| Acceptance (`acceptance_review`, `gate_decision`) | verdict, target, evidence package, immutable issue marker |
| Daily Memory (`daily_log`) | date, owner, key decisions, blockers, expected handoffs, append-only notice |
| Governance Doc (`coordination_rules`, `workflow_principles`, `collaboration_protocol`, `document_templates`) | status, scope, superseded rules, linked enforcement docs |

Comment thread card fields:

- author,
- anchor reference,
- mode (`manual` or `supervisor-assisted`),
- content,
- reply count,
- state,
- actions: `Reply`, `Resolve`, `Dispute`.

Review-change record card (shown when a thread turns into execution follow-up):

- tier badge (`L1`, `L2`, `L3`),
- execution-mode chip (`Direct patch`, `Compact ack`, `Full gate`),
- changed-clause list with clickable section refs,
- impact-level chip,
- reviewer and executor identity,
- evidence refs,
- acknowledgment preview or gate status.

Behavior rules:

- `Request change` opens a structured review-change composer instead of a freeform text field,
- `L2` uses a visually bounded compact-ack block in the rail and must not expand into a long conversational thread,
- `L3` keeps the gate badge and pending-review state visible from the reader header and the linked WorkItem detail.

Artifact detail must support:

- single WorkItem association,
- multi-WorkItem association,
- project-scope association,
- typed metadata header plus family/subtype-specific summary block for coordination artifacts,
- the same typed artifact entry treatment when opened from Timeline, Inbox, WorkItem, Session, or Handoff surfaces,
- exact search navigation,
- artifact family/subtype chips that stay visible in search-opened context,
- version history in P1.

Baseline prototype coverage must visibly support at least:

- Authority Doc: `prd`, `ux_spec`, `interaction_spec`,
- Task Packet: `task`, `fix`, `verification`,
- Review: `gap_review`, `benchmark`,
- Acceptance: `acceptance_review`, `gate_decision`,
- Daily Memory: `daily_log`,
- Governance Doc: `coordination_rules`, `workflow_principles`, `collaboration_protocol`, `document_templates`.

## 8. Playbook Library and learned-reuse surfaces

### UX-08 Playbook surfaces

Playbook list columns:

- title,
- type (`runtime workaround`, `workflow recipe`, `design pattern`),
- trigger conditions,
- scope (`project` or `global`),
- saved-effort estimate,
- reuse count,
- source evidence.

Playbook candidate card must show:

- what was learned,
- when it should trigger,
- which evidence created the lesson,
- where it will be stored,
- estimated savings.

A matching playbook inside continuity preview uses a compact card with:

- title,
- why it matched,
- `Open playbook`,
- `Do not include this time`.

A seat-skill block must not reuse the same visual treatment as playbooks. The user must be able to tell `authored guidance` from `learned reuse` immediately.

## 9. Proposal surfaces for P2

### UX-09 Proposals tray

This tray is not a chat log. It is a queue of structured draft actions awaiting approval.

Proposal card fields:

- source seat or Supervisor,
- affected objects,
- evidence links,
- expected impact,
- confidence or repeat-pattern note,
- actions: `Accept`, `Edit`, `Reject`.

Approved proposals must look identical to normal confirmed actions after promotion.

## 10. Evidence search and decision recall

### UX-10 Evidence search and recall surfaces

Evidence search panel must support:

- query input,
- filters for object type, artifact family, artifact subtype, document status, date range, seat, and WorkItem,
- grouped results: `Objects`, `Checkpoints`, `Artifacts`, `Text matches`,
- artifact result cards showing family badge, subtype chip, status, author/date, and tags when present,
- `Load related evidence` action,
- empty-state guidance.

Why-decision recall card must support:

- concise explanation,
- source evidence chips,
- confidence note,
- `Open sources`, `Pin to continuity`, and `Copy answer` actions.

Design rule:

- exact results must appear before semantic interpretation,
- before the full evidence-search panel is opened, at least one primary browsing surface must expose lightweight `artifact family` and `artifact subtype` filters so the user can narrow evidence from the main workflow,
- typed artifact metadata must remain visible on result cards so the user can distinguish task packets from reviews, acceptances, memory logs, and governance docs before opening them, and so they can tell a `verification` packet from a `fix` packet or a `gate_decision` from an `acceptance_review`,
- if semantic recall adds context in P1, it must be visually marked as an explanation layer rather than the raw truth source.

## 11. Live progress and execution transparency

### UX-11 Live progress, budgets, and execution transparency

Timeline live mode:

- off by default,
- entered through a clear `Live activity` toggle,
- visually distinct from replay history,
- can be filtered by seat, session, or WorkItem.

Live activity event card must show:

- timestamp,
- source seat,
- linked session or handoff,
- event summary,
- optional latest artifact or checkpoint reference.

Budget surfaces:

- session budget meter,
- launch budget estimate,
- warning state when over 80 percent,
- hard-stop state when exhausted.

Execution transparency surfaces:

- read-only Execution Template inspector,
- engine vs LLM step chips,
- per-step verification labels,
- last-used version marker.

ROI comparison surface (P2):

- seat, playbook, runtime, and date filters,
- saved-token counters,
- reuse counts,
- reissue-rate comparisons,
- drill-down link to source evidence.

## 11.5 Module topology follow-up

### UX-P3-01 Module session groups and delegation topology view

Purpose:

- Increase parallel throughput across complex modules before duplicating core PO or architect seats.
- Make delegation visible and bounded so module leads can coordinate local work without fragmenting final product truth.

Primary entry points:

- `Module topology` from Project Overview,
- Supervisor command result such as `Organize by module`,
- queue-pressure suggestion card when central-lane backlog crosses policy thresholds.

Layout:

```
+-----------------------------------------------------------------------------------+
| Core lane summary | active modules | delegated leads | replication pressure       |
+-----------------------------------------------------------------------------------+
| Module group list / board                  | Module detail / delegation / metrics |
| module name                                | scope summary                         |
| lead + gate owner                          | linked sessions / WorkItems           |
| sessions / work / blockers / last update   | handoffs / artifacts / blockers       |
| status                                     | delegation overlay + justification    |
+-----------------------------------------------------------------------------------+
```

Required module-group card fields:

- module name,
- current objective,
- temporary module lead if any,
- final gate owner,
- active session count,
- open WorkItem count,
- blocker count,
- last update time.

Required delegation overlay fields:

- delegate seat,
- issuer,
- scope,
- authority limits,
- expiry,
- current status.

Required replication-justification metrics:

- concurrent active module count,
- queue age on the core lane,
- cross-module reissue or conflict count,
- token-overhead estimate for staying centralized,
- recommendation state: `stay grouped`, `review replication`, or `replication not justified`.

Behavior rules:

- final gate owner must remain visible on every module group card and in the detail header;
- delegation overlay must read as temporary and scoped, not as seat replacement;
- replication review is a secondary evidence-driven action, not the default primary CTA;
- module grouping must extend existing WorkItem, Session, Handoff, Artifact, Timeline, and Detail surfaces instead of creating a disconnected shadow workflow.

## 12. Empty, loading, and failure states

- No matching playbooks: show `No reusable playbooks match this context yet`.
- No comments on artifact: show `No review comments yet` plus `Add comment`.
- Missing collaboration template: show `Choose a coordination model before assigning work`.
- Role conflict: show exact seat and rule causing the conflict.
- Pack generation failure: keep the preview shell open and replace content with reason, fallback path, and retry.
- Search index unavailable: show retry and source-folder fallback.
- Budget exhausted: keep object context visible and show the next safe actions.

## 13. Component inventory

| Component | Purpose |
|---|---|
| `ProjectSwitcher` | Switch projects and restore project workspace state |
| `SupervisorCommandBar` | Accept natural-language intents and evidence questions |
| `SuggestionCard` | Confirm or edit structured proposals |
| `MorningDigestCard` | Summarize changes since last active session |
| `SeatRegistryTable` | Manage global seat identities |
| `SeatCardPanel` | Show capability truth, budgets, and seat skills |
| `ProjectRoleDrawer` | View and edit project-specific role bindings |
| `ExecutionTemplateInspector` | Reveal step-level seat behavior and verification |
| `CollaborationTemplateSheet` | Select and preview collaboration modes |
| `GateStatusStrip` | Show why an object can or cannot move |
| `BudgetMeter` | Show estimated and live token usage |
| `ContinuityPackPreview` | Preview tiered runtime-switch and recovery context |
| `ArtifactReader` | Read full artifact content in app |
| `ReviewRail` | Show and manage review threads |
| `ReviewChangeCard` | Show `L1/L2/L3` routing, evidence, and compact/full follow-up state |
| `PlaybookLibrary` | Browse learned reusable lessons |
| `EvidenceSearchPanel` | Search structured objects and text evidence |
| `WhyAnswerCard` | Present evidence-backed decision explanations |
| `LiveActivityRail` | Show optional real-time progress events |
| `ProposalTray` | Review bounded automation proposals |
| `MobileOverview` | Show mobile-safe project health, blockers, and urgent next actions |
| `MobileInbox` | Queue mobile approvals, interrupts, and short feedback targets |
| `MobileActionCard` | Let the user approve, reject, escalate, or reserve desktop takeover |
| `MobileFeedbackComposer` | Return short structured feedback into the canonical work loop |
| `MobileQuickView` | Show read-only event and artifact evidence with typed metadata |
| `RoiComparisonView` | Compare cost, reuse, and routing outcomes |
| `ModuleTopologyView` | Organize module session groups, delegation overlays, and replication pressure |
| `DelegationOverlayCard` | Show scoped temporary module-lead authority and expiry |
| `ReplicationJustificationPanel` | Show the evidence used before any core-seat replication review |

## 14. UX acceptance notes

- P0 screens must work without relying on modal-heavy blank-form creation.
- P0 continuity surfaces must make context tiers and budget impact legible before launch.
- P0 evidence search must return exact results without LLM dependency.
- P1 additions must attach to existing P0 surfaces rather than inventing parallel workflows.
- P2 surfaces must preserve human approval, visible evidence, and auditability.
- Mobile surfaces must stay bounded to monitoring, approval/takeover, short feedback, and read-only quick view; full terminal development and heavy editing stay desktop-only.
- Mobile approvals, feedback, and interrupt actions must visibly map back into the same canonical project objects and audit trail used on desktop.
- Review follow-up surfaces must keep the `L1/L2/L3` tier, changed clauses, impact, and evidence legible, and `L2` acknowledgments must remain visibly compact.
- P3 topology surfaces must preserve centralized gate truth and make replication a reviewed evidence-based step.
