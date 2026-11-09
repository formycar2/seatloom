# Standing duty: **I finished last round's open loop and my published explanation was wrong in an instructive way. I wrote that `-a` was a flag `pgrep` silently ignored and that adding it made the probe find me. It is not a flag at all — `pgrep` on this macOS has no `-a`, so it was consumed as a *pattern*, and the pattern is what changed the match source. The real distinction is that `pgrep -P <pane>` matches against the process *name* while my agent's name is not resolvable that way, and the pattern form matches against the executable *path*.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-my-explanation-of-the-offline-reading-was-wrong-and-here-is-the-real-one-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## Last round's claim, and why it was wrong

**I published: "`pgrep -P "$pane"` without `-a` returns nothing while the same command with `-a` returns my own pid", and I framed adding `-a` as a one-word fix.**

**I could not pin the mechanism and I said so, which was right. What I should not have done is imply the fix.** Adding `-a` to `fleet-status.sh` would be adding an argument that this `pgrep` does not document, whose effect I did not understand, to a script I do not own. **A change whose mechanism you cannot explain is not a fix, it is a coincidence you have observed once.**

## The real mechanism, measured this round

**First, `-a` is not a flag. `pgrep` with no arguments prints its usage, and the option list is `[-Lfilnoqvx] [-d delim] [-F pidfile] [-G gid] [-P ppid] [-U uid] [-g pgrp] [-t tty] [-u euid]`. There is no `-a`.** So `-a` falls through to the positional *pattern* argument.

**Second, the pattern changes which name is matched:**

| invocation | result |
|---|---|
| `pgrep -P 5959` | **empty** |
| `pgrep -P 5959 -a` | **97159** |
| `pgrep -P 5959 -a -l` | **`97159 node`** |
| `pgrep -P 5959 -x saih` | empty |
| `pgrep -P 5959 -f saih` | empty |
| `pgrep -P 5959 -x node` | empty |

**So with a pattern present, `pgrep` reports the process; with no pattern it does not.** And `-l` shows the process name it resolves is `node`, matching what `lsof` reports for the executable: `/opt/homebrew/Cellar/node/25.2.1/bin/node`. `ps` calls it `saih` because `ps` reports the argv/comm name, which this process has rewritten.

**Third, and this is what disproves my remaining hypothesis from last round**: I tested whether CPU or process-group leadership caused it, by finding two other `saih` processes that share both properties.

| pid | ppid | pgid | %CPU | `pgrep -P <ppid>` |
|---|---|---|---|---|
| **97159 (me)** | 5959 | 97159 | 28.1 | **empty** |
| 63422 | 6709 | 63422 | 2.9 | **63422 — found** |
| 8518 | 6704 | 6704 | 6.8 | **8518 — found** |

**So neither CPU nor pgroup leadership is the cause.** Both disproved by processes that share the property and are found correctly.

**What is left, and what I can state without over-claiming: `pgrep`'s default name resolution does not resolve my agent's process, and the pattern form does.** I have not established *why* — whether it is the rewritten comm, the executable path, or something about how this process presents itself to `sysctl`-based enumeration. **I am recording the reproducible fact and explicitly not recording a cause I have not measured.**

## The part that does not change, and is the actionable one

**The instrument is wrong regardless of mechanism.**

| check | measurement |
|---|---|
| `fleet-status.sh:95` probe | `pgrep -P "$pane" \| head -1` |
| that form on my pane | **empty** |
| therefore `alive` | not incremented |
| therefore the script prints | **`lyra [❌掉线]`, `在线 5/6`, and tells the operator to run `seat-launch.sh --all`** |
| and the truth | **I am running, at 28% CPU, answering dispatches** |

**So the operator is being told to relaunch a seat that is working, by an instrument that cannot see it.** That is the finding, and it stands independently of the mechanism.

**And the generalisation is the one I should have drawn last round**: the probe asks "does this pane have a child process whose name `pgrep` can resolve", and treats that as "is the seat alive". **It works for five seats and not for the sixth, and the sixth is the one whose agent presents itself differently.** A proxy that works for the shape it was written against is not a liveness check — and the fact that it is *my* seat it fails on is the clearest possible evidence, because I am the one seat whose liveness is not in question.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not add `-a` to `fleet-status.sh`**, which is what I implied last round would fix it, because I have now measured that I do not know what `-a` does here. My measurements were reads: `pgrep` in eight forms across four parent pids, `ps` in several field combinations, `lsof` for the executable path, and `grep` over the script.
