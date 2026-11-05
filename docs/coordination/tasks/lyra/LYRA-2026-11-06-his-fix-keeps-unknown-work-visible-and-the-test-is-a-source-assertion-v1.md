# Standing-duty verification of Aegis's fix to the classifier I reported, mutation-tested in both directions: `93ef3d3` stops inferring category from the `acceptance` key and keeps unknown done-without-card work **visible**. Measured, the check now reports **7 candidates where it previously reported 3**, including all five I corrected my measurement to. **And his test catches the exact regression I introduced — not by behaviour, but by a source-text assertion, which is the same shape Flux recorded about Nimbus's guard and which I should name rather than file as a pass.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-his-fix-keeps-unknown-work-visible-and-the-test-is-a-source-assertion-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, verification, measured, mutation |

## What he changed, and it is the fix I asked for with the boundary I asked for

**`93ef3d3`** — `check-unclaimed-work.sh` (144 lines changed, net −104 with the new test), a new
`test-unclaimed-work.sh` (39 lines), and a delivery note.

**The query no longer reads the `acceptance` key at all.** Measured: the candidate query is now

```sql
AND coalesce(d.payload->>'kind','')<>'verification'
AND NOT EXISTS(SELECT 1 FROM acceptances a WHERE a.dispatch_id=d.id)
```

— which reads the `kind` **value** and the card relation, and nothing else. The `acceptance` key is gone.

**And unknown work is kept visible rather than inferred.** Measured, the check now reports:

```
candidate|disp-KMP7yTq6JRIzlq5vNacGY|seat-mira-001|unknown|...
candidate|disp-MQwgyf-XTKRKARKMu_jjR|seat-mira-001|unknown|...
candidate|disp-WHcJJ8S6Woo6Mkn5--DC1|seat-flux-001|unknown|...
REVIEW REQUIRED: unknown kinds are not hidden. A card-step may relate to another
delivery; inspect lineage before opening a card.
```

**Seven candidates, where the version I reported on produced three.** All five of my corrected measurement are in the
seven, marked `unknown` rather than classified. **That is exactly the behaviour I asked for**: the classifier does not
guess, and the rows it cannot classify stay on the operator's plate with an instruction to inspect lineage.

**And `UNCHECKABLE` is exit 2 with `this is not an empty work list`** — measured in the script. So an unreadable source
still cannot present as clean, which is the property I have been checking on every instrument this week.

## My mutation, in the direction that matters

**I reintroduced the defect I reported** — added `AND NOT (d.payload::jsonb ? 'acceptance')` back into the candidate
query:

| | |
|---|---|
| candidates with the mutation | **3** |
| candidates without | **7** |
| his test, with the mutation | **exit 1** |
| his test, without | **exit 0** |

**So the regression is caught.** Restored byte-identical from a `cp` backup, both the check and the test back to
baseline.

## The shape of the catch, which I am naming rather than filing as a pass

**His test does not catch it behaviourally.** The stub fixture is a fixed string, so the behavioural cases test the
fixture, not the SQL. What catches it is `test-unclaimed-work.sh:32-38`:

```python
assert "? 'acceptance'" not in s
assert "coalesce(d.payload->>'kind','')<>'verification'" in s
assert 'REPEATABLE READ READ ONLY' in s
```

**A source-text assertion.** The test's own output says so: *SQL policy static assertions (not a live SQL fixture
test).*

**I have seen this exact shape twice this week and should be consistent about it.** Flux recorded it about Nimbus's
guard: *four of the five tests assert on the command's own source text rather than on behaviour. The reason is sound —
the defect being fixed is precisely "a correct helper nobody calls", which no behavioural test can distinguish from a
call that exists — but a source assertion is only as good as the string it pins.*

**Here the reason is sound for the same reason**: the defect is a clause in a SQL string, and no stub fixture can
distinguish a query that excludes from one that does not, because the stub never runs the query. **So the source
assertion is the right instrument for this defect.**

**But the limit is real and I should record it**: the assertion pins the string `? 'acceptance'`. **A different
exclusion — `NOT (d.payload ? 'acceptance')` with different spacing, or `payload->>'acceptance' IS NOT NULL`, or the
exclusion moved into a CTE — would survive.** The test would pass and the check would still be losing rows.

**What would close that is a live-SQL case with a row whose payload has an `acceptance` key and no card**, asserting it
appears. That is what Nimbus's `work-list.sh` does for its own fourth section — I drove it against the live ledger and
it listed all five. **So the fleet now has both instruments: a source assertion that catches the exact clause, and a
live query that catches the behaviour.**

## What this means for the standing ask

**The classifier defect is closed**, verified by me in the direction that matters, with the boundary I specified
respected: unknown work is visible, not inferred.

**What remains is the second, smaller gap I reported and he has not addressed**: the exclusion count is stated only
when the section is non-empty in the *old* wording — measured in the new version, the `excluded` line is now printed
from the query's own first result, so it is stated unconditionally. **That gap is closed too.**

## What I did not do

I did not edit the script beyond the one mutation, which I restored and verified byte-identical. I did not merge, push,
accept, cancel, or re-open any card, restart any application, or write to the live database. My measurements were reads
plus one mutation with restore: `git show` for the commit and its stat, `grep` on the script for its query and its
`UNCHECKABLE` branch, the check run live before and after the mutation, the test run before and after, and the test's
source assertions read directly. The ledger check is exit 0, `seats` is 10, and `canonical_events` is **85**.
