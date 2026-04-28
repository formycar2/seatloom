# Flux Acting-Mira Design Suggestion Review

| Field | Value |
|---|---|
| Owner | Lyra |
| Date | 2026-04-27 |
| Source reviewed | Last 100 lines from `Flux-Quality&Ops-seatloom` tmux seat |
| Basis | `docs/prd-v0.4.md`, `docs/interaction-spec-v1.0.md`, `docs/acceptance-spec-v1.0.md`, `docs/mvp-scenarios.md`, current UI code |

## Executive verdict

Flux shows good contract-reading instincts and useful QA judgment, but this sample also shows that he sometimes jumps from observation to redesign before verifying the current implementation and the active product contract.

**Role fit recommendation:**
- Strong fit for acting UX reviewer / contract-gap spotter
- Safe for constrained contract-repair work under Lyra packet control
- Not yet safe to independently redefine information architecture or phase boundaries

## Findings

### High

1. **Suggestion #1 conflicts with the active product contract and should not be adopted.**  
   Flux proposed removing the Sidebar's broader navigation role and turning it into a Seat monitor only. The active contract explicitly requires Sidebar to expose Inbox count, Seats, Sessions, and WorkItems while Main Panel keeps Inbox/Timeline/WorkItems views. Reworking this now would create new contract drift, not reduce it.  
   References: `docs/prd-v0.4.md:83`, `docs/ux-spec.md:124`, `docs/ux-spec.md:176`, `docs/mvp-scenarios.md:63`

2. **Suggestion #2 is correct and should be adopted as a P0 contract repair.**  
   Flux is right that Terminal should not be modeled as a fourth full-page tab. The active contract requires a persistent bottom Terminal Panel that can be toggled independently and preserve session state. Current code still treats Terminal as a main-panel view.  
   References: `docs/prd-v0.4.md:103`, `docs/ux-spec.md:127`, `docs/mvp-scenarios.md:66`, `ui/src/layouts/TopNav.tsx:15`, `ui/src/App.tsx:124`, `ui/src/layouts/AppShell.tsx:35`

3. **Suggestion #5 identifies a real gap, but the diagnosis is imprecise. Adopt the underlying issue, not the wording.**  
   Timeline filtering is incomplete against the contract. Current code supports one type selector and free-text search, but it does not satisfy the required Seat / WorkItem / type / time filtering contract, nor the Sidebar-to-Timeline filter linkage.  
   References: `docs/prd-v0.4.md:236`, `docs/interaction-spec-v1.0.md:145`, `docs/acceptance-spec-v1.0.md:44`, `ui/src/views/TimelineView.tsx:12`, `ui/src/layouts/Sidebar.tsx:42`, `ui/src/layouts/Sidebar.tsx:107`

4. **Suggestion #7 is correct and should be adopted, at least as a truthfulness cleanup.**  
   `MorningDigest` still contains hard-coded claims that are not derived from current project data. Even in prototype mode, truth-like copy should not claim facts that the system does not compute.  
   References: `docs/mvp-scenarios.md:565`, `docs/acceptance-spec-v1.0.md:47`, `ui/src/components/MorningDigest.tsx:29`, `ui/src/components/MorningDigest.tsx:45`

### Medium

5. **Suggestion #3 misreads the current implementation and should not be adopted as stated.**  
   Flux argued that the right-side Detail Pane wastes 380px by default, but the current app does not render the pane when nothing is selected. This is not the current problem. A default project summary could be a future enhancement, but it is not a contract fix and should not displace active P0 work.  
   References: `ui/src/App.tsx:154`, `ui/src/App.tsx:206`, `ui/src/layouts/AppShell.tsx:47`

6. **Suggestion #4 is directionally useful, but it is a P1 UX refinement rather than an immediate blocker.**  
   Inbox actions are currently hover-hidden, which hurts scanability. However, the more important contract gap is that Inbox rows are not wired to open Detail on selection, which Flux did not call out. We should adopt the visibility improvement after the selection/detail behavior is corrected.  
   References: `docs/interaction-spec-v1.0.md:133`, `docs/prd-v0.4.md:237`, `ui/src/components/InboxItem.tsx:37`, `ui/src/views/InboxView.tsx:37`

7. **Suggestion #6 is partially inaccurate and should not be adopted as stated.**  
   Escape handling does exist globally, so the claim that there is no implementation is incorrect. That said, keyboard coverage is still incomplete overall, especially for the documented `Ctrl+1/2/3` and `Ctrl+\`` paths, and not every dialog state is clearly covered. The right follow-up is a broader shortcut audit, not this narrow claim.  
   References: `docs/interaction-spec-v1.0.md:22`, `docs/mvp-scenarios.md:72`, `ui/src/hooks/useGlobalShortcuts.ts:3`

### Low

8. **Suggestion #8 is accurate but should be treated as an integration note, not a design objection.**  
   After the init-triggered path was removed, `ReconcileNotification` now has no active caller. That is acceptable until an explicit reconcile flow is wired in. This is not a reason to restore noisy startup behavior.  
   References: `ui/src/App.tsx:39`, `ui/src/components/ReconcileNotification.tsx:11`

## Recommendation by suggestion

| # | Flux suggestion | Decision | Priority | Notes |
|---|---|---|---|---|
| 1 | Simplify Sidebar to Seat monitor only | Reject | — | Conflicts with active product contract |
| 2 | Restore Terminal as bottom collapsible panel | Adopt | P0 | Contract repair; required before SG-01 can clear |
| 3 | Show default detail/project summary | Reject for now | P2 | Optional enhancement, not current blocker |
| 4 | Keep Inbox row actions visible | Adopt later | P1 | Do after Inbox selection/detail contract is fixed |
| 5 | Make Timeline filters real | Adopt partially | P0 | Reframe as full contract-required filtering |
| 6 | Add Esc binding | Reject as stated | P1 follow-up | Replace with broader keyboard shortcut audit |
| 7 | Remove hard-coded Morning Digest claims | Adopt | P1 | Immediate truthfulness cleanup; Nimbus later wires full data |
| 8 | Reconcile notification now has no caller | Accept as note | P2 | Expected until explicit reconcile flow is wired |

## Capability assessment

### What Flux did well
- Spotted real structural drift around Terminal placement
- Correctly challenged fake truth signals in `MorningDigest`
- Showed awareness that some changes need PO approval before structural edits
- Distinguished between immediate fixes and larger design moves

### Where Flux needs guardrails
- He over-recommends information-architecture changes without first checking the frozen contract
- He sometimes states implementation gaps too strongly without verifying the code path
- He missed one of the more important current contract gaps: Inbox row selection does not open Detail
- His phase labeling is unreliable in this sample; Terminal placement is not a Phase 2/3 nice-to-have, it is current-contract scope

## Lyra decision

Use Flux as the temporary Mira replacement **for constrained contract-repair execution only**.

Approved adoption set for Flux to carry forward:
1. Restore Terminal to a bottom collapsible panel, not a top-level tab
2. Complete real contract-required Timeline filtering (Seat / WorkItem / type / time)
3. Remove or neutralize hard-coded Morning Digest truth claims
4. Improve Inbox action visibility **after** Inbox selection-to-Detail behavior is corrected

Do **not** authorize Flux to redesign Sidebar/Main information architecture without a separate product decision.
