# Delivery: P0 Step 2 — Write API (A1/G1): `append(event) -> projections`

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | NIMBUS-2026-09-30-p0-write-api-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-09-30 |
| version | v1 |
| to | flux (Layer A verify), lyra (acceptance) |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-p0-write-api-v1.md` |
| aegis_go | `docs/coordination/tasks/lyra/AEGIS-2026-09-30-p0-step2-write-api-v1.md` |
| base branch | `track/infra-foundation` @ `29adefd` |
| delivery branch | `packet/nimbus/NIMBUS-2026-09-30-p0-write-api-v1` |
| delivery commit SHA (code) | **`f5a3894`** — gate Layer A against this |
| delivery branch tip | the commit carrying this document, parent `f5a3894`. Resolve with `git rev-parse packet/nimbus/NIMBUS-2026-09-30-p0-write-api-v1`. |
| build location | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus` (the Nimbus worktree) |
| scratch database | `seatloom_s2_nimbus` — the live `seatloom` DB was never written |
| milestone | P0 capture loop — write side (UM1) |
| tags | nimbus, p0, write-api, a1, event-sourcing, um1, delivery |

---

## 1. Summary — 5 files, +1937/−0

```
$ git show --stat f5a3894
commit f5a3894f167a3ffbb37c88bafed5ea07a47cd6fb
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Wed Sep 30 22:47:38 2026 +0800

    feat(db): P0 step 2 write API — append(event) -> projections
    
    One write primitive. Every domain mutation is an event append PLUS its
    projection update in ONE transaction; on failure both roll back. Nothing
    outside db::write_api writes a projection table — apply_projection and
    replay_workitem_event are private to the module, so the only way to change
    `workitems` is to append an event.
    
    Honours all four obligations 010 left to step 2:
      - schema_version set EXPLICITLY on every append; the INSERT names the
        column, so dropping 010:87's DEFAULT 1 cannot change behaviour here.
      - event_object_refs.role set EXPLICITLY on every ref (010:220).
      - EXACTLY ONE subject: 010:332 gives at-most-one, EventDraft::validate
        gives presence. Zero subjects is a typed WriteError::NoSubject raised
        before a connection is taken.
      - Hand-written FromStr/Display for EventType (12), RefRole (3) and
        RefType (9); no serde derive at the TEXT boundary. ArtifactProduced is
        absent and a test fails if it is re-added.
    
    Live append and ledger replay share one implementation of "what this event
    does to the projection", so a rebuild cannot drift from a write.
    
    95 lib tests (18 new). Behavioural proofs in
    examples/p0_write_api_proof.rs, run against a scratch database.
    
    Contract deviation, flagged: RefType includes Delegation, which the packet
    §3 table omits. 010's merged CHECK admits it because `delegation` is in
    committed seed data (ev-020/ev-021); a domain without it could not read
    existing history.
    
    Packet: docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-p0-write-api-v1.md
    
    Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>

 .../seatloom-core/examples/p0_write_api_proof.rs   | 645 +++++++++++++++
 crates/seatloom-core/src/db/event_domain.rs        | 374 +++++++++
 crates/seatloom-core/src/db/mod.rs                 |   6 +
 crates/seatloom-core/src/db/repositories.rs        |   7 +
 crates/seatloom-core/src/db/write_api.rs           | 905 +++++++++++++++++++++
 5 files changed, 1937 insertions(+)
```

Zero `.tsx`, zero `.sql`, zero files under `ui/`, zero under `infra/`.

`SeatloomDb::append_event(&EventDraft)` is the single write primitive. It appends the event, writes
its object refs, and applies the projection **in one transaction**. `apply_projection` and
`replay_workitem_event` are **private to `db::write_api`** — the module system, not a convention,
is what stops a repository method from mutating a projection. Reads are untouched and still come
off the projections.

Live append and ledger replay call **the same** `replay_workitem_event`, so "what this event does
to the projection" has exactly one implementation and a rebuild cannot drift from a write.

---

## 2. §3 value-domain contract — reproduced verbatim

| Field | Rust | Serialized domain | PG column | Nullable |
|---|---|---|---|---|
| `event_type` | enum, hand-written `FromStr`/`Display` | `SessionStarted`, `WorkItemCreated`, `WorkItemStatusChanged`, `ArtifactCreated`, `HandoffSent`, `HandoffAccepted`, `HandoffCompleted`, `ReviewVerdictIssued`, `SeatDelegationIssued`, `SeatDelegationClosed`, `SupervisorMessage`, `SeatResponse` | `TEXT` | no |
| `schema_version` | `i32` | integer ≥ 1, per `event_type`, starts at 1 | `INT` | no — **set explicitly** |
| `role` | enum, hand-written | `subject`, `actor`, `context` | `TEXT` | no — **set explicitly** |
| `ref_type` | enum, hand-written | `workitem`, `session`, `handoff`, `artifact`, `document`, `seat`, `topic`, `project` | `TEXT` | no |
| `turn_seq` | `Option<i32>` | integer ≥ 0 | `INT` | yes — `NULL` = not recorded, never `0` |
| `payload` | typed per `(event_type, schema_version)` | JSON object | `JSONB` | nullable in SQL; **your API should require it** for new events — state your choice |

### 2.1 Deviation — `ref_type` has **nine** values, not eight. Lyra notified 2026-09-30 22:48 CST, before landing.

The table above omits **`delegation`**. The merged `010` CHECK admits it
(`010:270-276`, `event_object_refs_ref_type_vocab`), because `delegation` is in committed seed data
— 2 rows, `ev-020`/`ev-021` → `del-flux-acting-mira-001`. Onyx added it deliberately, flagged it in
their `010` delivery, and Lyra accepted that migration.

A Rust domain of eight could not read existing history. `RefType` therefore carries nine variants,
matching the database. Recorded in the code (`db/event_domain.rs`, the `RefType` doc comment) and
pinned by `serialized_sets_match_schema_010_checks`, which asserts the serialized set equals `010`
verbatim — so if a migration widens the domain again, that test is where the Rust side must follow.

**Resolution needed from Lyra:** amend the §3 table to add `delegation`. Nothing in the code needs
to change; the packet table does.

### 2.2 The `payload` choice §3 left to this API — **required**

`EventDraft.payload` is `serde_json::Value`, **not** `Option`, and `validate()` rejects anything
that is not a JSON **object**. The projection is rebuilt *from the payload*; an event without one is
unreplayable by construction, and admitting it would reintroduce exactly the decorative ledger step
2 exists to remove.

The SQL column stays nullable — `010` deliberately left it so, to keep `NULL` ("not recorded")
distinguishable from `{}` ("recorded, empty"). This is an **API tightening, not a schema change**;
no `.sql` file was touched.

### 2.3 Everything else, as written

- `event_type` — 12 values, PascalCase. **`ArtifactProduced` is absent**, and
  `artifact_produced_is_not_in_the_vocabulary` fails if anyone re-adds it without the sign-off that
  removed it (`AEGIS-2026-09-30-p0-step1-signoff-v1`). The first slice reuses `ArtifactCreated`.
- `role` — 3 values, snake_case.
- `turn_seq` — `Option<i32>`; `None` is *not recorded*, `Some(0)` is a real ordinal and is
  explicitly **accepted**, negative is rejected. `turn_seq_domain_is_enforced_without_conflating_none_and_zero`
  pins all three, because collapsing `None` and `0` is the exact defect `010:60-62` warns about.
- Casing is deliberately inconsistent between domains — PascalCase `event_type`, snake_case
  `role`/`ref_type` — because that is what is committed in PostgreSQL (AD-011). Not harmonized.

---

## 3. The four obligations `010` left to step 2

| # | Obligation | How it is met | Evidence |
|---|---|---|---|
| 2.1 | `schema_version` set **explicitly**; no reliance on `010:87`'s `DEFAULT 1` | `INSERT_EVENT_SQL` names the column; the value comes from `EventDraft.schema_version`, validated ≥ 1 | `inserts_name_the_columns_010_defaults_would_otherwise_fill` asserts the SQL text names it; `schema_version_below_one_is_rejected`; PROOF 1 shows `schema_version=1` on the stored row |
| 2.2 | `role` set **explicitly**; no reliance on `010:220`'s `DEFAULT 'context'` | `INSERT_REF_SQL` names `role`; `ObjectRefDraft` cannot be constructed without one (no `Default`, no optional field) | same SQL-text test; PROOF 1 reads back `actor` / `context` / `subject` — three distinct roles, none defaulted |
| 2.3 | **Exactly one subject** — the API supplies *presence* | `EventDraft::validate` returns `WriteError::NoSubject` for 0 and `MultipleSubjects` for >1, **before a connection is taken** | PROOF 3 + `zero_subject_append_is_rejected`, `append_with_no_refs_at_all_is_rejected`, `two_subjects_are_rejected_in_rust_before_the_index_fires` |
| 2.4 | Hand-written `FromStr`/`Display`, never serde derive | `db/event_domain.rs` — `EventType`, `RefRole`, `RefType` derive **no** serde traits and hand-write both | `all_domains_round_trip`, `serialized_sets_match_schema_010_checks`, `casing_is_significant_and_domain_specific`, `unknown_values_are_typed_errors` |

Because both INSERTs name their columns, **dropping `010`'s two transitional defaults cannot change
this module's behaviour** — which is the point of the tracked P1 cleanup. See finding F1 for what
still blocks that cleanup.

---

## 4. Layer A verification — verbatim

Run in the Nimbus worktree at `f5a3894`. The shared checkout was never branch-switched and Flux's
worktree was never touched.

**Tooling disclosure (packet §6).** There is no host `psql` client. Every SQL command below was
shimmed through `podman exec -i seatloom-postgres psql` via a wrapper on `PATH` that translates
`-f <hostfile>` into stdin (the host path does not exist inside the container). `apply-db.sh` ran
**unmodified**. Consequence: **`apply-db.sh`'s host-`psql` path stays unexercised locally** — CI's
`rust:test` job exercises it, so this branch needs to go through CI.

### 4.1 `cargo check -p seatloom-core`

```
$ cargo check -p seatloom-core
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.50s
[exit 0]
```

### 4.2 `cargo check -p seatloom-tauri`

```
$ cargo check -p seatloom-tauri
    Checking seatloom-tauri v0.1.0 (/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus/src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.91s
[exit 0]
```

Three `dead_code` warnings on `PipelineRunDto` / `ReviewThreadDto` / `ReviewCommentDto` are
pre-existing on `track/infra-foundation` and concern structs this packet does not touch.
ENGINEERING_STANDARDS records `src-tauri` as not yet clippy-gated for exactly this reason.

### 4.3 `cargo test -p seatloom-core --lib`

95 tests, 0 failed — 77 pre-existing plus 18 new.

```
$ cargo test -p seatloom-core --lib
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running unittests src/lib.rs (target/debug/deps/seatloom_core-767388bf53ce3b11)

running 95 tests
test db::document_parser::tests::ignores_header_examples_inside_code_fences ... ok
test db::document_parser::tests::normalizes_short_template_aliases ... ok
test db::document_parser::tests::parses_doc_id_and_status ... ok
test db::document_parser::tests::extracts_title_from_h1 ... ok
test db::document_parser::tests::header_subtype_valid_for_real_doc ... ok
test db::document_parser::tests::parses_depends_on_list ... ok
test db::document_parser::tests::parses_author_date_version ... ok
test db::document_parser::tests::parses_supersedes_and_tags ... ok
test db::document_parser::tests::parses_template_and_subtype ... ok
test db::document_parser::tests::returns_none_for_doc_without_header ... ok
test db::document_parser::tests::slugify_produces_lowercase_dashes ... ok
test db::document_parser::tests::validates_all_template_families ... ok
test db::document_parser::tests::slugify_strips_leading_trailing_dashes ... ok
test db::document_parser::tests::validates_subtype_t1_prd ... ok
test db::document_parser::tests::validates_subtype_t3_task ... ok
test db::event_domain::tests::all_domains_round_trip ... ok
test db::event_domain::tests::artifact_produced_is_not_in_the_vocabulary ... ok
test db::event_domain::tests::casing_is_significant_and_domain_specific ... ok
test db::event_domain::tests::serialized_sets_match_schema_010_checks ... ok
test db::reconcile::tests::changed_digest_detects_change ... ok
test db::event_domain::tests::unknown_values_are_typed_errors ... ok
test db::document_parser::tests::extracts_section_bodies ... ok
test db::document_parser::tests::extracts_sections_with_levels ... ok
test db::document_parser::tests::sections_have_deterministic_ordinals ... ok
test db::reconcile::tests::is_allowed_ingest_path_accepts_docs_coordination ... ok
test db::reconcile::tests::is_allowed_ingest_path_accepts_docs_root ... ok
test db::reconcile::tests::is_allowed_ingest_path_rejects_non_docs ... ok
test db::reconcile::tests::is_allowed_ingest_path_rejects_non_md ... ok
test db::reconcile::tests::sha256_hex_changes_on_body_change ... ok
test db::reconcile::tests::sha256_hex_is_deterministic ... ok
test db::reconcile::tests::path_to_doc_id_slugifies_correctly ... ok
test db::reconcile::tests::unchanged_digest_detects_no_change ... ok
test db::repositories::tests::null_harness_and_agent_are_accepted ... ok
test db::repositories::tests::out_of_domain_agent_fails_loudly_naming_the_row ... ok
test db::repositories::tests::out_of_domain_harness_fails_loudly ... ok
test db::repositories::tests::out_of_domain_seat_default_agent_fails_loudly ... ok
test db::repositories::tests::rendered_error_names_table_row_column_and_value ... ok
test db::repositories::tests::seeded_values_hydrate ... ok
test db::write_api::tests::inserts_name_the_columns_010_defaults_would_otherwise_fill ... ok
test db::write_api::tests::schema_version_below_one_is_rejected ... ok
test db::write_api::tests::blank_ref_id_is_rejected ... ok
test db::write_api::tests::append_with_no_refs_at_all_is_rejected ... ok
test db::write_api::tests::exactly_one_subject_is_accepted_and_returned ... ok
test db::write_api::tests::blank_required_fields_are_rejected ... ok
test db::write_api::tests::seeded_historical_payload_is_provably_unreplayable ... ok
test db::write_api::tests::payload_must_be_a_json_object ... ok
test db::write_api::tests::turn_seq_domain_is_enforced_without_conflating_none_and_zero ... ok
test db::write_api::tests::two_subjects_are_rejected_in_rust_before_the_index_fires ... ok
test db::write_api::tests::workitem_created_v1_payload_round_trips ... ok
test db::write_api::tests::workitem_status_changed_v1_payload_round_trips ... ok
test db::write_api::tests::zero_subject_append_is_rejected ... ok
test objects::seat::tests::custom_role_round_trips ... ok
test objects::seat::tests::every_seeded_role_value_parses ... ok
test ledger::index::tests::rebuilds_lookup_maps_from_seeded_events ... ok
test objects::seat::tests::serde_shape_is_snake_case ... ok
test objects::seat::tests::unknown_role_is_a_typed_error_not_a_custom_fallback ... ok
test objects::session::tests::agent_domain_round_trips ... ok
test objects::session::tests::dimensions_are_not_interchangeable ... ok
test objects::session::tests::harness_domain_round_trips ... ok
test objects::session::tests::model_id_is_opaque_free_form_text ... ok
test objects::session::tests::serde_shape_is_pascal_case_and_externally_tagged ... ok
test objects::session::tests::unknown_agent_is_a_typed_error_not_a_custom_fallback ... ok
test objects::session::tests::unknown_harness_is_a_typed_error ... ok
test pty::tests::default_transcripts_dir_nests_under_repo_root ... ok
test pty::tests::mirror_dir_is_tmp_seatloom_mirror ... ok
test pty::tests::shell_quote_handles_spaces_and_quotes ... ok
test storage::artifact_store::tests::resolves_artifact_paths_deterministically ... ok
test pty::tests::write_noop_on_empty_bytes ... ok
test pty::tests::write_noop_on_shutdown ... ok
test storage::artifact_store::tests::subtype_validation_follows_allowlist ... ok
test storage::handoff_store::tests::resolves_handoff_paths_deterministically ... ok
test storage::handoff_store::tests::missing_handoffs_dir_returns_empty_vec ... ok
test storage::artifact_store::tests::missing_artifacts_dir_returns_empty_vec ... ok
test db::reconcile::tests::collect_markdown_files_returns_sorted_paths ... ok
test storage::seat_registry::tests::delegation_validation_rejects_required_boundary_gaps ... ok
test pty::tests::list_tmux_sessions_returns_vec_without_panic ... ok
test storage::seat_registry::tests::missing_seats_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::missing_delegations_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::resolves_seat_layer_paths_deterministically ... ok
test storage::jsonl_io::tests::jsonl_append_and_read_round_trip ... ok
test storage::seat_registry::tests::delegation_round_trips ... ok
test storage::seat_registry::tests::save_delegation_returns_typed_validation_error ... ok
test storage::session_store::tests::resolves_session_paths_deterministically ... ok
test storage::handoff_store::tests::handoff_round_trip_and_list ... ok
test storage::artifact_store::tests::artifact_round_trip_and_list ... ok
test storage::session_store::tests::missing_sessions_dir_returns_empty_vec ... ok
test storage::workitem_store::tests::resolves_workitem_paths_deterministically ... ok
test storage::workitem_store::tests::missing_workitems_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::list_delegations_sorted_by_issued_at_descending ... ok
test storage::seat_registry::tests::seat_identity_round_trips ... ok
test storage::seat_registry::tests::list_seat_identities_sorted_by_name ... ok
test storage::seat_registry::tests::role_binding_round_trips ... ok
test storage::yaml_io::tests::project_config_round_trips_through_yaml ... ok
test storage::workitem_store::tests::workitem_round_trip_and_list ... ok
test storage::session_store::tests::session_round_trip_and_list ... ok

test result: ok. 95 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
[exit 0]
```

### 4.4 `cargo clippy -p seatloom-core -- -D warnings`

```
$ cargo clippy -p seatloom-core -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s
[exit 0]
```

ENGINEERING_STANDARDS:96 makes the **full** gate `--all-targets` (which the packet's §6 list does
not include, but the standard does). Run additionally, so the new example is linted too:

```
$ cargo clippy -p seatloom-core --all-targets -- -D warnings
    Checking seatloom-core v0.1.0 (…/seatloom-worktrees/nimbus/crates/seatloom-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.77s
[exit 0]
```

### 4.5 `cargo fmt --all --check`

A FULL gate per ENGINEERING_STANDARDS:94, also not in the packet's §6 list.

```
$ cargo fmt --all --check
[exit 0]
```

It was **not** clean on first run: `rustfmt` reflowed 8 sites in the two files I authored
(`write_api.rs`, `p0_write_api_proof.rs`) and nothing else. `cargo fmt --all` was applied before the
commit, so no pre-existing file was reformatted.

### 4.6 `cd ui && pnpm exec tsc --noEmit`

```
$ cd ui && pnpm exec tsc --noEmit
[exit 0]
```

Zero output, zero errors. **No `ui/` file was changed** — this packet adds no wire surface, so `tsc`
had nothing to be forced by.

### 4.7 `cd ui && pnpm build`

```
$ cd ui && pnpm build

> seatloom-ui@0.1.0 build /Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus/ui
> tsc && vite build

vite v6.4.2 building for production...
transforming...
✓ 1606 modules transformed.
rendering chunks...
computing gzip size...
dist/supervisor.html                        0.59 kB │ gzip:   0.35 kB
dist/index.html                             0.64 kB │ gzip:   0.36 kB
dist/assets/main-CFbL2ovg.css               4.64 kB │ gzip:   1.72 kB
dist/assets/SupervisorPanel-DVSr8ufT.css   48.25 kB │ gzip:   9.20 kB
dist/assets/supervisor-BzvFkL_J.js          0.86 kB │ gzip:   0.56 kB
dist/assets/SupervisorPanel-C8CizSjl.js   353.04 kB │ gzip: 103.03 kB
dist/assets/main-Bg8mJuT5.js              517.13 kB │ gzip: 125.73 kB

(!) Some chunks are larger than 500 kB after minification. Consider:
- Using dynamic import() to code-split the application
- Use build.rollupOptions.output.manualChunks to improve chunking: https://rollupjs.org/configuration-options/#output-manualchunks
- Adjust chunk size limit for this warning via build.chunkSizeWarningLimit.
✓ built in 2.19s
[exit 0]
```

### 4.8 Scratch database — `apply-db.sh`

```
$ psql .../postgres -c "DROP DATABASE IF EXISTS seatloom_s2_nimbus;" -c "CREATE DATABASE seatloom_s2_nimbus;"
NOTICE:  database "seatloom_s2_nimbus" does not exist, skipping
DROP DATABASE
CREATE DATABASE

$ DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_s2_nimbus bash scripts/apply-db.sh
apply-db: target=postgresql://seatloom:seatloom@localhost:5432/seatloom_s2_first
--- pre-seed schema (seed writes columns these create) ---
  infra/postgres/schema/001_seatloom_core.sql
      why: creates projects/seats/sessions/workitems/handoffs/artifacts/canonical_events — every table the seed inserts into
  infra/postgres/schema/002_document_authority.sql
      why: creates documents/document_sections; seed 002 inserts document rows
  infra/postgres/schema/003_write_ingest_reconcile.sql
      why: creates reconcile_runs/reconcile_items/document_versions written by reconcile and seed 002
  infra/postgres/schema/004_operational_review_and_continuity.sql
      why: creates checkpoints/review_threads/review_comments; seed 003 inserts into them
  infra/postgres/schema/005_prompt_and_channel_action_authority.sql
      why: creates prompt_instances/prompt_actions/channel_action_receipts; seed 004 targets them (zero-row baseline)
  infra/postgres/schema/009_runtime_triple.sql
      why: ADDS seats.default_harness/_agent/_model and sessions.harness/agent/model, which seed 001 WRITES — must exist before the seed runs. Safe here: it only touches columns created in 001 and constrains nothing the seed omits (its sole SET NOT NULL is on sessions.agent, which the seed supplies).
--- seed (001-004) ---
  infra/postgres/seed/001_real_collaboration_baseline.sql
  infra/postgres/seed/002_document_seed.sql
  infra/postgres/seed/003_operational_review_and_continuity_seed.sql
  infra/postgres/seed/004_prompt_and_channel_action_seed.sql
--- post-seed schema (constrains columns the seed does not provide) ---
  infra/postgres/schema/008_project_isolation.sql
      why: backfills project_id on workitems/sessions/handoffs/canonical_events from seeded rows, then SET NOT NULL. Seed 001 supplies no project_id, so running this before the seed makes the seed fail: 'null value in column "project_id" violates not-null constraint'.
  infra/postgres/schema/010_p0_schema_foundation.sql
      why: adds canonical_events.schema_version NOT NULL and event_object_refs.role NOT NULL, then backfills both from existing rows. Seed 001 inserts into BOTH tables and supplies NEITHER column, so pre-seed would break the seed exactly as 008 would have. Also widens the event_object_refs primary key to include role and adds a partial unique index scoped so legacy rows cannot violate it — both need the seeded rows present.
apply-db: done.
[exit 0]
```

**Second full run — the predicted pre-existing failure, not this packet's defect.** The packet says
to note it and move on:

```
$ DATABASE_URL=…/seatloom_s2_nimbus bash scripts/apply-db.sh      # second time
apply-db: target=postgresql://seatloom:seatloom@localhost:5432/seatloom_s2_nimbus
…
ERROR:  null value in column "project_id" of relation "sessions" violates not-null constraint
[exit 3]
```

`008` sets `sessions.project_id NOT NULL` post-seed; re-running seed 001 then inserts sessions
without it. Known seed defect with its own follow-up. Noted, not touched.

### 4.9 `git show --stat f5a3894`

See §1. Zero `.tsx`, zero `.sql`.

---

## 5. The three behavioural proofs

Harness: `crates/seatloom-core/examples/p0_write_api_proof.rs`, run against `seatloom_s2_nimbus`.
It **refuses to start** if the target database is named `seatloom`.

```
$ DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/seatloom_s2_nimbus \
    cargo run -q -p seatloom-core --example p0_write_api_proof
target database = seatloom_s2_nimbus
fixture: project p0-s2-proof present, its events and workitems cleared

########## PROOF 1 — atomicity, SUCCESS: one append writes event + refs + projection ##########
append_event -> WorkItemInserted { workitem_id: "wi-p0-s2-001" }
canonical_events row: event_type=WorkItemCreated schema_version=1 turn_seq=None project_id=p0-s2-proof
event_object_refs rows (3):
  ref_type=seat      ref_id=seat-nimbus-001      role=actor
  ref_type=project   ref_id=p0-s2-proof          role=context
  ref_type=workitem  ref_id=wi-p0-s2-001         role=subject
workitems projection row: WorkItemSnapshot { id: "wi-p0-s2-001", title: "P0 step 2 write API", goal: Some("append(event) -> projections"), acceptance_criteria: ["atomic", "replayable"], owner_seat_id: Some("seat-nimbus-001"), status: "in_progress", priority: "high", created_at: 2026-09-30T10:00:00Z, updated_at: 2026-09-30T10:00:00Z, project_id: "p0-s2-proof" }
HELD: event, 3 explicit-role refs, and the projection row all present.

########## PROOF 2 — atomicity, FAILURE: a forced mid-append failure rolls BOTH back ##########
Fault injected with a REAL constraint, not a test hook: owner_seat_id points at a
seat that does not exist, so the projection INSERT violates
workitems.owner_seat_id REFERENCES seats(id) (001:85) AFTER canonical_events and
event_object_refs have already been inserted inside the same transaction.
append_event returned Err, as required:
postgres error: db error
  caused by: db error
  caused by: ERROR: insert or update on table "workitems" violates foreign key constraint "workitems_owner_seat_id_fkey"
DETAIL: Key (owner_seat_id)=(seat-does-not-exist-999) is not present in table "seats".
  sqlstate=23503 constraint=Some("workitems_owner_seat_id_fkey") table=Some("workitems")
  detail: Key (owner_seat_id)=(seat-does-not-exist-999) is not present in table "seats".
fault constraint present in catalog: Some("workitems_owner_seat_id_fkey") (the FK the projection INSERT violated)
after the failed append:
  canonical_events  rows for ev-p0-s2-rollback = 0   (must be 0)
  event_object_refs rows for ev-p0-s2-rollback = 0   (must be 0)
  workitems         rows for wi-p0-s2-doomed = 0   (must be 0)
  proof-1 projection row still present and unchanged: status=in_progress
HELD: event, refs and projection all absent — rollback is atomic, not partial.

########## PROOF 3 — zero-subject append is rejected by a typed error, before the database ##########
typed error WriteError::NoSubject
  event_id   = ev-p0-s2-nosubject
  event_type = WorkItemCreated
  ref_count  = 2
  rendered   = event "ev-p0-s2-nosubject" (WorkItemCreated) carries no subject ref: exactly one ref with role='subject' is required, found 0 of 2 ref(s)
canonical_events rows for ev-p0-s2-nosubject = 0   (must be 0 — rejected pre-database)
HELD: 010:332 gives at-most-one; this gives presence. Together: exactly one.

########## PROOF 4 — rebuild: projection reconstructed from canonical_events alone ##########
append_event (status change) -> WorkItemStatusUpdated { workitem_id: "wi-p0-s2-001", to_status: "done" }

projection BEFORE truncate+rebuild:
  WorkItemSnapshot { id: "wi-p0-s2-001", title: "P0 step 2 write API", goal: Some("append(event) -> projections"), acceptance_criteria: ["atomic", "replayable"], owner_seat_id: Some("seat-nimbus-001"), status: "done", priority: "high", created_at: 2026-09-30T10:00:00Z, updated_at: 2026-09-30T12:00:00Z, project_id: "p0-s2-proof" }

rebuild_workitems_projection(p0-s2-proof) -> deleted 1 row(s), replayed 2 event(s)
projection AFTER  truncate+rebuild:
  WorkItemSnapshot { id: "wi-p0-s2-001", title: "P0 step 2 write API", goal: Some("append(event) -> projections"), acceptance_criteria: ["atomic", "replayable"], owner_seat_id: Some("seat-nimbus-001"), status: "done", priority: "high", created_at: 2026-09-30T10:00:00Z, updated_at: 2026-09-30T12:00:00Z, project_id: "p0-s2-proof" }

field-by-field equality:
  id                   ==  wi-p0-s2-001
  title                ==  P0 step 2 write API
  goal                 ==  Some("append(event) -> projections")
  acceptance_criteria  ==  ["atomic", "replayable"]
  owner_seat_id        ==  Some("seat-nimbus-001")
  status               ==  done
  priority             ==  high
  created_at           ==  2026-09-30T10:00:00+00:00
  updated_at           ==  2026-09-30T12:00:00+00:00
  project_id           ==  p0-s2-proof

HELD: status='done' and updated_at both came back from the ledger alone —
the UPDATE event replayed, not just the CREATE. The ledger is now causal.

########## PROOF 5 — seeded history is NOT replayable — measured per row, not assumed ##########
5a. Deserializing every seeded WorkItemCreated payload into WorkItemCreatedV1 —
    the exact type replay_workitem_event uses:

  ev-010  UNREPLAYABLE  payload = {"title":"Define product form and tech stack","workitem_id":"wi-001"}
            serde error: missing field `status`
  ev-011  UNREPLAYABLE  payload = {"title":"Deliver Phase 1 UX prototypes","workitem_id":"wi-004"}
            serde error: missing field `status`
  ev-012  UNREPLAYABLE  payload = {"title":"Flux acting-Mira UI contract repair","workitem_id":"wi-009"}
            serde error: missing field `status`
  ev-013  UNREPLAYABLE  payload = {"title":"Nimbus Rust foundation scaffold","workitem_id":"wi-scaffold"}
            serde error: missing field `status`
  ev-014  UNREPLAYABLE  payload = {"title":"Nimbus storage and ledger foundation","workitem_id":"wi-storage"}
            serde error: missing field `status`
  ev-015  UNREPLAYABLE  payload = {"title":"Nimbus Rust foundation hardening","workitem_id":"wi-hardening"}
            serde error: missing field `status`

  6 of 6 seeded WorkItemCreated events cannot reconstruct their projection row.
  Cause: workitems requires status NOT NULL and priority NOT NULL (001:86-87); no seeded
  payload carries status, priority, goal or acceptance_criteria. Those rows were written
  for display, not for replay.

5b. Independent obstacle: the rebuild's DELETE cannot even reach the replay for a
    project whose workitems have dependents.
  rebuild refused:
postgres error: db error
  caused by: db error
  caused by: ERROR: update or delete on table "workitems" violates foreign key constraint "handoffs_workitem_id_fkey" on table "handoffs"
DETAIL: Key (id)=(wi-004) is still referenced from table "handoffs".
  sqlstate=23503 constraint=Some("handoffs_workitem_id_fkey") table=Some("handoffs")
  detail: Key (id)=(wi-004) is still referenced from table "handoffs".
  That is handoffs.workitem_id NOT NULL REFERENCES workitems(id) (001:99) with no
  ON DELETE clause — so the rebuild fails at DELETE, BEFORE any payload is examined.
  5a is the reason seeded history is unreplayable; 5b is why the rebuild cannot even
  try. Both are real; neither is fixed here. Reported, not remediated.

  seeded workitems rows still present after the refused rebuild = 6
  (the rebuild runs in ONE transaction, so its DELETE rolled back too — a refused
   rebuild is not destructive)

HELD: `workitems` is replayable for events this API produced, from step 2 forward.
Pre-step-2 history is not, for two independent reasons, and no seed payload was
rewritten to hide either.

########## ALL PROOFS HELD ##########
[exit 0]
```

### 5.1 Reading the proofs

**PROOF 1 — atomicity, success.** One `append_event` produced the `canonical_events` row
(`schema_version=1`, `turn_seq=None`, both explicit), three `event_object_refs` rows with three
*distinct* explicit roles, and the `workitems` projection row.

**PROOF 2 — atomicity, rollback.** This is the one that matters, and it is proven, not asserted.
The fault is a **real constraint, not a test hook**: `owner_seat_id` points at a seat that does not
exist, so the projection INSERT violates `workitems_owner_seat_id_fkey` *after* the event and its
refs are already inserted in the same transaction. Afterwards all three counts are `0`, and the
PROOF 1 row is untouched.

> **The first version of this harness printed only `postgres error: db error`.** That is
> `tokio_postgres::Error`'s Display; the SQLSTATE, constraint name and server message live in its
> source chain. A rollback transcript that does not say *which* failure caused it is not evidence
> the injected fault is what rolled back — an unrelated error would have looked identical. The
> harness now walks the chain and prints `sqlstate=23503
> constraint=Some("workitems_owner_seat_id_fkey")`, and additionally asserts that constraint exists
> by name.

**PROOF 3 — zero-subject rejection.** `WriteError::NoSubject`, raised by `validate()` before a
connection is taken, so nothing reaches the database (count `0`). `010:332` gives *at most one*;
this gives *presence*; together, *exactly one*.

**PROOF 4 — rebuild from the ledger alone.** Two events appended (`WorkItemCreated` then
`WorkItemStatusChanged`, so the replay must handle an UPDATE and not only a CREATE), the projection
snapshotted, its rows deleted, then rebuilt from `canonical_events` + `event_object_refs` only. All
ten columns compare equal, including `status='done'` and `updated_at=12:00:00Z`, both of which exist
**only** in the second event — proving the UPDATE genuinely replayed.

---

## 6. Replayability — what is replayable, from when, and what is not

**Replayable:** the `workitems` projection, for events produced by `append_event`, **from this
commit forward**. Proven by PROOF 4.

**Not replayable:** all pre-step-2 history. Two *independent* reasons, both measured:

**(a) The payloads are lossy.** PROOF 5a runs every seeded `WorkItemCreated` payload through
`WorkItemCreatedV1` — the exact type the replay uses. **6 of 6 fail**, each with `missing field
status`. Seeded payloads carry only `{workitem_id, title}`, while the projection needs `status NOT
NULL` and `priority NOT NULL` (`001:86-87`); `priority`, `goal` and `acceptance_criteria` appear in
no seeded payload at all. Those rows were written for display, not for replay. Also pinned as a unit
test (`seeded_historical_payload_is_provably_unreplayable`) so it cannot regress into a silent
default.

**(b) The rebuild cannot even reach the replay** for a project whose workitems have dependents.
`handoffs.workitem_id` is `NOT NULL REFERENCES workitems(id)` (`001:99`) with no `ON DELETE` clause,
so the rebuild's `DELETE` fails first with `handoffs_workitem_id_fkey`.

> **Correction worth recording.** My first harness ran the rebuild against the seeded project, got a
> failure, and narrated it as proof of (a). It was actually (b) — the DELETE never reached a single
> payload. A passing proof that attributes the right conclusion to the wrong cause is not evidence.
> The harness now demonstrates (a) directly, per row, and reports (b) separately as what it is.

Neither is remediated here. **No seed payload was rewritten** — that is forbidden and out of scope.
A future backfill packet has to decide what to do; (b) in particular means any real rebuild strategy
needs an answer for dependent tables, not just for payloads.

A refused rebuild is **not destructive**: it runs in one transaction, so its `DELETE` rolls back too
(PROOF 5, 6 seeded rows still present afterwards).

---

## 7. Done-definition status (§7)

| # | Criterion | Status |
|---|---|---|
| 1 | One `append(event) -> projections` path; no repository method mutates a projection directly | **PASS** — `apply_projection`/`replay_workitem_event` are private to `db::write_api`; `grep 'INSERT INTO workitems\|UPDATE workitems\|DELETE FROM workitems' crates/seatloom-core/src/` returns hits only in `db/write_api.rs` |
| 2 | Event + projection commit in one transaction; rollback proven | **PASS** — PROOF 1 and PROOF 2, with the violated constraint named |
| 3 | `schema_version` explicit; no reliance on `DEFAULT 1` | **PASS** — §3 |
| 4 | `role` explicit; no reliance on the `context` default | **PASS** — §3 |
| 5 | Zero-subject append rejected, typed error, negative test output | **PASS** — PROOF 3 + 3 unit tests |
| 6 | Hand-written `FromStr`/`Display`; `ArtifactProduced` absent | **PASS** — §3 row 2.4 |
| 7 | §3 contract reproduced verbatim; deviation flagged before landing | **PASS with deviation** — §2.1, Lyra notified 2026-09-30 22:48 CST |
| 8 | A projection rebuilds from the ledger alone; seeded limitation stated plainly | **PASS** — PROOF 4; limitation in §6 |
| 9 | All §6 commands green, verbatim | **PASS** — §4 (plus `fmt` and `clippy --all-targets` from ENGINEERING_STANDARDS) |
| 10 | Zero `.tsx`, zero `.sql` | **PASS** — §1 |
| 11 | Built in the Nimbus worktree; shared checkout never branch-switched | **PASS** — §8 |
| 12 | Delivery doc complete; Flux commit-pinned verify PASSes | **PENDING** — Flux gate against `f5a3894` |

---

## 8. Findings — three, none of them blocking this packet

### F1. `append_supervisor_message` is unmigrated — it is the sole consumer of **both** `010` defaults, and it produces events this write API would refuse

**Lyra ruled this OUT OF SCOPE for step 2 and will issue its own follow-up packet. It is not fixed
here, and this section does not soften it.**

Line anchors, given for both commits because my own `pool_client` addition (+7 lines) shifted them:

| What | at `29adefd` (base) | at `f5a3894` (delivery) |
|---|---|---|
| `pub async fn append_supervisor_message` | `repositories.rs:405` | `repositories.rs:412` |
| `INSERT INTO canonical_events …` | `:421` | `:428` |
| `INSERT INTO event_object_refs …` | `:429` | `:437` |

Four specifics:

1. **The `canonical_events` INSERT does not name `schema_version`.** Its column list is
   `(id, event_type, occurred_at, actor_ref, payload, project_id)`. Every row it writes therefore
   takes `010:87`'s `DEFAULT 1`.
2. **The `event_object_refs` INSERT does not name `role`.** Its column list is
   `(event_id, ref_type, ref_id)`. Every ref it writes therefore takes the `'context'` default
   (`010:220`) — the very thing `010:220` calls "a bug" in new code.
3. **It writes only a `ref_type='seat'` ref, so every supervisor message is a ZERO-SUBJECT event.**
   And only when `target_seat_id` is `Some`; when it is `None` the event gets **no refs at all**.
   Both cases are zero-subject — exactly what `EventDraft::validate` now rejects with
   `WriteError::NoSubject`.
4. **It is therefore the sole remaining consumer of both `010` defaults**, and the P1 cleanup to
   drop them cannot land until it is migrated.

**Nothing is broken at runtime today.** `010:332` enforces *at most* one subject, and zero satisfies
that, so no database constraint is breached. The finding is the **asymmetry**: the ledger will keep
accumulating events that the write API guarding that same ledger would refuse to create.

Measured on the scratch database after the proof run, so this is counted rather than argued:

```
$ psql .../seatloom_s2_nimbus -c "SELECT count(*) AS events_total,
       count(*) FILTER (WHERE s.n IS NULL OR s.n = 0) AS zero_subject_events,
       count(*) FILTER (WHERE s.n = 1) AS exactly_one_subject
  FROM canonical_events e
  LEFT JOIN (SELECT event_id, count(*) n FROM event_object_refs
              WHERE role='subject' GROUP BY event_id) s ON s.event_id = e.id;"
 events_total | zero_subject_events | exactly_one_subject
--------------+---------------------+---------------------
           36 |                  34 |                   2
(1 row)
```

**34 of 36 events carry no subject. The only two that satisfy the invariant are the two this write
API produced** in PROOF 1 and PROOF 4. Both `010` defaults are still in place and still load-bearing
for the legacy writer:

```
$ psql .../seatloom_s2_nimbus -c "SELECT table_name, column_name, column_default FROM information_schema.columns
 WHERE (table_name='canonical_events' AND column_name='schema_version')
    OR (table_name='event_object_refs' AND column_name='role');"
    table_name     |  column_name   | column_default
-------------------+----------------+-----------------
 canonical_events  | schema_version | 1
 event_object_refs | role           | 'context'::text
(2 rows)
```

Reached from `src-tauri/src/commands/supervisor_cmds.rs:57` (L1 Supervisor IM), which is live.

This does **not** violate done-definition item 1 — `canonical_events` is the ledger, not a
projection — but the ledger now has two writers and only one of them is governed.

**What the follow-up has to decide:** what a `SupervisorMessage` is *about*, i.e. its mandatory
subject. There is no obvious answer: the target seat is the addressee (an `actor`-ish or
`context`-ish relation), not what the message is about, and an untargeted broadcast has no candidate
at all. That is a governed-dispatch question (step 6), not a mechanical port.

One related detail: its `ON CONFLICT DO NOTHING` (`:429`/`:437`) is the mechanism `010`'s G3 comment
names as how a role distinction gets silently dropped. `INSERT_REF_SQL` in the new path deliberately
has none, and `inserts_name_the_columns_010_defaults_would_otherwise_fill` asserts its absence.

### F2. Two `EventType` vocabularies now exist

`db::event_domain::EventType` is the PostgreSQL vocabulary (12 values, pinned to `010`'s CHECK).
`ledger::event::EventType` is the JSONL file ledger's (27 variants, serde-derived, AD-005) and is
strictly wider — `SessionCompleted`, `PromptDetected`, `PipelineStarted` and others would be
rejected by PostgreSQL. They are not unified, and unifying them is not this packet's scope. The file
ledger is untouched. Worth a decision before step 4 writes events from a third place.

### F3. `apply-db.sh`'s host-`psql` path is unexercised locally

Consequence of the shim (§4). CI's `rust:test` exercises it; this branch should go through CI before
merge.

---

## 9. Constraints honoured

- Built in `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus`. The shared checkout
  stayed on `track/infra-foundation`; **never branch-switched**. Flux's worktree untouched.
- The packet file is not present at `29adefd`; read via
  `git show track/infra-foundation:docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-p0-write-api-v1.md`,
  the recovery path the packet supplies.
- **Live `seatloom` database never written.** All work in `seatloom_s2_nimbus`; the harness refuses
  to run against a database named `seatloom`.
- `scripts/verify-postgres-baseline.sh` **not run**.
- No `.sql` changed — `010` stays a closed gate. No `010` default dropped.
- No seed payload rewritten.
- No MCP emit server, no governed dispatch, no first captured event, no UI.

---

*Nimbus · 2026-09-30 · P0 step 2, critical path · worktree-isolated · packet
NIMBUS-2026-09-30-p0-write-api-v1 · Aegis GO AEGIS-2026-09-30-p0-step2-write-api-v1*
