# Delivery: A4-β — SessionsWorkspace real-API wire-up (tmux-mirror v0.0.1)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | MIRA-2026-05-09-a4b-sessions-real-api-wire-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-05-09 |
| commit | ab672e53e2c55c93d1e13f8ce6b4cbdc07c546cc |
| packet_ref | MIRA-2026-05-09-a4b-sessions-real-api-wire-v1 |
| branch | track/infra-foundation |

---

[Mira -> Lyra] A4-β wire-up complete.

---

## git show --stat

```
$ git show ab672e5 --stat
commit ab672e53e2c55c93d1e13f8ce6b4cbdc07c546cc
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Sat May 9 20:13:35 2026 +0800

    feat(ui): A4-β — wire SessionsWorkspace to real tmux IPC (cmd_list/attach_tmux_session)

 ui/src/app-v2/panel/SessionsWorkspace.tsx | 74 +++++++++++++------------------
 ui/src/lib/api.ts                         | 13 ++++++
 ui/src/lib/types-dto.ts                   | 11 +++++
 3 files changed, 56 insertions(+), 42 deletions(-)
```

Three files touched, zero others. No backend, no i18n, no unrelated UI.

---

## tsc

```
$ cd ui && pnpm exec tsc --noEmit
(empty — zero errors)
```

---

## pnpm build

```
$ cd ui && pnpm build
> seatloom-ui@0.1.0 build /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/ui
> tsc && vite build

vite v6.4.2 building for production...
transforming...
✓ 1594 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                             0.57 kB │ gzip:   0.34 kB
dist/supervisor.html                        0.59 kB │ gzip:   0.35 kB
dist/assets/SupervisorPanel-BYlfx6MH.css   47.96 kB │ gzip:   9.13 kB
dist/assets/supervisor-CXH-R9FB.js          0.86 kB │ gzip:   0.56 kB
dist/assets/main-DPQ-58Bb.js              214.73 kB │ gzip:  48.97 kB
dist/assets/SupervisorPanel-Y1qH2-MH.js   352.78 kB │ gzip: 102.94 kB
✓ built in 1.50s
```

---

## §4 Compliance checklist

| Rule | Required behavior | Status | Line evidence |
|---|---|---|---|
| R1 attach-only | UI never sends a command that spawns a CLI process. Only `cmd_list_tmux_sessions` + `cmd_attach_tmux_session` called. | PASS | `api.ts:144–155`; no `cmd_launch_session` call in SessionsWorkspace |
| R3 failure-isolation | Close-tab path does not propagate `tmux kill-session`. Uses existing `closeTab()` which removes from local state only; `api.killSession` is available via existing `onSessionExit` path but not called on manual close. | PASS | `SessionsWorkspace.tsx:55–61` (`closeTab`) |
| v0.0.1 read-only | `disableStdin: true` on SessionTerminal — untouched from A4-α. No write-path added. | PASS | `SessionTerminal.tsx` not in this commit's diff |
| read-only banner | `⚠️ Read-only mode (v0.0.1)...` preserved per attached tab. | PASS | `SessionsWorkspace.tsx:108–110` |
| browser-dev degradation | `!isTauri()` → `setTmuxSessions([])` for list; user-visible error string for attach. No crash. | PASS | `SessionsWorkspace.tsx:20–24` (list), `SessionsWorkspace.tsx:44–47` (attach) |

**Wire-shape verification**: `TmuxSessionInfo` Rust struct at `crates/seatloom-core/src/pty/mod.rs:114` has no `#[serde(rename_all)]` — fields serialize as snake_case. TS interface uses `session_name` / `created_at` / `attached` to match. `AttachTmuxRequest` at `src-tauri/src/commands/session_cmds.rs:84` has `#[serde(rename_all = "camelCase")]` — `api.attachTmuxSession` passes `{ tmuxSessionName, rows, cols }` camelCase to match.

---

## Dev-server smoke

Tauri dev unavailable on this seat — Layer B deferred to Flux/Mr. Zhang.

---

## Known limits

None. A4-β is a drop-in wire-up; no carry-forward items.

---

blockers:
- none

verdict:
- PASS

next action:
- wait for Lyra acceptance; Flux Layer A verify follows

artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-05-09-a4b-sessions-real-api-wire-delivery-v1.md
