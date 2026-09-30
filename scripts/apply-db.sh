#!/usr/bin/env bash
# apply-db.sh — apply SeatLoom's schema + seed to a database in the canonical
# order, using $DATABASE_URL (CI-compatible; no docker/podman exec).
#
# Canonical order (verified 2026-09-29, AEGIS-2026-09-29 build-standards packet):
#   1. base schema  001–005  — create the tables the seed inserts into
#   2. seed         001–004  — real-collaboration baseline data
#   3. post-seed migrations ≥006 (currently 008 project-isolation; 009 runtime
#      triple when merged) — these ADD COLUMN, backfill from seeded rows, then
#      SET NOT NULL, so they MUST run AFTER seed. Applying all schema before
#      seed fails: 008 sets sessions.project_id NOT NULL but seed 001 has no
#      project_id.
#
# This ordering is a stopgap: SeatLoom has no migration runner, and the split
# "001–005 pre-seed vs ≥006 post-seed" is convention, not enforced structure.
# A proper ordered-migration + separate-idempotent-seed system is tracked as an
# Onyx follow-up (see docs/ENGINEERING_STANDARDS.md §Migrations).
#
# Usage: DATABASE_URL=postgresql://user:pass@host:port/db scripts/apply-db.sh
# Needs: psql on PATH. Exit nonzero on first SQL error.

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

: "${DATABASE_URL:=postgresql://seatloom:seatloom@localhost:5432/seatloom}"
SCHEMA_DIR="infra/postgres/schema"
SEED_DIR="infra/postgres/seed"

psql_f() { psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -f "$1"; }

echo "apply-db: target=$DATABASE_URL"

echo "--- base schema (001–005) ---"
for n in 001 002 003 004 005; do
  f=$(ls "$SCHEMA_DIR/${n}_"*.sql 2>/dev/null | head -1) || true
  [ -n "${f:-}" ] && [ -f "$f" ] && { echo "  $f"; psql_f "$f"; }
done

echo "--- seed (001–004) ---"
for f in "$SEED_DIR"/*.sql; do
  [ -f "$f" ] && { echo "  $f"; psql_f "$f"; }
done

echo "--- post-seed migrations (>=006) ---"
for f in "$SCHEMA_DIR"/*.sql; do
  n=$(basename "$f" | grep -oE '^[0-9]+' || echo 0)
  if [ "$((10#$n))" -ge 6 ]; then echo "  $f"; psql_f "$f"; fi
done

echo "apply-db: done."
