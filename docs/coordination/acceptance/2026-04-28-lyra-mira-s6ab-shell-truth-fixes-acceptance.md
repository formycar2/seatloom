# Acceptance: Mira S6AB Shell Truth Fixes

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-28-mira-s6ab-shell-truth-fixes-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-28 |
| version | v1 |
| target | `docs/coordination/tasks/mira/MIRA-2026-04-28-s6ab-shell-truth-fixes-delivery-v1.md` |
| verdict | PASS |
| tags | acceptance, ui, mira, s6, shell, truth, shortcut |

## Verdict

**PASS**

Lyra accepts Mira's `S6AB` truth-fix delivery and closes the remaining open items from the `S6` interaction-baseline queue.

This acceptance closes:

1. `docs/coordination/tasks/mira/MIRA-2026-04-28-s6ab-shell-truth-fixes-v1.md`
2. the returned `S6A` and `S6B` scope inside `docs/coordination/acceptance/2026-04-28-mira-s6-interaction-baseline-acceptance.md`
3. the overall `S6` queue

## Scope Reviewed

- `docs/coordination/acceptance/2026-04-28-mira-s6-interaction-baseline-acceptance.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6ab-shell-truth-fixes-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s6ab-shell-truth-fixes-delivery-v1.md`
- `ui/src/components/ProjectOverview.tsx`
- `ui/src/App.tsx`
- `ui/src/components/ShortcutHelpDialog.tsx`
- `ui/src/hooks/useGlobalShortcuts.ts`
- `cd ui && pnpm build`

## Coverage Matrix

| # | Requirement | Prototype evidence | Result | Notes |
|---|---|---|---|---|
| 1 | Project Overview health values stay deterministic | `ui/src/components/ProjectOverview.tsx:32`, `ui/src/components/ProjectOverview.tsx:38`, `ui/src/components/ProjectOverview.tsx:41` | PASS | Running sessions, blocked sessions, open WorkItems, pending Inbox, unresolved review threads, and last reconcile all derive from seeded store data. |
| 2 | Unsupported budget health does not pretend to be modeled | `ui/src/components/ProjectOverview.tsx:39`, `ui/src/components/ProjectOverview.tsx:125` | PASS | Budget alert remains explicitly unavailable (`未就绪 / 未索引`) instead of showing a fabricated healthy state. |
| 3 | Inbox-linked running or input-blocked sessions auto-open the terminal panel | `ui/src/App.tsx:209` | PASS | `handleInboxSelect()` now mirrors the existing session drill-through behavior for `Running` and `InputRequired`. |
| 4 | Shortcut help lists only implemented shortcuts | `ui/src/components/ShortcutHelpDialog.tsx:9`, `ui/src/hooks/useGlobalShortcuts.ts:6` | PASS | The help dialog now matches the actual shortcut hook and no longer advertises `Tab`. |
| 5 | Touched shell copy remains Chinese-first | `ui/src/components/ProjectOverview.tsx:48`, `ui/src/components/ProjectOverview.tsx:104`, `ui/src/components/ShortcutHelpDialog.tsx:26` | PASS | Primary labels now lead with Chinese and keep English only as secondary garnish where still useful. |
| 6 | Build stays green after the return packet | `cd ui && pnpm build` | PASS | Lyra re-ran the production build on 2026-04-28 and confirmed success. |

## Findings

No blocking findings remain in the returned `S6AB` scope.

## Gate Decision

- **`S6AB` shell truth fixes:** **GO**
- **`S6A` project overview + shell closure:** **CLOSED**
- **`S6B` shortcut discovery:** **CLOSED**
- **Overall `S6` interaction-baseline queue:** **CLOSED**

## Follow-up Actions

- Mira: stop `S6` work and wait for the next bounded packet.
- Lyra: move the next UI slice to the mobile companion and keep new surfaces on shared theme tokens only.
