| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-0200-lyras-pgrep-mechanism-is-ancestry-not-the-terminal |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, correction, pgrep, fleet-status, instrument-defect, lyra |

# Lyra's `pgrep` mechanism is ancestry, not the terminal — and her first explanation was the right fix for the wrong reason

## Why I am writing this

Lyra has four committed documents on track investigating why `fleet-status.sh` reports her offline. Each
corrects the last, and the newest — `LYRA-2026-11-10-the-caller-had-no-terminal-and-the-probe-never-saw-me-v1.md`
— concludes:

> `pgrep -P <pane>` fails when the *caller* has no controlling terminal.

**I measured that and it is false.** I am correcting it with a new document rather than editing hers,
because her documents are committed facts about what she observed, and rewriting a seat's record is not
mine to do.

## What I measured

My own context has **no controlling terminal** — `tty` returns `not a tty` — and `pgrep -P` works fine
from it:

```
tty -> not a tty
pgrep -P 5959 -> 97159          (Lyra's agent; found, from my no-tty shell)
fleet-status.sh -> 6/6 online   (run from the same no-tty context)
```

So the terminal is not the variable. The variable is **ancestry**, and `man pgrep` states it outright:

> **`-a` Include process ancestors in the match list. By default, the current pgrep or pkill process and
> all of its ancestors are excluded (unless `-v` is used).**

## The controlled comparison

One target that is the caller's ancestor, one that is not. Everything else held constant:

| target | relation to caller | `pgrep -P <pane>` | `pgrep -P <pane> -a` |
|---|---|---|---|
| `8518` | **my ancestor** (it is `ppid` of my own shell) | **invisible** | **visible** |
| `97159` | not my ancestor | visible | visible |

Two processes with the same `comm` (`saih`), same depth, same parent type — the exact pair Lyra built in
round 3 — differing in **one** property: whether the caller descends from them. That property decides
the answer. A tty has nothing to do with it.

And the same target through every probe form: `-x`, `-P`, `-f`, `-l`, with a pattern, without — **all
blind to the ancestor**, all sighted with `-a`. `ps -axo comm=` sees it either way, which is why `ps` and
`pgrep` disagree by exactly one pid.

## What this means for the four rounds

| round | published | measured verdict |
|---|---|---|
| 1 | "`-a` is a flag `pgrep` silently ignores; adding it makes the probe find me" | **right fix, right outcome, wrong reason.** `-a` is documented and it does work |
| 2 | "`pgrep` has no `-a`; it is consumed as a pattern" | **wrong** — the man page documents `-a`, and `pgrep -a` exits 0 |
| 3 | "two identical-`comm` processes, one found and one not" | **correct observation, no mechanism** |
| 4 | "the caller has no controlling terminal" | **wrong** — measured above |

**The cost of round 2 is the part worth naming.** Round 2 discarded the correct fix on the grounds that
the flag did not exist. That correction was more wrong than the thing it corrected, and it sent the
investigation through two more rounds to arrive at a mechanism that is also false. **A confident
correction of a half-understood observation is how a four-round loop becomes an eight-round one.**

## The actual defect, and its one-flag fix

`scripts/fleet-status.sh:95` probes liveness with `pgrep -P "$pane"` with no `-a`. So the answer depends
on **who runs the script**:

- from the operator's shell — no seat agent is an ancestor — every seat is reported correctly
- from inside a seat's own process tree — **that seat reports itself offline**

Which is exactly what Lyra saw: she ran the script from her own pane, so her own agent was excluded as an
ancestor, and she read "offline" about herself.

**The fix is `-a`**, and it is one flag, and it is documented. Lyra proposed it in round 1 and talked
herself out of it in round 2.

One thing this also explains: `fleet-status.sh` **hardcodes aegis as online**, printing the literal line
`aegis [online] ... that is me, you are talking to me right now`.
That hardcode was a workaround for this same defect — aegis is the seat most likely to run the script from
inside its own tree — installed without anyone naming the cause. It has been masking the symptom for one
seat while the instrument stayed broken for the other five.

## What I did and did not do

I did not change `fleet-status.sh`. It is a script the operator reads to decide whether the fleet is
healthy, and a change to a liveness instrument should be verified in both directions before it is
trusted — including the case that must still report offline, because a probe that always says "online"
is worse than one that is wrong about one seat. That is a dispatch, not a heartbeat edit.

I also did not touch Lyra's four documents. They are her record, and the correction belongs beside them,
not over them.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0, plus `check-unclaimed-work` at 0.
- `fleet-status.sh` from my context reports **6/6** — consistent with the ancestry rule, since no seat's
  agent is my ancestor.
- Three pending cards awaiting you.
- Push: local and remote both at `3599978`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **The `fleet-status.sh` liveness probe is context-dependent** and `-a` fixes it. I have not applied it.
- **Three cards are waiting**, each separate work. Mira's is missing one statement you should read first.
- **Option 1, 2 or 3 for the live-seats count** — my recommendation is 1, unchanged.
- **The sandbox decision unblocks five dispatches at once**, not one.
