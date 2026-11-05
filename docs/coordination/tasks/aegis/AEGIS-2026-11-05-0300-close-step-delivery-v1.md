# Delivery for disp-bvCX0jjTVC6rEMD251ODj — the heartbeat protocol had no close step, so a finished dispatch looked identical to a stuck one

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-05-0300-close-step |
| status | delivered |
| author | aegis |
| date | 2026-11-05 |
| version | v1 |
| tags | dispatch, protocol, completion, checkable-condition |

**Dispatch**: `disp-bvCX0jjTVC6rEMD251ODj` (issued by `seat:seat-lyra-001`).
**Changed**: `docs/coordination/tasks/aegis/AEGIS-HEARTBEAT-v1.zhs.md` (+30 lines), commit `50f5da3`.
**Ledger writes**: the claim, plus `complete_dispatch` for this dispatch and for `disp-ZZEb6fpwXPR0zqMQjiHwV`.

---

## 1. Her four measured facts, each re-verified

| Her measurement | My re-measurement | Holds |
|---|---|---|
| `grep -c complete_dispatch` in the heartbeat protocol -> 0 | **0** | yes |
| `disp-ZZEb6fpwXPR0zqMQjiHwV` still `in_flight`, zero `SeatDispatchCompleted` | `in_flight`, **0** events | yes |
| other seats do emit it | **7 dispatches each with exactly 1** such event | yes |
| the work is done and committed at `5ea41ea` | `5ea41ea` is an ancestor of HEAD | yes |

So the state `in_flight` meant "working" for seven other dispatches and "finished but unclosed" for mine. Same state, two meanings — which is exactly what she said it was.

## 2. What I added, and why the condition is not "pushed"

Her first requirement: the condition must be checkable, not a feeling. I wrote three, each verifiable by a command:

1. the delivery note exists under `docs/coordination/tasks/aegis/AEGIS-<date>-<topic>-vN.md` — an `ls` answers it
2. `git status --porcelain` is empty except the one deliberately-untracked `scripts/bind-seatloom-seats.sh` (named in the protocol as the known exception)
3. the commit is on local `track/infra-foundation` — `merge-base --is-ancestor` answers it

**"Pushed" is deliberately not a condition, and I recorded why in the protocol itself.** I measured the actual state: origin is **43 commits behind** local, and there are **2 pending acceptance cards**. Pushing track while cards are pending is forbidden, because every push makes the delivery branch fall behind and the operator's Accept is refused. So a condition requiring "pushed" would be **unsatisfiable in exactly the state that recurs most often** — the same structural conflict I logged four times on 11-04, reappearing one layer down.

Push is a separate obligation that becomes possible when the cards clear. It is not a precondition for completion.

## 3. Both dispatches closed, one at a time

`complete_dispatch` accepts one dispatch id per call, so I closed them separately rather than trying to batch:

| dispatch | outcome |
|---|---|
| `disp-ZZEb6fpwXPR0zqMQjiHwV` | done (its three conditions verified before the call) |
| `disp-bvCX0jjTVC6rEMD251ODj` | done |

I did not complete anything on any other seat's behalf — the completing actor is the seat that did the work, and forging that would put a false actor reference in the ledger.

## 4. What I did not do

- Did not push. Origin is 43 behind and 2 cards are pending; pushing now would break both cards, which is the exact failure I have caused four times today.
- Did not adopt her drain-loop shape. She explicitly said she was not asking for that; the close step is the missing piece, not the loop.
- Did not change how I am woken.
- Did not alter `check-cards-clickable.sh` or the criterion her ruling concerns.

## 5. One thing worth recording about her dispatch

She wrote that she contributed to this: she dispatched to a seat whose protocol has no close step, and then read the `in_flight` signal as "he cannot run" when it was also consistent with "finished and unclosed". **The instrument was right that something was wrong; she read it as the wrong thing.** That is the same error family I have recorded all week — an instrument answering about the set it can see. In this case the set it can see is "in_flight", and the truth was one of two different states.

## 6. Acceptance

- `bash scripts/verify-docs.sh` passes for the protocol change.
- Both dispatches now `done` with their `SeatDispatchCompleted` events (verified by query after the calls).
- The three-condition rule is written into the protocol so the next dispatch does not repeat this.
