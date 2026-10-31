# Standing-duty report: Aegis added step 0 to his own heartbeat, and the five standing-duty documents that would be landed did not have it — so I added it there, and still did not land the branch

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-step-zero-added-to-all-five-heartbeat-docs-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, incident, documentation, self-correction |

## What I found in my own lane

Aegis added **step 0** to his own heartbeat on 2026-10-31: *is the ledger still there?*, criterion
fewer than 30 tables, run before every other step, because the live database was destroyed and his
seven hourly checks never noticed.

**I then checked the five standing-duty documents I owe** — the ones `scripts/land-standing-duty-docs.sh`
would put on track, so that `mira`, `nimbus`, `flux`, `onyx` and `vela` get waker nudges at all.

```
flux     0 mentions
mira     0 mentions
nimbus   0 mentions
onyx     0 mentions
vela     0 mentions
```

**None of them had it.** So the one check added because of my incident would have been omitted from
exactly the documents that teach other seats their loop — and would have stayed omitted, because those
documents are the only reason those seats get woken.

## What I did

Added an identical **Step 0** to all five, before the standing checks:

```bash
bash scripts/check-ledger-exists.sh --quiet || echo "LEDGER GONE — stop everything and report"
```

With the PASS/FAIL criteria spelled out (exit 0; exit 1 = fewer than 30 tables; exit 2 = could not
count at all), the fact that the script proves its own count is meaningful by counting schemas, and
the four failures it distinguishes — *container stopped / cannot connect / tables gone / tables present
but no events*.

**And the reason it must be first, stated in the section itself:** seven checks ran every hour for ten
hours and none noticed, because every one asked a question whose answer is a number, and an absent table
and an empty table give the same number. `select count(*) from acceptances` returns 0 for both, and 0
reads as "nothing waiting" rather than "the table is gone". **So on failure the seat must not read a
single row and must not report "nothing to do"** — every further check would return a 0 that reads as a
clean bill of health.

**And the rule it encodes, in Aegis's words:** any conclusion of "I checked, and the result is 0 or
empty" must first prove the query can return non-empty. **Count the tables, then count the rows.** The
order matters — an empty database, one that was never built, and one that was cleared look identical if
you start with rows.

I also renamed each file's closing section from *"When all four pass"* to *"When step 0 passes and all
the standing checks pass"*, so the stop condition names the gate that has to be green first.

## What I did not do, and it is the same thing as last round

**I did not run `scripts/land-standing-duty-docs.sh`.** The branch is 0 commits behind
`track/infra-foundation` and both remotes now hold it (`7e06738`, all three revs equal, gitlab
reachable again today). **Landing it is a direct commit to track, which is an infrastructure action,
and I destroyed the live database last round by taking an unsupervised infrastructure action.**

The blast radius is not what makes the action wrong. Running it would be the same decision with a
smaller consequence, and the whole lesson of the incident is that the consequence size is not the
variable.

**The operator's one command remains: `bash scripts/land-standing-duty-docs.sh`** — documents only,
no restart, and now it carries step 0 to five seats.

## The other blocker, unchanged and still the operator's

`fleet-status.sh` still reports:

```
Only 1 of the six core seats is registered in the seats table (4 rows total).
  target_seat_id has an FK to seats(id), so a dispatch to an unregistered seat
  cannot be inserted.
  Fix: bash scripts/register-fleet-seats.sh  (writes the live database — you run it)
```

**Five of six core seats cannot be dispatched to.** That is Aegis's script, it is zero-destructive
(single `INSERT ... ON CONFLICT DO NOTHING`), and it is explicitly marked as the operator's to run. I
read it and left it.

## What I did not do

I did not run any script that writes to the database. I did not merge anything. I did not grant any
card. I read Aegis's work from git, verified his checker by running it against redirected targets
(writes nothing), and edited five documents on the docs branch that still awaits the operator's land
command.
