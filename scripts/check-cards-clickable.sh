#!/usr/bin/env bash
# Does the operator's Accept still work? Every pending card's branch must contain
# track HEAD. Run this BEFORE any commit to track, and again after.
#
# This exists because the rule "do not push to track while cards are pending" was
# written in prose twice and broken twice. A rule that depends on remembering it is
# not a control. This script is the control: it is run, it prints a number, and it
# exits non-zero when the operator cannot click.
#
# Usage:
#   scripts/check-cards-clickable.sh            # report, exit 1 if any unclickable
#   scripts/check-cards-clickable.sh --quiet     # only print on failure
#
# No `declare -A` (macOS bash 3.2). No assumptions about the DB clock: ages are not
# computed here at all, so a skewed clock cannot change the answer.

set -u

cd "$(dirname "$0")/.." || exit 2

QUIET=0
[ "${1:-}" = "--quiet" ] && QUIET=1

TRACK="track/infra-foundation"

if ! git rev-parse --verify -q "$TRACK" >/dev/null; then
  echo "FAIL: branch $TRACK does not exist in this checkout" >&2
  exit 2
fi

rows=$(podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F'|' -c "
  select a.branch, a.base, a.id
  from acceptances a
  where a.status='pending'
  order by a.branch, a.id;" 2>/dev/null)

if [ -z "$rows" ]; then
  [ "$QUIET" = "1" ] || echo "no pending cards - nothing can be blocked"
  exit 0
fi

total=0
bad=0
declare_bad=""

while IFS='|' read -r branch base card; do
  [ -n "$branch" ] || continue
  total=$((total + 1))

  if ! git rev-parse --verify -q "$branch" >/dev/null; then
    bad=$((bad + 1))
    declare_bad="${declare_bad}${branch}|${card}|branch missing in this checkout\n"
    continue
  fi

  if ! git merge-base --is-ancestor "$base" "$branch" 2>/dev/null; then
    behind=$(git rev-list --count "$branch".."$base" 2>/dev/null)
    bad=$((bad + 1))
    declare_bad="${declare_bad}${branch}|${card}|behind ${behind}\n"
  fi
done <<< "$rows"

if [ "$bad" -gt 0 ]; then
  echo "FAIL: ${bad} of ${total} pending cards are UNCLICKABLE - the operator's Accept is blocked"
  echo
  echo "which branch / which card / why"
  printf "$declare_bad" | sort -u
  echo
  echo "the affected seats must merge track into their packet branch"
  echo "   git merge track/infra-foundation    # on their branch, then push"
  echo
  echo "Do NOT merge their branches into track yourself: that is the operator's Accept."
  exit 1
fi

[ "$QUIET" = "1" ] || echo "OK: all ${total} pending cards are clickable"
exit 0
// probe
