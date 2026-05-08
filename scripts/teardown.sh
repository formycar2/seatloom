#!/usr/bin/env bash
# teardown.sh — SeatLoom v0.1 destructive cleanup.
#
# Stops the seatloom-postgres container AND wipes its volume.
# All document, session, workitem, and event data in the local DB is lost.
# The markdown files under docs/ are NOT touched (they are git-tracked source).
#
# After running this, use ./scripts/bootstrap.sh to rebuild.
#
# Usage:
#   ./scripts/teardown.sh          # prompts for confirmation
#   FORCE=1 ./scripts/teardown.sh  # skips prompt

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INFRA_DIR="$REPO_ROOT/infra/postgres"

if [ "${FORCE:-}" != "1" ]; then
  echo "This will STOP seatloom-postgres AND DELETE its Postgres volume."
  echo "Markdown source files under docs/ will NOT be touched."
  read -r -p "Type 'yes' to proceed: " confirm
  if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
  fi
fi

echo "Stopping container and removing volume..."
(cd "$INFRA_DIR" && docker compose down -v --remove-orphans)
echo "Teardown complete. Run ./scripts/bootstrap.sh to rebuild."
