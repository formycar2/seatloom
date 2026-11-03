# Standing-duty verification: the blocked item is RESOLVED — the operator ran the bind script, bindings went 1 → 7, all seven core seats are now dispatchable, and the reason it got done is that he moved the finding out of his heartbeat and into a push channel with a criterion that could fail

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-the-binding-defect-is-fixed-and-the-reason-is-the-channel-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, verification, closed, delivery |

## The finding is closed, measured

**`project_role_bindings` for `seatloom` went from 1 row to 7:**

```
before:  seat-vela-001 | data_engineer                                    (1 row)
after:   seat-aegis-001, seat-flux-001, seat-lyra-001, seat-mira-001,
         seat-nimbus-001, seat-onyx-001, seat-vela-001                    (7 rows)
```

**All seven are `active`, and all seven are the core seats.** Running the dispatch view's exact query —
`FROM seats s JOIN project_role_bindings b ON b.seat_id = s.id WHERE b.project_id='seatloom'` — now returns
seven seats where it returned one.

**And `seat-aegis-001` now exists in `seats`** (created 04:09 today), so the `dispatches_target_seat_id_fkey`
foreign key that blocked every dispatch to Aegis is satisfiable. **The dispatch channel is alive for all seven
core seats**, which it has not been since the database was destroyed.

**The three seats still unbound — `seat-ql-001`, `seat-networkorchestrator-001`, `seat-stepaicloud-001` — are
external-project seats and their absence is correct.** This is the first time in this stretch that a "missing
seat" is a designed scope rather than a defect, and it is worth saying so that the number 7 is not later read
as "3 still broken."

## Why it got done, and the reason is not the finding

**The finding was measured, correct, and reported for five consecutive rounds without moving.** What changed
is the channel, and Aegis changed it himself.

`f61ac59` adds a criterion to the notification script, and the comment states the reasoning in the plainest
terms I have seen in this repository:

> Why this must go into the push channel: this item existed only in my heartbeat body for five consecutive
> rounds, and he did not run that command. A to-do that exists only in my reply has the same effect on him as a
> to-do that does not exist.

**A to-do that exists only in my reply has the same effect on him as a to-do that does not exist.** That is
the same conclusion I reached on 10-28 and have re-derived a dozen times since — a governance artifact is
delivered when the consuming process can read it — and he reached it about his own reporting.

**And the criterion he wrote can fail, which is what makes it an instrument rather than a message:**

```bash
binds="$(q "select count(*) from project_role_bindings where project_id='seatloom';" | tr -d ' ')"
if [ "${binds:-0}" -lt 7 ]; then
  pop "Only ${binds} seats visible under the SeatLoom project" "... run: bash scripts/bind-seatloom-seats.sh --apply" ...
fi
```

**A count, a threshold, and the command.** It measured 1 when he wrote it. **It now measures 7 and will stop
firing** — which is the property that makes it worth having built: it retires itself when the defect closes,
rather than becoming another standing item.

## What this changes about my own standing duty

**I reported this defect on 11-02 18:05 and re-verified it unchanged on 11-03 11:31.** Both were true. Neither
moved it. **The thing that moved it was a seat putting the finding where the operator's attention already
was, with a number attached.**

**So the lesson for my lane is not "report harder."** It is that my ROADMAP has four other asks that live in a
document the operator reads when he chooses to, and I have no instrument that fires when they are open. **The
asks that are open are: land the standing-duty docs, push to GitLab (831 commits behind), merge Mira's branch,
and add the header or grandfather the budget document.** The first three need his action; the fourth is the
choice I tested last round and found was not a real choice.

**I am not going to build a notification script** — that is Aegis's lane and he has just demonstrated he owns
it. **What I can do is state, in the ROADMAP, which asks have no instrument and which have one**, so the
operator can see the difference rather than discovering it.

## What I did not do

I did not run `bind-seatloom-seats.sh --apply` — the operator ran it. I did not edit the notify script, the
backlog, or any binding. I did not commit the still-untracked script. My measurements were the live
`project_role_bindings` and `seats` queries, the dispatch view's exact JOIN, `git show` on `f61ac59`, and a
per-core-seat binding check. `seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
