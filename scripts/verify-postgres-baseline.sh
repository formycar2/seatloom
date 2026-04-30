#!/usr/bin/env bash
# verify-postgres-baseline.sh
#
# DESTRUCTIVE LOCAL BASELINE VERIFIER.
# This script tears down the repo-managed PostgreSQL verification volume on
# every run to guarantee a clean, deterministic schema + seed state.
# It is NOT the steady-state production write path.
# It is safe — and expected — to run this repeatedly on the same seat.
#
# Schema authority: this script is the sole authority for schema application.
# docker-compose.yml does NOT mount schema files into /docker-entrypoint-initdb.d,
# so there is no hidden auto-apply path during container init.
#
# Proof sequence:
#   1. static seed consistency (pure Rust, no DB)
#   2. volume teardown + clean container start
#   3. schema apply (001–004, copied to container by this script)
#   4. seed apply (001–003)
#   5. bounded document reconcile
#   6. body-ingest helper health check
#   7. DB integration tests (--include-ignored)
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

# Max seconds to wait for PostgreSQL to accept connections after container start.
# Override with: PG_READY_TIMEOUT=120 bash scripts/verify-postgres-baseline.sh
PG_READY_TIMEOUT="${PG_READY_TIMEOUT:-60}"

echo "=== SeatLoom PostgreSQL Baseline Verification ==="
echo "repo root: $REPO_ROOT"
echo "database:  $DATABASE_URL"
echo ""

echo "--- Step 0: Static cross-seed consistency ---"
$CARGO test -p seatloom-core postgres_seed_consistency -- --nocapture
echo ""

echo "--- Step 1: Teardown stale volume + start clean container ---"
echo "(destructive reset — ensures deterministic schema + seed state)"
(cd "$INFRA_DIR" && docker compose down -v --remove-orphans 2>&1 || true)
(cd "$INFRA_DIR" && docker compose up -d)

echo "Waiting for PostgreSQL to become ready (timeout: ${PG_READY_TIMEOUT}s) ..."
deadline=$(( $(date +%s) + PG_READY_TIMEOUT ))
until docker exec seatloom-postgres pg_isready -U seatloom -d seatloom -q 2>/dev/null; do
  if [ "$(date +%s)" -ge "$deadline" ]; then
    echo "ERROR: PostgreSQL did not become ready within ${PG_READY_TIMEOUT}s." >&2
    docker logs seatloom-postgres --tail 40 >&2
    exit 1
  fi
  sleep 1
done
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
  docker cp "$INFRA_DIR/schema/$schema_file" "seatloom-postgres:/tmp/$schema_file"
  docker exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
    -f "/tmp/$schema_file"
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

echo "--- Step 4: Run bounded document reconcile ---"
"$CARGO" run -p seatloom-cli -- reconcile --project seatloom --root "$REPO_ROOT"
echo ""

echo "--- Step 5: Verify direct body-ingest helper remains healthy ---"
bash "$REPO_ROOT/scripts/ingest-documents.sh"
echo ""

echo "--- Step 6: cargo test -p seatloom-core (with DB integration tests) ---"
$CARGO test -p seatloom-core -- --include-ignored 2>&1
echo ""

echo "=== All checks passed ==="
