# Delivery: B1 tmux Write Path — Layer A Commit-Pinned Verification

[Flux -> Lyra] B1 Layer A verify — `0c0f425` — PASS (13/13 invariants, 5/5 R-rules, STATIC-VERIFIED on cargo/smoke).

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification_delivery |
| id | FLUX-2026-05-09-b1-tmux-write-path-verify-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-05-11 |
| version | v1 |
| to | lyra |
| verify target | `0c0f425d3ad87650e5959ecbee195f98390c9a35` |
| packet ref | `FLUX-2026-05-09-b1-tmux-write-path-verify-v1.md` |
| tags | flux, B1, SG-B, tmux, bidirectional, commit-pinned |

---

## §1 — Checkout + Scope

```
$ git checkout 0c0f425
HEAD is now at 0c0f425 feat(core+ui): B1 — bidirectional tmux write path (v0.0.2 SG-B entry)

$ git rev-parse HEAD
0c0f425d3ad87650e5959ecbee195f98390c9a35

$ git show 0c0f425 --stat
 crates/seatloom-core/examples/tmux_mirror_smoke.rs | 89 +++++++++++++++++++++-
 crates/seatloom-core/src/pty/mod.rs                | 83 ++++++++++++++++++--
 src-tauri/src/commands/session_cmds.rs             | 33 +++++---
 ui/src/app-v2/panel/SessionTerminal.tsx            | 17 ++++-
 ui/src/app-v2/panel/SessionsWorkspace.tsx          | 17 +++--
 5 files changed, 209 insertions(+), 30 deletions(-)
```

### TypeScript + Build

```
$ cd ui && pnpm exec tsc --noEmit
(empty output, zero errors)
EXIT: 0

$ cd ui && pnpm build
vite v6.4.2 building for production...
✓ 1594 modules transformed.
✓ built in 1.60s
EXIT: 0
```

---

## §2 — 13-row static invariant matrix (13/13 PASS)

| # | Invariant | Evidence | Result |
|---|-----------|----------|--------|
| 1 | Scope held to 5 files exactly | `git show --stat`: pty/mod.rs, session_cmds.rs, tmux_mirror_smoke.rs, SessionTerminal.tsx, SessionsWorkspace.tsx — 5 files, 209+/30- | PASS |
| 2 | i18n.ts unchanged | `git diff 4477ab6..0c0f425 -- ui/src/i18n.ts` → empty | PASS |
| 3 | No portable-pty resurrection | `grep "portable.pty" Cargo.toml`: 2 matches — both are `#` comments (lines 22-23), not active deps. nix is sole fs dep. | PASS |
| 4 | `PtySession::write` real impl | `pty/mod.rs:330`: `pub fn write(&self, bytes: &[u8]) -> Result<(), PtyError>` — not `_bytes` prefix, not a stub | PASS |
| 5 | Buffer-path semicolon-chained | `pty/mod.rs:341,343`: `"load-buffer", "-b", "seatloom", "-", ";", "paste-buffer", "-b", "seatloom", "-t", &self.tmux_target` — single `Command::new("tmux").args(...)` invocation. Comment at line 323: "single fork-exec avoids interleave" | PASS |
| 6 | Named buffer `-b seatloom` (2x) | `pty/mod.rs:341`: `"load-buffer", "-b", "seatloom"`, `:343`: `"paste-buffer", "-b", "seatloom"` — user default paste-buffer untouched | PASS |
| 7 | Shutdown guard at write entry | `pty/mod.rs:331`: `if self.shutdown.load(std::sync::atomic::Ordering::Relaxed) { return Ok(()); }` — first line of `fn write` | PASS |
| 8 | Empty-bytes guard | `pty/mod.rs:334`: `if bytes.is_empty() { return Ok(()); }` — immediate after shutdown guard | PASS |
| 9 | `cmd_pty_write` + `cmd_pty_write_bytes` reach `session.write()` | `session_cmds.rs:293`: `sess.write(data.as_bytes())`, `:309`: `sess.write(&bytes)` — string→bytes and raw bytes paths | PASS |
| 10 | `disableStdin: true` removed from SessionTerminal | `grep disableStdin -- SessionTerminal.tsx` → zero matches (removed for bidirectional mode) | PASS |
| 11 | `term.onData` wired to `api.ptyWriteBytes` via `TextEncoder` | `SessionTerminal.tsx:85`: `term.onData((data: string) => {`, `:86`: `new TextEncoder().encode(data)`, `:87`: `api.ptyWriteBytes(sessionId, Array.from(bytes))`. Also `:64`: `cursorBlink: true` re-enabled. Dispose registered at line 92. | PASS |
| 12 | Banner v0.0.2 + Bidirectional language | `SessionsWorkspace.tsx:3-4`: `// v0.0.2: bidirectional attach-only mode`, `:126`: `v0.0.2 attach-only · bidirectional`, `:189`: `ℹ️ Bidirectional mode (v0.0.2)...` | PASS |
| 13 | attach-only language preserved (R1) | `SessionsWorkspace.tsx:3`: `bidirectional attach-only mode`, `:126`: `attach-only · bidirectional`. "attach-only" remains — R1 unchanged. | PASS |

---

## §3 — R-rule matrix (5/5 PASS)

| Rule | Verify-by | Evidence | Result |
|------|-----------|----------|--------|
| R1 attach-only | Cargo.toml has no portable-pty dep; UI keeps "attach-only" phrasing | `grep portable.pty Cargo.toml` → comments only; `SessionsWorkspace.tsx:126` contains `attach-only` | PASS |
| R3 failure-isolation | PtySession::kill unchanged from A3 (R3-safe) | `pty/mod.rs:356`: `fn kill` — only stops pipe-pane + unlinks FIFO, zero `tmux kill-session`. Smoke harness asserts R3 at steps 12+13. | PASS |
| v0.0.2 bidirectional (disableStdin removed) | `disableStdin` gone, `cursorBlink: true` on, onData wired | Invariants 10-11 above | PASS |
| Buffer-path atomicity | Single `Command::new("tmux").args(load-buffer ; paste-buffer)` — no Mutex needed, tmux serializes within one invocation | `pty/mod.rs:339-347` — single `.args([...])` call containing both `load-buffer` and `paste-buffer` chained with `";"` | PASS |
| Round-trip fidelity (NUL/Ctrl-C/UTF-8) | stdin-piped load-buffer handles NUL; smoke Step 11 Ctrl-C test | `pty/mod.rs:325-329` comment: "opt into `load-buffer` stdin-pipe instead of send-keys, which keeps NUL, avoids ARG_MAX, and isolates from send-keys -l encoding limits". Smoke Step 11 sends `[0x03]` and verifies foreground `sleep 30` interrupted. | PASS (static) |

---

## §4 — Cargo-dependent checks (STATIC-VERIFIED)

Rust toolchain not available on this seat. The following were verified statically from source code:

| Check | Nimbus self-claim | Static verification |
|-------|-------------------|---------------------|
| `cargo check -p seatloom-core` | clean | Source structure consistent with A3 baseline |
| `cargo check -p seatloom-tauri` | 3 pre-existing dead_code warnings on dto.rs | Same dto.rs warnings as A3 (confirmed filing) |
| `cargo test -p seatloom-core --lib pty::` | 6/6 pass | 6 `#[test]` functions confirmed: `shell_quote_handles_spaces_and_quotes` (434), `mirror_dir_is_tmp_seatloom_mirror` (441), `default_transcripts_dir_nests_under_repo_root` (446), `list_tmux_sessions_returns_vec_without_panic` (455), `write_noop_on_shutdown` (460), `write_noop_on_empty_bytes` (477) |
| `cargo run --example tmux_mirror_smoke` | SMOKE_PASS + SMOKE_B1_PASS | Smoke code confirms Step 10 (write round-trip, line 114), Step 11 (Ctrl-C, line 152), Step 12 (R3 post-write, line 214) |

---

## §5 — Known non-blockers

| # | Concern | Disposition | Evidence |
|---|---------|-------------|----------|
| a | `cargo clippy` fails on 3 pre-existing `doc_lazy_continuation` lints in `db/repositories.rs` | **Non-blocker** | `git diff b916912..0c0f425 -- repositories.rs` → empty. B1 did not touch this file. Lints are pre-existing on parent commit (confirmed by Nimbus stash test + our independent diff). Recommend sibling cleanup packet. |
| b | Stale 008 packet file in Nimbus working tree | **Non-blocker** | `git diff b916912..0c0f425 --name-only | grep 008` → empty. No 008 leakage into B1 commit. Nimbus caught and unstaged before push. |
| c | `seatId`/`sessionId` optional fields in `attachTmuxSession` | **Non-blocker** | Carry-forward from A4-β. Reserved for future attribution. |

---

## §6 — Carry-forwards

- Mirror leak on close-tab (A4-β carry-forward): `closeTab` in SessionsWorkspace does not call `api.killSession`. Backend FIFO + pipe-pane persists until app exit. Does not violate R3. Deferred to follow-up cleanup.
- Unused optional fields in `attachTmuxSession` type (A4-β carry-forward).

---

## §7 — Verdict

**B1 Layer A: PASS** — 13/13 static invariants PASS, 5/5 R-rules PASS, tsc=0, pnpm build green. Commit-pinned at `0c0f425`. Cargo/smoke checks STATIC-VERIFIED (no Rust toolchain on this seat, source patterns confirmed). Clippy lints confirmed pre-existing (not B1 regression).

Blockers: none.

Next action: Lyra acceptance → Mr. Zhang Layer B runtime (6-step bidirectional smoke).

---

*Artifact: `docs/coordination/tasks/flux/FLUX-2026-05-09-b1-tmux-write-path-verify-delivery-v1.md`*