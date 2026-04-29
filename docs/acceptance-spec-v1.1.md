# SeatLoom Acceptance Spec v1.1

| Field | Value |
|---|---|
| Product | SeatLoom |
| Document | Acceptance Spec v1.1 |
| Status | Draft - v0.5 aligned acceptance contract |
| Updated | 2026-04-28 |
| Language | English |
| Canonical entrypoint | `docs/PRODUCT_TRUTH.md` |
| Depends on | `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/architecture-decisions.md`, `docs/architecture-design.md` |
| Replaces | `docs/archive/product-history/acceptance-spec-v1.0.md` |

## 0. Purpose

This document defines the pass/fail contract for the active v0.5 product-design set.

It does not redefine product meaning. It verifies that the shipped behavior matches:

1. the value and scope contract in `docs/prd-v0.5.md`,
2. the flow contract in `docs/interaction-spec-v1.1.md`,
3. the screen contract in `docs/ux-spec-v1.1.md`,
4. the implementation constraints in `docs/architecture-decisions.md` and `docs/architecture-design.md`.

## 1. Acceptance principles

### 1.1 Precedence

| Topic | Acceptance source |
|---|---|
| Why the feature exists, what priority it belongs to, and what user pain it solves | `docs/prd-v0.5.md` |
| Trigger, confirm/edit/cancel behavior, success path, and failure path | `docs/interaction-spec-v1.1.md` |
| What is visible on screen and where the user performs the action | `docs/ux-spec-v1.1.md` |
| Data-model, runtime, retrieval, and persistence constraints | `docs/architecture-decisions.md`, `docs/architecture-design.md` |

Acceptance may reject a delivery that violates those contracts, but it may not invent a new product rule on its own.

### 1.2 Entry rule for implementation

No feature may enter implementation unless all of the following are true:

- it exists in the active contract set reachable from `docs/PRODUCT_TRUTH.md`,
- it has a user story with pain / omission / behavior-change framing,
- it has at least one explicit interaction flow,
- it has at least one explicit UX surface,
- its priority boundary and non-goals are visible.

### 1.3 Deterministic-first rule

The product fails acceptance if deterministic product truth is replaced by an LLM-only answer in any P0 baseline flow where structured or indexed data already exists.

## 2. Gate model

### 2.1 Gate A - Contract Integrity Gate

This gate must pass before implementation or acceptance work starts.

| Check | Pass condition |
|---|---|
| Active entrypoint | Work starts from `docs/PRODUCT_TRUTH.md` |
| P0 alignment | Every P0 story in `docs/prd-v0.5.md` maps to at least one `INT-*` flow and one `UX-*` surface |
| Conflict status | No unresolved P0 contradiction across PRD, Interaction Spec, UX Spec, and architecture constraints |
| Value framing | Every claimed feature answers pain / omission / behavior change |
| Acceptance authority | Verification packet cites this spec, not superseded acceptance docs |

### 2.2 Gate B - P0 Delivery Readiness Gate

This gate determines whether a P0 delivery is acceptable for implementation handoff, demo, or stage review.

P0 is accepted only if the claimed flow works end-to-end and the user can complete the primary action inside SeatLoom without reconstructing the product contract manually.

### 2.3 Gate C - Claimed-Scope Gate for P1/P2

P1 or P2 work may be claimed only when:

- the feature extends an existing P0 surface rather than inventing a disconnected workflow,
- the delivery keeps human approval and auditability where required,
- the evidence package clearly labels the scope as `claimed P1` or `claimed P2`,
- missing optional work outside the claimed scope is not represented as passed.

## 3. Coverage matrix

### 3.1 P0 contract coverage

| Story ID | Required user outcome | Required interaction refs | Required UX refs | Pass condition |
|---|---|---|---|---|
| `US-P0-01` | User starts from Overview / Morning Digest / Inbox and sees ranked next actions | `INT-01`, `INT-04` | `UX-01`, `UX-03` | Inbox is an action queue, digest links resolve to objects, and the next action is visible without terminal polling |
| `US-P0-02` | User creates and routes work from natural language with structured confirm/edit/cancel | `INT-02` | `UX-02`, `UX-05` | Supervisor proposal shows owner, evidence, AC, impact, and budget before commit |
| `US-P0-03` | User reuses a seat identity with project-specific role and authority visibility | `INT-03` | `UX-04` | Seat identity, project role, authority docs, and constraints are all visible before assignment |
| `US-P0-04` | User can issue scoped delegation without mutating original seat identity | `INT-03`, `INT-04` | `UX-04`, `UX-05`, `UX-11` | Delegation is visible in seat detail and timeline with scope and issuer |
| `US-P0-05` | User rejects or returns delivery with evidence-linked reissue path | `INT-05` | `UX-05`, `UX-07` | Failure reason, evidence, reissue target, review tier (`L1/L2/L3`), and structured `change_tier_record` are visible before reissue confirmation |
| `US-P0-06` | User switches runtime or recovers continuity through a tiered preview and fallback ladder | `INT-06` | `UX-06` | Tier 0-2, budget impact, seat skills, playbooks, and fallback instructions are visible before launch |
| `US-P0-07` | User reviews full artifact content and routes comments inside SeatLoom | `INT-07` | `UX-07` | Artifact reader shows family badge, subtype chip, metadata strip, subtype-specific summary, review rail, comment actions, thread state in-app, and a structured change-request path when review must turn into execution follow-up |
| `US-P0-08` | User checks capability truth before routing work | `INT-02`, `INT-03` | `UX-02`, `UX-04`, `UX-05` | Seat Card exposes capabilities, constraints, accepted inputs, and budget limits |
| `US-P0-09` | User sees context tiers, skill/playbook matches, and budget impact before continuity launch | `INT-06` | `UX-06` | Continuity preview is tiered and budget overflow identifies the specific cause |
| `US-P0-10` | User finds exact evidence deterministically before asking for explanation | `INT-13` | `UX-07`, `UX-10` | Structured and full-text hits appear without LLM dependency, support artifact family/subtype filtering, and open source evidence directly |
| `US-P0-11` | User can clear or route interactive prompts without rereading raw terminal noise | `INT-16` | `UX-12` | Prompt state shows classification, policy, bounded preview, allowed actions, and an audit-visible receipt after resolution |
| `US-P0-12` | User can monitor project health and urgent work from mobile | `INT-17` | `UX-13` | Mobile Overview and Mobile Inbox show sync freshness, urgent counts, and a recommended next action from canonical project state |
| `US-P0-13` | User can approve, reject, or escalate urgent decisions from mobile | `INT-18` | `UX-14`, `UX-15` | Mobile Action Card shows the pending object, governing state, bounded evidence, allowed CTAs, and the resulting audit-visible state change |
| `US-P0-14` | User can send bounded feedback from mobile back into the canonical work loop | `INT-19` | `UX-14`, `UX-15` | Mobile feedback creates a linked canonical feedback/comment object and projects it back into Inbox and Timeline |
| `US-P0-15` | User can triage `Prompt blocked`, `Gate needed`, or `Handoff pending` from mobile | `INT-20` | `UX-13`, `UX-14`, `UX-15` | Mobile interrupt cards expose only policy-allowed actions, including `Reserve desktop takeover`, and keep desktop/mobile state in sync |

### 3.2 Claimed P1 and P2 extension checks

| Story ID | Priority | Minimum pass condition |
|---|---|---|
| `US-P1-01` | P1 | Playbook publish/apply flow is visible and kept distinct from seat skills |
| `US-P1-02` | P1 | Collaboration template clone shows source, conflicts, and confirm path |
| `US-P1-03` | P1 | Supervisor-drafted comments remain editable before confirmation |
| `US-P1-04` | P1 | Supervisor continuity preview shows role-appropriate delta context |
| `US-P1-05` | P1 | Suspend/resume flow proves delta-only context behavior or clearly labels fallback |
| `US-P1-06` | P1 | `working` handoff visibility and optional live mode do not replace replay truth |
| `US-P1-07` | P1 | Execution Template inspector reveals engine vs LLM steps and verification points |
| `US-P1-08` | P1 | Why-answer output is evidence-backed and visually marked as explanation |
| `US-P2-01` | P2 | Draft proposals require explicit human approval before state mutation |
| `US-P2-02` | P2 | ROI comparison view is evidence-drillable and does not invent unsupported metrics |

## 4. Product acceptance checklist

| ID | Requirement | Pass condition |
|---|---|---|
| `P-01` | Start-day orientation | Project Overview, Morning Digest, Inbox, and budget/health signals appear on launch or reconcile |
| `P-02` | Inbox as action queue | Inbox contains action-requiring items only and rows update or disappear after action |
| `P-03` | Supervisor-driven work creation | User can create and route work via intent, review a structured proposal, and confirm without a blank-form dependency |
| `P-04` | Seat capability truth | Seat Card and role binding make routing constraints visible before assignment |
| `P-05` | Review failure loop | Reject / return / reissue path requires reason plus linked evidence |
| `P-06` | Review change tier routing | Review-driven follow-up exposes `L1/L2/L3`, persists the required `change_tier_record`, enforces compact `L2` acknowledgment, and keeps `L3` under full gate review until closure |
| `P-07` | Continuity and runtime switch | Runtime switch or recovery exposes tiered pack preview, budgets, and deterministic fallback |
| `P-08` | In-app artifact review | User can read artifacts, comment, resolve, and route review feedback inside SeatLoom |
| `P-09` | Exact evidence search | Search returns deterministic grouped results before any semantic explanation |
| `P-10` | Audit visibility | Accepted state changes are discoverable from Timeline and object Detail |
| `P-11` | Dual-key typed coordination artifacts | Detail and Artifact Reader render Authority Docs, Role Profiles, Task Packets, Reviews, Acceptances, Daily Memory logs, and Governance Docs using validated `template+subtype` metadata, family/subtype badges, and metadata-aware summaries instead of generic markdown-only treatment |
| `P-12` | Interactive prompt handling | Prompt-blocked state, classification, policy chips, bounded preview, and `Approve` / `Human takeover` / `Supervisor assist` / `Stop` actions are visible without requiring a second LLM pass over long logs |
| `P-13` | Mobile monitoring loop | Mobile companion shows health, approvals, blockers, and sync freshness in a bounded monitor-first surface rather than a desktop clone |
| `P-14` | Mobile decision loop | Mobile approvals, rejects, escalations, and reserve-desktop actions mutate the same canonical objects and audit history used on desktop |
| `P-15` | Mobile feedback return loop | Short mobile feedback returns to canonical WorkItem / Handoff / Artifact evidence instead of becoming a detached notification or chat message |
| `P-16` | Mobile interrupt triage | `Prompt blocked`, `Gate needed`, and `Handoff pending` can be triaged from mobile with policy-aware actions and clear desktop fallback |

## 5. UX acceptance checklist

| ID | Requirement | Pass condition |
|---|---|---|
| `U-01` | Mixed English/CJK readability | Typography, spacing, and chip sizing remain readable for mixed-script content |
| `U-02` | P0 on-screen legibility | Shell answers what needs action, who owns what, what evidence exists, and what budget is at risk |
| `U-03` | Intent-first interaction | P0 actions rely on inline confirm/edit/cancel flows instead of modal-heavy blank forms |
| `U-04` | Continuity preview clarity | Context tiers, budget impact, matched seat skills, and matched playbooks are visually distinct |
| `U-05` | Failure usability | Empty, loading, and failure states show reason plus next step and are not console-only |
| `U-06` | Keyboard behavior | `Cmd/Ctrl+K`, `Esc`, `Enter`, and documented focus behavior work on the claimed surfaces |
| `U-07` | Review workspace quality | Artifact reader and review rail support search, comment state, version-aware anchors, and structured change-request entry points |
| `U-08` | Typed artifact legibility | Family badge, subtype chip, metadata strip, and summary cards make coordination documents understandable before the user reads full markdown body |
| `U-09` | Review change clarity | Tier badge, execution mode, changed clauses, impact, evidence, and bounded `L2` acknowledgment remain legible in work-detail and artifact-review surfaces |
| `U-10` | Prompt-state clarity | Prompt classification, policy, bounded preview, and allowed actions are legible before the user decides how to continue |
| `U-11` | Mobile boundedness | Mobile surfaces stay compact, sync freshness stays visible, and the UI never pretends the phone is a full terminal or heavy-edit workspace |
| `U-12` | Mobile evidence sufficiency | Quick View exposes enough metadata and bounded evidence for approval or escalation without requiring the desktop review rail |

## 6. Engineering acceptance checklist

| ID | Requirement | Pass condition |
|---|---|---|
| `E-01` | Deterministic projection | Inbox and Timeline summaries can be produced from canonical state without LLM dependence |
| `E-02` | Retrieval order | Evidence lookup follows structured index -> full-text -> semantic -> explanation |
| `E-03` | Budget enforcement | Hard limits produce structured stop / retry / escalate behavior rather than silent truncation |
| `E-04` | Continuity persistence | Launch/continuity packs and checkpoints are durable, inspectable, and recoverable |
| `E-05` | Isolation policy | Worker seats only receive pack-included or explicitly loaded evidence |
| `E-06` | Auditability | State-changing actions emit audit-visible events traceable from Timeline and object detail |
| `E-07` | Fallback reliability | Runtime switch and recovery expose a deterministic fallback when native resume is unavailable |
| `E-08` | Artifact metadata indexing | `template`, `subtype`, and other header-derived fields are extracted into structured search/detail data, validated against the subtype allow-list, and degrade visibly when metadata is missing or invalid |
| `E-09` | Review change audit record | Every review-driven follow-up persists `tier`, `reason`, `changed_clauses`, `impact_level`, `executor`, `reviewer`, `ack_mode`, and `evidence_refs`; invalid or verbose `L2` acknowledgment is rejected |
| `E-10` | Prompt-state auditability | Interactive prompt detection, injected input, supervisor assist, and takeover decisions emit structured audit-visible events and preserve deterministic policy classification |
| `E-11` | Mobile action consistency | Mobile approvals, feedback, and interrupt triage reuse desktop object transitions and add only bounded channel metadata such as `source=mobile` or sync freshness state |

## 7. QA evidence package

Every acceptance claim must include an evidence package under a coordination artifact path.

Required evidence for claimed P0 work:

- screenshots or recordings for each claimed end-to-end P0 flow,
- proof that Inbox rows resolve correctly after action,
- proof that review rejection links to reason and evidence,
- proof that review follow-up creates a valid `change_tier_record`, shows the correct `L1/L2/L3` path, and keeps `L2` acknowledgment compact,
- proof that continuity preview shows tiers, budgets, and fallback,
- proof that evidence search returns deterministic exact results,
- proof that prompt-blocked states show deterministic classification, policy chips, bounded preview, and the correct action set,
- proof that typed coordination artifacts render the correct summary fields and filter correctly by artifact family and subtype,
- proof that mobile monitoring surfaces show sync freshness, urgent action counts, and canonical high-priority routing,
- proof that at least one mobile approval / escalation / reserve-desktop action mutates the same canonical object history visible on desktop,
- proof that mobile feedback returns as a canonical object linked to WorkItem / Handoff / Artifact evidence rather than a detached note,
- proof that failures are visible in-product rather than console-only,
- changed-file list and environment/run instructions where relevant.

Required evidence for claimed P1/P2 work:

- clear statement of claimed scope,
- screenshots or recordings of the extension surface,
- evidence that human approval and auditability are preserved where required,
- fallback behavior when optional systems such as semantic retrieval or live mode are unavailable.

## 8. Go / Hold rules

### Go

- Gate A passes,
- every claimed P0 requirement passes,
- no unresolved Critical issue exists in product truth, interaction, UX, or engineering acceptance,
- any High issue has an explicit owner, due date, and non-ambiguity about the user impact.

### Hold

- any claimed P0 flow fails end-to-end,
- deterministic truth is replaced by an LLM-only answer in a P0 baseline flow,
- continuity launch/recovery hides tiers, hides budget impact, or lacks fallback,
- review follow-up hides the tier, omits the structured `change_tier_record`, or lets `L2` drift into a long confirmation chain,
- exact evidence search depends on semantic or LLM interpretation to return the first usable result,
- the implementation requires users to leave SeatLoom for the primary action of a claimed P0 flow.

## 9. Required reporting format

Acceptance reviews should publish, at minimum:

1. verdict,
2. claimed scope,
3. coverage matrix,
4. findings by severity,
5. required fixes with exact file targets,
6. evidence artifact paths,
7. go / no-go recommendation.
