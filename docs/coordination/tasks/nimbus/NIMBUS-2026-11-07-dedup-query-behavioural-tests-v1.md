# T3 — Nimbus — 2026-11-07 — Lyra's mutation now fails, and driving the query found a panic in the guard's own error path

Dispatch: `disp-OJpQBzZ75CV9Oqk7L0DFA` (Lyra → Nimbus)
Branch: `packet/nimbus/session-work-readmodel` (from `57b7737`)

## Lyra's finding, reproduced before acting on it

She replaced **both `IS NOT DISTINCT FROM` predicates with plain `=`** — the exact
defect my own comments argue against — and my ten key-struct tests stayed green:
exit 0, 10 passed. She verified the mutation was present before reading the result
(remaining `IS NOT DISTINCT FROM` = 0, new `l2_line_start = $2` = 1), so this was a
real mutation that real tests did not notice.

I reproduced it myself before writing anything. Same result: **10 passed, exit 0.**

Why the green was not a lie but was also not a proof: the key tests never touch a
database. They import `memory_dedup_key` and assert on a struct's fields. The SQL
binds that struct's fields positionally into `$1..$4`, but **nothing connects the
struct's shape to the query's predicates.** So
`a_null_line_range_stays_null_rather_than_collapsing_to_zero` proves the key keeps
`None` as `None` — not that the query compares it with `IS NOT DISTINCT FROM`. Both
can be true while the query is broken.

What the mutation actually does: with `= $2` and a NULL `l2_line_start`, SQL
compares NULL to NULL, matches nothing, and the insert proceeds. A repeat write
carrying no line range is therefore let in — the silent failure my comment names,
in the class that covers three of the four duplicate groups. Not academic; the
majority case.

Her generalisation, which I am recording because it is the load-bearing one:
**a proof that reads source proves the source; a proof that calls a copy proves the
copy; only a proof that drives the real thing proves the real thing.** Here the copy
is sharper than usual because it sits one line above the real thing and its fields
are bound into it — which is exactly why ten passing tests and an unguarded
predicate are compatible.

## What I did

### 1. A second test file that drives the real query

`crates/seatloom-core/tests/memory_dedup_query.rs`, 7 tests. They call
`SeatloomDb::insert_memory` against a real PostgreSQL and assert on the *outcome* —
`Err(DbError::MemoryDuplicate { existing_id, .. })` naming the row already there, or
`Ok(())` for the false-positive direction.

| Test | Pins |
|---|---|
| `a_repeat_write_with_no_line_range_is_refused` | **the case the mutation defeated** |
| `a_title_only_difference_is_refused_by_the_query` | the query does not select on `title` |
| `a_repeat_write_with_identical_ranges_is_refused` | range-carrying repeats |
| `a_null_range_and_a_real_range_are_different_records_against_the_query` | `None` ≠ `Some(1)` |
| `a_different_range_at_the_same_file_is_not_refused` | different evidence is not a duplicate |
| `a_different_file_is_not_refused_by_the_query` | the pointer is part of the predicate |
| `four_distinct_substances_at_one_pointer_all_land` | the false-positive direction, driven |

**Measured against the mutation:**

| Suite | + mutation |
|---|---|
| `memory_dedup_key.rs` (10 tests) | **green — exit 0** (Lyra's finding, reproduced) |
| `memory_dedup_query.rs` (7 tests) | **2 red** — `a_repeat_write_with_no_line_range_is_refused`, `a_title_only_difference_is_refused_by_the_query` |

Three further mutations, each caught: dropping `l0_summary` from the WHERE → 1 red
(false-positive direction); adding `title` to the WHERE → 1 red (the rename defect);
dropping the file-path predicate → **all 7 red**. Each restored from a saved copy and
verified byte-identical.

### 2. A real production defect the new tests found immediately

The **first** run against a real database failed with
`panicked at repositories.rs:1293: error retrieving column 1: error deserializing
column 1`. The guard found the duplicate, then **panicked building its own error**:
`created_at` is `timestamptz` in the schema and the code read it straight into a
`String`.

This is worth more than the test gap. **A guard whose error path panics is worse
than no guard**: the caller gets a panic instead of a refusal, and the refusal is the
whole point — it is what makes a retry recognisable as a retry. Every existing test
passed the whole time, because none of them ever reached that line.

Fix: read it as `DateTime<Utc>` and render with `to_rfc3339()`, matching how every
other `created_at` in this module is handled. The key-struct tests could never have
caught this, because they never touch the query.

### 3. A gate that cannot silently skip

The obvious choice — make these tests **panic** when `DATABASE_URL` is unset — was
measured and rejected. It took `cargo test --workspace` from 188 passed to
**198 passed / 7 failed**, and all 7 failures were "DATABASE_URL is not set". A gate
that is red for unrelated reasons is a gate people learn to ignore, and I would have
made the workspace worse for every seat without a scratch database.

So the tests are `#[ignore]`d with a reason naming how to run them — which cargo
**counts** in its output as `N ignored`, not hidden — and
`scripts/test-memory-dedup-query.sh` is the actual gate. It runs them with
`--include-ignored` and **fails if zero tests ran**, which is the part that closes
the silent-skip hole: a configuration where they would all skip is itself a failure,
reported as such.

Exit codes, all measured: **2** for a database that does not exist, **1** for the
live `seatloom` database (refused by name, because these tests insert and delete
rows), **1** if a test fails, **0** when all seven pass.

The script also applies the schema to a fresh scratch database. One ordering bug I
hit and fixed there: the original treated "no `memories` table" as "schema not
applied", so a dead endpoint reported "schema apply failed" — correct by accident,
but the message named the wrong cause and the silent-skip guard was unreachable.
Connectivity is now checked first, separately.

### 4. A scratch database

`seatloom_nimbus_dedup_probe`, created and schema-applied by hand, following the
forty-plus existing scratch-DB convention. **Not the live database** — which, note,
is *not* schema-only as both Lyra and I had understood: it currently holds **43
memories**. Her 10-31 incident emptied it at the time; it has been written to since.
So the "unverifiable on live" framing for the four-groups claim still stands, but
the reason is different from "the rows are gone" — they exist, they are just not
*those* rows, and I did not query them for this.

## Gates

| Gate | Result |
|---|---|
| `cargo test --workspace` (no DB) | **332 passed / 0 failed**, ignored 74 (was 67) |
| `memory_dedup_key.rs` | 10 passed / 0 ignored |
| `scripts/test-memory-dedup-query.sh` | **exit 0**, 7 passed |
| Same gate + Lyra's mutation | **exit 1**, 2 red |
| `cargo build --workspace` | 0 errors, **0 warnings** |
| `rustfmt --check` | all touched files clean; `repositories.rs` was clean at HEAD |

## Not done

- No card, merge, install, restart, live-DB write, or sandbox replacement.
- Did not verify the four-groups-zero-false-positives claim; still unverifiable, and
  now for a different reason than the one stated last time.
- No Flux verification; nothing names this commit.
- I did not fold this into the earlier 023 slice's claim that the key tests
  "pin the three directions" — that was true of the key, and I have now written down
  explicitly, in both files, that it was never true of the query.
