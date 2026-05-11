# Acceptance: B1 — Bidirectional tmux Write Path (v0.0.2)

[Lyra -> Nimbus] B1 Layer A PASS, Lyra acceptance issued. Mr. Zhang Layer B runtime triggered via Aegis. Same three-layer pattern as A4-β.

| Field | Value |
|---|---|
| template | T3 |
| subtype | acceptance |
| id | LYRA-2026-05-09-nimbus-b1-tmux-write-path-acceptance-v1 |
| status | accepted |
| author | lyra |
| date | 2026-05-11 |
| version | v1 |
| to | nimbus, flux, aegis, mr_zhang |
| target_commit | `0c0f425d3ad87650e5959ecbee195f98390c9a35` |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-v1.md` |
| delivery_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-delivery-v1.md` (commit `b1d4b4b`) |
| flux_verify_ref | `docs/coordination/tasks/flux/FLUX-2026-05-09-b1-tmux-write-path-verify-delivery-v1.md` |
| layer_a_verdict | PASS (Flux: 13/13 static + 5/5 R-rules + tsc=0 + build green; cargo/smoke STATIC-VERIFIED with line-anchored confirmation) |
| layer_b_verdict | PENDING — Mr. Zhang via Aegis (6-step bidirectional smoke per §3 below) |
| tags | lyra, B1, SG-B, tmux, bidirectional, v0.0.2, layer-a-pass, layer-b-pending |

---

## §1 — Verdict

**PROVISIONAL ACCEPTED — Layer A unconditional PASS.** Acceptance promotes to UNCONDITIONAL upon Mr. Zhang Layer B runtime confirmation. SG-B entry remains gated on Layer B per A4-β precedent.

Layer A evidence chain is solid:
- Nimbus delivery `0c0f425` self-pinned 9 acceptance criteria with line evidence (delivery §4)
- Flux Layer A verify `FLUX-2026-05-09-b1-tmux-write-path-verify-delivery-v1.md` independently confirmed 13/13 static invariants + 5/5 R-rules with line-anchored evidence
- 12-step smoke harness output verbatim in delivery §5.7 (`SMOKE_PASS probe=true ls=true bytes=2284` + `SMOKE_B1_PASS round_trip=true ctrl_c=true` + `R3 OK reasserted post-write`)
- 6/6 unit tests including 2 new (`write_noop_on_shutdown`, `write_noop_on_empty_bytes`)
- TypeScript zero errors, pnpm build success

Architectural choice (load-buffer + paste-buffer single-invocation with `;` chaining + named `-b seatloom` buffer) is sound. Aegis ratified the choice in commit `ebc0e68`. Buffer-path resolves NUL-byte fidelity, ARG_MAX safety, and atomicity (no Mutex on `PtySession`) in one decision. Round-trip evidence including Ctrl-C SIGINT delivery proven in smoke Step 11.

## §2 — Three-layer status

| Layer | Owner | Status | Evidence |
|---|---|---|---|
| Delivery | Nimbus | ✓ delivered | `0c0f425` (5 files, +209/-30) + delivery doc `b1d4b4b` |
| Pre-flight (Aegis scope/drift) | Aegis | ✓ ratified | `ebc0e68` consolidation + verbal acknowledgement of buffer-path choice |
| Layer A commit-pinned verify | Flux | ✓ PASS | 13/13 static + 5/5 R-rules; verify delivery `FLUX-2026-05-09-b1-tmux-write-path-verify-delivery-v1.md` |
| Layer B runtime | Mr. Zhang via Aegis | ⏳ PENDING | 6-step bidirectional smoke per §3 below |
| Lyra final acceptance | Lyra | ⏳ provisional pending Layer B | This document promotes to UNCONDITIONAL on Layer B PASS |

## §3 — Mr. Zhang Layer B — 6-step bidirectional smoke

Mr. Zhang via Aegis runs these against `pnpm tauri dev` at HEAD `0c0f425` or descendant. Same observation pattern as A4-β.

| Step | Action | Expected | Verifies |
|------|--------|----------|----------|
| 1 | `pnpm tauri dev` boot, navigate to SessionsWorkspace | Banner reads "ℹ️ Bidirectional mode (v0.0.2)" in blue, pill reads "v0.0.2 attach-only · bidirectional", attach-only language preserved | UI v0.0.2 chrome flip, R1 wording |
| 2 | Attach to a real seat tmux session, type `echo hello\n` in the xterm | "hello" appears in the tmux pane (visible via `tmux attach` in a separate terminal) | Forward keystroke path: xterm.onData → ptyWriteBytes → cmd_pty_write_bytes → session.write → tmux load-buffer/paste-buffer |
| 3 | In the xterm, run `sleep 30`, then send `Ctrl-C` | Shell prompt returns immediately (within ~100ms); `sleep` does NOT block for 30s | Raw byte 0x03 round-trips as SIGINT, not interpreted as tmux key name |
| 4 | Paste a multi-line block (3+ lines) into the xterm | All lines arrive in the pane; no truncation, no interleave with other writes | NUL-safe stdin pipe + `;`-chained atomicity |
| 5 | Close the SeatLoom tab via the × button | Tmux session continues running externally (verify with `tmux list-sessions`) | R3 failure-isolation under write path; A4-β closeTab carry-forward (mirror leak ok, kill-session preserved) |
| 6 | Restart SeatLoom (kill app, reopen), re-attach to the same tmux session | Mirror resumes; bytes typed in tmux during the restart appear in the new attach (via tmux scrollback) | Restart-recovery, persistence of tmux session, FIFO recreation |

If all 6 PASS → Lyra promotes this acceptance to UNCONDITIONAL and SG-B entry is closed.

If any step FAILs → HOLD with named step + observation; Nimbus dispatches a B1-v2 fix.

## §4 — Carry-forwards (acknowledged, not blocking)

Three from A4-β + B1, all flagged in delivery §6 + verify delivery §6:

1. **Mirror leak on close-tab** (A4-β origin, unchanged in B1) — `closeTab` does not call `api.killSession`; FIFO + pipe-pane persist until app exit. Not an R3 violation. **Disposition**: sibling cleanup packet, post-SG-B.
2. **Pre-existing clippy lints in `db/repositories.rs`** (3× `doc_lazy_continuation`) — Flux confirmed via independent `git diff b916912..0c0f425 -- repositories.rs` empty. NOT a B1 regression. **Disposition**: standalone cleanup packet.
3. **Unused optional fields** (`seatId`, `sessionId`) in `attachTmuxSession` (A4-β origin) — reserved for future seat attribution. **Disposition**: leave as-is until used.

## §5 — Carry-forwards from B1 itself

Flagged in delivery §6, all v0.0.3+ scope:

- **Bracketed-paste handling** — buffer path pastes raw bytes; bracketed-paste-aware apps (vim with `:set paste`, bash readline) won't see `\e[200~`/`\e[201~` delimiters. Refinable later via xterm `onPaste` hook.
- **Single shared `-b seatloom` buffer across sessions** — fine within one process (tmux serializes); flagged for multi-process future.
- **No per-keystroke rate limit** — add client-side coalescing only if it becomes a problem.

## §6 — What this acceptance unlocks

- **SG-B (B1) entry closes** on Layer B PASS — first bidirectional v0.0.2 capability shipped.
- **Mr. Zhang can drive seats via SeatLoom IM** as a real workflow surface, not just a read-only mirror.
- **Next packets in queue**:
  - 008 (multi-project schema, Nimbus, in flight) — parallel, not blocking.
  - B2 (`cmd_append_supervisor_message` → seat write path with canonical_events emission) — needs 008 first (canonical_events.project_id).
  - Future closeTab cleanup (A4-β/B1 mirror leak).
  - Pre-existing clippy cleanup (`db/repositories.rs`).

## §7 — Acknowledgements

- **Nimbus**: clean architectural choice (buffer path with `;` chaining), pre-emptive flag-ups (clippy non-regression + stray 008 caught), §10 corrections to my 008 packet's `event_object_refs` SQL. The smoke harness extension (Steps 10-12) with `R3 OK reasserted post-write` is the right shape for write-path verify going forward.
- **Flux**: independent line-anchored verify in <2 hours after dispatch. Stash-test confirmation of clippy non-regression saves a sibling packet roundtrip.
- **Aegis**: decision-2 correction caught my over-extension on cross-point (c). Buffer-path ratification + consolidation adjudication landed clean.

---

*Lyra acceptance · 2026-05-11 · PROVISIONAL pending Mr. Zhang Layer B · same three-layer protocol as A4-β · UNCONDITIONAL promotion follows Layer B PASS*
