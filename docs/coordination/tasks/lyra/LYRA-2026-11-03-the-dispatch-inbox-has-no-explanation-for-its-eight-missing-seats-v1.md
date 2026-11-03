# Standing-duty finding: the seat list in the dispatch inbox is not empty, so the mechanism that explains why it is empty cannot fire — the operator sees 1 of 9 seats with no explanation, on the one screen where he dispatches work

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-the-dispatch-inbox-has-no-explanation-for-its-eight-missing-seats-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, product, finding, measured |

## What I measured, after ~15 hours with no commits

No new commits on track since 20:08 yesterday. All three blocked items re-measured and unchanged: bindings 1,
`seat-aegis-001` 0 rows, the bind script still `??`, the gate still 1 violation, and GitLab's track still 830
→ 831 commits behind. **Re-reporting those a fourth time would be the noise Aegis correctly declined to
produce**, so I went to my heartbeat's step 3.1 — find where the product is worse than it claims — and traced
the blocked item to its end.

## The finding

**The dispatch inbox's seat list is project-scoped, and the scope is enforced correctly, and the result is a
list of one with nothing explaining the other eight.**

The path, traced through three layers:

```
ui/src/views/DispatchInboxView.tsx:665   api.listSeats(activeProjectId)
ui/src/lib/api.ts:88                     invoke('cmd_list_seats', { projectId })
src-tauri/src/commands/seat_cmds.rs:9    require_project_id(...) then list_seats_for_project(pid)
crates/seatloom-core/src/db/repositories.rs:219
    FROM seats s JOIN project_role_bindings b ON b.seat_id = s.id WHERE b.project_id = $1
```

**Measured against the live database, running the exact query the dispatch view runs:**

```
seat-vela-001 | vela | active        <- one row, out of 9 registered seats
```

**The scoping is deliberate and documented in two places** — `DispatchInboxView.tsx:651` says *"a seat bound in
another project is correctly absent here rather than silently present"*, and `useDataStore.ts:1930` says
*"seats not bound to THIS project are contextual, not members of it."* **The backend is right. The view is
right. The table is one row deep.**

## Why this is a finding and not the same blocked item

**I already reported "the UI shows one seat of eight." This is the part I had not traced: the product already
contains the mechanism that prevents exactly this confusion, and it is wired to a different screen.**

`listAllSeats` exists for one stated purpose — `api.ts:90-92`: *"naming the seats that are bound in no project
at all, so an empty seat list can say why it is empty instead of rendering as broken."* The store computes it
(`useDataStore.ts:1933-1937`), keeps it separate from the scoped list, and exposes it as `unboundSeats`.

**And exactly one view consumes it** — `ui/src/views/SeatsView.tsx:67-73`:

> "There really are {n} seats in the system that are not bound to this project: {names}. They are not idle -
> they are simply not bound to this project. To make them appear here, they must first be given a binding."

**`DispatchInboxView` reads `unboundSeats` zero times.** So on the screen where the operator dispatches work,
the seat picker shows one seat and says nothing about the other eight. **On the screen where he browses seats,
it explains them.** The explanation exists, is written, is translated, is tested elsewhere — and is absent from
the one view where the missing seats cost him something.

## The specific trap, and why it is not caught by the design

**The explanation mechanism is conditional on the list being empty.** `SeatsView` renders it only when
`unboundSeats.length > 0`, and the store only sets the field at all when `unboundSeats.length > 0`
(`useDataStore.ts:2021`). **But the dispatch inbox's list is not empty — it has one seat.** So:

- A seat list of **0** → the mechanism fires, the operator is told why.
- A seat list of **1** → the mechanism is silent, and 1 reads as "the whole team."

**The design guards the case where the operator sees nothing. It does not guard the case where he sees almost
nothing, and the second is worse: an empty list is visibly wrong, while a one-item list is not.** The dispatch
inbox is the surface where the difference matters most, because a dispatch to the wrong seat is a real
consequence and a seat that cannot be dispatched to is invisible work.

## What would close it

**One conditional in `DispatchInboxView`, mirroring `SeatsView`'s:** if the scoped list is smaller than the
registry, say so — "N registered seats are not bound to this project; dispatch to them will not resolve until
they are bound." **The data is already in the store; the view just does not read it.** That is a UI change in
Mira's lane and I am not making it.

**The immediate unblock is still the operator's**: `bind-seatloom-seats.sh --apply`, once the script is
committed to track. **I am recording both, because the second does not remove the first** — even with all nine
seats bound, any future seat added without a binding will hit the same silent state.

## What I did not do

I did not edit any view, store, or command. I did not commit the bind script. I did not push to GitLab. I did
not add a header to the budget document. My measurements were the three-layer source trace, `grep -c` on
`unboundSeats` per view, and the live query the dispatch view runs. `seats` is 9, `canonical_events` is 1,
`check-ledger-exists.sh` is exit 0.
