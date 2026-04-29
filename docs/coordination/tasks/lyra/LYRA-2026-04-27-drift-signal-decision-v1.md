# LYRA-2026-04-27-drift-signal-decision-v1

| Field | Value |
|---|---|
| Owner | Lyra |
| Status | Active decision |
| Audience | Mira |
| Topic | Drift signal placement and noise policy |

## 1. Decisions needed

1. Should `drifted` WorkItems surface only in Inbox, or also as a separate floating alert?
2. If a transient alert exists, when is it allowed?

## 2. Decisions made

1. **Do not use a standalone floating `Project drift detected` alert as the primary or persistent surface.**
2. **Inbox is the canonical action surface for drifted WorkItems.** This follows `docs/archive/product-history/prd-v0.4.md` Inbox rules.
3. **A transient reconcile-result notification is allowed only as a short-lived summary after an explicit reconcile flow**, not as a per-item drift alert.
4. **Startup auto-reconcile should use Morning Digest + Inbox only.** No extra floating drift toast on launch.

## 3. Rationale

- `docs/archive/product-history/prd-v0.4.md` places `drifted` WorkItems into Inbox as low-priority actionable items.
- `docs/archive/product-history/mvp-scenarios.md` shows drift in Inbox and Morning Digest, and only defines a brief reconcile-result notification with `View in Inbox` / `Dismiss`.
- A dedicated floating drift card duplicates the same signal, increases noise, and weakens Inbox as the single action queue.

## 4. UI contract implication

Use this rule:

- `drifted` item detected -> create/update Inbox item
- startup reconcile -> show Morning Digest summary at Inbox top
- manual reconcile or pre-pipeline reconcile -> optional short-lived summary notice with aggregate issue count and CTA `View in Inbox`
- do **not** show a standalone per-drift floating card like `Project drift detected!`

## 5. Exact guidance for Mira

1. Remove the current standalone floating drift card pattern.
2. If you keep a notification component, repurpose it into a **reconcile summary** component, not a drift alert component.
3. Notification copy should be aggregate and neutral, for example: `Reconciliation: 2 issues found`.
4. Primary CTA should route to Inbox, not to a duplicate detail flow.
5. Drift detail remains in Inbox + Detail Pane, not in the toast itself.

## 6. File targets

- `ui/src/components/ReconcileNotification.tsx`
- `ui/src/i18n.ts`
- any caller that auto-shows a standalone drift alert

## 7. Blockers

- None. This is a product clarification and may be applied immediately.
