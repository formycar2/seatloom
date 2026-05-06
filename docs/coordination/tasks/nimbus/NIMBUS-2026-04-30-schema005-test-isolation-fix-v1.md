# Task: Schema 005 Test Isolation + Verifier Alignment Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-schema005-test-isolation-fix-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| to | nimbus |
| priority | P0 |
| deadline | 2026-05-06 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md`, `docs/coordination/tasks/flux/FLUX-2026-04-30-prompt-channel-authority-remote-verification-delivery-v1.md`, `crates/seatloom-core/tests/db_baseline_integration.rs`, `scripts/verify-postgres-baseline.sh` |
| tags | nimbus, postgres, infra, tests, schema005, prompt, channel-action |
| owner | Nimbus |
| acceptance owner | Lyra |
| concurrency rule | One bounded infrastructure packet only. Stay within seatloom-core and the verifier script. Do not widen into UI or product logic. |

---

## Objective

Fix the real implementation defect exposed by Flux remote verification of commit `68a7b38`.

This is an infrastructure-only packet. Do not add business logic.

---

## Verified problem statement

Flux's HOLD is real, but the root cause is more precise than the initial summary:

1. `infra/postgres/seed/004_prompt_and_channel_action_seed.sql` is still a zero-row seed by file content. (This is intentional and must NOT change — honest policy.)
2. The failing behavior comes from **order-dependent / shared-state schema-005 integration tests** inside `crates/seatloom-core/tests/db_baseline_integration.rs`.
3. Step 6 of `scripts/verify-postgres-baseline.sh` runs `cargo test -p seatloom-core -- --include-ignored`, so all ignored DB tests execute against the same seeded database.
4. The current schema-005 test set is not safe under that execution model.

---

## Exact defects to close

### Defect A — baseline emptiness assertion is not isolated

`prompt_instance_tables_exist_and_empty_at_baseline()` asserts global zero-row state for prompt families.

That assertion is only valid **before any mutating schema-005 test inserts fixture rows**.

Under the current verifier contract, this test shares the same database with later prompt tests, so it cannot safely assume pristine global state unless the verifier runs it in a dedicated isolated phase.

### Defect B — prompt action test depends on another test's side effect

`prompt_action_append_and_list()` assumes prompt instance `pi-test-001` already exists.

That is an invalid test contract. Tests must not depend on another ignored test to create prerequisite rows.

The test must create its own parent prompt instance or use an in-test helper to guarantee the prerequisite exists.

### Defect C — schema-005 proof path is not order-independent under `--include-ignored`

The accepted verifier contract uses a single `--include-ignored` pass.

Therefore schema-005 tests must be one of:
- self-contained and order-independent, or
- split by the verifier into deterministic isolated phases.

Right now they are neither.

---

## Required fix direction

Close the issue in the narrowest honest way.

### Required outcome 1

Make `prompt_action_append_and_list()` self-sufficient.

At minimum:
- create the required parent `PromptInstanceRow` inside the same test before inserting `PromptActionRow`, or
- use a small shared helper inside the test file that creates an isolated prompt fixture for that test.

Do not rely on `prompt_instance_write_read_round_trip()` having run first.

### Required outcome 2

Make the zero-row baseline proof compatible with the verifier model.

Choose one of these acceptable approaches:

#### Option A (preferred)
- move zero-row baseline proof out of the shared mutating test flow
- prove zero-row baseline in `scripts/verify-postgres-baseline.sh` with explicit SQL spot-checks before running mutating DB tests
- keep Rust schema-005 tests focused on write/read/idempotency behavior only

#### Option B
- keep the Rust zero-row test, but change the verifier so that the zero-row test runs in a dedicated isolated step before any mutating schema-005 tests

Whichever option you choose, the final contract must be deterministic on repeat runs.

### Required outcome 3

Ensure schema-005 proof is safe under repeat-run verification.

That means:
- no hidden dependency on test order
- no dependency on parallel test execution ordering
- no false failure caused by prior fixture rows from sibling tests in the same verifier run

---

## Preferred implementation guidance

Preferred narrow path:
1. remove cross-test dependency from `prompt_action_append_and_list()`
2. move the zero-row baseline proof into the verifier script as SQL checks immediately after seed 004
3. keep the schema-005 Rust integration tests for:
   - prompt instance write/read round-trip
   - prompt action append/list
   - channel action receipt write/read/idempotency
4. if needed, use unique IDs per test and explicit local fixture setup/cleanup inside each test

---

## Non-goals

- do not add product/business mutation APIs
- do not change accepted data semantics unless strictly required by the test/verifier fix
- do not fabricate non-zero prompt/channel baseline seed rows

---

## Done definition

- schema-005 tests are order-independent
- no schema-005 test depends on another test's side effect
- zero-row baseline proof is deterministic and verifier-compatible
- `cargo test -p seatloom-core` clean on Nimbus seat
- `scripts/verify-postgres-baseline.sh` contract updated if needed
- packet committed on `track/infra-foundation`
- new commit hash reported to `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md`

---

## Reporting format

Reply with a delivery file at:
`docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md`

```text
[Nimbus -> Lyra] Schema 005 Test Isolation + Verifier Alignment Fix
branch:
- track/infra-foundation
commit:
- <new commit>
completed:
- ...
validation:
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => ...
- `$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored` => ...
- `bash -n scripts/verify-postgres-baseline.sh` => ...
blockers:
- none OR ...
next action:
- wait for Flux commit-pinned reverification
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md
```
