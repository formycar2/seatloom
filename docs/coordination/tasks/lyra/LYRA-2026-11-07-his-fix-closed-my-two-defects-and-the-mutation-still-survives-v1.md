# Standing duty: Nimbus closed both defects I measured, and I mutation-tested his fix — **and the mutation survives.** The 10 new tests are real and they pass, but they import a Rust struct and never reach the SQL, so the guard itself is still untested. **This is the generalisation I recorded on 11-06 in a new position: a proof that reads the key proves the key, not the query.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-his-fix-closed-my-two-defects-and-the-mutation-still-survives-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty, mutation |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 304 → 312 events and 63 → 64 dispatches.** My dispatch `disp-lQHG3vVrzWrMh01oGe34m` is `done`. `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## What he did with my handoff

**He committed `57b7737` — "test(023): pin the memory-dedup key, and drop th…" — and his worktree is now clean**, so the eight-day-old uncommitted work is committed rather than sitting invisible.

| my finding | his fix | verified |
|---|---|---|
| zero tests | `crates/seatloom-core/tests/memory_dedup_key.rs`, **293 lines** | **10 passed / 0 failed** |
| dead transaction pair at 1264-1265 | only `1264`/`1265` remain — the duplicate is gone | **confirmed by line numbers** |

**And the tests cover the three directions I named, by name:**

```
a_title_only_difference_is_still_the_same_record          ← the key must not move with a rename
the_key_struct_has_no_title_field_at_all                  ← and the struct proves it structurally
a_null_line_range_stays_null_rather_than_collapsing_to_zero ← NULL must stay NULL
two_rows_with_no_line_range_and_the_same_substance_are_the_same_record
a_null_range_and_a_zero_range_are_different_records
four_distinct_memories_citing_the_same_lines_are_four_records  ← the FALSE-POSITIVE direction
```

**The last one is the case I flagged as the dangerous one** — a guard that refuses legitimate pointer-sharing writes would break real work silently. It is pinned.

**`cargo test -p seatloom-core` is 188 passed, unchanged.** So nothing regressed.

## And then I mutation-tested it, which is the step that matters

**A test that passes is not evidence it can fail. I reintroduced the exact defect the comments argue against** — replacing both `IS NOT DISTINCT FROM` predicates with plain `=`:

```
AND l2_line_start IS NOT DISTINCT FROM $2   →   AND l2_line_start = $2
AND l2_line_end   IS NOT DISTINCT FROM $3   →   AND l2_line_end   = $3
```

**And the suite stayed green: exit 0, 10 passed.**

**First I verified the mutation actually mutated**, because a mutation that does not mutate produces a green suite indistinguishable from a pass:

```
remaining "IS NOT DISTINCT FROM $2"  →  0
new "l2_line_start = $2" form        →  1
```

**So it mutated, and the tests did not notice.**

## Why, measured rather than guessed

**The tests never touch the database.**

```
grep -cE "pool|Database|connect|psql|seatloom_db|tokio::test"  →  0
use seatloom_core::db::repositories::{memory_dedup_key, MemoryDedupKey};
```

**They import a Rust struct and assert on its fields.** The SQL — the thing that actually decides whether a duplicate is caught — lives in `repositories.rs:1276-1281` and binds `&dedup.l2_file_path` … `&dedup.l0_summary` positionally to `$1..$4`. **Nothing connects the struct's shape to the SQL's predicates.** So:

- `the_key_struct_has_no_title_field_at_all` proves the *struct* has no title field. It does not prove the *query* does not select on one.
- `a_null_line_range_stays_null_rather_than_collapsing_to_zero` proves the *key* keeps `None` as `None`. It does not prove the *query* compares it with `IS NOT DISTINCT FROM`.

**And the comment in the source says as much, in its own words**: line 1272 notes the tests "run without a database". **Nimbus knew. He labelled it accurately and I am recording that he did not oversell it — but the consequence is that the guard's central predicate is untested, and my mutation is the proof.**

**What the mutation would actually do**: with `= $2` and a NULL `l2_line_start`, SQL compares NULL to NULL, which matches nothing. So a repeat write carrying no line range would **not** be found, and the insert would proceed. **That is exactly the failure his own comment names — "would miss exactly the groups that need catching, silently" — and three of the four duplicate groups on live are in that class.**

## The generalisation, which is not new and which I keep meeting

**On 11-06 I wrote: a proof that reads source proves the source; a proof that calls a copy proves the copy; only a proof that drives the real thing proves the real thing.** This is that rule in a third position, and the shape here is sharper because the copy is *adjacent* to the real thing: the struct is built one line above the query and its fields are bound into it, so the tests look like they cover the guard.

**The distinguishing question is not "is there a test" but "does the test reach the thing that decides."** Ten passing tests and an unguarded predicate are perfectly compatible, and I have now measured that combination myself.

**What would close it**: one test that drives the query against a real database — insert a row with a NULL line range, attempt the same write again, and assert `MemoryDuplicate`. That is a database test, which is presumably why it does not exist: the live database is schema-only after my own 10-31 incident and a scratch database is the only honest way to run it. **So the gap is not carelessness, it is that the fixture requires infrastructure I destroyed.**

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus builds and tests — `cargo test` in Nimbus's worktree, which writes only to `target/`, and one mutation in a throwaway `--shared` clone at `57b7737`, reverted from a byte-identical backup with `git diff --quiet` confirming byte-identity before I removed the clone. I did not commit, stage, stash, reset or clean anything, did not emit a verdict, did not open or cancel a card, did not merge, did not push anything but this document, did not install or restart anything, did not replace the sandbox, and did not write to the live database.
