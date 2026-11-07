| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-07-2300-a-missing-log-is-not-a-dead-waker |
| status | delivered |
| author | aegis |
| date | 2026-11-07 |
| version | v1 |
| tags | aegis, self-wake, honesty, false-alarm |

# The heartbeat protocol told me to declare the waker dead. It was not, and the log had been deleted

## What the protocol says, and what I found

The protocol is explicit: no `POKED` lines in `/tmp/seatloom_aegis_waker.log` means the
self-wake is dead, and that is more serious than any single failure, so fix it first.

The log was missing. I followed the instruction to investigate before acting, and the waker
is **not dead**:

| evidence | what it proves |
|---|---|
| system cron log | cron ran at 22:00:00, as scheduled |
| the notify script's dedup file | the 22:00 run reached the **end** of the script — it got past the tmux poke and into the notification step |
| running the installed script by hand | it poked the pane successfully, exit 0 |

So the waker ran, did its work, and then **its log was removed**. The same thing happened to
the fleet waker's log (`/tmp/seatloom_waker.log`, held open by a `tee` since Nov 2). Both
`.log` files went; the `.txt` files and subdirectories in the same batch survived.

I did not find what removed them, and I am not going to guess. What I can state is that the
cause is external to the waker, not the waker failing.

## The defect this exposed

`fleet-status.sh` treated a missing log as **proof of death**, and set the flag that asks you
to act. `need=1` and a red `❌`.

That is a false alarm, and false alarms of this kind are expensive in a specific direction:
the protocol's own remedy is "fix it first", and the fix is to reinstall the self-wake. Run
that on a healthy system and you overwrite working configuration in order to make a line turn
green. **A check that reports a problem that is not there pushes you to break what works.**

I changed the criterion from "is the file there" to "can this be proven". Three layers, each
reporting only what it can show:

| state | reported |
|---|---|
| log present and recent | running — provable |
| log present but over 75 minutes old | ❌ genuinely no records — provable |
| log absent | ⚠️ cannot prove, **plus** the two independent facts that decide it: is the cron entry there, and is the script cron runs there and executable |

The last row matters most. With both facts present the line now says the waker *is installed*,
that this hour left no record, and **explicitly not to reinstall** — with the exact command to
confirm it by hand instead. With either fact absent it says not installed, which is the real
failure and keeps the red.

## Why I did not use the pane as a substitute signal

I tried it, and it is unusable. I searched the pane scrollback for a string that should not
exist, as a control — and got **1 hit**, because the command containing that string had
itself been echoed into the pane. A channel contaminated by the act of measuring it cannot
serve as evidence. Recorded in the code comment so nobody rebuilds it.

## Proven in four directions

`scripts/test-fleet-status-selfwake.sh` runs against a stand-in log path so the real log —
the thing under test — is never moved. It is repeatable, verified twice.

| direction | result |
|---|---|
| fresh log | reports running, and does not say "cannot prove" |
| log 120 minutes old | reports ❌ — so a genuinely dead waker still alarms |
| **log absent, cron entry and script present** | reports cannot prove, does **not** declare death, and says not to reinstall |
| **log absent and cron entry absent** | reports not installed — two different failures do not collapse into one |

## Measured state

- Ledger: 35 public tables. Six seats online.
- All six checks return their real exit code 0.
- Main application pid 90805, never written. Sandbox still the 95-byte fake from 20:09,
  fingerprint `90fcd5ecd2107bd7782aec850158fd89`, still needs your authorisation to rebuild.
- The two pending cards are clickable; the commits I pushed this hour did not break them
  (merge clean, 0 files deleted, 0 mainline commits dropped).

## Not done

- No card, no merge, no install, no restart, no live-database write.
- I did not find what deletes `/tmp/*.log`, and I have not tried to prevent it. If you know
  of a cleanup tool on this machine, that is the answer; otherwise the honest position is
  that the signal is unreliable, not that the waker is broken.
- The hook change from earlier this hour is in `.git/hooks/` — untracked operational state by
  design. The test script is committed; whether the hook itself should be tracked is your call.
