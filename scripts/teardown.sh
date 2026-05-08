#!/usr/bin/env bash
# teardown.sh — SeatLoom v0.1 destructive cleanup.
#
# Stops the seatloom-postgres container AND wipes its volume.
# All document, session, workitem, and event data in the local DB is lost.
# The markdown files under docs/ are NOT touched (they are git-tracked source).
#
# After running this, use ./scripts/bootstrap.sh to rebuild.
#
# Works with either Docker Desktop or Podman (auto-detected).
#
# Usage:
#   ./scripts/teardown.sh          # prompts for confirmation
#   FORCE=1 ./scripts/teardown.sh  # skips prompt

set -euo pipefail

# --- Detect container runtime ---
if [ -n "${CONTAINER:-}" ]; then
  :
elif command -v docker >/dev/null 2>&1; then
  CONTAINER="docker"
elif command -v podman >/dev/null 2>&1; then
  CONTAINER="podman"
else
  echo "ERROR: neither docker nor podman found on PATH." >&2
  exit 1
fi

CONTAINER_NAME="seatloom-postgres"
VOLUME_NAME="seatloom-pgdata"

if [ "${FORCE:-}" != "1" ]; then
  echo "This will STOP $CONTAINER_NAME AND DELETE its Postgres volume ($VOLUME_NAME)."
  echo "Markdown source files under docs/ will NOT be touched."
  read -r -p "Type 'yes' to proceed: " confirm
  if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
  fi
fi

echo "Stopping container (if running) ..."
"$CONTAINER" stop "$CONTAINER_NAME" >/dev/null 2>&1 || true

echo "Removing container (if present) ..."
"$CONTAINER" rm "$CONTAINER_NAME" >/dev/null 2>&1 || true

echo "Removing volume $VOLUME_NAME ..."
"$CONTAINER" volume rm "$VOLUME_NAME" >/dev/null 2>&1 || true

echo "Teardown complete. Run ./scripts/bootstrap.sh to rebuild."
