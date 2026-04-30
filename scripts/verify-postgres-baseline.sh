#!/usr/bin/env bash
# verify-postgres-baseline.sh
# Deterministic bootstrap and verification for the SeatLoom PostgreSQL baseline.
# Starts the repo-managed PostgreSQL service, applies schema, seeds real collaboration data,
# and runs the database integration tests.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Source cargo env if available
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck source=/dev/null
  . "$HOME/.cargo/env"
fi

CARGO="${CARGO:-$(command -v cargo 2>/dev/null || true)}"
if [ -z "$CARGO" ] || [ ! -x "$CARGO" ]; then
  echo "ERROR: cargo not found. Install Rust via https://rustup.rs" >&2
  exit 1
fi

INFRA_DIR="$REPO_ROOT/infra/postgres"
export DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/seatloom"

echo "=== SeatLoom PostgreSQL Baseline Verification ==="
echo "repo root: $REPO_ROOT"
echo "database:  $DATABASE_URL"
echo ""

echo "--- Step 0: Static cross-seed consistency ---"
$CARGO test -p seatloom-core postgres_seed_consistency -- --nocapture
echo ""

echo "--- Step 1: Start PostgreSQL via docker compose ---"
(cd "$INFRA_DIR" && docker compose up -d --wait)
echo "PostgreSQL ready."
echo ""

echo "--- Step 2: Apply schema ---"
for schema_file in \
  001_seatloom_core.sql \
  002_document_authority.sql \
  003_write_ingest_reconcile.sql \
  004_operational_review_and_continuity.sql
do
  echo "Applying schema/$schema_file"
  docker exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
    -f "/docker-entrypoint-initdb.d/$schema_file"
done
echo ""

echo "--- Step 3: Seed real collaboration baseline ---"
for seed_file in \
  001_real_collaboration_baseline.sql \
  002_document_seed.sql \
  003_operational_review_and_continuity_seed.sql
do
  echo "Applying seed/$seed_file"
  docker cp "$INFRA_DIR/seed/$seed_file" "seatloom-postgres:/tmp/$seed_file"
  docker exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
    -f "/tmp/$seed_file"
done
echo "Seed complete."
echo ""

echo "--- Step 4: Ingest full document bodies ---"
bash "$REPO_ROOT/scripts/ingest-documents.sh"
echo ""

echo "--- Step 5: cargo test -p seatloom-core (with DB integration tests) ---"
$CARGO test -p seatloom-core -- --include-ignored 2>&1
echo ""

echo "=== All checks passed ==="
