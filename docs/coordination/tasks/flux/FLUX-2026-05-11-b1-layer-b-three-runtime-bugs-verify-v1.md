# Flux Verify: B1 Layer B Three Runtime Bugs Fix

[Lyra -> Flux] Three-bug fix landed at `e809099`. Run commit-pinned Layer A verify against your own bug packet (`d25728a`).

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-05-11 |
| to | flux |
| priority | P0 (gates B1 UNCONDITIONAL promotion) |
| target_commit | `e809099` |
| target_branch | `track/infra-foundation` |
| bug_packet_ref | `docs/coordination/tasks/flux/FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-v1.md` (`d25728a`) |
| flux_delivery_path | `docs/coordination/tasks/flux/FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-delivery-v1.md` |
| tags | flux, verify, b1, layer-b, runtime-bugs, tmux-mirror, commit-pinned |

---

## §1 — Summary

Mr. Zhang ran B1 Layer B and surfaced three runtime bugs:
1. Onyx attach: format corruption (xterm 120-wide, tmux pane 220-wide → double-wrap).
2. Onyx attach: scrollbar flood (not a bug — Onyx is running Claude Code, expected).
3. Mira/Flux attach: black screen (idle pane, no new output, FIFO never fills).

You diagnosed root causes + fix scope in `d25728a`. Code now landed at `e809099`:
- `crates/seatloom-core/src/pty/mod.rs` — `display-message` query + `capture-pane -e -p` snapshot
- `src-tauri/src/commands/session_cmds.rs` — `LiveSessionDto` extended
- `ui/src/lib/types-dto.ts` — TS DTO mirror
- `ui/src/app-v2/panel/SessionsWorkspace.tsx` — drop hardcoded rows/cols, thread props
- `ui/src/app-v2/panel/SessionTerminal.tsx` — accept dims props, conditional `Terminal` init, gated `fit()`/ResizeObserver

5 files / +126 / -9.

---

## §2 — Verify scope

### Layer A — automated checks

```bash
git fetch && git checkout e809099
git rev-parse HEAD                                # must equal e809099 (full sha)
git show e809099 --stat                           # expect 5 files / +126 / -9

# Backend
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test -p seatloom-core --lib                 # must remain ≥60/60 pass
cargo clippy -p seatloom-core -- -D warnings      # same 3 pre-existing lints, no new

# Frontend
cd ui && pnpm exec tsc --noEmit                   # zero errors
cd ui && pnpm build                               # exit 0
```

### Static invariant matrix (15 rows)

| # | Invariant | Where to look |
|---|-----------|---------------|
| 1 | Scope held to 5 files | `git show --stat e809099` |
| 2 | `PtySession` gains `pane_rows: u16`, `pane_cols: u16`, `initial_snapshot: Vec<u8>` | `pty/mod.rs` PtySession struct |
| 3 | `display-message -t <target> -p '#{pane_width} #{pane_height}'` invoked before pipe-pane | `pty/mod.rs` `attach_tmux` |
| 4 | display-message parse failure falls back to (120, 30) — no panic | `pty/mod.rs` parse path |
| 5 | `capture-pane -t <target> -e -p` invoked after pipe-pane succeeds | `pty/mod.rs` capture path; `-e` flag preserves ANSI |
| 6 | LF→CRLF normalization applied to capture-pane output | `pty/mod.rs` post-capture transform |
| 7 | Empty capture-pane stdout treated as empty snapshot, not error | `pty/mod.rs` capture path |
| 8 | `LiveSessionDto` has `pane_rows: u16`, `pane_cols: u16`, `initial_snapshot_b64: String` (all `#[serde(default)]`) | `session_cmds.rs:127-136` |
| 9 | DTO constructor in `cmd_attach_tmux_session` reads all three from `session` | `session_cmds.rs:241-244` |
| 10 | TS `LiveSessionDto` mirrors with `paneRows?`, `paneCols?`, `initialSnapshotB64?` | `types-dto.ts:239-244` |
| 11 | `SessionsWorkspace.attachToTmux` drops hardcoded `rows: 30, cols: 120` | `SessionsWorkspace.tsx:59` — `attachTmuxSession({ tmuxSessionName })` only |
| 12 | `SessionsWorkspace` renders `<SessionTerminal sessionId paneRows paneCols initialSnapshotB64 />` | `SessionsWorkspace.tsx:188` |
| 13 | `SessionTerminal` `Terminal` constructor conditionally applies `{ rows, cols }` when both present | `SessionTerminal.tsx:78` |
| 14 | `fit.fit()` and `ResizeObserver` refit both skipped when `paneRows && paneCols` | `SessionTerminal.tsx:87, 110-112` |
| 15 | Historical snapshot decoded + written to xterm before `onSessionOutput` subscription | `SessionTerminal.tsx:122-131` (snapshot write) precedes `onSessionOutput(...)` (line ~137) |

### R-rule matrix (4 rows)

| Rule | Verify-by |
|------|-----------|
| R1 (read/observe vs own) | snapshot capture is one-shot read — no tmux config mutation, no pane ownership change |
| R3 (failure isolation) | `display-message`/`capture-pane` failures don't kill attach; pipe-pane is still primary; SeatLoom crash leaves tmux untouched |
| AD-013 v2 frozen | no project-mode command additions; no DTO `projectId` changes |
| B1 write path frozen | `pty/mod.rs::write` (load-buffer/paste-buffer) untouched; verify with `git diff 0c0f425..e809099 -- crates/seatloom-core/src/pty/mod.rs` showing no edits in `pub fn write` body |

### Pre-existing clippy lints

Same 3 `doc_lazy_continuation` lints in `db/repositories.rs` that B1/008 verify both flagged as non-blockers. Confirm count unchanged at `e809099` vs parent `9da7844`. New clippy errors beyond those 3 = HOLD.

---

## §3 — Layer B — Mr. Zhang re-run (steps 4–9 only)

After your PASS, Mr. Zhang re-runs the bidirectional smoke from step 4 onward. Steps 1–3 (boot / NavRail / dropdown) already proven on prior Layer B run.

| Step | Action | Expected |
|------|--------|----------|
| 4 | Attach to **Onyx-data-seatloom** | xterm width matches tmux pane width (e.g. 220 cols, no double-wrap); historical content visible immediately; live updates from Claude Code stream in (active session — auto-scroll is expected, not a bug) |
| 5 | Attach to **Mira-ui-seatloom** (idle) | xterm shows current shell prompt + last command output (NOT black screen) |
| 6 | In Mira pane, type `echo hello` + Enter | "hello" appears in tmux pane |
| 7 | Run `sleep 30` then Ctrl-C | Shell prompt returns within ~100ms |
| 8 | Paste a 3-line block | All lines arrive, no truncation |
| 9 | Close tab → tmux list-sessions still shows the session; restart SeatLoom; re-attach → snapshot of current state appears | R3 + restart-recovery |

---

## §4 — Concurrency note

This runs after the existing B1 + 008 PROVISIONAL acceptances. On your PASS + Mr. Zhang re-run PASS, both B1 and 008 promote to UNCONDITIONAL together (combined Layer B has now run end-to-end successfully).

---

## §5 — Delivery doc requirements

Write to `docs/coordination/tasks/flux/FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-delivery-v1.md`. Same shape as your B1 verify delivery doc:

1. T3 frontmatter (template, subtype=verification_delivery, status=delivered, target_commit=`e809099`, packet_ref=this packet).
2. §1 verbatim outputs of all §2 commands (cargo check/test/clippy + tsc + build).
3. §2 15-row static invariant matrix with PASS/FAIL + line evidence.
4. §3 4-row R-rule matrix.
5. §4 known non-blockers (clippy lints, same disposition as prior verifies).
6. §5 verdict: `Layer A PASS` / `HOLD` / `FAIL` with one-paragraph rationale.
7. §6 next-action pointer (→ Mr. Zhang Layer B re-run steps 4–9 → Lyra acceptance → B1 + 008 UNCONDITIONAL).

---

## §6 — Timeline

ASAP. B1 UNCONDITIONAL is the last gate before SG-B stage-gate decision. The faster this closes, the cleaner v0.0.2.

---

*Dispatched by Lyra · 2026-05-11 · P0 · Same three-layer protocol · Targets `e809099` · Mr. Zhang re-runs steps 4–9 on PASS*
