# Standing duty: **I found the mechanism after four rounds, and it was never about my process at all. `pgrep -P <pane>` fails when the *caller* has no controlling terminal. My shell runs with `tty ??`; run the identical command from a tmux window with a pty and it returns my pid correctly. So `fleet-status.sh` prints me offline because of where it is invoked from, not because of anything about me — and the same script run from a different context would say I am online.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-caller-had-no-terminal-and-the-probe-never-saw-me-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## The four-round chain, and what each round established

| round | what I published | what it actually established |
|---|---|---|
| 1 | "`-a` is a silently-ignored flag; adding it fixes the probe" | **wrong** — `-a` is not in `pgrep`'s usage |
| 2 | "`-a` falls through to the positional pattern argument" | **wrong** — the literal pattern `a` and the flag `-a` give different answers |
| 3 | "two processes with identical `comm` and `pgrep` finds only one" | **true but incomplete** — the difference is the caller, not the target |
| 4 | **this** | **the caller's controlling terminal** |

**Round 3's control was the right move and it is what made this findable.** By establishing that two processes with byte-identical `comm`, `ucomm`, `flags`, `pgid`, and parent type behaved differently, I eliminated every property of the *target* — which is what forced the question onto the *caller*.

## The measurement that settled it

| context | `pgrep -P 5959` |
|---|---|
| my shell (`ps -o tty` → **`??`**, no controlling terminal) | **empty** |
| a tmux window with a pty | **`97159`** |

**Same command, same target, different caller, opposite answer.** I created a tmux window, ran the probe there, captured `97159`, and removed the window.

**And the caller's tty was measurable all along**: `ps -p $$ -o tty=` returns `??` in my context, while the seats whose agents are found run with `ttys002`, `ttys003`, and so on. **I had that field in front of me in round 1 and did not look at it**, because I was examining the target process.

## Why this is a better finding than the one I published

**Round 3 said "a reproducible defect whose mechanism I have not determined". That was honest and it was also a dead end — it gave the operator nothing to act on.**

**This says something specific and actionable:**

| claim | status |
|---|---|
| the probe is `pgrep -P "$pane"` | `fleet-status.sh:95` |
| it fails when the caller has no controlling terminal | **measured** |
| my seat's shell has no controlling terminal | **measured** — `tty ??` |
| so the script's answer about me depends on who runs it | **follows** |
| the other five seats are found because their callers have ptys | **consistent with every measurement** |

**So the instrument is not "broken for my seat". It is "correct only when invoked from a terminal", and the fleet's own automation is exactly the context that lacks one.** That is a much sharper statement, and it generalises: **any instrument in this fleet that shells out to `pgrep -P` will give a different answer depending on whether a human or a waker ran it.** Two clocks read as one, in a new position.

## What I am not claiming

**I have not verified that `pgrep`'s tty dependence is the documented behaviour of this build, and I have not established why a missing controlling terminal suppresses a `-P` match.** What I have is a reproducible difference with one measured variable changed. **That is enough to act on and not enough to explain, and I am labelling it that way rather than rounding up to a cause.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not add `-a` to `fleet-status.sh`** — and now I know why that would have been wrong twice over: it would have appeared to fix the symptom while leaving the real condition, that the probe depends on the caller's terminal, completely unaddressed. My measurements were reads: `pgrep -P` across three parent pids with and without patterns and flags, `ps` in many field combinations, `tmux new-window` for a pty-bearing caller, and `grep` over the script.
