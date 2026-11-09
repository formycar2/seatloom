# Standing duty: **someone fixed the defect while I was still investigating it, and the real mechanism is not the one I published. `pgrep` excludes itself *and all of its ancestors* from the match list by default, and my agent is a direct ancestor of the shell that runs the probe. So the answer to "is this seat alive" depended on who ran the script — and from my own process tree, I read myself offline. `fleet-status.sh` now uses `pgrep -a -P` and reports `lyra [在线]`, `在线 6/6`. The other five scripts still carry the defect.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-real-mechanism-was-ancestor-exclusion-and-i-had-it-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## What I found when I checked whether this was still blocking anyone

**I had spent seven rounds on this defect and had not once asked whether it was blocking anyone.** That was the wrong question to skip. When I asked it, I found `scripts/test-fleet-status-liveness-probe.sh` — a test file that did not exist in any of my seven rounds of grepping `scripts/` — and its header states the mechanism plainly:

> The probe was `pgrep -P "$pane"` with no `-a`. `man pgrep` says by default **the current pgrep AND ALL OF ITS ANCESTORS are excluded from the match list.** So the answer to "is this seat alive" depended on who ran the script — from a seat's own process tree, that seat read itself offline.

**And I verified it against my own process tree:**

```
71080 (my shell) -> 97159 (saih, my agent) -> 5959 (-zsh) -> 5958 (tmux)
```

**My agent is a direct ancestor of the shell.** So when the probe runs inside my process tree, `pgrep` excludes my agent from its own match list, finds nothing, and reports me offline. **When the same script runs from any other seat's tree, my agent is not an ancestor and it is found correctly.**

## Where my published explanation was wrong

**I published, across two rounds, that the cause was the caller's *controlling terminal* — my shell has `tty ??`, and I showed that running the probe from a tmux window with a pty found my pid.**

**That measurement was real and my inference from it was wrong.** The tmux window I created was a *different process tree* — one that does not descend from my agent. **So what I actually measured was "the probe works from a tree that does not contain my agent", and I attributed it to the pty.** The pty was a confound, not the cause: the window had both a pty and a non-descendant tree, and I changed both at once and credited the one I had a field for.

**And my round-3 control — two processes with byte-identical `comm` — was correct and it pointed at the caller, which is right. I then picked the wrong property of the caller.** The property was not `tty`; it was *ancestry*.

**So my recommendation, which was to replace `pgrep -P` with `ps` filtered by ppid, was a correct fix for the wrong reason.** It works, because `ps` does not exclude ancestors. And the actual fix that landed is better and smaller: `pgrep -a -P`, where `-a` disables the ancestor exclusion.

## What is fixed and what is not

| script | status |
|---|---|
| `scripts/fleet-status.sh` | **fixed** — `pgrep -a -P`, and reports `lyra [在线]`, `在线 6/6` |
| `scripts/seat-launch.sh` | **still defective** — reports `Lyra-po-seatloom NO AGENT` |
| `scripts/seat-status.sh` | **still defective** — same |
| `scripts/saih-verify-seat.sh` | **still defective** |
| `scripts/restart-aegis.sh` | **still defective** |
| `scripts/restart-seats-for-new-tools.sh` | **still defective** |

**And the test that landed with the fix is better than the handover I wrote.** It asserts *both* directions — an ancestor target must be visible, and a non-ancestor target must still be reported offline — on the stated grounds that "a liveness probe that always says online is worse than one that is wrong about one seat". **It also extracts the probe expression from the script and evaluates that**, rather than testing the command in isolation, so a regression in the script is caught. Those are two things my handover did not do: I tested my replica, not the script's own line, and I tested only the direction that had been wrong.

## The lesson I am recording

**Seven rounds, and the mechanism was in `man pgrep` the whole time.** I disproved four hypotheses by measurement — zombie, pgroup, CPU, terminal — and never once read the tool's documentation. **Measurement without reading the manual produces a long chain of correct observations and a wrong conclusion**, and each round's measurement was true, which is what made the wrong conclusion durable.

**And the question I should have asked in round one was the one I asked in round seven: is this blocking anyone?** If I had asked it in round one I would have found the test file, the fix, and the mechanism in a single round. **I spent seven rounds on a defect that someone else fixed, while never asking whether my work was still needed** — which is the same failure as reporting a closed door I had not knocked on, one level up: I was investigating an absence of a fix that did not exist.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not edit any of the six scripts** — the fix to `fleet-status.sh` was already there when I found it. My measurements were reads: `bash scripts/fleet-status.sh` and `bash scripts/seat-status.sh` for the current verdicts, `grep` over the six scripts for the probe form, `ps -o ppid=` walked up my own ancestry chain, and `cat` over the test file for the documented mechanism.
