#!/usr/bin/env bash
# Read-only candidate report, not an automatic acceptance obligation.
# payload.acceptance carries verification instructions, NOT a task category.
# Unknown/missing kinds remain visible; only explicit kind=verification is excluded.
# Exit 0: no candidates; 1: candidates require review; 2: source unavailable.
set -uo pipefail
QUIET=0
case "${1:-}" in
  '') ;;
  --quiet) QUIET=1 ;;
  *) echo "Usage: $0 [--quiet]" >&2; exit 2 ;;
esac
if ! rows=$(podman exec -i seatloom-postgres psql -X -v ON_ERROR_STOP=1 -U seatloom -d seatloom -t -A -F'|' -c "
BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY;
SELECT 'excluded', count(*)::text FROM dispatches d
 WHERE d.status='done' AND d.completed_at>now()-interval '48 hours'
 AND d.payload->>'kind'='verification'
 AND NOT EXISTS(SELECT 1 FROM acceptances a WHERE a.dispatch_id=d.id);
SELECT 'candidate',d.id,d.target_seat_id,coalesce(d.payload->>'kind','unknown'),d.completed_at::text
 FROM dispatches d WHERE d.status='done'
 AND d.completed_at>now()-interval '48 hours'
 AND coalesce(d.payload->>'kind','')<>'verification'
 AND NOT EXISTS(SELECT 1 FROM acceptances a WHERE a.dispatch_id=d.id)
 ORDER BY d.completed_at DESC,d.id;
COMMIT;"); then
  echo 'UNCHECKABLE: ledger read failed; this is not an empty work list.' >&2
  exit 2
fi
excluded=$(printf '%s\n' "$rows" | awk -F'|' '$1=="excluded"{print $2}')
case "$excluded" in
  ''|*[!0-9]*) echo 'UNCHECKABLE: missing/invalid exclusion count.' >&2; exit 2 ;;
esac
candidates=$(printf '%s\n' "$rows" | awk -F'|' '$1=="candidate"')
count=$(printf '%s\n' "$candidates" | awk 'NF{n++} END{print n+0}')
if [ "$QUIET" -eq 0 ] || [ "$count" -gt 0 ]; then
  printf 'Excluded explicit verification requests without cards: %s\n' "$excluded"
  printf 'Done-without-card candidates requiring classification/review: %s\n' "$count"
fi
if [ "$count" -gt 0 ]; then
  printf '%s\n' "$candidates"
  echo 'REVIEW REQUIRED: unknown kinds are not hidden. A card-step may relate to another delivery; inspect lineage before opening a card.'
  exit 1
fi
[ "$QUIET" -eq 1 ] || echo 'OK: no done-without-card candidates in the 48-hour window.'
exit 0
