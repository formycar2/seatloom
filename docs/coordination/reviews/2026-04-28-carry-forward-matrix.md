# SeatLoom Carry-Forward Matrix

| Field | Value |
|---|---|
| Document | Carry-Forward Matrix |
| Status | Working migration artifact |
| Updated | 2026-04-28 |
| Owner | Lyra |
| Canonical entrypoint | `docs/PRODUCT_TRUTH.md` |
| Purpose | Track which legacy product rules are retained, where they belong in the active contract set, and what still needs migration or explicit rejection |

## 0. Decision legend

| Decision | Meaning |
|---|---|
| Keep | Retain as active product truth in the active contract set |
| Keep with rewrite | Retain the intent, but rewrite it into the active contract set instead of copying legacy wording |
| Reject | Do not carry forward because the rule conflicts with the active v0.5 contract or creates product noise |
| Historical only | Keep only as background context or validation history; not an active delivery rule |

## 1. `docs/archive/product-history/prd-v0.3.md`

| Legacy rule / decision | Decision | Target active doc | Current status | Required action |
|---|---|---|---|---|
| SeatLoom is a local-first continuity layer, not another agent, IDE, or platform. | Keep | `docs/prd-v0.5.md`, `docs/PRODUCT_TRUTH.md` | Already preserved | None beyond keeping all new references on the active set. |
| Product value depends on heterogeneous tool coexistence rather than single-tool dominance. | Keep with rewrite | `docs/prd-v0.5.md` | Partial | Add an explicit cross-tool rationale note in a future PRD cleanup pass so this value does not live only in historical documents. |
| Even if upstream tools improve memory, SeatLoom still retains cross-tool continuity, governance, and audit value. | Keep with rewrite | `docs/prd-v0.5.md` | Partial | Carry forward as a durable market-defense rule in a later thesis/rationale section. |
| Five shovels framing: Attach, Capture, Rehydrate, Replay, Govern. | Keep with rewrite | `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md` | Partial | Preserve as a crosswalk or glossary so legacy capability language still maps to v0.5 modules and stories. |
| Governance is intentionally light in MVP and should not expand into a heavy policy system too early. | Keep | `docs/prd-v0.5.md` | Already preserved | None. |
| Progressive adoption model (`Observe`, `Assist`, `Structure`) explains adoption depth. | Historical only | Future GTM or adoption doc | Missing | Do not block implementation; migrate later only if the team wants an onboarding/adoption narrative. |
| Core object vocabulary must remain stable: Seat, Session, WorkItem, Handoff, Artifact, Checkpoint. | Keep | `docs/prd-v0.5.md`, `docs/architecture-design.md` | Partial | Add an explicit active glossary or schema crosswalk in a later cleanup pass. |
| Local data ownership and offline usefulness are part of the product promise. | Keep | `docs/prd-v0.5.md`, `docs/architecture-design.md` | Already preserved | None. |

## 2. `docs/archive/product-history/prd-v0.4.md`

| Legacy rule / decision | Decision | Target active doc | Current status | Required action |
|---|---|---|---|---|
| First delivery must let the user observe active work, switch tools, replay history, complete handoffs, recover sessions, and resume from a morning digest. | Keep with rewrite | `docs/prd-v0.5.md`, `docs/acceptance-spec-v1.1.md` | Partial | Keep the six-user-outcome frame explicit in acceptance reviews so validation still matches the product promise. |
| Unified observation surface is mandatory: Inbox, Timeline, Detail, Terminal. | Keep | `docs/prd-v0.5.md`, `docs/ux-spec-v1.1.md` | Already preserved | None. |
| Inbox is an action queue, not a generic notification feed. | Keep | `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Already preserved | None. |
| Timeline must show readable human summaries, not raw event codes. | Keep | `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md` | Already preserved | None. |
| WorkItem, Session, and Handoff lifecycle semantics must be explicit and testable. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/architecture-design.md`, `docs/acceptance-spec-v1.1.md` | Partial | Add an active lifecycle/state appendix so validation does not depend on v0.4 wording. |
| Inbox entry, exit, and priority rules should be contract-level behavior. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Partial | Add a compact active Inbox projection table during the next spec-alignment pass. |
| LaunchPack continuity must survive runtime switching and interruption recovery. | Keep | `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Already preserved | None. |
| LaunchPack / continuity packs must be file-backed and recoverable, not memory-only. | Keep with rewrite | `docs/architecture-design.md`, `docs/acceptance-spec-v1.1.md` | Partial | Add an explicit storage rule in architecture language and verify it in implementation evidence. |
| Multi-project switching needs recent/pinned projects, state memory, and switch protection. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Partial | Restore the missing switch-protection and state-restore specifics into the active interaction and acceptance chain. |
| `All Projects` summary view is part of project-level orientation. | Keep with rewrite | `docs/ux-spec-v1.1.md`, `docs/interaction-spec-v1.1.md` | Missing | Decide whether to keep an explicit `All Projects` surface or replace it with a lighter project-overview pattern. |
| Handoff must remain traceable through purpose, expected outcome, evidence, and receipt state. | Keep | `docs/prd-v0.5.md`, `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Already preserved | None. |

## 3. `docs/archive/product-history/mvp-scenarios.md`

| Legacy rule / decision | Decision | Target active doc | Current status | Required action |
|---|---|---|---|---|
| No-project entry and project initialization must be first-class flows, not setup footnotes. | Keep | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Partial | Keep the empty-state and initialization flow explicit when active specs are finalized. |
| Attach existing session is a core continuity path. | Keep | `docs/interaction-spec-v1.1.md` | Partial | Ensure the active continuity flow keeps `attach` visible as a distinct user path, not only as generic recovery text. |
| Wrap new session is a core continuity path. | Keep | `docs/interaction-spec-v1.1.md` | Partial | Keep `wrap` explicit in the active continuity flow and acceptance checks. |
| Morning Digest should appear after reconcile and orient the user before deeper navigation. | Keep | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Already preserved | None. |
| Inbox rows should be actionable and resolve cleanly after the user acts. | Keep | `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Already preserved | None. |
| Timeline must support replay filters and click-through to object detail. | Keep | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Partial | Restore the exact filter coverage to the active interaction or acceptance layer. |
| Handoff create/send/accept/return/complete loop must remain complete. | Keep | `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Partial | Keep the full closed-loop chain visible in active acceptance language. |
| Runtime-switch and interruption recovery should use a deterministic fallback ladder. | Keep | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Already preserved | None. |
| L3 clipboard fallback for LaunchPack injection. | Reject | None | Intentionally superseded | Keep rejected; active continuity must remain file-first and deterministic. |
| The old handoff phase ordering from Scenario 7. | Reject | None | Intentionally superseded | Keep the v0.5 priority boundaries instead. |
| Dedicated multi-project scenario coverage. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Missing | Add an active multi-project flow so this behavior is not implied only by shell chrome. |
| Pipeline-trigger scenario from the old MVP phase map. | Historical only | Future roadmap doc | Partial | Do not let old pipeline emphasis distort the current pre-implementation freeze. |

## 4. `docs/archive/product-history/ux-spec.md`

| Legacy rule / decision | Decision | Target active doc | Current status | Required action |
|---|---|---|---|---|
| Four-zone shell remains the baseline: Sidebar, Main, Detail, Terminal, plus status context. | Keep | `docs/ux-spec-v1.1.md` | Already preserved | None. |
| Terminal panel stays embedded, bottom-aligned, and collapsible by default. | Keep | `docs/ux-spec-v1.1.md` | Already preserved | None. |
| Detail pane needs explicit object templates rather than ad hoc freeform panels. | Keep with rewrite | `docs/ux-spec-v1.1.md` | Partial | Preserve the object-template discipline when active surface specs are refined. |
| LaunchPack preview and runtime-switch UI must be visible before commit. | Keep | `docs/ux-spec-v1.1.md`, `docs/interaction-spec-v1.1.md` | Already preserved | None. |
| No-project, empty-list, and no-result states need guided next steps. | Keep | `docs/ux-spec-v1.1.md`, `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Partial | Keep the full set of empty/error states explicit in the active acceptance chain. |
| Project switcher, title-bar context, and project-level orientation need concrete surface rules. | Keep with rewrite | `docs/ux-spec-v1.1.md` | Partial | Add the missing `All Projects` or equivalent project-overview decision into the active UX contract. |
| Mixed-script readability needs deliberate font, size, chip, and spacing rules. | Keep | `docs/ux-spec-v1.1.md` | Already preserved | None. |
| Pipeline execution view as a stage-by-stage primary surface. | Historical only | Future roadmap doc | Partial | Keep only as future design inventory unless re-prioritized in the active contract. |
| Mandatory dual light/dark theme parity from day one. | Historical only | Future design system decision | Missing | Do not treat as blocking until the active contract reintroduces it explicitly. |

## 5. `docs/archive/product-history/interaction-spec-v1.0.md`

| Legacy rule / decision | Decision | Target active doc | Current status | Required action |
|---|---|---|---|---|
| Keyboard focus loop should cycle Sidebar -> Main -> Detail -> Terminal with `Tab/Shift+Tab`. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Partial | Restore the explicit focus-loop rule into the active interaction layer. |
| `Esc` closes the current surface and `Enter` confirms the current highlighted action when valid. | Keep | `docs/interaction-spec-v1.1.md` | Already preserved | None. |
| Async actions must expose `pending`, `success`, and `error`, and error states must show reason plus next step. | Keep | `docs/interaction-spec-v1.1.md` | Already preserved | None. |
| Long lists need keyboard movement plus pagination or lazy loading. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Partial | Add the large-list behavior back into the active interaction/UX set if the list density remains high. |
| Multi-project switching must include recent/pinned choices, protection checks, and last-state restore. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Partial | Restore the missing project-switch detail so the flow is not underspecified. |
| Project initialization is a concrete user flow with retry behavior. | Keep | `docs/interaction-spec-v1.1.md` | Partial | Keep the retry branch explicit in the active no-project flow. |
| Add-seat creation is a concrete user flow, not just a schema capability. | Keep | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Partial | Make sure the active seat-registry flow stays operable from the UI, not only conceptually defined. |
| Attach existing session must show selectable candidates and clear failure handling. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` | Partial | Carry the attach-specific failure and recovery detail into active acceptance criteria. |
| Wrap new session must validate runtime choice and show post-launch visibility. | Keep with rewrite | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Partial | Keep `wrap` explicit and confirm post-launch visibility rules in active specs. |
| No-project and error states must stay visible in-product rather than console-only. | Keep | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md` | Already preserved | None. |

## 6. `docs/archive/product-history/product-positioning.md`

| Legacy rule / decision | Decision | Target active doc | Current status | Required action |
|---|---|---|---|---|
| SeatLoom is a local-first continuity layer above heterogeneous agent tools. | Keep | `docs/prd-v0.5.md`, `docs/PRODUCT_TRUTH.md` | Already preserved | None. |
| Progressive adoption matters because users should not need a full platform migration to get value. | Keep with rewrite | Future positioning or onboarding doc | Partial | Preserve in future positioning/onboarding material, not as a blocking contract rule. |
| The four value proofs / vital signs should remain a success-measure frame. | Keep with rewrite | Future validation or metrics doc | Missing | Rebuild as a v0.5-aligned validation artifact instead of leaving it stranded in positioning. |
| Non-goals must remain explicit to prevent drift into cloud-first agent-management software. | Keep | `docs/prd-v0.5.md`, `docs/PRODUCT_TRUTH.md` | Already preserved | None. |

## 7. `docs/archive/product-history/mvp-validation.md`

| Legacy rule / decision | Decision | Target active doc | Current status | Required action |
|---|---|---|---|---|
| Validation should answer whether SeatLoom materially reduces context-carrying overhead without forcing users to change tools. | Keep with rewrite | Future validation plan aligned to v0.5 | Missing | Create a v0.5 validation artifact after the active contract set is frozen. |
| Quantified success metrics such as context-transfer reduction, recovery time, and replay coverage should survive. | Keep with rewrite | Future validation plan and future acceptance refinement | Partial | Recast the useful metrics against the v0.5 module model and current user stories. |
| Stop-loss conditions are valuable and should not disappear during expansion. | Keep with rewrite | Future validation/governance doc | Missing | Add a stop-or-scope-down section when the validation plan is rewritten. |
| MVP exclusions from the old validation plan should not override the newer P0/P1/P2 contract. | Reject | None | Intentionally superseded | Keep the v0.5 priority boundary as the active scope rule. |

## 8. First-pass migration priorities

| Priority | Carry-forward item | Why it comes first |
|---|---|---|
| P0 | Unify active reference chain on `docs/PRODUCT_TRUTH.md` | Prevent new work from branching into stale authority files. |
| P0 | Upgrade acceptance references away from v0.4 / scenario authority | Verification cannot remain anchored to a superseded contract. |
| P1 | Restore missing multi-project, switch-protection, and focus-loop details into the active interaction/UX/acceptance set | These are still user-visible behaviors with incomplete active definitions. |
| P1 | Add an active lifecycle/state appendix for WorkItem / Session / Handoff | Prevent hidden dependence on v0.4 semantics during implementation and testing. |
| P2 | Rebuild validation and positioning support docs around v0.5 | Important for product confidence, but not the first blocker for contract cleanup. |

## 9. Active cleanup status after this pass

This matrix is intentionally paired with the active-truth cleanup pass that repoints live documents toward `docs/PRODUCT_TRUTH.md`.

The next cleanup pass should focus on two things:

1. explicit migration or rejection of the missing multi-project and lifecycle details listed above,
2. active-spec refinement for lifecycle appendix, project switching detail, and focus behavior completeness.
