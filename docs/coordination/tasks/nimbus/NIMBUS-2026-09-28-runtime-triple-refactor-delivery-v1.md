# Delivery: Runtime Triple Refactor — split `Runtime` into harness / agent / model

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | NIMBUS-2026-09-28-runtime-triple-refactor-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-09-30 |
| version | v1 |
| to | flux (Layer A verify), lyra (acceptance) |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1.md` |
| base branch | `track/infra-foundation` |
| base commit | `9599136d615db0c4cdb15811ccaca5532820870d` (packet-declared) |
| merged base | `d70a5e0` — `track/infra-foundation` after Onyx schema 009 landed (merge `80be8b5`, acceptance `docs/coordination/acceptance/2026-09-30-lyra-onyx-runtime-triple-migration-acceptance-v1.md`) |
| delivery branch | `packet/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1` |
| delivery commit SHA (code) | **`9bef948`** — gate Layer A against this; it is the last commit that changes any `.rs`/`.ts` file |
| delivery branch tip | the commit carrying this document, whose parent is `9bef948`. Not hardcoded here: a SHA cannot name its own commit. Resolve with `git rev-parse packet/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1`. |
| branch history | `b10280d` refactor → `c88898c` merge of `track/infra-foundation` (`d70a5e0`) → `9bef948` added test → this doc |
| milestone | pre-ingestion schema debt paydown |
| tags | nimbus, schema, runtime-triple, seat-role, ingestion-prep, option-B, no-ui, delivery |

---

## 1. Summary

`Runtime` is gone. `Session` now carries `harness: Option<Harness>`, `agent: Agent`,
`model: Option<ModelId>`; `SeatIdentity` carries `default_harness` / `default_agent` /
`default_model`. `SeatRole` gains `Supervisor` and `DataEngineer` and serializes snake_case.

A single fallible parse boundary at row hydration (`db/repositories.rs`) converts database strings
into `Harness` / `Agent` / `SeatRole`, rejecting anything outside the declared domain with a typed
error that names the table, row id, column and offending value.

Branch history, in order:

| SHA | What |
|---|---|
| `b10280d` | the refactor itself — 10 files, +716/−66, authored against base `9599136` |
| `c88898c` | merge of `track/infra-foundation` (`d70a5e0`), bringing in Onyx schema 009 |
| `9bef948` | one added test pinning the rendered error text (§2.4.5 hardening) |
| tip | this delivery doc |

The merge was required, not cosmetic: the SELECTs in `db/repositories.rs` name
`harness` / `agent` / `model` and `default_harness` / `default_agent` / `default_model`, which only
exist after Onyx's 009. That is why Onyx merged first.

**Net Rust/TS delta of this packet over `d70a5e0`** — exactly the 10 files, zero `.tsx`, zero
`infra/postgres/`:

```
$ git diff --stat d70a5e0 HEAD
 crates/seatloom-core/src/adapter/traits.rs        |   8 +-
 crates/seatloom-core/src/db/models.rs             |  18 +-
 crates/seatloom-core/src/db/repositories.rs       | 280 +++++++++++++++++---
 crates/seatloom-core/src/objects/seat.rs          | 133 +++++++++-
 crates/seatloom-core/src/objects/session.rs       | 300 +++++++++++++++++++++-
 crates/seatloom-core/src/storage/seat_registry.rs |  20 +-
 crates/seatloom-core/src/storage/session_store.rs |   6 +-
 src-tauri/src/dto.rs                              |  16 +-
 ui/src/lib/dto-to-v1.ts                           |   5 +-
 ui/src/lib/types-dto.ts                           |  10 +-
 10 files changed, 730 insertions(+), 66 deletions(-)
```

---

## 2. `git show --stat` (§6.2)

Code commit `9bef948`:

```
$ git show --stat 9bef948
commit 9bef94830332c45afbcf27b6852acadfc809a723
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Wed Sep 30 18:59:49 2026 +0800

    test(core): pin the operator-facing text of the value-domain error
    
    The negative-path tests already proved a bad value yields a typed error
    rather than a `Custom(...)` fallback or a panic. This pins the rendered
    message itself, so a refactor cannot silently drop the table, row id,
    column or offending value from what an operator sees.
    
    Packet: docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1.md §2.4.5
    
    Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>

 crates/seatloom-core/src/db/repositories.rs | 14 ++++++++++++++
 1 file changed, 14 insertions(+)
```

Original refactor commit `b10280d` (the 10-file scope, for the §4 `.tsx` row):

```
$ git show --stat b10280d
commit b10280d1e4672ab16a79b0f1e123bf471aaaf74e
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Mon Sep 28 21:46:21 2026 +0800

    refactor(core): split Runtime into harness / agent / model triple
    
    Aegis review 2026-09-28 §4: `Runtime` collapsed three orthogonal
    dimensions (harness / agent / model) into one field. All six seats run
    stepcode -> Claude Code -> claude-opus-5[1m]; `default_runtime =
    'ClaudeCode'` cannot express that.
    
    - objects/session.rs: add `Harness`, `Agent`, `ModelId`; delete `Runtime`.
      `Session` carries `harness: Option<Harness>`, `agent: Agent`,
      `model: Option<ModelId>`. PascalCase serde, matching existing PG values.
    - objects/seat.rs: `SeatIdentity` carries `default_harness` /
      `default_agent` / `default_model`. `SeatRole` gains `Supervisor` and
      `DataEngineer` plus `rename_all = "snake_case"`, matching the seed.
    - Single fallible parse boundary at row hydration (`db/repositories.rs`
      `check_domain`): an out-of-domain string yields `DbError::ValueDomain`
      naming table, row id, column and value — never a `Custom(...)`
      fallback, never a panic.
    - DTO/TS wire surface carries the triple as flat strings.
    
    `stepcode`/`stepgo` is a HARNESS; `opencode` is an AGENT. Do not re-add
    either as the other dimension.
    
    Requires Onyx schema 009 (`harness`/`agent`/`model` columns) to be
    applied before the session/seat queries can run.
    
    Packet: docs/coordination/tasks/nimbus/NIMBUS-2026-09-28-runtime-triple-refactor-v1.md
    
    Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>

 crates/seatloom-core/src/adapter/traits.rs        |   8 +-
 crates/seatloom-core/src/db/models.rs             |  18 +-
 crates/seatloom-core/src/db/repositories.rs       | 266 ++++++++++++++++---
 crates/seatloom-core/src/objects/seat.rs          | 133 +++++++++-
 crates/seatloom-core/src/objects/session.rs       | 300 +++++++++++++++++++++-
 crates/seatloom-core/src/storage/seat_registry.rs |  20 +-
 crates/seatloom-core/src/storage/session_store.rs |   6 +-
 src-tauri/src/dto.rs                              |  16 +-
 ui/src/lib/dto-to-v1.ts                           |   5 +-
 ui/src/lib/types-dto.ts                           |  10 +-
 10 files changed, 716 insertions(+), 66 deletions(-)
```

---

## 3. §5 verification plan — verbatim output (§6.3)

Run in the main checkout `~/Documents/GitHub/seatloom` at branch tip, Layer A only. No database
was read or written; `scripts/verify-postgres-baseline.sh` was **not** run (Lyra dispatch
2026-09-30: its line 59 `compose down -v` destroys the live volume, and podman here has no compose
provider).

### 3.1 `cargo check -p seatloom-core`

```
$ cargo check -p seatloom-core
    Checking seatloom-core v0.1.0 (/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/crates/seatloom-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.50s
[exit 0]
```

### 3.2 `cargo check -p seatloom-tauri`

```
$ cargo check -p seatloom-tauri
    Checking seatloom-core v0.1.0 (/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/crates/seatloom-core)
    Checking seatloom-tauri v0.1.0 (/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/src-tauri)
warning: struct `PipelineRunDto` is never constructed
   --> src-tauri/src/dto.rs:594:12
    |
594 | pub struct PipelineRunDto {
    |            ^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: struct `ReviewThreadDto` is never constructed
   --> src-tauri/src/dto.rs:632:12
    |
632 | pub struct ReviewThreadDto {
    |            ^^^^^^^^^^^^^^^

warning: struct `ReviewCommentDto` is never constructed
   --> src-tauri/src/dto.rs:676:12
    |
676 | pub struct ReviewCommentDto {
    |            ^^^^^^^^^^^^^^^^

warning: `seatloom-tauri` (bin "seatloom-tauri") generated 3 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.49s
[exit 0]
```

The three `dead_code` warnings are **pre-existing on `track/infra-foundation`** and concern
`PipelineRunDto`, `ReviewThreadDto`, `ReviewCommentDto` — structs this packet does not touch:

```
$ git diff d70a5e0 HEAD -- src-tauri/src/dto.rs | grep -E 'PipelineRunDto|ReviewThreadDto|ReviewCommentDto'
(no output — my diff does not touch those three structs)
```

### 3.3 `cargo test -p seatloom-core --lib`

```
$ cargo test -p seatloom-core --lib
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.22s
     Running unittests src/lib.rs (target/debug/deps/seatloom_core-767388bf53ce3b11)

running 77 tests
test db::document_parser::tests::parses_author_date_version ... ok
test db::document_parser::tests::ignores_header_examples_inside_code_fences ... ok
test db::document_parser::tests::parses_depends_on_list ... ok
test db::document_parser::tests::extracts_title_from_h1 ... ok
test db::document_parser::tests::header_subtype_valid_for_real_doc ... ok
test db::document_parser::tests::normalizes_short_template_aliases ... ok
test db::document_parser::tests::extracts_section_bodies ... ok
test db::document_parser::tests::extracts_sections_with_levels ... ok
test db::document_parser::tests::parses_doc_id_and_status ... ok
test db::document_parser::tests::parses_supersedes_and_tags ... ok
test db::document_parser::tests::parses_template_and_subtype ... ok
test db::document_parser::tests::slugify_produces_lowercase_dashes ... ok
test db::document_parser::tests::slugify_strips_leading_trailing_dashes ... ok
test db::document_parser::tests::sections_have_deterministic_ordinals ... ok
test db::document_parser::tests::returns_none_for_doc_without_header ... ok
test db::document_parser::tests::validates_all_template_families ... ok
test db::document_parser::tests::validates_subtype_t1_prd ... ok
test db::document_parser::tests::validates_subtype_t3_task ... ok
test db::reconcile::tests::is_allowed_ingest_path_accepts_docs_coordination ... ok
test db::reconcile::tests::is_allowed_ingest_path_accepts_docs_root ... ok
test db::reconcile::tests::is_allowed_ingest_path_rejects_non_docs ... ok
test db::reconcile::tests::is_allowed_ingest_path_rejects_non_md ... ok
test db::reconcile::tests::changed_digest_detects_change ... ok
test db::reconcile::tests::unchanged_digest_detects_no_change ... ok
test db::reconcile::tests::sha256_hex_is_deterministic ... ok
test db::reconcile::tests::sha256_hex_changes_on_body_change ... ok
test db::repositories::tests::null_harness_and_agent_are_accepted ... ok
test db::reconcile::tests::path_to_doc_id_slugifies_correctly ... ok
test db::repositories::tests::out_of_domain_harness_fails_loudly ... ok
test db::repositories::tests::out_of_domain_seat_default_agent_fails_loudly ... ok
test db::repositories::tests::rendered_error_names_table_row_column_and_value ... ok
test db::repositories::tests::seeded_values_hydrate ... ok
test db::repositories::tests::out_of_domain_agent_fails_loudly_naming_the_row ... ok
test ledger::index::tests::rebuilds_lookup_maps_from_seeded_events ... ok
test objects::seat::tests::custom_role_round_trips ... ok
test objects::seat::tests::every_seeded_role_value_parses ... ok
test objects::session::tests::agent_domain_round_trips ... ok
test objects::seat::tests::serde_shape_is_snake_case ... ok
test objects::session::tests::dimensions_are_not_interchangeable ... ok
test objects::seat::tests::unknown_role_is_a_typed_error_not_a_custom_fallback ... ok
test objects::session::tests::harness_domain_round_trips ... ok
test objects::session::tests::model_id_is_opaque_free_form_text ... ok
test objects::session::tests::serde_shape_is_pascal_case_and_externally_tagged ... ok
test objects::session::tests::unknown_agent_is_a_typed_error_not_a_custom_fallback ... ok
test objects::session::tests::unknown_harness_is_a_typed_error ... ok
test pty::tests::default_transcripts_dir_nests_under_repo_root ... ok
test pty::tests::mirror_dir_is_tmp_seatloom_mirror ... ok
test pty::tests::shell_quote_handles_spaces_and_quotes ... ok
test pty::tests::write_noop_on_empty_bytes ... ok
test storage::artifact_store::tests::subtype_validation_follows_allowlist ... ok
test storage::artifact_store::tests::resolves_artifact_paths_deterministically ... ok
test storage::artifact_store::tests::missing_artifacts_dir_returns_empty_vec ... ok
test pty::tests::write_noop_on_shutdown ... ok
test storage::handoff_store::tests::resolves_handoff_paths_deterministically ... ok
test storage::handoff_store::tests::missing_handoffs_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::delegation_validation_rejects_required_boundary_gaps ... ok
test storage::seat_registry::tests::delegation_round_trips ... ok
test storage::jsonl_io::tests::jsonl_append_and_read_round_trip ... ok
test storage::seat_registry::tests::missing_delegations_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::resolves_seat_layer_paths_deterministically ... ok
test db::reconcile::tests::collect_markdown_files_returns_sorted_paths ... ok
test storage::seat_registry::tests::missing_seats_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::save_delegation_returns_typed_validation_error ... ok
test storage::handoff_store::tests::handoff_round_trip_and_list ... ok
test storage::seat_registry::tests::list_delegations_sorted_by_issued_at_descending ... ok
test storage::session_store::tests::resolves_session_paths_deterministically ... ok
test storage::session_store::tests::missing_sessions_dir_returns_empty_vec ... ok
test storage::workitem_store::tests::resolves_workitem_paths_deterministically ... ok
test storage::artifact_store::tests::artifact_round_trip_and_list ... ok
test storage::workitem_store::tests::missing_workitems_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::role_binding_round_trips ... ok
test storage::seat_registry::tests::seat_identity_round_trips ... ok
test storage::seat_registry::tests::list_seat_identities_sorted_by_name ... ok
test storage::yaml_io::tests::project_config_round_trips_through_yaml ... ok
test storage::session_store::tests::session_round_trip_and_list ... ok
test storage::workitem_store::tests::workitem_round_trip_and_list ... ok
test pty::tests::list_tmux_sessions_returns_vec_without_panic ... ok

test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
[exit 0]
```

### 3.4 `cargo clippy -p seatloom-core -- -D warnings`

```
$ cargo clippy -p seatloom-core -- -D warnings
    Checking seatloom-core v0.1.0 (/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/crates/seatloom-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.20s
[exit 0]
```

> **Reported, because it changes what this line means.** At the packet-declared base commit
> `9599136` this command was **red**, with four errors that this packet did not introduce: three
> `doc_lazy_continuation` at `db/repositories.rs:445-447` and one `trim_split_whitespace` at
> `pty/mod.rs:252`. Verified in a detached worktree at `9599136`:
>
> ```
> $ CARGO_TARGET_DIR=/tmp/seatloom-base-target cargo clippy -p seatloom-core -- -D warnings
>    --> crates/seatloom-core/src/db/repositories.rs:445:9
>    --> crates/seatloom-core/src/db/repositories.rs:446:9
>    --> crates/seatloom-core/src/db/repositories.rs:447:9
>    --> crates/seatloom-core/src/pty/mod.rs:252:46
> error: could not compile `seatloom-core` (lib) due to 4 previous errors
> ```
>
> Done-definition criterion 8 (clippy green) was therefore **unsatisfiable within §4's R1 row**,
> which requires `git show <sha> -- crates/seatloom-core/src/pty/` to be empty — I could not fix
> the `pty/mod.rs` lint without breaking a binding compliance row. `track/infra-foundation` fixed
> all four independently before I merged (`pty/mod.rs` dropped the `trim()`; `repositories.rs` got
> blank lines around the doc list), so the criterion is now met on merit. Recording it because had
> the merge not happened, this delivery would have shipped with criterion 8 failed and the conflict
> unresolved — it was not my fix.

### 3.5 `cd ui && pnpm exec tsc --noEmit`

```
$ cd ui && pnpm exec tsc --noEmit
[exit 0]
```

Zero output, zero errors. Per §2.5 this proves only that the TS types agree with each other — the
Rust⇄TS wire agreement is evidenced separately in §6 below.

### 3.6 `cd ui && pnpm build`

```
$ cd ui && pnpm build

> seatloom-ui@0.1.0 build /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/ui
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
✓ built in 8.79s
[exit 0]
```

### 3.7 `ls -a .seatloom/`

```
$ ls -a .seatloom/
.
..
bootstrap
seat-sessions.tsv
transcripts
[exit 0]
```

`.seatloom/seats/` and `.seatloom/delegations/` still do not exist, so the §2.3.4 precondition
holds and the `SeatRole` representation change breaks no on-disk artifact:

```
$ ls -d .seatloom/seats .seatloom/delegations
ls: .seatloom/delegations: No such file or directory
ls: .seatloom/seats: No such file or directory

$ find .seatloom -name '*.yaml' -o -name '*.yml'
.seatloom/bootstrap/source-map.yaml
```

The single YAML present is the bootstrap source map, not a `SeatIdentity`. **Zero on-disk seat
identity files.** (§1.3 drift on this row is noted in §9.)

### 3.8 `rg -n 'Runtime' crates/ src-tauri/`

```
$ rg -n 'Runtime' crates/ src-tauri/
crates/seatloom-core/src/db/connection.rs:1:use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
crates/seatloom-core/src/db/connection.rs:15:    Ok(config.create_pool(Some(Runtime::Tokio1), NoTls)?)
crates/seatloom-core/src/db/models.rs:19:    /// Runtime triple, mirrored verbatim from `seats.default_harness` /
crates/seatloom-core/src/db/models.rs:60:    /// Runtime triple, mirrored verbatim from `sessions.harness` / `agent` /
crates/seatloom-core/src/objects/session.rs:32:// Runtime triple: harness / agent / model
crates/seatloom-core/src/objects/session.rs:34:// These three replace the former single `Runtime` enum, which collapsed three
crates/seatloom-core/src/objects/session.rs:369:    /// PascalCase, externally tagged — unchanged from the former `Runtime`
[exit 0]
```

Seven hits, each justified — the type `seatloom_core::objects::session::Runtime` no longer exists:

| Hit | Justification |
|---|---|
| `db/connection.rs:1,15` | `deadpool_postgres::Runtime` — third-party tokio-runtime selector, unrelated to the deleted enum |
| `db/models.rs:19,60` | prose in a doc comment: "Runtime triple, mirrored verbatim from …" |
| `objects/session.rs:32,34` | the section banner naming the concept, and the sentence recording that the three types *replace* the former enum |
| `objects/session.rs:369` | a test doc comment recording that the serde shape is unchanged from the former enum |

Lowercase `runtime` also survives in `src-tauri/src/commands/session_cmds.rs:116,234,260,276-282`
(`LiveSessionDto.runtime`, `LaunchRequest.runtime`, values `"tmux"` / `"tmux-mirror"`). **That is a
different concept** — the tmux mirror transport marker, not the agent dimension. It never referenced
the `Runtime` type, §2.5 does not name it, and renaming it would change a live UI command contract
and force a `.tsx` edit, which §3 forbids. Left alone deliberately.

---

## 4. Invariant compliance (§6.4)

| Rule | Required behavior | Evidence |
|---|---|---|
| Option B — no L2 surface | No rendered surface added | `git diff --stat d70a5e0 HEAD` (§1) lists 10 files, **zero `.tsx`**. Only `ui/src/lib/types-dto.ts` (+10/−4, interface fields) and `ui/src/lib/dto-to-v1.ts` (+5/−1, one mapping line) changed under `ui/`, both type-only fixes forced by the DTO change. The V1 display type `Runtime` in `ui/src/types/index.ts` is deliberately **untouched**, so no component re-renders differently. |
| R1 attach-only | No spawn / no PTY behavior change | `git diff d70a5e0 HEAD -- crates/seatloom-core/src/pty/` → **empty**. No file under `pty/` appears in the §1 delta. |
| Review §2 — attribution is revisable | `NULL`/`None` harness+model mean *not recorded*; no guessed backfill | `objects/session.rs:12-17` (`harness: Option<Harness>`, `model: Option<ModelId>`), `objects/seat.rs:18-21` (all three `Option`). `rg -n 'unwrap_or|unwrap_or_else|unwrap_or_default' crates/seatloom-core/src/objects/` → no hit on `Harness`/`Agent`/`ModelId`; there is no `Default` impl on any of the three. `check_domain` (`repositories.rs:1294-1310`) returns `Ok(())` early for `None` rather than substituting a value. |
| Review §5 — fail loudly | Unknown enum string produces a typed error naming the value; not `Custom` fallback, not panic | §6 below: `DbError::ValueDomain` at `repositories.rs:1752-1759`; tests `out_of_domain_agent_fails_loudly_naming_the_row`, `out_of_domain_harness_fails_loudly`, `out_of_domain_seat_default_agent_fails_loudly`, `rendered_error_names_table_row_column_and_value`, `unknown_agent_is_a_typed_error_not_a_custom_fallback`, `unknown_harness_is_a_typed_error`, `unknown_role_is_a_typed_error_not_a_custom_fallback` — all pass (§3.3). `Custom` is reachable only via its explicit string form (`objects/session.rs:126-135`), never as a fallback. |
| AD-011 — PG is canonical | Serialized domains match existing PG values per §2.1 | Contract table restated in §5. `Harness`/`Agent` carry **no** `#[serde(rename_all)]` (`objects/session.rs:56,63`) so variants serialize PascalCase verbatim, matching `sessions.runtime`. `SeatRole` carries `#[serde(rename_all = "snake_case")]` (`objects/seat.rs:78`), matching `project_role_bindings.role`. Test `serde_shape_is_pascal_case_and_externally_tagged` and `serde_shape_is_snake_case` pin both. |
| One parse boundary | Exactly one place converts string → enum | `rg 'impl FromStr for' crates/seatloom-core/src/` → exactly three impls (`session.rs:147` `Harness`, `session.rs:186` `Agent`, `seat.rs:93` `SeatRole`), all reached only through `check_domain` (`repositories.rs:1294`), called from `row_to_seat:1340`, `row_to_role_binding:1369`, `row_to_session:1399`. See §6. |

---

## 5. §2.1 value-domain contract — restated (§6.5)

| Field | Rust type | Serialized domain | PG column | Nullable |
|---|---|---|---|---|
| harness | `Option<Harness>` | `Direct`, `Stepcode`, `Custom(String)` | `TEXT` | yes — `NULL` means *not recorded*, never *none* |
| agent | `Agent` | `ClaudeCode`, `Codex`, `CursorCli`, `GeminiCli`, `Opencode`, `Custom(String)` | `TEXT` | no, on `sessions`; yes on `seats.default_*` |
| model | `Option<ModelId>` (newtype over `String`) | free-form, e.g. `claude-opus-5[1m]` | `TEXT` | yes — `NULL` means *not recorded* |

**No deviation.** All four binding rules implemented as written:

1. PascalCase verbatim, no `rename_all` on `Harness`/`Agent` — `objects/session.rs:56,63`.
2. `ModelId` is a `String` newtype, not an enum — `objects/session.rs:79`. It serializes
   transparently as a bare string (serde newtype-struct rule), pinned by
   `model_id_is_opaque_free_form_text`. No `CHECK`-equivalent, no per-model branching.
3. `None`/`NULL` is never backfilled. No `Default` impl, no `unwrap_or` — see §4 row 3.
4. `Custom(String)` retained on both, with the same externally-tagged serde shape
   (`{"Custom":"…"}`), pinned by `serde_shape_is_pascal_case_and_externally_tagged`.

### 5.1 One point the contract left implicit — resolved, Onyx notified

The contract fixes the **serde** shape of `Custom` but not its **plain-TEXT** spelling in a PG
column, which cannot hold an externally-tagged map. `sessions.runtime` already stores the bare
literal `Custom` for `ses-flux-verify-001`
(`infra/postgres/seed/001_real_collaboration_baseline.sql:67` at `9599136`), and Onyx's 009
backfills `agent := runtime` as a straight copy, so that value had to remain parseable.

Resolution (`objects/session.rs:118-144`): a bare tag `Custom` parses to `Custom("")`; a payload is
spelled `Custom(<payload>)`, matching how the contract table itself writes the domain. `Display`
inverts `FromStr` exactly, pinned by `harness_domain_round_trips` / `agent_domain_round_trips`.

This is a clarification, not a deviation — Onyx's migration writes neither form, so nothing on the
SQL side changes. **Onyx notified in `Onyx-data-seatloom` on 2026-09-28, before their migration
landed** (merged `80be8b5`, accepted `d70a5e0`), together with the three other facts they needed:
that the Rust read path now requires 009, that `project_role_bindings.role` is now strictly
validated, and that my §1.3 has the same dirty-tree defect as their drift finding D-1.

---

## 6. Serde wire-shape evidence (§6.6) and the parse boundary (§6.7)

### 6.1 Wire shape — `#[serde(...)]` lines verbatim, paired with the TypeScript

Read from the attribute, not inferred from the Rust field name. Both structs are
`rename_all = "camelCase"`, so the wire keys are camelCase.

`src-tauri/src/dto.rs:44-55`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeatDto {
    pub id: String,
    pub name: String,
    pub default_harness: Option<String>,
    pub default_agent: Option<String>,
    pub default_model: Option<String>,
    pub capability_tags: Vec<String>,
    pub status: String,
    pub created_at: String,
}
```

`ui/src/lib/types-dto.ts:16-26`:

```ts
export interface SeatDto {
  id: string;
  name: string;
  // Runtime triple (schema 009). `null` means not recorded, not "default".
  defaultHarness: string | null;
  defaultAgent: string | null;
  defaultModel: string | null;
  capabilityTags: string[];
  status: string;
  createdAt: string;
}
```

`src-tauri/src/dto.rs:128-136`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    pub id: String,
    pub seat_id: String,
    pub harness: Option<String>,
    pub agent: String,
    pub model: Option<String>,
    pub native_session_id: Option<String>,
```

`ui/src/lib/types-dto.ts:50-57`:

```ts
export interface SessionDto {
  id: string;
  seatId: string;
  // Runtime triple (schema 009). `harness`/`model` null means not recorded.
  harness: string | null;
  agent: string;
  model: string | null;
  nativeSessionId: string | null;
```

Pairing, checked by eye: `default_harness` → `defaultHarness`, `default_agent` → `defaultAgent`,
`default_model` → `defaultModel`, `harness` → `harness`, `agent` → `agent`, `model` → `model`.
Single-word fields are unaffected by camelCase, so `harness`/`agent`/`model` cross the wire
unchanged; the three `default_*` fields do get re-cased and the TS matches.

**Request structs — checked, and none carry the triple.** `seats` and `sessions` have no Rust write
path at all:

```
$ git grep -n 'INSERT INTO seats\|INSERT INTO sessions\|UPDATE seats\|UPDATE sessions' -- crates/ src-tauri/
(no output — these two tables are read-only from Rust)
```

The two request-shaped structs that do mention a `runtime` are
`src-tauri/src/commands/session_cmds.rs:258-266` (`LaunchRequest`, `rename_all = "camelCase"`) and
`:111-117` (`LiveSessionDto`, likewise). Both carry `pub runtime: String` holding `"tmux"` /
`"tmux-mirror"` — the mirror transport marker, **not** the agent dimension. Their TS counterparts
(`ui/src/lib/types-dto.ts:225-233`, `:236+`) are unchanged. Out of scope per §2.5 and untouched.

### 6.2 The parse boundary — one place, and it rejects garbage

**Where:** `crates/seatloom-core/src/db/repositories.rs:1294-1310`, the generic
`check_domain::<T>(table, row_id, column, value)`. It is the only caller of `FromStr` for these
types anywhere in the crate:

```
$ rg -n 'impl FromStr for' crates/seatloom-core/src/
crates/seatloom-core/src/objects/session.rs:147:impl FromStr for Harness
crates/seatloom-core/src/objects/session.rs:186:impl FromStr for Agent
crates/seatloom-core/src/objects/seat.rs:93:impl FromStr for SeatRole
```

**Called from** exactly three row-hydration functions, the boundary §2.4.4 asks for:

| Function | Line | Validates |
|---|---|---|
| `row_to_seat` | `repositories.rs:1340` | `seats.default_harness`, `seats.default_agent` |
| `row_to_role_binding` | `repositories.rs:1369` | `project_role_bindings.role` |
| `row_to_session` | `repositories.rs:1399` | `sessions.harness`, `sessions.agent` |

All three are now `Result`-returning; the five call sites in the query methods propagate with
`collect()` / `.transpose()` instead of `Ok(... .collect())`, so a malformed row fails the query.

**Why validate rather than retype.** `SeatRow` / `SessionRow` stay faithful `String` mirrors of the
SQL columns, exactly as §2.4.1 requires. `check_domain` parses purely to prove the value is
in-domain and discards the result. Consequence, and the §2.4 choice I was asked to state: **the DTO
layer keeps `String`.** Reason — carrying typed enums to the wire would serialize `Custom` as
`{"Custom":"…"}`, changing the JSON shape from a flat string to a map, which would break
`coerceRuntime` in `ui/src/lib/dto-to-v1.ts` and force a `.tsx` edit that §3 forbids. The invariant
§2.4 actually requires is met: one place turns a string into an enum, and it rejects garbage loudly.

`row_to_seat` also **drops** `default_runtime` from `SeatRow` and its SELECT, mirroring the
`SessionRow` change §2.4.2 mandates and §2.5's "three fields instead of the collapsed one". The PG
*columns* `seats.default_runtime` and `sessions.runtime` are untouched and still exist — §3's
prohibition is on dropping the columns, which no file under `infra/postgres/` in this packet does
(zero such files in the §1 delta).

`model` / `default_model` are deliberately **not** validated: model ids are free-form text per
§2.1 rule 2, so there is no domain to check against (`repositories.rs:1322-1324`).

**The typed error** — `repositories.rs:1752-1759`:

```rust
    /// A stored string is outside the value domain its Rust type declares.
    /// Raised only at the row-hydration parse boundary (see `check_domain`).
    #[error("{table} row {row_id:?}: column {column} is out of domain: {source}")]
    ValueDomain {
        table: &'static str,
        row_id: String,
        column: &'static str,
        #[source]
        source: ParseValueError,
    },
```

### 6.3 Negative-path test output (§2.4.5)

A passing happy path is not evidence the failure mode works, so the failure mode is tested
directly. Seven negative tests, run in isolation:

```
$ cargo test -p seatloom-core --lib -- out_of_domain unknown_ rendered_error_names

running 7 tests
test objects::session::tests::unknown_harness_is_a_typed_error ... ok
test objects::seat::tests::unknown_role_is_a_typed_error_not_a_custom_fallback ... ok
test db::repositories::tests::out_of_domain_seat_default_agent_fails_loudly ... ok
test db::repositories::tests::out_of_domain_harness_fails_loudly ... ok
test db::repositories::tests::rendered_error_names_table_row_column_and_value ... ok
test objects::session::tests::unknown_agent_is_a_typed_error_not_a_custom_fallback ... ok
test db::repositories::tests::out_of_domain_agent_fails_loudly_naming_the_row ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 70 filtered out
```

`out_of_domain_agent_fails_loudly_naming_the_row` hydrates a session row with `agent = "Bogus"` and
asserts the variant, table, row id, column, inner domain and inner value. `9bef948` adds
`rendered_error_names_table_row_column_and_value`, which pins the exact operator-facing string, so
a later refactor cannot silently drop a field from it:

```
sessions row "ses-flux-verify-001": column agent is out of domain: "Bogus" is not a valid Agent value (expected one of: ClaudeCode, Codex, CursorCli, GeminiCli, Opencode, Custom, Custom(<payload>))
```

Table, row id, column, offending value and the full expected domain — no `Custom(...)` fallback, no
panic. `null_harness_and_agent_are_accepted` pins the converse: `NULL` is a legitimate state, not an
error (review §2).

**Testability note, stated rather than glossed.** `check_seat_domains` / `check_session_domains`
(`repositories.rs:1316`, `:1330`) are split out of `row_to_seat` / `row_to_session` precisely so the
failure path is reachable without constructing a `tokio_postgres::Row`, which has no public
constructor. The tests call those functions with the same arguments the `r.get(n)` calls above them
produce. The `Row`-getter wrappers themselves are thin and untested — that is the one gap in this
evidence, and it is a consequence of the driver's API, not a choice.

---

## 7. Dimension placement — `stepcode` is a harness, `opencode` is an agent (§6.8)

Review §4 flagged opencode and stepgo/stepcode as "missing variants". Under the triple they resolve
by **dimension**, not by adding variants to one enum:

- **`stepcode` / `stepgo` is a HARNESS.** It is a process wrapper and credential/proxy layer; it
  does not drive a conversation. It is `Harness::Stepcode`.
- **`opencode` is an AGENT.** It is a CLI being driven. It is `Agent::Opencode`.

One enum could not express that distinction; three fields can. This is recorded in the code itself
at `crates/seatloom-core/src/objects/session.rs:36-39`, in the section banner, so it is visible to
someone editing the enums rather than only to someone reading this doc:

```rust
// Dimension placement is load-bearing and must not be re-flattened:
//   - `stepcode` / `stepgo` is a HARNESS, not an agent.
//   - `opencode` is an AGENT, not a harness.
```

and pinned by a test, `dimensions_are_not_interchangeable`, which asserts
`Agent::try_from("Stepcode").is_err()` and `Harness::try_from("Opencode").is_err()`.

**Do not add `Stepcode` as an `Agent` variant, and do not add `Opencode` as a `Harness` variant.**
Doing either re-creates the collapse this packet exists to undo.

The casing asymmetry is also deliberate and recorded in both files: `Harness`/`Agent` are PascalCase
(`objects/session.rs:40-45`), `SeatRole` is snake_case (`objects/seat.rs:69-77`). Both follow
*match what is already in PostgreSQL*; the existing data is inconsistent. Harmonizing means
rewriting historical seed rows, which is out of scope.

---

## 8. Scope decisions and what was deliberately not done

| Item | Decision |
|---|---|
| `Runtime` deprecated alias | **Not** kept, per §2.2.3. No compatibility shim was needed — every caller was reachable in this packet. |
| `adapter/traits.rs` | `fn runtime(&self) -> Runtime` became `fn agent(&self) -> Agent` (`traits.rs:31`): an adapter identifies the *agent* dimension; harness and model are session-level facts. `AgentAdapter` has **zero implementors** (`rg 'AgentAdapter' crates/ src-tauri/` → the trait declaration and its module comment only; `claude.rs`, `codex.rs`, `generic.rs`, `wrapper.rs` are 3-line stubs), so the blast radius is nil. |
| `crates/seatloom-ingest/` | Not created. Review §8 item 3, separate packet. No adapter, no `HarnessAdapter` trait, no probe code. |
| SQL / `infra/postgres/` | Not written. Zero files under `infra/postgres/` in the §1 delta. Onyx's packet owns it and has landed. |
| `seats.default_runtime` / `sessions.runtime` columns | Not dropped by any route. Retirement stays deferred to a follow-up after Flux confirms no reader references them. |
| Historical seed rows | Not rewritten. Nothing was edited to make anything typecheck. |
| Model-specific logic | None. `ModelId` is an opaque string newtype — no routing, no capability table, no per-model branching. |
| `ui/src/types/index.ts` V1 `Runtime` type | **Deliberately untouched.** It is the V1 display model consumed by ~8 `.tsx` files. `sessionFromDto` maps `coerceRuntime(d.agent)` onto it (`dto-to-v1.ts:175-178`), so the agent dimension reaches the existing UI unchanged and harness/model are simply not surfaced yet. Surfacing them is an L2 change, deferred until L1 ships. **No `.tsx` file needed to change and none did.** |
| `LiveSessionDto.runtime` / `LaunchRequest.runtime` | Untouched — tmux transport marker, different concept. See §3.8. |

---

## 9. Contradictions with packet §1.3 (§6.9)

Every §1.3 row was re-measured with `git show 9599136:<path>`, not by reading the working tree.
**Two rows are wrong. Both were measured against a dirty working tree** — the same defect class as
Onyx drift finding D-1.

### 9.1 Row "DB session row" — off by one

§1.3 cites `crates/seatloom-core/src/db/models.rs:52` for `runtime: String`. At `9599136`:

```
$ git show 9599136:crates/seatloom-core/src/db/models.rs | sed -n '52p'
    pub seat_id: String,
$ git show 9599136:crates/seatloom-core/src/db/models.rs | sed -n '53p'
    pub runtime: String,
```

The field is at **`:53`**, not `:52`. Cosmetic, but the same citation appears in the Onyx packet
§2.4, so both inherited it. §2.4.2's own range (`db/models.rs:50-64`) is correct.

### 9.2 Row "Seed role values" — wrong line range, wrong value set. Material.

§1.3 claims: *`infra/postgres/seed/001_real_collaboration_baseline.sql:40-45` → `supervisor`,
`product_owner`, `designer`, `architect`, `verifier`, `data_engineer` — snake_case.*

At `9599136`, lines 40-45 are not role rows at all:

```
$ git show 9599136:infra/postgres/seed/001_real_collaboration_baseline.sql | sed -n '40,45p'
-- =============================================================================
-- Seat Delegation: Flux acting for Mira (WI-009 contract repair)
-- Evidence: MEMORY.md 2026-04-28 "Flux temporarily reassigned as acting Mira seat"
-- =============================================================================
INSERT INTO seat_delegations (id, issuer_seat_id, from_seat_id, to_seat_id, workitem_id, scope_description, issued_at, expires_at, status)
VALUES (
```

The role rows are at **`:33-37`**, and there are **five**, with **no `data_engineer`**:

```
$ git show 9599136:infra/postgres/seed/001_real_collaboration_baseline.sql | sed -n '33,37p'
  ('seat-aegis-001',  'seatloom', 'supervisor',     ...),
  ('seat-lyra-001',   'seatloom', 'product_owner',  ...),
  ('seat-mira-001',   'seatloom', 'designer',       ...),
  ('seat-nimbus-001', 'seatloom', 'architect',      ...),
  ('seat-flux-001',   'seatloom', 'verifier',       ...)
```

`data_engineer` enters at `:45` only in `18a69fb`, Onyx's separate commit carrying the
already-uncommitted 2026-09-28 seat convergence.

**Why this matters and what I did about it.** §2.3.3 justifies adding `SeatRole::DataEngineer`
*because* "the seed already writes `data_engineer` at `seed/001…:45`". At the packet's declared base
commit it does not. I did **not** work around it and I did **not** drop the variant. The underlying
evidence the packet also cites is sound and independent of the seed: `docs/coordination/memory/
2026-09-28.md` Addendum decision 3 promotes onyx to the sixth registered seat, and as of `18a69fb`
— now an ancestor of this branch — the seed does write `data_engineer` at `:45`. So the variant is
correct on the merits; only the citation was to a line that did not yet exist. Both `Supervisor` and
`DataEngineer` are kept, and `every_seeded_role_value_parses` asserts all six values parse.

The stale citation was corrected in the code comments rather than reproduced: `objects/seat.rs:69-77`
and the test doc comment now name `:33-37` at `9599136` plus `:45` at `18a69fb` explicitly.

### 9.3 Row "File-backed seat store" — drifted, but the load-bearing part holds

§1.3: *`.seatloom/` contains only `bootstrap/` and `transcripts/`.* It now also contains
`seat-sessions.tsv`, written by the seat-launch tooling after the packet was authored (§3.7). The
load-bearing claim is unaffected: `.seatloom/seats/` and `.seatloom/delegations/` still do not
exist and there are **zero** on-disk `SeatIdentity` YAML files, so the §2.3.4 `SeatRole`
representation change breaks no existing artifact. Verified in §3.7, not assumed.

### 9.4 §2.6's construction-site list is incomplete

§2.6 names `seat_registry.rs:188, 245, 261, 432, 440` and `seat.rs:7` — all five verified correct at
`9599136`. The sweep it asks for found two more files it did not list:

```
$ git grep -n 'Runtime' 9599136 -- crates/seatloom-core/src/adapter/ crates/seatloom-core/src/storage/session_store.rs
9599136:crates/seatloom-core/src/adapter/traits.rs:4:use crate::objects::session::Runtime;
9599136:crates/seatloom-core/src/adapter/traits.rs:31:    fn runtime(&self) -> Runtime;
9599136:crates/seatloom-core/src/storage/session_store.rs:81:    use crate::objects::session::{Runtime, Session, SessionStatus};
9599136:crates/seatloom-core/src/storage/session_store.rs:103:            runtime: Runtime::ClaudeCode,
```

Both updated. Not a defect in the packet — §2.6 explicitly delegates the sweep — but recorded so the
count reconciles: 4 files carried `Runtime` constructions, not 2.

### 9.5 Rows verified correct at `9599136`

`Runtime` enum at `session.rs:23-30` (5 variants, no serde attrs) · `Session.runtime` at
`session.rs:10` · `SeatIdentity.default_runtime` at `seat.rs:16` · `SeatRole` at `seat.rs:57-64`
(4 variants + `Custom`, no `Supervisor`, no `DataEngineer`, no serde attrs) · `SeatRow.
default_runtime` at `models.rs:19` · row hydration `r.get(2)` at `repositories.rs:1276` and `:1323`
· DTO at `dto.rs:49,60,129,147` · PG columns at `001_seatloom_core.sql:24` and `:65`.

### 9.6 Working-tree condition during this packet — disclosed

All six seats share one checkout. While I was authoring `b10280d`, Onyx was working in the same
tree on `packet/onyx/…`; the tree's HEAD moved from `9599136` to `18a69fb` mid-session. My 10
Rust/TS files and Onyx's SQL/seed/script files had **zero path overlap**, so neither packet's
verification was contaminated. `b10280d` was cut with a temporary index
(`GIT_INDEX_FILE` + `write-tree` + `commit-tree`) rather than by switching HEAD, so Onyx's index and
working tree were never disturbed; my files were then removed from the shared tree once the branch
held them. Onyx's `18a69fb` correctly excluded every `.rs` file. Recorded because a shared checkout
is a standing hazard for commit-pinned gates, not because anything went wrong here.

---

## 10. Done-definition status (§7)

| # | Criterion | Status |
|---|---|---|
| 1 | `Runtime` gone; `rg` zero hits or each justified | **PASS** — §3.8, 7 hits, each justified |
| 2 | `Session` carries the triple | **PASS** — `objects/session.rs:12-17` |
| 3 | `SeatIdentity` carries `default_harness`/`_agent`/`_model` | **PASS** — `objects/seat.rs:18-21` |
| 4 | `SeatRole` has `Supervisor` + `DataEngineer` + snake_case | **PASS** — `objects/seat.rs:78-88` |
| 5 | Domains match §2.1, or deviation documented + Onyx notified | **PASS** — no deviation; the implicit `Custom` TEXT spelling resolved and Onyx notified 2026-09-28 before their migration landed (§5.1) |
| 6 | One parse boundary, called at row hydration, typed error with value + row id | **PASS** — §6.2 |
| 7 | §2.4.5 negative test exists and passes | **PASS** — §6.3, 7 negative tests |
| 8 | `cargo check` ×2, `cargo test --lib`, `cargo clippy -D warnings` green, verbatim | **PASS** — §3.1-3.4, 77 tests. Clippy was red at `9599136` for 4 pre-existing reasons and is green after the `track` merge; see the §3.4 note |
| 9 | `tsc --noEmit` zero errors; `pnpm build` exit 0 | **PASS** — §3.5, §3.6 |
| 10 | `git show --stat` zero `.tsx`, zero `infra/postgres/` | **PASS** — §1 delta and §2 |
| 11 | Delivery doc contains all nine §6 items | **PASS** — §6.1↔§2, §6.2↔§3, §6.3↔§4, §6.4↔§5, §6.5↔§6.1, §6.6↔§6.2-6.3, §6.7↔§7, §6.8↔§9 (frontmatter is item 1) |
| 12 | Flux commit-pinned Layer A PASS | **PENDING** — Flux gate against the tip SHA above |

---

*Nimbus · 2026-09-30 · packet NIMBUS-2026-09-28-runtime-triple-refactor-v1 · Aegis review §8 item 1
· option B (no UI surface until L1 ships) · paired with ONYX-2026-09-28-runtime-triple-migration-v1
(landed `80be8b5`, accepted `d70a5e0`) via the §2.1 value-domain contract*
