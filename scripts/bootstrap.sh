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
# Works with either Docker Desktop or Podman. Auto-detected at startup.
# For Podman on macOS, the podman machine is auto-started if stopped.
#
# Usage:
#   ./scripts/bootstrap.sh
#
# Environment:
#   DATABASE_URL     default: postgresql://seatloom:seatloom@localhost:5432/seatloom
#   PG_READY_TIMEOUT default: 60 (seconds to wait for Postgres to become queryable)
#   CONTAINER        override container runtime (docker|podman); auto-detected otherwise
#   SKIP_RECONCILE   set to 1 to skip the reconcile step

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

# --- Detect container runtime ---
if [ -n "${CONTAINER:-}" ]; then
  if ! command -v "$CONTAINER" >/dev/null 2>&1; then
    echo "ERROR: CONTAINER=$CONTAINER but that binary is not on PATH." >&2
    exit 1
  fi
elif command -v docker >/dev/null 2>&1; then
  CONTAINER="docker"
elif command -v podman >/dev/null 2>&1; then
  CONTAINER="podman"
else
  echo "ERROR: neither docker nor podman found on PATH." >&2
  echo "Install Docker Desktop OR Podman (https://podman.io/docs/installation)." >&2
  exit 1
fi

# --- Podman on macOS needs a running VM ---
if [ "$CONTAINER" = "podman" ] && [ "$(uname -s)" = "Darwin" ]; then
  machine_state="$(podman machine inspect podman-machine-default --format '{{.State}}' 2>/dev/null || echo 'missing')"
  case "$machine_state" in
    running) ;;
    stopped)
      echo "Podman machine is stopped. Starting podman-machine-default ..."
      podman machine start podman-machine-default
      ;;
    missing)
      echo "No podman machine found. Initialising default machine ..."
      podman machine init
      podman machine start
      ;;
    *)
      echo "Podman machine state is '$machine_state' — attempting start anyway ..."
      podman machine start podman-machine-default || true
      ;;
  esac
fi

INFRA_DIR="$REPO_ROOT/infra/postgres"
export DATABASE_URL="${DATABASE_URL:-postgresql://seatloom:seatloom@localhost:5432/seatloom}"
PG_READY_TIMEOUT="${PG_READY_TIMEOUT:-60}"
CONTAINER_NAME="seatloom-postgres"
VOLUME_NAME="seatloom-pgdata"
POSTGRES_IMAGE="postgres:16-alpine"

echo "=== SeatLoom v0.1 Bootstrap ==="
echo "runtime:   $CONTAINER"
echo "repo root: $REPO_ROOT"
echo "database:  $DATABASE_URL"
echo ""

# --- Step 1: Ensure Postgres container is running ---
echo "--- Step 1: Postgres container ---"
if "$CONTAINER" ps --format '{{.Names}}' 2>/dev/null | grep -q "^${CONTAINER_NAME}$"; then
  echo "$CONTAINER_NAME already running."
elif "$CONTAINER" ps -a --format '{{.Names}}' 2>/dev/null | grep -q "^${CONTAINER_NAME}$"; then
  echo "Starting existing $CONTAINER_NAME container ..."
  "$CONTAINER" start "$CONTAINER_NAME" >/dev/null
else
  # Ensure the named volume exists for persistence
  if ! "$CONTAINER" volume inspect "$VOLUME_NAME" >/dev/null 2>&1; then
    "$CONTAINER" volume create "$VOLUME_NAME" >/dev/null
  fi
  # Pull image explicitly so progress is visible (otherwise run -d hides it).
  if ! "$CONTAINER" image inspect "$POSTGRES_IMAGE" >/dev/null 2>&1; then
    echo "Pulling $POSTGRES_IMAGE (first run only, ~230MB) ..."
    "$CONTAINER" pull "$POSTGRES_IMAGE"
  fi
  echo "Creating $CONTAINER_NAME ..."
  "$CONTAINER" run -d \
    --name "$CONTAINER_NAME" \
    -e POSTGRES_USER=seatloom \
    -e POSTGRES_PASSWORD=seatloom \
    -e POSTGRES_DB=seatloom \
    -p 5432:5432 \
    -v "${VOLUME_NAME}:/var/lib/postgresql/data" \
    "$POSTGRES_IMAGE"
fi

echo "Waiting for seatloom database to be queryable (timeout: ${PG_READY_TIMEOUT}s)..."
deadline=$(( $(date +%s) + PG_READY_TIMEOUT ))
until "$CONTAINER" exec "$CONTAINER_NAME" psql -U seatloom -d seatloom -c "SELECT 1" -q >/dev/null 2>&1; do
  if [ "$(date +%s)" -ge "$deadline" ]; then
    echo "ERROR: seatloom database not queryable within ${PG_READY_TIMEOUT}s." >&2
    "$CONTAINER" logs "$CONTAINER_NAME" --tail 40 >&2 || true
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
  005_prompt_and_channel_action_authority.sql \
  008_project_isolation.sql
do
  echo "Applying schema/$schema_file"
  "$CONTAINER" cp "$INFRA_DIR/schema/$schema_file" "${CONTAINER_NAME}:/tmp/$schema_file"
  "$CONTAINER" exec "$CONTAINER_NAME" psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
    -f "/tmp/$schema_file" >/dev/null
done
echo "Schema applied."
echo ""

# --- Step 3: Apply seed (idempotent via ON CONFLICT DO NOTHING) ---
echo "--- Step 3: Apply seed baseline ---"
# Seed 002 pre-populates a small fixed set of document rows + sections as a
# fallback for environments without a reconcile engine. Once our reconcile has
# run against docs/, those rows are superseded by the live markdown — and
# re-applying seed 002 would collide with document_sections rows reconcile
# already wrote (unique key on (document_id, ordinal)). So: skip seed 002
# automatically on subsequent runs once reconcile has populated documents.
DOC_COUNT=$("$CONTAINER" exec "$CONTAINER_NAME" psql -U seatloom -d seatloom -t -A -c \
  "SELECT count(*) FROM documents WHERE parse_status = 'parsed'" 2>/dev/null || echo "0")
DOC_COUNT=$(echo "$DOC_COUNT" | tr -d '[:space:]')

for seed_file in \
  001_real_collaboration_baseline.sql \
  002_document_seed.sql \
  003_operational_review_and_continuity_seed.sql \
  004_prompt_and_channel_action_seed.sql
do
  if [ "$seed_file" = "002_document_seed.sql" ] && [ "${DOC_COUNT:-0}" -gt 0 ]; then
    echo "Skipping seed/002_document_seed.sql (reconcile has populated documents; $DOC_COUNT parsed rows)"
    continue
  fi
  echo "Applying seed/$seed_file"
  "$CONTAINER" cp "$INFRA_DIR/seed/$seed_file" "${CONTAINER_NAME}:/tmp/$seed_file"
  "$CONTAINER" exec "$CONTAINER_NAME" psql -v ON_ERROR_STOP=1 -U seatloom -d seatloom \
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
"$CONTAINER" exec "$CONTAINER_NAME" psql -U seatloom -d seatloom -c "
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
echo "  - Inspect data manually:     $CONTAINER exec -it $CONTAINER_NAME psql -U seatloom -d seatloom"
echo "  - Tear down (destructive):   ./scripts/teardown.sh"
