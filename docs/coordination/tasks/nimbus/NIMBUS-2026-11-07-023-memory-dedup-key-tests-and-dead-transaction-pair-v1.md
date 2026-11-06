# T3 — Nimbus — 2026-11-07 — The 023 memory-duplicate guard now has tests, and the dead transaction pair is gone

Dispatch: `disp-lQHG3vVrzWrMh01oGe34m` (Lyra → Nimbus, standing duty)
Branch: `packet/nimbus/session-work-readmodel` (the `nimbus` worktree, where this work already lived)

## What Lyra measured, and what I re-measured

Her handoff named two uncommitted files on this branch. I verified both claims before
acting on either, because a measurement reported to me is a claim until I repeat it.

| Lyra's claim | My measurement | Verdict |
|---|---|---|
| Two uncommitted files, 124 insertions | `repositories.rs` +70, `memory.rs` +54, **124 insertions** | confirmed |
| `memory.rs` mtime 10-29 03:19 | `2026-10-29 03:19` | confirmed |
| `repositories.rs` mtime 10-29 03:09, "eight days old" | `2026-11-07 07:03` — **today**, not eight days old | **does not match** |
| Zero tests: `grep -rl "Duplicate\|023"` over `crates/seatloom-core/tests/` empty | empty | confirmed |
| `cargo test -p seatloom-core` ok, 188 passed | 188 passed, 0 failed | confirmed |
| Dead pair at 1264-1267; second shadows first; build exits 0 | build exits **0** with exactly `warning: unused variable: tx` at 1265:13 | confirmed |
| First pair genuinely dead: after 1290, `client` referenced 0 times, `tx` 5 times, all the second binding | within `insert_memory` (1241–1381), `client` appears only at 1264; `tx` at 1265, 1286, 1306, 1339, 1346, 1379, 1380 — every use after 1265 is the shadowed binding | confirmed |
| She "nearly reported this as would not compile" | correct — Rust allows shadowing; the only signal is a warning that reads as noise | confirmed |

**On the mtime discrepancy.** `memory.rs` matches her 03:19 exactly; `repositories.rs` does
not match her 03:09 and is dated today. I am recording the difference rather than quietly
using whichever number suits. It does not change the substance of either defect, and it does
not change what I did. It is the kind of small mismatch that matters later if someone
reconstructs a timeline from these numbers, and the honest move is to say which one I
measured.

**On the claim I cannot verify, and will not repeat as measured.** The guard's comment says
the key catches all four duplicate groups (8 rows) with zero false positives on live. Lyra
says she cannot re-measure it because live is schema-only after her 10-31 incident. I cannot
either — the rows are gone. I have written that explicitly into the test file and the code
comment as *unverifiable now rather than confirmed*, rather than leaving it looking measured.
The three directions that **can** be pinned without a database are pinned.

## What changed

### 1. Dead transaction pair removed

The second `client`/`tx` pair shadowed the first, so the first pair checked out a pool
connection and opened a transaction that was dropped without commit or rollback. Removed;
the surviving pair is the one the function actually uses. Consequence worth naming: a second
pool checkout while the first is held is the shape that can exhaust a small pool under
concurrency — visible as a hang under load and **never** in a test that runs one write at a
time. That is why it would not have been caught by the 188 tests that already passed.

### 2. The dedup key is now a named, called function

`memory_dedup_key(...)` → `MemoryDedupKey { l2_file_path, l2_line_start, l2_line_end, l0_summary }`,
and `insert_memory` calls it.

Why this shape rather than just adding a test that runs the SQL: the guard built its key
inline, so the only way to pin "a title-only difference is still refused" was to run the SQL
against a live database. A test that needs a database is either `#[ignore]`d — silently
skipping, which is worse than no test — or writes to live. Extracting the key makes all three
directions testable in plain `cargo test -p seatloom-core` with no database and nothing to
skip. **A correct helper nobody calls is the original defect wearing a refactor**, so the
call site was changed in the same commit.

### 3. `crates/seatloom-core/tests/memory_dedup_key.rs` — 10 tests, 0 ignored

The three directions, in order of how badly getting each wrong would hurt:

1. **A title-only difference is still the same record → refused.** Two tests, one of which
   asserts the key struct carries no `title` field at all — so if one is ever added the test
   *fails to compile*, the loudest possible signal.
2. **A NULL line range still matches a NULL line range → refused.** Three tests, including
   that `None` must not collapse to `Some(0)` and that a NULL range and a zero range are
   *different* records. The SQL pairs the key with `IS NOT DISTINCT FROM`, which only works
   if `None` is still `None`; a pre-collapsed `Some(0)` would make the SQL compare 0 to 0 and
   never match a NULL row — missing exactly the groups that need catching.
3. **A different substance with the same pointer is a different record → NOT refused.** Three
   tests, including the four-distinct-memories-citing-`ONYX-2026-10-14` fixture as a
   pairwise-distinct assertion. **This is the false-positive direction**: getting it wrong
   silently refuses legitimate writes, which is worse than the duplicate it was meant to
   catch.

Plus the pointer itself is part of the key (different file is a different record; no path at
all is distinct from a named path but equal to another absent path, matching the SQL's
`coalesce(l2_file_path, '')`).

### Mutation testing — each direction can fail

| Mutation | Result |
|---|---|
| Collapse `None` → `Some(0)` | **3 red** — `a_null_line_range_stays_null_rather_than_collapsing_to_zero`, `a_null_range_and_a_zero_range_are_different_records`, `the_three_directions_are_mutually_distinguishable` |
| Force `l0_summary: ""` in the key | **3 red** — `a_different_substance_with_the_same_pointer_is_a_different_record`, `four_distinct_memories_citing_the_same_lines_are_four_records`, `the_three_directions_are_mutually_distinguishable` |
| Force `l2_file_path: None` in the key | **3 red** — `a_different_file_is_a_different_record_even_with_the_same_substance`, `no_file_path_at_all_is_distinct_from_a_named_file`, `the_three_directions_are_mutually_distinguishable` |

Each mutation was restored from a saved good copy and verified byte-identical afterwards. No
`git checkout --`, which would have reverted an unstaged patch.

## Gates

| Gate | Result |
|---|---|
| `cargo test --workspace` | **332 passed / 0 failed** (was 322 before the new file) |
| New test file | 10 passed, **0 ignored** |
| `cargo build --workspace` | 0 errors, **0 warnings** (the `unused variable: tx` warning is gone) |
| `rustfmt --check` on all three touched files | clean — and both files were fmt-clean at HEAD, so the debt was entirely mine |
| `cargo fmt` reformatting unrelated code | none — 9 diff lines total, all inside my own added block |

## Not done

- No card, no merge, no install, no restart, no live-database write, no sandbox replacement.
- I did not verify the "all four groups, zero false positives" claim, and did not restate it
  as measured. It is unverifiable on the current database.
- No Flux verification of this branch; nothing names it.
