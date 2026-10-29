#!/usr/bin/env bash
# Does each pending acceptance card's how_to_verify actually exercise the work?
#
# Two checks in one, because two different failure modes have now occurred:
#
# 1. SHELL: the verification tells the operator to run a command. A command he
#    runs in a terminal is not a thing he does in the product. On 2026-10-29
#    acc-disp-session-key-ui told him to run check-verdict-refs.sh and
#    check-cards-clickable.sh - two scripts written to catch AEGIS's own ledger
#    and branch-syncing mistakes. Both pass trivially, and both would keep
#    passing if the feature under test were deleted. He would have seen two
#    green results and accepted a card whose content he never looked at.
#
# 2. MY-INSTRUMENT: the verification names a script or query that belongs to the
#    control plane rather than to the product. Same disease, harder to spot,
#    because the command looks substantive.
#
# This script is a SCREEN, not a verdict. It cannot tell whether a step is
# meaningful - only a human can. What it does is force the question: for each
# pending card, is the verification a thing done in the app, and does it name
# the app at all? Print them, and the operator or the issuing seat decides.
#
# Exit 0 always: this is a review aid. A check that failed the build on a
# heuristic would be ignored within a day.

set -u

cd "$(dirname "$0")/.." || exit 2

# The field separator is a unit separator (0x1f), not a pipe. acceptance_plan is
# JSON and JSON is full of pipes - with IFS='|' the row was shredded and $how
# received only the first fragment, so a card whose acceptance_plan is five psql
# commands still printed shell-based=0. The separator must be a character that
# cannot occur in the payload, not one that is merely unlikely.
US=$(printf '\037')
db() {
  podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F"$US" -c "$1" 2>/dev/null
}

if [ -z "$(db "select 1 from pg_tables where tablename='acceptances'")" ]; then
  echo "FAIL: no acceptances table" >&2
  exit 2
fi

# acceptance_plan IS part of the verification. This script originally read only
# how_to_verify, and reported shell-based=0 while four cards' acceptance_plan
# fields told Mr. Zhang to run psql, run apply-db.sh twice, and INSERT rows into
# the acceptances table itself. The shell was one column over. A check that reads
# the column the seat filled in nicely and not the one the product renders is not
# a check on the card - it is a check on the seat's typing habits.
# Join with a space, NOT chr(10). A newline inside the payload splits the row in
# `while read`, and then the JSON lands in $card on the NEXT iteration - the shell
# pattern then never sees it, and the script prints shell-based=0 for a card whose
# acceptance_plan is five psql commands. Newlines are field separators to `read`,
# so a payload containing them is not one row.
rows=$(db "select a.id, a.dispatch_id,
              coalesce(a.how_to_verify,'') || ' ' || coalesce(a.acceptance_plan::text,'')
            from acceptances a where a.status='pending' order by a.id")

if [ -z "$rows" ]; then
  echo "no pending cards"
  exit 0
fi

shell=0
no_app=0
total=0

while IFS="$US" read -r card dispatch how; do
  [ -n "$card" ] || continue
  total=$((total + 1))

  # 1. Does it ask the operator to run a shell command?
  case "$how" in
    *"bash "*|*"psql"*|*"cargo "*|*"pnpm "*|*"npm "*|*"git "*)
      printf 'SHELL   %s\n' "$card"
      printf '        %s\n' "$(printf '%s' "$how" | cut -c1-160)"
      shell=$((shell + 1))
      ;;
  esac

  # 2. Does it mention the product surface at all? A verification that never
  #    names the app, a panel, a page, a list, a badge, or a button is describing
  #    something other than what the operator will see.
  case "$how" in
    *应用*|*SeatLoom*|*面板*|*页面*|*页*|*列表*|*徽章*|*按钮*|*主题*|*记忆*|*席位*|*派单*|*镜像*|*卡片*)
      : ;;
    *)
      printf 'NO-APP %s — the verification never names a surface he can look at\n' "$card"
      printf '        %s\n' "$(printf '%s' "$how" | cut -c1-160)"
      no_app=$((no_app + 1))
      ;;
  esac
done <<< "$rows"

echo
echo "total=$total  shell-based=$shell  never-names-a-surface=$no_app"
echo
echo "Neither count is a verdict. Both are questions:"
echo "  SHELL — can this be re-stated as something he does in the app?"
echo "  NO-APP — what would he look at to see this work?"
echo
echo "The test that matters, and only a human can apply it: if the work under"
echo "test were deleted, would any of these steps change? If not, it is not a"
echo "verification step, however green it looks when run."
