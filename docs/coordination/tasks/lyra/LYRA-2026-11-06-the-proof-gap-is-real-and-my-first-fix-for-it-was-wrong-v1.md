# Standing-duty verification of the fix I proposed last round, tested in both directions rather than asserted: **the gap is real and the fix closes it.** With the guard made unable to refuse, Nimbus's proof script prints `ALL POINTS PASS` and exits 0 while the core suite goes 3 passed / 2 failed. **Adding one point that drives the real suite instead of restating it turns the proof exit 0 into exit 101.** And my first attempt at that fix was the wrong one, which is the more useful half of this round: I added a source-text assertion on the branch, and it passed under the mutation, because the string was still there and only its input had changed.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-06-the-proof-gap-is-real-and-my-first-fix-for-it-was-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-11-06 |
| version | v1 |
| tags | standing-duty, verification, measured, mutation |

## The gap, re-measured on the branch as it stands

**On `packet/nimbus/deletion-guard-only` @ `86a65a5`, with the guard mutated so it can never refuse:**

| instrument | result |
|---|---|
| `proof/accept_deletion_guard_proof.sh` | **exit 0**, `ALL POINTS PASS` |
| `cargo test -p seatloom-core --test accept_deletion_guard` | **FAILED, 3 passed / 2 failed** |

**And the script's own header says the reason**: *a proof that calls a copy proves the copy.* It generates a test
file, and that generated test does points 1–3 against the core helper and points 4–5 against
`include_str!` of `acceptance_cmds.rs`. **Nothing drives the command's use of the answer.**

## My first fix was wrong, and the way it was wrong is the point

**I added a sixth point to the generated test:**

```rust
let uses = src.contains("if !undeleted.is_empty()");
assert!(uses, "the accept command must branch on the guard's answer");
```

**Under the mutation it passed.** Because the mutation replaced the *value* assigned to `undeleted`, not the branch on
it — the string `if !undeleted.is_empty()` was still in the source, and the guard was still dead.

**So my fix was the same shape as the gap I was fixing**: an assertion on text, which cannot see that the text now
operates on an empty vector. **I would have shipped a sixth green point over a dead guard.**

**That is the fourth time this week I have written an assertion that could not fail** — after the empty-search-as-finding
instances and the two invalid mutations on 11-06 05:41. **The pattern is consistent: I reach for a string check because
it is cheap, and it is cheap because it is weak.**

## The fix that works, verified in both directions

**One point that runs the real suite and fails the proof if it is red:**

```bash
echo "  PROOF 6 wiring, driven by the real suite:"
cargo test -p seatloom-core --test accept_deletion_guard -- --nocapture 2>&1 \
  | grep -E "^test |test result" | sed 's/^/    /'
cargo test -p seatloom-core --test accept_deletion_guard >/dev/null 2>&1 || {
  echo "PROOF FAILED: the wiring suite is red" >&2; exit 1; }
```

| | with the mutation | without |
|---|---|---|
| proof exit code | **101** | **0** |
| output | `PROOF FAILED: the wiring suite is red` | six `PROOF` lines, `ALL POINTS PASS` |

**So it fails in the direction that matters and passes in the direction that must keep working.** Restored byte-identical
afterwards, and the clone removed.

## Why this is the right shape and not just a working patch

**Because the suite already drives the contract.** `the_guard_is_wired_into_the_accept_command_before_the_merge`
asserts the call exists and precedes the merge; the behavioural tests assert a destructive branch is refused and a safe
one is not. My mutation failed two of them. **The proof script's job is not to restate that — it is to run it, so that
"ALL POINTS PASS" means the wiring was exercised, not that six strings were found.**

**And the generalisation is the thing worth recording**: *a proof that reads source proves the source; a proof that
calls a copy proves the copy; only a proof that drives the real thing proves the real thing.* Nimbus wrote the first
half of that himself. The second half is what this round adds.

## What the operator can and cannot do, unchanged

**Still cannot open a card, correctly.** No verdict in the ledger pins
`payload->>'branch' = packet/nimbus/deletion-guard-only`; all eight pin other branches. **The branch is ready, its
verdict does not exist, and the equality check I adopted on 11-06 06:13 is what establishes that.**

## What I did not do

I did not edit the branch, the proof script on the branch, or any script in the repository — both the mutation and the
fix were applied in a throwaway clone and the clone was removed. I did not merge, push, accept, cancel, or re-open any
card, restart any application, or write to the live database. My measurements were reads plus one mutation and one fix
in a scratch clone: the proof script's generated test read in full, the proof run at baseline and under the mutation,
the core suite at baseline and under the mutation, the weak fix applied and measured, the real fix applied and measured
in both directions, and the branch-equality check across all eight verdicts. The ledger check is exit 0, `seats` is 10,
and `canonical_events` is **114**.
