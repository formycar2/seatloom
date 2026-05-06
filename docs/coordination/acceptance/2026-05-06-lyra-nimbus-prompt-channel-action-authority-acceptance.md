# Acceptance: Nimbus Prompt + Channel Action Authority (Schema 005)

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-05-06-nimbus-prompt-channel-action-authority-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-05-06 |
| version | v1 |
| target | `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md` |
| verdict | PASS |
| tags | acceptance, nimbus, flux, postgres, prompt, channel-action, schema005, test-isolation, infrastructure |

## Verdict

**PASS**

Lyra accepts the prompt + channel action authority (schema 005) packet as scope-complete and remotely verified.

The schema-005 test isolation HOLD is now closed on the exact remotely verified commit:
- branch: `track/infra-foundation`
- final verified commit: `1bb561ab7872ed69b5dd7d40335ea8c689a5563a`
- verification seat: sponsor-provided remote workspace (`buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com`)
- proof path: `bash scripts/verify-postgres-baseline.sh` run twice consecutively with no manual cleanup between runs; `cargo test -p seatloom-core -- --include-ignored` passes cleanly

The HOLD issued by Flux on commit `68a7b38` was real and correctly identified three test isolation defects. Nimbus fixed all three without widening scope. Flux then re-ran the commit-pinned remote proof and returned PASS on all acceptance signals.

## Scope Reviewed

- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-fix-v1.md`
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md`
- `docs/coordination/tasks/lyra/LYRA-2026-04-30-flux-prompt-channel-authority-remote-verification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-prompt-channel-authority-reverification-v1.md`
- `docs/coordination/tasks/flux/FLUX-2026-04-30-prompt-channel-authority-reverification-delivery-v1.md`
- `infra/postgres/schema/005_prompt_and_channel_action_authority.sql`
- `infra/postgres/seed/004_prompt_and_channel_action_seed.sql`
- `crates/seatloom-core/tests/db_baseline_integration.rs`
- `scripts/verify-postgres-baseline.sh`

## Coverage Matrix

| Requirement slice | Evidence paths | Result | Notes |
|---|---|---|---|
| Schema 005 tables exist and are structurally correct | Flux SQL spot-checks: `to_regclass(...)` non-null for all three families | PASS | `prompt_instances`, `prompt_actions`, `channel_action_receipts` all present. |
| `channel_action_receipts.idempotency_key` uniqueness enforced | Flux SQL spot-check: unique index present | PASS | Idempotency constraint confirmed on remote seat. |
| Zero-row seed is honest and policy-correct | Verifier Step 4 SQL proof, Flux run 1 + run 2 output | PASS | `seed/004_prompt_and_channel_action_seed.sql` is intentionally zero-row; confirmed by SQL DO block before any mutating test. |
| Schema-005 Rust tests are order-independent | Nimbus fix delivery, Flux `--include-ignored` pass | PASS | All three remaining schema-005 tests use unique fixture IDs and own their own writes. |
| No cross-test dependency in `prompt_action_append_and_list` | Nimbus fix delivery (Defect B), Flux PASS | PASS | Test now creates its own parent `PromptInstanceRow` with ID `pi-action-fixture-001`. |
| Zero-row baseline proof is verifier-owned and runs before mutating tests | Nimbus fix delivery (Defect A + C), verifier Step 4 | PASS | SQL `DO $$ ... $$` block runs at Step 4, before reconcile and DB integration tests. |
| Verifier is repeat-run safe (double-run without manual cleanup) | Flux run 1 + run 2 both PASS | PASS | Both runs exited 0 with `=== All checks passed ===`. |
| `cargo test -p seatloom-core -- --include-ignored` passes (the HOLD trigger) | Flux validation: 27 ignored DB tests passed | PASS | Was 28 before; removal of global zero-row test is correct and intentional. |
| Prior baseline tables remain non-zero and healthy | Flux SQL spot-checks | PASS | `reconcile_runs`, `document_versions`, `checkpoints`, `handoff_receipts`, `pipeline_runs`, `review_threads`, `review_comments` all non-zero. |
| Packet stayed within infra-only scope | Nimbus deliveries | PASS | No schema semantics change, no seed fabrication, no UI/business logic touched. |

## Findings

### Resolved blockers

1. **Defect A closed — zero-row baseline proof moved to verifier**
   - Prior issue: `prompt_instance_tables_exist_and_empty_at_baseline()` asserted global zero-row state inside the shared mutating test flow; this assertion was only valid before any other schema-005 test inserted fixture rows.
   - Fix: test removed from Rust; verifier Step 4 now runs a SQL `DO $$ ... $$` block that `RAISE EXCEPTION`s on any non-zero count in `prompt_instances`, `prompt_actions`, or `channel_action_receipts`, before any mutating step.
   - Result: zero-row baseline proof is now deterministic and verifier-owned.

2. **Defect B closed — `prompt_action_append_and_list` is self-sufficient**
   - Prior issue: test assumed `pi-test-001` row was created by a sibling test (`prompt_instance_write_read_round_trip`), making it order-dependent.
   - Fix: test now creates its own parent `PromptInstanceRow` with unique ID `pi-action-fixture-001` at the start of the test.
   - Result: test is independently runnable in any order.

3. **Defect C closed — all schema-005 tests are order-independent under `--include-ignored`**
   - Prior issue: the three schema-005 tests collectively relied on shared state and implicit ordering.
   - Fix: unique fixture IDs per test (`pi-test-001`, `pi-action-fixture-001`, `car-test-001`, `car-test-002`); each test owns its writes.
   - Result: verifier Step 7 (`cargo test -p seatloom-core -- --include-ignored`) now passes clean on repeat runs.

### Non-blocking notes

1. The sponsor remote workspace is the accepted Docker-capable proof seat for this and future verifier-path packets.
2. The verifier now has a full 8-step contract (Steps 1–7 with Step 4 as the schema-005 SQL baseline proof) and is stable enough to serve as the gate for future schema packets.
3. The prompt/channel-action tables are correctly zero-row at baseline — this is intentional policy and is now enforced by the verifier rather than a fragile shared test.

## Required Fixes

None. All HOLD defects resolved.

## Go / No-Go Recommendation

- **Close schema-005 prompt + channel action authority as final PASS:** **GO**
- **Advance to the next infrastructure or product layer packet:** **GO**
- **Reopen or widen this packet:** **NO-GO**
- **Fabricate non-zero prompt/channel seed rows:** **NO-GO**

## Evidence Paths

- Initial delivery (schema 005 implementation): `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md`
- Test isolation fix delivery: `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-schema005-test-isolation-verifier-alignment-fix-delivery-v1.md`
- Flux PASS delivery: `docs/coordination/tasks/flux/FLUX-2026-04-30-prompt-channel-authority-reverification-delivery-v1.md`
- Verifier script: `scripts/verify-postgres-baseline.sh`
- Integration tests: `crates/seatloom-core/tests/db_baseline_integration.rs`
- Remote evidence: `.local/evidence/2026-04-30-prompt-channel-authority-reverification-v1/`
