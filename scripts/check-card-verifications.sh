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
#
# AND the payload's OWN newlines must be stripped too, not just the joiner. A
# space join fixed the connector but acceptance_plan is JSON and JSON pretty-prints
# with newlines inside it, so the row still broke: total went to 10 for two cards
# and $how received only the first fragment, which is why expanding the keyword
# list changed nothing. translate() removes CR and LF from the JSON text itself.
rows=$(db "select a.id, a.dispatch_id,
              translate(coalesce(a.how_to_verify,''), chr(10)||chr(13), ' ')
                || ' ' ||
              translate(coalesce(a.acceptance_plan::text,''), chr(10)||chr(13), ' ')
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
  #
  #    THIS IS A KEYWORD TABLE AND A KEYWORD TABLE IS ALWAYS SHORTER THAN THE WAY
  #    PEOPLE WRITE. It reported never-names-a-surface=9 on 2026-10-29 for two cards
  #    whose verification steps are good: they say 看终端头部, 把鼠标停在它上面,
  #    打开 crates/.../repositories.rs 把常量改名. Those name real things he looks at;
  #    they just did not contain any word in my list. A false negative here reads as
  #    "the seat wrote a bad verification" when the seat wrote a fine one.
  #
  #    So treat this count as a PROMPT TO READ, never as a verdict. When it is
  #    non-zero, open the steps and judge them yourself. The question that matters
  #    is at the bottom of this file and only a human can apply it.
  case "$how" in
    *应用*|*SeatLoom*|*面板*|*页面*|*页*|*列表*|*徽章*|*按钮*|*主题*|*记忆*|*席位*|*派单*|*镜像*|*卡片*|\
    *终端*|*鼠标*|*点击*|*点一*|*打开*|*看到*|*应该出现*|*提示*|*行数*|*文件*|*目录*|*导航*|*菜单*|*左边栏*|*左边*|*高亮*)
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
