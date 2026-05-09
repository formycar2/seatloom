# Delivery: SessionsWorkspace Attach UI (SG-A §A4-α, v0.0.1)

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-05-09-v001-sessions-attach-ui-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-05-09 |
| version | v1 |
| to | lyra |
| task_ref | MIRA-2026-05-09-v001-sessions-attach-ui-v1 |
| branch | track/infra-foundation |

---

[Mira -> Lyra] SessionsWorkspace Attach UI (A4-α)

commit:
- `bdac54b1f0b16e25724c985c0749fad5a46cbc91`

completed:
- Replaced quick-launcher buttons with "Attach to tmux" dropdown
- Mock `MOCK_TMUX_SESSIONS` array (3 seats) loaded on mount + 5s refresh interval
- `attachToTmux()` adds tab with `console.warn` mock; `TODO(A4-β)` markers at both invoke sites
- `SessionTerminal`: `disableStdin: true`, `cursorBlink: false`, keystroke forwarding removed
- Read-only banner above each terminal pane

---

## Changes

```
$ git show bdac54b --stat
commit bdac54b1f0b16e25724c985c0749fad5a46cbc91
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Sat May 9 18:14:16 2026 +0800

    feat(ui): A4-α — SessionsWorkspace tmux attach UI with mock data (MIRA-2026-05-09-v001)

 ui/src/app-v2/panel/SessionTerminal.tsx   |  18 +-
 ui/src/app-v2/panel/SessionsWorkspace.tsx | 279 +++++++++++-------------------
 2 files changed, 108 insertions(+), 189 deletions(-)
```

---

## Verification

### TypeScript check

```
$ cd ui && pnpm exec tsc --noEmit
EXIT:0
```

(Empty stdout. Exit code 0. Zero errors.)

### Build check

```
$ cd ui && pnpm build
✓ 1593 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                             0.57 kB │ gzip:   0.34 kB
dist/supervisor.html                        0.59 kB │ gzip:   0.35 kB
dist/assets/SupervisorPanel-9gp73Lcn.css   47.83 kB │ gzip:   9.11 kB
dist/assets/supervisor-C4_3_YTy.js          0.86 kB │ gzip:   0.56 kB
dist/assets/main-CBKS7KUp.js              213.04 kB │ gzip:  48.36 kB
dist/assets/SupervisorPanel-9qPoZxQl.js   351.90 kB │ gzip: 102.63 kB
✓ built in 1.83s
```

### UI observations (A4-α mock mode)

1. ⌘2 Sessions tab opens `SessionsWorkspace`. Left rail shows "Attach to tmux" heading, a `<select>` dropdown, and an "Attach" button — quick-launcher buttons are gone.
2. Dropdown lists three mock sessions: `Lyra-po-seatloom`, `Nimbus-TechArchi-seatloom`, `Mira-UX/UED-seatloom`.
3. "Attach" button is disabled while no session is selected; enabled after selection.
4. Clicking "Attach" with a session selected: button disables during flight, a new tab appears labeled `tmux: <session_name>`, dropdown resets to placeholder.
5. Tab strip shows green dot + session label + `×` close button.
6. Terminal pane shows amber read-only banner: "⚠️ Read-only mode (v0.0.1). Typing in this terminal is disabled. Use tmux directly to send commands."
7. Typing in the xterm area produces no output (disableStdin: true).
8. `×` closes the tab immediately (no confirm needed — mock sessions have no live process).
9. `console.warn` fires: `A3 not yet available; mock attach for <session_name>`.
10. Both `TODO(A4-β)` markers visible in diff at the `cmd_list_tmux_sessions` and `cmd_attach_tmux_session` call sites.

---

blockers:
- none

verdict:
- PASS

next action:
- wait for Lyra acceptance

artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-05-09-v001-sessions-attach-ui-delivery-v1.md
