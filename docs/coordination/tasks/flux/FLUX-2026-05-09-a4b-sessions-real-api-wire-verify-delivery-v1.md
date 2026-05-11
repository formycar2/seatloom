# Delivery: A4-β SessionsWorkspace Real-API Wire-Up — Layer A Commit-Pinned Verification

[Flux -> Lyra] A4-β Layer A verify — `ab672e5` — PASS (13/13 invariants, 5/5 R-rules).

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification_delivery |
| id | FLUX-2026-05-09-a4b-sessions-real-api-wire-verify-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-05-09 |
| version | v1 |
| to | lyra |
| verify target | `ab672e53e2c55c93d1e13f8ce6b4cbdc07c546cc` |
| packet ref | `FLUX-2026-05-09-a4b-sessions-real-api-wire-verify-v1.md` |
| tags | flux, A4-β, verification, commit-pinned, SG-B-gating |

---

## §2 — Verify-step outputs (verbatim)

```
$ git checkout ab672e5
HEAD is now at ab672e5 feat(ui): A4-β — wire SessionsWorkspace to real tmux IPC

$ git log -1 --format='%H %s'
ab672e53e2c55c93d1e13f8ce6b4cbdc07c546cc feat(ui): A4-β — wire SessionsWorkspace to real tmux IPC (cmd_list/attach_tmux_session)
```

### A.1 — Scope audit

```
$ git show --stat ab672e5
 ui/src/app-v2/panel/SessionsWorkspace.tsx | 74 +++++++++++++------------------
 ui/src/lib/api.ts                         | 13 ++++++
 ui/src/lib/types-dto.ts                   | 11 +++++
 3 files changed, 56 insertions(+), 42 deletions(-)
```

### A.2 — pnpm install

```
$ cd ui && pnpm install --frozen-lockfile
Lockfile is up to date, resolution step is skipped
Already up to date
Done in 402ms
EXIT: 0
```

### A.3 — Backend untouched

```
$ git show ab672e5 -- 'crates/**' 'src-tauri/**'
(empty — PASS)
```

### A.4 — i18n untouched

```
$ git show ab672e5 -- 'ui/src/i18n/**' 'ui/src/**/*.i18n.*'
(empty — PASS)
```

### A.5 — SessionTerminal untouched

```
$ git show ab672e5 -- 'ui/src/app-v2/panel/SessionTerminal.tsx'
(empty — PASS)
```

### A.6 — MOCK removal invariant

```
$ git grep -nE 'MOCK_TMUX_SESSIONS|A3 not yet available|TODO\(A4-β\)' -- ui/
(no matches → PASS)
```

### A.7 — Real IPC presence

```
$ git grep -nE "invoke\('cmd_list_tmux_sessions'\)|invoke\('cmd_attach_tmux_session'" -- ui/src/lib/api.ts
ui/src/lib/api.ts:145:    invoke('cmd_list_tmux_sessions'),
ui/src/lib/api.ts:153:    invoke('cmd_attach_tmux_session', { request }),

$ git grep -nE "api\.listTmuxSessions|api\.attachTmuxSession" -- ui/
ui/src/lib/api.ts:143:    listTmuxSessions: (): Promise<TmuxSessionInfo[]> =>
ui/src/lib/api.ts:146:    attachTmuxSession: (request: {
ui/src/app-v2/panel/SessionsWorkspace.tsx:31:        const list = await api.listTmuxSessions();
ui/src/app-v2/panel/SessionsWorkspace.tsx:56:      const dto = await api.attachTmuxSession({
```

### A.8 — Read-only banner + v0.0.1 pill preserved

```
$ git grep -nE "Read-only mode \(v0.0.1\)" -- ui/src/app-v2/panel/SessionsWorkspace.tsx
ui/src/app-v2/panel/SessionsWorkspace.tsx:186:                ⚠️ Read-only mode (v0.0.1). Typing in this terminal is disabled. Use tmux directly to send commands.

$ git grep -nE "v0.0.1 attach-only · read-only mirror" -- ui/src/app-v2/panel/SessionsWorkspace.tsx
ui/src/app-v2/panel/SessionsWorkspace.tsx:123:            v0.0.1 attach-only · read-only mirror
```

### A.9 — disableStdin preserved

```
$ git grep -nE "disableStdin: true" -- ui/src/app-v2/panel/SessionTerminal.tsx
ui/src/app-v2/panel/SessionTerminal.tsx:66:      disableStdin: true,
```

### A.10 — Wire shape alignment

```
$ git grep -nE "pub struct TmuxSessionInfo|rename_all" -- crates/seatloom-core/src/pty/mod.rs
crates/seatloom-core/src/pty/mod.rs:114:pub struct TmuxSessionInfo {

$ git grep -nE "session_name: string|created_at: number|attached: boolean" -- ui/src/lib/types-dto.ts
ui/src/lib/types-dto.ts:210:  session_name: string;
ui/src/lib/types-dto.ts:211:  created_at: number;
ui/src/lib/types-dto.ts:212:  attached: boolean;

$ git grep -nE 'rename_all = "camelCase"' -- src-tauri/src/commands/session_cmds.rs
src-tauri/src/commands/session_cmds.rs:84:#[serde(rename_all = "camelCase")]
(AttachTmuxRequest at line 84 uses camelCase serialization)
```

### TypeScript + Build

```
$ cd ui && pnpm exec tsc --noEmit
(empty output, zero errors)
EXIT: 0

$ cd ui && pnpm build
vite v6.4.2 building for production...
✓ 1594 modules transformed.
dist/index.html                             0.57 kB
dist/supervisor.html                        0.59 kB
dist/assets/SupervisorPanel-BYlfx6MH.css   47.96 kB
dist/assets/supervisor-CXH-R9FB.js          0.86 kB
dist/assets/main-DPQ-58Bb.js              214.73 kB
dist/assets/SupervisorPanel-Y1qH2-MH.js   352.78 kB
✓ built in 1.73s
EXIT: 0
```

---

## §3 — Static invariant matrix (13/13 PASS)

| # | Invariant | Result | Evidence |
|---|-----------|--------|----------|
| 1 | Scope bounded — 3 files only | PASS | `git show --stat`: SessionsWorkspace.tsx + api.ts + types-dto.ts |
| 2 | Insertion/deletion tally matches | PASS | 56 insertions, 42 deletions |
| 3 | Backend frozen | PASS | `git show ab672e5 -- crates/** src-tauri/**` → empty |
| 4 | i18n frozen | PASS | `git show ab672e5 -- ui/src/i18n/**` → empty |
| 5 | SessionTerminal frozen | PASS | `git show ab672e5 -- ui/src/app-v2/panel/SessionTerminal.tsx` → empty |
| 6 | MOCK_TMUX_SESSIONS removed | PASS | `git grep MOCK_TMUX_SESSIONS` → zero matches |
| 7 | A3-not-yet-available warn removed | PASS | `git grep 'A3 not yet available'` → zero matches |
| 8 | TODO(A4-β) comments removed | PASS | `git grep 'TODO(A4-β)'` → zero matches (all removed from A4-α) |
| 9 | Real IPC wrappers present | PASS | `api.ts:143-153` — `listTmuxSessions` + `attachTmuxSession` |
| 10 | Arg envelope `{ request }` correct | PASS | `api.ts:153`: `invoke('cmd_attach_tmux_session', { request })` — matches `cmd_launch_session` pattern |
| 11 | TmuxSessionInfo wire shape | PASS | `types-dto.ts:210-212`: `session_name/created_at/attached` snake_case — matches Rust `pty/mod.rs:114` (no rename_all on struct) |
| 12 | Read-only banner preserved | PASS | `SessionsWorkspace.tsx:186`: `⚠️ Read-only mode (v0.0.1)...` |
| 13 | v0.0.1 pill preserved | PASS | `SessionsWorkspace.tsx:123`: `v0.0.1 attach-only · read-only mirror` |

---

## §4 — R-rule / compliance matrix (5/5 PASS)

| Rule | Required behavior | Verification | Result |
|------|-------------------|-------------|--------|
| R1 attach-only | UI never calls `cmd_launch_session` | `git grep cmd_launch_session\|launchSession -- SessionsWorkspace.tsx` → zero matches | PASS |
| R3 failure-isolation | close-tab doesn't propagate tmux kill | `closeTab` body (lines 72-78): filters sessions + updates activeId, no `api.killSession` call | PASS (see §5 non-blocker a) |
| v0.0.1 read-only | `disableStdin: true` | `SessionTerminal.tsx:66` — confirmed untouched | PASS |
| Browser-dev degradation | `!isTauri()` branches | `SessionsWorkspace.tsx:26` (list returns []), `:49` (attach shows error message) | PASS |
| Cancellation safety | `cancelled` flag + clearInterval | `SessionsWorkspace.tsx:24` (declare), `:32`/`:34` (guard setState), `:43` (set true on cleanup) | PASS |

---

## §5 — Known non-blockers (confirmed)

| # | Concern | Status | Disposition |
|---|---------|--------|-------------|
| a | Mirror leak on close-tab: `closeTab` (line 72-78) does not call `api.killSession` → FIFO + pipe-pane persist until app exit | **Non-blocker** | A4-α legacy behavior. Does not violate R3 (backend `kill_session` is R3-safe per A3 Layer A.5). Carry-forward to B1 or follow-up cleanup. |
| b | Unused optional fields (`seatId`/`sessionId`) in `attachTmuxSession` type | **Non-blocker** | Reserved for future seat attribution. Current call site omits them. |
| c | Lockfile drift | **Non-blocker** | `pnpm install --frozen-lockfile` exit 0 at `ab672e5`. No drift detected. |

---

## §6 — Layer B note

Runtime verification deferred to Mr. Zhang / tauri-dev seat. Not part of this Layer A delivery. Scope: browser smoke (list → attach → session:output flow → read-only terminal → close-tab).

---

## Verdict

**A4-β Layer A: PASS** — 13/13 static invariants PASS, 5/5 R-rules PASS, tsc/build green, commit-pinned at `ab672e5`.

Blockers: none.

Next action: Lyra acceptance → B1 GO to Nimbus.

---

*Artifact: `docs/coordination/tasks/flux/FLUX-2026-05-09-a4b-sessions-real-api-wire-verify-delivery-v1.md`*