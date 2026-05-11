# B1 Layer B — Three Runtime Bugs (2026-05-11)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_packet |
| id | FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-v1 |
| status | issued |
| author | flux |
| date | 2026-05-11 |
| to | lyra |
| priority | P0 |
| tags | b1, layer-b, runtime-bugs, tmux-mirror |

---

## 0. Context

Mr. Zhang ran B1 Layer B verification (steps 4–9 from the checklist). Three runtime bugs surfaced:

1. **Onyx session: format corruption** — each line wraps multiple times, content wider than xterm viewport. Root cause: xterm.js initializes at 120×30 (hardcoded in `SessionsWorkspace.tsx` line 62), but Onyx's tmux pane is actually 220×57. Bytes flow from tmux at pane width → xterm re-wraps at 120 → double line-break.

2. **Onyx session: auto-scroll flood** — scrollbar shrinks continuously, new lines appear without user input. Root cause: Onyx is running Claude Code in that tmux session, actively outputting. This is **not a bug** — it's expected behavior. The issue is that Mr. Zhang expected a static pane, but Onyx is live. (No code fix needed; clarify in acceptance that active sessions will scroll.)

3. **Mira/Flux sessions: black screen** — attach succeeds (no error), but xterm shows all black, no content. Root cause: `tmux pipe-pane` only captures **new** output after attach. If the pane is idle (no new writes), the FIFO never receives bytes, so xterm stays empty. Historical content (what was already in the pane before attach) is not sent.

**Why Flux Layer A didn't catch these**: Layer A is static-only (tsc + build). Runtime bugs require `pnpm tauri dev` + real tmux sessions, which is Layer B's scope.

---

## 1. Fix scope

Three changes, all in the attach path:

### §1.1 — Query real pane dimensions on attach

**File**: `crates/seatloom-core/src/pty/mod.rs`, `PtySession::attach_tmux` (line 197).

**Current**: `opts.rows` / `opts.cols` are passed from the frontend (hardcoded 30×120 in `SessionsWorkspace.tsx` line 62), but never validated against the actual tmux pane size.

**Fix**: Before `tmux pipe-pane`, run:
```bash
tmux display-message -t <tmux_target> -p '#{pane_width} #{pane_height}'
```
Parse the output (two space-separated integers). Use these as the **authoritative** pane dimensions. Return them in `LiveSessionDto` (new fields `paneRows`, `paneCols`).

**Rationale**: xterm.js must initialize at the same size as the tmux pane to avoid double line-wrapping.

---

### §1.2 — Extend LiveSessionDto with pane dimensions

**File**: `src-tauri/src/commands/session_cmds.rs`, `LiveSessionDto` struct (line 113).

**Current**:
```rust
pub struct LiveSessionDto {
    pub id: String,
    pub seat_id: Option<String>,
    pub runtime: String,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: String,
    pub transcript_path: String,
    pub tmux_session_name: String,
    pub fifo_path: String,
}
```

**Add**:
```rust
    /// Actual tmux pane dimensions at attach time (queried via display-message).
    #[serde(default)]
    pub pane_rows: u16,
    #[serde(default)]
    pub pane_cols: u16,
```

**Frontend**: `ui/src/lib/types-dto.ts`, `LiveSessionDto` interface — add `paneRows?: number; paneCols?: number;`.

**Frontend**: `ui/src/app-v2/panel/SessionsWorkspace.tsx` line 59–63 — replace hardcoded `rows: 30, cols: 120` with:
```typescript
const dto = await api.attachTmuxSession({
  tmuxSessionName,
  // No rows/cols — backend queries real pane size
});
```

**Frontend**: `ui/src/app-v2/panel/SessionTerminal.tsx` line 60–69 — read `paneRows`/`paneCols` from a new prop (passed from `SessionsWorkspace`), use them in `new Terminal({ ... })` if present, else fall back to current defaults.

---

### §1.3 — Send historical pane content on attach

**File**: `crates/seatloom-core/src/pty/mod.rs`, `PtySession::attach_tmux` (line 197).

**Current**: After `tmux pipe-pane` succeeds, the tail-read thread opens the FIFO and waits for new bytes. If the pane is idle, no bytes arrive, xterm stays black.

**Fix**: Immediately after `tmux pipe-pane` succeeds (line 239), before spawning the tail-read thread, run:
```bash
tmux capture-pane -t <tmux_target> -p
```
This dumps the current pane content (scrollback + visible lines) to stdout. Encode it as a `PtyEvent::Output(bytes)` and send it via the broadcast channel **before** the tail-read thread starts. This way, the first subscriber (the Tauri event forwarder in `session_cmds.rs` line 193) receives the historical snapshot immediately, and xterm renders it before any new output arrives.

**Edge case**: If `capture-pane` returns empty (exit 0 but no stdout), that's fine — the pane was genuinely empty. Don't treat it as an error.

**Rationale**: Matches user expectation — attaching to a tmux session should show what's currently in the pane, not a blank screen.

---

## 2. Implementation order

1. §1.1 + §1.2 (pane dimensions) — backend + frontend DTO changes
2. §1.3 (historical content) — backend only
3. Flux Layer A verify (tsc + build + static checks)
4. Mr. Zhang Layer B re-run (steps 4–9, same checklist)

---

## 3. Files to modify

**Backend (Rust)**:
- `crates/seatloom-core/src/pty/mod.rs` — `PtySession::attach_tmux` (query pane size, capture-pane)
- `src-tauri/src/commands/session_cmds.rs` — `LiveSessionDto` struct (add `pane_rows`, `pane_cols`)

**Frontend (TypeScript)**:
- `ui/src/lib/types-dto.ts` — `LiveSessionDto` interface (add `paneRows?`, `paneCols?`)
- `ui/src/app-v2/panel/SessionsWorkspace.tsx` — remove hardcoded rows/cols from `attachTmuxSession` call
- `ui/src/app-v2/panel/SessionTerminal.tsx` — accept `paneRows`/`paneCols` as props, use in `new Terminal({ rows, cols })`

---

## 4. Acceptance criteria

After fix, Mr. Zhang re-runs B1 Layer B steps 4–9:

- **Step 5 (Onyx attach)**: xterm viewport matches tmux pane width, no double line-wrapping. Historical content (what was in the pane before attach) appears immediately.
- **Step 5 (Mira/Flux attach)**: xterm shows the current pane content (shell prompt, last command output, etc.), not a black screen.
- **Step 6–9**: bidirectional write (echo/Ctrl-C/paste/close-tab) still works as before.

**Onyx auto-scroll**: Not a bug. Clarify in acceptance that active sessions (Claude Code running) will continue to output and scroll. This is expected tmux-mirror behavior.

---

## 5. Out of scope

- Scrollback sync beyond the initial `capture-pane` snapshot. If the user scrolls up in xterm, they see xterm's local scrollback buffer (10k lines, line 66 in `SessionTerminal.tsx`), not tmux's scrollback. Full scrollback sync is a future enhancement (R5+).
- Auto-resize on xterm container resize. The `ResizeObserver` → `cmd_pty_resize` path (line 93–100 in `SessionTerminal.tsx`) already exists and should work after §1.1 fix, but Layer B verification doesn't test it. If Mr. Zhang reports resize issues, file a follow-up packet.

---

*Issued by Flux · 2026-05-11 · P0 blocker for B1 UNCONDITIONAL promotion*
