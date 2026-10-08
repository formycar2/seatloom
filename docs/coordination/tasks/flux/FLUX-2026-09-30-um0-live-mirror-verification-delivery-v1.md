# Delivery: Commit-pinned verification — UM0 live tmux mirror + sample-data marking

[Flux -> Lyra] Verdict: **HOLD** — on a CI gate nobody has measured, not on Mira's work

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-09-30-um0-live-mirror-verification-delivery-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-01 |
| to | lyra |
| verifies | `docs/coordination/tasks/flux/FLUX-2026-09-30-um0-live-mirror-verification-v1.md` |
| delivery under test | `MIRA-2026-09-30-live-mirror-um0-delivery-v1.md` (on Mira's branch) |
| target branch | `packet/mira/MIRA-2026-09-30-live-mirror-um0-v1` |
| **verified commit** | **`0701bf9ded96638b378742bb627c4a93d119e101`** (detached HEAD) |
| code commit | `cd5788f` |
| base | `324e185` |
| verdict | **HOLD** |
| tags | flux, verification, commit-pinned, um0, ui, live-mirror, hold |

---

## 0. Verdict, and what it is and is not about

**HOLD.** Two reasons, neither of which says Mira's work is wrong:

1. **The gate CI actually runs fails.** `scripts/lint-changed.sh` — which `frontend:lint` executes
   — **exits 1** on this commit. Her branch was never pushed, so CI has never run, and nobody has
   measured this: Mira correctly measured `eslint .` / `prettier --check .` repo-wide and proved
   **zero regression** (I reproduced her numbers exactly), but the repo-wide command is not the
   project's standard. The ratchet is, and the ratchet is file-granular. Detail in §2.3. This is
   a merge blocker as things stand and it needs your ruling, not a fix from me.
2. **I could not observe the UI rendering.** I proved a great deal about liveness — more than a
   static list, see §3 — but I never saw the byte counters climb on screen, and §3 is explicitly
   about what a command cannot prove. Claiming PASS on the strength of everything *around* the
   pixels would be exactly the "PASS with a caveat" the rule forbids.

**Everything else passes**, including the hard parts: scope, the fence, zero regression, the
AppShell, the provenance architecture, and — importantly — **the mirror demonstrably moves**.

I also have one finding of my own that extends Mira's §0.1 and belongs to Nimbus's in-flight fix:
**the app leaks pipe state across exit** (§3.4).

---

## 1. Identity, scope, and the fence

```
$ git -C …/flux rev-parse HEAD
0701bf9ded96638b378742bb627c4a93d119e101     <- MATCHES target
$ git -C …/flux rev-parse --abbrev-ref HEAD
HEAD                                          <- detached, as prepared
$ git -C …/flux status --short
(empty — clean, before and after)
```

```
$ git diff --name-only 324e185..0701bf9 | grep -vE '^(ui/|docs/)'      -> (empty)
$ git diff --name-only 324e185..0701bf9 | grep -E '^(crates|src-tauri|infra)/|\.sql$'  -> (empty)
$ git diff --name-only 324e185..0701bf9 | grep -E 'types-dto\.ts|dto-to-v1\.ts'        -> (empty)
```

Zero outside `ui/` and `docs/`; zero `crates/`, `src-tauri/`, `infra/`; zero `.sql`. **The fence
held** — `types-dto.ts` and `dto-to-v1.ts` are untouched, so the two parallel packets did not
collide. 8 files, +1308/−9.

**V1 AppShell preserved, not replaced.** `ui/src/main.tsx` still renders `App` from
`./App` (the V1 shell) inside `TopErrorBoundary`; Mira added a tab to the existing
`<AppShell>` rather than introducing a new root. Confirmed by reading the entry point, not
inferred from the diff.

## 2. The four gates

Run in the packet's order, **`prettier --check` before `build`**, with `ui/dist` confirmed absent
first (`ls: ui/dist: No such file or directory` — your removal held).

| gate | exit | result |
|---|---|---|
| `pnpm --dir ui typecheck` | **0** | green |
| `pnpm --dir ui exec eslint .` | 1 | **151 problems (143 errors, 8 warnings)** |
| `pnpm --dir ui exec prettier --check .` | 1 | **96 files** |
| `pnpm --dir ui build` | **0** | green (`✓ built in 2.31s`) |

### 2.1 Ordering and the count, for reconciliation

`prettier --check .` reported **96 files**, run **before** any build, with `ui/dist` absent. That
is exactly Mira's base measurement, so our numbers reconcile with no argument needed. Had I built
first it would have been 103 — I did not.

### 2.2 Zero regression — confirmed independently

Mira measured at base `324e185`: eslint 151 problems (143 errors, 8 warnings), prettier 96 files.
I measured at her head `0701bf9`: **151 problems (143 errors, 8 warnings), 96 files** — identical.
She introduced no new lint or format debt.

I checked attribution rather than trusting the totals:

- Her **four new files** — `ProvenanceBanner.tsx`, `provenance.ts`, `tmux-seats.ts`,
  `LiveSeatsView.tsx` — have **zero** eslint problems and **zero** prettier complaints. The new
  code is clean.
- The only changed file with eslint problems is `App.tsx` (15, all `no-explicit-any`). Those sit
  at lines 198–299; **every one of Mira's hunks is elsewhere** (11, 35-36, 39, 44-45, 56-57, 135,
  274-276, 332-343). And `App.tsx` contains **15** `any` occurrences at base and **15** at head —
  unchanged.

### 2.3 The finding that drives the HOLD — CI's real gate fails

`.gitlab-ci.yml` `frontend:lint` does **not** run `eslint .`. It runs:

```yaml
script:
  - (cd ui && pnpm typecheck)      # FULL
  - bash scripts/lint-changed.sh   # RATCHET — eslint + prettier on changed ui/src
```

I ran the ratchet:

```
$ bash scripts/lint-changed.sh
ui/src/App.tsx   15 problems (15 errors, 0 warnings)     [@typescript-eslint/no-explicit-any]
Checking formatting...
[warn] src/App.tsx
[warn] src/components/NavRail.tsx
[warn] src/layouts/StatusBar.tsx
[warn] Code style issues found in 3 files.
lint-changed: violations above.
exit code: 1
```

The ratchet's own header states the policy: *"The existing tree has 143 eslint errors / 87
unformatted files that predate the standard; gating the whole tree would block every MR. **New or
changed files must be clean.**"*

So the mechanism is: the ratchet is **file-granular**, and all three files Mira edited were
**already dirty at base** (I verified each: `App.tsx`, `NavRail.tsx`, `StatusBar.tsx` all fail
`prettier --check` at `324e185` as well as at `0701bf9`). By touching them she inherits their
pre-existing debt into the gate. She created none of it.

Consequences, plainly:

- `git ls-remote` shows her branch is **not on GitLab**, and the API returns **no pipelines** for
  it, so CI has never run. My local run is the only CI-equivalent evidence, and it is red.
- As it stands, **this branch cannot go green in `frontend:lint`**.
- The cost is asymmetric: the 3 prettier files are a one-command fix, but cleaning 15 `any`s out
  of `App.tsx` is a refactor well outside UM0 — and a repo-wide `prettier --write` is precisely
  what Mira avoided because it would rewrite files in the DTO boundary the fence protects.

**This is your call, not mine, and it is a policy question rather than a defect:** either the
standard for this packet is zero-regression (Mira met it, and the ratchet base should be adjusted
or the three files granted an exception), or the standard is the ratchet (and the three files must
be cleaned before merge). I have not touched anything.

### 2.4 Finding: there is no `.prettierignore`

Confirmed — none in the repo root or in `ui/`. That is what makes `prettier --check .` walk
`ui/dist` and makes the count depend on whether `build` ran (96 → 103). A one-line
`.prettierignore` containing `dist/` closes it permanently. Real defect, **not Mira's to fix
here** — it touches shared config while a parallel packet is in flight. Noting it as instructed.

---

## 3. Does the mirror actually move?

Yes — I established this, and with stronger evidence than a screenshot would have given, though
not the specific evidence you asked for. What I could not do is watch the rendered counters; see
§3.5 for why, stated plainly rather than papered over.

**How I ran it, without disturbing anything.** The app was already running from the *shared*
checkout (`tauri dev`, 1h32m) — but that is `track` code, which has no `LiveSeatsView`. Port 1420
is held by its vite under `strictPort: true`, so a second `tauri dev` could not start normally.
Rather than kill the running app, I launched a **second, additive instance** against Mira's code:

```
vite (Mira's UI)      -> 127.0.0.1:1421
tauri dev --config '{"build":{"devUrl":"http://127.0.0.1:1421","beforeDevCommand":""}}'
```

The original app was never stopped and is still running (pid 52131, 49m at close). Confirmed the
webview really loaded Mira's UI, not a cached bundle:

```
$ lsof -nP -iTCP:1421
node       12948  …  127.0.0.1:1421->127.0.0.1:61511 (ESTABLISHED)
com.apple  20600  …  127.0.0.1:61511->127.0.0.1:1421 (ESTABLISHED)   <- the WebKit process
```

### 3.1 Her UI drove real attaches on real seats

Baseline, recorded before I touched anything (§3.6): **all five seats `pane_pipe=0`**, re-confirmed
still all 0 at 00:01. Then, with Mira's UI running, a 1-second watcher across an app restart caught
the transition:

```
t=1   Flux=0  Lyra=1  Mira=0  Nimbus=0  Onyx=0     <- Lyra left piped by my first instance
t≥N   Flux=1  Lyra=1  Mira=0  Nimbus=0  Onyx=0     <- Flux flips 0 -> 1 as her view auto-attaches
      (51 samples with Flux=1, 22 with Flux=0)
```

**Nothing else in the system calls `attachTmuxSession`** — the running app is `track` code with no
such view, and the seats were untouched at 0 for the twenty minutes before. The 0 → 1 transitions
are caused by `LiveSeatsView`'s auto-attach. That is the UI reaching through IPC into real tmux.

### 3.2 Real output really flows through the mirror

The attach opens `pipe-pane` → FIFO → transcript. Those files appeared in my worktree during the
run with real volume (123305 and 49071 bytes), and the live one grew continuously while I watched:

```
size BEFORE : 17636 bytes
size AFTER  : 24690 bytes   (3 seconds later)   delta 7054 bytes
… later samples: 59052 -> 60081 -> 66512 bytes
```

### 3.3 And it is provably *that* pane — the strongest evidence I have

A growing byte count could in principle be anything. So I read the mirrored stream back, and it
contains **my own output from this very verification session**, verbatim, with ANSI cursor
positioning intact:

```
Real signal: panes transitioned 0 → 1. Flux-Quality&Ops-seatloom flipped during the relaunch
window, and Lyra-po-seatloom was already 1 from my first instance. Nothing else in the system
calls attachTmuxSession …
7054 bytes in 3 seconds of genuinely live pane output. My marker didn't appear because …
```

That is text I wrote into this conversation minutes earlier, captured out of the real
`Flux-Quality&Ops-seatloom` pane by the mirror Mira's UI attached. **The mirror is live, it is
carrying the right pane, and it moves.** A fixture cannot produce that.

(One honest note on method: I first tried planting a marker with `echo`. It did not appear, because
Bash-tool output does not render into the pane's TTY — a property of how I execute, not a mirror
failure. The recognisable-content check above is the correct test and it succeeded.)

### 3.4 My own finding — the app leaks pipe state across exit (for Nimbus)

Extends Mira's §0.1 rather than duplicating it. After I killed my first app instance,
`Lyra-po-seatloom` **stayed at `pane_pipe=1`** with no live reader — the app does not close the
pipe on exit. Combined with the `pipe-pane -o` toggle Mira reported, the consequence is worse than
either alone:

> After any unclean app exit, the *next* run's **first** attach to that pane is effectively a
> second `-o` call, so it silently closes the pipe and lands straight in the frozen-but-populated
> state — without anyone having attached twice in the same session.

So "only a pane's first attach works" is optimistic; it is "only the first attach since the pipe
was last cleared". Worth folding into `NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1`: closing the
pipe on session drop is as important as dropping `-o`.

### 3.5 What I did not establish, and why

I did not see the byte/chunk counters climb in the rendered view. Three concrete blockers:

1. The `browser-automation` skill cannot run on this host — `patchright` is not resolvable, so
   there is no headless browser and no DOM/accessibility-tree read.
2. In a plain browser Mira's view would be useless anyway: `isTauri()` is false, so the tmux IPC
   is unavailable by design.
3. That leaves screenshotting a native window on the user's machine at 00:05. I judged
   GUI-automating and capturing Mr. Zhang's desktop to be beyond what this gate authorises, and I
   was not willing to capture his whole screen to get one counter.

So: the UI→backend half and the backend→stream half are both proven; the stream→pixels half is
read from code (`LiveSeatsView` accumulates `session:output` bytes into `pendingRef` and flushes
them on a single 500 ms tick into the per-card counters) but not observed. That is the gap, and it
is the second reason for HOLD.

### 3.6 Seats exercised, and the hazard handling you required

Seats exercised: **`Flux-Quality&Ops-seatloom`** (attached by the relaunched instance, and the pane
whose content I read back) and **`Lyra-po-seatloom`** (attached by the first instance). I attached
only to panes reading `pane_pipe=0`, as instructed.

| seat | baseline 23:47 | during run | **at close** |
|---|---|---|---|
| `Flux-Quality&Ops-seatloom` | 0 | 1 | **0** |
| `Lyra-po-seatloom` | 0 | 1 | **0** |
| `Mira-UX/UED-seatloom` | 0 | 0 | **0** |
| `Nimbus-TechArchi-seatloom` | 0 | 0 | **0** |
| `Onyx-data-seatloom` | 0 | 0 | **0** |

**All five restored to 0 and confirmed by re-reading each one.** `Lyra` required an explicit
`tmux pipe-pane -t` to close (the §3.4 leak); `Flux` had already dropped to 0 when my instance
exited. **No session was killed** — 10 sessions at baseline, 10 at close, same names.

**Possible disturbance to declare:** my `tauri dev` rebuilt into the shared `CARGO_TARGET_DIR` and
so replaced `target/debug/seatloom-tauri` on disk. The running app kept executing from the old
inode and is unaffected, but if the shared checkout's own watcher rebuilds it will recompile once.
Beyond that I believe I disturbed nothing: the running app was never stopped, and the two panes I
piped are both restored. If Mr. Zhang had a terminal attached in the shared app to Flux or Lyra, it
may need re-attaching.

---

## 4. Is mock unmistakable? — my judgement

**Yes, architecturally — and this is the strongest part of the packet.** My reservation is about
visual weight, not about whether the mechanism is right.

What convinced me is *where* the labelling lives. `App.tsx` wraps `renderMainView()` — every tab,
in exactly one place — in `<ProvenanceFrame provenance={provenanceForTab(activeTab,
backendHydrated)}>`. It is not per-view opt-in, so **a new view cannot be added and silently read
as live**; it inherits a label by construction. That is the difference between a fix and a habit,
and it is the right answer to Mr. Zhang's complaint.

The model is honest about a subtlety most would miss: `provenance.ts` notes that every V1 view
already reads from PostgreSQL, so *"it came from the database" is not the same as "it is live"* —
the seed corpus is a fixture set with no live backing and no per-row `source` column. Hence three
states, not two: `live` (mirrored from tmux now), `seed` (a real Postgres read of fixture rows),
`fixture` (backend unreachable, in-memory mock). Both non-live states carry the packet's contract
string **verbatim** — `SAMPLE DATA — not yet from live capture` — held in one constant so it
cannot drift.

Rendered, a non-live surface gets a full-width amber bar above the content: an 800-weight `SAMPLE`
/ `MOCK` badge, the headline at 12.5px/700, a sentence of explanation, an amber border plus a 3px
inset stripe, and a button that jumps to the live view. Live surfaces get the same bar in green
reading `LIVE — mirrored from tmux as it happens`. Colour, wording and position all agree, and the
app opens on the live tab by default.

**Could someone tell at a glance, without asking? Yes.** The amber band with `SAMPLE DATA — not yet
from live capture` sits directly above the content on every non-live surface; nobody could use
those views for a minute and believe they were watching the seats work.

Two honest qualifications:

- **My judgement here is from the code and computed styles, not from seeing it rendered** (§3.5).
  I can tell you the bar is full-width, amber, persistent and unavoidable; I cannot tell you it
  looks right, and 12.5px is not large. If you want the pixel judgement it needs a human or a host
  with a browser runtime.
- Provenance is per-**tab**, not per-row — correctly, and `provenance.ts` explains why. One edge
  follows from it: the live tab declares `LIVE` even when discovery fails, so a failed
  `cmd_list_tmux_sessions` shows a green LIVE bar above an error message. The view does surface its
  own error text, so it is not misleading in practice, but the banner is not conditioned on the
  attach actually succeeding. Worth a line in a follow-up, not a defect here.

---

## 5. Per-assertion table

| § | Assertion | Expected | Actual | Verdict |
|---|---|---|---|---|
| 1 | HEAD == target, detached | `0701bf9` | `0701bf9ded966…`, detached | PASS |
| 1 | worktree clean before/after | empty | empty both | PASS |
| 1 | zero files outside `ui/`+`docs/` | 0 | 0 | PASS |
| 1 | zero `crates/`, `src-tauri/`, `infra/`, `.sql` | 0 | 0 | PASS |
| 1 | **fence: `types-dto.ts` / `dto-to-v1.ts` untouched** | untouched | untouched | **PASS** |
| 1 | V1 AppShell preserved | preserved | `main.tsx` renders V1 `App`; tab added | PASS |
| 2 | `typecheck` | 0 | 0 | PASS |
| 2 | `build` | 0 | 0 | PASS |
| 2 | `prettier --check` count + ordering | reconcilable | **96**, before build, `ui/dist` absent | PASS |
| 2.2 | zero lint/format regression | zero | identical to base (151 / 96) | PASS |
| 2.2 | her new files clean | clean | 0 eslint, 0 prettier | PASS |
| **2.3** | **CI's real gate (`lint-changed.sh`)** | green | **exit 1** | **FAIL → HOLD-1** |
| 2.4 | `.prettierignore` exists | — | absent | finding, not Mira's |
| 3.1 | UI drives real attaches | moves | `pane_pipe` 0 → 1 on two seats | PASS |
| 3.2 | live bytes flow | flow | +7054 B/3 s; 59k→66k | PASS |
| 3.3 | it is the right pane, live | identifiable | this session's own output, verbatim | PASS |
| **3.5** | **rendered counters observed** | observed | **not observed** | **HOLD-2** |
| 3.6 | attach only to `pane_pipe=0` panes | honoured | honoured | PASS |
| 3.6 | **all five restored to 0** | all 0 | **all 0, re-read individually** | **PASS** |
| 3.6 | no session killed | none | 10 at baseline, 10 at close | PASS |
| 4 | mock unmistakably labelled | unmistakable | yes, architecturally enforced | PASS (see caveat) |

---

## 6. What would clear this HOLD

1. **Rule on §2.3.** Either the three touched files get formatted (trivial) and `App.tsx`'s 15
   pre-existing `any`s get an exception or a scoped cleanup, or the ratchet's expectations change.
   Whichever you choose, the branch should then be pushed so CI actually runs — it never has.
2. **Get a pixel confirmation of §3.5**, from a human in front of the app or a host with a browser
   runtime. Everything either side of the render is proven; this is a narrow gap.
3. Optional but cheap: **add `.prettierignore` with `dist/`** (§2.4) once the parallel packets land.

For the record, Mira's own position is that UM0 is not fully closed until Nimbus's `pipe-pane` fix
lands, and I agree with her reasoning — §3.4 makes the case slightly stronger than she put it.

I did **not** fix anything, did not touch Mira's or Nimbus's worktrees or branches, did not switch
the shared checkout, and created no stash.

---

*Flux · 2026-10-01 · HOLD · verified commit `0701bf9ded96638b378742bb627c4a93d119e101` ·
the mirror demonstrably moves — it carried this session's own pane content live · HOLD is on the
untested CI ratchet and on the one thing I could not put eyes on*
