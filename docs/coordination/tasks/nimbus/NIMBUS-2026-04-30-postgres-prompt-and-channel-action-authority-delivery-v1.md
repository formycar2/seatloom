# Delivery: PostgreSQL Prompt + Channel Action Authority

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-v1` |
| tags | nimbus, infrastructure, postgres, prompt, channel-action, schema, repositories, tests |
| owner | Nimbus |

## 1. Packet Mission Restated

Add prompt lifecycle truth and generic channel-action receipts as first-class PostgreSQL authority via:

- Schema 005: three new table families (`prompt_instances`, `prompt_actions`, `channel_action_receipts`)
- Rust row structs and repository methods for all three families
- Integration tests (ignored) covering write/read round trips and the idempotency constraint
- Verifier extended to apply schema 005 and seed 004
- Source-map v4 with zero-row seed rationale documented

## 2. Files Changed

### New Files
- `infra/postgres/schema/005_prompt_and_channel_action_authority.sql`
- `infra/postgres/seed/004_prompt_and_channel_action_seed.sql` (zero-row, honest policy)
- `docs/coordination/tasks/nimbus/NIMBUS-2026-04-30-postgres-prompt-and-channel-action-authority-delivery-v1.md` (this file)

### Modified Files
- `.seatloom/bootstrap/source-map.yaml` — updated to v4; added `prompt_and_channel_action_families` section
- `crates/seatloom-core/src/db/models.rs` — added `PromptInstanceRow`, `PromptActionRow`, `ChannelActionReceiptRow`
- `crates/seatloom-core/src/db/repositories.rs` — added 10 write/read methods and 3 row-mapping helpers
- `crates/seatloom-core/tests/db_baseline_integration.rs` — added 4 new `#[ignore]` integration tests
- `scripts/verify-postgres-baseline.sh` — schema loop extended to 001–005; seed loop extended to 001–004

## 3. Schema Design

### `prompt_instances`
Tracks the lifecycle of a prompt from detection to resolution. Key design choices:
- `status` CHECK: `active | resolved | stopped | expired | superseded`
- `prompt_kind` CHECK: `deterministic | wizard_menu | freeform | sensitive`
- `prompt_policy` CHECK: `auto_allowed | needs_approval | human_required`
- Budget tracking fields: `assist_max_steps`, `assist_max_tokens`, `assist_steps_used`, `assist_tokens_used`
- `result_event_id` FK → `canonical_events` (nullable — set when resolved)
- 5 indexes on project_id, session_id, status, prompt_kind, detected_at

### `prompt_actions`
Append-only audit log of actions taken against a prompt. Key design choices:
- `action_kind` CHECK: `approve | human_takeover | supervisor_assist | stop | input_injected | auto_completed`
- `source_channel` CHECK: `desktop | mobile | supervisor | system`
- `result_status` CHECK: `applied | rejected | stopped | conflict`
- FK `prompt_id → prompt_instances(id)`
- 3 indexes on prompt_id, action_kind, created_at

### `channel_action_receipts`
Generic cross-channel action receipt with deduplication. Key design choices:
- `target_kind` CHECK: `workitem | handoff | review_thread | prompt | session | artifact | document | project`
- `action_kind` CHECK: `approve | reject | escalate | reserve_desktop_takeover | return | comment_submit | stop`
- `source_channel` CHECK: `desktop | mobile | supervisor | system`
- `receipt_status` CHECK: `applied | rejected | conflicted | noop`
- `idempotency_key TEXT NOT NULL UNIQUE` — DB-level duplicate action prevention
- Optimistic write guard: `expected_revision INT`, `applied_revision INT`
- 5 indexes on project_id, target, action_kind, source_channel, idempotency_key

All tables use `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS`.

## 4. Repository Methods Added

In `SeatloomDb` (`crates/seatloom-core/src/db/repositories.rs`):

| Method | Description |
|---|---|
| `create_prompt_instance` | INSERT all 20 fields |
| `list_prompt_instances_for_session` | SELECT by session_id |
| `list_active_prompt_instances` | SELECT by project_id WHERE status='active' |
| `get_prompt_instance` | SELECT by id |
| `append_prompt_action` | INSERT 11 fields |
| `list_prompt_actions_for_prompt` | SELECT by prompt_id ORDER BY created_at ASC |
| `create_channel_action_receipt` | INSERT 16 fields |
| `list_channel_action_receipts_for_target` | SELECT by target_kind + target_id |
| `list_channel_action_receipts` | SELECT with optional source_channel + receipt_status filters |
| `get_channel_action_receipt` | SELECT by id |

Row-mapping helpers: `row_to_prompt_instance`, `row_to_prompt_action`, `row_to_channel_action_receipt`.

## 5. Integration Tests Added

All four new tests are `#[ignore]` (require live PostgreSQL with schema 005 applied):

- `prompt_instance_tables_exist_and_empty_at_baseline` — verifies COUNT(*) = 0 on both tables
- `prompt_instance_write_read_round_trip` — creates fixture PromptInstanceRow, inserts, reads back, verifies all fields including `available_actions` Vec
- `prompt_action_append_and_list` — appends a PromptActionRow, reads back via `list_prompt_actions_for_prompt`
- `channel_action_receipt_write_read_and_idempotency` — creates receipt, verifies by-target list, mobile-filter list, and proves duplicate `idempotency_key` is rejected with a DB error

## 6. Seed Policy

`004_prompt_and_channel_action_seed.sql` is intentionally zero-row. Prompt instances and channel action receipts are runtime artifacts: they record actions that have actually occurred. Seeding fabricated rows without a real source event would misrepresent the baseline state. Source-map.yaml v4 documents this rationale under `prompt_and_channel_action_families`.

## 7. Validation Commands and Results

```
$HOME/.cargo/bin/cargo fmt --all --check
=> exit 0 (PASS)

$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings
=> Finished `dev` profile — 0 warnings (PASS)

$HOME/.cargo/bin/cargo test -p seatloom-core
=> 54 unit tests: ok
=> 3 seed-consistency tests: ok
=> 28 DB integration tests: ignored (no Docker on this seat) (PASS)

bash -n scripts/verify-postgres-baseline.sh
=> SYNTAX OK (PASS)
```

## 8. Docker Availability Note

Docker is not installed on this seat (darwin, no Docker daemon). End-to-end proof cannot be run locally. Flux should perform the remote double-run proof against the exact commit hash below.

## 9. Exact Branch and Commit

- Branch: `track/infra-foundation`
- Commit: (reported after push)
