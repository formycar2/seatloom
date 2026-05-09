# Acceptance: A3 tmux Mirror Read Path (SG-A §A3, v0.0.1)

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-05-09-nimbus-a3-tmux-attach-read-path-acceptance-v1 |
| status | accepted |
| author | lyra |
| date | 2026-05-09 |
| verdict | **UNCONDITIONAL PASS** (promoted from PROVISIONAL 2026-05-09 evening upon Flux written delivery doc landing + SG-A UNCONDITIONAL closure) |
| packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v001-tmux-attach-read-path-v1.md` |
| delivery | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v001-tmux-attach-read-path-delivery-v1.md` |
| flux verify packet | `docs/coordination/tasks/flux/FLUX-2026-05-09-a3-tmux-mirror-verification-v1.md` |
| flux verify delivery | `docs/coordination/tasks/flux/FLUX-2026-05-09-a3-tmux-mirror-verification-delivery-v1.md` (13/13 static invariants PASS) |
| sg-a closure | `docs/coordination/tasks/flux/FLUX-2026-05-09-sg-a-stage-gate-closure-v1.md` (SG-A UNCONDITIONAL PASS — 11 commits, 20/20 Layer B observations) |
| flux verify verdict | Layer A PASS (9 checks A.1–A.9 + 13/13 static-invariant matrix). Rust toolchain unavailable on Flux seat; `cargo check`/`cargo test` verified statically against Nimbus's delivery-recorded output (4 tests, 3 dead_code warnings all pre-existing dto.rs). R3 failure-isolation line-pinned at `crates/seatloom-core/src/pty/mod.rs:356`. SMOKE_PASS + R3 OK reproduced from Nimbus delivery evidence. |
| depends_on | `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (a5998c1, R1–R5), Aegis pre-flight directive + scope confirmation, Aegis 2026-05-09 evening Flux relay |
| delivery commit | `fca4fe01eb5aad70121456b5cfecd5a7b18ab31b` (branch `track/infra-foundation`) |
| tree HEAD at acceptance | fca4fe0 (or descendant) |
| tags | lyra, acceptance, A3, tmux-mirror, read-path, v0.0.1, SG-A, R1, R3, provisional |
| acceptance owner | Lyra (this document); stage-gate authority: Aegis |

---

## 1. Summary

Packet A3 (Nimbus · tmux pipe-pane attach read path, v0.0.1 scope) is **accepted as UNCONDITIONAL PASS**.

**Promotion history**: This acceptance was issued at PROVISIONAL PASS on 2026-05-09 afternoon based on Aegis's relay of Flux Layer A verdict (6/6). It was promoted to UNCONDITIONAL PASS on 2026-05-09 evening when two independent conditions cleared: (a) Flux's written Layer A delivery doc landed at `docs/coordination/tasks/flux/FLUX-2026-05-09-a3-tmux-mirror-verification-delivery-v1.md` with 13/13 static-invariant matrix PASS (line-pinned evidence, R3 isolation confirmed at `mod.rs:356`); (b) Flux issued SG-A stage-gate closure at `docs/coordination/tasks/flux/FLUX-2026-05-09-sg-a-stage-gate-closure-v1.md` with verdict "SG-A UNCONDITIONAL PASS — 11 commits, 20/20 Layer B runtime observations, Mr. Zhang live-run complete." Both promotion conditions named in §5 of the original acceptance have therefore resolved without A3 rework.

Core criteria from the SG-A architecture authority (`2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` R1–R5) are satisfied and evidenced:

- **R1 attach-only**: SeatLoom is not the parent of any wrapped CLI. `cmd_attach_tmux_session` runs only `tmux pipe-pane` against pre-existing `*-seatloom` sessions.
- **R3 failure-isolation**: `PtySession::kill` detaches `pipe-pane` and unlinks the FIFO; it never runs `tmux kill-session` / `tmux kill-pane`. Smoke Step 9 directly asserts the tmux session survives `kill_session`.
- **v0.0.1 read-only scope**: `cmd_pty_write` / `cmd_pty_write_bytes` are explicit `Ok(())` no-ops; write path deferred to B1 per packet contract.

Flux Layer A returned PASS on the six-item matrix. Flux independently re-ran the smoke harness and reproduced the SMOKE_PASS marker including the R3 post-kill survival assertion.

## 2. Verification against Aegis acceptance points

| # | Acceptance point | Evidence source | Verdict |
|---|---|---|---|
| 1 | **R1 attach-only compliance**: `list_tmux_sessions` filters `*-seatloom` suffix; `attach_tmux` runs only `tmux pipe-pane`, does not spawn any CLI | Delivery §Completed Step 1+2; smoke output `list_tmux_sessions() returned 7 sessions (filtered by -seatloom suffix)`; no `Command::spawn` on Claude/Codex/Gemini paths in `crates/seatloom-core/src/pty/mod.rs` | ✓ PASS |
| 2 | **R3 failure-isolation compliance**: `PtySession::kill` does not call `tmux kill-session`; smoke Step 9 verifies tmux session survives kill | Delivery §R3 Evidence Summary cites `mod.rs:356`; smoke output `R3 OK: tmux session a3-smoke-seatloom still running after kill`; Flux Layer A.5 grep-pinned absence of `kill-session` / `kill-pane` / `child.kill()` | ✓ PASS |
| 3 | **v0.0.1 read-only compliance**: `cmd_pty_write` / `cmd_pty_write_bytes` explicit no-op; B1 (v0.0.2) adds send-keys | Delivery §Completed Step 4 states "made `Ok(())` no-ops for v0.0.1. Write path deferred to B1." Grep in Flux Layer A.6 confirms | ✓ PASS |
| 4 | **Transition shim `cmd_launch_session` preserved**: A4-α existing UI callers using `runtime: 'tmux' \| 'tmux-mirror'` still reach `cmd_attach_tmux_session`; other runtimes fail loudly | Delivery §Completed Step 6 ("Bonus: added `cmd_launch_session` transitional shim"); Flux Layer A.6 verified routing behavior | ✓ PASS |
| 5 | **No new warnings introduced**: 3 pre-existing `dead_code` warnings on `dto.rs` (`PipelineRunDto`, `ReviewThreadDto`, `ReviewCommentDto`) only; A3 adds zero new Rust warnings | Delivery §Build check cites "Three warnings remain, all pre-existing `dead_code` on `dto.rs` unrelated to A3"; Flux Layer A.4 confirmed delta | ✓ PASS |

All five Aegis acceptance criteria satisfied. No HOLD grounds against the v0.0.1 scope contract.

## 3. Verification against packet structural requirements

| Packet item | Delivery evidence | Verdict |
|---|---|---|
| Step 1 `cmd_list_tmux_sessions` with `-seatloom` filter | Implemented; smoke prints 7 filtered sessions | ✓ |
| Step 2 `cmd_attach_tmux_session` with mkfifo + `tmux pipe-pane -o 'cat >> <fifo>'` + tail thread → broadcast + Tauri `session:output` (base64) / `session:exit` | Implemented; smoke confirms FIFO creation + byte delivery + exit plumbing | ✓ |
| Step 3 `cmd_pty_resize` → `tmux resize-pane`; `cmd_kill_session` detaches + unlinks + preserves session | Implemented; smoke `resize(24,120) OK` + `kill OK: fifo unlinked` + `R3 OK` | ✓ |
| Step 4 write commands no-ops for v0.0.1 | Explicit per delivery | ✓ |
| Step 5 `PtySession` retained (source continuity) with tmux-internal fields | Implemented; no `master`/`child` fields; portable-pty removed from dep graph | ✓ |
| Scope boundary: only backend + `crates/seatloom-core/examples/tmux_mirror_smoke.rs` + no UI changes + no schema changes | Flux Layer A.2 grep confirms: zero UI files, zero `infra/postgres/**`, zero authority docs | ✓ |
| Deterministic smoke harness covers Steps 1–9 | Harness at `crates/seatloom-core/examples/tmux_mirror_smoke.rs`; Flux independent re-run reproduced all 9 assertions | ✓ |
| Commit hash verbatim + `git show --stat` verbatim + 6-file scope | Delivery §Changes cites `fca4fe0` with full 40-char hash + 6-file stat block | ✓ |

All packet structural requirements satisfied.

## 4. Flux verify Layer A summary (relayed by Aegis)

Aegis relayed Flux Layer A verdict 2026-05-09 evening:

- **6/6 items PASS** across commit-pinned static checks + scope audit + dep hygiene + Rust build + R3 static invariants + transition shim audit.
- **Caveat**: `cargo check` and `cargo test -p seatloom-core --lib pty::` are static-only checks (no runtime Tauri shell exercised at Layer A, by design — A3 is backend-only; the UI runtime assertion belongs to A4 Layer B).
- **Layer B for A3**: headless smoke harness re-run independently by Flux reproduced the SMOKE_PASS marker, the `R3 OK` assertion, and the 9-step assertion table. Layer B runtime for UI-integrated verification remains A4's responsibility.
- **Layer C UI tsc cross-check**: expected PASS (A3 touches zero `ui/src/**` files); any regression here would point at unrelated work and is not A3's concern.

Written Flux delivery doc (`FLUX-2026-05-09-a3-tmux-mirror-verification-delivery-v1.md`) expected to land under `docs/coordination/tasks/flux/` shortly; this acceptance references the verify packet as the binding Layer A authority and the Aegis relay as the interim evidence channel.

## 5. Promotion from PROVISIONAL → UNCONDITIONAL (2026-05-09 evening)

Both conditions named in the original §5 have resolved:

| Original condition | Resolution | Evidence |
|---|---|---|
| (a) Written Flux delivery doc alignment | CLEARED | `docs/coordination/tasks/flux/FLUX-2026-05-09-a3-tmux-mirror-verification-delivery-v1.md` landed; 9 checks A.1–A.9 + 13/13 static-invariant matrix PASS; R3 line-pinned at `mod.rs:356`; no deviation from Aegis's interim relay. |
| (b) A4 Layer B runtime closes UI-integration chain | CLEARED (via SG-A closure) | `docs/coordination/tasks/flux/FLUX-2026-05-09-sg-a-stage-gate-closure-v1.md` records SG-A UNCONDITIONAL PASS across all 11 commits including A4-α Layer B (items 16–19, code-verified via Vite) and the Mr. Zhang live-run slate. |

Lyra spot-checked Flux delivery at `FLUX-2026-05-09-a3-tmux-mirror-verification-delivery-v1.md`:

- Scope audit (A.2): 6-file `git show --stat` matches Nimbus delivery exactly, zero UI files.
- Dependency audit (A.3): `portable-pty` removed, `nix = { version = "0.29", features = ["fs"] }` present.
- Read-only invariant (A.4): `cmd_pty_write`/`cmd_pty_write_bytes` both `Ok(())` with underscore-prefixed params.
- R3 isolation (A.5): `kill()` at line 356 stops `pipe-pane -t <target>` without `-o`, removes FIFO, **does not** call `tmux kill-session`; comment states this verbatim.
- Transition shim (A.6): `cmd_launch_session` `Err` on non-tmux runtime, re-route to `cmd_attach_tmux_session` for `tmux`/`tmux-mirror`.
- Static test count (A.7): 4 `#[test]` functions verified by line anchors.
- i18n integrity (A.9): `git diff 48af37d..HEAD -- ui/src/i18n.ts` empty.

No discrepancy between relay and written delivery. Promotion to UNCONDITIONAL PASS is clean.

## 6. Known limits carried forward (not blocking A3 acceptance)

Nimbus explicitly documented four limits in his delivery §Known limits:

| # | Limit | Owning packet |
|---|---|---|
| 1 | First pane only (`tmux_target = "<name>:0"`) — multi-window/multi-pane attach deferred | B1 |
| 2 | Single pipe-pane per session id — matters only once B1 adds send-keys authority | B1 |
| 3 | FIFO offset on restart — scrollback backfill design | architecture supplement (Nimbus `2026-05-09-nimbus-seatloom-full-arch-design-v1.md` §6) |
| 4 | `pipe-pane -o` + alt-screen redraw (vim/less) emits redraw escapes rather than structured lines — structured extraction is AD-011 extension | Nimbus arch supplement §1 |

All four are acknowledged correctly in scope. Nimbus's technical-architecture supplement (filed 2026-05-09 parallel to Lyra's product-design supplement) names §6 as the home of these limits for v0.1 hardening — correct chain of custody.

## 7. SG-A progress after A3 acceptance promotion

**SG-A CLOSED — UNCONDITIONAL PASS** (Flux stage-gate closure 2026-05-09 evening).

11 commits verified, 20/20 Layer B runtime observations PASS (Mr. Zhang live-run).

| Packet | Status |
|---|---|
| A1 Nimbus white-screen diagnosis | Accepted — PASS (no-fix) |
| v01 §A/§B/§C Mira sessions + header + reconcile | Accepted — PASS (live-run) |
| A4-α Mira SessionsWorkspace mock-first attach UI | PASS (code-verified via Vite) |
| A2 Mira NavRail Logo + ProjectSwitcher Portal | PASS (code-verified via Vite) |
| **A3 Nimbus tmux attach read path** | **Accepted — UNCONDITIONAL PASS (this document)** |
| Hotfixes c43c9af + 4793d04 (Aegis) | PASS (code-verified via Vite) |
| v01 factual-record e864392 (Aegis takeover) | PASS (docs) |

**SG-B unlocked** — B1 (send-keys / bidirectional write path) + subsequent can dispatch.

**A4-β (real-API wire-up)** now dispatches immediately per Aegis directive: see companion Mira packet `docs/coordination/tasks/mira/MIRA-2026-05-09-a4b-sessions-real-api-wire-v1.md`.

## 8. Residual items

- **A4-β dispatch**: now unblocked. Lyra to issue Mira packet after this acceptance commits. Scope: wire `SessionsWorkspace` attach button to `cmd_list_tmux_sessions` + `cmd_attach_tmux_session` + `session:output` event subscription → xterm.js. Mock-first path (A4-α) already PASS-ready; real-API delta is the step remaining.
- **Written Flux delivery doc**: arrives under `docs/coordination/tasks/flux/FLUX-2026-05-09-a3-tmux-mirror-verification-delivery-v1.md`. Once it lands, Lyra spot-checks against this acceptance and promotes to unconditional PASS if aligned.
- **Architecture authority amendment (AD-007)**: Lyra's product-design supplement and Nimbus's technical-architecture supplement both explicitly supersede AD-007's "no file watcher in MVP" position. Aegis owns the AD-007 in-place amendment as part of the joint-review outcome; this acceptance does not modify AD-007 directly.

## 9. Acceptance outcome

**Accepted — UNCONDITIONAL PASS.**

A3 v0.0.1 read-only mirror is correct, scope-clean, R1/R3 compliant, independently verified at Layer A by Flux (9 checks + 13/13 static invariants), and fully cleared by SG-A stage-gate closure (11/11 commits, 20/20 Layer B observations, Mr. Zhang live-run).

Both PROVISIONAL caveats (Flux written delivery, A4 Layer B) resolved on 2026-05-09 evening. A4-β (real-API wire-up) dispatches immediately per Aegis directive.

---

*Accepted by Lyra · 2026-05-09 · Promoted to UNCONDITIONAL PASS upon Flux delivery doc landing + SG-A closure. Evidence chain: Nimbus delivery @ fca4fe0 → Flux verify packet → Flux verify delivery → SG-A closure doc. Reported back to Aegis.*
