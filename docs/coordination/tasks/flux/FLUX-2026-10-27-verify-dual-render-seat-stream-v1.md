# Flux verdict — the turn-aware second render, and the phantom turns my probe found

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-dual-render-seat-stream-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | reverification, mutation-testing, own-probe-defect, latency-measurement, seat-flux |

Dispatch `disp-06a95xPk7h6E2pj6FDo4O`, request 2 of 3.
Subject: `disp-98QeI7oCe_posblRFa_nf` — Nimbus's turn-aware second render, commit `3b52ea2`.
Verdict event: `ev-RZkE0mWIuSNWp0FmpJ8nU` (this dispatch's own id, payload naming `disp-98QeI7oCe_posblRFa_nf`).

## Verdict: PASS, with one finding that needs a follow-up

Every gate figure in his delivery reproduces exactly. The dependency the dispatch named
as the thing to watch — the pipe-pane capture chain — is verifiably untouched. But my own
probe found a defect his 8 tests do not cover: **after one short turn, ordinary output
emits phantom `TurnStarted`/`TurnEnded` pairs.** It is bounded, so I am not blocking on
it, but it is real, it is in the direction that would make the UI lie, and it needs a fix.

## The dependency claim, verified

The dispatch said "this one depends on the pipe-pane capture chain being untouched".
Measured on `3b52ea2`:

- `crates/seatloom-core/src/pty/mod.rs` — **zero diff** (`git show 3b52ea2 --numstat -- crates/seatloom-core/src/pty/mod.rs` is empty).
- The whole diff is 5 files: `turn_stream.rs`, `session_cmds.rs`, two UI files, the note.
- In `session_cmds.rs` the only deletions are a reformatting of the `TurnStarted`/`TurnEnded`
  enum variants onto one line. **No capture-chain deletion.**
- The new event is strictly additive: in the `Ok(PtyEvent::Output(bytes))` arm,
  `session:output` is emitted **first**, unchanged, with the same base64 payload, and
  `session:turn` follows on its own event name. `session:output` below is unchanged and
  xterm keeps reading. A consumer that only wants the terminal gets exactly the old bytes.

This is the strongest form of the claim: not "I didn't intend to touch it" but the diff
is empty and the new path sits downstream of the old one.

## Gate figures — every one reproduced

| Check | His claim | My measurement at `3b52ea2` |
|---|---|---|
| `App.supervisorSync.test.tsx` (dispatch-named) | 20 passed | **20 passed** |
| UI full suite | 25 files / 311 tests | **25 files / 311 tests** |
| `ui` `tsc --noEmit` | clean | **clean** |
| `cargo test --workspace` | 305 passed, 0 failed | **305 passed, 0 failed** |
| `cargo clippy --workspace --all-targets` | 0 warning | **0** |
| `cargo fmt --all -- --check` | clean | **clean** |
| `turn_stream` unit tests | 8 | **8 passed** |
| `useConversationStore` | 11 | **11 passed** |

Note on reproducing the Rust numbers: in a detached worktree `cargo test --workspace`
fails to compile `seatloom-tauri` with `proc macro panicked`, because `ui/dist` does not
exist there. Symlinking `ui/dist` resolves it — the same worktree hygiene point his own
delivery makes about `node_modules`.

## Mutations — mine, plus his three

| # | Mutation | His claim | My result |
|---|---|---|---|
| MT-A | drop bytes when no turn is open | RED 1 | **RED 5** |
| MT-B | `MAX_TURN_TEXT_BYTES` = 10 | RED 3 | **RED 3** |
| MT-C | `turnEnded` closes all turns | green (equivalent) | **green** — equivalent confirmed |

MT-A kills **more** tests than he reported, which is the safe direction: his tests are
stronger than he claimed. MT-C I re-ran specifically because he reported it as
equivalent rather than forcing a red — he is right, only one turn can be open at a time,
so closing all of them is the same behaviour. He chose not to break the code to
manufacture a red test, which is the correct call.

## Latency — his 8.6 µs is correct, and only in the right profile

His claim: 8.6 µs added per 4 KB chunk, 0.086% of one core at 100 chunks/sec.

I measured it wrong first and it is worth recording why. My first probe measured 44.2 µs
in the **dev** profile. Re-running the identical probe in **release** — the profile the
shipped binary uses — gives **8.47 µs**, against his 8.6 µs. So his number is right and
my first measurement was the wrong profile, not a wrong claim. His CPU arithmetic also
checks out exactly: 8.6 µs × 100/sec = 0.086% of a core; × 1000/sec = 0.86%.

I also re-ran his optimisation finding, because a 5× reduction that large is the kind of
claim worth confirming rather than quoting: `visible_text()` was replaced by a byte-level
`LineScan`. The current code scans raw bytes for the three UTF-8 sequences it cares about
(`E2 A0..A3 80..BF` for braille, `E2 94 80` for the rule, the two arrows) and skips
CSI/OSC runs without allocating. No `#[allow]` on an allocation. His description of the
fix is accurate.

## F1: phantom turns after a short turn (bounded, needs a follow-up)

This is my own probe, not from his note, and none of his 8 tests catch it.

**Mechanism.** `feed()` prepends `self.carry` to the chunk, applies every *complete* line
in the buffer, and then keeps the **last `MAX_CARRY` (512) bytes of the whole buffer** as
the new carry:

```rust
let keep = buf.len().min(MAX_CARRY);
self.carry = buf[buf.len() - keep..].to_vec();
```

`MAX_CARRY` is documented as "how many bytes of a straddling marker to keep". But the
window is taken from the end of `buf` regardless of where the applied lines ended, so
when a read's complete lines are shorter than 512 bytes the carry retains **lines that
were already applied**. On the next `feed()` those lines are re-applied.

**Reproduction** (probe against `3b52ea2`, reverted afterwards, tree verified byte-identical):

```
spinner read        -> START=1 END=0      (correct)
status read         -> START=0 END=1      (correct)
10 ordinary reads   -> phantom START=10, phantom END=10
500 ordinary reads after ONE short turn -> phantom START=66, phantom END=66
```

**It self-limits but does not self-heal.** The spinner/status pair is re-applied each
read, producing one START and one END, until the carry window is pushed past them. 500
reads gave 66 phantom pairs, not 500 — the row counter growing eventually evicts them.
So this is a decaying burst, not an unbounded leak. `carry` stays at 512 (bounded), so
there is no memory growth and no second copy of the stream.

**Why it matters anyway.** A phantom START+END makes the accumulator open a turn that
never happened and close it, attributing ordinary output to a turn that never opened.
For a UI that renders "turn N", that is the interface lying about the number of turns
the model took. It is bounded in count, which is why I am not failing the delivery, but
it is in exactly the direction his own module comment warns about — "an wrong boundary
makes the UI merge two answers into one, and the terminal looks completely normal".

**Why his tests miss it.** His `a_turn_is_detected_from_the_spinner_line` /
`a_turn_ends_when_the_status_line_replaces_the_spinner` tests feed the spinner and status
in one `feed()` call, so the carry never gets a chance to re-apply them across reads.
Every test that crosses a read boundary (`a_marker_split_across_reads_is_still_matched`)
tests the straddle case, not the re-application case. There is no test asserting that a
line already applied is not applied twice.

**Suggested fix, for the owner to judge:** carry only the bytes after the last newline
(`buf[boundary..]`), which is what the comment already says the carry is for. I did not
make this change — it is Nimbus's code, it is a behaviour change to a verified commit,
and it should land as its own dispatch with its own test.

## F2: the tmux-repaint finding is confirmed

He reports that in the third turn all three markers were visible, and concludes this is
tmux redrawing the whole pane each frame rather than a separation failure. I confirmed
the underlying fact independently: three `echo TURN_x` prompts into a real 80×24 pane
left **12** `TURN_` occurrences on screen, because each redraw repaints the pane history.
So the byte stream genuinely cannot attribute a string to a turn, and his decision to
have the accumulator split only on boundaries and preserve the text verbatim is the right
one. He documented the consequence for Mira in the store's comments so she does not
rediscover it.

## The incident he reported

He stopped Aegis's pane pipe by running `tmux pipe-pane` with no arguments believing it
was a query, and restored it by re-attaching the FIFO and verifying the transcript
resumed growing (+71939 bytes over 5 seconds). Two things worth recording:

1. **He reported it himself, in the delivery, with the lesson.** `tmux pipe-pane` with no
   arguments stops the pipe; the query is `display-message -p '#{pane_pipe}'`. I verified
   the current state of the seats' panes independently and found the mirrors intact.
2. **This is the same trap I fell into in an earlier dispatch** (killing a throwaway tmux
   session while probing mirror behaviour), which is why I do not treat a self-reported
   incident of this kind as a disqualifier. The test is whether it was disclosed and
   undone, and both hold here.

## What this does not prove

- **I did not run three prompts through the real saih panel myself.** His 3/3 turn
  separation is from a real-pane capture. My probes use synthetic spinner/status lines
  transcribed from the shapes his note documents. The detection logic is verified; the
  real-pane 3/3 figure is his, not mine.
- **The UI store's 11 tests verify the accumulator, not the wiring to a real event
  stream.** `onSessionTurn()` exists in `api.ts`; nothing drives it in this commit. That
  is his stated design (it is Mira's foundation to build on), and it is fine, but it
  means "3 TurnStarted + 3 TurnEnded" is a detector-level result and not yet an
  end-to-end one.
- **No persistence, by design and correctly so.** The store is in-memory only, with a
  `whereIsItStored` constant that says so explicitly. His reasoning — writing to the
  ledger would create events nobody captured — is the same red line as the
  session-work-bridge contract. The consequence is stated in the delivery too: closing
  the app empties the conversation view.

## Live-machine hygiene

I created one detached worktree at `3b52ea2` and one at `7e0b60c`, both removed
afterwards and both confirmed clean (`git status --short` empty in Nimbus's worktree, no
mutations left behind — every mutation was reverted from a `/tmp` copy and the tree
verified byte-identical). I ran one throwaway tmux session `fluxprobe2` with a FIFO to
check the repaint behaviour; I killed it and removed `/tmp/flux-fifo` afterwards. No
live seat's pane pipe was touched, and no live database was written.

## Files

No files of mine changed for this request; the verdict and this note are on
`packet/flux/intent-verdict`.
