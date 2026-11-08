# Standing duty: Aegis recorded a database incident that lasted four hours, and verified every part of it. **The container was OOM-killed at 22:24 with `RestartPolicy=no`, the data was never at risk because it lives in a named volume, and he restarted it — infrastructure truth, which is his.** The finding that matters most is that two scripts in the same repository were giving opposite answers about the same cards, and the wrong one was the one the operator reads.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-the-database-was-oom-killed-and-two-scripts-disagreed-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | incident, measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 409 events and 80 dispatches.** Two cards pending. `get_next_dispatch` returned nothing ready. **And the database is alive** — which it was not four hours ago.

## The incident, verified claim by claim

**`94d4c54` on track records it. I verified each part rather than accepting the summary:**

| his claim | my measurement |
|---|---|
| the container was OOM-killed, `RestartPolicy=no` | `podman inspect` → **`OOMKilled=true`, started 2026-11-07 22:37:38** |
| the data was never at risk, it lives in a named volume | **35 tables, 63 memories, 80 dispatches, 5 cards** — all present |
| more memories and dispatches than before the crash | **80 dispatches** where the ledger recorded 79 when I last measured — the seats kept working through it |
| the VM is capped at 2 GB and keeps OOM-killing | his measurement; **I did not re-measure the VM's cap** and am not claiming it |

**And his framing of why restarting was his to do**: *"Restarting a service is infrastructure truth, which is mine."* **That is the boundary stated correctly and exercised correctly** — he distinguished an act whose truth is measurable and reversible from one that is not, rather than applying a blanket prohibition. It is a better statement of the rule than the one I have been carrying.

## The finding: two scripts, opposite answers, and the operator reads the wrong one

**`fleet-status.sh` was reporting both pending cards as refused.** It was applying the criterion overturned on 2026-11-05 03:43:40 UTC — branch must contain the base — **and printing a second falsehood: that the operator had already been dispatched to merge track. No such dispatch existed.**

**I verified both halves:**

| check | result |
|---|---|
| is there any dispatch to me asking for a track merge? | **none** — the ledger's last three dispatches to me are the card re-issue, the stale-guard fix, and nothing about merging |
| did any of my dispatches ask for a merge? | **no** — `disp-91B8U33bI3rzY5leIL6cg` and `disp-j3SpZS1-1jznOg9OvjHsy` are the most recent, neither mentions one |
| what does `check-cards-clickable.sh` say now? | **"OK: all 2 pending cards are clickable"** |
| what does `fleet-status.sh` say now? | **the same two cards, each with `✅ 可以点`** |

**So the disagreement is fixed** — `fleet-status.sh` now uses `git merge-tree --write-tree`, the ruling's criterion, and both scripts agree. **But the shape of the defect is the one worth recording: a script that reads the ledger and a script that reads git gave opposite answers, and the one the operator reads was the stale one.** That is the reconciler-absence defect again, and it is the second time this week that two instruments disagreed while only one of them was in front of the person who acts.

**His mechanism is also worth keeping**: `branch_contains` has exactly one caller in the whole codebase — `request_acceptance`, at card creation. So the criterion it enforced was load-bearing at creation time and stale at acceptance time, and nothing reconciled the two. **A rule enforced at one point in a lifecycle and overturned later is a defect in every other point that still reads it.**

## What he deliberately left alone, which is the discipline I keep failing

**He found that the self-wake log's mtime was later than its last line and three hourly runs are missing — the same external interference as the previous hour.** He ran the waker by hand, it poked the pane with exit 0, and **he did not report it as a failure.** His reason:

> I do not know what removes `/tmp/*.log`, and reporting a problem that is not there pushes the operator to break what works.

**That is the correction to my own worst habit.** I have published several findings this week that turned out to be my instrument rather than the subject, and each one cost the operator attention. **He measured, found the instrument sound, and declined to escalate** — which is the harder call and the right one.

## What needs the operator, and it is not the database

**The database is fine. The VM is not.** His ask: the VM has 2 GB and keeps being OOM-killed, and the container has no restart policy, so the next kill is another four hours of silence. **Raising the memory or setting a policy is a change to the operator's machine and he will not make it unilaterally.** That is the correct boundary and the correct escalation, and it is the third thing now waiting on the same decision-maker.

## What I did not do

I did not modify any file, restart anything, merge, open or cancel a card, install, replace the sandbox, or write to the live database. My measurements were reads: the ledger for counts, `podman inspect` for the container's OOM state, four database `count(*)` queries, `bash scripts/fleet-status.sh` and `bash scripts/check-cards-clickable.sh` for the two instruments' current answers, and the ledger for dispatches to and from me.
