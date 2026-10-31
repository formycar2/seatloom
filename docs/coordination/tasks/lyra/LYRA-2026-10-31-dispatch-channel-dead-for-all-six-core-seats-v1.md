# Standing-duty verification: the dispatch channel is dead for all six core seats, proved in both directions on a scratch database, with the target verified twice before every write

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-dispatch-channel-dead-for-all-six-core-seats-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, blocking, finding |

## Why this round exists

`fleet-status.sh` has reported the same red line for three rounds: **only 1 of the six core seats is
registered in `seats`**, and `dispatches.target_seat_id` has an FK to `seats(id)`. Aegis has written it,
his heartbeat repeats it, and the operator's fix command is written and waiting.

**I have been repeating the red line without proving what it actually blocks.** "A dispatch cannot be
inserted" is an inference from a foreign key. **An inference is not a measurement, and I have spent this
whole stretch recording exactly that failure.**

So I measured it.

## What I measured, on a scratch database

I created a scratch database (name `lyra_fk_probe_<timestamp>`, dropped immediately after), applied the
23 schema files with **no seed rows**, so `seats` was empty — the same shape as live.

**Target verification before every write**, because that is the discipline the incident demands:

```
db: lyra_fk_probe_1793422901
CONFIRMED: lyra_fk_probe_1793422901 exists and is not seatloom
```

### Test 1 — dispatch to an unregistered seat, with a valid `project_id`

```
ERROR:  insert or update on table "dispatches" violates foreign key constraint "dispatches_target_seat_id_fkey"
```

### Test 2 — the control: register the seat, then the identical insert

```
SUCCEEDS with the seat registered — the FK is the only blocker
```

**Both directions. The red line is real, and it is the only thing standing between this fleet and being
dispatched to.**

## The consequence nobody has stated explicitly yet

**`seat-lyra-001` is not registered either.** Measured on live:

```
select count(*) from seats where id='seat-lyra-001';   ->   0
```

**So I cannot be dispatched to.** Not "the other five seats are stuck" — *all six, including me*. The
`enqueue_dispatch` tool I have used five times this stretch writes through `dispatch.rs:1045`, which
inserts into `dispatches` and **does not create a missing seat** (grep for `INSERT INTO seats` in the
dispatch path: no matches). The insert will fail on the FK.

**This means my one working delivery channel is gone.** For three rounds I have reported the blocker as
something affecting other seats while my own seat was equally unregistered. That is the same shape as the
`activeProjectId` measurement and the three-cards count: **I read a fact about a set and applied it to the
part of the set I happened to be thinking about.**

## What I did not do

**I did not register the seat.** The control proved that a single `INSERT INTO seats ... ON CONFLICT DO
NOTHING` would restore the channel — and I did not run it, because it writes the live database and Aegis
has explicitly reserved that action for the operator. **Three rounds ago I destroyed the live database by
taking an unsupervised infrastructure action; running this would be that decision again with a smaller
blast radius, and the blast radius was never the variable.**

**I verified I did not repeat the incident:** `check-ledger-exists.sh --quiet` exit 0, 32 tables present,
and `select datname from pg_database where datname like 'lyra_fk%' or datname='lyra_probe'` returns
empty — every scratch database I created is gone.

## The ask, unchanged and now measured

```bash
bash scripts/register-fleet-seats.sh
```

It is zero-destructive (one `INSERT ... ON CONFLICT DO NOTHING`), it restores all six core seats, and
**it is the precondition for any seat receiving work at all — including me.** Until it runs, the fleet's
standing duty is documentation and git, and nothing can be routed.
