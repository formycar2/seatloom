#!/usr/bin/env bash
# Does check-verdict-defects.sh still refuse a verdict that says nothing, and
# no longer refuse one that says plenty under field names it never enumerated?
#
# The bug this test exists for: ev-oO1C34dPuLklTsJRUETgj is the richest verdict in
# the ledger - 7351 chars, 20 keys, behaviours, merges_and_holds, the_numbers,
# short_circuit, not_certified, verdict_rationale - and the check told the operator
# it "carries no account of what it did". It failed because the account lived in
# field names none of the eleven spellings the check read covers. The check's own
# comment already said a field name I did not anticipate is the same as a field
# that is not there, and then it enumerated eleven fields.
#
# A FIXTURE THAT CANNOT FAIL PROVES NOTHING. Vela's knowledge-convention test built
# an empty scratch repo, so its BEFORE number was structurally zero and every
# assertion after it was measuring an empty directory. Both directions here are
# built so they can actually fail: the thin case must still be refused, and the
# rich case must now be accepted. If the check were made vacuous - everything
# passes - the thin case catches it.

set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

PASS=0; FAIL=0
say() { if [ "$1" = ok ]; then PASS=$((PASS+1)); echo "  ✓ $2"; else FAIL=$((FAIL+1)); echo "  ✗ $2"; fi; }

TMP=$(mktemp -d /tmp/verdict-defects.XXXXXX) || exit 2
trap 'rm -rf "$TMP"' EXIT

# The check's own query, run against a fixture database, is not available here:
# the check talks to the live ledger. So the two directions are exercised by
# feeding the exact same payload shapes through the check's decision logic,
# extracted verbatim, rather than by writing to the real database.
#
# Extracting the logic rather than the whole script is deliberate: the script's
# other half queries the live DB, and running that against a fixture would
# measure nothing about the rule under test.

extract_rule4() {
  sed -n '/^  #    So the test asks whether ANY substantive account exists/,/^  fi$/p' "$1"
}

GATE="$PWD/scripts/check-verdict-defects.sh"

echo "── 1. the rule is extracted, not paraphrased ──"
[ -s "$TMP/rule4.sh" ] || extract_rule4 "$GATE" > "$TMP/rule4.sh"
grep -q 'account_len' "$TMP/rule4.sh" && say ok "rule 4 now measures the payload" || say no "rule 4 does not measure the payload"

echo
echo "── 2. a verdict that says ONLY pass must still be refused ──"
# This is the direction that stops the fix from being vacuous. The floor must
# hold: {"verdict":"pass"} plus an id is a claim, not an account.
(
  cd "$TMP" || exit 2
  summary=""; evidence=""; directions=""; mutations=""
  not_verified=""; criteria=""; product_code=""
  f1=""; f2=""; f3=""; farr=""
  account_len=24          # {"verdict":"pass","id":"ev-x"} minus the excluded keys
  reasons=""
  # shellcheck disable=SC1090
  source ./rule4.sh
  printf '%s' "$reasons"
) > "$TMP/thin.out"
if [ -s "$TMP/thin.out" ]; then
  say ok "a verdict carrying only a pass is still refused"
else
  say no "a verdict carrying only a pass now PASSES - the fix made the check vacuous"
fi

echo
echo "── 3. a rich verdict under UNANTICIPATED field names must be accepted ──"
# The exact shape of the verdict that was called empty: 7351 chars spread over
# behaviours, merges_and_holds, the_numbers, short_circuit, not_certified.
(
  cd "$TMP" || exit 2
  summary=""; evidence=""; directions=""; mutations=""
  not_verified=""; criteria=""; product_code=""
  f1=""; f2=""; f3=""; farr=""
  account_len=7351
  reasons=""
  # shellcheck disable=SC1090
  source ./rule4.sh
  printf '%s' "$reasons"
) > "$TMP/rich.out"
if [ -s "$TMP/rich.out" ]; then
  say no "a rich verdict under unanticipated field names is still refused"
else
  say ok "a rich verdict under unanticipated field names is accepted"
fi

echo
echo "── 4. the floor is a real boundary, not decoration ──"
# One account at just under the floor and one just over, so a future edit that
# sets the floor to zero cannot hide.
for len in 100 200 199 201; do
  (
    cd "$TMP" || exit 2
    summary=""; evidence=""; directions=""; mutations=""
    not_verified=""; criteria=""; product_code=""
    f1=""; f2=""; f3=""; farr=""
    account_len=$len
    reasons=""
    # shellcheck disable=SC1090
    source ./rule4.sh
    printf '%s' "$reasons"
  ) > "$TMP/len.$len.out"
  if [ -s "$TMP/len.$len.out" ]; then verdict=refused; else verdict=accepted; fi
  echo "    account_len=$len -> $verdict"
done
[ -s "$TMP/len.100.out" ] && say ok "100 chars refused" || say no "100 chars accepted"
[ -s "$TMP/len.201.out" ] || say ok "201 chars accepted" || say no "201 chars refused"

echo
echo "── 5. the SQL that produces account_len really sums the payload ──"
# A rule that measures the payload is only as good as the expression feeding it.
# Run it against a JSON literal - no live database, no fixture that can't fail.
SUM=$(podman exec seatloom-postgres psql -X -v ON_ERROR_STOP=1 -U seatloom -d seatloom -t -A -c "
  with e as (select '{\"verdict\":\"pass\",\"id\":\"ev-x\",\"branch\":\"packet/b\",\"commit\":\"abc\",\"dispatch_id\":\"d\",\"subject\":\"s\",\"the_numbers\":[\"a\",\"b\"],\"verdict_rationale\":\"because it works both ways\"}'::jsonb as payload)
  select (select coalesce(sum(length(e.payload->>k.key)), 0) from jsonb_each_text(e.payload) k
           where k.key not in ('verdict','branch','commit','dispatch_id','verification_dispatch_id','id','subject'))
  from e;" 2>&1)
# the_numbers is an ARRAY, so ->> renders it as JSON text: 9 chars. plus 27 = 36.
if [ "$SUM" = "36" ]; then
  say ok "account_len sums every non-identity field, arrays included (got $SUM)"
else
  say no "account_len is not summing what it should (got '$SUM', expected 36)"
fi

echo
echo "── 6. the read list and the select list carry the same number of fields ──"
# A read list that gained a column the SQL did not gain shifts every field after
# the first: the summary is read as a card id and the check examines garbage. And
# the reverse - the SQL gaining a column the read list did not - makes postgres
# refuse the whole UNION. BOTH DIRECTIONS HAPPENED IN ONE EDIT: I added account_len
# to the select list and forgot the sentinel row, and postgres rejected the query
# with each UNION query must have the same number of columns. The check reported it
# honestly as the QUERY is broken, which is the sentinel doing its job. So the
# assertion is on the observed counts, not on a grep.
#
# COUNT THE VARIABLES, NOT THE LINES OR THE COMMAS. My first version of this
# assertion did sed -n '/while.../,/; do/p' | tail -1 | wc -w and reported 8, because
# the range ends on the very line it is reading and tail -1 took a fragment. It also
# did grep -c "''", which counts LINES not occurrences. Two broken measurements
# reporting a clean script as broken - the same shape as the defect under test.
# Extract the one line and count its tokens, minus the read -r prefix.
# 17 tokens = read + -r + 15 variable names. Getting this arithmetic wrong is how
# my first version reported 8 for a line with fifteen names on it.
NTOK=$(grep -o 'read -r card.*account_len' "$GATE" | tr ' ' '\n' | grep -c .)
NREAD=$((NTOK - 2))
if [ "$NREAD" = "15" ]; then
  say ok "the read list carries exactly 15 variables ($NTOK tokens minus read -r)"
else
  say no "the read list carries $NREAD variables ($NTOK tokens), expected 15"
fi

# The sentinel row is the second arm of the same UNION, so it must carry 15 too.
# Count every quoted literal on that select, including the non-empty ones.
NSENT=$(sed -n "/select 'SENTINEL-CARD'/,/order by 1/p" "$GATE" | grep -o "'[^']*'" | grep -c .)
if [ "$NSENT" = "15" ]; then
  say ok "the sentinel row carries 15 columns (got $NSENT) - the UNION will not be refused"
else
  say no "the sentinel row carries $NSENT columns, expected 15 (would break the UNION)"
fi

# The end-to-end proof: run the real check against the real ledger. This is the
# assertion that would have caught my UNION break the moment I made it, because a
# broken query and an empty result print the same thing and only the run tells
# them apart.
if bash "$GATE" --quiet >"$TMP/live.out" 2>&1; then
  say ok "the real check exits 0 on the live ledger - query runs, columns aligned"
elif grep -q 'SENTINEL' "$TMP/live.out"; then
  say no "the real check says the QUERY is broken: $(head -2 "$TMP/live.out" | tr '\n' ' ')"
else
  say no "the real check still names a defect: $(head -3 "$TMP/live.out" | tr '\n' ' ')"
fi

# And the check must not have gone vacuous: the one pending card must actually be
# examined. Its verdict is ev-oO1C34dPuLklTsJRUETgj, whose account lives in
# unanticipated field names and whose account length is measured here.
ACC=$(podman exec seatloom-postgres psql -X -v ON_ERROR_STOP=1 -U seatloom -d seatloom -t -A -c "
  select a.id||'='||(select coalesce(sum(length(x.payload->>k.key)), 0)
                       from jsonb_each_text(x.payload) k
                      where k.key not in ('verdict','branch','commit','dispatch_id',
                                          'verification_dispatch_id','id','subject'))
    from acceptances a
    join canonical_events x on x.id = a.flux_verdict_ref
   where a.status = 'pending'" 2>&1)
echo "    measured: $ACC"
case "$ACC" in
  *=*[0-9][0-9][0-9][0-9]*) say ok "the pending card's verdict carries a measured account ($ACC)" ;;
  *) say no "could not measure the pending card's account: $ACC" ;;
esac

echo
echo "passed=$PASS failed=$FAIL"
[ "$FAIL" -eq 0 ]
