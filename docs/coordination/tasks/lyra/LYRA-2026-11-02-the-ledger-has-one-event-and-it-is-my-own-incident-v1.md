# Standing-duty verification: the ledger has one event and it is my own incident report from 36 hours ago — every green check in the fleet is green against an empty subject, which is exactly the failure my Step 0 was built to catch, one level down

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-the-ledger-has-one-event-and-it-is-my-own-incident-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, measurement, ledger, failure-family |

## Why I measured this

Aegis's 16:00 heartbeat reports six checks at exit 0, and states the right caveat about each: *"0 pending
cards. **Empty is not green.**"* That is the correct instinct and I want to record that he has it. **But a
caveat is not a measurement, so I measured the emptiness myself.**

## Measured, in the live database, at 11-02 16:01

```
canonical_events   1
seats              9
dispatches         0
acceptances        0
workitems          0
artifacts          0
memories           0
topics             0
documents          0
sessions           0
handoffs           0
```

**And the one event is mine:**

```
ev-__d45RX5PFjTs33KTPuy6 | ArtifactCreated | 2026-10-31 02:57:27 | seat:seat-lyra-001
  {"path": "docs/coordination/tasks/lyra/LYRA-2026-10-31-i-destroyed-the-live-database-v1.md", ...}
```

**The ledger's entire contents, 36 hours after the wipe, is the document in which I reported destroying it.**
Nothing has been written to the canonical ledger since — not by any seat, not by any check, not by any of
today's work.

## Why this is the failure my own Step 0 was built to catch, one level down

**Step 0 (`check-ledger-exists.sh`) counts tables.** It exits 0 when there are 30 or more, and it caught the
state where the schema was gone. **It cannot see this state**, and this state is the same shape:

- Before: **0 tables** → every query returns 0 → *"0 reads as nothing to do, not as everything is gone."*
- Now: **35 tables, 1 event** → every query returns 0 or 1 → *"1 reads as a working ledger."*

**The instrument that was supposed to make "empty" distinguishable from "absent" is now looking at a ledger
that is empty and reporting 35.** The number that tells you the schema exists is being read as the number that
tells you the system works. **I wrote that failure mode into my own heartbeat's Step 0 rationale and the
instrument I commissioned still cannot catch it.**

**Concretely: `check-cards-clickable.sh` exits 0 with 0 pending cards. It is not wrong — there are no cards to
break. But it is also not evidence that anything works, and a green row in a fleet status table reads
identically either way.** Six green checks, zero subjects. That is a fleet of instruments answering questions
about an empty set.

## The part that is not a defect and should be said

**Seats is 9.** The operator ran the registration, and the six core seats are all present. The dispatch
channel's foreign key is satisfied. **So the system is in the state where it *could* start recording, and
nothing has been dispatched to anyone.** The last real dispatch was destroyed with the ledger; the git history
survived, and the code is all there.

**This is the state I named on 10-31: the code is on git, only the acceptance record was lost.** 36 hours
later that is still exactly the state, and the reason is that every remaining ask is the operator's — land the
standing-duty docs, merge Mira's branch — and neither of those writes to the ledger either.

## What I did not do

I did not write a synthetic event to test whether the ledger accepts writes — that would be fabricating
history to make an instrument look alive, which is the specific failure this document is about. I did not edit
any check. I did not dispatch anything. My measurements were `psql` counts and one `select *` on
`canonical_events`. `check-ledger-exists.sh` is exit 0, which is the point.
