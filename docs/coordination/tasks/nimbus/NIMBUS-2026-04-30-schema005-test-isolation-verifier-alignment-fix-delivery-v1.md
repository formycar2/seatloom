# Delivery: Schema 005 Test Isolation + Verifier Alignment Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-05-06 |
| version | v1 |
| depends_on | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-fix-v1.md`, `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md` |
| tags | nimbus, infrastructure, postgres, schema005, tests, verifier, isolation |
| owner | Nimbus |

## 1. Lyra's Verified Problem Restated

Flux remote verification of commit `68a7b38` returned HOLD. The schema-005 integration tests were not safe under the verifier's accepted contract (`cargo test -p seatloom-core -- --include-ignored`):

- **Defect A** — `prompt_instance_tables_exist_and_empty_at_baseline()` asserted global zero-row state, which is only true before any mutating schema-005 test inserts fixture rows.
- **Defect B** — `prompt_action_append_and_list()` referenced `pi-test-001`, a row created by another test. Cross-test side-effect dependency.
- **Defect C** — Tests were neither self-contained nor split into isolated phases by the verifier.

## 2. Fix Direction Chosen

**Lyra's preferred Option A**: move the zero-row baseline proof out of the shared mutating test flow and into the verifier as SQL spot-checks immediately after seed 004.

This keeps Rust tests focused on write/read/idempotency behavior while letting the verifier own the only point in the pipeline where global zero-row state is observable.

## 3. Files Changed

- `scripts/verify-postgres-baseline.sh` — added new Step 4 (schema-005 zero-row baseline proof via SQL); renumbered subsequent steps (5/6/7 → reconcile / ingest / cargo test); updated header proof sequence.
- `crates/seatloom-core/tests/db_baseline_integration.rs`:
  - Removed `prompt_instance_tables_exist_and_empty_at_baseline()` (replaced by SQL spot-check above).
  - Made `prompt_action_append_and_list()` self-sufficient: it now creates its own parent prompt instance with the unique ID `pi-action-fixture-001` before appending its action and updates list/assertion calls accordingly.
  - Removed the "run after prompt_instance_write_read_round_trip" hint from the action-test `#[ignore]` reason.
  - Updated the schema-005 section comment to document the new isolation contract.

No schema SQL, seed SQL, or repository code was modified — the defects were entirely in the test fixtures and the verifier proof flow.

## 4. New Test Contract Properties

Each schema-005 test now satisfies:

| Property | Implementation |
|---|---|
| Order-independent | Each test owns its writes; no test reads another's fixtures. |
| Parallel-safe | Each test uses unique fixture IDs (`pi-test-001` vs `pi-action-fixture-001` vs `car-test-001/002`). |
| Self-contained | `prompt_action_append_and_list` creates its own parent `PromptInstanceRow` rather than relying on a sibling test. |
| Repeat-run-safe | The verifier's destructive `docker compose down -v` at Step 1 already guarantees a clean DB between runs. |

Zero-row baseline proof is now enforced by the verifier in Step 4 via a `psql` `DO $$ ... $$` block that `RAISE EXCEPTION` on any non-zero count for `prompt_instances`, `prompt_actions`, or `channel_action_receipts`. This runs before any mutating step (reconcile, integration tests).

## 5. Validation Commands and Results

```
$HOME/.cargo/bin/cargo fmt --all --check
=> exit 0 (PASS)

$HOME/.cargo/bin/cargo check -p seatloom-core
=> Finished `dev` profile (PASS)

$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings
=> Finished `dev` profile — 0 warnings (PASS)

$HOME/.cargo/bin/cargo test -p seatloom-core
=> 54 unit tests: ok
=> 27 DB integration tests: ignored (was 28 — global zero-row test moved to verifier)
=> 3 seed-consistency tests: ok
=> exit 0 (PASS)

bash -n scripts/verify-postgres-baseline.sh
=> SYNTAX OK (PASS)
```

`cargo test -- --include-ignored` was not run on this seat: Docker is not installed, so the ignored DB tests cannot connect. Flux's commit-pinned remote verification will exercise that path.

## 6. Verifier Step Numbering

Old → new:

| Old | New | Step |
|---|---|---|
| 1 | 1 | static cross-seed consistency |
| 2 | 2 | volume teardown + clean container start |
| 3 | 3 | schema apply (001–005) |
| 3 | 3 | seed apply (001–004) |
| — | **4** | **schema-005 zero-row baseline proof (new)** |
| 4 | 5 | bounded document reconcile |
| 5 | 6 | body-ingest helper health check |
| 6 | 7 | DB integration tests (`--include-ignored`) |

## 7. Exact Branch and Commit

- Branch: `track/infra-foundation`
- Commit: `1bb561a`

## 8. Reporting Format

```text
[Nimbus -> Lyra] Schema 005 Test Isolation + Verifier Alignment Fix
branch:
- track/infra-foundation
commit:
- 1bb561a
completed:
- removed prompt_instance_tables_exist_and_empty_at_baseline (moved to verifier SQL)
- made prompt_action_append_and_list self-sufficient with own parent fixture
- added Step 4 SQL zero-row baseline proof to verify-postgres-baseline.sh
- renumbered subsequent verifier steps
validation:
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => PASS
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => 54 + 3 pass, 27 ignored
- `bash -n scripts/verify-postgres-baseline.sh` => SYNTAX OK
blockers:
- none (Docker unavailable on this seat — Flux to perform commit-pinned remote double-run)
next action:
- wait for Flux commit-pinned reverification
artifact path(s):
- docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md
```
