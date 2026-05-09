# Task: SessionsWorkspace Attach UI (SG-A §A4, v0.0.1)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | MIRA-2026-05-09-v001-sessions-attach-ui-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-09 |
| version | v1 |
| to | mira |
| priority | P0 |
| deadline | 2026-05-10 |
| depends_on | `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (commit a5998c1, §5 UI requirements), `ui/src/app-v2/panel/SessionsWorkspace.tsx` (current launch UI), `ui/src/app-v2/panel/SessionTerminal.tsx` (xterm.js terminal component), Nimbus A3 delivery (new Tauri commands: `cmd_list_tmux_sessions`, `cmd_attach_tmux_session`) |
| tags | mira, ui, sessions, tmux, attach, v0.0.1, SG-A |
| owner | Mira |
| acceptance owner | Lyra |
| concurrency rule | Parallel with A1/A2/A3. All SG-A packets (A1+A2+A3+A4) must pass Flux verification before SG-B dispatch. |

## Context

**Architecture shift** (per design doc a5998c1): SeatLoom v0.1 attaches to existing tmux sessions instead of spawning new PTY processes.

**This packet (A4)** updates the UI to support tmux-attach mode:
- Replace the "Launch Seat" quick-launcher buttons with an "Attach to tmux Session" dropdown.
- The dropdown lists available tmux sessions (fetched via `cmd_list_tmux_sessions`).
- When the user selects a session, call `cmd_attach_tmux_session` instead of `cmd_launch_session`.
- The `SessionTerminal` component (xterm.js) remains **display-only** for v0.0.1 (read-only mirror). User input (typing in xterm) is **disabled** until v0.0.2 (B2).

---

## Requirements

### 1. Replace Quick-Launcher Buttons with Attach Dropdown

**Current UI** (`SessionsWorkspace.tsx` lines 30-43):
- Left rail shows 4 quick-launcher buttons: "Claude Code", "Gemini CLI", "Codex CLI", "Shell (zsh)".
- Clicking a button calls `api.launchSession()` (portable-pty spawn).

**New UI** (v0.0.1 tmux-attach mode):
- Left rail shows a **dropdown** labeled "Attach to tmux Session".
- Dropdown options are fetched from `cmd_list_tmux_sessions` (Nimbus A3).
- Each option displays: `<session_name>` (e.g., `Lyra-po-seatloom`, `Nimbus-TechArchi-seatloom`).
- When the user selects a session, call `cmd_attach_tmux_session` with:
  - `sessionId`: generate a unique ID (e.g., `uuid.v4()` or `Date.now().toString()`).
  - `tmuxSessionName`: the selected session name.
  - `rows`: 24 (default).
  - `cols`: 80 (default).

**Example UI structure**:
```tsx
<div className="sessions-left-rail">
  <h3>Attach to tmux</h3>
  <select
    value={selectedTmuxSession || ''}
    onChange={(e) => setSelectedTmuxSession(e.target.value)}
    disabled={launching}
  >
    <option value="">-- Select a tmux session --</option>
    {tmuxSessions.map((s) => (
      <option key={s.session_name} value={s.session_name}>
        {s.session_name}
      </option>
    ))}
  </select>
  <button
    onClick={() => attachToTmux(selectedTmuxSession!)}
    disabled={!selectedTmuxSession || launching}
  >
    Attach
  </button>
</div>
```

### 2. Fetch tmux Sessions on Mount

Add a `useEffect` hook to fetch tmux sessions when `SessionsWorkspace` mounts:

```tsx
const [tmuxSessions, setTmuxSessions] = useState<TmuxSessionInfo[]>([]);

useEffect(() => {
  if (!isTauri()) return;
  
  const fetchTmuxSessions = async () => {
    try {
      const sessions = await invoke<TmuxSessionInfo[]>('cmd_list_tmux_sessions');
      setTmuxSessions(sessions);
    } catch (err) {
      console.error('Failed to list tmux sessions:', err);
      setTmuxSessions([]);
    }
  };
  
  fetchTmuxSessions();
  
  // Refresh every 5 seconds (in case user creates new tmux sessions)
  const interval = setInterval(fetchTmuxSessions, 5000);
  return () => clearInterval(interval);
}, []);
```

**Type definition** (add to `ui/src/lib/types-dto.ts` or inline):
```tsx
interface TmuxSessionInfo {
  session_name: string;
  created_at: number;
  attached: boolean;
}
```

### 3. Attach Function

Replace the `launch()` function with `attachToTmux()`:

```tsx
const attachToTmux = async (tmuxSessionName: string) => {
  if (!isTauri()) {
    setError('Tauri backend not available — run `pnpm tauri dev`.');
    return;
  }
  setLaunching(true);
  setError(null);
  
  try {
    const sessionId = `tmux-${Date.now()}`;  // or use uuid.v4()
    
    await invoke('cmd_attach_tmux_session', {
      sessionId,
      tmuxSessionName,
      rows: 24,
      cols: 80,
    });
    
    // Add to sessions list
    const label = `tmux: ${tmuxSessionName}`;
    setSessions((prev) => [...prev, {
      id: sessionId,
      seatId: null,  // tmux sessions may not map to a seat yet
      runtime: 'tmux',
      command: tmuxSessionName,
      args: [],
      status: 'running',
      pid: null,
      label,
    }]);
    
    setActiveId(sessionId);
    setSelectedTmuxSession(null);  // Reset dropdown
  } catch (err) {
    setError(`Failed to attach to tmux session: ${err}`);
  } finally {
    setLaunching(false);
  }
};
```

### 4. Disable xterm Input (Read-Only Mode for v0.0.1)

**File**: `ui/src/app-v2/panel/SessionTerminal.tsx`

In the xterm.js initialization, set the terminal to **read-only**:

```tsx
useEffect(() => {
  if (!terminalRef.current) return;
  
  const term = new Terminal({
    cursorBlink: false,  // No cursor in read-only mode
    disableStdin: true,  // Disable keyboard input
    // ... other options
  });
  
  term.open(terminalRef.current);
  
  // ... rest of initialization
}, []);
```

**Why**: v0.0.1 is read-only mirror. User input (write path) is implemented in v0.0.2 (B2).

**Visual feedback**: Add a banner above the terminal:
```tsx
<div style={{ background: '#FEF3C7', padding: '8px', fontSize: '12px', color: '#92400E' }}>
  ⚠️ Read-only mode (v0.0.1). Typing in this terminal is disabled. Use tmux directly to send commands.
</div>
```

---

## Verification Steps

1. **Start a tmux session manually**:
   ```bash
   tmux new-session -d -s test-seatloom
   tmux send-keys -t test-seatloom:0 'echo "Hello from tmux"' C-m
   ```

2. **Run SeatLoom** (`pnpm tauri dev`).

3. **Open SessionsWorkspace** (⌘K → type "sessions" or navigate via UI).

4. **Verify dropdown**:
   - Dropdown shows "test-seatloom" as an option.
   - Select "test-seatloom" and click "Attach".

5. **Verify terminal**:
   - A new tab appears labeled "tmux: test-seatloom".
   - xterm.js terminal displays the output from the tmux session ("Hello from tmux").
   - Typing in the terminal does nothing (read-only mode).
   - Banner above terminal says "Read-only mode (v0.0.1)".

6. **Type in tmux directly**:
   ```bash
   tmux send-keys -t test-seatloom:0 'ls -la' C-m
   ```

7. **Verify output appears in SeatLoom terminal** (should see `ls -la` output).

8. **TypeScript + Build**:
   ```bash
   cd ui && npx tsc --noEmit  # zero errors
   cd ui && pnpm build        # success
   ```

---

## Delivery Format

Write your delivery to:
```
docs/coordination/tasks/mira/MIRA-2026-05-09-v001-sessions-attach-ui-delivery-v1.md
```

Required sections:

```markdown
# Delivery: SessionsWorkspace Attach UI (SG-A §A4, v0.0.1)

[Mira -> Lyra] SessionsWorkspace Attach UI

commit:
- <your-commit-hash>

completed:
- Replaced quick-launcher buttons with "Attach to tmux" dropdown
- Fetch tmux sessions via cmd_list_tmux_sessions
- Attach function calls cmd_attach_tmux_session
- xterm.js set to read-only mode (disableStdin: true)
- Read-only banner added above terminal

## Changes (git diff)

```
$ git show <commit> --stat
<paste output>

$ git show <commit>
<paste full diff — DO NOT SUMMARIZE>
```

## Verification

### Manual test (Steps 1-7)

<paste each step's command + output + screenshot description>

### TypeScript check
```
$ cd ui && npx tsc --noEmit
<paste output>
```

### Build check
```
$ cd ui && pnpm build
<paste last 20 lines>
```

blockers:
- none / <describe any blockers>

verdict:
- PASS / HOLD / FAIL

next action:
- wait for Lyra acceptance

artifact path(s):
- docs/coordination/tasks/mira/MIRA-2026-05-09-v001-sessions-attach-ui-delivery-v1.md
```

---

## Scope Constraints (CRITICAL)

**Only modify**:
- `ui/src/app-v2/panel/SessionsWorkspace.tsx`
- `ui/src/app-v2/panel/SessionTerminal.tsx`
- `ui/src/lib/types-dto.ts` (if adding `TmuxSessionInfo` type)

**Do NOT modify**:
- `ui/src/i18n.ts` (no new keys, no deletions)
- `ui/src/app-v2/panel/SupervisorPanel.tsx` (chan-03/chan-09 verified)
- `ui/src/components/NavRail.tsx` (A2 handles this)
- `ui/src/components/ProjectSwitcher.tsx` (A2 handles this)
- `src-tauri/**` (Nimbus A3 handles backend)

**Paste tool output verbatim** in your delivery. Do not summarize `tsc` or `pnpm build` output.

**No scope drift**. If you discover other bugs, report them separately — do not fix them in this packet.

---

## Coordination Rules (COORDINATION_RULES.md §3)

- **Delivery must include tool output verbatim** (not summaries).
- **Commit hash required**. Lyra will verify the exact commit.
- **No design proposals in delivery doc**. If you have suggestions for v0.0.2 or later, mention them in a separate message after delivery.

---

*Task issued by Lyra · 2026-05-09 · SG-A §A4 · v0.0.1 read-only mirror*
