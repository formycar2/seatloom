#!/usr/bin/env bash
# bootstrap.sh — SeatLoom v0.1 one-command bring-up.
#
# Idempotent: safe to run repeatedly. Starts Postgres (if not running),
# applies schema migrations (IF NOT EXISTS), applies seed baseline (ON CONFLICT
# DO NOTHING), then runs the CLI reconcile to ingest docs/ markdown into the
# document authority tables.
#
# Non-destructive. For a clean reset use scripts/teardown.sh followed by this.
#
# Usage:
#   ./scripts/bootstrap.sh
#
# Environment:
#   DATABASE_URL    default: postgresql://seatloom:seatloom@localhost:5432/seatloom
#   PG_READY_TIMEOUT default: 60 (seconds to wait for Postgres to become queryable)
#   SKIP_RECONCILE  default: unset (set to 1 to skip the reconcile step)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck source=/dev/null
  . "$HOME/.cargo/env"
fi

CARGO="${CARGO:-$(command -v cargo 2>/dev/null || true)}"
if [ -z "$CARGO" ] || [ ! -x "$CARGO" ]; then
  echo "ERROR: cargo not found. Install Rust via https://rustup.rs" >&2
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: docker not found. Install Docker Desktop." >&2
  exit 1
fi

INFRA_DIR="$REPO_ROOT/infra/postgres"
export DATABASE_URL="${DATABASE_URL:-postgresql://seatloom:seatloom@localhost:5432/seatloom}"
PG_READY_TIMEOUT="${PG_READY_TIMEOUT:-60}"

echo "=== SeatLoom v0.1 Bootstrap ==="
echo "repo root: $REPO_ROOT"
echo "database:  $DATABASE_URL"
echo ""

# --- Step 1: Ensure Postgres container is running ---
echo "--- Step 1: Postgres container ---"
if docker ps --format '{{.Names}}' | grep -q '^seatloom-postgres$'; then
  echo "seatloom-postgres already running."
else
  echo "Starting seatloom-postgres via docker compose..."
  (cd "$INFRA_DIR" && docker compose up -d)
fi

echo "Waiting for seatloom database to be queryable (timeout: ${PG_READY_TIMEOUT}s)..."
deadline=$(( $(date +%s) + PG_READY_TIMEOUT ))
until docker exec seatloom-postgres psql -U seatloom -d seatloom -c "SELECT 1" -q >/dev/null 2>&1; do
  if [ "$(date +%s)" -ge "$deadline" ]; then
    echo "ERROR: seatloom database not queryable within ${PG_READY_TIMEOUT}s." >&2
    docker logs seatloom-postgres --tail 40 >&2
    exit 1
  fi
  sleep 1
done
echo "seatloom database ready."
echo ""

# --- Step 2: Apply schema (idempotent via CREATE TABLE IF NOT EXISTS) ---
echo "--- Step 2: Apply schema ---"
for schema_file in \
  001_seatloom_core.sql \
  002_document_authority.sql \
  003_write_ingest_reconcile.sql \
  004_operational_review_and_continuity.sql \
  005_prompt_and_channel_action_authority.sql
do
  echo "Applying schema/$schema_file"
  docker cp "$INFRA_DIR/schema/$schema_file" "seatloom-postgres:/tmp/$schema_file"
  docker exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
    -f "/tmp/$schema_file" >/dev/null
done
echo "Schema applied."
echo ""

# --- Step 3: Apply seed (idempotent via ON CONFLICT DO NOTHING) ---
echo "--- Step 3: Apply seed baseline ---"
for seed_file in \
  001_real_collaboration_baseline.sql \
  002_document_seed.sql \
  003_operational_review_and_continuity_seed.sql \
  004_prompt_and_channel_action_seed.sql
do
  echo "Applying seed/$seed_file"
  docker cp "$INFRA_DIR/seed/$seed_file" "seatloom-postgres:/tmp/$seed_file"
  docker exec seatloom-postgres psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
    -f "/tmp/$seed_file" >/dev/null
done
echo "Seed applied."
echo ""

# --- Step 4: Reconcile docs/ markdown into document authority tables ---
if [ "${SKIP_RECONCILE:-}" = "1" ]; then
  echo "--- Step 4: Reconcile SKIPPED (SKIP_RECONCILE=1) ---"
else
  echo "--- Step 4: Reconcile docs/ markdown ---"
  "$CARGO" run -q -p seatloom-cli -- reconcile --project seatloom --root "$REPO_ROOT"
fi
echo ""

# --- Step 5: Summary ---
echo "--- Summary: document authority state ---"
docker exec seatloom-postgres psql -U seatloom -d seatloom -c "
SELECT
  template,
  COUNT(*) AS docs,
  COUNT(*) FILTER (WHERE parse_status = 'parsed') AS parsed,
  COUNT(*) FILTER (WHERE parse_status = 'partial') AS partial
FROM documents
WHERE project_id = 'seatloom'
GROUP BY template
ORDER BY template;
"

echo ""
echo "=== Bootstrap complete ==="
echo ""
echo "Next:"
echo "  - Launch Tauri desktop app:  cd ui && pnpm install && pnpm tauri dev"
echo "  - Inspect data manually:     docker exec -it seatloom-postgres psql -U seatloom -d seatloom"
echo "  - Tear down (destructive):   ./scripts/teardown.sh"
