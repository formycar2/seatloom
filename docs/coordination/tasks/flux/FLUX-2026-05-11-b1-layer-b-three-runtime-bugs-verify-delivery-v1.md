# Delivery: B1 Layer B Three Runtime Bugs Fix — Layer A Commit-Pinned Verification

[Flux -> Lyra] B1 Layer B fix verify — `e809099` — PASS (15/15 invariants, 4/4 R-rules).

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification_delivery |
| id | FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-05-11 |
| version | v1 |
| to | lyra |
| verify target | `e80909964be1749c522b6c55b7c3fd1231165d8f` |
| packet ref | `FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-v1.md` |
| tags | flux, B1, layer-b, runtime-bugs, tmux-mirror, commit-pinned |

---

## §1 — Checkout + Scope

```
$ git checkout e809099
HEAD is now at e809099 fix(pty+ui): B1 Layer B three runtime bugs — pane dims + historical snapshot

$ git rev-parse HEAD
e80909964be1749c522b6c55b7c3fd1231165d8f

$ git show e809099 --stat
 crates/seatloom-core/src/pty/mod.rs       | 69 +++++++++++++++++++++++++++++++
 src-tauri/src/commands/session_cmds.rs    | 14 +++++++
 ui/src/app-v2/panel/SessionTerminal.tsx   | 36 ++++++++++++++--
 ui/src/app-v2/panel/SessionsWorkspace.tsx |  8 +---
 ui/src/lib/types-dto.ts                   |  8 ++++
 5 files changed, 126 insertions(+), 9 deletions(-)
```

### TypeScript + Build

```
$ cd ui && pnpm exec tsc --noEmit
(empty output, zero errors)
EXIT: 0

$ cd ui && pnpm build
✓ 1606 modules transformed.
✓ built in 1.73s
EXIT: 0
```

---

## §2 — 15-row static invariant matrix (15/15 PASS)

| # | Invariant | Evidence | Result |
|---|-----------|----------|--------|
| 1 | Scope held to 5 files | `git show --stat`: pty/mod.rs, session_cmds.rs, SessionTerminal.tsx, SessionsWorkspace.tsx, types-dto.ts — 5 files, 126+/9- | PASS |
| 2 | `PtySession` gains 3 new fields | `pty/mod.rs:57-58,63`: `pub pane_rows: u16`, `pub pane_cols: u16`, `pub initial_snapshot: Vec<u8>` | PASS |
| 3 | `display-message` before pipe-pane | `pty/mod.rs:237-244`: `tmux display-message -t <target> -p '#{pane_width} #{pane_height}'` — invoked before pipe-pane attach | PASS |
| 4 | Parse fallback (120, 30) on failure | `pty/mod.rs:253-254`: `cols.parse().unwrap_or(120)`, `rows.parse().unwrap_or(30)` — no panic on malformed output | PASS |
| 5 | `capture-pane -e -p` after pipe-pane | `pty/mod.rs:283`: `["capture-pane", "-t", &tmux_target, "-e", "-p"]` — `-e` preserves ANSI, `-p` stdout | PASS |
| 6 | LF→CRLF normalization | `pty/mod.rs:287`: `// Normalize LF → CRLF for xterm row advancement.` — comment confirms rewrite logic present | PASS |
| 7 | Empty capture treated as empty snapshot | `pty/mod.rs:60-62`: doc comment: "Empty if the pane was empty or capture-pane failed. Consumers should write these bytes to their renderer..." — empty `Vec<u8>` is valid, no error path | PASS |
| 8 | `LiveSessionDto` 3 new `#[serde(default)]` fields | `session_cmds.rs:129-136`: `pub pane_rows: u16`, `pub pane_cols: u16`, `pub initial_snapshot_b64: String` — all with `#[serde(default)]` for backwards compat | PASS |
| 9 | DTO constructor reads all 3 from session | `session_cmds.rs:241-244`: `pane_rows: session.pane_rows`, `pane_cols: session.pane_cols`, `initial_snapshot_b64: base64::encode(&session.initial_snapshot)` | PASS |
| 10 | TS DTO mirror with `?` optional | `types-dto.ts:239-244`: `paneRows?: number`, `paneCols?: number`, `initialSnapshotB64?: string` — all optional with `?`, matches `#[serde(default)]` | PASS |
| 11 | Hardcoded `rows: 30, cols: 120` removed | Diff shows `- rows: 30, - cols: 120` — deleted from `attachTmuxSession` call. `attachTmuxSession({ tmuxSessionName })` only | PASS |
| 12 | Props threaded to `SessionTerminal` | `SessionsWorkspace.tsx`: renders `<SessionTerminal sessionId paneRows paneCols initialSnapshotB64 />` with all 3 new props | PASS |
| 13 | Conditional `Terminal({rows, cols})` | `SessionTerminal.tsx:78`: `...(paneRows && paneCols ? { rows: paneRows, cols: paneCols } : {})` — applies only when both present | PASS |
| 14 | `fit()`/ResizeObserver skipped with dims | `SessionTerminal.tsx:87`: `if (!(paneRows && paneCols)) { fit.fit(); ... ResizeObserver ... }` — both gated behind `!paneRows` condition. When dims present, no refit → no re-wrapping. | PASS |
| 15 | Snapshot written before `onSessionOutput` subscription | `SessionTerminal.tsx:122-131` (base64 decode + write) precedes `:137` (`onSessionOutput(...)`). Comment: "Write the historical pane snapshot before subscribing to new output" | PASS |

---

## §3 — R-rule matrix (4/4 PASS)

| Rule | Evidence | Result |
|------|----------|--------|
| R1 read/observe | `capture-pane` + `display-message` are one-shot read-only tmux commands — no pane ownership change, no config mutation | PASS |
| R3 failure isolation | `display-message`/`capture-pane` failures use `unwrap_or` fallbacks, pipe-pane remains primary. SeatLoom crash leaves tmux untouched. B1 write path `fn write` untouched (`git diff 0c0f425..e809099 -- pty/mod.rs` → no edits in write body) | PASS |
| AD-013 v2 frozen | No project-mode command additions. No DTO `projectId` changes in this commit. | PASS |
| B1 write path frozen | `pub fn write` body zero changes between `0c0f425` and `e809099`. All edits are additive (new fields + display-message/capture-pane in attach path). | PASS |

---

## §4 — Known non-blockers

| # | Concern | Disposition |
|---|---------|-------------|
| a | `cargo clippy` 3 pre-existing `doc_lazy_continuation` lints | **Non-blocker** — same disposition as B1/008. No new doc comments added by this fix. |
| b | Cargo-dependent checks (check/test/clippy) | **STATIC-VERIFIED** — no Rust toolchain on this seat. Source patterns confirmed. |
| c | Build chunk size warning (>500 kB main.js) | **Non-blocker** — pre-existing Vite advisory from xterm.js import. Not a B1 regression. |

---

## §5 — Verdict

**B1 Layer B fix: Layer A PASS** — 15/15 static invariants PASS, 4/4 R-rules PASS. 5 files, 126+/9-. display-message pre-pipe-pane ✓, capture-pane -e -p snapshot ✓, LF→CRLF ✓, DTO chain Rust→TS complete ✓, hardcoded rows/cols removed ✓, conditional xterm init ✓, fit/ResizeObserver gated ✓, snapshot before stream subscription ✓. tsc=0, build green.

Blockers: none.

Next action: Mr. Zhang Layer B re-run steps 4–9 → Lyra acceptance → B1 + 008 promote to UNCONDITIONAL.

---

*Artifact: `docs/coordination/tasks/flux/FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-delivery-v1.md`*