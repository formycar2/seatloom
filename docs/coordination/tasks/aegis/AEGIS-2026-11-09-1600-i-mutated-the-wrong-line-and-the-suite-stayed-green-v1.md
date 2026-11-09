| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1600-i-mutated-the-wrong-line-and-the-suite-stayed-green |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, mutation-testing, own-error, not-wired, verification |

# I mutated the wrong line and the suite stayed green — which is the exact trap Flux had just documented

## What happened

Flux returned **FAIL** on Nimbus's `db error` message fix, with the sharpest finding of the week:
the renderer was written, tested, and **left one function call away from the caller**.

`db_chain` walks the source chain and emits `sqlstate=… constraint=… table=…`. It had exactly one
caller in the workspace — **the test that asserts it**. The path a caller actually takes went through
`From<DbError>`'s fallback arm, which called `other.to_string()`, which renders as the near-useless
string `db error`. So the refusal still refused, and the caller still received `db error`.

Flux drove the real `remember` tool against both real constraints and measured the defect unchanged.
Then, in an isolated worktree, it wired the one line and measured both constraints becoming legible —
and reverted.

Nimbus had already fixed it at `41da35b` by the time I looked: the wiring is in, and the test now
asserts on `rejection.to_string()` — the caller's path — rather than on `db_chain(&err)`.

## My mutation test, and why the first one was worthless

A test that passes on both the fixed and the unfixed code proves nothing. So I reverted the wiring and
ran the suite.

**It passed. 5/5.**

The reason: `memory.rs` has **two** arms reading `other => MemoryRejection::Db(...)` — one at line 321
in `From<DispatchRejection>`, one at line 416 in `From<DbError>`. My `sed` replaced the **first**
occurrence. The arm the caller takes was untouched, so the suite was correctly green.

**I had just read Flux's verdict saying "a test that only proves the renderer works proves the
decorative half", and then ran a mutation that mutated nothing the test measures.** That is the same
error one level down: a check that cannot fail, built while reading a document about checks that cannot
fail.

## The mutation that actually proves it

Mutating **line 416 only** — the arm `?` uses in `remember`/`promote`/`forget`:

```
test memory::tests::a_refused_write_names_the_constraint_and_the_table ... FAILED
  the caller-visible message must name the constraint.
  Got: memory tool rejected: postgres error: db error
test result: FAILED. 4 passed; 1 failed
```

**That is the original defect, reproduced by removing the wiring.** So the test is load-bearing: it
fails when the fix is absent and passes when it is present. Restored from a byte-identical backup,
`git diff` is empty, and the suite is 5/5 again.

## What is now proven, and by whom

| | measured by | result |
|---|---|---|
| the renderer names the constraint | Flux, and Nimbus's own test | yes |
| the caller's path reaches the renderer | Flux's FAIL, then my mutation at line 416 | yes — and it fails without it |
| a refused write still refuses, 0 rows | Flux, on the real tool | yes |
| a valid write still succeeds | Flux, and the suite | yes |
| the suite refuses to report green when nothing ran | Flux, and its own comment | yes, correctly built |
| the live database was never written | Flux, measured afterwards: 64 rows, zero matching | confirmed |

## The pattern, stated once

This is the fourth instance of the same shape in two days: a capability that exists, is verified, and
is not connected.

- `MUST_HAVE` at `6dd68c5` — a required field silently discarded.
- the two-`pass` guard at `0a7f1a7` — a second condition that never ran.
- `db_chain` at `1d2cabb` — a renderer with one caller, its own test.
- my mutation at line 321 — a probe that mutated a sibling and reported the fix as proven.

**A green suite proves the renderer works. It does not prove the renderer is used.** The only thing
that closes that gap is a mutation on the path the caller takes, and a mutation on the wrong line is
indistinguishable from a passing test.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Nimbus's branch `packet/nimbus/candidate-final` at `41da35b`, worktree clean, 13 ahead of track.
- Salvage: 2 live patches round-tripping, 7 superseded. Nimbus's tree is clean so nothing is at risk.
- Push: local and remote both at `94dcd1c`, confirmed.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **The card on `packet/vela/washing-pipeline`** is still waiting, with the corrected 63-of-90 figure.
- **`l2_ref_kind=document` while `documents` has zero rows.** Every such call is refused by the FK and
  today the caller can at least be told which constraint. Whether to populate the table or refuse the
  kind early is your call, and I have not made it.
- **The sandbox rebuild**, still the only thing between seven verified deliverables and you.
