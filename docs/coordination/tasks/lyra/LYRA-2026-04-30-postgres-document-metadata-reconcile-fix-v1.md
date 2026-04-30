# Delivery: PostgreSQL Document Metadata Reconcile Fix

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-04-30-postgres-document-metadata-reconcile-fix-v1 |
| status | delivered |
| author | lyra |
| date | 2026-04-30 |
| version | v1 |
| depends_on | `docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v4.md`, `docs/coordination/tasks/lyra/LYRA-2026-04-30-postgres-reconcile-verification-alignment-v1.md`, `crates/seatloom-core/src/db/document_parser.rs`, `crates/seatloom-core/src/db/reconcile.rs` |
| tags | postgres, infrastructure, documents, reconcile, metadata, commit-pinned, hardening |
| owner | Lyra |

## Scope

This was a bounded infrastructure-only repair to the typed document ingest/reconcile path.

Out of scope:
- UI or frontend code
- schema redesign
- seed content expansion
- runtime API work
- document authoring cleanup
- non-infra coordination cleanup

## Trigger

Flux v4 remote verification on commit `62f6f901655e3b9a92acae8d84f8277fc126fdd8` returned a correct `HOLD`.

Exact blocker:
- `crates/seatloom-core/src/db/reconcile.rs` updated existing `documents.template` and `documents.subtype` from parsed Markdown header values even when the current file body did not contain canonical `template` / `subtype` rows.
- That wiped seed-provided typed metadata to `NULL` during reconcile.

Material runtime evidence captured by Flux:
- `reconcile_runs=1`
- `document_versions=230`
- `T1AuthorityDoc=0`
- `NULL-template documents=106`

Failing DB tests:
1. `db_baseline::documents_cover_t1_through_t7`
2. `db_baseline::document_template_filter_works`

## Root Cause

There were two bounded implementation defects in the typed document import path:

1. `parse_header()` scanned fenced Markdown examples, so governance/template-spec documents could surface bogus header values from code blocks.
2. `process_one_file()` treated missing in-file metadata as authoritative on update, overwriting the seeded `template` / `subtype` with `NULL` instead of preserving the existing typed DB identity.

This was incompatible with the real repository because the live doc corpus still contains mixed header styles:
- canonical `template` / `subtype` rows in some files,
- legacy short aliases such as `T1`, `T3`, etc. in some files,
- and many valid files with seeded authority metadata but no canonical header rows yet.

## Delivered

### 1. Hardened Markdown header parsing

Updated `crates/seatloom-core/src/db/document_parser.rs` to:
- ignore fenced code blocks while scanning header tables,
- normalize short template aliases (`T1`..`T7`) into canonical DB families (`T1AuthorityDoc` .. `T7GovernanceDoc`),
- keep existing subtype validation intact,
- add focused unit tests for both regressions.

### 2. Preserved seeded typed metadata during reconcile

Updated `crates/seatloom-core/src/db/reconcile.rs` so that existing documents now resolve typed identity as:
- parsed header `template` / `subtype` when explicitly present,
- otherwise the current DB `template` / `subtype` already seeded for that file path.

This changes reconcile behavior from “missing header field clears typed identity” to “missing header field preserves existing typed identity”.

### 3. Parse-status alignment

`parse_status` now derives from the resolved typed identity rather than only the raw in-file header, so existing seeded documents are not incorrectly downgraded after a no-op reconcile.

## Files Changed

Updated:
- `crates/seatloom-core/src/db/document_parser.rs`
- `crates/seatloom-core/src/db/reconcile.rs`

## Validation

Passed locally on the bounded infra surface:
- `$HOME/.cargo/bin/cargo test -p seatloom-core document_parser -- --nocapture`
- `$HOME/.cargo/bin/cargo test -p seatloom-core reconcile -- --nocapture`
- `$HOME/.cargo/bin/cargo test -p seatloom-core`
- `$HOME/.cargo/bin/cargo check -p seatloom-core`
- `$HOME/.cargo/bin/cargo fmt --all --check`
- `$HOME/.cargo/bin/cargo clippy -p seatloom-core --all-targets -- -D warnings`

Environment note:
- Docker is still unavailable on this seat, so end-to-end PostgreSQL proof remains commit-pinned remote verification work for Flux.

## Commit Identity

Infrastructure repair commit pushed for verification:
- branch: `track/infra-foundation`
- target commit: `dc01d54411f81f1a5798af57e52e52efb58ac7ec`
- compare base commit: `62f6f901655e3b9a92acae8d84f8277fc126fdd8`

## Next Step

Issue one more commit-pinned Flux packet against commit `dc01d54411f81f1a5798af57e52e52efb58ac7ec` on the sponsor-provided docker-capable workspace.

The packet should prove:
1. the exact commit was checked out,
2. the narrowed `seatloom-core` Rust surface still passes,
3. `bash scripts/verify-postgres-baseline.sh` now passes end to end,
4. typed document families remain materially present after reconcile,
5. `documents.template IS NULL` returns `0` after bootstrap.
