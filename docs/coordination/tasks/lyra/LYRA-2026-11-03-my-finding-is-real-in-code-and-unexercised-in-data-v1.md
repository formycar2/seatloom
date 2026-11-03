# Standing-duty verification: I tried to prove my published finding with data instead of source, and the data says the divergence is currently unreachable — the only writer that populates any of the four tables writes statuses the inbox correctly ignores. The finding is real in code and unexercised in data, and those are different claims.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-my-finding-is-real-in-code-and-unexercised-in-data-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, verification, scoping, honest |

## What I set out to do

My published finding is that the inbox badge and the supervisor panel count different sets under one label. I
verified it from source. **This round I tried to verify it from data** — to build a state where the two counts
actually disagree, so the finding would be demonstrated rather than traced.

**I could not, and the reason is a measurement about the system that I did not have before.**

## Measured: which of the four tables has a writer at all

```
workitems    writers: 1     (crates/seatloom-core/src/db/write_api.rs:639)
handoffs     writers: 0     (no INSERT INTO handoffs in crates/ or src-tauri/)
acceptances  writers: 3
dispatches   writers: 3
```

**`handoffs` has no application writer at all.** It is populated only by `infra/postgres/seed/001`, and that
seed inserts exactly one handoff, with status `completed`.

**And `completed` is one of the five statuses the inbox deliberately does not count** (`Drafted`, `Accepted`,
`Returned`, `Completed`, `Expired` are uncounted; `Sent`, `Received`, `Working` are counted). **So the one
seeded handoff is correctly ignored by the badge, and the badge's handoff half has never been exercised by a
row it should count.**

## What this means for the finding, stated precisely

**The finding stands as a statement about code.** Two surfaces, two independent fetch paths, no reconciling
layer, different populations, same label, co-visible in one shell. **All of that is measured and true.**

**The finding is not currently a statement about what the operator sees.** With the ledger empty, both counts
are 0. And the divergence I predicted — a badge of 0 beside a panel of 1 — requires a pending acceptance,
which requires the operator to open a card, which requires the cards that were destroyed with the ledger.

**So the honest form is: "this will mislead the operator the moment a card exists," not "this is misleading the
operator now."** I published it last round with the first framing, and the measurement this round tightens it
to the second. **I am not retracting it, because a latent defect in the surface the operator glances at is
worth knowing before the data arrives. I am narrowing what it claims.**

## The thing I nearly got wrong again, and why I am recording it

**The handoff filter counts 3 of 8 statuses, and my first instinct was that this was another silent drop** —
the same shape as the workitem 3-of-9 I caught myself on last round.

**It is not the same, and the difference is measurable.** For workitems, the six uncounted statuses are
correctly uncounted because none of them waits on the operator. **For handoffs, the five uncounted include
`Returned`** — a handoff that was sent back. **A returned handoff is arguably the operator's to act on**, and
nothing in the code says otherwise.

**But `Returned` is never written by anything.** No application writer, no seed. So it is not a live gap; it is
a status the vocabulary declares and the system has never produced. **That is a different claim from "the
filter drops an actionable state," and I would have published the second if I had stopped at the 3-of-8
count.**

**The rule this reinforces, and it is now twice in two rounds:** a count of the vocabulary is not a count of
the reachable states. **Nine declared, three filtered, and the question that decides whether the gap matters
is how many of the six are ever written.** Last round the answer was "all of them, correctly ignored." This
round the answer is "one of the five, and it is never written at all." **Same shape of discrepancy, opposite
conclusions, and only the writer measurement tells them apart.**

## What I did not do

I did not edit any filter, type, or store. I did not build a scratch database to force the divergence — the
divergence needs a pending acceptance, and creating one would mean writing to a live ledger I have already
destroyed once. My measurements were `grep` for `INSERT INTO` per table across `crates/` and `src-tauri/`, the
seed's handoff row, the handoff status vocabulary and its map, and the per-status filter comparison. `seats` is
10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
