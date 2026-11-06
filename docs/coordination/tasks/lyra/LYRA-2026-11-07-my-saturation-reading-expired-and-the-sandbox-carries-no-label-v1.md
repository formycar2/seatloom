# Standing duty: Nimbus completed my dispatch 34 minutes after I published a document saying he was working and not stalled — and my "saturation" reading expired within the same hour. **The fleet compacted: every seat dropped from 99%+ to between 10% and 43%.** What survives from this round is the one measured fact that outlives it: the running sandbox binary carries no build-commit label at all.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-my-saturation-reading-expired-and-the-sandbox-carries-no-label-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, standing-duty, correction |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 293 → 304 events, and my dispatch `disp-LMgZ46PyUy9R8yrbThu13` is now `done`.** `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## The correction I owe on my own last document

**Last round I published that the fleet was saturated — Flux 99.8%, Mira 99.4%, Aegis 99.9% — and recorded it as an operator fact about context limits. Measured now, in the same hour:**

| seat | last round | now |
|---|---|---|
| Nimbus | 40.8% | **10.3%** |
| Flux | 99.8% | **43.2%** |
| Mira | 99.4% | **42.4%** |
| Aegis | 99.9% | **22.1%** |

**Every seat compacted.** The saturation was real when I read it and it did not persist, which means what I published as a standing condition was a snapshot with a short life. **A context window near its limit is a transient state, not a property of the fleet, and I presented it as the latter.** The honest label for last round's document is "correct at 01:56, expired by 02:26" — and the part that still holds is the method error, not the measurement.

**Nimbus's own completion is the other half of the correction.** He finished at 17:58:38, which is the minute I published. So "working on it, not stalled" was right, and the 32.4-minute `in_flight` that nearly triggered a stall report was a seat in a turn. **Two rounds in a row, the ledger's silence was a seat working.**

## What Nimbus actually delivered, and the part I could not find

**He ran the frontend suite** — `ui/node_modules/.vite/vitest/results.json` is the only file touched in his worktree in the window, which is the one thing I explicitly asked him to confirm and could not verify myself, because my clone had no `node_modules`.

**But I could not find his answer.** There is no `disp-LMgZ46PyUy9R8yrbThu13-delivery.md`, no commit on either branch I asked about — `work-list-snapshot-actors` is still `310c80a8f486` and `git-status-honesty` is still `4ae18ecd8347` — and the `SeatDispatchCompleted` event carries only the dispatch id, no payload. **His session scrollback contains fragments of other dispatches' work, and I found no text answering either of my two questions.**

**So the honest statement is: my dispatch is closed, its confirmation was not delivered to me in any form I can read, and I am recording that rather than assuming the scopes are confirmed.** A completion is a lifecycle event; a confirmation is an artifact. **The dispatch closed without one, which is the same gap I recorded on 11-05 — a lifecycle with no state for finished-and-unsaid.**

## The measured fact that outlives the round

**Nimbus flagged it in his scrollback and I verified it directly: the installed sandbox binary carries no `seatloom-build-commit` label at all.**

```
strings /Applications/SeatLoom-Sandbox.app/Contents/MacOS/seatloom-sandbox | grep -c seatloom-build-commit
  → 0
binary mtime: 11-06 05:33
```

**And `5af0e16` — the branch I verified last round — emits `seatloom-build-commit=5af0e164d5a1` as a literal.** So the running sandbox predates the label entirely. **This is the same finding I published on 11-06 in a different position: the binary cannot state what it is.** Last time I proved it by finding only the environment variable *names* and no hash; this time the proof is that the label the fix introduces is absent.

**Which closes a loop worth naming.** On 11-06 I reported the missing capability, Nimbus built it, Flux verdicted it, I verified it in a linked worktree in four states, and I recorded that the rendering layer was unverified on that branch. **None of that reached the running application.** The capability is real in source and absent in the artifact an operator would click Accept against.

## What I did not do

I did not modify any file, emit a verdict, open or cancel a card, merge, push anything but this document, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the dispatch's completion and the fleet's events, `git log` and `git rev-parse` on the two branches I asked about, `find` for files touched in Nimbus's worktree, `tmux capture-pane` for every seat's live context, and `strings` plus `stat` on the installed sandbox binary.
