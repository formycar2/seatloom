# LYRA-2026-04-28-Cross-Priority-Value-Map-v1

| Field | Value |
|---|---|
| Owner | Lyra |
| Status | Active baseline artifact |
| Purpose | Clarify P0, P1, and P2 capability value before implementation |
| Active contract basis | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/acceptance-spec-v1.0.md` |
| Constraint references | `docs/archive/product-history/mvp-scenarios.md`, `docs/architecture-decisions.md`, `docs/coordination/reviews/2026-04-28-lyra-review-response.md`, `docs/coordination/reviews/2026-04-28-aegis-vs-lyra-multica-alignment.md` |

## 1. Decision

SeatLoom will not start implementation from a vague "MVP bucket."

For this cycle, every capability must belong to one of three lanes:

- `P0`: required for the first implementation baseline and stage-gate readiness
- `P1`: not blocking baseline, but P0 must leave room for it intentionally
- `P2`: reserved future expansion; current work must not close the door on it

## 2. Priority closure rules

This artifact closes four planning ambiguities for the current cycle:

1. Multi-project switching is `P0`, not a later enhancement.
2. Handoff is `P0`, even though `docs/archive/product-history/mvp-scenarios.md` still tags it as Phase 3.
3. Pipeline is split into two layers:
   - `P0`: inbox/error-surface accommodation for pipeline failure signals
   - `P1`: full pipeline execution workflow and deep-focus operational UI
4. Deep decision retrospection beyond normal Timeline replay is `P2`; baseline replay remains `P0`.

## 3. P0 value map

| ID | Capability | User value / job-to-be-done | Baseline commitment | Must leave room for |
|---|---|---|---|---|
| P0-01 | Project open and initialize | Let a user bring a repo under SeatLoom control without setup confusion. | Open repo, detect missing `.seatloom/`, initialize, land in guided empty state. | Future onboarding shortcuts and runtime discovery. |
| P0-02 | Multi-project switching and memory | Let one operator move between projects without losing context or corrupting boundaries. | Project Switcher, recent/pinned projects, switch protection, per-project UI-state restore, All Projects summary. | Deeper cross-project command palette and project health overview. |
| P0-03 | Core work surface and default overview | Let the app answer "what needs attention now" in one screen without forcing a workflow commitment. | Sidebar, Main, Detail, Terminal, Status surfaces; default Detail shows `Project Overview`, not a blank pane. | Richer dashboard widgets and runtime-ops surfaces. |
| P0-04 | Seat and session visibility | Make active operators and active execution visible at a glance. | Seat list, session list, session status, owning seat, runtime visibility. | P1 seat-card enrichment and runtime-health views. |
| P0-05 | Attach and wrap session capture | Let the user continue work in existing tools or start a new tracked session without changing tool habit. | Attach running session when available; wrap new session; capture terminal evidence and status. | Runtime capability matrix, auto-discovery, richer execution history. |
| P0-06 | WorkItem lifecycle and gates | Keep work scoped, reviewable, and restartable instead of living in chat scrollback. | `draft -> ready -> active -> in_review -> verified -> done` plus `blocked/reopened/drifted`, with explicit gate visibility. | Claim events, inline editing, and future child-work proposals. |
| P0-07 | Inbox as human action queue | Surface only items that require human action and remove notification noise. | Inbox contains only actionable handoff/session/workitem/pipeline-failure items; actioning the item changes canonical object state. | Structured blocker evidence, live mode, automation signals that still stay human-actionable. |
| P0-08 | Timeline replay with readable summaries | Let the user answer who did what, when, and based on what evidence. | Human-readable event labels, filters, search in Timeline, event-to-object drill-through. | Live toggle, fuller transcript drilldown, broader search. |
| P0-09 | Handoff closed loop | Transfer work across seats/tools with purpose, expected outcome, artifacts, and receipt. | Create, send, accept, return, complete; all states traceable to WorkItem and Artifact. | Draft assistance, skill-assisted routing, autonomy-safe draft creation. |
| P0-10 | Rehydrate by LaunchPack and runtime switch | Move work from one runtime to another without re-explaining the task. | Generate LaunchPack file first, show preview, switch runtime, provide L1/L2 fallback only. | SkillPack injection, runtime capability truth enrichment, compression for oversized context. |
| P0-11 | Interrupted-session recovery | Recover from interruption without forcing the user to restate the job. | Recovery order is fixed: native resume -> checkpoint + LaunchPack rebuild -> minimal rebuild. | Timed checkpoints and stronger native resume support. |
| P0-12 | Reconcile and Morning Digest | Re-anchor the user quickly at startup and before risky operations. | Startup/manual/pre-pipeline reconcile, digest of interruptions/drift/new commits, Inbox handoff into work. | Scheduled automation, background runs, richer project health summaries. |
| P0-13 | Runtime capability truth at execution points | Prevent false promises about what a runtime can resume, inject, or attach to. | `Wrap`, `Attach`, `Switch Runtime`, and session detail must show capability truth and failure fallback. | Dedicated runtime-health dashboard and provider matrix management. |

## 4. P1 value map

| ID | Capability bundle | Why it matters | What P0 must preserve | Not required for baseline |
|---|---|---|---|---|
| P1-01 | Faster editing and navigation | Reduce modal friction and speed up frequent edits once the baseline contract is stable. | Stable object IDs, focus model, and detail-pane actions. | Baseline can still ship with dialogs and explicit forms. |
| P1-02 | Manual evidence intake and richer inspection | Allow screenshots, notes, and external artifacts to enter the same continuity chain. | Artifact model, attachment refs, detail-pane evidence slots. | No manual upload/import is required before baseline freeze. |
| P1-03 | Runtime operations layer | Help operators judge runtime availability, limits, and health before launching work. | Runtime identity, capability fields, session ownership, health placeholders. | No standalone runtime-ops dashboard is required for P0. |
| P1-04 | Context intelligence | Improve continuity quality through timed checkpoints, SkillPack/Seat Pack reuse, and lossy compression when needed. | LaunchPack file-first flow, checkpoint object model, runtime capability flags. | P0 remains deterministic and rule-based. |
| P1-05 | Live operational awareness | Let users watch work as it happens without sacrificing replay quality. | Event stream, object linking, filters, and Terminal/session evidence. | Timeline does not default to a chat-like live wall in baseline. |
| P1-06 | Visible automation with history | Make recurring reconcile/pipeline/runtime checks trustworthy because they leave visible evidence. | Ledger append, Inbox rule engine, Timeline projection, artifact/evidence model. | No daemon-first or invisible background automation is allowed in P0. |
| P1-07 | Readability and dense-work polish | Raise throughput by making mixed Chinese/English interfaces easier to scan at real working density. | Typography tokens, status semantics, consistent label hierarchy. | Functional correctness comes first, but P0 cannot ignore readability debt. |

## 5. P2 value map

| ID | Capability bundle | Future value | Boundary for this cycle |
|---|---|---|---|
| P2-01 | Bounded seat autonomy | Let seats generate artifacts and draft handoffs with less operator ceremony while staying auditable. | Seats may assist, but humans still own acceptance, promotion, and closure. |
| P2-02 | Human-approved child-work proposals | Allow a seat to suggest decomposition without silently mutating the work graph. | Proposed child WorkItems remain drafts/proposals until a human accepts them. |
| P2-03 | Scratch collaboration outside the main flow | Give operators a temporary thinking space without polluting the main work graph. | It must never bypass WorkItem/Handoff/Artifact ledger rules for accepted work. |
| P2-04 | Expanded project stewardship surfaces | Support broader project-level coordination once baseline execution continuity is stable. | SeatLoom must not drift into a generic issue-board or agent-management platform. |

## 6. Cross-cutting baseline constraints

These rules apply across P0, P1, and P2:

1. Inbox remains a human action queue, not a notification center.
2. Cross-project views are summaries only; no cross-project object merge/edit in this cycle.
3. All state-changing actions must produce durable evidence, not terminal-only claims.
4. LaunchPack is always written to disk before any injection attempt.
5. Failure handling must be explicit: reason + next step, never silent degradation.
6. Readability for mixed Chinese/English content is a product requirement, not a cosmetic afterthought.

## 7. Implementation gate implication

Implementation may proceed only when downstream packets use this priority model consistently:

- Mira designs `P0` as contractual and treats `P1/P2` as preserved expansion space.
- Nimbus implements only `P0` obligations unless a later gate explicitly promotes a `P1` item.
- Flux verifies `P0` against baseline acceptance and records `P1/P2` only as non-blocking observations.
