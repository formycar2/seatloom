#!/usr/bin/env bash
# Is the ledger still there?
#
# WHY THIS EXISTS. On 2026-10-31 09:00 the live `seatloom` database was found
# with ZERO tables in `public`. Every table - canonical_events, memories,
# dispatches, acceptances, topics, seats, projects - gone. Last successful write
# was 00:11:52; first "relation does not exist" was 00:34:10.
#
# SEVEN checks ran every hour and none of them noticed, because every one of
# them asked a question whose answer is a NUMBER:
#
#   how many pending cards?            -> 0   (reads as "none waiting")
#   how many memories?                 -> 0   (reads as "none recorded")
#   is the migration registered?       -> yes (reads from git, not the DB)
#   is the branch synced?              -> yes (reads from git)
#   does the verdict name the dispatch?-> yes (reads from a table that is gone)
#
# An ABSENT table and an EMPTY table produce the same number. 0 reads as
# "nothing to do", not as "everything is gone". This is the final form of the
# lesson from 2026-10-30 - I guarded against "the file is absent" and "the
# query returned empty", and never against "the database is absent".
#
# So this check asks the one question that cannot be answered by a count:
# does the ledger EXIST? It counts the tables, and it proves the count is
# meaningful by also counting something that must be non-zero.
#
# Usage:
#   scripts/check-ledger-exists.sh          # report, exit 1 if the ledger is gone
#   scripts/check-ledger-exists.sh --quiet  # only print on failure

set -u

DB_CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
DB_NAME="${SEATLOOM_DB_NAME:-seatloom}"

# A healthy seatloom database carries 32+ tables. The exact number grows as
# migrations land, so this is a FLOOR, not an equality - a check stricter than
# the thing it checks is wrong, not conservative. 30 leaves room for two tables
# to be added or removed without a false alarm.
FLOOR=30

QUIET=0
[ "${1:-}" = "--quiet" ] && QUIET=1

dbq() {
  podman exec -e PGPASSWORD=seatloom "$DB_CONTAINER" \
    psql -U seatloom -d "$DB_NAME" -t -A -c "$1" 2>&1
}

# 1. Is the container even up? A stopped container is a different failure from
#    an empty database, and conflating them sends the operator down the wrong
#    path. Say which one it is.
if ! podman ps --format '{{.Names}}' 2>/dev/null | grep -qx "$DB_CONTAINER"; then
  echo "FAIL: container $DB_CONTAINER is not running."
  echo "      That is not an empty ledger - it is an unreachable one, and the fix"
  echo "      is different (start it) from the fix for a dropped schema (rebuild)."
  exit 2
fi

# 2. Count the tables. stderr is NOT discarded: a psql error and a zero both
#    arrive on stdout as nothing, and that is the exact ambiguity this script
#    exists to remove.
tables=$(dbq "select count(*) from information_schema.tables where table_schema='public';")

if ! printf '%s' "$tables" | grep -qE '^[0-9]+$'; then
  echo "FAIL: could not count tables in $DB_NAME at all."
  echo "      The query returned: $tables"
  echo "      An unreachable database is not an empty one. This check must not"
  echo "      report OK when it has not looked."
  exit 2
fi

# 3. Prove the count is meaningful. A query that returns 0 for a missing table
#    is indistinguishable from one that returns 0 for an empty table - so ask a
#    second question whose answer CANNOT be 0 in a healthy database. The number
#    of schemas in any PostgreSQL database is at least 4 (public,
#    information_schema, pg_catalog, pg_toast).
schemas=$(dbq "select count(*) from information_schema.schemata;")
if ! printf '%s' "$schemas" | grep -qE '^[0-9]+$' || [ "$schemas" -lt 4 ]; then
  echo "FAIL: the schema count came back as '$schemas', which is not a number >= 4."
  echo "      That means the database itself is not answering like a PostgreSQL"
  echo "      database, so the table count above cannot be trusted either."
  exit 2
fi

if [ "$tables" -lt "$FLOOR" ]; then
  # --quiet prints nothing: fleet-status.sh prints its own, more prominent
  # message when this returns non-zero, and two messages about one failure
  # read as two failures.
  if [ "$QUIET" != "1" ]; then
  echo "FAIL: $DB_NAME has $tables table(s) in public, fewer than the floor of $FLOOR."
  echo ""
  echo "      THE LEDGER IS GONE OR WAS NEVER BUILT. These are different facts and"
  echo "      they need different words - do not say 'not built yet' if it was built"
  echo "      and then dropped. Check the postgres log for the boundary:"
  echo ""
  echo "        podman logs $DB_CONTAINER 2>&1 | grep -n 'does not exist' | head -1"
  echo ""
  echo "      and the last successful write before it:"
  echo ""
  echo "        podman logs $DB_CONTAINER 2>&1 | grep -E 'INSERT INTO (canonical_events|memories|dispatches)' | tail -1"
  echo ""
  echo "      Everything downstream of this - cards, dispatches, memories, gates -"
  echo "      is reading from tables that do not exist, and every count it prints"
  echo "      is a 0 that reads as 'nothing to do'."
  fi
  exit 1
fi

# 4. One more, because a table count alone could be satisfied by a database that
#    has the tables but lost the rows. A ledger with tables and no events is a
#    different failure, and it deserves its own message rather than a pass.
events=$(dbq "select count(*) from canonical_events;" 2>/dev/null)
if printf '%s' "$events" | grep -qE '^[0-9]+$' && [ "$events" -eq 0 ]; then
  if [ "$QUIET" != "1" ]; then
  echo "WARN: $tables tables exist but canonical_events has 0 rows."
  echo "      The schema is there and the history is not. That is a smaller loss"
  echo "      than a dropped schema, but it is not OK either - say so rather than"
  echo "      letting a 0 read as 'nothing has happened'."
  fi
  exit 1
fi

[ "$QUIET" = "1" ] || echo "OK: $DB_NAME has $tables table(s) and $events event(s) - the ledger is there."
exit 0
