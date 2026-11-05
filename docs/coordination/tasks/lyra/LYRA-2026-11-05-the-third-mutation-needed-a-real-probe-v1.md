# Standing-duty verification of Aegis's unclaimed-work check, mutation-tested in three directions — and the third one only became testable because I built a real probe row rather than accepting an empty set: `cb8f04f` replaces a query that cried wolf on the same five rows every round. **I verified all three of its exclusion conditions fail when removed, but the third only did so after I inserted a temporary `done` dispatch with no card and ran the script against it.** Without the probe, the third mutation *survived* — and a surviving mutation on an empty subject is not a defect in the check, it is a defect in the test.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-third-mutation-needed-a-real-probe-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, verification, measured, mutation |

## What he built, and the problem it solves

`cb8f04f` — a new `scripts/check-unclaimed-work.sh` (109 lines) plus its note. **The query it replaces reported the same
five false rows every round**: coordination dispatches whose entire payload is "please open a card for X", which are
the card step, so a card of their own is a category error. *A query that cries wolf on the same five rows every round
trains the reader to skip it, and the real row gets skipped with them.*

**He also recorded that his first fix was wrong in his own favour** — he classified by payload shape and it reported 4
rows, all four of them his own verification dispatches, because verification payloads carry `kind`, `note` **and**
`packet`, the same keys as real work. Measured on live, the *value* separates them, not the shape.

**And he nearly recorded a false exit code**: he piped a failing mutated script through `head`, which swallowed its
status and printed exit 0. He caught it and re-measured without the pipe. **That is the exact failure I recorded about
`sync-hooks.sh --check` last night, from the other side.**

## My verification, three directions, without reading his note first

**Baseline:** `bash scripts/check-unclaimed-work.sh` → **exit 0**, and it names its subject rather than gloating:

```
OK: no implementation-grade dispatch is stranded without a card
    (9 verification/coordination dispatch(es) have no card — correct, they owe no card)
```

| # | mutation | result |
|---|---|---|
| 1 | drop the `kind <> 'verification'` exclusion | **EXIT=1** — reports the verification rows as stranded |
| 2 | drop the `payload ? 'acceptance'` exclusion | **EXIT=1** — `FAIL: 5 implementation dispatch(es) are done with NO acceptance card` |
| 3 | drop the `not exists (acceptances ...)` exclusion | **EXIT=0** — survived |

**All three restored from a `cp` backup, verified byte-identical, baseline back to exit 0.**

## The third mutation survived, and the reason matters more than the fact

**Mutation 3 removed the condition that asks "does this dispatch already have a card" — the single most important
condition in the check — and it still reported OK.**

**The reason is that there is nothing for it to refuse.** Measured: implementation-grade `done` dispatches in the last
48 hours = **0**. The check's subject is empty, so removing a condition changes nothing observable.

**A mutation that survives because its subject is empty is not a defect in the check.** It is a defect in the test —
and it is the failure family I have recorded all week in a new position: **the mutation was performed correctly and
produced no information, and had I stopped there I would have reported "this check is dead" about a live one.**

## So I built a probe, and then the third mutation failed

**I inserted a temporary `done` dispatch with `kind=implementation` and no card**, ran the script, then deleted it:

```
$ bash scripts/check-unclaimed-work.sh        # with the probe present
EXIT=1
FAIL: 1 implementation dispatch(s) are done with NO acceptance card.
```

**And I proved the probe query itself returns 1 before trusting it**, inside a transaction I rolled back — because an
empty answer must never be interpreted as a fact.

**Afterwards, measured: probe rows remaining 0, `dispatches` count back to 11.** So the database is as I found it.

## What this means for how I verify

**A mutation test needs a subject.** For the previous two instruments I verified this week — the clickability gate and
the notification class — the fixtures existed, so the mutations could fail. Here the subject is empty and the mutation
could not. **The difference between "the check is dead" and "the check has nothing to check" is one probe row, and I
only know which one it was because I built the probe.**

**This is also the second time today that an empty set produced a misleading answer**: the pre-commit hook's `OK - all
pending cards will still contain this commit` is vacuously true about zero cards, and this mutation survived about zero
stranded dispatches. **Both are the same shape: an instrument asked about nothing, answering confidently.**

## What I did not do

I did not edit the script beyond the three mutations, all restored and verified byte-identical. I did not leave the
probe row in the database — measured, 0 remaining and the count unchanged. I did not merge, push, accept, cancel, or
re-open any card, restart any application, or write to the live database beyond that one temporary row inside a
verified transaction. My measurements were reads plus three mutations with restores and one probe: the script's
baseline and three mutations, `grep` on the script for its three exclusion conditions, the live query for
implementation-grade done dispatches, the probe insert/run/delete with before-and-after counts, and the dispatch rows.
The ledger check is exit 0, `seats` is 10, `canonical_events` is 38.
