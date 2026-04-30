#!/usr/bin/env bash
# ingest-documents.sh
# Reads actual repository markdown files and upserts their full body content
# into the PostgreSQL documents authority table.
# This script is the authoritative path for populating document body_text.
# The seed file (002_document_seed.sql) provides metadata + excerpts only.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck source=/dev/null
  . "$HOME/.cargo/env"
fi

export DATABASE_URL="${DATABASE_URL:-postgresql://seatloom:seatloom@localhost:5432/seatloom}"

PSQL_CMD=(docker exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom)

echo "=== SeatLoom Document Body Ingest ==="
echo "repo root: $REPO_ROOT"
echo "database:  $DATABASE_URL"
echo ""

ingest_doc() {
  local doc_id="$1"
  local file_path="$2"
  if [ ! -f "$REPO_ROOT/$file_path" ]; then
    echo "  SKIP (missing): $file_path"
    return
  fi
  local body
  body=$(cat "$REPO_ROOT/$file_path")
  local body_len=${#body}
  local digest
  digest=$(echo -n "$body" | shasum -a 256 | cut -d' ' -f1)
  local updated
  # Use dollar-quoted string to safely pass arbitrary content
  updated=$("${PSQL_CMD[@]}" -t -A -c "
WITH updated AS (
  UPDATE documents
  SET body_text = \$BODY\$$body\$BODY\$,
      body_length = $body_len,
      body_digest = '$digest',
      parse_status = 'parsed',
      updated_at = NOW(),
      revision = revision + 1
  WHERE id = '$doc_id'
  RETURNING id
)
SELECT count(*) FROM updated;
")
  updated=$(echo "$updated" | tr -d '[:space:]')
  if [ "$updated" != "1" ]; then
    echo "ERROR: expected documents.id=$doc_id to exist before ingest (updated=$updated)" >&2
    exit 1
  fi
  echo "  OK ($body_len chars): $file_path"
}

echo "--- Ingesting T1 Authority Docs ---"
ingest_doc 'doc-product-truth'    'docs/PRODUCT_TRUTH.md'
ingest_doc 'doc-prd-v05'          'docs/prd-v0.5.md'
ingest_doc 'doc-interaction-v11'  'docs/interaction-spec-v1.1.md'
ingest_doc 'doc-ux-v11'           'docs/ux-spec-v1.1.md'
ingest_doc 'doc-acceptance-v11'   'docs/acceptance-spec-v1.1.md'
ingest_doc 'doc-arch-decisions'   'docs/architecture-decisions.md'
ingest_doc 'doc-arch-design'      'docs/architecture-design.md'
echo ""

echo "--- Ingesting T2 Role Profile ---"
ingest_doc 'doc-role-mira' 'docs/coordination/roles/MIRA.md'
echo ""

echo "--- Ingesting T3 Task Packets ---"
ingest_doc 'doc-task-hardening'   'docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foundation-hardening-v1.md'
ingest_doc 'doc-task-db-baseline' 'docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-real-collaboration-db-baseline-v1.md'
echo ""

echo "--- Ingesting T4 Review ---"
ingest_doc 'doc-review-process-mapping' 'docs/coordination/reviews/2026-04-28-process-mapping-review.md'
echo ""

echo "--- Ingesting T5 Acceptance ---"
ingest_doc 'doc-acceptance-hardening' 'docs/coordination/acceptance/2026-04-29-lyra-nimbus-foundation-hardening-acceptance.md'
echo ""

echo "--- Ingesting T6 Daily Memory ---"
ingest_doc 'doc-memory-2026-04-29' 'docs/coordination/memory/2026-04-29.md'
echo ""

echo "--- Ingesting T7 Governance ---"
ingest_doc 'doc-coordination-rules' 'docs/coordination/COORDINATION_RULES.md'
echo ""

# Extract full sections from ingested documents
echo "--- Updating section search_text from ingested bodies ---"
"${PSQL_CMD[@]}" -c "
UPDATE document_sections ds
SET search_text = ds.heading_text || ' ' || coalesce(ds.body_excerpt, '')
WHERE EXISTS (SELECT 1 FROM documents d WHERE d.id = ds.document_id AND d.body_text IS NOT NULL);
"

echo ""
echo "--- Verification: document body counts ---"
"${PSQL_CMD[@]}" -c "
SELECT
  template,
  COUNT(*) as docs,
  COUNT(body_text) as with_body,
  SUM(body_length) as total_chars
FROM documents
GROUP BY template
ORDER BY template;
"

echo ""
echo "=== Document ingest complete ==="
