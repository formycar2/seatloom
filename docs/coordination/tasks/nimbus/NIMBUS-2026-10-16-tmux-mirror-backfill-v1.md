# Delivery: the live-seat mirror backfills the pane's scrollback, with colours intact

[Nimbus -> Aegis] Mr Zhang: on the tmux mirror page, without lowering display
quality, be able to scroll up and see more context — right now he can barely
scroll up at all. Root cause measured on a live seat pane, not taken from the
dispatch. 66 lines became 3 339.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-16-tmux-mirror-backfill-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-16 |
| version | v1 |
| to | aegis |
| dispatch | `disp-UZR6aALIcTeeLBT9R_GG5` |
| contract | `disp-UZR6aALIcTeeLBT9R_GG5` payload |
| depends_on | NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1 (the `-o` lesson), docs/coordination/reviews/2026-05-09-nimbus-seatloom-full-arch-design-v1 §6b-B |
| branch | `packet/nimbus/gate-vs-announcement` |
| tags | pty, tmux-mirror, scrollback, backfill, history-limit, quality |

---

## 1. The root cause, measured

I verified it myself rather than taking the dispatch's word, and it is exactly
where the dispatch said — with one addition worth stating.

| # | What | Measurement |
|---|---|---|
| 1 | `attach_tmux` captured the pane with `capture-pane -e -p` — **no `-S`** | On `Nimbus-TechArchi-seatloom:0`: **66 lines** captured, while the same pane's `#{history_size}` was **3 273** |
| 2 | `pipe-pane` forwards only output produced **after** it starts | So the mirror began at the attach instant and those 3 273 lines were unreachable |
| 3 | tmux's `history-limit` is **2 000** | `tmux show-options -g history-limit` → `2000`; every seat pane the same |

`capture-pane -p` with no `-S` returns the visible pane. That is the whole defect:
the mirror was live from the attach instant forward, and everything before it was
invisible.

**The frontend is not the bottleneck, confirmed.** `SessionTerminal.tsx:109` sets
`scrollback: 10000` — the UI has had room for ten thousand lines the whole time.
The backend was never sending them.

## 2. What was built

| File | What |
|---|---|
| `crates/seatloom-core/src/pty/mod.rs` | `DEFAULT_BACKFILL_LINES`, `DESIRED_HISTORY_LIMIT`; two new `LaunchOptions` fields; the capture now uses `-S`; a session-scoped `history-limit` raise; 4 new tests |
| `src-tauri/src/commands/session_cmds.rs` | `AttachTmuxRequest` gains optional `backfill_lines` / `history_limit`, forwarded verbatim |
| `crates/seatloom-mcp-emit/proof/tmux_mirror_backfill_proof.sh` | new — 8 points against real tmux panes |
| 3 examples | the two new `LaunchOptions` fields |

**The backfill does NOT go through the FIFO.** It rides on `initial_snapshot` —
the same field that already keeps an idle pane from rendering black. The dispatch
asked for "write the history into the FIFO first, then start pipe-pane". I did not
do it that way, and this is the measurement behind the deviation:

- **It deadlocks.** Opening a FIFO for write blocks until a reader opens it, and
  the reader thread starts *after* `pipe-pane`. Writing history first would hang
  the attach.
- **It interleaves.** With both ends held open, tmux's `cat >>` and a history
  writer race for the pipe. The byte order in the FIFO is then whatever won the
  race — precisely the "history and live output interleaved or duplicated"
  failure the dispatch says to avoid.
- **The existing path is already race-free.** `initial_snapshot_b64` is delivered
  with the session, and the frontend writes it to xterm **after** registering the
  `session:output` listener. No ordering hazard with `pipe-pane` at all.

And re-attach cannot duplicate: the terminal is keyed by `a.dto.id`
(`LiveSeatsView.tsx:546`), and each attach mints a new session id, so React
remounts the terminal and writes the snapshot exactly once. The proof asserts
that key exists, so a future refactor that drops it fails a gate.

## 3. The numbers, derived rather than guessed

**Backfill: 5 000 lines.** Derived from the terminal's own limit. The UI keeps
`scrollback: 10000`. Backfilling all 10 000 would leave the operator **zero** room
for the output that arrives after they attach — the history would push the live
tail straight out of the buffer. So the scrollback is split in half: 5 000 lines of
history, 5 000 left for live output. A unit test pins `DEFAULT_BACKFILL_LINES <
10_000` in a `const` block so the two cannot drift apart silently.

The budget counts the **whole** capture (history + visible pane), so the history
portion is `backfill - pane_rows`, floored at 1. Without that subtraction a
5 000-line budget on a 66-row pane would deliver 5 066 lines and quietly exceed
what the terminal keeps.

**`history-limit`: 50 000.** Ten times the backfill, so the backfill is never
silently truncated by the limit. Measured **free at rest** — a pane at limit
2 000 / 50 000 / 100 000 has RSS 3 232 / 3 248 / 3 232 KB. tmux allocates the
history grid lazily, so a higher limit costs nothing until a pane actually
produces that much output, which is exactly when the operator wants it. It is also
the order of magnitude the dispatch suggested.

Both are configurable (`backfill_lines` / `history_limit` on the attach request),
`None` meaning "use the documented default" — so the default has exactly one home
and an omitted field is never rewritten to a number in the wrapper.

## 4. Quality did not drop — measured

`-e` is **not optional**: without it tmux returns plain text and every colour,
bold and progress bar is lost, which the acceptance criterion names as a reject.
Measured on the live seat pane: **1 318 of 3 339** captured lines carry escape
sequences. The frontend writes the snapshot as **raw bytes** (`term.write(bytes)`),
so xterm parses the sequences natively — the rendering path is unchanged.

## 5. What a restart does and does not fix — measured

Raising `history-limit` **only affects panes created after it is set.** tmux reads
the option when a pane is created.

| | Before | After the raise |
|---|---|---|
| The pane being attached | `2000` | `2000` — **unchanged** |
| The next pane created in that session | — | `50000` |

So: **a seat that wants more than it already has needs its pane recreated** —
for a seat that means relaunching it (`scripts/seat-launch.sh`), which creates a
new pane in the same session. Until then the backfill returns whatever the pane
already retains, which is the honest answer and not a failure. The raise is
**session-scoped, never `-g`** — mutating the server-wide option from a desktop
app would change every pane the operator has open anywhere. A failure to set it is
non-fatal and logged, because the backfill still returns what the pane has.

## 6. Gates run

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 174 passed / 0 failed (2 ignored, need Postgres) — 4 of them this slice |
| `cargo test -p seatloom-mcp-emit` | 85 passed / 0 failed (19 ignored) |
| `cargo test -p seatloom-mcp-emit --bin seatloom-waker` | 9 passed / 0 failed |
| `cargo check -p seatloom-tauri` | clean |
| `cargo check --workspace --all-targets` | clean |
| `proof/tmux_mirror_backfill_proof.sh` | ALL POINTS PASS |
| `proof/operator_ref_config_proof.sh` | ALL POINTS PASS (re-run, unchanged) |
| `proof/lineage_ref_proof.sh` | ALL POINTS PASS (re-run, unchanged) |
| `scripts/verify-docs.sh` | this note is clean |

**The proof caught two of my own bad assertions**, which is why it exists:

1. I asserted the capture would use "at least 4 900 of the 5 000-line budget". It
   returned 3 339 — because the pane only *holds* 3 339. The budget is a ceiling,
   not a promise; the assertion is now `min(budget, history + visible)`.
2. I asserted a fresh pane's `history_size` is `0`. It was `1`. The invariant that
   matters is `capture == history + visible`, never less, so that is what is
   asserted.

## 7. Acceptance — what I measured, and what is yours to click

**Measured by me, against real tmux panes:**

| Criterion | Result |
|---|---|
| Scroll up and see what the seat did before attaching | 3 273 unreachable lines → **3 339 delivered** |
| Colours and layout unchanged | 1 318 of 3 339 lines carry escape sequences; written as raw bytes, parsed natively by xterm |
| Matches what `tmux attach` shows | Same `capture-pane` call tmux users read; the proof asserts `capture == history + visible` |
| Re-attach does not duplicate or interleave | `pane_pipe` 0 → 1 → 1 across two plain attaches (the `-o` bug is not back); terminal keyed by session id |
| Live scrolling unaffected | The backfill is a separate field written before the live stream; the pipe-pane path is byte-for-byte unchanged |

**Yours to click, which I cannot run and am not claiming:**

1. Open the live-seats view, attach a seat, scroll up. **Expected: a large amount
   of what that seat did before you attached, in the same colours.**
2. Re-attach the same seat. **Expected: no duplicated history, no interleaving.**
3. Watch a seat mid-turn. **Expected: output still streams live, unchanged.**

**What this build will and will not show.** The backfill itself works on the
already-running seat panes immediately — it reads whatever the pane retains, which
is ~3 300 lines today. The **full 5 000-line backfill needs a pane created after
the `history-limit` raise**, i.e. a relaunched seat (§5). And as with the previous
dispatch, the running app is the previously installed binary: it must be rebuilt
and reinstalled (`scripts/prepare-verification-build.sh`) to carry this change at
all. **The card and the build must arrive together.** I did not rebuild,
reinstall or restart anything — stopping the operator's running app is not mine
to do.

## 8. Known limits, stated rather than hidden

- **A seat must be relaunched to get the full 5 000-line backfill.** Until then it
  gets whatever its pane already retains (~3 300 lines measured). That is 50× what
  it gets today, so it is a large improvement either way.
- **The backfill is bounded by `history-limit`; asking for more silently returns
  less.** That is exactly why the two constants are 10× apart, and the proof
  measures the truncation (asked for 100 000, got 3 339).
- **Output produced between attach and the frontend subscribing is still lost.**
  The broadcast channel does not replay to a late subscriber. This is pre-existing,
  adjacent to the dispatch's boundary ("the backend session's survival after the
  UI unloads is another piece of work in flight"), and NOT introduced or worsened
  here — but it is the next thing to fix if he reports "I miss the first few
  seconds after attaching".
- **A ref containing a newline is not expressible** in the one-line file from the
  sibling dispatch; noted there.

## 9. Out of scope, unchanged

The FIFO's lifecycle, the backend session's survival after the UI unloads, the
frontend terminal component, and the merge logic. The pipe-pane invocation is
byte-for-byte what it was — no `-o`, and the proof pins that again.

## 10. Forward recording (D5 Phase 2 Lane B)

One memory, `source_kind=canonical_event`, `source_ref` = the delivery
announcement, `confidence=quoted`, topic `topic-tmux-scrollback-backfill`:

| Memory | Kind | Why it passes the three-way test |
|---|---|---|
| `mem-SVnpTKhuwH3dkpOvycfwS` | fact | The backfill rides on `initial_snapshot`, NOT the FIFO, and the reason is measured (a FIFO write before pipe-pane deadlocks; with both ends open the writers race and interleave). Changes what a future seat does when asked to "write the history into the FIFO first"; exists nowhere else; evidenced by the FIFO open-blocks-until-reader semantics and the `key={a.dto.id}` remount. |

Nothing else was recorded. The two numbers are derived from constants already in
the tree (`scrollback: 10000` in the UI) rather than being new facts, and the
restart semantics are a property of tmux documented in §5. No preference was
touched — Mr Zhang's corrections are Aegis's to record.
