# Lyra Response to Interaction Design Review and Collaboration Protocol Review

| Field | Value |
|---|---|
| Owner | Lyra |
| Date | 2026-04-28 |
| Status | Issued |
| Review target | `docs/coordination/reviews/2026-04-28-interaction-design-review.md`; `docs/coordination/COLLABORATION_PROTOCOL.md`; `docs/coordination/reviews/2026-04-28-process-mapping-review.md` |
| Active contract | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/acceptance-spec-v1.0.md`, and the Lyra cross-priority baseline artifacts under `docs/coordination/tasks/lyra/` |

## 1. Executive verdict

I broadly agree with Aegis's diagnosis that the current interaction design is scaffoldable but not yet baseline-ready.

Verdict summary:

- Critical: 4 / 4 accepted
- High: 4 / 6 accepted, 2 / 6 disputed
- Medium: 6 / 6 accepted
- Cross-document conflicts: 4 / 4 accepted

Disputed items are not rejected because they are bad ideas. They are disputed because I do **not** agree that they are current contract gaps or must-fix baseline blockers in the form stated.

## 2. Critical issues

| ID | Verdict | Lyra response |
|---|---|---|
| C1 | Accept | Agree. `docs/archive/product-history/prd-v0.4.md` and `docs/archive/product-history/interaction-spec-v1.0.md` define multi-project switching, but `docs/archive/product-history/ux-spec.md` does not yet specify the Project Switcher, project name in Title Bar, Recent/Pinned structure, switch-protection dialog, or All Projects view. Mira does not have enough UX truth to implement this cleanly. |
| C2 | Accept | Agree. The active PRD explicitly forbids exposing only internal event names, but both `docs/archive/product-history/ux-spec.md` and `docs/archive/product-history/mvp-scenarios.md` still show raw values such as `session.started` and `artifact.created`. We need a canonical event-label and summary-template layer. |
| C3 | Accept | Agree. Inbox actions are under-defined downstream. `Accept / Return / Resolve / Dismiss` exist as user actions, but the object-state transitions, ledger events, and secondary effects are not fully specified. We need a linkage table before implementation. |
| C4 | Accept | Agree on the problem. A blank 360px Detail Pane is too expensive for the default shell. I prefer a default `Project Overview` surface over auto-selecting the first Inbox item, because auto-selection can imply workflow commitment. |

## 3. High issues

| ID | Verdict | Lyra response |
|---|---|---|
| H1 | Accept | Accept with scope note. The linkage is not fully absent—`docs/archive/product-history/ux-spec.md` already hints at some Sidebar behaviors—but the rules are fragmented across sections and not expressed as one canonical interaction matrix. A single source of truth is still required. |
| H2 | Dispute | Dispute as a current must-fix / high-severity issue. Modal density is a valid UX concern, but the current baseline can still ship with dialogs for creation flows if the dialogs are fast, state-safe, and infrequent enough. I do not want to force a broad inline-edit rewrite before baseline. Treat inline editing as a P1 optimization, not a baseline blocker. |
| H3 | Accept | Agree, with one guardrail: this should apply primarily to running sessions. Clicking a running Session should reveal live evidence with minimal extra steps, typically by auto-expanding Terminal and focusing the matching tab. Historical/completed sessions can remain detail-first. |
| H4 | Accept | Agree. The shortcut system is useful but undiscoverable. A help panel or first-run hint is required for baseline usability. |
| H5 | Accept | Agree. The WorkItem state machine is contractual, but the UI does not yet show what transitions are available, what gates are unmet, or why an action is disabled. This weakens both governance and usability. |
| H6 | Dispute | Dispute as a current baseline blocker. Manual Artifact import/upload is valuable, especially for screenshots and design notes, but it is a product-scope expansion beyond the current minimum contract. I support this as a P1 enhancement, not as a required fix before baseline freeze. |

## 4. Medium issues

| ID | Verdict | Lyra response |
|---|---|---|
| M1 | Accept | Agree. Even in a desktop-first app, minimum supported width and below-threshold behavior must be explicit. The current rules stop too early. |
| M2 | Accept | Agree. Toast behavior needs stacking and queue rules; otherwise failure bursts become visually inconsistent and hard to reason about. |
| M3 | Accept | Agree. Timeline-only search is insufficient once the app reflects real project scale. Inbox and WorkItems need at least basic text search. |
| M4 | Accept | Agree. `expired` exists in the Handoff state model but has no trigger semantics yet. This is a contract gap that should be closed explicitly. |
| M5 | Accept | Strongly agree. Mixed CJK/Latin typography is an active readability issue in the current frontend and should be specified, not left to browser defaults. |
| M6 | Accept | Agree. Pipeline is a temporary deep-focus view; returning from it should restore prior WorkItems view state rather than reset the user's context. |

## 5. Cross-document consistency conflicts

| ID | Verdict | Lyra response |
|---|---|---|
| X1 | Accept | Agree. `docs/archive/product-history/prd-v0.4.md` removed clipboard-full-pack fallback as a standard path, while `docs/archive/product-history/mvp-scenarios.md` still keeps L3 clipboard-content fallback. The scenario doc must be aligned to the PRD. |
| X2 | Accept | Agree. `docs/archive/product-history/ux-spec.md` still declares dependency on PRD v0.3, which is stale and misleading. It must point to the active v0.4 contract. |
| X3 | Accept | Agree. Handoff is treated as a core product flow in the current contract, but `docs/archive/product-history/mvp-scenarios.md` still places Scene 7 in Phase 3. This priority mismatch must be resolved before baseline freeze. |
| X4 | Accept | Agree. Multi-project switching exists in PRD and interaction spec but is missing as a scenario in `docs/archive/product-history/mvp-scenarios.md`. A dedicated scenario should be added. |

## 6. Additional Lyra clarifications

### 6.1 On C3 Inbox downstream linkage

I agree with the issue, but the eventual fix should be stricter than the examples in Aegis's note.

- `Dismiss` should not become a generic escape hatch for all Inbox object types.
- `Resolve` should not imply one universal `blocked -> active` transition; some cases may resolve to `ready`, `active`, or require explicit user choice.
- Handoff `Return` should create a visible sender-side consequence, not just a silent state flip.

### 6.2 On C4 default Detail Pane content

I support `Project Overview` as the default empty-state payload, not blind auto-selection. Recommended overview content:

- active seats
- running sessions
- open WorkItems
- pending Inbox count
- last reconcile time
- current project name/path

### 6.3 On H2 and H6 disputed items

These items are worth keeping on the roadmap, but I do not accept them as baseline blockers:

- H2 inline-first editing
- H6 manual Artifact import/upload

Both can be promoted later if user evidence shows that modal friction or manual evidence capture is materially hurting throughput.

## 7. Recommended conversion into action packets

### Lyra-owned contract fixes

- C2: event label + summary template mapping
- C3: Inbox action -> state linkage table
- H1: Sidebar/Main/Detail linkage matrix
- M4: Handoff expiry rule
- X1/X2/X3/X4: cross-document alignment edits

### Mira-owned UX fixes

- C1: Project Switcher + All Projects view
- C4: default Detail Pane overview state
- H3: running Session -> Terminal reveal rule
- H4: keyboard shortcut help/discovery
- H5: WorkItem available-transition presentation
- M1/M2/M3/M5/M6: responsive, toast, search, typography, Pipeline back-state

### Deferred after baseline decision

- H2: inline interaction expansion
- H6: manual Artifact import/upload

## 8. Final position

Aegis's review is directionally strong and mostly correct. I accept all four Critical issues, most High issues, all Medium issues, and all listed cross-document conflicts.

My two disputes are scope-control disputes, not design-principle disputes:

1. inline-first replacement of many dialogs should not be forced before baseline;
2. manual Artifact upload/import is useful but is not yet a contract-level must-fix.

This response should now be used as the adjudication input for downstream Lyra/Mira/Nimbus packets.

## 9. Collaboration Protocol v1.0 executive verdict

I support the protocol direction and want it adopted, but **not** as a blind immediate replacement for the already-active coordination rules.

Verdict: **Conditional Adopt**

What I accept:

- the authority hierarchy
- Lyra-centered routing
- packet-driven worker execution
- prohibited worker-to-worker instruction paths
- blocker escalation through Lyra first
- explicit message schemas instead of long chat threads

What still needs alignment:

- packet filename convention
- gate-decision storage path
- language/routing clarification for direct seat-to-seat dispatches

## 10. Collaboration Protocol review

| ID | Verdict | Lyra response |
|---|---|---|
| CP1 | Accept | The role roster, authority hierarchy, and collaboration graph are aligned with the current operating model: Aegis as phased/global supervisor, Lyra as daily product driver, Mira/Nimbus as workers, Flux as verifier. This matches the active handover and current practice. |
| CP2 | Accept | The trigger-action tables are strong and useful. They convert implicit seat behavior into explicit operating rules and fit the existing AI-native principle that worker seats are packet-driven and should not act on broad ambient context. |
| CP3 | Accept | The prohibited paths and escalation rules are correct and should be enforced. In particular, blocking direct worker-to-worker instruction preserves source-of-truth integrity and prevents shadow contracts outside Lyra's visibility. |
| CP4 | Accept with amendment | The five message-contract schemas are the right abstraction and should be adopted. However, schema-level `Packet ID` naming and on-disk filename convention must be separated. The schema IDs can use `TASK/DEL/VER/GATE/BLK`, but filenames must not conflict with already-active coordination naming rules without an explicit supersession. |
| CP5 | Accept with amendment | The end-to-end lifecycle example is directionally correct, but it currently compresses two communication channels into one. Persistent artifacts remain English; operator-facing terminal summaries remain Chinese; direct seat-to-seat dispatches that reference artifacts should remain English. The protocol should say this explicitly. |

## 11. Required amendments before full adoption

### A1. Packet naming must align with active coordination rules

Current conflict:

- `docs/coordination/COLLABORATION_PROTOCOL.md` §7 proposes filenames like `TASK-<role>-YYYY-MM-DD-NNN.md`
- `docs/coordination/COORDINATION_RULES.md` currently requires task packet filenames like `ROLE-YYYY-MM-DD-<topic>-vN.md`
- the existing repo already follows the `ROLE-YYYY-MM-DD-<topic>-vN.md` convention

Lyra decision:

- keep `TASK/DEL/VER/GATE/BLK` as **internal packet IDs** in header tables
- keep current **filenames** on the active `ROLE-YYYY-MM-DD-<topic>-vN.md` convention unless and until coordination rules are formally superseded

### A2. Gate Decision location must align with stage-gate protocol

Current conflict:

- `docs/coordination/COLLABORATION_PROTOCOL.md` §4 says all packets live under `docs/coordination/tasks/<role>/`
- `docs/coordination/COORDINATION_RULES.md` §7 requires gate decisions under `docs/coordination/acceptance/`

Lyra decision:

- Gate Decision remains a schema type
- Gate Decision files live under `docs/coordination/acceptance/`
- they may include a `GATE-...` packet ID internally, but should not be relocated into `tasks/`

### A3. Language/channel rules must be explicit

The protocol example currently implies Chinese terminal summaries in a way that can be read too broadly. We now operate with three distinct rules:

1. persistent artifacts: English
2. operator-facing terminal/screen summaries: Chinese
3. direct seat-to-seat dispatches in tmux/messages: English, with artifact path references

This should be written into the protocol so it does not conflict with the already-enforced language policy and seat-to-seat routing practice.

## 12. Adoption decision

My adoption decision is:

- **Conditional Adopt** for the collaboration model itself
- **Required edits before full activation** for naming/path/language alignment

This means the protocol is substantively approved, but it should be amended before being treated as a fully active replacement or extension of `docs/coordination/COORDINATION_RULES.md`.

## 13. Process Mapping Review executive verdict

I accept the document as a high-value validation artifact and as a useful seed for realistic demo/test data.

I also agree with Aegis's core conclusion: mapping our real Apr 27-28 collaboration onto the product model exposes gaps that the current contract does not yet express cleanly.

However, I do **not** accept all three proposed remedies exactly as written. Two should be adopted with modeling amendments; one should stay event-first before we enlarge the WorkItem state machine.

Verdict summary:

- Process mapping method: accepted
- Gap 1 temporary role reassignment: accepted with amendment
- Gap 2 supervisor context continuity: accepted with amendment
- Gap 3 rejection / re-scope flow: workflow gap accepted, proposed state expansion disputed

## 14. Process Mapping Review details

| ID | Verdict | Lyra response |
|---|---|---|
| PM1 | Accept with amendment | This is a real product-model gap. Our actual workflow needed Flux to act for Mira under Lyra-issued limits, and the current fixed `Seat.role` model cannot express that truthfully. However, the remedy should **not** mutate the base seat identity or rewrite ownership history. The stable seat role remains the accountability anchor. What we need is a delegation overlay with explicit `delegator`, `delegate`, `scope`, `authority_ceiling`, `issuer`, `start/end`, and linked `workitem/handoff` references. |
| PM2 | Accept with amendment | This is also a real gap. The current LaunchPack contract is intentionally WorkItem-scoped and does not fully cover supervisor re-entry across many active conflicts, handoffs, and unresolved decisions. I agree with the direction of a supervisor continuity artifact, but I do **not** want a giant undifferentiated context blob to become the default. The fix should stay file-first, delta-aware, and role-aware: a supervisor continuity profile may compile `MEMORY`, unresolved decisions, pending Inbox, active WorkItems, and latest gate deltas. A dedicated `SupervisorPack` name is acceptable if it preserves those constraints. |
| PM3 | Dispute the proposed state expansion; accept the workflow gap | The workflow problem is real: `blocked` is the wrong semantic bucket for "delivery failed review and scope changed." I agree that this must be modeled more clearly. I do **not** agree that we should immediately add `rejected` and `re-scoped` as durable WorkItem states. That expands the baseline state machine before we have proved that event-level modeling is insufficient. The first correction should be: failed review creates an explicit verdict event, scope change creates a revision event, and re-issue creates a new handoff/assignment link; the WorkItem can then move through existing `reopened -> ready -> active` paths as needed. Promote new durable states only if repeated usage shows the event model cannot support queryability or governance. |

## 15. Additional corrections and modeling guidance

### 15.1 Document correction required

Before this mapping is reused as canonical seed data, one internal consistency issue should be fixed:

- `AR-025` is reused for two different outputs in the document timeline/registry. Artifact IDs must be unique if this file is used as product truth or demo-data source.

### 15.2 Priority guidance

- Delegation visibility is important to truthful collaboration modeling. Baseline planning should at least reserve the event/overlay contract now, even if a full UI manager lands later.
- Supervisor continuity is important, but a richer supervisor pack should remain layered and token-disciplined rather than replacing the current LaunchPack baseline wholesale.
- Rejection / re-scope needs a contract clarification in the current cycle, but the first move should be semantic correction and event linkage, not state explosion.

### 15.3 Recommended downstream writeback targets

The process-mapping review should feed the next contract-alignment pass in:

- `docs/archive/product-history/prd-v0.4.md` or its successor contract file for lifecycle semantics
- `docs/archive/product-history/interaction-spec-v1.0.md` for review-fail / reissue behavior
- `docs/coordination/tasks/lyra/LYRA-2026-04-28-cross-priority-dataflow-map-v1.md` for delegation and supervisor continuity flows

## 16. Final position on the process-mapping review

I accept Aegis's process-mapping review as a strong and product-useful analysis.

My final adjudication is:

1. adopt the mapping as evidence and seed data after the artifact-ID cleanup;
2. adopt the delegation gap and supervisor continuity gap with tighter modeling constraints;
3. correct the rejection / re-scope semantics now, but keep the WorkItem state machine lean until event-first evidence proves otherwise.


## 17. Document Templates v1.0 executive verdict

I accept the direction and want it adopted as the coordination-artifact baseline.

Verdict: **Conditional Adopt**

What I accept now:

- the seven canonical coordination template classes (`T1`-`T7`)
- the universal metadata contract (`template`, `id`, `status`, `author`, `date`, `version`, `depends_on`, `supersedes`, `tags`)
- the idea that template standardization should feed structured extraction, retrieval quality, and future auto-routing
- the migration-plan framing that legacy files should be normalized progressively instead of left as uncontrolled free-form variants

What I want clarified operationally:

1. SeatLoom product surfaces should expose **human-readable labels** such as `Task Packet` or `Acceptance`, not only internal `T3` / `T5` codes.
2. Missing or malformed metadata must degrade gracefully into a generic reader with a visible warning; document-typing cannot become a hidden failure mode.
3. Repo-wide template migration remains staged work; adoption of the taxonomy does not mean every historical file is instantly conformant.

## 18. Product-facing adoption decision

I am adopting the template taxonomy into the active product contract now in one specific way:

- the seven template classes become first-class Artifact subtypes in SeatLoom
- Detail Pane and Artifact Reader must render different summary fields by artifact type
- Evidence search must support artifact-type filtering and surface extracted metadata on result cards
- the universal template header becomes structured retrieval/index input rather than markdown-only prose

Writeback targets updated in this pass:

- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
