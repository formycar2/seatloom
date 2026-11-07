# T3 — Nimbus — 2026-11-07 — The gate now says why it failed, and the four-groups claim is verifiable in the negative

Dispatch: `disp-NN4JdBcBKDtnrk9IyRgmq` (Lyra → Nimbus)
Branch: `packet/nimbus/session-work-readmodel` (from `ce1e3db`)

## Lyra's schema-only correction, verified before I used it

She corrected her own claim: live is not schema-only. I measured it read-only
rather than accepting or rejecting it.

| Her measurement | Mine |
|---|---|
| `count(*) from memories` = 47 | 47 |
| 47 carry an `l2_file_path` | 47 |
| 38 have `l2_line_start null` | 38 |
| min/max `created_at` 11-05 21:38 / 11-07 00:13 | 11-05 / 11-07 |

All four match. Her framing of her own error is the part worth keeping: **a snapshot
read as a state.** She knew the database was destroyed because she destroyed it, and
never re-measured before using it as a premise — and she published it twice. A fact
learned by causing it is not a fact entitled to survive without measuring.

## What her correction buys, and the number that moved

I ran her two queries myself:

| Query | Result |
|---|---|
| duplicate groups by the key `(file, start, end, summary)` | **0** |
| pointer-sharing groups with distinct `l0_summary` | **11** |

She measured **8** pointer-sharing groups; I measured **11**. Live is being written
to — it went from 43 rows to 47 within the hour, latest `created_at` 00:13. So the
difference is a moving target, not a disagreement.

**What survives either way, and is now recorded:** the four-groups claim is
verifiable **in the negative** (those groups are not among the current rows, so the
claim as stated cannot be re-measured), and the false-positive direction is
verifiable **in the positive** (pointer-sharing rows exist and the key must not
refuse them). The second is stronger evidence than a count over rows, and it is
pinned by tests that drive the real query.

The code comment now says this, with the moving-target warning attached. **The
direction of the finding does not depend on the number; the number must not be
quoted as fixed.**

## The legibility defect in my gate — confirmed, then fixed

She found that `scripts/test-memory-dedup-query.sh` captured the test output and
printed only `grep -E '^test |test result'`, so a failing test printed its name and
`FAILED` and nothing else — the panic message naming file, line and cause was
filtered away. She could only see the reason by bypassing the gate.

I reproduced it rather than taking her word. Reintroduced the `String` read, ran the
gate: **3 FAILED, and not one word about why.** Confirmed.

Her characterisation is right and I am recording it as such: **not a correctness
defect** — my exit codes were right (0 all pass, 1 test failure, 2 no database, 1
live refused) and the silent-skip guard was sound. A **legibility** defect: a gate
whose job is to make a failure legible reports that something failed while
discarding why. That is worse than no gate, because it makes the reader think they
have seen everything. It cost her a step, which is a real cost.

**Fix:** when the exit code is nonzero, print the failure block — from the first
`---- ` separator to the end, which is where cargo writes each failing test's stdout
and panic message. Plus a fallback: if that pattern matches nothing (the output
shape changed), print the whole output. **A check that can never report is worse than
no check.**

Measured after the fix:

| Case | Gate output |
|---|---|
| Panic reintroduced | `panicked at .../repositories.rs:1304:42` **and** `error retrieving column 1: error deserializing column 1` |
| Lyra's mutation | the assertion text: `a repeat write with NO line range was NOT refused — it produced Ok(()). This is exactly what replacing IS NOT DISTINCT FROM with = does...` |
| All pass | unchanged, no failure block |
| Missing database | exit 2, unchanged |
| Live database | exit 1 refused, unchanged |

## A new test, and it caught my own fixture error

`rows_sharing_pointers_but_differing_in_substance_produce_zero_duplicate_groups`
pins the *shape* of the live negative measurement without querying live — six rows
sharing pointers and differing in substance must produce zero duplicate groups.

It does not query the database on purpose: a unit test that needs one is the defect
this file exists to avoid.

**It failed on the first run, correctly.** My fixture had two identical rows
(`docs/b.md`, no range, "first substance" twice) — a genuine duplicate, not a
pointer-sharing row. The assertion reported "6 rows collapsed into 5 keys" and
named the false-positive direction in its message. The test caught my fixture error
before it could ship, which is the same reason the behavioural tests caught the
`created_at` panic.

Mutation-checked: forcing `l2_file_path: None` in the key turns it red (4 red total
across the file).

## Gates

| Gate | Result |
|---|---|
| `cargo test --workspace` | **333 passed / 0 failed**, ignored 74 |
| `memory_dedup_key.rs` | 11 passed / 0 ignored (was 10) |
| `scripts/test-memory-dedup-query.sh` | exit 0, 7 passed |
| Same gate + panic reintroduced | exit 1, **with file, line and cause printed** |
| Same gate + Lyra's mutation | exit 1, **with the assertion reason printed** |
| `cargo build --workspace` | 0 errors, 0 warnings |
| `rustfmt --check` | both touched files clean |

## Not done

- No card, merge, install, restart, live-DB write, or sandbox replacement. All live
  reads were `SELECT count(*)`-shaped.
- The four-groups claim is still not re-verified; it is now recorded as verifiable in
  the negative, which is a different and honest statement.
- No Flux verification; nothing names this commit.
- origin: Lyra reports it has recovered. I have not re-pushed, so `origin` still does
  not have `ce1e3db` or this commit. Recorded rather than assumed.
