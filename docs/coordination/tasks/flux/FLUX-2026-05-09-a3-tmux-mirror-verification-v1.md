# Commit-Pinned Verify: SG-A §A3 tmux Mirror Read Path (Nimbus `fca4fe0`)

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-05-09-a3-tmux-mirror-verification-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-09 |
| version | v1 |
| to | flux |
| acceptance owner | lyra |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v001-tmux-attach-read-path-v1.md` (packet), `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-v001-tmux-attach-read-path-delivery-v1.md` (delivery), `docs/coordination/reviews/2026-05-09-aegis-seatloom-tmux-mirror-architecture-v1.md` (R1-R5 design authority) |
| tags | flux, verification, SG-A, A3, commit-pinned, tmux-mirror |

---

## Pinned target

- **Commit**: `fca4fe01eb5aad70121456b5cfecd5a7b18ab31b`
- **Branch**: `track/infra-foundation`
- **Packet**: SG-A §A3 v0.0.1 tmux attach read path
- **Owner**: Nimbus

---

## Layer A — Commit-pinned static verification

### A.1 Tree state

```
$ git rev-parse fca4fe0
$ git show fca4fe0 --stat --format=""
$ git status --short
```

Assert: commit exists, staged scope matches delivery doc (6 files: `Cargo.lock`, `crates/seatloom-core/Cargo.toml`, `crates/seatloom-core/examples/tmux_mirror_smoke.rs`, `crates/seatloom-core/src/pty/mod.rs`, `src-tauri/src/commands/session_cmds.rs`, `src-tauri/src/main.rs`), no UI files touched, no `infra/postgres/**` touched.

### A.2 Scope audit

```
$ git show fca4fe0 --name-only | grep -E "^(ui/|infra/postgres/|docs/coordination/DOCUMENT_TEMPLATES\.md)"
```

Expected: empty (A3 is backend + examples only; no UI scope, no schema scope, no governance scope).

### A.3 Dependency hygiene

```
$ grep -E "^portable-pty|^nix" crates/seatloom-core/Cargo.toml
```

Expected: `portable-pty` absent; `nix` present with version. Matches R3 (SeatLoom not parent of any wrapped CLI).

### A.4 Rust build + unit tests

```
$ . "$HOME/.cargo/env" && cargo check -p seatloom-tauri
$ . "$HOME/.cargo/env" && cargo test -p seatloom-core --lib pty::
```

Expected: `cargo check` exit 0; `cargo test -p seatloom-core --lib pty::` reports `4 passed; 0 failed`.

### A.5 R3 static invariants (grep-pinned)

| Invariant | grep target | Expected |
|---|---|---|
| No `tmux kill-session` in kill path | `grep -n "kill-session" crates/seatloom-core/src/pty/mod.rs` | empty OR present only in test/doc context |
| No `tmux kill-pane` in kill path | `grep -n "kill-pane" crates/seatloom-core/src/pty/mod.rs` | empty |
| `kill` calls only `pipe-pane -t <target>` + `remove_file` | Read `fn kill` body | matches delivery-doc citation at mod.rs:356 |
| FIFO under `/tmp/seatloom-mirror/` | `grep -n "seatloom-mirror" crates/seatloom-core/src/pty/mod.rs` | present, matches `mirror_dir_is_tmp_seatloom_mirror` test |
| Session name filter `-seatloom` suffix | `grep -n "\\-seatloom" crates/seatloom-core/src/pty/mod.rs src-tauri/src/commands/session_cmds.rs` | present in `list_tmux_sessions` |

### A.6 Transitional shim audit

```
$ grep -n "cmd_launch_session" src-tauri/src/commands/session_cmds.rs
```

Expected: the shim routes `runtime: 'tmux'` / `'tmux-mirror'` to `cmd_attach_tmux_session`; any other runtime value fails loudly. Document this behavior verbatim in the verify delivery (future callers need to see it).

---

## Layer B — Headless smoke harness verification

Re-run the Nimbus smoke harness from scratch against commit `fca4fe0` in a clean environment:

```
$ git checkout fca4fe0
$ tmux new-session -d -s a3-flux-smoke-seatloom 'sh -c "while :; do sleep 60; done"'
$ tmux list-sessions | grep a3-flux-smoke-seatloom
$ . "$HOME/.cargo/env" && cargo run -p seatloom-core --example tmux_mirror_smoke -- a3-flux-smoke-seatloom 2>&1 | tail -40
```

**Expected output markers** (verbatim in delivery doc):
- `list_tmux_sessions() returned <N> sessions (filtered by -seatloom suffix)`
- `attached: id=..., fifo=/tmp/seatloom-mirror/...fifo, tmux_target=a3-flux-smoke-seatloom:0`
- `received <>0 bytes from mirror`
- `probe_marker_seen=true ls_output_seen=true`
- `resize(24,120) OK`
- `kill OK: fifo unlinked at /tmp/seatloom-mirror/...fifo`
- `R3 OK: tmux session a3-flux-smoke-seatloom still running after kill`
- `SMOKE_PASS probe=true ls=true bytes=<>0`

**Post-smoke R3 re-assert**:

```
$ tmux list-sessions | grep a3-flux-smoke-seatloom && echo "R3_HELD"
$ tmux kill-session -t a3-flux-smoke-seatloom
```

Expected: `R3_HELD` before manual cleanup.

### Smoke failure-mode probes (optional but preferred)

| Probe | Command | Expected |
|---|---|---|
| No tmux server | `pkill -0 tmux 2>/dev/null && tmux kill-server; cargo run --example tmux_mirror_smoke -- nothing-there` | harness exits with a clear error, no panic |
| Non-matching session name | `tmux new -d -s foo-bar` (no `-seatloom` suffix) + `list_tmux_sessions()` | `foo-bar` filtered out |
| Session disappears mid-attach | attach, then `tmux kill-session -t <target>`, observe subscriber | `session:exit` event emitted |

These probe failures do not block PASS; record them as informational if time allows.

---

## Layer C — tsc + UI build cross-check (negative)

A3 is backend-only, but confirm no UI regression snuck in:

```
$ cd ui && pnpm exec tsc --noEmit
$ cd ui && pnpm build
```

Expected: tsc exit 0, build exit 0. Same output as prior SG-A verify (commit 34bd0a1 baseline); no new errors introduced.

---

## Verdict schema

Flux reports per-layer verdicts in the delivery:

| Layer | Scope | Expected verdict |
|---|---|---|
| A.1–A.6 | commit + scope + static invariants + Rust build | PASS |
| B | headless smoke harness | PASS |
| C | UI tsc + build (negative cross-check) | PASS |

Overall: `A3 fca4fe0: PASS` on all three layers → Lyra finalizes SG-A §A3 acceptance.

Any FAIL → pin the failing invariant verbatim, revert `fca4fe0` to HOLD, Lyra issues corrective packet to Nimbus.

---

## Runtime observation (Mr. Zhang, optional)

A3 does not require Tauri runtime — the smoke harness is headless. However, if Mr. Zhang has `pnpm tauri dev` running after Mira's A4 lands, the following observation can be recorded alongside the A4 Layer B checklist:

- Attach to an existing tmux session via the SessionsWorkspace UI → output streams into xterm within 1 s of typing in tmux.

This observation is not required for A3 PASS; it will be captured during A4 Layer B verification.

---

## Why this packet is issued standalone (not bundled with the SG-A combined verify)

The existing FLUX-2026-05-09-sg-a-combined-verification-delivery-v1 (commit 3c476aa) covers A1 + v01 + A4-α + A2 at HEAD `34bd0a1`. A3 (`fca4fe0`) landed after that verify and changes only backend/Rust files — a clean boundary. Separate verify packet keeps commit pinning unambiguous and avoids re-running the full UI-side Layer A work that was already PASS at `34bd0a1`.

SG-A closes when all five packets (A1, v01, A4-α, A2, A3) PASS both Flux verify AND Mr. Zhang's Layer B runtime (where applicable). A4 is still Mira's dispatch and verifies separately.

---

## Next action

Flux: run Layers A + B + C against `fca4fe0`, produce `FLUX-2026-05-09-a3-tmux-mirror-verification-delivery-v1.md` with verbatim command outputs, file under `docs/coordination/tasks/flux/`.

Lyra: on PASS, file T5 acceptance for A3; update MEMORY.md; dispatch A4-β as next Mira packet.

---

*Packet issued by Lyra · 2026-05-09 · SG-A §A3 v0.0.1 verify*
