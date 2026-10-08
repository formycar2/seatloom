# Acceptance: UM0 mirror fidelity — pi's TUI renders cleanly in the UI

[Lyra] Verdict: **ACCEPT — PASS**. Both mirror-fidelity fixes are merged into `track/infra-foundation`. Mr. Zhang's garbled-input-line bug is closed: the xterm viewport is the geometry authority and the tmux pane follows it exactly.

| Field | Value |
|---|---|
| template | T3 |
| subtype | acceptance |
| id | LYRA-2026-10-08-um0-mirror-fidelity-acceptance-v1 |
| status | accepted |
| author | lyra |
| date | 2026-10-08 |
| version | v1 |
| milestone | UM0 — one screen for the whole team (clean) |
| governing dispatch | `docs/coordination/tasks/lyra/AEGIS-2026-10-08-um0-mirror-fidelity-v1.md` |
| nudges | `/tmp/lyra-nudge-mirror-merge.md`, `/tmp/lyra-nudge-mirror-merge2.md` |
| delivery under test (UI) | `docs/coordination/tasks/mira/MIRA-2026-10-08-um0-mirror-fidelity-delivery-v1.md` @ `da4213b` |
| delivery under test (backend) | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-pty-resize-window-delivery-v1.md` @ `45d6c15` |
| verification | `docs/coordination/tasks/flux/FLUX-2026-10-08-um0-mirror-fidelity-verify-delivery-v1.md` |
| verified commit (combined tree) | `f5c3719c5f4d060c08de86f914d12e83df6efcc1` |
| target branch | `track/infra-foundation` |
| mira branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| nimbus branch | `packet/nimbus/NIMBUS-2026-10-08-pty-resize-window-v1` |
| tags | um0, acceptance, mirror-fidelity, xterm, tmux, pi |

---

## 1. Decisions needed

None outstanding. The residual governance question carried from the earlier UM0 gate — whether `App.tsx`'s 15 pre-existing `no-explicit-any` findings must be cleaned before merge or the ratchet base adjusted — remains a follow-up, not a blocker, per Flux's recommendation and the zero-regression standard.

## 2. Decisions made

1. **UM0 mirror fidelity is accepted as PASS.** The acceptance bar Aegis set — a pi seat's TUI (input line, truecolor, boxes) renders cleanly and matches the real pane, and resizing the UI resizes the pane and stays clean — is met against the **real binary**, not a shim.
2. **Both halves were required.** With only Mira's fix the pane could not follow the viewport because the backend was a silent no-op; with only Nimbus's fix nothing would call it. Neither packet alone closes the defect.
3. **The verification target was the combined tree.** Neither branch contains the other's fix, and Mira's own evidence necessarily shimmed the Rust `invoke()` layer. Flux therefore built base `59c162c` + `da4213b` + `45d6c15` and reported the exact merge SHA `f5c3719c5f4d060c08de86f914d12e83df6efcc1`.
4. **Both branches are merged into `track/infra-foundation`.**

## 3. Action packets (executed)

| # | Owner | Action | Done-definition | Status |
|---|---|---|---|---|
| 1 | Mira | UI geometry sync + feed correctness | viewport fitted, pane follows via `cmd_pty_resize`, raw feed, no stale-snapshot double-inject, frontend only | **DELIVERED** @ `da4213b`, tip `a8ce2ed` |
| 2 | Nimbus | `cmd_pty_resize` silent no-op | `resize-window` instead of `resize-pane`, geometry read-back verified, typed error on mismatch, doc comments corrected | **DELIVERED** @ `45d6c15`, tip `d8c02ed` |
| 3 | Flux | Commit-pinned verify of the **combined** tree | exact merge SHA from a clean workspace, real-binary behavioural gate, zero-regression, hazards handled | **PASS** @ `f5c3719`, evidence commit `844ba8a` |
| 4 | Lyra | Accept + merge both branches into `track/infra-foundation` | merge recorded, writeback | **DONE** |
| 5 | Lyra | Park the merge note for Aegis | daily-log entry with merged SHAs | **DONE** |

## 4. Evidence summary (from Flux's PASS)

**Geometry — pane equals the fitted xterm viewport, exactly:**

| moment | fitted xterm | `#{pane_width} #{pane_height}` | equal |
|---|---|---|---|
| mount (fullscreen window) | 66 rows × 146 cols | **258 66** | yes |
| resize A — container 880×540 | 28 rows × 108 cols | **108 28** | yes |
| after typing at A | 28 × 108 | 108 28 | yes |
| resize B — container 1180×700 | 36 rows × 146 cols | **146 36** | yes |
| settle B +6 s / +16 s / +30 s | 36 × 146 | 146 36 | yes |

The 258×66 mount is the decisive data point: the pane followed the viewport to a geometry far from its 219×54 launch size, which is exactly what the old `resize-pane` path could not do.

**Clean render:** the input line sits in-viewport on the last content row; box-drawing spans the full 146 columns; truecolor is parsed into inline `rgb()` styles (`rgb(109,109,109)`, `rgb(128,128,128)`, `rgb(46,46,46)` — the exact colours the real pane emits); prompt and status lines match `tmux capture-pane -p`.

**Typing round-trips:** two runtime-generated markers typed through the UI's real key path appear **exactly once** in both the rendered xterm buffer and the real pane — no doubling, no loss.

**Resize stays clean:** re-dumped at +6 s, +16 s and +30 s after each resize — geometry still equal, frame repainted at the new width, no stale frame, no doubled borders, no shifted rows.

**`ResizeNotApplied` is live code** returned by `resize()` on geometry mismatch, covered by five passing unit tests.

**Gates:** cargo check / test (104) / clippy `-D warnings` / fmt all green; `pnpm typecheck` 0; `pnpm test` 10/10; `pnpm build` 0; eslint 151 problems (143 errors, 8 warnings) and prettier 92 files both equal to the accepted baselines; the single changed UI file is clean on both; the ratchet's red is entirely pre-existing (`App.tsx` `any` + `StatusBar.tsx` prettier).

**Scope:** 4 files, +500/−67 — `ui/src/app-v2/panel/SessionTerminal.tsx`, `crates/seatloom-core/src/pty/mod.rs`, `crates/seatloom-core/examples/nimbus_resize_proof.rs`, `src-tauri/src/commands/session_cmds.rs` (doc comment). Zero `.sql`, zero DTO-boundary files, zero unrelated files.

## 5. Blockers

- None open against mirror fidelity.
- Carried as non-blocking follow-ups:
  1. The `sessions` (会话) tab still asserts `live` provenance unconditionally — same false-live family as HOLD-3; Mira consolidation follow-up.
  2. `App.tsx`'s 15 pre-existing `no-explicit-any` findings behind `scripts/lint-changed.sh` — ratchet-base decision, unchanged from the earlier gate.
  3. Re-attach / multi-client detach semantics after the pipe-pane merge — a Flux pass against the merged branch is a follow-up.
  4. **The shared UI is down** (§6).

## 6. Stage-gate status

- **UM0: GO, both halves.** The UI shows the five live PI seats (`Flux-Quality&Ops-seatloom`, `Lyra-po-seatloom`, `Mira-UX/UED-seatloom`, `Nimbus-TechArchi-seatloom`, `Onyx-data-seatloom`) and now renders their real pi TUIs cleanly, with the pane tracking the viewport exactly.
- Next: Aegis reconciles `track/infra-foundation` to GitLab and restarts the shared UI onto the merged code. Per the ROADMAP, B1 (supervisor-IM via the Aegis pane) follows.
- **Operational note:** Flux reports the shared app (pid 66269) was closed by a human at 21:07:08 — a graceful `mouseUp` / `windowShouldClose` exit with no crash report — and he could not have caused it (screen capture and assistive access are both denied on this host). This is the second time an additive Flux launch has coincided with the shared window being closed by hand. A fresh `tauri dev` on the shared checkout is required before re-pointing it at the merged code, and because Flux restored all five seats' `pane_pipe` to `0`, the mirrors will need re-attaching on the next run.

## 7. Merge record

| Step | Branch | Commit | Result | Merge commit |
|---|---|---|---|---|
| Mira merge | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` | `a8ce2ed05af3f4bc4e7f6e1d7fb187b52f40d3b1` | merged `--no-ff`; conflicts in `docs/coordination/MEMORY.md` and `docs/coordination/memory/2026-10-08.md` resolved by retaining both seats' writeback; no code conflicts | recorded at merge |
| Nimbus merge | `packet/nimbus/NIMBUS-2026-10-08-pty-resize-window-v1` | `d8c02ed` (code `45d6c15`) | merged into `track/infra-foundation` | recorded at merge |

Both `da4213b` and `45d6c15` are confirmed ancestors of `track/infra-foundation` after the merges.

## 8. Follow-ups and owners

| Item | Owner | Note |
|---|---|---|
| Consolidate the two attach surfaces (`sessions` tab unconditional `live`) | Mira | Follow-up packet |
| `App.tsx` 15 `no-explicit-any` behind the ratchet | Nimbus / Aegis governance | Ratchet-base decision |
| Re-attach / multi-client detach verification against the merged branch | Flux | Follow-up; `abe4245` now reachable |
| Pipe state close on session drop | Nimbus | Already recommended inside the pipe-pane packet scope |
| Aegis reconcile track → GitLab + restart shared UI | Aegis | Parked in daily log |

---

*Accepted by Lyra · 2026-10-08 · UM0 mirror fidelity GO · Flux PASS at `f5c3719` (combined tree) · Mira `da4213b` + Nimbus `45d6c15` merged into `track/infra-foundation`*
