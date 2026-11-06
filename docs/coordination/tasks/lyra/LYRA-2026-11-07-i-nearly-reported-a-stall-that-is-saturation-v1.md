# Standing duty: I nearly reported a stalled dispatch. **The ledger showed no seat activity for 33 minutes and my dispatch to Nimbus had been `in_flight` for 32.4 — past the 30-minute rule I helped write. The reading was wrong: Nimbus is mid-turn in his session, and every other seat is at 99%+ context.** What the measurement actually showed is fleet-wide saturation, not a stall.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-i-nearly-reported-a-stall-that-is-saturation-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, standing-duty, instrument-error |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 293 events and 62 dispatches — unchanged for three consecutive rounds.** Cards are still three: two accepted, one cancelled, none pending. `get_next_dispatch` returned nothing ready.

## What I measured, and what I nearly concluded

**My dispatch `disp-LMgZ46PyUy9R8yrbThu13` (the two CLI scopes) had been `in_flight` since 17:24:20, delivered 17:24:36, and it was then 17:56 — 32.4 minutes by `extract(epoch from (now()-issued_at))/60`.** The stall rule is 30 minutes. I wrote the rule. I was ready to apply it.

**The corroborating measurement that made it look certain**: the last event in the entire ledger was `SeatDispatchDelivered` by Nimbus at 17:24:36 — my own dispatch arriving. **Nothing in the coordination state had moved in 33 minutes.** A query for any seat active in the last 40 minutes returned two rows: Nimbus at 17:24:36 and myself at 17:24:23.

**And Nimbus's worktree looked idle too**: two modified files, both with mtimes of **10-29, eight days old**, and nothing touched in the last 40 minutes. So I had three independent-looking signals all pointing at "Nimbus is stuck on my dispatch."

## What was actually true

**Nimbus is working on it right now.** His tmux pane reads:

```
── ⠦ Working ──────────────────────────────────────────────────
~/Documents/GitHub/seatloom-worktrees/nimbus (packet/nimbus/session-work-readmodel)
```

**And the other seats are not idle either — they are saturated:**

| seat | context | provider |
|---|---|---|
| Nimbus | **40.8%** / 1.0M | `sp-personal` |
| Flux | **99.8%** | `sp-team` |
| Mira | **99.4%** | `sp-team` |
| Aegis | **99.9%** | `modelproxy` gpt-6.1-sol |

**So the ledger's silence has an entirely different cause than the one I assumed.** A seat that is mid-turn emits no events, because events are emitted at completion. The three seats at 99%+ context are the ones that have been running longest without compaction, and the reason none of them completed anything in 33 minutes is that they are working on long turns, not that they are dead. **Nimbus at 40.8% is the only seat with real headroom, and he is the one with a live dispatch — which is consistent with him working it, not stuck on it.**

## The error, named

**This is the "an instrument answering about the set it can see, presented as an answer about the set that exists" family, in a new position.** The ledger is a record of *completed* acts. I read its silence as an answer about *current* activity. **The set it can see is what has been recorded; the set that exists includes turns in progress, which by construction leave no trace until they end.**

**And the stall rule itself is the instrument I misused.** Its inclusion rule is >30 minutes, and I have recorded before that a ready dispatch is seat work while the stall rule governs what gets *reported to the operator*. What I did this round was treat a 32-minute `in_flight` as the trigger, without checking the one thing that distinguishes "stuck" from "working": whether the seat is in a turn. **The rule is not wrong; my application of it was, because I applied it from the ledger alone when the session state was one command away.**

**The three signals agreeing is what made it dangerous.** Two modified files with old mtimes, no filesystem activity, and no ledger activity are three readings of the same blind spot, not three independent confirmations. **I have now been caught by the same trap in four positions this week, and the common shape is: a set of measurements that all answer about recorded state, read as an answer about live state.**

## What I did not do

**I did not report a stall, did not re-dispatch, did not nudge Nimbus, and did not write anything to any worktree.** My measurements were reads: the ledger for the dispatch's timestamps and the fleet's recent activity, the database for per-seat completion counts, Nimbus's worktree for file mtimes and `git status`, and `tmux capture-pane` for each seat's live state. That last command is the one that would have corrected me at the start.

## The one thing worth acting on

**The context saturation is real and it is not my dispatch's problem — it is the fleet's.** Flux at 99.8%, Mira at 99.4% and Aegis at 99.9% are all near their limit, and the seats that most need to run (Aegis for the gate and hook defects, Flux for verdicts) are the two most saturated. **This is an operator fact rather than a seat fact, because compaction or a restart is theirs to authorize, and I am recording it rather than acting on it.**
