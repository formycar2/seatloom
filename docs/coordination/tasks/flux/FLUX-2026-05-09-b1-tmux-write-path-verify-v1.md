# Flux Verify: B1 — Bidirectional tmux Write Path (v0.0.2)

[Lyra -> Flux] B1 delivered at commit `0c0f425`. 12-step smoke PASS, 9 acceptance criteria self-checked, 6/6 unit tests, scope clean (5 files). Run commit-pinned Layer A and write your verify delivery doc with 13-row matrix + R-rule matrix.

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-05-09-b1-tmux-write-path-verify-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-05-09 (late evening) |
| to | flux |
| priority | P0 (SG-B entry; B1 acceptance hinges on your PASS) |
| target_commit | `0c0f425fb14f6a5db3b1f0e8be0c7c70b8c0ea4d` (Nimbus B1) |
| target_branch | `track/infra-foundation` |
| delivery_doc | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-delivery-v1.md` (commit `b1d4b4b`) |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-v1.md` |
| flux_delivery_path | `docs/coordination/tasks/flux/FLUX-2026-05-09-b1-tmux-write-path-verify-delivery-v1.md` |
| tags | flux, verify, B1, SG-B, tmux, bidirectional, commit-pinned |

---

## 1. Summary

Nimbus's B1 implements the bidirectional write path on top of A3's read mirror. **Architectural choice that needs your eye**: he chose `tmux load-buffer -b seatloom - ; paste-buffer -b seatloom -t <target>` over the more obvious `send-keys -l -- <string>`, with rationale on NUL-byte fidelity, ARG_MAX safety, and `;`-chained atomicity inside one tmux invocation (eliminates Mutex). Round-trip evidence in delivery §3.

Scope held to 5 files (delivery §2). All 9 self-acceptance criteria line-pinned in delivery §4. Two flag-ups in delivery §7: (a) pre-existing clippy lints in `db/repositories.rs` (validated NOT a B1 regression via stash test), (b) stray 008 packet file in his working tree caught + unstaged before push (final commit 5 files exactly).

## 2. Verify scope

Same three-layer matrix as A4-β verify, scaled up for the larger surface (Rust pty + Tauri command + UI on the read path now becomes Rust pty write + Tauri command write + UI keystroke wiring + smoke harness extension):

### Layer A — automated checks (your seat)

```bash
git fetch && git checkout 0c0f425
git rev-parse HEAD                                # must equal 0c0f425fb14f6a5db3b1f0e8be0c7c70b8c0ea4d
git show 0c0f425 --stat                           # must be exactly 5 files / +209 / -30
```

Run from repo root unless noted:

```bash
cargo check -p seatloom-core
cargo check -p seatloom-tauri
cargo test -p seatloom-core --lib pty::           # must be 6/6 pass (4 existing + 2 new write_noop_*)
cargo clippy -p seatloom-core -- -D warnings      # see §3 below — Nimbus flagged pre-existing lints
cd ui && pnpm exec tsc --noEmit                   # must exit 0
cd ui && pnpm build                               # must exit 0
```

Then re-run his smoke harness with a fresh tmux session of your own naming:

```bash
tmux new-session -d -s flux-b1-verify-seatloom 'sleep 86400'
. "$HOME/.cargo/env" && cargo run --example tmux_mirror_smoke -- flux-b1-verify-seatloom 2>&1 | tail -40
tmux kill-session -t flux-b1-verify-seatloom
```

Must emit both `SMOKE_PASS probe=true ls=true bytes=<>0` and `SMOKE_B1_PASS round_trip=true ctrl_c=true`. Capture verbatim.

### Layer A — 13-row static invariant matrix

| # | Invariant | Where to look |
|---|-----------|---------------|
| 1 | Scope held to 5 files exactly | `git show --stat 0c0f425` |
| 2 | i18n.ts unchanged | `git diff 4477ab6..0c0f425 -- ui/src/i18n.ts` empty |
| 3 | No backend portable-pty resurrection | `grep -n portable_pty crates/seatloom-core/Cargo.toml` empty (nix only) |
| 4 | `PtySession::write` real impl, not stub | `crates/seatloom-core/src/pty/mod.rs:324` — `pub fn write(&self, bytes: &[u8])` (not `_bytes`) |
| 5 | Buffer-path uses `;` chaining single invocation (atomicity claim) | grep `"load-buffer"` and `"paste-buffer"` in same `Command::new("tmux").args` block |
| 6 | `-b seatloom` named buffer (user paste-buffer untouched) | grep `"-b", "seatloom"` two occurrences in same write |
| 7 | shutdown guard at write entry | `self.shutdown.load(...)` early-return `Ok(())` in `fn write` |
| 8 | empty-bytes guard | `bytes.is_empty()` early-return `Ok(())` in `fn write` |
| 9 | `cmd_pty_write` + `cmd_pty_write_bytes` reach `session.write()` | `src-tauri/src/commands/session_cmds.rs:270` (`sess.write(data.as_bytes())`), `:285` (`sess.write(&bytes)`) |
| 10 | `SessionTerminal` has no `disableStdin: true` | grep result must be empty |
| 11 | `SessionTerminal` wires `term.onData` → `api.ptyWriteBytes` via `TextEncoder` | grep `term.onData` + `api.ptyWriteBytes` + `TextEncoder` co-located |
| 12 | `SessionsWorkspace` banner blue (#DBEAFE family) + "v0.0.2" + "Bidirectional" | grep `v0.0.2` + `Bidirectional` |
| 13 | `SessionsWorkspace` pill v0.0.2 attach-only language preserved | grep `attach-only` (must remain — R1 unchanged) |

### Layer A — R-rule matrix (5 rows)

| Rule | Verify-by |
|------|-----------|
| R1 attach-only | `cargo check` confirms no portable-pty / no spawn-by-SeatLoom; tmux is the runtime, SeatLoom invokes tmux client commands only |
| R3 failure-isolation | smoke Step 12 `R3 OK: tmux session ... still running after kill` reproduced |
| v0.0.2 read-only on `disableStdin` removal | inverse — confirm `disableStdin` is gone, `cursorBlink: true` is on (UI must accept input) |
| Buffer-path atomicity | smoke harness has no Mutex on PtySession; concurrent write test (if you want to add one) confirms no interleaving — but Nimbus's `;`-chain argument is structurally sound; manual reproduction sufficient for v0.0.2 |
| Round-trip fidelity (NUL/Ctrl-C/UTF-8) | smoke Step 11 `Ctrl-C OK` reproduces (the `sleep 30` interrupt test fails loudly if SIGINT path is broken) |

## 3. Pre-existing clippy lints — Nimbus flag-up §7.1

Nimbus reports `cargo clippy -p seatloom-core -- -D warnings` fails on 3 `doc_lazy_continuation` errors in `crates/seatloom-core/src/db/repositories.rs`. He validated these are present on parent commit `b916912` (NOT a B1 regression) via `git stash; clippy; stash pop`.

**Your job**: re-run the stash test independently to confirm. If confirmed pre-existing:
- Treat clippy failure as a non-blocking yellow flag (do NOT fail B1 verify on this).
- Note in your verify delivery §5 (Known non-blockers) that B1 acceptance proceeds and a sibling cleanup packet is recommended for `repositories.rs` doc-lint cleanup.

If clippy regresses on B1 (i.e., the 3 errors are NEW or there are MORE errors at `0c0f425` than at `b916912`), then it IS a B1 regression — flag and HOLD.

## 4. Layer B — runtime (Mr. Zhang via Aegis)

Out of your scope; rolls into Lyra acceptance window. The runtime checklist will be:

1. `pnpm tauri dev`, attach to a real seat tmux session via SessionsWorkspace.
2. Type `echo hello\n` in the xterm → "hello" appears in the tmux pane.
3. Run `sleep 30` in the tmux pane via xterm typing, then send Ctrl-C from the xterm → shell prompt returns immediately.
4. Paste a multi-line block into the xterm → all lines arrive in the pane.
5. Close the SeatLoom tab → tmux session continues running.
6. Restart SeatLoom → re-attach → mirror resumes.

Mr. Zhang has been running these for SG-A; same pattern.

## 5. Delivery doc requirements (your output)

Write to `docs/coordination/tasks/flux/FLUX-2026-05-09-b1-tmux-write-path-verify-delivery-v1.md`. Same structure as your A4-β verify delivery doc:

1. T3 frontmatter (template, subtype=verification_delivery, status=delivered, target_commit=0c0f425, packet_ref=this packet).
2. §1 verbatim outputs of all §2 commands (verbatim — paste full stdout/stderr).
3. §2 13-row static invariant matrix with PASS/FAIL + line evidence per row.
4. §3 R-rule matrix (5 rows) with PASS/FAIL.
5. §4 known non-blockers: clippy-lint situation + the stale 008 packet file note (both were pre-emptively flagged by Nimbus — confirm or refute his pre-emption).
6. §5 verdict: `Layer A PASS` / `Layer A HOLD` / `Layer A FAIL` with one-paragraph rationale.
7. §6 carry-forwards (smoke step output verbatim).
8. §7 next-action pointer (→ Lyra acceptance + Mr. Zhang Layer B).

Same verbatim discipline as A4-β: paste tool output, do not summarize.

## 6. Concurrency note

008 (NIMBUS-2026-05-09-schema-008-project-isolation-v1.md) is dispatched in parallel. If Nimbus delivers 008 while you're mid-verify on B1, finish B1 first then start a separate 008 verify. Do not bundle.

## 7. Timeline

ASAP. SG-B entry sits on B1 acceptance. Mr. Zhang Layer B can run in parallel with your Layer A if Aegis schedules it; doesn't gate your verify.

---

*Dispatched by Lyra · 2026-05-09 late-evening · Same three-layer protocol as A4-β · 13-row static + R-rule matrices · 12-step smoke reproduction*
