#!/usr/bin/env bash
# Does an acceptance card's plan actually RUN? Every git ref it names must exist.
#
# WHY THIS EXISTS. On 2026-10-28 three pending cards told the operator to run
# `git rev-parse track/infa-foundation` — a branch name missing an `r`. The real
# branch is `track/infra-foundation`. Measured: that command fails with
# `fatal: Needed a single revision`, so the operator died on step 1 of a card
# whose claim was TRUE, whose verdict was a valid pass, and whose Accept button
# worked. Only the instructions were unrunnable.
#
# The acceptance gate validates the verdict ref, the dispatch ownership and the
# plan SHAPE (plan_version, non-empty steps/expected). It has never validated
# that the refs the plan NAMES exist. That is the gap this script closes.
#
# This is the same family as two already-recorded defects: the `-landed` vs
# `-live` gate-typo (an event id whose type was wrong for its suffix), and the
# wild-pointer verdict (a dispatch_id with letter-o instead of digit-zero). A
# token that names the wrong thing is believed because it looks right.
#
# What it checks, per card:
#   * every `track/...`, `packet/...`, and `scripts/....sh` token named in
#     how_to_verify or acceptance_plan exists as a local branch, or on a remote
#   * a warning (not a failure) for a ref that exists only remotely
#
# Deliberately NOT matched: bare `main` / `master` — they are ordinary English
# words that appear in prose, and matching them makes the checker cry wolf.
# The defect this was built for is a NAMESPACED ref (`track/`, `packet/`).
#
# Deliberately NOT matched: `<track HEAD>`-style placeholders — those are prose
# describing a step, not a token the operator types.
#
# Exit 0 = every named ref resolves. Exit 1 = at least one card cannot be run
# as written, and the card is named with the exact offending string.
#
# Usage: scripts/check-cards-runnable.sh [--quiet] [--include-accepted]
#
# --include-accepted REPORTS on decided cards and never re-decides them. It exists
# because the default (pending-only) view certifies a history it has not read:
# three accepted cards were found on 2026-10-28 whose evidence was fine but whose
# instructions were unrunnable. Reporting is the point; a checker that only looks
# forward is a checker whose "OK" is a statement about the future.
set -u

cd "$(dirname "$0")/.." || exit 2

QUIET=0
INCLUDE_ACCEPTED=0
for a in "$@"; do
  case "$a" in
    --quiet) QUIET=1;;
    --include-accepted) INCLUDE_ACCEPTED=1;;
    *) printf 'usage: %s [--quiet] [--include-accepted]\n' "$0" >&2; exit 2;;
  esac
done

STATUS_FILTER="'pending'"
[ "$INCLUDE_ACCEPTED" = "1" ] && STATUS_FILTER="'pending', 'accepted'"

db() {
  podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -c "$1" 2>/dev/null
}

if [ -z "$(db "select 1 from pg_tables where tablename='acceptances'")" ]; then
  echo "FAIL: the acceptances table is not reachable — cannot check card runnability" >&2
  exit 2
fi

# resolve <ref> — 0 if it names something git can use.
#
# Two bugs were found by mutation-testing this function, and both made it accept
# things it should reject, which is the failure direction that matters:
#
# 1. It checked `[ -f "$t" ]` for scripts in the CHECKER'S OWN checkout. The
#    operator follows a card in the primary checkout on `track/infra-foundation`,
#    so a script that exists only on track reported a false FAIL
#    (acc-disp-session-key-ui names scripts/check-verdict-refs.sh). Now a script
#    is checked with `git cat-file -e $REF_BASE:$path`.
# 2. An earlier fix added `git ls-tree -r --name-only "$REF_BASE" >/dev/null &&
#    return 0`, which returns 0 whenever the BASE EXISTS and says nothing about
#    $1 at all — so `packet/x/y` and `track/does-not-exist` both resolved.
#    Caught by the mutation test below, not by running the checker.
#
# The correct question is simply: can git resolve this string as a ref, locally
# or on either remote? A branch that is not on the base is still a real branch.
REF_BASE=${REF_BASE:-track/infra-foundation}
resolve_ref() {
  git rev-parse --verify -q "$1" >/dev/null 2>&1 && return 0
  git rev-parse --verify -q "origin/$1" >/dev/null 2>&1 && return 0
  git rev-parse --verify -q "gitlab/$1" >/dev/null 2>&1 && return 0
  return 1
}

# Does a path exist ON THE REFERENCE BASE, not in this checkout?
path_on_base() {
  git cat-file -e "$REF_BASE:$1" 2>/dev/null
}

bad=0
total=0
remote_only=0

# Fetch each card's body by id rather than splitting a row on a separator.
# An earlier version read `IFS='~' read -r cid branch body` while `db()` formats
# with `-F'|'`, so $body was always empty and every check silently passed. A
# separator mismatch is a check that cannot fail. Ids are `acc-<nanoid>`, which
# contains no quote, so inlining one is safe here.
while IFS= read -r cid; do
  [ -n "$cid" ] || continue
  total=$((total + 1))

  body=$(db "select replace(replace(coalesce(how_to_verify,'') || ' | ' || coalesce(acceptance_plan::text,''), chr(10), ' '), chr(13), ' ') from acceptances where id='$cid' and status in ($STATUS_FILTER);")

  # Split on the range separator FIRST. A body like
  # `a..track/infa-foundation` is one grep match when `.` is in the character
  # class, and the second ref would never be checked. This was found by mutation:
  # reintroducing the typo inside a range went undetected.
  #
  # THEN drop SQL example text. A card that demonstrates the acceptance-plan
  # constraint shows its reader an INSERT with deliberately fake values
  # (`'packet/x/y'`, `'demo'`, `'<done dispatch id>'`). Extracting a git ref out
  # of a demo VALUES list reports a FAIL for a card whose instructions are
  # perfectly runnable — acc-disp-accplan-lane-c-onyx, measured 2026-10-29.
  # An example is not an instruction, and a checker that cannot tell them apart
  # cries wolf until it is ignored. Cut any line that is an INSERT/UPDATE/SELECT
  # before extracting tokens.
  instructions=$(printf '%s' "$body" | sed -E '/[Ii][Nn][Ss][Ee][Rr][Tt] +(INTO|into)/d; /[Uu][Pp][Dd][Aa][Tt][Ee] +[a-z_]+ +SET/d; /^[[:space:]]*[Ss][Ee][Ll][Ee][Cc][Tt][[:space:]]/d')
  split=$(printf '%s' "$instructions" | sed 's/\.\./\n/g')
  tokens=$(printf '%s' "$split" | grep -oE 'track/[A-Za-z0-9_./-]+|packet/[a-z]+/[A-Za-z0-9_.-]+|scripts/[A-Za-z0-9_./-]+\.sh' | sort -u)

  for t in $tokens; do
    # A trailing range fragment can still survive the split; strip it.
    t="${t%%..*}"
    t="${t%[.,;:]}"

    case "$t" in
      scripts/*)
        if ! path_on_base "$t"; then
          printf '  ✗ %s  script not on %s: %s\n' "$cid" "$REF_BASE" "$t"
          bad=$((bad + 1))
        fi
        ;;
      *)
        if ! resolve_ref "$t"; then
          printf '  ✗ %s  git ref does not exist: %s\n' "$cid" "$t"
          bad=$((bad + 1))
        elif ! git rev-parse --verify -q "$t" >/dev/null 2>&1; then
          printf '  ! %s  ref exists only on a remote: %s\n' "$cid" "$t"
          remote_only=$((remote_only + 1))
        fi
        ;;
    esac
  done
done < <(db "select id from acceptances where status in ($STATUS_FILTER) order by status, id;")

if [ "$bad" -gt 0 ]; then
  echo
  echo "FAIL: $bad named ref(s) in $total card(s) cannot be resolved (scope: $STATUS_FILTER)."
  echo "The operator following those cards literally fails. The card's claim may be"
  echo "true and its verdict a valid pass — the INSTRUCTIONS are still unrunnable."
  echo "Fix: cancel_acceptance + request_acceptance under the SAME id with the string"
  echo "corrected, then re-run this script."
  exit 1
fi

if [ "$QUIET" = "1" ]; then
  exit 0
fi

echo "OK: every git ref and script path named by $total card(s) resolves (scope: $STATUS_FILTER)."
[ "$remote_only" -gt 0 ] && echo "note: $remote_only ref(s) exist only on a remote — run git fetch first"
exit 0
