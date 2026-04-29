# Acceptance: Mira S6 Interaction Baseline Closure

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-mira-s6-interaction-baseline-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/mira/MIRA-2026-04-28-s6a-project-overview-shell-closure-delivery-v1.md`, `docs/coordination/tasks/mira/MIRA-2026-04-28-s6b-shortcut-discovery-delivery-v1.md`, `docs/coordination/tasks/mira/MIRA-2026-04-28-s6c-deterministic-list-search-delivery-v1.md` |
| verdict | CONDITIONAL PASS |
| tags | acceptance, ui, mira, s6, shell, shortcut, search |

## Verdict

**CONDITIONAL PASS**

Lyra accepts only the deterministic list-search slice from the current `S6` queue and returns the other two slices for one bounded truth-fix follow-up.

Packet-level decision:

1. `S6C` — **PASS / CLOSED**
2. `S6A` — **HOLD / RETURNED**
3. `S6B` — **HOLD / RETURNED**

This means:

- the search baseline is accepted and does not need rework,
- the shell-closure queue remains open,
- no Nimbus or Flux handoff is needed for this slice yet, and
- Mira receives one serial micro-fix packet to close the remaining gaps.

## Scope Reviewed

- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6a-project-overview-shell-closure-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6b-shortcut-discovery-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6c-deterministic-list-search-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6a-project-overview-shell-closure-delivery-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6b-shortcut-discovery-delivery-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6c-deterministic-list-search-delivery-v1.md`
- `ui/src/App.tsx`
- `ui/src/components/ProjectOverview.tsx`
- `ui/src/components/ShortcutHelpDialog.tsx`
- `ui/src/hooks/useGlobalShortcuts.ts`
- `ui/src/layouts/StatusBar.tsx`
- `ui/src/views/InboxView.tsx`
- `ui/src/views/WorkItemsView.tsx`
- `cd ui && pnpm build`

## Packet Result Summary

| Packet | Verdict | Result | Notes |
|---|---|---|---|
| `S6A` | HOLD | Returned | Default detail is fixed, but Overview truth still contains fabricated values and Inbox session drill-through still misses terminal auto-open. |
| `S6B` | HOLD | Returned | Help entry is discoverable, but the dialog still lists unsupported `Tab`. |
| `S6C` | PASS | Closed | Deterministic local filtering is implemented correctly on both required list surfaces. |

## Coverage Matrix

| # | Requirement | Prototype evidence | Result | Notes |
|---|---|---|---|---|
| 1 | Default Detail state shows a real `Project Overview` surface | `ui/src/App.tsx:307`, `ui/src/App.tsx:374` | PASS | The Detail pane now falls back to `ProjectOverview` instead of a blank state. |
| 2 | Running or prompt-blocked Session selection from standard object drill-through auto-opens live evidence | `ui/src/App.tsx:145` | PASS | `handleSelectObject()` opens the terminal for `Running` and `InputRequired` sessions. |
| 3 | Running or prompt-blocked Session selection from Inbox also auto-opens live evidence | `ui/src/App.tsx:184` | FAIL | `handleInboxSelect()` selects the session but does not open the terminal panel. |
| 4 | Project Overview health values stay deterministic and seeded from current prototype data | `ui/src/components/ProjectOverview.tsx:36`, `ui/src/components/ProjectOverview.tsx:41` | FAIL | `unresolvedThreads` and `budgetAlert` are hardcoded; this breaks the packet rule against fabricated shell truth. |
| 5 | Shortcut help is discoverable from shell chrome | `ui/src/layouts/StatusBar.tsx:38`, `ui/src/hooks/useGlobalShortcuts.ts:8` | PASS | The status bar hint and `?` / `F1` hook are both present. |
| 6 | Shortcut help lists only real supported shortcuts | `ui/src/components/ShortcutHelpDialog.tsx:9`, `ui/src/hooks/useGlobalShortcuts.ts:8` | FAIL | `Tab` is displayed in help, but no `Tab` shortcut is implemented. |
| 7 | Inbox supports deterministic local text filtering with visible counts and no-match state | `ui/src/views/InboxView.tsx:21`, `ui/src/views/InboxView.tsx:61`, `ui/src/views/InboxView.tsx:81` | PASS | Filtering, count display, and empty-state recovery all match the packet scope. |
| 8 | WorkItems supports deterministic local text filtering with visible counts and no-match state | `ui/src/views/WorkItemsView.tsx:18`, `ui/src/views/WorkItemsView.tsx:58`, `ui/src/views/WorkItemsView.tsx:109` | PASS | Filtering, count display, and clear reset path are in place. |
| 9 | Current build remains green after the S6 deliveries | `cd ui && pnpm build` | PASS | Lyra re-ran the build on 2026-04-28 and confirmed success. |

## Findings

### High

1. `ui/src/components/ProjectOverview.tsx:41` still fabricates shell-health truth with hardcoded values (`unresolvedThreads = 3`, `budgetAlert = false`) instead of deriving them from seeded project data or explicitly showing an unavailable state. This violates the packet's deterministic requirement and weakens operator trust in the default shell.

### Medium

1. `ui/src/App.tsx:209` selects Inbox-linked sessions without the terminal auto-open behavior already implemented in `handleSelectObject()`. This leaves one extra step on the exact running-session drill-through path that the packet was meant to close.
2. `ui/src/components/ShortcutHelpDialog.tsx:15` lists `Tab` as a supported shortcut even though `ui/src/hooks/useGlobalShortcuts.ts` does not implement it. The help dialog must not advertise future or guessed shortcuts.

### Low

1. `ui/src/components/ProjectOverview.tsx:51`, `ui/src/components/ProjectOverview.tsx:100`, and `ui/src/components/ProjectOverview.tsx:144` still mix several English shell headings into a surface that is otherwise Chinese-first. This is not a gate blocker by itself, but it should be cleaned while the truth-fix packet is open.

## Required Fixes Before Closure

Mira must close the returned scope in:

- `ui/src/components/ProjectOverview.tsx`
- `ui/src/App.tsx`
- `ui/src/components/ShortcutHelpDialog.tsx`

Exact fixes required:

1. Replace fabricated `Project Overview` values with deterministic projections from the existing seeded data model, or show an explicit `not yet indexed / unavailable` state when the model does not support a truthful metric yet.
2. Extend the running/input-blocked session terminal auto-open behavior to the Inbox selection path.
3. Remove unsupported `Tab` from shortcut help unless the shortcut is actually implemented in the same packet.
4. Clean the touched `Project Overview` labels so the surface stays Chinese-first.

## Gate Decision

- **`S6C` deterministic list search:** **GO / CLOSED**
- **`S6A` project overview + shell closure:** **HOLD**
- **`S6B` shortcut discovery:** **HOLD**
- **Overall `S6` interaction-baseline closure queue:** **OPEN**

## Follow-up Actions

- Mira: complete the truth-fix follow-up packet and stop for acceptance.
- Lyra: issue the bounded return packet immediately and keep the `S6` queue serial until the two remaining gaps are closed.
