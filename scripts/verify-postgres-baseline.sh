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

echo "--- Step 1: Start PostgreSQL via docker compose ---"
(cd "$INFRA_DIR" && docker compose up -d --wait)
echo "PostgreSQL ready."
echo ""

echo "--- Step 2: Apply schema ---"
docker exec seatloom-postgres psql -U seatloom -d seatloom \
  -f /docker-entrypoint-initdb.d/001_seatloom_core.sql 2>&1 || \
  echo "(schema already applied — continuing)"
echo ""

echo "--- Step 3: Seed real collaboration baseline ---"
docker cp "$INFRA_DIR/seed/001_real_collaboration_baseline.sql" \
  seatloom-postgres:/tmp/seed.sql
docker exec seatloom-postgres psql -U seatloom -d seatloom -f /tmp/seed.sql
echo "Seed complete."
echo ""

echo "--- Step 4: cargo test -p seatloom-core (with DB integration tests) ---"
$CARGO test -p seatloom-core -- --include-ignored 2>&1
echo ""

echo "=== All checks passed ==="
