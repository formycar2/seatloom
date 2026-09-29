# Delivery: Remote Workspace PostgreSQL Review + Continuity Re-Verification

## 1. Verdict

**HOLD**

The narrowed infra verification did prove the exact pinned commit and the narrowed Rust surface on the sponsor-provided remote workspace:
- `$HOME/.cargo/bin/cargo check -p seatloom-core` passed,
- `$HOME/.cargo/bin/cargo test -p seatloom-core` passed.

However, the primary proof point `bash scripts/verify-postgres-baseline.sh` failed on the exact target commit `ee0159fa440b910de38b29824574760722a18316` due to a real infrastructure defect in the document seed/bootstrap path, not the previously irrelevant GTK/Tauri preflight.

## 2. Execution environment

- Remote login used exactly as required: `ssh -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com`
- Remote host: `buildthoughtonly`
- Remote OS: `Linux buildthoughtonly 5.10.25-nvidia-gpu #1 SMP Sat May 6 02:51:26 UTC 2023 x86_64 x86_64 x86_64 GNU/Linux`
- Remote working directory: `/data/seatloom-verify/repo`
- Docker: `/usr/bin/docker`
- Docker version: `Docker version 29.4.0, build 9d7ad9f`
- Docker Compose version: `Docker Compose version v5.1.2`
- Cargo used for the narrowed infra path: `$HOME/.cargo/bin/cargo`
- Cargo version: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`

## 3. Command results matrix

| Command | Result | Notes |
|---|---|---|
| `ssh -o BatchMode=yes -o ConnectTimeout=10 -CAXY buildthoughtonly.zhangxiaolong.shai-core.ws@platform.shaipower.com 'uname -a && hostname'` | PASS | Remote connectivity proved. |
| remote fetch + checkout in `/data/seatloom-verify/repo` | PASS | Detached `HEAD` at the exact pinned commit. |
| `git rev-parse HEAD` | PASS | Returned `ee0159fa440b910de38b29824574760722a18316`. |
| `git status --short` | PASS | Empty output. |
| `$HOME/.cargo/bin/cargo check -p seatloom-core` | PASS | Narrowed infra crate compiled on the remote seat. |
| `$HOME/.cargo/bin/cargo test -p seatloom-core` | PASS | 52 unit tests passed; DB integration tests remained ignored as expected in this non-DB run. |
| `bash scripts/verify-postgres-baseline.sh` | FAIL | Primary end-to-end PostgreSQL verification failed on the pinned commit. |
| fallback `docker compose down -v && docker compose up -d --wait` | PASS | Clean DB reset and healthy `seatloom-postgres` container reproduced the failing surface. |
| fallback `docker exec ... SELECT COUNT(*) ...` for all 5 new families | PASS as diagnostic, FAIL as acceptance signal | All 5 tables existed, but each count was `0` after the clean reset because the expected seeded baseline was not present. |
| fallback `$HOME/.cargo/bin/cargo test -p seatloom-core -- --include-ignored` | FAIL | 20 DB integration tests failed against the empty baseline. |

Exact failing primary command:

```bash
cd /data/seatloom-verify/repo
bash scripts/verify-postgres-baseline.sh
```

Exact blocking error chain inside the primary script:

```text
Applying seed/002_document_seed.sql
psql:/tmp/002_document_seed.sql:190: ERROR:  insert or update on table "documents" violates foreign key constraint "documents_artifact_id_fkey"
DETAIL:  Key (artifact_id)=(ar-task-db-baseline) is not present in table "artifacts".
...
ROLLBACK
...
failures:
    db_baseline::active_contract_set_documents_present
    db_baseline::document_associations_link_to_project_and_workitems
    db_baseline::document_sections_seeded_and_ordered
    db_baseline::document_template_filter_works
    db_baseline::documents_cover_t1_through_t7
    db_baseline::documents_seeded_for_project
    db_baseline::reconcile_run_recorded_after_run
```

Probable root cause:

- `infra/postgres/seed/002_document_seed.sql` inserts document rows that reference artifact id `ar-task-db-baseline`, but that artifact is not present in the seeded artifact baseline available at this exact commit.
- `scripts/verify-postgres-baseline.sh` calls `psql -f` for seed files without `-v ON_ERROR_STOP=1`, so the script continues after the seed transaction rolls back and only fails later when the DB integration tests observe missing document truth.

Smallest safe owner/fix scope:

- Infrastructure owner for the PostgreSQL seed/bootstrap path should fix the document seed reference mismatch and make the verification script fail immediately on SQL errors.
- No product-code or UI scope is implicated by this blocker.

## 4. SQL spot-check results

Primary-script-pass row spot-checks were **not run**, because the packet entered fallback isolation after the main proof point failed.

Fallback diagnostic SQL results after a clean Docker reset:

- `SELECT COUNT(*) FROM checkpoints;` => `0`
- `SELECT COUNT(*) FROM handoff_receipts;` => `0`
- `SELECT COUNT(*) FROM pipeline_runs;` => `0`
- `SELECT COUNT(*) FROM review_threads;` => `0`
- `SELECT COUNT(*) FROM review_comments;` => `0`

Interpretation:

- The 5 new canonical families are present as tables on the remote seat.
- The expected seeded baseline was not present in the fallback-reset database, which is consistent with the primary-script failure path and the later DB integration test failures.
- Therefore Lyra's expected seeded counts for schema 004 / seed 003 were **not** proven on this exact commit.

Answers to the required verification questions:

1. Was the exact pinned commit `ee0159fa440b910de38b29824574760722a18316` verified?
   - **Yes.** `git rev-parse HEAD` returned the exact target commit.
2. Does `cargo check -p seatloom-core` pass on the remote seat?
   - **Yes.** It completed successfully.
3. Does `cargo test -p seatloom-core` pass on the remote seat?
   - **Yes.** The narrowed non-DB test command passed.
4. Does `bash scripts/verify-postgres-baseline.sh` pass end to end?
   - **No.** It fails on the pinned commit because the document seed phase rolls back and later DB integration tests fail.
5. Are the 5 new canonical families present with the expected seeded counts?
   - **No.** The families exist, but the expected seeded counts were not present in the fallback-reset DB, and the primary script did not complete successfully.
6. Do the included DB integration tests pass materially enough to accept schema 004 / seed 003?
   - **No.** The end-to-end included-ignored DB path fails materially because the baseline document/association seed is broken, even though the new schema-004 family-specific checks that depend on data would only be meaningful after the baseline seed succeeds.
7. Is there any actual infra blocker after removing the irrelevant GTK/Tauri preflight?
   - **Yes.** A real PostgreSQL seed/bootstrap blocker remains in the infra verification path.

## 5. Findings / blockers

- Real blocker: `infra/postgres/seed/002_document_seed.sql` references missing artifact id `ar-task-db-baseline`, causing a foreign-key failure and a transaction rollback during `bash scripts/verify-postgres-baseline.sh`.
- Hidden operational flaw: `scripts/verify-postgres-baseline.sh` does not stop immediately on `psql` seed errors because it does not force `ON_ERROR_STOP`, so the script prints later progress and only fails during DB integration tests.
- Impact on this packet: the narrowed Rust surface is healthy, but the repo-managed PostgreSQL baseline verification path is not acceptance-ready at commit `ee0159fa440b910de38b29824574760722a18316`.
- Scope discipline held: no code edits, no UI widening, no product behavior review.

## 6. Evidence file paths

Generated:
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/ssh-probe.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/git-target.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/remote-env.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/cargo-check-seatloom-core.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/cargo-test-seatloom-core.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/verify-postgres-baseline.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/docker-ps.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-checkpoints.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-handoff-receipts.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-pipeline-runs.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-review-threads.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-review-comments.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-review-thread-rows.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-checkpoint-rows.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-handoff-receipt-rows.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/sql-pipeline-run-rows.txt`
- `.local/evidence/2026-04-30-remote-workspace-postgres-review-continuity-reverification-v2/cargo-test-include-ignored.txt`

## 7. Recommended next action

- Keep this packet at **HOLD**.
- Re-scope to the smallest infra fix packet for the PostgreSQL baseline path:
  1. resolve the missing `ar-task-db-baseline` artifact reference in `002_document_seed.sql`,
  2. make `scripts/verify-postgres-baseline.sh` fail immediately on SQL errors,
  3. rerun this same remote commit-pinned verification flow on the resulting fix commit.
