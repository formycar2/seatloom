| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-0400-the-database-was-killed-and-restarted |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, incident, database, recovery, false-alarm |

# The database was OOM-killed four hours ago. I restarted it, and nothing was lost

## What I found when I woke up

The first thing the protocol asks is whether the ledger still exists. I could not answer it:
`podman` hung, even `podman version`. The container was **not** gone — it had been killed.

```
State=exited  ExitCode=137  OOMKilled=true
FinishedAt=2026-11-07 22:24:44 +0800   RestartPolicy=no
```

Exit code 137 is SIGKILL, and podman reports the cause outright: the VM ran out of memory. The
VM is capped at 2 GB (`vfkit --memory 2048`), and its kernel log was OOM-killing `podman`
processes continuously — 60 kills in the last 60 lines, with the log still being written as I
read it.

**The reason it stayed down is the restart policy.** `RestartPolicy=no` means that when the
container dies, nothing brings it back. It had been dead for about four hours.

## What I did, and why it was mine to do

The data is in a named volume (`seatloom-pgdata`) on the VM's disk, which is intact — the data
was never at risk. Restarting a service is infrastructure truth, which is my responsibility, so
I started it.

I verified with the protocol's own query rather than with podman's opinion:

```
tables=35, memories=63, dispatches=80, cards=5
```

63 memories and 80 dispatches — **more** than the 62 and 77 recorded before the crash, because
the seats kept working and their writes are here. Nothing was lost.

## Two false alarms I fixed while I was in here

**1. `fleet-status.sh` was telling you the two pending cards cannot be clicked. They can.**

It was applying the criterion you overturned on 2026-11-05 03:43:40 UTC — "the branch must
contain the base" — and printing a second falsehood: a line claiming the seat owner had
already been dispatched to merge track. No such dispatch was ever sent. I measured the same
branches with `check-cards-clickable.sh`, which was corrected on 11-05 to ask your ruling's
question: both merge clean, 0 files deleted, 0 mainline commits dropped. So the two scripts in
one repository gave opposite answers, and the one you look at was the wrong one.

The accept path confirms it: it runs `git merge --no-ff`, which cannot drop commits, and
`branch_contains` has exactly one caller in the whole codebase — in `request_acceptance`, at
card creation. Containment is where you said it is.

Now it asks whether the merge conflicts or deletes files, and being behind is printed as a
**note** rather than a refusal, which is what the ruling says.

**2. The self-wake reading.** The log's last line was 00:00 while the file's mtime was later —
modified without a new line. Combined with the 01:00, 02:00 and 03:00 runs missing entirely,
this is the same external interference as last hour (both `.log` files deleted, the `.txt`
files surviving), not the waker failing. Running it by hand pokes the pane with exit 0.

I have not tried to fix it, because I do not know what removes `/tmp/*.log`, and last hour's
lesson applies: reporting a problem that is not there pushes you to break what works.

## Measured state

- Ledger: 35 tables, 63 memories, 80 dispatches, 5 cards — all present.
- All six checks exit 0.
- Both pending cards report as **clickable**, with the behind count shown as a note.
- The 6 pass-verdict-without-card dispatches are all the sandbox and identity subjects already
  held back; each would ask you to accept work you cannot run while the sandbox is a fake.
- Sandbox: still the 95-byte fake from 20:09, no backup, fingerprint
  `90fcd5ecd2107bd7782aec850158fd89`.
- Main application pid 90805, never written.

## Needs you

- **The VM has 2 GB and keeps getting OOM-killed.** The container now has `RestartPolicy=no`, so
  the next OOM kill means another four hours of silence unless someone notices. Raising the VM's
  memory, or setting a restart policy, is a change to your machine that I will not make on my
  own. If you tell me to, I can write it as a repeatable script.
- **The sandbox still needs your authorisation** to rebuild. It has now been a fake for two days.

## Not done

- No card, no merge, no install, no restart of the application, no live-database write.
- I did not change the VM's memory or the container's restart policy.
- I did not find what deletes `/tmp/*.log`.
