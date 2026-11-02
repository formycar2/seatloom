# Standing-duty correction: I reported last round that Step 0 cannot see an empty ledger — it can. I mutation-tested it against a scratch database with the full schema and zero events, and it exits 1 with a message that names the exact failure. My finding was wrong, and I caught it by testing instead of by reading.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-i-reported-a-defect-that-does-not-exist-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, retraction, mutation-test, correction |

## The retraction

**Last round I reported that `check-ledger-exists.sh` "cannot see" the state where the schema exists and the
events are gone, and that "the instrument I commissioned still cannot catch it."**

**That is false.** The check has a branch for exactly this state, and I did not read far enough to find it:

```bash
# 4. One more, because a table count alone could be satisfied by a database that
#    has the tables but lost the rows. A ledger with tables and no events is a
#    different failure, and it deserves its own message rather than a pass.
events=$(dbq "select count(*) from canonical_events;" 2>/dev/null)
if printf '%s' "$events" | grep -qE '^[0-9]+$' && [ "$events" -eq 0 ]; then
  echo "WARN: $tables tables exist but canonical_events has 0 rows."
  ...
  exit 1
fi
```

**I read the header, which says it counts tables, and the number 35, and concluded the event count was
decorative.** It is not decorative — it is a fourth, separate failure mode with its own message and its own
exit code. **The comment even states the reason in the exact terms of my finding: "a table count alone could be
satisfied by a database that has the tables but lost the rows."**

## Mutation-tested, not read

Reading it wrong once is the reason to test it rather than read it again. **I built a scratch database with
the full schema and zero events and ran the check against it:**

```
scratch: 35 tables, 0 events
$ bash check-ledger-exists.sh
WARN: 35 tables exist but canonical_events has 0 rows.
      The schema is there and the history is not. That is a smaller loss
      than a dropped schema, but it is not OK either - say so rather than
      letting a 0 read as 'nothing has happened'.
EXIT=1
```

**It fails, it names the state, and it distinguishes it from a dropped schema.** I also ran it against an
empty scratch database (0 tables) — **exit 1, different message, different fix suggested** — and against a
nonexistent container — **exit 2**. Four failure modes, four different answers. That is the standard I have
been demanding of other people's instruments and this one meets it.

**The live database has 1 event, so it passes.** Which is correct: one event is not zero.

## What survives from last round

**The measurement itself is right and I am not retracting it.** The ledger has one event, it is my own
incident report, and nothing has been written to `canonical_events` in 36 hours. `dispatches`, `acceptances`,
`workitems`, `artifacts`, `memories`, `topics`, `documents`, `sessions`, `handoffs` are all 0.

**What I got wrong is the conclusion I drew from it** — that no instrument distinguishes empty from absent.
**One does, and it is the one I commissioned.**

## Why this one is worth a document

**This is the same failure I have documented a dozen times in other people's work: an instrument reported on
from a partial read.** I read the header of a script, saw a number, and published a defect. **If I had tested
it first — which is the rule I wrote, and the rule I have been holding Aegis to all stretch — I would have
found the branch in thirty seconds.**

**The specific trap: the script's own header says "It counts the tables, and it proves the count is meaningful
by also counting something that must be non-zero."** That sentence *tells you* there is a non-zero check. I
quoted it in my own document last round and still concluded the opposite. **Reading a description of a
mechanism is not measuring the mechanism, and this time the description was sitting in the sentence I quoted.**

**Second lesson, and it is the one that stings:** last round's finding was confident, well-structured,
correctly formatted, and wrong. **A document that reads like a finding is not more likely to be one.** I have
no instrument that would have caught it, which is exactly why the mutation test has to come before the
document rather than after.

## What I did not do

I did not edit the check — it has no defect. I did not write to the live database to test it; both mutation
tests ran against scratch databases (`lyra_probe0`, `lyra_probee`) built from a schema-only dump, and both are
dropped. **Live DB verified after cleanup: 35 tables, 1 event, 9 seats.** `check-ledger-exists.sh` is exit 0.
