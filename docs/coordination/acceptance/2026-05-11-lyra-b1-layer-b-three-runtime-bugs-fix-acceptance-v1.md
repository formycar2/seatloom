# Acceptance: B1 Layer B Three-Bug Fix — Layer A PASS

[Lyra -> Flux + Mira + Aegis] Flux Layer A PASS at `e809099`. Lyra acceptance issued. Mr. Zhang Layer B re-run (steps 4–9) triggered via Aegis. On re-run PASS, B1 promotes PROVISIONAL → UNCONDITIONAL; 008 Layer B smoke runs in the same Tauri window and promotes in parallel.

| Field | Value |
|---|---|
| template | T3 |
| subtype | acceptance |
| id | LYRA-2026-05-11-b1-layer-b-three-runtime-bugs-fix-acceptance-v1 |
| status | accepted |
| author | lyra |
| date | 2026-05-11 |
| version | v1 |
| to | flux, mira, aegis, mr_zhang |
| target_commit | `e80909964be1749c522b6c55b7c3fd1231165d8f` |
| bug_packet_ref | `docs/coordination/tasks/flux/FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-v1.md` (`d25728a`) |
| flux_verify_packet | `docs/coordination/tasks/flux/FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-v1.md` (`8565b02`) |
| flux_verify_delivery | `docs/coordination/tasks/flux/FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-delivery-v1.md` |
| layer_a_verdict | PASS (Flux: 15/15 static + 4/4 R-rules + tsc=0 + build green; cargo STATIC-VERIFIED) |
| layer_b_verdict | PENDING — Mr. Zhang via Aegis (steps 4–9 re-run) |
| tags | lyra, b1, layer-b, runtime-bugs, tmux-mirror, layer-a-pass, layer-b-pending |

---

## §1 — Verdict

**PROVISIONAL ACCEPTED — Layer A unconditional PASS.** Promotes to UNCONDITIONAL on Mr. Zhang Layer B re-run PASS.

Flux evidence chain solid:
- 15/15 static invariants PASS with line-anchored evidence
- 4/4 R-rules PASS: R1 (observe-only), R3 (failure-isolation via unwrap_or fallbacks + untouched write path), AD-013 v2 frozen, B1 write path frozen
- Diff confirmed `pub fn write` body zero changes between `0c0f425` (B1 landing) and `e809099` (this fix) — write path regression surface is clean
- tsc=0, pnpm build exit 0 (1606 modules)

Three fixes land exactly where Flux's `d25728a` packet called for:
1. **§1.1 pane dims** — `display-message -t <target> -p '#{pane_width} #{pane_height}'` invoked before pipe-pane, parsed with `unwrap_or(120)`/`unwrap_or(30)` fallback — Onyx 220-wide panes will render without double-wrap
2. **§1.2 DTO chain** — `LiveSessionDto` extended with `pane_rows`, `pane_cols`, `initial_snapshot_b64` (all `#[serde(default)]`); TS mirror with `?` optionals; hardcoded `rows: 30, cols: 120` removed from `attachTmuxSession` call; conditional `Terminal({rows, cols})` init; `fit()` + ResizeObserver both gated behind `!hasAuthoritativeSize`
3. **§1.3 historical snapshot** — `capture-pane -e -p` (ANSI-preserving) executed after pipe-pane success; LF→CRLF normalized; written to xterm *before* `onSessionOutput` subscription so idle panes render their current content, not a black screen

## §2 — Three-layer status

| Layer | Owner | Status | Evidence |
|---|---|---|---|
| Delivery | Mr. Zhang (via seat) | ✓ landed | `e809099` (5 files, +126/-9) |
| Pre-flight | Flux | ✓ ratified in bug packet | `d25728a` — root cause + fix spec |
| Layer A commit-pinned verify | Flux | ✓ PASS | 15/15 + 4/4; `FLUX-2026-05-11-b1-layer-b-three-runtime-bugs-verify-delivery-v1.md` |
| Layer B runtime | Mr. Zhang via Aegis | ⏳ PENDING | Re-run steps 4–9 per §3 |
| Lyra final acceptance | Lyra | ⏳ provisional pending Layer B | This document promotes to UNCONDITIONAL on Layer B PASS |

## §3 — Mr. Zhang Layer B — re-run steps 4–9

Target: `pnpm tauri dev` at HEAD `e809099` or descendant (currently `8565b02` on `track/infra-foundation`, descendant).

Steps 1–3 (boot / NavRail shows 会话 / dropdown lists *-seatloom sessions) already PASSed on the initial Layer B run. No need to re-run.

| Step | Action | Expected | Verifies |
|------|--------|----------|----------|
| 4 | Attach to **Onyx-data-seatloom** (active, running Claude Code) | xterm viewport matches tmux pane width (expected ≥200 cols, no double-line-wrap on content). Historical content visible immediately on attach. Live output continues streaming (auto-scroll **expected** — Onyx is active; **not a bug**). | §1.1 dims + §1.3 snapshot on active session |
| 5 | Attach to **Mira-UX/UED-seatloom** (idle) | xterm shows current shell prompt / last command output. **NOT** a black screen. | §1.3 snapshot on idle session — the core black-screen bug |
| 6 | Attach to **Flux-Quality&Ops-seatloom** (idle or active) | Same as step 5 — historical content visible. | §1.3 snapshot generality |
| 7 | In the attached pane, type `echo hello` + Enter | "hello" appears in the real tmux pane (cross-check `tmux attach-session -t <name>`) | B1 forward write still works (unchanged, sanity check) |
| 8 | Run `sleep 30`, then Ctrl-C | Shell prompt returns within ~100ms | B1 Ctrl-C / SIGINT path still works (unchanged) |
| 9 | Close the SeatLoom 会话 tab (× button); then restart SeatLoom + re-attach | tmux session still alive (`tmux list-sessions` shows it); scrollback preserved via tmux; re-attach renders the post-close state via snapshot | R3 + restart-recovery + snapshot-on-recovery |

**On all 6 steps PASS**:
- B1 promotes PROVISIONAL → UNCONDITIONAL
- 008 Layer B 4-step smoke (Inbox/WorkItems/Sessions/Handoffs render + Project mode counts 6/5/3/35 + Global mode preserved) runs in the same Tauri window
- 008 promotes PROVISIONAL → UNCONDITIONAL on its PASS
- Aegis SG-B stage-gate decision follows

**On any FAIL**: HOLD with named step + observation; Mr. Zhang dispatches a fix-v2 packet.

## §4 — Clarification: Onyx auto-scroll

Flux's `d25728a` §0 bug-2 diagnosis stands: **Onyx's continuous output is not a bug**. Onyx-data-seatloom runs Claude Code actively, which emits output whenever the session processes work. Mr. Zhang's "scrollbar shrinks continuously" observation is correct and expected behavior — it's a live tmux mirror of an active Claude session, not a static pane.

No code change needed. Acceptance acknowledges this as expected tmux-mirror behavior for active sessions.

## §5 — Carry-forwards (acknowledged, not blocking)

1. **Pre-existing clippy lints** (`db/repositories.rs` × 3 `doc_lazy_continuation`) — Flux confirmed non-regression via static inspection. Same disposition as B1 + 008. Recommend sibling cleanup packet.
2. **Cargo-dependent checks STATIC-VERIFIED** — no Rust toolchain on the seat where the fix was authored. Flux Layer A confirmed structural correctness from source. If cargo check/test surfaces any issue during Mr. Zhang Layer B runtime, HOLD and dispatch fix-v2.
3. **Scrollback beyond initial capture** — if the user scrolls up in xterm past the captured snapshot, they see xterm's local scrollback buffer (10k lines), not tmux's full history. Full tmux-scrollback sync is out-of-scope per Flux §5 — future R5+ enhancement.
4. **xterm resize on container resize** — ResizeObserver is now gated when authoritative dims are in use. If Mr. Zhang resizes the SeatLoom window during Layer B and reports display issues, file follow-up packet (out of this scope per Flux §5).

## §6 — What this acceptance unlocks

On Layer B PASS:
- **B1 UNCONDITIONAL** — bidirectional tmux mirror write path promotes, zero outstanding runtime concerns.
- **008 UNCONDITIONAL** (combined Layer B window) — multi-project schema isolation promotes, B2 onward fully unlocked at both API + runtime.
- **SG-B stage-gate decision** — Aegis evaluates whether v0.0.2 milestone is ready for branch merge / product-doc freeze.
- **Mobile companion (US-P0-12/13)** — can proceed on confirmed wire-safe DTOs + cross-project isolation.

## §7 — Acknowledgements

- **Mr. Zhang**: direct-seat fix landed cleanly with tight scope (5 files, +126/-9), matching Flux's packet §3 file list exactly. Tsc-clean + build-green on the seat. Pre-flight correct even without cargo available locally.
- **Flux**: bug diagnosis in `d25728a` was surgical — three distinct root causes correctly isolated, with root-cause analysis (xterm hardcoded size vs tmux pane size; pipe-pane only captures new output) rather than symptom patching. Verify delivery line-anchored every invariant. STATIC-VERIFIED cargo disposition with explicit `pub fn write` diff confirmation saves a toolchain roundtrip.
- **Mira**: packet was written to Mira but code landed via Mr. Zhang direct-seat path — Mira's Layer B surface (`SessionTerminal` + `SessionsWorkspace`) is what this fix extends, so the wiring is consistent with her prior work (A4-α/A4-β/sessions-tab).
- **Aegis**: R4 stage-gate held — if B1 Layer B had promoted prematurely, the three bugs would have blocked real seat-to-seat workflow. Flagging these at Layer B before UNCONDITIONAL promotion is exactly the right discipline.

---

*Lyra acceptance · 2026-05-11 · PROVISIONAL pending Mr. Zhang Layer B re-run · UNCONDITIONAL promotion follows re-run PASS · B1 + 008 combined Layer B window · SG-B stage-gate decision follows*
