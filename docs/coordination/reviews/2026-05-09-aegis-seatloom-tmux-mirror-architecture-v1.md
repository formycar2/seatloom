# Design Proposal: SeatLoom v0.1 as tmux Mirror + Augmentation Layer

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | 2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1 |
| status | issued |
| author | aegis |
| date | 2026-05-09 |
| to | lyra |
| priority | P0 |
| depends_on | `docs/coordination/reviews/2026-05-09-aegis-mvp-gap-to-tmux-replacement.md` (commit 0cc401b, now superseded by this design), `docs/coordination/reviews/2026-05-08-aegis-cli-plan-mode-integration-design.md` (chan-10 Layer A/B/C still valid), `docs/architecture-decisions.md` (AD-008 – AD-013), `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md` |
| tags | architecture, tmux, mirror, v0.1, design-proposal |
| acceptance owner | lyra |
| concurrency rule | Design authority. Supersedes gap review 0cc401b §2 "tmux replacement" framing. Lyra re-scopes §4 packet list per this design before dispatching A1/A2. |

---

## Context: product positioning shift (2026-05-09)

**Previous framing** (gap review 0cc401b §2): "SeatLoom v0.1 replaces tmux coordination. User launches seat sessions from SeatLoom UI via portable-pty. tmux is retired."

**New framing** (Mr. Zhang directive 2026-05-09): "SeatLoom v0.1 **mirrors + augments** the existing tmux multi-CLI workflow. tmux remains the authoritative runtime. SeatLoom observes tmux output, surfaces it structurally, and optionally injects keystrokes back. SeatLoom can crash / upgrade / have bugs without disturbing tmux. User operates in either or both surfaces in parallel."

**Why the shift**: Mr. Zhang explicitly lacks confidence in v0.1 stability. He needs a fallback: if SeatLoom misbehaves, he continues working in tmux without losing context. "我对 v0.1 没有信心，我需要一个备案" was his exact framing.

---

## Hard requirements (Mr. Zhang Q1-Q5 answers)

| # | Requirement | Implication |
|---|---|---|
| **R1** | **Attach mode primary**. SeatLoom joins existing tmux sessions; never replaces them. Current tmux setup (`Aegis-Supervisor`, `Lyra-po-seatloom`, `Mira-UX/UED-seatloom`, `Nimbus-TechArchi-seatloom`, `Flux-Quality&Ops-seatloom`) must remain usable while SeatLoom v0.1 boots alongside. | portable-pty spawn mode is OUT for v0.1. Tauri commands `cmd_launch_session` / `cmd_pty_write` / `cmd_pty_resize` / `cmd_kill_session` must be re-implemented to attach to tmux, not spawn new PTY. |
| **R2** | **Bidirectional**. tmux PTY output → SeatLoom backend mirror (read). SeatLoom UI input → `tmux send-keys` into target session (write). User switches surfaces freely. | Read path: `tmux pipe-pane` → fifo/log → Tauri backend tail. Write path: Tauri backend → `tmux send-keys -t <session>:<pane>`. |
| **R3** | **Failure isolation, hard rule**. SeatLoom crash / upgrade / bug → tmux workflow zero impact. | SeatLoom must NOT be the parent process of tmux. tmux mirror plumbing (`pipe-pane`) must continue producing data even if SeatLoom is down; SeatLoom restart picks up the tail. |
| **R4** | **Data flow lossless**. Everything CLIs in tmux produce — terminal output, written markdown under `docs/coordination/`, git operations — must be captured / reconciled into PostgreSQL with structural fidelity. | File-system watchers (`notify` crate) on `docs/coordination/` tree trigger `cmd_reconcile` automatically. Manual reconcile button (Mira's §C commit 2f83624) becomes fallback, not primary. |
| **R5** | **Sub-version pacing OK**. v0.0.1 / v0.0.2 / v0.1 progression acceptable, but v0.1 final ships all of R1-R4. | Lyra may sequence packets as v0.0.1 (read-only mirror) → v0.0.2 (bidirectional) → v0.1 (auto-reconcile + failure-hardened). |

---

## Architecture: tmux as runtime, SeatLoom as observer + injector

```
┌─────────────────────────────────────────────────────────────────┐
│ tmux (authoritative runtime, never owned by SeatLoom)           │
├─────────────────────────────────────────────────────────────────┤
│ session: Lyra-po-seatloom                                       │
│   pane 0: `claude` (PID 12345)                                  │
│     ├─ stdout/stderr → tmux pane buffer                         │
│     ├─ tmux pipe-pane -o → /tmp/seatloom-mirror/lyra.fifo      │
│     └─ user types in tmux → stdin                               │
│                                                                  │
│ session: Nimbus-TechArchi-seatloom                              │
│   pane 0: `zsh` (PID 67890)                                     │
│     ├─ stdout/stderr → tmux pane buffer                         │
│     ├─ tmux pipe-pane -o → /tmp/seatloom-mirror/nimbus.fifo    │
│     └─ user types in tmux → stdin                               │
└─────────────────────────────────────────────────────────────────┘
                         ↓ mirror data flow
┌─────────────────────────────────────────────────────────────────┐
│ SeatLoom Tauri backend (observer + injector)                    │
├─────────────────────────────────────────────────────────────────┤
│ 1. tmux session discovery:                                      │
│    `tmux list-sessions -F '#{session_name}'` → match pattern   │
│    `<seat>-*-seatloom` → map to seat ID in PostgreSQL          │
│                                                                  │
│ 2. Attach read path (per session):                              │
│    `tmux pipe-pane -t <session>:<pane> -o 'cat >> <fifo>'`     │
│    Tauri backend tail-reads <fifo> → base64 → emit              │
│    `session:output` event to UI (same as portable-pty did)      │
│                                                                  │
│ 3. Attach write path (per session):                             │
│    UI calls `cmd_pty_write(session_id, data)` →                │
│    Tauri backend: `tmux send-keys -t <session>:<pane> <data>`  │
│                                                                  │
│ 4. File watcher (notify crate):                                 │
│    Watch `docs/coordination/**/*.md` → on write event →         │
│    debounce 2s → call reconcile engine → update PostgreSQL     │
│                                                                  │
│ 5. Failure isolation:                                           │
│    - tmux pipe-pane runs as tmux child, not SeatLoom child     │
│    - fifo persists on disk; SeatLoom restart re-opens tail     │
│    - if SeatLoom crashes, tmux sessions unaffected             │
└─────────────────────────────────────────────────────────────────┘
                         ↓ structured data
┌─────────────────────────────────────────────────────────────────┐
│ SeatLoom UI (Tauri frontend)                                    │
├─────────────────────────────────────────────────────────────────┤
│ - SessionsWorkspace: shows tmux sessions as tabs                │
│ - SessionTerminal: xterm.js rendering `session:output` events   │
│ - SupervisorPanel: canonical_events from reconcile + PTY parse  │
│ - DocumentsWorkspace: reconciled markdown from PostgreSQL       │
│ - User can type in SessionTerminal → cmd_pty_write → tmux      │
│ - User can also work directly in tmux; SeatLoom mirrors it      │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation phases (v0.0.1 → v0.0.2 → v0.1)

### Phase v0.0.1: Read-only mirror (SG-A revised)

**Goal**: SeatLoom UI displays tmux session output in real-time. No write-back yet. Manual reconcile only.

| Packet | Owner | Scope |
|---|---|---|
| **A1** | Nimbus | White-screen diagnosis (unchanged from original gap review) |
| **A2** | Mira | NavRail Logo + ProjectSwitcher z-index (unchanged) |
| **A3** | Nimbus | `cmd_attach_tmux_session(session_name)` — discover tmux session by name pattern, set up `pipe-pane` to fifo, tail-read fifo → emit `session:output` events. Replace portable-pty spawn logic in `crates/seatloom-core/src/pty/mod.rs`. |
| **A4** | Mira | SessionsWorkspace: replace "launch seat" buttons with "attach to tmux session" dropdown (list from `tmux list-sessions`). SessionTerminal remains xterm.js display-only (no input yet). |

**Acceptance gate SG-A**: User runs `pnpm tauri dev`, sees tmux sessions listed in SessionsWorkspace, clicks one, xterm tab shows live mirrored output from tmux. Typing in tmux → output appears in SeatLoom UI within 500ms. SeatLoom crash → tmux unaffected, restart SeatLoom → output resumes.

### Phase v0.0.2: Bidirectional (SG-B revised)

**Goal**: User can type in SeatLoom SessionTerminal → keystrokes injected into tmux via `send-keys`.

| Packet | Owner | Scope |
|---|---|---|
| **B1** | Nimbus | `cmd_pty_write(session_id, data)` — map session_id to tmux session name, call `tmux send-keys -t <session>:<pane> -l <data>`. Handle special keys (Enter, Ctrl-C, etc.) via `send-keys` key names. |
| **B2** | Mira | SessionTerminal: enable xterm.js `onData` → call `api.ptyWrite(sessionId, data)`. Add visual indicator (green dot) when session is attached + bidirectional. |
| **B3** | Nimbus | PTY output → SeatResponse canonical_event bridge (unchanged from original gap review §B2, but now reads from tmux mirror fifo instead of portable-pty). |

**Acceptance gate SG-B**: User types in SeatLoom SessionTerminal → characters appear in both SeatLoom xterm AND in the underlying tmux pane. User switches to tmux, types there → output appears in SeatLoom. Bidirectional loop confirmed.

### Phase v0.1: Auto-reconcile + hardening (SG-C/D/E collapsed)

**Goal**: File watcher auto-reconciles `docs/coordination/` changes. Failure-hardened: SeatLoom restart recovers state.

| Packet | Owner | Scope |
|---|---|---|
| **C1** | Nimbus | File watcher (`notify` crate) on `docs/coordination/**/*.md` → debounce 2s → call reconcile engine. Add Tauri command `cmd_start_file_watcher()` / `cmd_stop_file_watcher()`. Call on app startup. |
| **C2** | Nimbus | ClaudeAdapter Layer A (unchanged from chan-10 design) — read `~/.claude/projects/<slug>/<sid>.jsonl` transcript tail, detect ExitPlanMode → write `prompt_instances` table. |
| **C3** | Nimbus | schema 006 migration (unchanged from original gap review §C3) — add 'plan_approval' to prompt_kind CHECK + 'cli_plan' to validate_subtype. |
| **C4** | Nimbus | Restart recovery: on `cmd_attach_tmux_session`, if fifo already exists, re-open tail from last read offset (store offset in `sessions` table `metadata` jsonb). |

**Acceptance gate v0.1**: User writes a new T3 packet markdown under `docs/coordination/tasks/mira/`, saves file → within 3s, SeatLoom DocumentsWorkspace shows the new doc (no manual reconcile button click). User kills SeatLoom (`pkill seatloom`), restarts → attached sessions resume output streaming from where they left off. tmux sessions never interrupted.

---

## Impact on original gap review 0cc401b §4 packet list

| Original packet | Status | Revision |
|---|---|---|
| A1 (Nimbus white-screen) | **Keep as-is** | No change |
| A2 (Mira NavRail Logo + z-index) | **Keep as-is** | No change |
| B1 (SessionTerminal in SeatDetail) | **Defer to v0.2** | Not needed for v0.1 mirror mode; SeatDetail can link to SessionsWorkspace tab |
| B2 (PTY → SeatResponse bridge) | **Revise** | Now reads from tmux mirror fifo, not portable-pty. Becomes v0.0.2 §B3. |
| B3 (reconcile UI) | **Partial done** | Mira's §C commit 2f83624 already has manual button. C1 adds auto-watcher. |
| C1 (ClaudeAdapter Layer A) | **Keep, sequence later** | Becomes v0.1 §C2. No change to design. |
| C2 (GeminiAdapter Layer A) | **Defer to v0.2** | Not P0 for v0.1. |
| C3 (schema 006) | **Keep, sequence later** | Becomes v0.1 §C3. |
| D1 (cmd_search_documents) | **Defer to v0.2** | Not P0 for v0.1 mirror mode. |
| D2 (review threads) | **Defer to v0.2** | Not P0. |
| D3 (schema 007 seats budget) | **Defer to v0.2** | Not P0. |
| D4 (session exit → checkpoint) | **Defer to v0.2** | Not P0. |
| E1 (Mira multi-project switcher) | **Defer to v0.2** | Not P0. |
| E2 (Mira mobile responsive) | **Defer to v0.3** | Not P0. |

**New packets** (not in original gap review):
- **A3** (Nimbus tmux attach read path) — NEW, P0 for v0.0.1
- **A4** (Mira SessionsWorkspace attach UI) — NEW, P0 for v0.0.1
- **B1** (Nimbus tmux send-keys write path) — NEW, P0 for v0.0.2
- **B2** (Mira SessionTerminal bidirectional) — NEW, P0 for v0.0.2
- **C1** (Nimbus file watcher auto-reconcile) — NEW, P0 for v0.1
- **C4** (Nimbus restart recovery) — NEW, P0 for v0.1

---

## Lyra's next actions

1. **Acknowledge this design proposal**:
   ```
   [Lyra -> Aegis] Design proposal 2026-05-09 tmux-mirror acknowledged.
   Supersedes gap review 0cc401b §2 "replacement" framing.
   Re-scoping §4 packet list per phases v0.0.1 / v0.0.2 / v0.1.
   ```

2. **Draft revised packet sequence** (cite this design doc as depends_on):
   - v0.0.1: A1 (Nimbus white-screen) + A2 (Mira UI) + **A3 (Nimbus tmux attach read)** + **A4 (Mira attach UI)**
   - v0.0.2: **B1 (Nimbus send-keys write)** + **B2 (Mira bidirectional terminal)** + B3 (Nimbus PTY→SeatResponse, revised)
   - v0.1: **C1 (Nimbus file watcher)** + C2 (Nimbus ClaudeAdapter Layer A) + C3 (Nimbus schema 006) + **C4 (Nimbus restart recovery)**

3. **Dispatch A1 + A2 immediately** (unchanged from original plan), then draft A3 + A4 while Nimbus/Mira work on A1/A2.

4. **Update memory** after each phase gate passes Flux verification.

---

*Design proposal issued by Aegis · 2026-05-09 · Supersedes gap review 0cc401b §2 per Mr. Zhang directive.*
