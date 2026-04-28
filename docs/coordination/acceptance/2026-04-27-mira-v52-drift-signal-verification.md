# 2026-04-27 Mira V5.2 Drift Signal Verification

| Field | Value |
|---|---|
| Owner | Lyra |
| Seat reviewed | Mira |
| Scope | V5.2 drift signal update |
| URL verified | `http://localhost:5173/` |
| Status | Conditional pass |

## 1. Verdict

**CONDITIONAL PASS**

The V5.2 update resolves the main noise issue:
- the standalone floating `Project drift detected` alert is gone;
- drift remains canonicalized in Inbox/WorkItems;
- the replacement notification is now an aggregate reconcile summary;
- the summary auto-hides and the CTA returns the user to Inbox.

One contract issue remains: the summary is currently fired from the initialization flow, not from an explicit reconcile flow.

## 2. What was verified

### Passed checks

1. **No standalone drift alert on initial load**
   - Initial load at `http://localhost:5173/` showed Morning Summary + Inbox only.
   - No floating `Project drift detected` card was present.

2. **Transient summary behavior works**
   - After `Init Demo` -> `Initialize`, a floating aggregate summary appears.
   - Summary copy is aggregate: `Reconciliation: 2 issues found`.
   - The summary auto-hides after approximately 5 seconds.

3. **CTA behavior works**
   - From `Project Goal`, clicking `View in Inbox` returned the UI to Inbox.
   - The summary was dismissed after CTA.

## 3. Contract issue

### Medium

**Summary trigger is attached to init, not to an explicit reconcile flow.**

- Current caller: `ui/src/App.tsx:225`
- Current behavior: `Init Demo` -> `Initialize` immediately triggers:
  - `setReconcileIssueCount(2)`
  - `setShowReconcileSummary(true)`
- This conflicts with the frozen drift policy in `docs/coordination/tasks/lyra/LYRA-2026-04-27-drift-signal-decision-v1.md`:
  - startup/init should use **Morning Digest + Inbox only**;
  - a floating summary is allowed only after an **explicit reconcile flow**.

## 4. Required fix for Mira

1. **Remove the init-triggered summary call path**
   - Target: `ui/src/App.tsx:225`
   - Do not show the reconcile summary directly from `InitDialog.onInitialize`.

2. **Keep the summary component, but reserve it for a true explicit reconcile trigger**
   - Component may remain in:
     - `ui/src/components/ReconcileNotification.tsx`
     - `ui/src/i18n.ts`
   - Acceptable future triggers:
     - manual reconcile action;
     - pre-pipeline reconcile action.

3. **If no true explicit reconcile entry point exists yet, leave the component dormant**
   - Better to have no summary trigger than to violate the startup/init noise policy.

## 5. Recommendation

- **No-Go for closing the V5.2 drift-signal packet as accepted until the init trigger is removed or re-routed.**
- After the caller fix in `ui/src/App.tsx`, this can be rechecked quickly.

## 6. Evidence

### DOM/state capture
- `.local/evidence/2026-04-27/verify-v52-results.json`

### Screenshots
- `.local/evidence/2026-04-27/verify-v52-initial.png`
- `.local/evidence/2026-04-27/verify-v52-workitems.png`
- `.local/evidence/2026-04-27/verify-v52-init-dialog.png`
- `.local/evidence/2026-04-27/verify-v52-summary-after-init.png`
- `.local/evidence/2026-04-27/verify-v52-after-autohide.png`
- `.local/evidence/2026-04-27/verify-v52-second-summary.png`
- `.local/evidence/2026-04-27/verify-v52-after-view-inbox.png`
