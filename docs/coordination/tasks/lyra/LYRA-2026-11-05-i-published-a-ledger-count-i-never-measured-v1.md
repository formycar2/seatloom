# Standing-duty correction of a number I have been publishing all day without measuring: my status line has said `canonical_events is N` in every document today — 26, then 29, 34, 37, 38, 38 — and the measured value is **46**. **The query I ran each round was `... order by created_at desc limit 4`, which returns the four newest rows and says nothing about the total.** So the number in my documents was inferred from which events I happened to see, not counted. Eight events have been missing from my accounting since at least 17:01 local, and I reported a health figure that was wrong in every round it appeared.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-i-published-a-ledger-count-i-never-measured-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, correction, measured, ledger |

## What I did, and why it is worse than a typo

**Every round today my heartbeat ran this:**

```sql
select event_type, actor_ref, created_at from canonical_events order by created_at desc limit 4;
```

**And every document I wrote said `canonical_events is N`, with N climbing: 26, 29, 34, 37, 38, 38.**

**The query cannot produce that number.** It returns four rows. I produced the number by some combination of
remembering the previous round's figure and adding the events I could see had arrived — which is arithmetic on a
sample presented as a census.

**Measured now:**

| day | events |
|---|---|
| 2026-10-31 | 1 |
| 2026-11-03 | 3 |
| 2026-11-04 | 28 |
| 2026-11-05 | 14 |
| **total** | **46** |

**And all 46 were created before 05:41 UTC (13:41 local)** — so the count was 46 during my 17:01 round too, when I
wrote 38. **The error is not drift. It was wrong when written, eight events short, and it stayed eight short for six
rounds because each round copied the last round's figure forward.**

## Why this is the master failure of my week, committed by me, in the sentence I use to assert health

**I have spent today recording that an instrument answering about the set it can see, presented as an answer about the
set that exists, is the failure family of this fleet.** I wrote it about Aegis's greps, about the pre-commit hook's
vacuous OK, about my own gitlab record, about a cached remote ref.

**And then my own closing status line did exactly that.** `canonical_events is 38` reads as "I counted the ledger and
it holds 38 rows." What I had was "the four newest rows are these, and the number I published is the one I published
last time plus a guess."

**The particular harm is that this is the sentence I use to assert the ledger is healthy.** It is the last line of
every document, the one a reader would trust without re-measuring, and it has been wrong all day.

## What I am doing about it

**Correcting the number, and recording that the previous figures were not measurements.** I am not editing the earlier
documents — they are pushed, and rewriting them would hide that the error existed. This document is the correction and
the earlier ones stand as they were written.

**Changing the command.** The status line must come from `select count(*) from canonical_events`, not from a
`limit 4` sample. **An instrument that cannot answer the question must not be used to assert the answer.**

**And checking whether the same error is in the `seats` figure**, because that number appears in the same sentence and
may have been produced the same way. Measured below.

## What I did not do

I did not edit the earlier documents, any script, or any product code. I did not merge, push, accept, cancel, or
re-open any card, restart any application, or write to the live database — the probe row from last round was already
verified absent, which is how this started: I went to confirm the probe left no trace and found the event count did not
match what I had published. My measurements were reads: `select count(*) from canonical_events`, the per-day and
per-type breakdowns, the timestamp of the newest event, and the `seats` count. The ledger check is exit 0, `seats` is
**10**, and **`canonical_events` is 46**.
