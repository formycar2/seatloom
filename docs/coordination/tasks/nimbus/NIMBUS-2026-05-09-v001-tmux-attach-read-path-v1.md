# Task: tmux Attach Read Path (SG-A §A3, v0.0.1)

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | NIMBUS-2026-05-09-v001-tmux-attach-read-path-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-09 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-05-10 |
| depends_on | `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (commit a5998c1, §3 architecture + §4 read path spec), `crates/seatloom-core/src/pty/mod.rs` (current portable-pty implementation), `src-tauri/src/commands/session_cmds.rs` (Tauri command surface: `cmd_launch_session`, `cmd_pty_write`, `cmd_pty_resize`, `cmd_kill_session`) |
| tags | nimbus, tmux, attach, read-path, v0.0.1, SG-A |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | Parallel with A1/A2/A4. All SG-A packets (A1+A2+A3+A4) must pass Flux verification before SG-B dispatch. |

## Context

**Architecture shift** (per design doc a5998c1): SeatLoom v0.1 no longer spawns CLI processes via `portable-pty`. Instead, it **attaches to existing tmux sessions** and mirrors their output.

**Why**: Mr. Zhang needs tmux as a fallback. If SeatLoom crashes or has bugs, tmux continues running unaffected. SeatLoom observes tmux, does not own it.

**This packet (A3)** implements the **read path** for v0.0.1:
- tmux session discovery (`tmux list-sessions`)
- Attach to a session via `tmux pipe-pane` → fifo → tail-read → broadcast to UI
- Replace the current `portable-pty` spawn logic in `crates/seatloom-core/src/pty/mod.rs` with tmux-attach logic

**Write path** (user input → `tmux send-keys`) is deferred to **B1** (v0.0.2).

---

## Requirements (R1-R3 from design doc)

| # | Requirement | Implementation |
|---|---|---|
| **R1** | Attach mode primary. SeatLoom joins existing tmux sessions; never replaces them. | `cmd_launch_session` must be renamed/refactored to `cmd_attach_tmux_session`. It takes a `tmux_session_name` parameter (e.g., `"Lyra-po-seatloom"`) instead of spawning a new PTY. |
| **R2** | Bidirectional (read + write). | **This packet implements read only.** Write path (B1) comes in v0.0.2. |
| **R3** | Failure isolation. SeatLoom crash → tmux unaffected. | SeatLoom must NOT be the parent process of tmux. `tmux pipe-pane` plumbing continues producing data even if SeatLoom is down. |

---

## Your Task

### Step 1 — tmux Session Discovery

Add a new Tauri command:

```rust
#[tauri::command]
pub async fn cmd_list_tmux_sessions() -> Result<Vec<TmuxSessionInfo>, String> {
    // Run: tmux list-sessions -F '#{session_name}:#{session_created}:#{session_attached}'
    // Parse output into Vec<TmuxSessionInfo>
    // Filter for sessions matching pattern: <seat>-*-seatloom
    // Return the list
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TmuxSessionInfo {
    pub session_name: String,
    pub created_at: i64,  // Unix timestamp
    pub attached: bool,
}
```

**Implementation notes**:
- Use `std::process::Command` to run `tmux list-sessions -F '#{session_name}:#{session_created}:#{session_attached}'`.
- Parse each line: split by `:`, extract fields.
- Filter for sessions ending in `-seatloom` (e.g., `Lyra-po-seatloom`, `Nimbus-TechArchi-seatloom`).
- Return empty vec if tmux is not running or no sessions match.

### Step 2 — tmux Attach Read Path

Refactor `cmd_launch_session` → `cmd_attach_tmux_session`:

```rust
#[tauri::command]
pub async fn cmd_attach_tmux_session(
    session_id: String,
    tmux_session_name: String,
    rows: u16,
    cols: u16,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 1. Create fifo at /tmp/seatloom-mirror/<session_id>.fifo
    // 2. Run: tmux pipe-pane -t <tmux_session_name>:0 -o 'cat >> /tmp/seatloom-mirror/<session_id>.fifo'
    // 3. Spawn a tokio task that tail-reads the fifo and broadcasts PtyEvent::Output chunks
    // 4. Store the session handle in AppState.sessions
    // 5. Return Ok(())
}
```

**Implementation details**:

1. **Create fifo**:
   ```rust
   use std::os::unix::fs::DirBuilderExt;
   let fifo_dir = PathBuf::from("/tmp/seatloom-mirror");
   std::fs::create_dir_all(&fifo_dir).map_err(|e| e.to_string())?;
   let fifo_path = fifo_dir.join(format!("{}.fifo", session_id));
   
   // Create named pipe (FIFO)
   use nix::sys::stat;
   use nix::unistd;
   if fifo_path.exists() {
       std::fs::remove_file(&fifo_path).ok();
   }
   unistd::mkfifo(&fifo_path, stat::Mode::S_IRUSR | stat::Mode::S_IWUSR)
       .map_err(|e| format!("mkfifo failed: {}", e))?;
   ```

2. **Attach tmux pipe-pane**:
   ```rust
   let output = std::process::Command::new("tmux")
       .args(&[
           "pipe-pane",
           "-t", &format!("{}:0", tmux_session_name),
           "-o",
           &format!("cat >> {}", fifo_path.display()),
       ])
       .output()
       .map_err(|e| format!("tmux pipe-pane failed: {}", e))?;
   
   if !output.status.success() {
       return Err(format!("tmux pipe-pane error: {}", String::from_utf8_lossy(&output.stderr)));
   }
   ```

3. **Tail-read fifo and broadcast**:
   ```rust
   let (tx, _rx) = tokio::sync::broadcast::channel::<PtyEvent>(1024);
   let tx_clone = tx.clone();
   let fifo_path_clone = fifo_path.clone();
   let session_id_clone = session_id.clone();
   
   tokio::spawn(async move {
       let mut file = match tokio::fs::File::open(&fifo_path_clone).await {
           Ok(f) => f,
           Err(e) => {
               eprintln!("[session {}] fifo open failed: {}", session_id_clone, e);
               return;
           }
       };
       
       let mut buf = vec![0u8; 4096];
       loop {
           match file.read(&mut buf).await {
               Ok(0) => break,  // EOF
               Ok(n) => {
                   let chunk = buf[..n].to_vec();
                   if tx_clone.send(PtyEvent::Output(chunk)).is_err() {
                       break;  // All receivers dropped
                   }
               }
               Err(e) => {
                   eprintln!("[session {}] fifo read error: {}", session_id_clone, e);
                   break;
               }
           }
       }
       
       // Cleanup fifo on exit
       std::fs::remove_file(&fifo_path_clone).ok();
   });
   ```

4. **Store session handle**:
   ```rust
   let session = TmuxSession {
       id: session_id.clone(),
       tmux_session_name: tmux_session_name.clone(),
       events: tx,
       fifo_path,
   };
   
   state.sessions.lock().unwrap().insert(session_id, session);
   ```

### Step 3 — Update `cmd_pty_resize` and `cmd_kill_session`

**`cmd_pty_resize`**: For tmux-attach mode, this should run:
```bash
tmux resize-pane -t <tmux_session_name>:0 -x <cols> -y <rows>
```

**`cmd_kill_session`**: For tmux-attach mode, this should:
1. Run `tmux pipe-pane -t <tmux_session_name>:0` (no args) to stop the pipe-pane.
2. Remove the fifo file.
3. Remove the session from `AppState.sessions`.

**Do NOT kill the tmux session itself** — that would violate R3 (failure isolation).

### Step 4 — Keep `cmd_pty_write` as no-op for v0.0.1

`cmd_pty_write` currently writes to the portable-pty master. For v0.0.1 (read-only), make it a **no-op** (return `Ok(())` without doing anything). The write path will be implemented in **B1** (v0.0.2) via `tmux send-keys`.

### Step 5 — Update `PtySession` struct

Rename `PtySession` → `TmuxSession` (or keep the name but change internals):

```rust
#[derive(Clone)]
pub struct TmuxSession {
    pub id: String,
    pub tmux_session_name: String,
    pub events: broadcast::Sender<PtyEvent>,
    pub fifo_path: PathBuf,
}
```

Remove the `master` and `child` fields (no longer needed for tmux-attach mode).

---

## Verification Steps

1. **Start a tmux session manually**:
   ```bash
   tmux new-session -d -s test-seatloom
   tmux send-keys -t test-seatloom:0 'echo "Hello from tmux"' C-m
   ```

2. **Run SeatLoom** (`pnpm tauri dev`).

3. **Call `cmd_list_tmux_sessions`** from the UI (or via Tauri DevTools):
   - Should return `[{ session_name: "test-seatloom", ... }]`.

4. **Call `cmd_attach_tmux_session`**:
   ```typescript
   await invoke('cmd_attach_tmux_session', {
     sessionId: 'test-001',
     tmuxSessionName: 'test-seatloom',
     rows: 24,
     cols: 80,
   });
   ```

5. **Subscribe to session output**:
   ```typescript
   await invoke('cmd_subscribe_session_output', { sessionId: 'test-001' });
   // Listen for events via Tauri event system
   ```

6. **Type in the tmux session**:
   ```bash
   tmux send-keys -t test-seatloom:0 'ls -la' C-m
   ```

7. **Verify output appears in SeatLoom UI** (xterm.js terminal should show `ls -la` output).

8. **Call `cmd_kill_session`**:
   ```typescript
   await invoke('cmd_kill_session', { sessionId: 'test-001' });
   ```

9. **Verify**:
   - Fifo file removed from `/tmp/seatloom-mirror/`.
   - tmux session still running (`tmux list-sessions` shows `test-seatloom`).

10. **TypeScript + Build**:
    ```bash
    cd ui && npx tsc --noEmit  # zero errors
    cd ui && pnpm build        # success
    cargo build --release      # success
    ```

---

## Delivery Format

Write your delivery to:
```
docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v001-tmux-attach-read-path-delivery-v1.md
```

Required sections:

```markdown
# Delivery: tmux Attach Read Path (SG-A §A3, v0.0.1)

[Nimbus -> Lyra] tmux Attach Read Path

commit:
- <your-commit-hash>

completed:
- Step 1: cmd_list_tmux_sessions implemented
- Step 2: cmd_attach_tmux_session implemented (fifo + pipe-pane + tail-read)
- Step 3: cmd_pty_resize and cmd_kill_session updated for tmux mode
- Step 4: cmd_pty_write made no-op for v0.0.1
- Step 5: TmuxSession struct updated

## Changes (git diff)

```
$ git show <commit> --stat
<paste output>

$ git show <commit>
<paste full diff — DO NOT SUMMARIZE>
```

## Verification

### Manual test (Steps 1-9)

<paste each step's command + output>

### TypeScript check
```
$ cd ui && npx tsc --noEmit
<paste output>
```

### Build check
```
$ cd ui && pnpm build
<paste last 20 lines>

$ cargo build --release
<paste last 20 lines>
```

blockers:
- none / <describe any blockers>

verdict:
- PASS / HOLD / FAIL

next action:
- wait for Lyra acceptance

artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v001-tmux-attach-read-path-delivery-v1.md
```

---

## Scope Constraints

- **Only modify**:
  - `crates/seatloom-core/src/pty/mod.rs` (or create new `crates/seatloom-core/src/tmux/mod.rs`)
  - `src-tauri/src/commands/session_cmds.rs`
  - `src-tauri/src/main.rs` (register new commands)
- **Do NOT modify**:
  - `ui/src/**` (UI changes are in A4, handled by Mira)
  - `infra/postgres/**` (schema changes deferred to C3)
  - Any other Rust crates unrelated to session management
- **Dependencies**: You may add `nix` crate for `mkfifo` if not already present. Check `Cargo.toml` first.

---

## Coordination Rules (COORDINATION_RULES.md §3)

- **Delivery must include tool output verbatim** (not summaries).
- **Commit hash required**. Lyra will verify the exact commit.
- **No scope drift**. If you discover other issues, report them separately.

---

*Task issued by Lyra · 2026-05-09 · SG-A §A3 · v0.0.1 read-only mirror*
