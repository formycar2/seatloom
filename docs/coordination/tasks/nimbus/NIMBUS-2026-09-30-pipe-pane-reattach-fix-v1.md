# Task: Fix the tmux mirror re-attach — `pipe-pane -o` is a toggle, so the second attach kills it

[Lyra -> Nimbus] Mira found this while building UM0 and reported it without touching Rust, as her
packet required. **I reproduced it empirically before dispatching** — see §1. It is small, bounded,
and it is the one thing standing between UM0 and a *reliably* live mirror.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | nimbus |
| priority | P0 (gates reliable UM0) |
| depends_on | `docs/coordination/tasks/mira/MIRA-2026-09-30-live-mirror-um0-delivery-v1.md` §0.1 |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` (**created**) |
| build location | `/Users/…/seatloom-worktrees/nimbus` (already on the branch, clean) |
| delivery path | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-delivery-v1.md` |
| tags | nimbus, pty, tmux, live-mirror, um0, bugfix |

---

## 1. The bug, reproduced

`crates/seatloom-core/src/pty/mod.rs:271`:

```rust
.args(["pipe-pane", "-t", &tmux_target, "-o", &cat_cmd])
```

The comment above it says `-o` *"makes the pipe-pane command invoke only once (so we don't
duplicate if attach is called twice)"*. **That reading is wrong**, and I verified it on a
throwaway tmux session rather than from the man page:

```
initial pane_pipe  = 0
after 1st  -o call = 1
after 2nd  -o call = 0      <- the pipe CLOSED
after plain (no -o)= 1
after 2nd plain    = 1      <- replaces, stays open
```

`-o` toggles. So: first attach opens the pipe and the mirror works; **second attach closes it** —
`pane_pipe=0`, zero bytes, and `cmd_attach_tmux_session` still returns a successful
`LiveSessionDto`. It looks live and is dead. Any page reload, any second viewer, any seat already
mirrored by the running app hits this.

Mira's recommended fix is correct: **drop `-o`.** Plain `pipe-pane <cmd>` replaces an existing
pipe unconditionally, which is exactly re-attach semantics.

## 2. Scope

1. **`pty/mod.rs:271` — remove `-o`.** Replace the misleading comment with what the flag actually
   does, citing the measured behaviour above so nobody "restores" it later.
2. **Check the detach/kill path at `:477`** — `pipe-pane -t <target>` with no command and no `-o`.
   That unconditionally disables the pipe, which is right for detach, but confirm and state the
   consequence: **one client detaching stops the mirror for every other client on that pane.**
   Do not redesign multi-client ownership in this packet — say whether it needs a follow-up.
3. **Surface a dead mirror instead of reporting success.** Today attach returns OK with
   `pane_pipe=0` and no bytes. At minimum, verify `pane_pipe` after issuing `pipe-pane` and return
   a typed error if the pipe is not open. A silent dead mirror is the actual user harm here — it is
   what made the UI look real when it was not.

## 3. Out of scope

- **No UI.** Mira owns `ui/src`; her UM0 packet is at gate. Zero `.tsx`.
- **No multi-client pipe ownership model** (ref-counting, per-client FIFOs). If §2.2 shows it is
  needed, recommend a follow-up packet — do not build it here.
- **No `.sql`, no write-API changes.** Step 2 is merged and accepted.
- **Do not kill or restart any seat tmux session.** Five seats are live and working.

## 4. Verification

```
cargo check -p seatloom-core && cargo test -p seatloom-core --lib
cargo clippy -p seatloom-core -- -D warnings && cargo fmt --all --check
```

Plus the behavioural proof, **on a throwaway tmux session you create and kill — never on a seat**:

1. attach once → `pane_pipe=1`, bytes flow;
2. attach **again** → `pane_pipe` stays `1` and bytes **still** flow (this is the regression that
   must not come back);
3. detach → pipe closes, FIFO unlinked, **tmux session still alive**;
4. the §2.3 typed error fires when the pipe fails to open.

Paste `pane_pipe` readings at each step verbatim.

**Leave tmux exactly as you found it.** All five seats currently read `pane_pipe=0` — I checked.
Confirm the same at the end.

## 5. Done-definition

1. `-o` removed from the attach path; comment corrected with the measured semantics.
2. Detach path assessed; the shared-pipe consequence stated; follow-up recommended or explicitly
   ruled unnecessary.
3. Attach returns a **typed error** rather than success when the pipe is not open.
4. Re-attach proven: second attach keeps `pane_pipe=1` and bytes flowing, with readings.
5. Tmux left clean; no seat session killed; all five seats back at `pane_pipe=0`.
6. Layer A green, verbatim.
7. Zero `.tsx`, zero `.sql`.
8. Flux commit-pinned verify PASSes.

---

*Dispatched by Lyra · 2026-09-30 · Mira reported it and did not touch Rust, correctly · I
reproduced the toggle empirically before dispatching · gates reliable UM0*
