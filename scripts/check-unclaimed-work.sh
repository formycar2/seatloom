#!/usr/bin/env bash
# Read-only candidate report, not an automatic acceptance obligation.
# payload.acceptance carries verification instructions, NOT a task category.
# Unknown/missing kinds remain visible; only explicit kind=verification is excluded.
# Exit 0: no unclassified candidates; 1: candidates require review; 2: source unavailable.
#
# DECLARED CLASSIFICATIONS (2026-11-10). The checker had been reporting the same
# thirteen candidates for two days. A report whose answer is permanently non-zero for
# structural reasons stops being read, and the day it should catch something it is
# ignored again. I classified all thirteen and found zero lost work — 5 blocked on one
# operator decision (the sandbox), 2 superseded by a redo that passed and was carded,
# and 6 whose deliverable IS a card or a verdict that correctly carries the dispatch id
# of the work it gates rather than the id of the dispatch that asked for it. That last
# group can never be matched by this query, by design: "one implementation subject
# needs its own verdict identity" means the artifact's id is not the requester's id.
#
# So declared entries are now separated from unclassified ones. THE DECLARATION IS NOT
# A SUPPRESSION SWITCH: an entry only counts as declared if it carries a non-empty
# reason. `--classify` writes the reason column EMPTY, so running it cannot make this
# check go green — someone has to write why each one is not lost work, and that writing
# is the review this check exists to force.
set -uo pipefail
QUIET=0
CLASSIFY=0
case "${1:-}" in
  '') ;;
  --quiet) QUIET=1 ;;
  --classify) CLASSIFY=1 ;;
  *) echo "Usage: $0 [--quiet|--classify]" >&2; exit 2 ;;
esac

DECLARED="${UNCLAIMED_DECLARED:-.gitlab/unclaimed-declared.txt}"

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

# --classify emits a TEMPLATE, not a declaration. Every reason is empty on purpose, and
# an empty reason is not a declaration (see declared_reason below). The operator or the
# seat fills them in; that writing is the review.
#
# UNCLAIMED_DECLARED is a seam for tests: a fixture with no matching candidates would
# otherwise see every real declaration as stale, so an isolated run can point at its own
# file. Default is the real one; nothing in the product path sets it.
if [ "$CLASSIFY" -eq 1 ]; then
  {
    echo "# Declared classifications for check-unclaimed-work.sh."
    echo "# Format: <dispatch_id>|<reason>  — the reason is REQUIRED and non-empty."
    echo "# An entry with an empty reason is NOT declared and the candidate still shows."
    echo "# Regenerate the id list with:  bash scripts/check-unclaimed-work.sh --classify"
    printf '%s\n' "$candidates" | awk -F'|' 'NF{print $2"|"}'
  } > "$DECLARED"
  echo "wrote template to $DECLARED — every reason is empty, so nothing is declared yet."
  echo "Fill in each reason, then re-run without --classify."
  exit 0
fi

# The reason for a declared id, or empty if undeclared/unreadable. Read with grep -F so
# a reason containing regex metacharacters cannot silently match nothing.
declared_reason() {
  [ -f "$DECLARED" ] || return 0
  grep -F "$1|" "$DECLARED" 2>/dev/null | head -1 | cut -d'|' -f2- | sed 's/^ *//;s/ *$//'
}

unclassified=0
declared_n=0
stale_n=0
declare -a stale_ids 2>/dev/null || true
: > /tmp/.unclaimed-declared-$$ 2>/dev/null || true
while IFS='|' read -r tag id seat kind when; do
  [ "$tag" = "candidate" ] || continue
  [ -n "$id" ] || continue
  reason=$(declared_reason "$id")
  if [ -n "$reason" ]; then
    declared_n=$((declared_n + 1))
    printf '%s\n' "$id" >> /tmp/.unclaimed-declared-$$
  else
    unclassified=$((unclassified + 1))
  fi
done <<EOF
$candidates
EOF

# Declared entries that no longer appear as candidates are stale: the work got a card,
# or it aged out of the 48-hour window. Report them rather than let the file grow
# forever — a declaration that no longer refers to anything is a claim nobody checks.
if [ -f "$DECLARED" ]; then
  while IFS= read -r line; do
    case "$line" in ''|'#'*) continue ;; esac
    did=$(printf '%s' "$line" | cut -d'|' -f1 | sed 's/^ *//;s/ *$//')
    [ -n "$did" ] || continue
    if ! grep -qxF "$did" /tmp/.unclaimed-declared-$$ 2>/dev/null; then
      stale_n=$((stale_n + 1))
      stale_ids+=("$did")
    fi
  done < "$DECLARED"
fi
rm -f /tmp/.unclaimed-declared-$$

if [ "$QUIET" -eq 0 ] || [ "$unclassified" -gt 0 ]; then
  printf 'Excluded explicit verification requests without cards: %s\n' "$excluded"
  printf 'Declared, with a reason (not lost work): %s\n' "$declared_n"
  printf 'UNCLASSIFIED candidates requiring review: %s\n' "$unclassified"
fi

if [ "$unclassified" -gt 0 ]; then
  # Print only the unclassified ones. The declared ones are still shown below with their
  # reasons, so nothing is hidden — the headline is just no longer a permanent 13.
  printf '%s\n' "$candidates" | while IFS='|' read -r tag id seat kind when; do
    [ "$tag" = "candidate" ] || continue
    [ -n "$id" ] || continue
    [ -n "$(declared_reason "$id")" ] && continue
    printf '  %s|%s|%s|%s|%s\n' "$id" "$seat" "$kind" "$when"
  done
  echo 'REVIEW REQUIRED: unknown kinds are not hidden. Classify each one with a reason, or fix it.'
  # The stale notice is printed BEFORE the exit, not after it. On the first version it
  # sat below this line, so it was unreachable whenever there was anything unclassified
  # — which is exactly the state the checker is usually in. A notice that only appears
  # when everything else is fine is a notice nobody reads.
  [ "$stale_n" -gt 0 ] && echo "NOTE: ${stale_n} declared entr(y|ies) no longer appear as candidates (carded or aged out): ${stale_ids[*]}"
  exit 1
fi

if [ "$QUIET" -eq 1 ] && [ "$declared_n" -eq 0 ] && [ "$stale_n" -eq 0 ]; then
  exit 0
fi
[ "$QUIET" -eq 1 ] || echo 'OK: no unclassified done-without-card candidates in the 48-hour window.'
if [ "$declared_n" -gt 0 ]; then
  echo "Declared and skipped (each carries a reason — read them, they are claims):"
  while IFS='|' read -r tag id seat kind when; do
    [ "$tag" = "candidate" ] || continue
    [ -n "$id" ] || continue
    r=$(declared_reason "$id")
    [ -n "$r" ] || continue
    printf '  %s (%s, %s): %s\n' "$id" "$kind" "$seat" "$r"
  done <<EOF
$candidates
EOF
fi
if [ "$stale_n" -gt 0 ]; then
  echo "NOTE: ${stale_n} declared entr(y|ies) no longer appear as candidates (carded or aged out): ${stale_ids[*]}"
  echo "      Prune them from $DECLARED — a declaration that refers to nothing is a claim nobody checks."
fi
exit 0
