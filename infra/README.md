# SeatLoom v0.1 — Running the System

This guide brings up a running SeatLoom v0.1 on a local machine: PostgreSQL +
coordination document authority + CLI reconcile. The Tauri desktop app wiring
is Phase 1 and runs on top of this foundation.

## Prerequisites

- Docker Desktop (or any Docker daemon with `docker compose`)
- Rust toolchain (install via https://rustup.rs)
- Node.js ≥ 20 and pnpm (only needed to launch the Tauri desktop app)

## One-command bootstrap

From the repository root:

```bash
./scripts/bootstrap.sh
```

This is **idempotent** — safe to run repeatedly. It will:

1. Start the `seatloom-postgres` container (Postgres 16-alpine).
2. Wait for the database to be queryable.
3. Apply all five schema files (`infra/postgres/schema/00{1..5}_*.sql`). Tables
   use `CREATE TABLE IF NOT EXISTS`, so re-runs are no-ops.
4. Apply the four seed files (`infra/postgres/seed/00{1..4}_*.sql`). Rows use
   `ON CONFLICT DO NOTHING`, so re-runs are no-ops.
5. Run the CLI reconcile: scans `docs/` markdown, parses T1–T7 typed headers,
   and upserts into the `documents` / `document_sections` / `document_versions`
   tables.
6. Print a per-template document count summary.

Expected output on a first run is roughly:

```
scanned=271 inserted=271 updated=0 unchanged=0 failed=0 conflicted=0
```

On subsequent runs (no markdown changes):

```
scanned=271 inserted=0 updated=0 unchanged=271 failed=0 conflicted=0
```

## What just happened

After bootstrap the local database contains:

| Table                    | Approx rows (baseline) | Source                                                  |
|--------------------------|-----------------------|----------------------------------------------------------|
| `projects`               | 1                      | seed 001 (SeatLoom)                                     |
| `seats`                  | 5                      | seed 001 (aegis, lyra, mira, nimbus, flux)             |
| `sessions`               | 5                      | seed 001                                                |
| `workitems`              | 9                      | seed 001                                                |
| `handoffs`               | 3                      | seed 001                                                |
| `canonical_events`       | ~20                    | seed 001                                                |
| `documents`              | ~271                   | reconcile (live `docs/` markdown)                       |
| `document_sections`      | ~2000                  | reconcile (projected headings)                          |
| `document_versions`      | ~271                   | reconcile (revision snapshots)                          |
| `reconcile_runs`         | 1 per bootstrap        | reconcile                                               |
| `checkpoints`            | several                | seed 003                                                |
| `review_threads`         | several                | seed 003                                                |
| `prompt_instances`       | 0                      | schema 005 baseline (runtime-only)                      |
| `channel_action_receipts`| 0                      | schema 005 baseline (runtime-only)                      |

## Inspecting the database manually

```bash
docker exec -it seatloom-postgres psql -U seatloom -d seatloom
```

Useful queries:

```sql
-- Document template distribution
SELECT template, count(*) FROM documents GROUP BY template ORDER BY template;

-- Full-text search over document body (uses GIN tsvector on body + title + tags)
SELECT id, title, template, subtype
FROM documents
WHERE to_tsvector('english', coalesce(body_text,'') || ' ' || coalesce(title,''))
      @@ plainto_tsquery('english', 'chan-09')
ORDER BY updated_at DESC;

-- Recent reconcile runs
SELECT id, trigger, scanned, inserted, updated, unchanged, failed, conflicted,
       started_at, completed_at
FROM reconcile_runs
ORDER BY started_at DESC
LIMIT 5;

-- Documents that failed to parse (subtype not in allow-list)
SELECT file_path, parse_status FROM documents WHERE parse_status = 'partial';
```

## Teardown (destructive)

```bash
./scripts/teardown.sh           # prompts for confirmation
FORCE=1 ./scripts/teardown.sh   # skip prompt
```

This stops the container and deletes the Postgres volume. Markdown source
files under `docs/` are not touched.

## Launching the Tauri desktop app (Phase 1)

Once bootstrap has populated the database:

```bash
cd ui
pnpm install
pnpm tauri dev
```

The Tauri shell connects to the same `DATABASE_URL`
(`postgresql://seatloom:seatloom@localhost:5432/seatloom`) by default. Override
via env var if your local Postgres listens elsewhere.

## CLI operations

The `seatloom` CLI (`src-cli`) exposes reconcile directly:

```bash
# Manually trigger reconcile (without bootstrap)
cargo run -p seatloom-cli -- reconcile --project seatloom --root .
```

## Troubleshooting

**"docker exec seatloom-postgres ... no such container"**
The container isn't running. Re-run `./scripts/bootstrap.sh`, or start it
manually: `cd infra/postgres && docker compose up -d`.

**"seatloom database not queryable within 60s"**
Bootstrap raises this if Postgres doesn't become queryable in time. Increase
the timeout: `PG_READY_TIMEOUT=180 ./scripts/bootstrap.sh`. If it still fails,
check `docker logs seatloom-postgres --tail 40`.

**Reconcile reports "failed" items**
Query `reconcile_items` for the failure reason:
```sql
SELECT file_path, failure_reason
FROM reconcile_items
WHERE outcome = 'failed'
  AND run_id = (SELECT id FROM reconcile_runs ORDER BY started_at DESC LIMIT 1);
```
Most common cause: file header uses a template/subtype not in the allow-list
(`crates/seatloom-core/src/db/document_parser.rs::validate_subtype`). Fix the
file header or extend the allow-list — do not silently accept invalid types.

**Reconcile reports "conflicted" items**
Same `doc_id` in header, different `file_path` in the DB. Inspect:
```sql
SELECT file_path, failure_reason
FROM reconcile_items
WHERE outcome = 'conflict'
  AND run_id = (SELECT id FROM reconcile_runs ORDER BY started_at DESC LIMIT 1);
```
Resolution: either rename the duplicate file's `doc_id`, or remove the stale
`documents` row and re-run bootstrap.
