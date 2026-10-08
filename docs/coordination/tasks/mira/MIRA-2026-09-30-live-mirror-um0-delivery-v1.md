# Delivery: UM0 — make the live tmux mirror real, and label the mock

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | MIRA-2026-09-30-live-mirror-um0-delivery-v1 |
| packet | `docs/coordination/tasks/mira/MIRA-2026-09-30-live-mirror-um0-v1.md` |
| status | delivered |
| author | mira |
| date | 2026-09-30 |
| to | lyra |
| base branch / commit | `track/infra-foundation` @ `324e185` |
| delivery branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| **delivery commit** | **`cd5788fc6debfd48fbe3d3961217bbd03e40e289`** |
| build location | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira` |
| tags | mira, ui, um0, live-mirror, mock-labelling, p0-parallel |

---

## 0. Read this first — three findings, one of them blocking for UM0

Per §2 and the packet's closing instruction, I am reporting rather than working around.

### 0.1 BLOCKING (backend): `tmux pipe-pane -o` is a toggle, so the second attach to a pane kills the mirror

The packet states: *"The backend already does all of it — NO Rust change is needed and none is
permitted."* **That is not true for re-attach.** I did not edit any Rust; I am reporting it.

`crates/seatloom-core/src/pty/mod.rs` (`PtySession::attach_tmux`) runs:

```rust
.args(["pipe-pane", "-t", &tmux_target, "-o", &cat_cmd])
```

with the comment:

```rust
// `-o` makes the pipe-pane command invoke only once (so we don't duplicate if
// attach is called twice)
```

The comment misreads the flag. In tmux, `-o` means *"only open a new pipe if no previous pipe
exists"* — i.e. it is the **toggle** flag. If a pipe already exists on that pane, `pipe-pane -o`
**closes** it. So:

- 1st attach to a pane → pipe opens → mirror works.
- 2nd attach to the same pane → pipe **closes** → mirror silently dies, `pane_pipe=0`, zero bytes,
  no error surfaced anywhere. `cmd_attach_tmux_session` still returns a successful `LiveSessionDto`
  with a valid initial snapshot, so the UI shows a populated, permanently frozen terminal.

Measured directly on a real pane, nothing to do with my UI:

```
state before:        pane_pipe=0
after 1st -o call:   pane_pipe=1
  bytes captured:        2835
after 2nd -o call:   pane_pipe=0
  bytes after 2nd:       2835      <- frozen
```

Why this matters for UM0: a frozen-but-populated pane is exactly the failure mode UM0 exists to
kill. It looks live and is not. Any page reload, any second attach, any seat already mirrored by
another running SeatLoom instance lands in it.

**Recommended fix (Nimbus or whoever owns pty, not me):** drop `-o` — plain `pipe-pane` replaces
an existing pipe unconditionally, which is the intended "re-attach" semantics. If explicit
idempotency is wanted, read `#{pane_pipe}` first and only then decide. I have deliberately left
this unfixed: §2 forbids Rust here and Nimbus is live in that crate.

I worked within the bug to produce the liveness evidence below (attach only to panes whose
`pane_pipe` was 0). The UI change is correct and proven; the re-attach path stays broken until the
Rust fix lands. **I do not consider UM0 fully closed until it does**, because "typing in a real
seat shows up in the UI" holds only for a pane's first attach.

### 0.2 The packet's gate expectation does not hold at this base

§3 asks for `typecheck`, `eslint`, `prettier --check`, `build` "all green, verbatim output".
Measured at base `324e185` on a clean tree, **before I changed anything**:

| gate | exit | result |
|---|---|---|
| `pnpm --dir ui typecheck` | 0 | green |
| `pnpm --dir ui exec eslint .` | **1** | **151 problems (143 errors, 8 warnings)** |
| `pnpm --dir ui exec prettier --check .` | **1** | **96 files** |
| `pnpm --dir ui build` | 0 | green |

eslint and prettier are red at base and cannot be made green by this packet: the fix is
reformatting/refactoring 96+ files across the whole UI, which is far outside UM0 and would collide
head-on with Nimbus — a repo-wide `prettier --write` rewrites files in the DTO boundary area §2
fences off.

So I held to **zero regression** instead, and measured it. See §3.

### 0.3 `prettier --check .` is order-dependent — Flux will get a different number than the packet implies

There is no `.prettierignore`, so `prettier --check .` also checks `ui/dist/`. Running `build`
before `prettier --check` adds 7 build artifacts to the count (96 → 103). The packet lists
`prettier --check` before `build`, which happens to avoid it. Flux should either run them in that
order or `rm -rf ui/dist` first, otherwise the counts will not reconcile. A `.prettierignore` with
`dist/` would close this permanently (not done here — out of scope, and it would touch a shared
config while a parallel packet is in flight).

---

## 1. What shipped

Frontend only, `ui/src` only. 7 files, +921/−9.

```
 ui/src/App.tsx                         |  29 +-
 ui/src/components/NavRail.tsx          |  15 +-
 ui/src/components/ProvenanceBanner.tsx | 148 +++++++++
 ui/src/layouts/StatusBar.tsx           |  12 +-
 ui/src/lib/provenance.ts               |  86 +++++
 ui/src/lib/tmux-seats.ts               |  54 +++
 ui/src/views/LiveSeatsView.tsx         | 586 +++++++++++++++++++++++++++++++++
```

### 1.1 The live mirror (`views/LiveSeatsView.tsx`, `lib/tmux-seats.ts`)

- Discovers the **real** seats from `api.listTmuxSessions()` (3s poll), parses
  `<Seat>-<Role>-seatloom` into seat + role, and renders them in a rail. No fixture path: with no
  tmux sessions it says so instead of inventing rows, and in browser-dev (`isTauri()` false) it
  states there is no backend rather than showing mock seats.
- Attaches via `api.attachTmuxSession()` and streams through the existing `SessionTerminal`
  (unchanged), passing the authoritative `paneRows`/`paneCols`/`initialSnapshotB64`.
- **The design problem UM0 actually poses is that a list of real seats looks exactly like a list of
  fake ones.** So the proof of life is not the list, it is movement: each card carries a byte
  counter, a chunk counter, and a "last output" clock fed from the real `session:output` stream,
  and the dot pulses while bytes are arriving. A screenshot of this view is self-evidencing —
  fixtures cannot make a counter climb. Counters are accumulated in a ref and flushed on one 500ms
  tick so a chatty pane cannot trigger a render per chunk.
- Made the **first nav item and the default tab**, so the app opens on the surface that proves
  itself. The first discovered seat auto-attaches once, so the screen is already moving on open.
- Attachments register in `useLiveSessionsStore`, which the Supervisor IM already reads — so IM
  routing now has a real pane to target. (The store's own header comment said
  `SessionsWorkspace` would do this; nothing did.)
- Detach stops our mirror only. R1/R3 preserved: SeatLoom observes, never owns.

### 1.2 Sample data made unmistakable (`lib/provenance.ts`, `components/ProvenanceBanner.tsx`)

The honest framing, which the packet's own wording understates: **every V1 view already reads from
PostgreSQL**, so "it came from the database" is not "it is live". The seed corpus is a fixture set
with no live backing and, as §1.2 says, no `source` column to key off. Provenance is therefore a
property of the *view*, and `lib/provenance.ts` is the one list to keep honest.

Three states, deliberately distinguished — `seed` and `fixture` are both "not real", but the
operator's next action differs:

| state | meaning | badge |
|---|---|---|
| `live` | mirrored from a real tmux pane now | green `LIVE` |
| `seed` | real Postgres read, but the rows are seed fixtures | amber `SAMPLE` |
| `fixture` | backend unreachable, in-memory mock | amber `MOCK` |

Both non-live states carry the contract string **"SAMPLE DATA — not yet from live capture"**
verbatim, plus a line saying which case it is and what would change it, plus a **查看实时席位**
button that jumps to the live mirror.

Applied in **one place**: `App.renderMainView()` is wrapped in a single `ProvenanceFrame`. No
individual V1 view was edited to gain a banner — which is why the diff is 7 files and not 15, and
why no view can silently be added later without inheriting a marker.

`NavRail` carries the same dot per tab, because "at a glance" has to include the glance you take
while deciding where to click — provenance is legible *before* a tab is opened, expanded or
collapsed.

V1 AppShell, NavRail, ProjectSwitcher and master-detail are preserved, not replaced.

### 1.3 The one thing in the shell that read as live

`StatusBar` had a hard-coded green `对账完成` ("reconcile complete") with an active-status icon. It
is a placeholder with no reconcile behind it, and it is very likely the "sync timestamp" Mr. Zhang
read as real. It now renders `对账状态 · 示例` in warning colour with a tooltip saying it is a
placeholder and represents no actual reconcile.

---

## 2. Liveness evidence (§3 — the part no gate command can prove)

**Screen capture and assistive access are both denied to this session**, so a screenshot was not
available. I produced strictly stronger evidence instead: I read the **rendered xterm buffer out of
the running webview** and cross-checked it against the real tmux pane. A screenshot shows pixels; a
buffer dump shows the exact text the UI pane is displaying, timestamped, and can be grepped.

Method (temporary instrumentation, fully reverted — see §2.3):

1. Ran the real app in my worktree: `pnpm tauri dev` against vite on **port 1421** with
   `devUrl` overridden, so the shared checkout's dev server on 1420 was never touched.
   Backed by the real Postgres (`DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom`);
   the app did not panic, so the DB was live.
2. A temporary probe clicked **my own view's** `接入实时镜像` button (real user code path, not a
   direct backend call) and POSTed each terminal's `buffer.active` text to a local collector.
3. Printed a unique marker into the **real** `Mira-UX/UED-seatloom:0` pane.

### 2.1 The seats it listed were real

Scraped from the rendered rail, all five, from `cmd_list_tmux_sessions`:

```
['Flux-Quality&Ops-seatloom', 'Lyra-po-seatloom', 'Mira-UX/UED-seatloom',
 'Nimbus-TechArchi-seatloom', 'Onyx-data-seatloom']
```

Terminals reported `50 rows x 200 cols` — the real tmux pane geometry from
`display-message`, not an xterm default.

### 2.2 Typing in a real seat's tmux appeared in the UI pane

Marker `UM0LIVEPROOF231451XKQ9` printed into `Mira-UX/UED-seatloom:0`. Present twice in the real
pane per `tmux capture-pane`. Then found **in the UI's rendered buffer**:

```
==============================================================================
UM0 LIVENESS EVIDENCE — rendered xterm buffer inside the running SeatLoom UI
==============================================================================
Marker printed into real tmux pane 'Mira-UX/UED-seatloom:0': UM0LIVEPROOF231451XKQ9

UI session id : ses-18da22105efdd0f8
xterm geometry: 50 rows x 200 cols (real tmux pane dims)
sampled at    : 23:14:55  buffer hash=d0219381
marker found on rendered buffer line(s): [54, 55]
------------------------------------------------------------------------------
     50 | ⏺ Mira's pane is now piped. The definitive test — unique marker into my own real pane, then read the
     51 |
     52 | ⏺ Bash(MARKER="UM0LIVEPROOF$(date +%H%M%S)XKQ9"; echo "$MARKER" > /tmp/mira-evidence/marker2.txt; ec
     53 |   ⎿  === printing marker into the REAL Mira tmux pane ===
>>   54 |      UM0LIVEPROOF231451XKQ9
>>   55 |      UM0LIVEPROOF231451XKQ9
     56 |      (4s)
     57 |      (ctrl+b ctrl+b (twice) to run in background)
------------------------------------------------------------------------------
```

That buffer is my own Claude Code session rendering inside the SeatLoom UI — unmistakably the real
Mira pane, not a fixture.

**Movement, not a static list.** Content hashes of the same session across the sample window —
21 distinct states:

```
ses-18da22105efdd0f8: distinct content hashes: 21
  [('23:14:17','9334fb8f'), ('23:14:18','be52ae2b'), ('23:14:20','4c6afaea'),
   ('23:14:21','ba7dfcd2'), ('23:14:23','b2d93ecc'), ('23:14:25','99d2e3b1'),
   ('23:14:27','50ace03a'), ('23:14:29','c19ce79'), ...]
```

An earlier run independently showed the same for a different seat — `Flux-Quality&Ops-seatloom`
churned through 9 distinct buffer states while Flux worked, with its transcript growing to 37,906
bytes, with no input from me at all.

### 2.3 Which seats I actually exercised — stated honestly

| seat | attached through my UI | live bytes observed in the UI pane |
|---|---|---|
| `Mira-UX/UED-seatloom` | yes | **yes — marker + 21 buffer states (primary evidence)** |
| `Flux-Quality&Ops-seatloom` | yes (auto-attach) | yes — 9 buffer states, 37,906 B transcript |
| `Lyra-po-seatloom` | yes (mis-targeted probe click, see below) | no — pane idle, 0 bytes; nothing to mirror |
| `Nimbus-TechArchi-seatloom` | **no** | no — deliberately left alone, Nimbus is live in Rust |
| `Onyx-data-seatloom` | **no** | no — not exercised |

I attached to **three of five**. I deliberately did not touch Nimbus (§2 of the packet) and did not
get to Onyx. I generated pane movement **only in my own seat's pane**, by running my own commands —
I did not `send-keys` into any other seat, because every one of the five is a live `node` agent
session and injecting keystrokes would corrupt another seat's conversation. Flux's movement was
Flux's own work, observed passively.

Two honest notes on my own method:
- My first probe used a top-down `querySelectorAll('div')` match, which matched the whole rail
  (it names every seat) and clicked the wrong card — it attached Lyra, not Mira. **That was a bug
  in my throwaway probe, not in the shipped view.** I found it by checking `pane_pipe` per seat,
  fixed the probe to walk *up* from each button and accept only an ancestor naming exactly one
  session, and re-ran. The shipped UI binds each button to its own seat object directly and was
  never affected.
- The instrumentation touched `SessionTerminal.tsx` and `main.tsx` and added
  `ui/src/__um0_probe.ts`. All three are reverted: the probe file is deleted, the two tracked files
  are byte-identical to base (`git diff 324e185 -- ui/src/app-v2/panel/SessionTerminal.tsx
  ui/src/main.tsx` is empty), and `grep -rn "__um0_probe\|__slTerms\|1425" ui/src/` returns
  nothing. None of it is in `cd5788f`.

### 2.4 Side effects I caused, disclosed

Attaching runs `tmux pipe-pane`, and because of §0.1 my attaches toggled pipes on panes that the
**shared checkout's running app** may have been mirroring. I reset `pane_pipe` to 0 on all five
seat panes when I finished, so tmux is in a clean state, but if Mr. Zhang's open window had live
terminals they will have stopped updating and need re-attaching in that app. No tmux session was
killed; no seat's process was touched. The shared checkout was never branch-switched and its dev
server (pid 35217) is still running — it has since moved to `b705ba1` under other seats' commits,
not mine.

---

## 3. Gate output, verbatim, with base-vs-delivery deltas

Run in `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira`, `ui/dist` removed first
so the prettier count is comparable to base (§0.3).

```
$ pnpm --dir ui typecheck
EXIT=0

> seatloom-ui@0.1.0 typecheck /Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/mira/ui
> tsc --noEmit


$ pnpm --dir ui exec eslint .
EXIT=1
✖ 151 problems (143 errors, 8 warnings)

$ pnpm --dir ui exec prettier --check .
EXIT=1
[warn] Code style issues found in 96 files. Run Prettier with --write to fix.

$ pnpm --dir ui build
EXIT=0
dist/assets/supervisor-C7HeaqHx.js          0.86 kB │ gzip:   0.56 kB
dist/assets/SupervisorPanel-_vfc2PLc.js   353.05 kB │ gzip: 103.02 kB
dist/assets/main-DHq9L_nv.js              532.25 kB │ gzip: 130.94 kB
✓ built in 2.38s
```

### Delta vs base `324e185` — zero regression

| gate | base | delivery | delta |
|---|---|---|---|
| typecheck | exit 0 | exit 0 | — |
| eslint | exit 1, 151 problems / 143 errors | exit 1, **151 / 143** | **0** |
| prettier | exit 1, 96 files | exit 1, **96 files** | **0** |
| build | exit 0 | exit 0 | — |

The prettier offender **set** is byte-identical, not merely the count:

```
$ diff <(base non-dist warns | sort) <(delivery non-dist warns | sort)
IDENTICAL — zero prettier regression in source
```

My four new files are clean on both gates. The 15 eslint errors reported in `App.tsx` are the same
15 `no-explicit-any` errors present at base in the same pre-existing handlers, shifted +5 lines by
my imports. `NavRail.tsx`, `App.tsx` and `StatusBar.tsx` were already prettier-dirty at base and
remain so; I did not reformat them, because doing so would bury a 7-file change in whole-file
churn and make the diff unreviewable.

### §14 forbidden-path gate

```
$ git show --name-only --format='' cd5788f | grep -E "^(crates|src-tauri|infra)/|\.sql$|types-dto|dto-to-v1"
PASS: zero crates//src-tauri//infra/ files, no .sql, DTO files untouched
```

---

## 4. Done-definition

| # | requirement | status |
|---|---|---|
| 1 | main/live view lists **real** seats from `listTmuxSessions()` | **done** — all five, §2.1 |
| 2 | live panes render, attached and streaming | **done** — §2.2 |
| 3 | typing in a real seat's tmux reflected in the UI pane, with evidence | **done for a pane's first attach** — §2.2. **Not true on re-attach**, blocked by §0.1 backend bug |
| 4 | every seed-backed view unmistakably labelled or hidden | **done** — §1.2, one wrapping point + NavRail markers |
| 5 | V1 AppShell preserved | **done** — added a nav item and a wrapper; shell untouched |
| 6 | zero `crates/`/`src-tauri/`/`infra/`, zero `.sql`, DTO files untouched | **done** — §3 |
| 7 | typecheck/eslint/prettier/build green, verbatim | **cannot hold at this base** — §0.2. Delivered zero-regression instead, measured in §3 |
| 8 | built in my worktree, shared checkout never branch-switched | **done** — §2.4 |
| 9 | delivery doc written; Flux commit-pinned verify PASSes | this doc; Flux to gate `cd5788f` |

## 5. Not done, and why

- **The `pipe-pane -o` fix.** Requires Rust; §2 forbids it and Nimbus is live in that crate. §0.1
  has the diagnosis and the one-line fix. **This is the one thing standing between UM0 and "the
  mirror is reliably live", and I recommend it be dispatched next.**
- **Repo-wide eslint/prettier cleanup.** Out of scope and would collide with Nimbus's packet.
- **`.prettierignore`.** Would fix §0.3 permanently but touches shared config mid-flight.
- **`Onyx-data-seatloom` and `Nimbus-TechArchi-seatloom` not exercised.** Nimbus deliberately;
  Onyx simply not reached. Both are discovered and listed; neither was attached.
- **No screenshot.** Screen capture and assistive access are denied to this session. §2 substitutes
  rendered-buffer dumps, which I consider stronger, but if a literal screenshot is required for
  sign-off it needs a seat with those permissions.
- **`SessionsWorkspace` (会话 tab) left as-is.** It is the older attach-only workspace and is
  marked `live`; I did not merge it into the new view, to keep the diff reviewable. Worth
  consolidating later.

---

*Delivered by Mira · 2026-09-30 · UM0 · frontend only · `cd5788f` on
`packet/mira/MIRA-2026-09-30-live-mirror-um0-v1`*
