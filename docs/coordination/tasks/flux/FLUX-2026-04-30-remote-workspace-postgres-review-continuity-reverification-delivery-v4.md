[Flux -> Lyra] PostgreSQL Review + Continuity Re-Verification v4
completed:
- commit identity verified: 62f6f901655e3b9a92acae8d84f8277fc126fdd8 on track/infra-foundation
- cargo check -p seatloom-core: passed (0.13s)
- cargo test -p seatloom-core unit tests: passed (52/52)
- scripts/verify-postgres-baseline.sh: FAILED (db_baseline_integration: 22 passed, 2 failed)
- reconcile bookkeeping: reconcile_runs=1, document_versions=230 (present)
- seeded families: T2=7, T3=72, T4=8, T5=36, handoff_receipts=3, checkpoints=3, pipeline_runs=1, review_threads=1, review_comments=2
- **T1AuthorityDoc family: 0 (expected 7) — all 7 T1 docs have NULL template**
- **NULL-template documents: 106 (includes all T1 Authority docs, many T4/T5/T6/T7)**
validation:
- `git rev-parse HEAD` => 62f6f901655e3b9a92acae8d84f8277fc126fdd8
- `$HOME/.cargo/bin/cargo check -p seatloom-core` => Finished dev profile in 0.13s
- `$HOME/.cargo/bin/cargo test -p seatloom-core` => 52 passed; 0 failed
- `bash scripts/verify-postgres-baseline.sh` => FAILED with 2 integration test failures
- SQL spot-checks => reconcile_runs=1, document_versions=230, T1AuthorityDoc=0, NULL-template=106
blockers:
- crates/seatloom-core/src/db/reconcile.rs:487 — reconcile UPDATE sets `template=$7` where $7 is `None` for any document whose Markdown header lacks an explicit `template:` field, overwriting the seed-provided template value with NULL. This affects all 7 T1 Authority docs (PRD, UX Spec, Interaction Spec, Acceptance Spec, Architecture Decisions, Architecture Design, Product Truth) plus 99 other documents across T4/T5/T6/T7 that also lack `template:` in their frontmatter.
  - Evidence: 106 documents with NULL template after reconcile; seed SQL `infra/postgres/seed/002_document_seed.sql` correctly inserts `template='T1AuthorityDoc'` but reconcile UPDATE (line 487) overwrites to NULL.
  - Failing tests: `db_baseline::documents_cover_t1_through_t7` (missing T1AuthorityDoc), `db_baseline::document_template_filter_works` (got 0 T1 docs)
verdict:
- HOLD
next action:
- code fix required in reconcile.rs: preserve existing template when header has no template field (UPDATE should only set template=$7 when $7 IS NOT NULL), or derive template from subtype/template allowlist when header is silent
artifact path(s):
- docs/coordination/tasks/flux/FLUX-2026-04-30-remote-workspace-postgres-review-continuity-reverification-delivery-v4.md
- .local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v4/
