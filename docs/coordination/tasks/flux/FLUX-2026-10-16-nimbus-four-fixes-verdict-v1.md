# Verdict — Nimbus's four 2026-10-16 fixes, on the clean branch `packet/nimbus/verified-fixes-20261016`

- **Verification dispatch**: `disp-8x5OErFkgktn49IN6tB-c` (issued by seat-aegis-001).
- **Branch under test**: `packet/nimbus/verified-fixes-20261016` — 11 commits on top of
  `track/infra-foundation`, tip `f291baf`. Verified at the tip, read-only: nothing committed on
  that branch, no tracked file touched, no service restarted, live DB read-only.
- **Verdict event**: `ev-fFfLGnJIQu5trM1qVHJCq` — the pre-declared id, emitted exactly as the
  dispatch requires, with one `ref_type='dispatch'` context ref per verified dispatch.
- **VERDICT: PASS — all four fixes, each against its own dispatch's original criteria.**

The packet path in the dispatch payload (`docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md`)
does not exist on any branch I can see except `packet/lyra/milestone-summary`, and it is the
session↔work bridge contract — a different unit of work. I worked from the payload's inline note,
which is self-contained, and I pulled each fix's acceptance standard from its own dispatch
(`disp-UZR6aALIcTeeLBT9R_GG5`, `disp-ftEie9vQ9MJEh0lyv7YkA`, `disp-SfHOAaw2hl0EfvAZdAxFv`,
`disp-Krsp8j-EDOpWDIiWjuHo_`), as the note instructs. **Reported to Aegis: the packet pointer on
this dispatch is wrong; the note was not.**

---

## Gates, run by me at the tip of the branch under test

| Gate | Result |
|---|---|
| `cargo check --workspace` | clean |
| `cargo test -p seatloom-core` (lib) | **164 passed / 0 failed** (2 ignored) |
| `cargo test -p seatloom-mcp-emit` (no DB) | **97 passed / 0 failed** — includes the waker's 16 |
| `cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit -- --ignored` on scratch `seatloom_verdict_gate` | **22 passed / 0 failed** |
| Nimbus's proof `tmux_mirror_backfill_proof.sh` | **ALL POINTS PASS** (8) |
| Nimbus's proof `acceptance_verdict_gate_proof.sh` | 5/6 points as committed — **POINT 1 dies on a stale live assertion**; re-run with the drift measured honestly → **ALL POINTS PASS** |
| Nimbus's proof `operator_ref_config_proof.sh` | **ALL POINTS PASS** (8) |
| Nimbus's proof `heartbeat_call_proof.sh` | **ALL POINTS PASS** (7) — with the proof's own documented `SEATLOOM_WAKER_BEFORE_REF` override |

Every number Nimbus reported in his dispatches reproduces exactly. I did not take his self-report
as the conclusion; each proof was re-run by me, and where a proof's own baseline or live-state
assertion did not fit this branch I say so below rather than passing it silently.

---

## Fix 1 — `disp-UZR6aALIcTeeLBT9R_GG5`: the mirror backfills the scrollback, colours intact — **PASS**

**What the criterion demanded** (from the dispatch): `capture-pane` with `-e` (or colours are lost)
and with `-S` (or there is no history); history written before live output starts, never
interleaved or duplicated; a configurable line count; `history-limit` raised; the no-`-o` lesson
untouched; live output unaffected.

**What the code does** (`crates/seatloom-core/src/pty/mod.rs`, commit `4b5e317`):

- The capture is now `capture-pane -t <target> -e -p -S -<history_lines>`, where
  `history_lines = backfill − visible_rows` floored at 1 (`DEFAULT_BACKFILL_LINES = 5_000`, derived
  as half the terminal's own `scrollback: 10000`). `-e` is present — the proof measured **1 311 of
  3 339** captured lines carrying escape sequences. `-S` is present — the proof reproduced the
  defect first: the old `-e -p` with no `-S` returned **66 lines** on a pane holding **3 273** of
  scrollback, and the new capture returns **3 339 = history + visible**.
- `history-limit` is raised to **50 000**, session-scoped (`set-option -t <session>`, never `-g`),
  and the proof measured that a raise only affects panes created after it (existing pane stays at
  2 000, the next pane gets 50 000) and costs nothing at rest (RSS identical at 2 000 / 50 000 /
  100 000). The delivery states the restart/relaunch consequence instead of papering over it.
- **The one deviation, and it is measured, not a shortcut.** The dispatch said to write history
  into the FIFO before starting `pipe-pane`. Nimbus instead rides `initial_snapshot`, with the
  measurement behind it: a FIFO write before `pipe-pane` deadlocks (the reader thread starts after
  `pipe-pane`), and with both ends held open the writers race and interleave — the exact failure
  the dispatch forbids. I verified the delivery path myself: `initial_snapshot` is base64'd to the
  frontend (`session_cmds.rs`), and `SessionTerminal.tsx` writes it to xterm **after** the output
  listener is registered, so ordering is history-then-live by construction, and each attach is a
  new session id, so the terminal remounts — no re-attach duplication. The proof's POINT 8
  confirms `pane_pipe 0 → 1 → 1` across two attaches with no `-o`.
- The dispatch's boundary (FIFO lifecycle untouched, no frontend terminal changes) is respected:
  the diff touches `pty/mod.rs`, the attach request DTO and three proof examples only.

**Re-attach and live-output criteria**: POINT 8 (re-attach safe, no duplication, no `-o`) and the
byte-level path above (live stream untouched) both hold. 66 → 3 339 lines with colours, on a real
seat pane. **PASS.**

---

## Fix 2 — `disp-ftEie9vQ9MJEh0lyv7YkA`: the merge gate reads the verdict's value and subject — **PASS**

**What the criterion demanded**: a card citing a **fail** verdict must be refused, naming the
verdict and its value; a card citing a green verdict **about something else** must be refused,
saying why; the normal flow must not regress; and the refusal must not be one opaque sentence.

**What the code does** (commit `fd274d3`): a new `VerdictFacts` read (`flux_verdict_facts`) answers
exists / event_type / verdict value / payload dispatch_id / names-dispatch, where `names_dispatch`
is true iff `payload->>'dispatch_id' = <dispatch>` **or** an `event_object_refs` row with
`ref_type='dispatch'` names it — the two channels Flux actually writes (measured: 27 of 30
verdicts via payload, 5 of 30 via refs; `lane` was rejected as free text, correctly).
`is_merge_criterion()` ANDs all four; there is deliberately **no `is_green()`**, because the proof
caught that a value-only method is the half-criterion this dispatch exists to close. Both gates
consume it: `cmd_accept_delivery` renders three distinct refusals (missing / not a verdict event /
red-or-valueless / not about this dispatch), each naming the ref, its value and whose it is; and
`request_acceptance`'s guard 2 refuses with typed `VerdictNotGreen` / `VerdictNotAboutThisDispatch`
before any card is written.

**Verified by me, not by his proof alone**:

- The probe against **live**: for both cards that cite `ev-verdict-history-sessions`
  (`acc-disp-history-sessions-ui`, `acc-disp-ol420Y586tNIsbbeh5nlU`) the read answers
  `exists=true verdict=Some("fail") names_dispatch=false green=false` — the incident's exact shape,
  now refused.
- A green verdict about another dispatch: `ev-NFPr-zN36by-i9SE-0FPH` (pass, about
  `disp-65NYElf3D-Ha7tXOTSguG`) asked about `disp-some-other-dispatch` → `green=false`; asked about
  its own dispatch → `green=true`. **No regression.**
- The three required tests exist (`a_fail_verdict_cannot_be_the_merge_criterion`,
  `another_dispatches_verdict_cannot_be_the_merge_criterion`,
  `a_green_verdict_about_this_dispatch_still_opens_a_card`) and all **22** DB tests pass on a
  scratch database I rebuilt in the canonical order.
- The read is read-only on live (ledger counts unchanged across the probe calls).

**The harm, confirmed and still standing**: two cards were accepted and merged on that fail verdict
(merge `4cfd2302`, which exists in this repository). Flux re-verified those two dispatches at
08:46Z today (`ev-NFPr…`), so the work behind them now has a real verdict; whether the merges
themselves stand is Aegis's ruling, and nothing here reverts them.

**One proof defect, reported, not a fix defect**: the committed proof dies at POINT 1
(`expected at least 4 such cards, found 2`). Live moved on since it was written — the three
pending cards were withdrawn and are gone, and Flux's 08:46Z re-verification means
"Flux never verified these dispatches" is no longer true. The fix is not implicated; the proof's
live snapshot is brittle by construction. I re-ran the proof with the count measured honestly
(two cards, both the accepted harm) and every substantive point passes. **PASS.**

---

## Fix 3 — `disp-SfHOAaw2hl0EfvAZdAxFv`: the operator identity is configurable without the environment — **PASS**

**What the criterion demanded**: a Launchpad launch (no environment) must be able to merge; with
the config file deleted the error must say where to configure and what; no default may ever be
invented; the env channel must keep working.

**What the code does** (commit `04fa67e`): `operator_ref()` now delegates to
`seatloom_core::config::read_operator_ref()` — one implementation, unit-tested without Tauri.
Precedence **file > env**: `~/.seatloom/operator` (one line) first, `SEATLOOM_OPERATOR_REF` second,
**no third channel and no default** — with both silent the only outcome is a refusal. Four distinct
failures (absent / blank / unreadable / two stated identities), and the refusal names the file's
**absolute path**, the exact `echo` command to write it, that the ref must already exist in the
ledger, and that the file is read at click time so **no restart is needed**.

**Verified by me**: the proof's 8 points run against the real file and the real filesystem — the
real file on this machine (`/Users/jyxc-dz-0100609/.seatloom/operator` → `human:supervisor`)
resolves with **no environment at all**, which is exactly the Launchpad case; the delete-the-file
case produces the actionable refusal quoted above; a blank file and a two-line file are refused
rather than guessed; env still resolves; file beats env. 11 unit tests pin every branch and the
read writes nothing.

**What I could not do, stated plainly**: I am a CLI seat — I cannot click Accept in the GUI. What I
verified is the exact resolution path `cmd_accept_delivery` calls, with no environment and the real
file present. The click itself remains the operator's one-step test; the mechanism behind it is
proven. **PASS.**

---

## Fix 4 — `disp-Krsp8j-EDOpWDIiWjuHo_`: the waker calls an idle seat to its standing duties — **PASS**

**What the criterion demanded**: an idle seat with zero ready dispatches must be called to its
**standing duties**, not the drain instruction; a dispatched seat must receive the **byte-identical
old** instruction; a seat with **no heartbeat document** must not be called; cooldown must prevent
spam; the two existing paths must not break; and I must judge the restart consequence.

**What the code does** (commit `448e97e`, `crates/seatloom-mcp-emit/src/bin/waker.rs`): a third
match arm `(None, 0)` — the arm used to be `(None, _) => {}` — calls the seat when
`has_heartbeat_doc(...) && heartbeat_poker.allow(...)`. The document check is **per sweep, not
cached**, so writing a document connects a seat with no code change and no restart. The cooldown is
**separate and longer**: `DEFAULT_HEARTBEAT_COOLDOWN = 30 min` on its own `Poker`, versus the
dispatch cooldown's 120 s — 15×, at most two interruptions an hour, and tuning one never moves the
other. `NudgeKind::Dispatch` vs `NudgeKind::Heartbeat` are two variants of one nudge function, so
the messages cannot drift.

**Verified by me**: the proof's 7 points pass. POINT 2 extracts the dispatch literal from the
before-source and the after-source and compares them — **byte-identical**
(`📥 new governed dispatch available — run your dispatch loop now: …`). POINT 3: the heartbeat
message names the document and never mentions draining. POINT 4/5 against the real seat map
(`.seatloom/dispatch-seats.tsv`, 6 seats): exactly **1 of 6** has a document (Lyra) and is the only
one that would be called; the other five are left alone, measured on the real filesystem. POINT 6:
both cooldowns, no spam. POINT 7: the existing paths intact (the dispatch-kind call sites grew from
the old ones only by the heartbeat arm). Waker's own 16 tests pass.

The proof's baseline pin (`47079e1^`) is the original branch's commit id and does not exist on this
cherry-picked branch — the proof documents its own override for exactly this, and I ran it with
`SEATLOOM_WAKER_BEFORE_REF=448e97e^`, the true before-state here (the commit immediately below the
fix). That adaptation is to the proof's baseline, not to the fix.

**The restart consequence, judged as the dispatch asks**:

- **Before a restart** (today's state): the running waker is the previously built binary
  (`ps`: `./target/debug/seatloom-waker`, pid 72008, kept alive by a supervisor `while true` loop).
  Old behavior: Lyra's document exists but nothing executes it; **no seat is heartbeat-called at
  all**; dispatch nudges behave exactly as before.
- **After a rebuild + restart** (Aegis's call — it holds the fleet's wakeup path, and my boundary
  and Nimbus's both say do not restart services unsupervised): Lyra is called to
  `docs/coordination/tasks/lyra/LYRA-HEARTBEAT.md` within 30 minutes of being idle with zero ready
  dispatches, at most twice an hour; every other seat is untouched (no documents); a dispatched
  seat still gets the identical dispatch instruction. **No restart is needed for future documents** —
  the check is per sweep, so the next seat to get a `<SEAT>-HEARTBEAT.md` is connected the moment
  the file exists.
- **Deployment precondition, checked**: the waker resolves the document from its working directory,
  the main checkout on `track/infra-foundation` — where `LYRA-HEARTBEAT.md` (and Aegis's own
  `AEGIS-HEARTBEAT.md`) already exist. Note for the restart: `seat-aegis-001` is **not** in the seat
  map, so `AEGIS-HEARTBEAT.md` is not reached by the waker (his self-wake path is separate) — by
  design, stated in the delivery.

**PASS.**

---

## What I am flagging to Aegis (no action taken by me — boundaries)

1. **The dispatch's packet pointer is wrong** — it names the session↔work bridge contract, a
   different unit of work that does not exist on this branch. The inline note was the real task and
   I followed it. The gate now requires per-dispatch refs, and I emitted them.
2. **Two proofs carry environment-brittle assertions**: `acceptance_verdict_gate_proof.sh` POINT 1
   asserts a live row count (died today at 4-vs-2); `heartbeat_call_proof.sh` pins a baseline commit
   that does not survive cherry-picking (its override saved it). Both are proof-hygiene issues, not
   fix defects, and both are worth a follow-up so a future re-run does not die on drift.
3. **The waker restart is still pending** and is an operational act: until it happens, fix 4 is
   correct code with no effect. Nimbus reported it as `ev-heartbeat-call-needs-waker-restart` and
   correctly did not restart the daemon himself. I did not restart it either.
4. **The two unverified merges** (`4cfd2302`) still stand on `track/infra-foundation`; the work
   behind them has since been genuinely verified by me, but whether the merges stand is Aegis's
   ruling.

## Bottom line

All four fixes **PASS** their own dispatches' criteria, measured by me at the tip of
`packet/nimbus/verified-fixes-20261016`: gates clean, four proofs re-run and passing, live-shaped
checks performed where the proofs allow, and every deviation from a dispatch's letter either
measured and justified (fix 1's snapshot-over-FIFO) or flagged (the two brittle assertions, the
wrong packet pointer, the pending restart).
