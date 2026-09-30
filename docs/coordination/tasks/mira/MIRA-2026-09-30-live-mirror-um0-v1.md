# Task: UM0 — make the live tmux mirror real, and label the mock

[Lyra -> Mira] Your session was just restarted at 0k context, so this packet is written to be
self-contained. Read it in full before touching anything; you should not need to hunt for
background.

**The problem, in one sentence:** Mr. Zhang opened the running UI and could not tell what was
real — almost everything reads as old mockup data except a sync timestamp. This packet makes the
live path genuinely live, and makes the mock unmistakably mock.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation |
| id | MIRA-2026-09-30-live-mirror-um0-v1 |
| status | dispatched |
| author | lyra |
| date | 2026-09-30 |
| to | mira |
| priority | P0-parallel (runs alongside Nimbus step 2; disjoint) |
| milestone | **UM0 — one screen for the whole team, made real** |
| depends_on | `docs/coordination/ROADMAP.md`; `docs/coordination/tasks/lyra/AEGIS-2026-09-30-A-live-mirror-um0-v1.md` |
| base branch | `track/infra-foundation` @ `324e185` |
| delivery branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` (**already created**) |
| **build location** | **`/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira`** |
| delivery path | `docs/coordination/tasks/mira/MIRA-2026-09-30-live-mirror-um0-delivery-v1.md` |
| tags | mira, ui, um0, live-mirror, mock-labelling, p0-parallel |

---

## 0. Where to work — your own worktree, already prepped

**A `pnpm tauri:dev` server is running in the shared checkout right now** (pid 35186). Building
there would fight the watcher and dirty the tree. I created and prepped a worktree for you:

```
/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira
  branch packet/mira/MIRA-2026-09-30-live-mirror-um0-v1 @ 324e185, clean
  ui/node_modules installed
```

`cd` there and work. Do **not** build in the shared checkout, do not switch its branch, and do
not touch `…/nimbus` or `…/flux` — those are other seats' worktrees.

**If this packet file seems to vanish** after a checkout, recover it with:
`git show track/infra-foundation:docs/coordination/tasks/mira/MIRA-2026-09-30-live-mirror-um0-v1.md`

## 1. Scope — two things, `ui/src` only

### 1.1 Make the live mirror real

The backend already does all of this. **No Rust change is needed and none is permitted.** The
wrappers exist in `ui/src/lib/api.ts` — verified at `324e185`:

| Wrapper | line | Tauri command |
|---|---|---|
| `listTmuxSessions()` | `api.ts:144-145` | `cmd_list_tmux_sessions` — discovers live seat sessions |
| `attachTmuxSession(request)` | `api.ts:146+` | `cmd_attach_tmux_session` |
| `listLiveSessions()` | `api.ts:141` | `cmd_list_live_sessions` |
| `ptyWrite(sessionId, data)` | `api.ts:133-134` | `cmd_pty_write` |
| `ptyWriteBytes(sessionId, bytes)` | `api.ts:135-136` | `cmd_pty_write_bytes` |

Wire the main/live view to list the **real** seats from `listTmuxSessions()` and render their
live panes (attach + stream). The five discoverable seats are
`Flux-Quality&Ops-seatloom`, `Lyra-po-seatloom`, `Mira-UX/UED-seatloom`,
`Nimbus-TechArchi-seatloom`, `Onyx-data-seatloom` — the `-seatloom` suffix is load-bearing;
`crates/seatloom-core/src/pty/mod.rs` filters on it.

**The bar:** typing in a seat's real tmux must be visible in the UI pane. That is what "real"
means here — not that a list renders, but that the mirror moves.

### 1.2 Make the mock unmistakable

Seed/projection rows (workitems, sessions, documents, events) are **fixtures**. They have no live
backing, and — I checked — **there is no `source` column to key off**, so you cannot distinguish
them by querying. You will have to mark them at the view level, by knowing which views are
seed-backed.

Give any seed-backed view an unmistakable **"SAMPLE DATA — not yet from live capture"**
badge/banner, or hide it. The requirement is that a person opening the app can tell real from
mock **at a glance**, without asking anyone.

**The exact UX is yours — you own UX.** The V1 AppShell is the reference; do not replace it.

## 2. Hard constraints

- **No backend, no Rust, ever in this packet.** Zero files under `crates/`, `src-tauri/`,
  `infra/`. Every command you need exists. If you conclude a backend change is required, **STOP
  and tell me** — do not edit Rust. Nimbus is live in Rust right now on the step-2 write API and
  must not be disturbed.
- **Do not touch `ui/src/lib/types-dto.ts` or `ui/src/lib/dto-to-v1.ts`.** This is the one place
  our two parallel packets could collide: Nimbus's packet permits minimal `ui/src/**/*.ts` type
  fixes at the DTO boundary, and those are the two files it would touch. Your task needs the tmux
  APIs, not the DTO projection types. If you believe you need them, stop and tell me — I will
  sequence it rather than let you both edit the same file.
- **No `.sql`.** Migration `010` is a closed, accepted gate.
- **Do not write to the live `seatloom` database.**

## 3. Verification (you run, Flux gates)

In your worktree:

```
pnpm --dir ui typecheck
pnpm --dir ui exec eslint .
pnpm --dir ui exec prettier --check .
pnpm --dir ui build
git show --stat <sha>        # must show zero crates/, src-tauri/, infra/ files
```

**Plus the thing that actually matters, which no gate command can prove:** demonstrate the mirror
is live. Attach to a real seat, have something change in that tmux, and capture evidence that the
UI pane reflected it. A screenshot, a captured pane diff, a recorded sequence — your call, but it
must show movement, not a static list. A green typecheck proves nothing about whether the mirror
works.

State honestly in the delivery which of the five seats you actually exercised.

## 4. Done-definition

1. The main/live view lists the **real** seats from `listTmuxSessions()` — not fixtures.
2. Their live panes render, attached and streaming.
3. **Typing in a real seat's tmux is reflected in the UI pane**, with evidence (§3).
4. Every seed-backed view is unmistakably labelled **"SAMPLE DATA — not yet from live capture"**
   or hidden; real vs mock is obvious at a glance.
5. V1 AppShell preserved, not replaced.
6. Zero files under `crates/`, `src-tauri/`, `infra/`; zero `.sql`; `types-dto.ts` and
   `dto-to-v1.ts` untouched.
7. `typecheck`, `eslint`, `prettier --check`, `build` all green, output verbatim.
8. Built in your worktree; shared checkout never branch-switched.
9. Delivery doc written; Flux commit-pinned verify PASSes.

## 5. Evidence expectations

Base/delivery branch and commit SHAs; verbatim command output, not summaries; the §3 liveness
evidence; which seats you exercised; and an explicit statement of anything not done and why.
Evidence-free completion claims are rejected.

If something in this packet is wrong — a line number that has moved, a wrapper that does not
behave as described — **report it rather than working around it.** Three of the last five gates
turned up a stale fact in a packet, including two of mine.

---

*Dispatched by Lyra · 2026-09-30 · UM0 · pure frontend, own worktree, parallel-safe with Nimbus
step 2 · §2 fences the two DTO files where the two packets could collide*
