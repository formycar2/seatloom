# Standing-duty finding: acceptance card `acc-disp-bc20UzOznJQ8hCgbHw876` tells the operator to expect 10 seats, and the app will show 7

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-seat-count-card-cannot-pass-v1 |
| status | open |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | standing-duty, acceptance, blocking, card-defect |

## The defect

Aegis sent the operator a ranked list of the seven pending acceptance cards at 00:01, so the operator
could clear a seven-hour backlog. Item 7 is:

> `acc-disp-bc20UzOznJQ8hCgbHw876` — open the seat list: it should show **10 seats, not 6**, and opening
> vela should show its corpus-washing capability tags.

**The app will show 7, not 10, in the seat list under project `seatloom`.** The card cannot pass as
written, and the operator following it literally will either reject correct work or conclude the product
is broken.

## What I measured

The seats tab reads the **project-scoped** list — `useDataStore.ts:1908` calls
`api.listSeats(projectId)`, which is `cmd_list_seats`, a JOIN through `project_role_bindings`. It is not
`list_all_seats`.

```
seats bound to project 'seatloom'     7
seats bound to any other project      3   (seat-ql-001 → project_delivery/QL,
                                            seat-networkorchestrator-001 → project_delivery/networkOrchestrator,
                                            seat-stepaicloud-001 → project_delivery/stepaicloud)
unbound seats (SeatsView renders these separately)   0
                                        --
what the seat list shows under 'seatloom'   7
total rows in `seats`                     10
```

So the card's step 2 — *"count the seats: should be 10 — aegis / flux / lyra / mira / nimbus / onyx /
vela / ql / networkorchestrator / stepaicloud"* — names four seats that are **not bound to the
`seatloom` project** and therefore do not appear in the list it tells him to look at.

## The card contradicts itself, which is how I know it is a card defect and not a product defect

Its own step 4 says: *"switch to project `ql_delivery` and look at the ql seat: it hangs under
ql_delivery, its role is custom(worker), not a formal SeatLoom seat."* **That step is only coherent if
`ql` is not in the `seatloom` project** — which is exactly why step 2 cannot also show 10.

And its step 5 offers a read-only confirmation:

```
podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc \
  "select count(*) from seats;"
```

with expected result 10. That command returns **10** — it counts the table, not the list. So the card
gives the operator one instruction that yields 7 and one that yields 10, and calls both correct.

## A second defect in the same card

Step 4 names the project `ql_delivery`. **No such project exists.** The real id is
`project_delivery/QL`:

```
$ select id,name from projects where id like '%ql%' or name like '%ql%';
(no rows)
$ select project_id, role from project_role_bindings where seat_id='seat-ql-001';
project_delivery/QL|custom(worker)
```

The role in the card is right; the project id is wrong. If the operator goes looking for `ql_delivery` in
the project switcher he will not find it, and will reasonably conclude the card is broken.

## Why this is worth a round

This is the same failure that has now cost three cards and a Mira re-do: **a card whose verification step
cannot produce the stated result.** Aegis's own `3d1f6b3` landed `check-card-verifications.sh` to screen
for exactly this — *"one of them could not fail"* — and this card passes that screen, because its steps
name a real surface and a real command. What the screen cannot catch is a step whose **expected value is
wrong**. That needs a measurement, and the measurement is one query.

**The specific number is the whole problem.** "Should be 10" is a claim about a table. The seat list is
a claim about a project. The card conflates them, and the conflation is invisible until someone counts.

## What should change

The card's step 2 should read, in the project-scoped terms the surface actually has:

> count the seats under project `seatloom`: should be **7** — aegis / flux / lyra / mira / nimbus /
> onyx / vela. The three external seats (`ql`, `networkorchestrator`, `stepaicloud`) belong to their own
> projects and appear only when you switch to those; the database holds 10 in total, which is what the
> optional command in step 5 counts.

And step 4's project id should be `project_delivery/QL`.

**Whether to cancel-and-reopen this card is not my call** — the work it verifies (the 10-seat seed
correction Onyx landed) is real and delivered, and `acceptances` rows are immutable once decided. I am
recording the defect so it is not discovered by the operator at the moment he is trying to accept
correct work.

## What I did not do

I did not edit the card. I did not cancel or re-open it. I did not touch `useDataStore.ts` — the
project-scoped read is deliberate (its own comment records why: seats are a global registry and the
project association is the binding row). The card is what is wrong, not the read.
