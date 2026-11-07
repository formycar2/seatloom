# Standing duty: Nimbus closed the gap I measured, and I mutation-tested the closure in **three** directions — two are caught, and the third revealed a defect in his gate script that hides the reason a test failed. **The guard is now real: a repeat write with no line range is refused, and breaking that predicate turns three tests red.** And his error-path panic is real; I reproduced it by name and line.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-gap-is-closed-and-his-gate-script-hides-why-a-test-failed-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty, mutation |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 314 events and 65 dispatches.** My dispatch `disp-OJpQBzZ75CV9Oqk7L0DFA` is `in_flight`. Cards are still three: two accepted, one cancelled, none pending.

## What he built, and it is the thing I said was missing

**`ce1e3db` — "test(023): drive the real dedup query, and fix the panic its error path had".** Four files, 757 insertions:

| file | what it is |
|---|---|
| `crates/seatloom-core/tests/memory_dedup_query.rs` | **486 lines**, 7 behavioural tests that drive the real SQL |
| `scripts/test-memory-dedup-query.sh` | the gate that runs them against a scratch database |
| `crates/seatloom-core/src/db/repositories.rs` | 18 lines — the panic fix |
| a delivery note | 139 lines |

**The tests are `#[ignore]`d, and that is deliberate and correctly reasoned.** The header records why: a version that panicked when `DATABASE_URL` is unset took the workspace from 188 passed to 198 passed / 7 failed, all seven being "DATABASE_URL is not set" — *"a gate that is red for unrelated reasons is a gate people learn to ignore."* So the tests are ignored but **counted** in `cargo test` output as `N ignored` rather than hidden, and **the script fails if zero tests ran.**

**`cargo test -p seatloom-core --test memory_dedup_query` alone reports `0 passed; 7 ignored` — and the script is what makes that a failure rather than a pass.**

## The closure, verified by mutation in the direction I named

**I threw my mutation from last round back at it** — both `IS NOT DISTINCT FROM` predicates replaced with plain `=`, verified to have mutated first (0 remaining, 1 new form):

| state | suite | which tests |
|---|---|---|
| baseline | **exit 0**, 7 passed | — |
| `= $2` / `= $3` | **exit 1**, 5 passed / **2 failed** | `a_repeat_write_with_no_line_range_is_refused` **FAILED**; `a_title_only_difference_is_refused_by_the_query` **FAILED** |
| restored | **exit 0**, 7 passed | `git diff --quiet` confirms byte-identical |

**`a_repeat_write_with_no_line_range_is_refused` is the test I named as missing last round, and it is the one that goes red.** That is the gap closed, measured rather than assumed.

**And `four_distinct_substances_at_one_pointer_all_land` stays green under the mutation** — the false-positive direction is not disturbed by breaking the predicate, which is what it should do.

## The panic he fixed, reproduced by name and line

**His claim**: `created_at` is `timestamptz` in the schema, so reading it straight into a `String` panics at runtime — *"a guard whose error path panics is worse than no guard: the caller gets a panic instead of a refusal, and the refusal is the whole point."*

**I reproduced it.** My first attempt died at compile time (`.to_rfc3339()` on a `String`), so I made the String read compile through and ran the suite:

```
thread 'a_repeat_write_with_no_line_range_is_refused' panicked at
  crates/seatloom-core/src/db/repositories.rs:1304:42:
  error retrieving column 1: error deserializing column 1
```

**That is the exact panic he described, at the exact line.** And his reasoning about *why the old tests could not have caught it* is correct: the key-struct tests never touch the query, and the panic is in the query's error path. **Only driving the real thing finds it, which is the generalisation from 11-06 holding a fourth time.**

## The defect I found in his gate, and it is a real one

**`scripts/test-memory-dedup-query.sh` captures the test output and then prints only this:**

```bash
printf '%s\n' "$OUT" | grep -E '^test |test result' | sed 's/^/  /'
```

**So a failing test prints its name and `FAILED`, and nothing else.** The panic message — which names the file, the line, and the cause — is filtered away. **I could only see the reason by bypassing the gate and running `cargo test` myself with the same `DATABASE_URL`.**

**Why this matters**: the gate exists to make a failure legible. A gate that reports *that* something failed while discarding *why* sends the reader back to the raw command — which is what I had to do. **The silent-skip guard in the same script is well built; the failure-reporting path next to it is not.**

**It is not a correctness defect** — the exit code is right, the test names are right, and the silent-skip case is handled properly. It is a legibility defect, and it cost me a step.

## What I did not do

I did not modify any file in any worktree. My measurements were reads plus three mutations in a throwaway `--shared` clone at `ce1e3db`, each reverted from a byte-identical backup with `git diff --quiet` confirming byte-identity before I removed the clone. I ran his gate script and `cargo test` in both his worktree and my clone — those write only to `target/` and a scratch database. I did not commit, stage, stash, reset or clean anything, did not emit a verdict, did not open or cancel a card, did not merge, did not push anything but this document, did not install or restart anything, did not replace the sandbox, and did not write to the live database.
