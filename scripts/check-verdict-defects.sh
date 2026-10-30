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

# SQL comments live inside a double-quoted bash string, so bash performs
# command substitution on them BEFORE psql ever sees the SQL. A backtick in a
# -- comment is not a comment to bash: it is a subshell. That happened here -
# `findings` and `finding` were eaten as command names, the query that actually
# ran had those words deleted, and the check printed "no pending cards".
# RULE: in SQL passed to psql -c, no backticks and no double quotes, in the
# comment as well as in the statement.
db() {
  podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F"$(printf '\037')" -c "$1"
}

# Which pending card rests on which verdict, and what that verdict actually says.
# The payload is read, not just the dispatch id.
#
# EVERY TEXT FIELD IS NEWLINE-STRIPPED IN SQL. `while read` reads LINES, so a
# payload containing a newline is not one row - the JSON lands on the next line
# and gets consumed as a new card id, leaving the shell patterns looking at an
# empty or truncated summary. That exact bug shipped in
# check-card-verifications.sh earlier today and I reproduced it here within
# hours. translate() removes both CR and LF so one row is one row.
rows=$(db "select a.id, a.flux_verdict_ref, e.payload->>'verdict',
                  translate(coalesce(e.payload->>'the_honest_summary',''), chr(10)||chr(13), ' '),
                  translate(coalesce(e.payload->>'what_i_did_not_verify',''), chr(10)||chr(13), ' '),
                  translate(coalesce(e.payload->>'finding_1',''), chr(10)||chr(13), ' '),
                  translate(coalesce(e.payload->>'finding_2',''), chr(10)||chr(13), ' '),
                  translate(coalesce(e.payload->>'finding_3',''), chr(10)||chr(13), ' '),
                  -- findings AS AN ARRAY. Flux wrote the plural field findings (a JSON
                  -- array) on one verdict and the singular field finding (a string) on
                  -- another. My query read only finding_1/2/3, so it saw NEITHER -
                  -- two verdicts carrying two real findings each were invisible to
                  -- this check, which then reported nothing wrong. A field name I
                  -- did not anticipate is the same as a field that is not there.
                  translate(coalesce(array_to_string(array(SELECT jsonb_array_elements_text(
                     CASE WHEN jsonb_typeof(e.payload->'findings') = 'array'
                          THEN e.payload->'findings' ELSE '[]'::jsonb END)), ' | '), ''),
                    chr(10)||chr(13), ' ')
           from acceptances a
           join canonical_events e on e.id = a.flux_verdict_ref
           where a.status = 'pending'
           union all
           -- SELF-TEST ROW. A query that returns no rows is indistinguishable from
           -- a query that is broken - both print nothing. With zero pending cards
           -- there is nothing to count either, so a count probe cannot catch a
           -- broken query in the one state where I most want to trust it. This row
           -- is returned no matter what the tables contain, so its absence from the
           -- output proves the QUERY failed, independently of the data.
           -- The second column carries the number of pending cards the SAME query
           -- would return if the sentinel were not there. The sentinel proves the
           -- SQL runs; this number proves the card arm still FINDS cards. Without
           -- it, a join that silently matches nothing - e.g. a condition changed to
           -- a constant - returns only the sentinel and the check reports no pending
           -- cards, which is exactly the false clean it exists to catch.
           select 'SENTINEL-CARD',
                  (select count(*)::text from acceptances a2
                    where a2.status = 'pending'),
                  'pass', '', '', '', '', '', ''
           order by 1")

# An empty result and a broken query look identical. I shipped that bug once
# already in check-card-verifications.sh and rebuilt it here within hours: a
# psql error left rows empty, the script printed no pending cards and exited 0,
# which reads as a clean bill of health. So the query now carries a sentinel row
# that it returns regardless of the data. If it is missing from the output,
# the query is broken and this check FAILS. Matched on the sentinel name alone,
# not on the field separator: BSD grep does not read 037 as an octal escape, so
# a pattern that includes the separator matches nothing and would report FAIL on
# a working query - the mirror image of the bug it was added to catch.
# Proof by mutation: I broke the query (renamed a payload column) and ran it -
# before the sentinel it printed no pending cards and exited 0, which is a
# check that can never fail.
if ! printf '%s\n' "$rows" | grep -q '^SENTINEL-CARD'; then
  echo "FAIL: the verdict query returned no sentinel row - the QUERY is broken,"
  echo "      not that there are no cards. A broken query and an empty result"
  echo "      print the same thing, which is how this check would otherwise report"
  echo "      OK while having looked at nothing."
  echo ""
  echo "      The query, for diagnosis:"
  printf '%s\n' "$rows" | sed 's/^/        /' | head -5
  exit 1
fi

# The sentinel also carries how many pending cards a simple count sees. If the
# sentinel is present but the count is zero while the query found no card rows,
# the card arm is not finding them - a join that matches nothing looks exactly
# like no pending cards. Fail loudly rather than report a clean I did not check.
sentinel_count=$(printf '%s\n' "$rows" | sed -n 's/^SENTINEL-CARD\x1f\([0-9][0-9]*\)\x1f.*/\1/p')
if [ -z "$sentinel_count" ]; then
  echo "FAIL: the sentinel row is present but carries no count - the query shape"
  echo "      changed under me. I cannot tell a working query from a silent one."
  exit 1
fi
# ...but the count is zero while the query DID return card rows. The two
# halves of the same query disagree about the state of the same table. Either
# the count or the card arm is wrong, and I cannot tell which, so I must not
# silently pick the more convenient reading - which would be the card arm,
# because that is the one that lets the check report OK.
card_rows=$(printf '%s\n' "$rows" | grep -vc '^SENTINEL-CARD')
if [ "$sentinel_count" -eq 0 ] && [ "$card_rows" -gt 0 ]; then
  echo "FAIL: the verdict query returned $card_rows card row(s), but its own count says"
  echo "      there are 0 pending cards. The two halves of one query disagree about"
  echo "      the state of one table. Reporting either number as the truth would be"
  echo "      picking the convenient one, so this check stops instead."
  exit 1
fi
if [ "$sentinel_count" -gt 0 ] && [ "$card_rows" -eq 0 ]; then
  echo "FAIL: $sentinel_count pending card(s) exist but the verdict query found none of them."
  echo "      The sentinel proves the SQL runs; this proves the card arm does not."
  echo "      A join that matches nothing is indistinguishable from no pending cards,"
  echo "      and reporting no pending cards is the false clean this check exists to catch."
  exit 1
fi

# Now the sentinel has done its job, drop it from what we read. The first field
# alone recognises it, for the same BSD-grep reason as above.
rows=$(printf '%s\n' "$rows" | grep -v '^SENTINEL-CARD')

if [ -z "$rows" ]; then
  [ "$QUIET" = "1" ] || echo "no pending acceptance cards"
  exit 0
fi

bad=0

while IFS="$(printf '\037')" read -r card verdict verdict_word summary not_verified f1 f2 f3 farr; do
  [ -n "$card" ] || continue
  reasons=""

  # 1. A verdict that is not a pass is not a pass. State it plainly.
  if [ "$verdict_word" != "pass" ]; then
    reasons="${reasons}verdict is '${verdict_word:-<empty>}', not pass|"
  fi

  # 2. The payload names findings. A finding is content in a finding_* field.
  #
  #    BUT NOT EVERY FINDING IS A DEFECT IN THE WORK. Flux's re-verification of
  #    026_topic_close recorded a finding whose own text ends "Documentation, not
  #    behaviour; the guard passes and the file is correct" - a verifier noting a
  #    stale registration comment. That is an honest annotation, not a reason to
  #    withhold the operator's Accept, and flagging it as a named defect would have
  #    had me tell him a passing card was broken. Twice now a keyword rule has
  #    produced exactly that kind of false alarm.
  #
  #    So: a finding that DISCLAIMS itself is not a defect. Look for the verifier
  #    explicitly downgrading it before treating it as one.
  for f in "$f1" "$f2" "$f3" "$farr"; do
    [ -n "$f" ] || continue
    flow=$(printf '%s' "$f" | tr '[:upper:]' '[:lower:]')
    case "$flow" in
      *"documentation, not behaviour"*|*"not a defect"*|*"cosmetic"*|*"stale comment"*|\
      *"stale documentation"*|*"does not affect"*|*"no behavioural impact"*|*"informational"*)
        # Self-disclaimed. Record it as a note, not a reason to refuse.
        ;;
      *)
        reasons="${reasons}names a finding: $(printf '%s' "$f" | cut -c1-120)|"
        break
        ;;
    esac
  done

  # 3. The honest summary says the thing does not work. These phrases are the
  #    tell. They are written in lower case and matched case-insensitively so a
  #    shouted "THE FIX DOES NOT WORK" is caught as readily as a quiet one.
  #
  #    FALSE POSITIVE, caught the first time this ran for real: a verifier
  #    routinely writes "the defect is real and reproduces at the line numbers
  #    cited" - meaning the ORIGINAL defect he was asked to verify is genuine,
  #    which is a PASS with confirmation, not a defect in the new work. Matching
  #    the bare word "defect" flagged a clean verdict and would have had me tell
  #    the operator a good card was broken.
  #
  #    So the rule is not "does the payload contain the word defect". It is "does
  #    the verdict say THE WORK IT IS PASSING does not work". Phrases that
  #    describe the pre-existing defect being confirmed are excluded first.
  low=$(printf '%s %s' "$summary" "$not_verified" | tr '[:upper:]' '[:lower:]')
  # Strip the confirmation-of-the-original-defect phrasing, then look for what
  # is left. "the defect is real", "the premise is real", "the reported defect
  # reproduces" all describe the thing being verified, not a fault in the fix.
  low=$(printf '%s' "$low" | sed -E 's/the (defect|premise|reported defect|bug) is (real|correct|confirmed|genuine|accurate)//g; s/(defect|premise) (is|was) real//g; s/reproduces at the line numbers cited//g')
  # Second false positive, caught the same run: "the mutation that PREVIOUSLY
  # survived now goes red" is a verifier reporting that a fix WORKS. Matching the
  # bare word "survives" flagged a clean verdict. The honest lesson is that a
  # keyword cannot carry tense - "survived, and was then fixed" and "survives"
  # are opposite facts spelled with the same stem. So the patterns below require
  # the present-tense claim ("one mutation survives", "mutation survives",
  # "survives the mutation") rather than the bare stem.
  # This check is therefore a PROMPT, not a verdict. It cannot read; it can only
  # ask a human to look. Every FAIL it prints must be read, not obeyed.
  case "$low" in
    *"does not work"*|*"not working"*|*"fix is not complete"*|*"is unpinned"*|\
    *"not covered"*|*"uncovered"*|*"failing"*|*"not pinned"*|*"one mutation survives"*|\
    *"mutation survives"*|*"survives the mutation"*|*"gap in the test"*|\
    *"tests are red"*|*"is red"*|*"stays red"*|*"remains red"*)
      # head -1, not just -m1: BSD grep's -m1 limits matching LINES, and one
      # line can yield several overlapping matches, so $hit arrived with two
      # lines and printed as two separate "reasons" - which reads as two defects
      # where there is one verdict.
      hit=$(printf '%s' "$low" | grep -o '.\{0,60\}\(does not work\|not working\|is unpinned\|not covered\|uncovered\|failing\|defect\).\{0,60\}' | head -1)
      reasons="${reasons}the honest summary says so: ...${hit}...|"
      ;;
  esac

  # 4. A verdict with an empty summary is not a verdict I can rely on. Not a
  #    defect in the work, but it is one in the record: I cannot tell a clean
  #    pass from an unexamined one.
  if [ -z "$summary" ] && [ -z "$f1" ] && [ -z "$f2" ] && [ -z "$f3" ] && [ -z "$farr" ]; then
    reasons="${reasons}the verdict carries no summary and no findings - nothing to distinguish it from 'I did not look'|"
  fi

  if [ -n "$reasons" ]; then
    bad=$((bad + 1))
    # Printed AS FOUND, not accumulated into a buffer and re-split later. The
    # buffer version used '|' as a separator while the reasons themselves can
    # contain '|', and it emitted blank card/verdict lines when a payload held a
    # newline. Printing at the point of discovery removes both failure modes.
    echo "  card:     $card"
    echo "  verdict:  $verdict"
    printf '%s\n' "$reasons" | tr '|' '\n' | while IFS= read -r line; do
      [ -n "$line" ] && echo "            - $line"
    done
    echo
  fi
done <<< "$rows"

if [ "$bad" -gt 0 ]; then
  echo "FAIL: ${bad} pending card(s) rest on a verdict that names a defect."
  echo
  echo "A pass that names its own defect is not a pass. Mr. Zhang's Accept is the"
  echo "only thing that merges a card, and he would be merging on a verdict that"
  echo "already told us the work is broken."
  echo
  # (each card's reasons were printed inside the loop above)
  echo "The rework belongs to the seat that did the work. A defect found by the"
  echo "verifier is not a rejection of the work - it is the verifier doing its job."
  exit 1
fi

[ "$QUIET" = "1" ] || echo "OK: every pending card rests on a verdict that does not name a defect."
exit 0
