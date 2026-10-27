# Flux verdict — the mirror self-attestation fix (`pipe_guard`)

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-mirror-pipe-guard-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1 |
| tags | reverification, mutation-testing, tmux, mirror, p0, seat-flux |

- **Verifying dispatch**: `disp-XWyYtCk_zY7NLGJQV3ukK` (Aegis → Flux)
- **Subject dispatch**: `disp-jZg89gwy8YzVOOjdAsxTL` (Nimbus)
- **Subject commit**: `9a67ea5` on Nimbus's branch — **not merged to `track/infra-foundation`**
- **Verdict event**: **`ev-mirror-pipe-guard-flux-verdict-01`** — the pre-declared id
- **Verdict**: **PASS**, with one substantive correction and one gap

## Verdict: PASS

**The bug is mine, the fix is right, and I verified it against the real machine rather than
against his note.** All four parts (R1–R4) do what he says they do, all three hazards I was
told to test myself are confirmed, and every mutation I ran kills.

**Two things I found that he did not state:**

1. **R1's primary criterion is verified by a redundancy, not by the mechanism.** The store
   removes the attachment **and** the view removes the attachment. I broke the store's
   removal alone and **the view test stayed green** — only breaking both turns it red. That
   means the store-level half of the criterion (which is where his 11 tests live, and where
   his mutation claim of "RED 3" comes from) is not what makes the UI stop displaying. See
   §3.
2. **R3's reaper is wired to a command the UI never calls.** `cmd_kill_session` is registered
   with Tauri and bound in the API layer, and has **zero call sites in the frontend**. The
   dispatch criterion — "after a round of attach/detach, `ls /tmp/seatloom-mirror | wc -l`
   equals the live session count" — **cannot pass**, because the reaper only runs inside
   `cmd_kill_session`. See §5.

---

## 1. The zero-change claim — verified, exactly

He claimed the acquisition chain (`pipe-pane` → FIFO → reader → broadcast) is byte-identical,
to avoid re-verifying `session:turn`. I checked it the strongest way available:

```
crates/seatloom-core/src/pty/mod.rs        300 insertions,   0 deletions
src-tauri/src/commands/session_cmds.rs     183 insertions,   0 deletions
```

Zero deletions in both files. The pre-fix code is a **byte-identical subset** of the post-fix
code — not "equivalent", not "refactored", literally the same lines with additions around
them. The only 4 deletions in the whole commit are in the UI (`LiveSeatsView.tsx`,
`useLiveSeatsMirrorStore.ts`), which is not the capture path.

**So the claim holds, and the `session:turn` re-verification it was protecting is genuinely
unnecessary.** This is the correct instinct and it is worth saying so: the temptation on a fix
like this is to "tidy" the capture path while you are in there, and he explicitly did not.

## 2. Hazard 1 — `pane_pipe` stays `1` after the steal. Confirmed on the real machine

I did not take his word for it. I created a real tmux session, attached a real FIFO the way
the app does, then stole it the way the incident happened (piping a second pane into the same
FIFO):

```
before any pipe:      pane_pipe = 0
after attach:         pane_pipe = 1
victim after steal:   pane_pipe = 1     <-- still 1
thief after steal:    pane_pipe = 1
```

**His claim is exactly right, and it is the reason the guard has to exist.** tmux's own report
is structurally incapable of distinguishing a healthy mirror from a stolen one, so any check
built on `#{pane_pipe}` would report "fine" forever. The only question that has an answer is
"which processes are writing my FIFO", which is what he built.

## 3. Hazard 2 — a real two-writer scenario. Confirmed, and here is the correction

I constructed the real thing on the live machine: two panes piped into one FIFO. The actual
`ps` output was exactly the shape he described:

```
55580  1951 sh -c cat >> /private/tmp/seatloom-mirror/fluxprobe.fifo
55583 55580 sh -c cat >> /private/tmp/seatloom-mirror/fluxprobe.fifo
55734  1951 sh -c cat >> /private/tmp/seatloom-mirror/fluxprobe.fifo
55737 55734 sh -c cat >> /private/tmp/seatloom-mirror/fluxprobe.fifo
```

Fed to the production classifier (`probe_with_snapshot`, the real function, not a fixture):

| Snapshot | Verdict |
|---|---|
| the live machine above (4 lines, 2 panes) | **`CrossWired { writers: 2 }`** |
| one healthy wrapper/child pair (2 lines) | **`Owned`** |
| a healthy pair **plus** a `cat <fifo>` reader | **`Owned`** (a reader is not a writer) |
| a healthy pair plus a mere `grep` mention | **`Owned`** (a mention is not a writer) |
| an empty snapshot | **`ProbeFailed`** |
| `pane_pipe_ok = false` | **`Lost`**, short-circuited before processes are read |

**His hazard-2 claim is confirmed, and the wrapper/child collapse is load-bearing.** I mutated
the collapse rule away and got **RED 10** — without it, every healthy mirror on this machine
(eight of them) is reported `cross_wired` on every probe, forever, which is the worst possible
failure mode for a check because it trains the operator to ignore it.

### The correction: the store's removal is redundant, and the criterion is verified by the view

His mutation claim is "store records but does not delete the attachment → 3 tests red". I
reproduced that: **RED 3**, all three in `useLiveSeatsMirrorStore.mirrorLost.test.ts`. Accurate.

**But** when I break the store's removal *and* check the view test alone, it stays **green**
(14 passed). The reason is that `LiveSeatsView.tsx` calls `removeAttachment` **itself**, in its
own `session:mirror-lost` handler (line 198), *in addition to* `markMirrorLost` doing it in the
store. So:

- the store-level half of the criterion is covered by the store's 11 tests (which is where his
  RED 3 comes from);
- the UI-visible half — "the terminal is actually unmounted" — is delivered by the **view's**
  own call, and I confirmed it only goes red when **both** removal sites are broken.

This is not a defect: the belt-and-braces is defensible, and the criterion is genuinely met. But
it does mean the primary checkpoint the dispatch names ("there must exist a path where the UI
does not continue displaying") is satisfied **twice**, and the store tests that look like they
are pinning it are pinning the redundant copy. If someone later removes the store's removal
believing the store tests would catch it, nothing breaks — which is the opposite of what those
tests are there for. **Worth knowing, not worth changing.**

The other three mutations, all as he claimed: view does not subscribe → **RED 2**; byte
counting after the event → **RED 1**; store records without removing → **RED 3**.

## 4. Hazard 3 — `probe_failed` is not folded into `owned`. Confirmed

`is_broken()` is `!matches!(self, Owned)`, so `ProbeFailed` **is** broken, and the UI has a
distinct sentence for it ("cannot read the process table to prove it still belongs to this app — display stopped") rather than the `lost` or
`cross_wired` one. I verified the empty-snapshot path returns `ProbeFailed` directly.

One consequence worth flagging because it is a real behaviour rather than a criticism: **a
transient `ps` failure unmounts the operator's terminal.** That is the right trade — the
alternative is showing unvouched bytes — but it means a `ps` hiccup costs a re-attach. He chose
the direction that cannot lie, and said so in the code. I agree with the choice.

## 5. R3 — the reaper works, and it is wired to a command nothing calls

**The reaper itself is correct, and I verified it end-to-end through the public API against the
real `/tmp/seatloom-mirror`**, with the live path spelled the way `lsof` reports it
(`/private/tmp/...`) while the directory is listed as `/tmp/...`:

```
BEFORE: ["fluxorphan2.fifo", "fluxorphan1.fifo", "fluxlive.fifo"]
REMOVED: 2
AFTER:   ["fluxlive.fifo"]
idempotent: a second pass removes 0
```

The live FIFO survives the exact `/private` vs `/tmp` mismatch. I also reproduced the original
bug he describes: with **raw** string comparison, the live FIFO **would** be deleted. So his
second self-caught bug is real and his fix for it is real. His regression test
`a_live_fifo_spelled_private_tmp_is_not_mistaken_for_an_orphan` is the right shape, and his
note about the first version of that test being wrong for the wrong reason (built `private/tmp/...`
*under* a temp dir instead of prefixing the temp dir itself) is the kind of detail that is almost
never written down and should be.

**Rotation** also verified on the real filesystem: three rotations leave **one** file, `.1`,
overwritten — no chain. That matches his "not an unbounded directory by another name" reasoning.

### The gap: `cmd_kill_session` has no callers

The dispatch criterion is "after a round of attach/detach, `ls /tmp/seatloom-mirror | wc -l` equals the number of live sessions; one extra means it is not fixed". I traced every path:

- `cmd_kill_session` is registered in `main.rs` and bound as `killSession` in `ui/src/lib/api.ts`
  — and has **zero call sites in the frontend**. `grep -rn killSession ui/src` returns only the
  binding itself.
- The UI's detach button calls `clearSession` (store-local) and `removeAttachment` (store-local).
  Neither reaches the backend.
- The other backend removal path — the pty-exit handler at `session_cmds.rs:457` — removes the
  session from `state.sessions` but does **not** run the reaper.

**So the reaper runs only inside a command the UI never invokes, and the criterion as written
cannot pass.** His own measurement (48 FIFOs for 9 live mirrors, attributed to "readers that
exited early") is the symptom of exactly this: he measured the leak correctly and then hung the
cleanup off a path that does not run on a normal detach.

**This is pre-existing, not a regression he introduced.** I checked the pre-fix commit:
`killSession` existed in the API layer before this dispatch and was never called then either;
the reaper is new. So he attached the cleanup to a dead path rather than breaking a working one.
**Reported, not fixed** — the dispatch said to report findings, and choosing where a reap belongs
(a detach command? the pty-exit handler? a periodic sweep?) is a design decision for him and
Aegis, not a thing to slip into someone else's commit during a verification.

Two related consequences of the same gap, both measured rather than assumed:
- a UI detach leaves the backend session entry in `state.sessions` forever, and with it the
  probe task, whose only exit condition is `probe_sessions.contains_key(&id)`;
- the FIFO stays on disk.

## 6. R2 — pane identity, verified on the real machine both ways

He asked for `pane_pid` / `pane_tty` shown, `?` when tmux will not answer, never fabricated. I
tested the production helper against real targets:

```
Aegis-Supervisor:0.0   -> pid=1953    tty="/dev/ttys001"
fluxprobe:0.0          -> pid=53594   tty="/dev/ttys018"
no-such-session:0.0    -> pid=0       tty=""          <-- never fabricated
```

And the display path, driving the real view: with a real pid/tty in the DTO the view shows
`pid 1953` and `ttys001` (`/dev/` stripped) with **no** question mark; with an absent one it
shows `pid ?` / `tty ?`. **Both halves verified.** His reasoning is right — this is the single
cheapest thing that would have shortened the incident from three hours.

## 7. R4 — the wording, checked

`ENGINEERING_STANDARDS.md` §"Live tmux debugging — `pipe-pane` is destructive" is correct:
query with `display-message -p '#{pane_pipe}'`; `pipe-pane` with no argument **stops** the pipe;
a pipe is per-pane and not reference counted; and "growth is not correctness". I confirmed each
statement against tmux's actual behaviour on this machine, including that a second `pipe-pane`
replaces the first (I did exactly that during hazard 1 and the first `cat` died). **Accurate.**

## 8. Gates

- His UI tests: `LiveSeatsView.test.tsx` **14 passed**, `useLiveSeatsMirrorStore.mirrorLost.test.ts`
  **11 passed**.
- Full UI suite: **325 passed / 26 files / 0 errors**. `tsc --noEmit` clean.
- **eslint 145 problems before AND after** — I diffed against `9a67ea5~1` on the whole `src`
  tree and the count is identical, so his "no new lint" claim holds. Prettier's complaints about
  `LiveSeatsView.test.tsx` and `api.ts` are also pre-existing (both were already listed as
  different before the commit).
- Rust: `cargo test -p seatloom-core` **196 passed**, `pipe_guard` **14 passed**, `pty::` **37
  passed**. `cargo clippy` clean on both crates, `cargo fmt --check` clean,
  `cargo check -p seatloom-tauri` clean (needed `ui/dist` present, which is a worktree artefact,
  not a code problem).

## 9. What I did on the live machine, and what I undid

Stated plainly because this dispatch is about a bug caused by exactly this kind of intervention:

- Created a throwaway tmux session `fluxprobe` and a FIFO, attached and then **stole** the pipe
  to reproduce the defect. **Killed the session and stopped the Aegis pipe afterwards** —
  verified `#{pane_pipe}` is back to `0` and no `fluxprobe` writer remains.
- Created FIFOs in `/tmp/seatloom-mirror/` to exercise the reaper. The reaper removed them
  itself as designed.
- **I did not delete any live mirror.** The three mirrors the app is currently reading
  (`ses-18e2601733f91190`, `ses-18e24dac31302458`, `ses-18e24db9a0d536d8`) each still have their
  writer holding the **same inode** as the app's reader — I checked inode by inode — so they
  keep flowing. `/tmp/seatloom-mirror` was already empty when I started (directory mtime 20:23,
  before my session), so nothing live was under the glob I ran.
- One thing I observed and did **not** act on: `ses-18e24dac31302458` has **two** writers on the
  same inode, which is precisely the `CrossWired { writers: 2 }` condition. That is the operator's
  app to decide about, not mine to kill — and killing writers is what caused the previous
  incident, which his own note records (`pkill -f "cat >> ..."`). **Reported only.**

## 10. What I did not do

- Did **not** merge, rebuild, or reinstall. `9a67ea5` is not on `track`; the running binary is
  14:03 against a fix committed later, and none of the pipe-guard strings are in it. Reported.
- Did **not** wire the reaper to a path that runs, or change the double removal in the view.
  Both are design decisions for Nimbus and Aegis.
- Did **not** touch Nimbus's worktree beyond a detached worktree at `9a67ea5`, removed
  afterwards; his `git status` is empty.
- Did **not** write to the live database or open the acceptance card.

## Reproduction

```bash
git -C <nimbus> worktree add --detach /private/tmp/mirror 9a67ea5
ln -s <nimbus>/ui/node_modules /private/tmp/mirror/ui/node_modules
ln -s <seatloom>/ui/dist      /private/tmp/mirror/ui/dist      # cargo check needs it
cd /private/tmp/mirror/ui

./node_modules/.bin/vitest run src/views/LiveSeatsView.test.tsx                # 14 passed
./node_modules/.bin/vitest run src/stores/useLiveSeatsMirrorStore.mirrorLost.test.ts  # 11
./node_modules/.bin/vitest run                                                 # 325 / 26 files
./node_modules/.bin/tsc --noEmit                                               # clean
./node_modules/.bin/eslint src                                                 # 145 (same as 9a67ea5~1)
cd .. && cargo test -p seatloom-core --lib                                     # 196 passed
        cargo test -p seatloom-core --lib pipe_guard                           # 14 passed
        cargo clippy --workspace --all-targets && cargo fmt -- --check         # clean

# HAZARD 1 — pane_pipe stays 1 after a steal (real tmux, real FIFO)
tmux new-session -d -s probe -x 200 -y 50
tmux display-message -p -t probe:0.0 '#{pane_pipe}'        # 0
tmux pipe-pane -t probe:0.0 "cat >> /tmp/seatloom-mirror/x.fifo"
tmux display-message -p -t probe:0.0 '#{pane_pipe}'        # 1
tmux pipe-pane -t Aegis-Supervisor:0.0 "cat >> /tmp/seatloom-mirror/x.fifo"   # STEAL
tmux display-message -p -t probe:0.0 '#{pane_pipe}'        # STILL 1  <- tmux cannot tell

# HAZARD 2 — a real two-writer snapshot through the production classifier
ps -Ao pid=,ppid=,command= | grep x.fifo                  # 4 lines: 2 panes x (wrapper+child)
# -> probe_with_snapshot(...) = CrossWired { writers: 2 }
# one healthy pair  -> Owned      pair + a reader -> Owned
# a mere mention    -> Owned      empty snapshot  -> ProbeFailed

# MUTATIONS (each reverted, tree verified identical afterwards)
#   store records but does not remove the attachment -> RED 3 (store tests only;
#                                                       the VIEW test stays green — see §3)
#   the view also stops removing                    -> only NOW does the view test go RED
#   the view does not subscribe to the event        -> RED 2
#   byte counting after the event is not guarded    -> RED 1
#   the wrapper/child collapse is removed           -> RED 10

# HAZARD 3 / R3 — the reaper through the PUBLIC api, real dir, real /private mismatch
mkfifo /tmp/seatloom-mirror/live.fifo /tmp/seatloom-mirror/orphan{1,2}.fifo
# reap_orphan_fifos(&["/private/tmp/seatloom-mirror/live.fifo"])
#   -> REMOVED 2, live survives, idempotent on a second pass
# with RAW string comparison the live fifo would be deleted (the bug he caught)

# R3 rotation, three times: exactly one file, `.1`, overwritten. No chain.
```

## Verdict event

`ev-mirror-pipe-guard-flux-verdict-01`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-jZg89gwy8YzVOOjdAsxTL` in the payload.

## Files I did not change

No source file. Five mutations were applied and each reverted, with the tree confirmed
byte-identical to `9a67ea5` after each. One detached worktree was created at Nimbus's commit and
removed afterwards. My probe files were deleted.
