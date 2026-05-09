# Task: V0.1 Sessions Wiring + Header Truth (Mira Packet)

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-2026-05-09-v01-sessions-live-wiring-v1 |
| status | issued |
| author | aegis |
| date | 2026-05-09 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-05-10 (EOD) |
| depends_on | commit `3581b18` (Mira baseline), `docs/coordination/reviews/2026-05-09-aegis-mvp-gap-to-tmux-replacement.md` (held, but §2 workflow rows guide scope here), `ui/src/app-v2/panel/SessionsWorkspace.tsx`, `ui/src/app-v2/panel/SessionTerminal.tsx`, `ui/src/lib/api.ts` |
| tags | mira, v0.1, sessions, pty, header, tmux-replacement |
| acceptance owner | aegis (direct — Lyra gated per `feedback_hold_coordination_until_mr_zhang_ratifies_direct_seat_work.md`) |
| concurrency rule | Single track. Do not touch SupervisorPanel, mock-data.ts, or any file under `src-tauri/`. |

---

## Why this packet

Mira's 2026-05-09 rework (commit `3581b18`) landed the V2 hybrid shell (vertical sidebar + SupervisionDashboard as ⌘1 default + i18n purification + hover popups + DAG alignment) — accepted as the v0.1 baseline. The remaining user-facing gap to "tmux replacement" is entirely frontend wiring of already-built backend capabilities. This packet closes three concrete gaps, no scope creep.

---

## Scope (three items, ALL must land in the same commit)

### §A — Sessions workspace: prove the launch-→-type-→-stream loop works end-to-end

**File**: `ui/src/app-v2/panel/SessionsWorkspace.tsx` (existing; do not rewrite, only polish + harden).

**What's required**:

1. **Seats rail must use hydrated backend seats**. Currently it calls `api.listSeats()` into local `seats` state on mount. After `useDataStore.hydrateFromBackend()` runs, seats are in the store. Change the rail to read `useDataStore((s) => s.projectData[s.activeProjectId ?? '']?.seats ?? [])` instead of the local `api.listSeats()` call. If the resulting V1 `Seat` shape differs, map the fields you need (`id`, `name`, `capabilities` for suggestion label).

2. **Visible error state** when backend is unreachable. The current error banner is good; verify it still shows if `launch failed:` returns a non-empty string. Style aligned with the red error token used elsewhere in V2.

3. **Session-exited pill shows exit code**. Current tab UI marks exited sessions with a grey dot. Add the integer `exitCode` as text after the seat name — e.g. `Lyra · Shell (zsh) (exit 0)`. The `markExited` callback already carries it; just render it.

4. **Clear-keystroke-before-close warning**. If a tab is killed with the ×  button while the session is still running (not exited), show a small confirm dialog "Kill live session?" with OK/Cancel. Use the same minimal dialog pattern as other confirmations in the codebase (reuse `SwitchProtectionDialog.tsx` styling if straightforward — otherwise a `window.confirm()` is acceptable for v0.1).

**Out of scope for §A**: changes to `SessionTerminal.tsx` internals, xterm theme, PTY backend. The Rust side already emits `session:output` and `session:exit` events; do not touch.

**Acceptance for §A**: from ⌘2 Sessions tab, click "Lyra" in left rail → real `claude`/`gemini` process launches (whatever is on `$PATH`), xterm tab appears, typing works, process output streams, clicking the × on a running session shows a confirm, closing tab of exited session works, the exit code is visible.

### §B — Top-bar header truth: no mock numbers

**File**: `ui/src/app-v2/AppV2.tsx` (the header block currently lines 63–72).

**What's required**:

1. Replace the hard-coded `1 阻塞` and `5 进行中` spans with derived counts from `useDataStore`:
   - 阻塞 count = number of workitems with `status === 'Blocked'` across `projectData[activeProjectId].workItems`
   - 进行中 count = number of workitems with `status === 'Active'`

2. If the count is `0`, hide that pill entirely (don't show "0 阻塞"; show nothing).

3. Wire the current Supervisor IM button: keep it where it is; its click should continue to call `setShowSupervisor(true)`. No changes needed beyond verifying it still works after your rework.

**Out of scope for §B**: new metrics, changes to the supervisor popover layout, touching `SupervisionDashboard` counters (they compute their own numbers correctly).

**Acceptance for §B**: with the seeded database (9 workitems all `Done`), both pills are hidden (empty-state visually clean); after mutating a workitem to `Blocked` via psql (`UPDATE workitems SET status='blocked' WHERE id='wi-001'`) and hitting reload in the dev window, the 阻塞 pill re-appears with count 1.

### §C — Docs workspace: manual reconcile button

**File**: `ui/src/app-v2/views/DocumentsWorkspace.tsx` (existing; add a button to the filter rail only).

**What's required**:

1. In the filter rail at the top, beside the search input, add a button labelled **"🔄 同步文档"** (use the same vocabulary style Mira chose in commit `3581b18`). Disabled while a reconcile is in flight.

2. On click, call `api.reconcile()`. Show a small toast-style inline banner below the button with the summary: `已扫描 N · 新增 X · 更新 Y · 未变 Z · 失败 F · 冲突 C`. If `F > 0` or `C > 0`, use error token color.

3. After reconcile returns success, call `useDataStore.getState().hydrateFromBackend()` to refresh the artifact list in place. No page reload.

**Out of scope for §C**: file-system watcher, automatic reconcile on focus, background polling. The button is manual-only for v0.1.

**Acceptance for §C**: click button, banner shows numbers within 2 s, artifact list count matches `SELECT count(*) FROM documents` in Postgres within the refreshed view.

---

## Required read order (before writing any code)

1. This packet.
2. `ui/src/lib/api.ts` — existing `api.listSeats / launchSession / reconcile` signatures.
3. `ui/src/app-v2/panel/SessionsWorkspace.tsx` — current 300+ lines you will polish, not replace.
4. `ui/src/app-v2/views/DocumentsWorkspace.tsx` — filter rail structure for §C placement.
5. `ui/src/stores/useDataStore.ts` — `hydrateFromBackend` signature.
6. `ui/src/app-v2/AppV2.tsx` lines 63–72 — the header block to surgically update.

---

## Validation (run before reporting delivery)

```bash
cd ui && pnpm exec tsc --noEmit    # must be zero errors
cd ui && pnpm build                # must succeed
```

Then `pnpm tauri dev` from repo root and exercise all three acceptance sequences above. Screenshot each.

---

## Out of scope (hard line)

- **SupervisorPanel**: untouched. No edits to `ui/src/app-v2/panel/SupervisorPanel.tsx`, `ui/src/app-v2/hooks/useSupervisorData.ts`, or any IM message / contact rendering.
- **mock-data.ts**: untouched. The data layer is now backend-driven; mocks remain only as fallback.
- **Any Rust / Tauri command** (`src-tauri/**`): untouched. All commands needed already exist.
- **i18n dictionary**: your previous commit settled this; do not expand scope into vocabulary.
- **SupervisionDashboard** counter logic: already computes from live data; do not change.

If any acceptance above requires touching an out-of-scope file, **stop and report the conflict**. Do not widen scope.

---

## Delivery

1. One commit on `track/infra-foundation`. Commit message: `feat(ui): Mira v01 sessions wiring + header truth + docs reconcile (MIRA-2026-05-09-v01)`.
2. Write a delivery packet: `docs/coordination/tasks/mira/MIRA-2026-05-09-v01-sessions-live-wiring-delivery-v1.md` with the commit hash, per-section outcome, and the three acceptance screenshots (or recorded observations if screenshots are hard).

---

## Reporting format (for tmux dispatch to Aegis)

```text
[Mira -> Aegis] V0.1 Sessions + Header + Reconcile delivery
branch: track/infra-foundation
commit: <new hash>
completed:
- §A SessionsWorkspace: seats-from-store wired, exit-code pill, kill-confirm → <one line on observed flow>
- §B AppV2 header counters: blocked=N active=M (hidden when 0)
- §C DocumentsWorkspace reconcile button: banner + hydrate refresh working
validation:
- pnpm exec tsc --noEmit => zero errors
- pnpm build => OK
- pnpm tauri dev browser smoke: §A end-to-end PASS / §B PASS / §C PASS
blockers:
- none OR ...
next action:
- wait for Aegis review. Do NOT push for Lyra's dispatch — review is held.
artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-05-09-v01-sessions-live-wiring-delivery-v1.md
```

---

*Packet issued by Aegis · 2026-05-09 · Direct supervision of Mira during v0.1 frontend consolidation.*
