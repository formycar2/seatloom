#!/usr/bin/env bash
# Does any pending acceptance card rest on a verdict that NAMES A DEFECT?
#
# WHY THIS EXISTS. On 2026-10-29 an acceptance card merged a fix that does not
# work, with its own two tests red, because the verdict said so in capital letters:
#
#   "BUT THE FIX DOES NOT WORK, and her own two red tests say so"
#
# and I had built four checks, none of which reads a verdict's payload:
#
#   check-cards-clickable.sh    asks whether the branch is synced
#   check-verdict-refs.sh       asks whether the verdict names the right dispatch
#   check-card-verifications.sh asks whether the verification step is worthless
#   fleet-status.sh             asks about the queue and the seats
#
# check-verdict-refs.sh is the near miss: it reads payload.dispatch_id and nothing
# else. So a verdict that says "clean" and a verdict that says "THE FIX DOES NOT
# WORK" were indistinguishable to every control I had. A pass that names its own
# defect is not a pass, and it must not reach the operator's Accept silently.
#
# WHAT COUNTS AS A NAMED DEFECT. Not a vibe. Any of:
#   - verdict is not "pass"
#   - the payload contains a finding / defect / caveat field with content
#   - the honest summary matches a phrase that says the thing does not work
#
# Usage:
#   scripts/check-verdict-defects.sh            # report, exit 1 if any named defect
#   scripts/check-verdict-defects.sh --quiet    # only print on failure

set -u

cd "$(dirname "$0")/.." || exit 2

QUIET=0
[ "${1:-}" = "--quiet" ] && QUIET=1

db() {
  podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F"$(printf '\037')" -c "$1" 2>/dev/null
}

# Which pending card rests on which verdict, and what that verdict actually says.
# The payload is read, not just the dispatch id.
rows=$(db "select a.id, a.flux_verdict_ref, e.payload->>'verdict',
                  coalesce(e.payload->>'the_honest_summary',''),
                  coalesce(e.payload->>'what_i_did_not_verify',''),
                  coalesce(e.payload->>'finding_1',''),
                  coalesce(e.payload->>'finding_2',''),
                  coalesce(e.payload->>'finding_3','')
           from acceptances a
           join canonical_events e on e.id = a.flux_verdict_ref
           where a.status = 'pending'
           order by a.id")

if [ -z "$rows" ]; then
  [ "$QUIET" = "1" ] || echo "no pending acceptance cards"
  exit 0
fi

bad=0
declare_bad=""

while IFS="$(printf '\037')" read -r card verdict verdict_word summary not_verified f1 f2 f3; do
  [ -n "$card" ] || continue
  reasons=""

  # 1. A verdict that is not a pass is not a pass. State it plainly.
  if [ "$verdict_word" != "pass" ]; then
    reasons="${reasons}verdict is '${verdict_word:-<empty>}', not pass|"
  fi

  # 2. The payload names findings. A finding is content in a finding_* field.
  for f in "$f1" "$f2" "$f3"; do
    if [ -n "$f" ]; then
      reasons="${reasons}names a finding: $(printf '%s' "$f" | cut -c1-120)|"
      break
    fi
  done

  # 3. The honest summary says the thing does not work. These phrases are the
  #    tell. They are written in lower case and matched case-insensitively so a
  #    shouted "THE FIX DOES NOT WORK" is caught as readily as a quiet one.
  low=$(printf '%s %s' "$summary" "$not_verified" | tr '[:upper:]' '[:lower:]')
  case "$low" in
    *"does not work"*|*"not working"*|*"fix is not complete"*|*"is unpinned"*|\
    *"not covered"*|*"uncovered"*|*"red"*"test"*|*"test"*"red"*|*"failing"*|*"defect"*)
      hit=$(printf '%s' "$low" | grep -o -m1 '.\{0,60\}\(does not work\|not working\|is unpinned\|not covered\|uncovered\|failing\|defect\).\{0,60\}')
      reasons="${reasons}the honest summary says so: ...${hit}...|"
      ;;
  esac

  # 4. A verdict with an empty summary is not a verdict I can rely on. Not a
  #    defect in the work, but it is one in the record: I cannot tell a clean
  #    pass from an unexamined one.
  if [ -z "$summary" ] && [ -z "$f1" ] && [ -z "$f2" ] && [ -z "$f3" ]; then
    reasons="${reasons}the verdict carries no summary and no findings - nothing to distinguish it from 'I did not look'|"
  fi

  if [ -n "$reasons" ]; then
    bad=$((bad + 1))
    declare_bad="${declare_bad}${card}|${verdict}|${reasons}"$'\n'
  fi
done <<< "$rows"

if [ "$bad" -gt 0 ]; then
  echo "FAIL: ${bad} pending card(s) rest on a verdict that names a defect."
  echo
  echo "A pass that names its own defect is not a pass. Mr. Zhang's Accept is the"
  echo "only thing that merges a card, and he would be merging on a verdict that"
  echo "already told us the work is broken."
  echo
  printf '%s' "$declare_bad" | while IFS='|' read -r card verdict rest; do
    [ -n "$card" ] || continue
    echo "  card:     $card"
    echo "  verdict:  $verdict"
    printf '%s\n' "$rest" | while IFS= read -r line; do
      [ -n "$line" ] && echo "            - $line"
    done
    echo
  done
  echo "The rework belongs to the seat that did the work. A defect found by the"
  echo "verifier is not a rejection of the work - it is the verifier doing its job."
  exit 1
fi

[ "$QUIET" = "1" ] || echo "OK: every pending card rests on a verdict that does not name a defect."
exit 0
