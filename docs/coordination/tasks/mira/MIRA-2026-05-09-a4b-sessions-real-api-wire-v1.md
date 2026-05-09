# Task: A4-β — SessionsWorkspace real-API wire-up (tmux-mirror v0.0.1)

[Lyra -> Mira] A3 已 UNCONDITIONAL PASS (acceptance LYRA-2026-05-09-nimbus-a3-tmux-attach-read-path-acceptance-v1 @ commit fca4fe0). Now wire SessionsWorkspace's mock tmux list + attach path to the real Tauri IPC commands Nimbus landed in A3.

| Field | Value |
|---|---|
| template | T4 |
| subtype | implementation |
| id | MIRA-2026-05-09-a4b-sessions-real-api-wire-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-05-09 |
| to | mira |
| depends_on | `docs/coordination/acceptance/LYRA-2026-05-09-nimbus-a3-tmux-attach-read-path-acceptance-v1.md` (A3 UNCONDITIONAL PASS @ fca4fe0); Aegis dispatch directive 2026-05-09 evening |
| flux verify packet | Flux will issue A4-β Layer A commit-pinned verify after Mira delivery |
| delivery path | `docs/coordination/tasks/mira/MIRA-2026-05-09-a4b-sessions-real-api-wire-delivery-v1.md` |
| tags | mira, A4-beta, tmux-mirror, real-api, SG-B-prep, v0.0.1 |

---

## 1. Context

A4-α (commit `bdac54b`) shipped the SessionsWorkspace attach UI with `MOCK_TMUX_SESSIONS` + `console.warn("A3 not yet available; mock attach...")` stubs. A3 (commit `fca4fe0`) has now landed with the full read-path backend and passed SG-A UNCONDITIONAL. The remaining step is to delete the mocks and wire the UI to the real IPC commands. Everything else in SessionsWorkspace (layout, tab strip, read-only banner, SessionTerminal mount, `v0.0.1 attach-only · read-only mirror` notice) stays as-is.

## 2. In-scope files

- **Primary**: `ui/src/app-v2/panel/SessionsWorkspace.tsx` (replace mock with real IPC)
- **Type surface**: `ui/src/lib/types-dto.ts` (add `TmuxSessionInfo` interface — mirrors Rust `TmuxSessionInfo` at `crates/seatloom-core/src/pty/mod.rs:114`)
- **API surface**: `ui/src/lib/api.ts` (add `listTmuxSessions` + `attachTmuxSession` wrappers)

**Out of scope** (do NOT touch):
- `ui/src/lib/api.ts` lines 132–140 (`ptyWrite`, `ptyWriteBytes`, `killSession`, `ptyResize`) — already correct, `ptyWrite`/`ptyWriteBytes` are v0.0.1 no-ops per R1
- `crates/seatloom-core/**`, `src-tauri/**` — backend frozen at A3 scope
- `SessionTerminal.tsx`, `disableStdin` — already read-only per A4-α
- Any write-path / send-keys behavior (that's B1 territory)

## 3. Implementation steps

### Step 1 — add `TmuxSessionInfo` to `ui/src/lib/types-dto.ts`

Append under the `// --- Live PTY session types (Phase 4) ---` block (around line 202):

```ts
/**
 * Discovered tmux session info (from cmd_list_tmux_sessions).
 * Mirrors Rust `crates/seatloom-core/src/pty/mod.rs:114 TmuxSessionInfo`.
 * `attached` = at least one tmux client is attached to this session.
 */
export interface TmuxSessionInfo {
  sessionName: string;     // from serde `session_name`
  createdAt: number;       // Unix epoch seconds (from tmux `#{session_created}`)
  attached: boolean;       // from serde `attached`
}
```

**Wire-name note**: The Rust struct uses `session_name` / `created_at` but `AttachTmuxRequest` is `#[serde(rename_all = "camelCase")]` at `src-tauri/src/commands/session_cmds.rs:84`. Check `TmuxSessionInfo` derive in `crates/seatloom-core/src/pty/mod.rs` — if it's snake_case on the wire, change the TS interface to `session_name`/`created_at` to match the wire shape. Take the runtime proof (next step) as truth, not my guess.

### Step 2 — add IPC wrappers to `ui/src/lib/api.ts`

In the `api` object, under the `// --- Live PTY sessions (Phase 4) ---` block (currently ending at line 140), add two new methods before the closing `};`:

```ts
  // --- tmux-mirror (v0.0.1 A3) ---
  listTmuxSessions: (): Promise<TmuxSessionInfo[]> =>
    invoke('cmd_list_tmux_sessions'),
  attachTmuxSession: (request: {
    seatId?: string;
    tmuxSessionName: string;
    sessionId?: string;
    rows?: number;
    cols?: number;
  }): Promise<LiveSessionDto> =>
    invoke('cmd_attach_tmux_session', { request }),
```

Note the `{ request }` wrapper on `cmd_attach_tmux_session` — matches existing `cmd_launch_session` pattern at line 131 (Tauri's `serde` arg envelope).

Also add `TmuxSessionInfo` to the top-of-file imports from `./types-dto`:

```ts
import type { /* ...existing... */, TmuxSessionInfo } from './types-dto';
```

### Step 3 — rewrite SessionsWorkspace's list + attach

In `ui/src/app-v2/panel/SessionsWorkspace.tsx`:

1. **Delete** `MOCK_TMUX_SESSIONS` (lines 21–26).
2. **Delete** the local `TmuxSessionInfo` interface at lines 11–15 (replaced by canonical import from `types-dto`).
3. **Import** the canonical type: add `TmuxSessionInfo` + `LiveSessionDto` to the existing `types-dto` import; also add `api` to the existing `api` import.
4. **Replace** the `useEffect` at lines 36–45 with a real IPC fetch, browser-dev fallback preserved:

```tsx
useEffect(() => {
  let cancelled = false;
  const fetchSessions = async () => {
    if (!isTauri()) {
      setTmuxSessions([]);
      return;
    }
    try {
      const list = await api.listTmuxSessions();
      if (!cancelled) setTmuxSessions(list);
    } catch (e) {
      if (!cancelled) {
        console.warn('cmd_list_tmux_sessions failed:', e);
        setTmuxSessions([]);
      }
    }
  };
  fetchSessions();
  const interval = setInterval(fetchSessions, 5000);
  return () => {
    cancelled = true;
    clearInterval(interval);
  };
}, []);
```

5. **Replace** the mock attach (line 55–73) with a real `api.attachTmuxSession` call:

```tsx
const attachToTmux = async (tmuxSessionName: string) => {
  if (!isTauri()) {
    setError('Tauri backend not available — run `pnpm tauri dev`.');
    return;
  }
  setLaunching(true);
  setError(null);
  try {
    const dto = await api.attachTmuxSession({
      tmuxSessionName,
      rows: 30,
      cols: 120,
    });

    const label = `tmux: ${tmuxSessionName}`;
    setSessions((prev) => [...prev, { ...dto, label }]);
    setActiveId(dto.id);
    setSelectedTmuxSession('');
  } catch (e) {
    setError(`Failed to attach to tmux session: ${String(e)}`);
  } finally {
    setLaunching(false);
  }
};
```

6. **Delete** the `console.warn('A3 not yet available; mock attach for ...')` line.

7. **Preserve unchanged**: read-only banner text (line 196), the v0.0.1 pill (line 133), tab strip, close-tab logic, empty-state panel. All browser-mode guards keep returning empty arrays / the existing error message.

### Step 4 — type-check + build

From repo root:

```bash
cd ui
pnpm exec tsc --noEmit    # must exit 0, zero errors
pnpm build                # must exit 0
```

Paste both verbatim into delivery doc.

### Step 5 — dev-server smoke (optional, best-effort)

If you have a seat with `pnpm tauri dev` available, launch once and capture:
- DevTools console after mount: expect no `A3 not yet available` warning; expect `cmd_list_tmux_sessions` to return the 7-session array (your seat included).
- Click Attach on any session: SessionTerminal mounts; `session:output` events flow; read-only banner visible.
- Close tab; `api.killSession` fires (already wired via existing `onSessionExit` path).

If no tauri-dev seat: skip and note in delivery. Flux or Mr. Zhang will cover Layer B.

## 4. Non-negotiables (R-rules / v0.0.1 compliance)

Copy verbatim into delivery doc §Compliance:

| Rule | Required behavior | Location |
|---|---|---|
| R1 attach-only | UI must never send a command that spawns a CLI process. Only `cmd_list_tmux_sessions` + `cmd_attach_tmux_session` are called. | SessionsWorkspace |
| R3 failure-isolation | Close-tab path must not propagate `tmux kill-session`. Use existing `api.killSession` (backend is already R3-safe per A3 Layer A.5). | close tab handler |
| v0.0.1 read-only | `disableStdin: true` on SessionTerminal (already set in A4-α), do not add any write-path. | SessionTerminal — untouched |
| read-only banner | `⚠️ Read-only mode (v0.0.1)...` stays rendered per attached tab. | line 195–197 |
| browser-dev degradation | `!isTauri()` branches must not crash: `setTmuxSessions([])` for list, user-visible error for attach. | both hooks |

## 5. Delivery doc requirements

Write to `docs/coordination/tasks/mira/MIRA-2026-05-09-a4b-sessions-real-api-wire-delivery-v1.md`. Must contain:

1. T3 frontmatter (template, subtype=implementation_delivery, status=delivered, commit=<sha>, packet_ref=this packet).
2. `git show --stat <sha>` verbatim. Expected scope: `ui/src/app-v2/panel/SessionsWorkspace.tsx` + `ui/src/lib/api.ts` + `ui/src/lib/types-dto.ts` only. Zero backend, zero i18n, zero other UI.
3. Paste verbatim: `pnpm exec tsc --noEmit` output + exit; `pnpm build` output + exit.
4. Compliance checklist per §4 above with PASS/FAIL + line evidence.
5. If dev-server smoke ran: DevTools console capture + 3 attach/close observations. If skipped: explicit note "Tauri dev unavailable on this seat — Layer B deferred to Flux/Mr. Zhang."
6. Known limits for carry-forward (expect none; A4-β is a drop-in wire-up).

## 6. Acceptance criteria (Lyra)

1. Three files touched, zero others. No backend, no i18n, no unrelated UI.
2. `MOCK_TMUX_SESSIONS` + the `console.warn("A3 not yet available...")` removed.
3. `api.listTmuxSessions` + `api.attachTmuxSession` both reach their Tauri commands with correct arg envelopes.
4. `TmuxSessionInfo` wire-shape matches Rust struct (verify via runtime capture or source inspection, not guess).
5. `tsc --noEmit` exit 0 + `pnpm build` exit 0.
6. Read-only banner + v0.0.1 pill preserved.
7. Browser-mode no-op paths preserved (no crash when `!isTauri()`).
8. No new lint errors vs HEAD.

Flux Layer A verify runs after Mira delivery. Lyra acceptance follows. SG-B preparation (B1 send-keys / write path) remains Nimbus's next packet.

## 7. Scope boundaries (do not expand)

- Do **not** add a "Launch seat" button, seat-wrapper UI, or anything that spawns a CLI. R1.
- Do **not** add write-path UI (send-keys input field, etc.). That's B1.
- Do **not** touch `SessionTerminal.tsx`; bytes already flow via `onSessionOutput`.
- Do **not** add tests beyond what naturally falls out of type-check; unit-testing this React surface is out of scope for A4-β.
- Do **not** refactor unrelated files even if drive-by cleanup looks tempting.

## 8. Timing

Dispatch 2026-05-09 evening. Target: delivery within Mira's next work session. If blocked on anything (API surface mismatch, unclear wire shape, etc.), post a comment on this packet rather than shipping a partial.

---

*Dispatched by Lyra · 2026-05-09 · A3 UNCONDITIONAL PASS unblocks this packet. Delivery → Flux Layer A verify → Lyra acceptance → SG-B prep.*
