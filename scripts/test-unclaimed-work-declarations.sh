#!/usr/bin/env bash
# Does check-unclaimed-work.sh still refuse an unclassified candidate, and no longer
# report a permanent 13 for candidates that have a declared reason?
#
# The bug this test exists for: the checker reported the same 13 candidates for two
# days. All 13 were classified by hand — 0 lost, 5 blocked on one operator decision, 2
# superseded by a passing redo, 6 whose deliverable is a card or verdict that correctly
# carries the work's dispatch id rather than the requester's. A report whose answer is
# permanently non-zero for structural reasons stops being read.
#
# THE GUARD THAT MAKES THIS TEST WORTH RUNNING: an entry only counts as declared if its
# reason is non-empty. So `--classify` cannot make the check green. If a future edit
# drops that requirement, the "template must not go green" case below catches it.
#
# A FIXTURE THAT CANNOT FAIL PROVES NOTHING. Both directions here are built to fail:
# the unclassified case must still be refused, and the declared case must be accepted.
# If the change made everything pass, the unclassified case catches it.

set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

PASS=0; FAIL=0
say() { if [ "$1" = ok ]; then PASS=$((PASS+1)); echo "  ✓ $2"; else FAIL=$((FAIL+1)); echo "  ✗ $2"; fi; }

TMP=$(mktemp -d /tmp/unclaimed.XXXXXX) || exit 2
trap 'rm -rf "$TMP"' EXIT

GATE="$PWD/scripts/check-unclaimed-work.sh"
DECL="$PWD/.gitlab/unclaimed-declared.txt"
BACKUP="$TMP/declared.backup"
[ -f "$DECL" ] && cp "$DECL" "$BACKUP"
restore() { if [ -f "$BACKUP" ]; then cp "$BACKUP" "$DECL"; else rm -f "$DECL"; fi; }
trap 'restore; rm -rf "$TMP"' EXIT

# The checker's real query runs against the live ledger, so the directions are exercised
# against the LIVE candidate set — which is non-empty and therefore a real fixture. What
# varies is the declaration file, which is the thing under test.
live=$(bash "$GATE" --quiet 2>&1); live_rc=$?
n_live=$(printf '%s\n' "$live" | awk '/^UNCLASSIFIED/{print $NF}')
[ -n "$n_live" ] || n_live=0
echo "── live state: $n_live unclassified candidate(s), exit $live_rc ──"

echo
echo "── 1. an undeclared candidate is still refused ──"
# With no declaration file at all, every live candidate is unclassified.
rm -f "$DECL"
bash "$GATE" --quiet >"$TMP/none.out" 2>&1; rc=$?
n_none=$(awk '/^UNCLASSIFIED/{print $NF}' "$TMP/none.out")
if [ "$rc" -eq 1 ] && [ "$n_none" -gt 0 ]; then
  say ok "no declaration file -> $n_none unclassified, exit 1 (the gate did not go vacuous)"
else
  say no "no declaration file -> rc=$rc unclassified=$n_none (expected exit 1 with candidates)"
fi

echo
echo "── 2. a declared candidate WITH a reason is skipped ──"
# Declare every live candidate with a non-empty reason. This is the only way to reach 0,
# and it requires writing a reason per entry — which is the review the check forces.
#
# MY FIRST VERSION OF THIS ASSERTION WAS WRONG: it parsed `UNCLASSIFIED candidates
# requiring review: N` out of --quiet output, but that line is suppressed when the count
# is zero, so awk printed nothing and the case failed on an empty string while the script
# was behaving correctly. Parse the count from the non-quiet run instead.
: > "$DECL"
bash "$GATE" --classify >/dev/null 2>&1
python3 - "$DECL" <<'PY'
import sys
p=sys.argv[1]
out=[]
for l in open(p,encoding='utf-8'):
    s=l.rstrip('\n')
    if not s or s.startswith('#'):
        out.append(s); continue
    if s.endswith('|'):
        out.append(s+'declared for the test: classified by hand, not lost work')
    else:
        out.append(s)
open(p,'w',encoding='utf-8').write('\n'.join(out)+'\n')
PY
bash "$GATE" >"$TMP/all.out" 2>&1; rc=$?
n_all=$(awk '/^UNCLASSIFIED candidates/{print $NF}' "$TMP/all.out")
if [ "$rc" -eq 0 ] && [ "$n_all" = "0" ]; then
  say ok "every candidate declared with a reason -> 0 unclassified, exit 0"
else
  say no "all declared -> rc=$rc unclassified='$n_all' (expected exit 0, 0 unclassified)"
fi

echo
echo "── 3. the LOAD-BEARING case: a template with EMPTY reasons must NOT go green ──"
# This is what stops the change from being a suppression switch. --classify writes empty
# reasons; an empty reason is not a declaration. If a future edit drops that requirement
# the check would go green from running one command, and this catches it.
bash "$GATE" --classify >/dev/null 2>&1
empty_reasons=$(grep -c '|$' "$DECL" 2>/dev/null || echo 0)
bash "$GATE" --quiet >"$TMP/tmpl.out" 2>&1; rc=$?
n_tmpl=$(awk '/^UNCLASSIFIED/{print $NF}' "$TMP/tmpl.out")
if [ "$rc" -eq 1 ] && [ "$n_tmpl" -gt 0 ]; then
  say ok "template with $empty_reasons empty reasons -> $n_tmpl unclassified, exit 1 (cannot suppress by regenerating)"
else
  say no "template with empty reasons -> rc=$rc unclassified=$n_tmpl (the check can be suppressed by --classify)"
fi

echo
echo "── 4. the exit code is preserved, not collapsed ──"
# 0 ok / 1 review required / 2 self-failure. A collapsed 1 into 0 is the false clean.
#
# TWO THINGS MY FIRST VERSION GOT WRONG HERE. It expected exit 0 from an ABSENT
# declaration file — but absent means everything is unclassified, which is case 1 and
# exit 1; the exit-0 state is the fully-declared one. And it tried to break the source
# with PATH=/nonexistent-dir as a command prefix, which makes the shell unable to find
# bash itself and yields 127 before the script's own guard ever runs. Break podman only.
write_all_declared() {
  bash "$GATE" --classify >/dev/null 2>&1
  python3 - "$DECL" <<'PY'
import sys
p=sys.argv[1]
out=[]
for l in open(p,encoding='utf-8'):
    s=l.rstrip('\n')
    if not s or s.startswith('#'):
        out.append(s); continue
    out.append(s if not s.endswith('|') else s+'declared for the test')
open(p,'w',encoding='utf-8').write('\n'.join(out)+'\n')
PY
}
write_all_declared; bash "$GATE" --quiet >/dev/null 2>&1; got=$?
[ "$got" = "0" ] && say ok "fully declared -> exit 0" || say no "fully declared -> exit $got (expected 0)"
bash "$GATE" --classify >/dev/null 2>&1; bash "$GATE" --quiet >/dev/null 2>&1; got=$?
[ "$got" = "1" ] && say ok "empty reasons -> exit 1" || say no "empty reasons -> exit $got (expected 1)"
# 2: an unreadable source must be UNCHECKABLE, never "no candidates". A shim
# directory containing a podman that FAILS breaks the source and nothing else.
#
# MY FIRST ATTEMPT set PATH to a directory with no binaries at all, which also broke the
# lookup of `bash` itself and produced exit 127 before the script's guard ran. What has to
# be broken is podman, not the shell — so the shim dir holds a podman that fails, and the
# real PATH is preserved for everything else.
mkdir -p "$TMP/bin"
printf '#!/bin/sh\necho "podman: shim, deliberately failing" >&2\nexit 1\n' > "$TMP/bin/podman"
chmod +x "$TMP/bin/podman"
PATH="$TMP/bin:$PATH" bash "$GATE" --quiet >"$TMP/uncheck.out" 2>&1; got=$?
if [ "$got" -eq 2 ] && grep -q 'UNCHECKABLE' "$TMP/uncheck.out"; then
  say ok "an unreachable ledger -> exit 2 UNCHECKABLE (not a false clean)"
else
  say no "unreachable ledger -> exit $got (expected 2 UNCHECKABLE): $(head -1 "$TMP/uncheck.out")"
fi

echo
echo "── 5. a stale declaration is reported, not silently kept ──"
# A declaration that no longer refers to any candidate is a claim nobody checks. Prune
# notices must appear even when other candidates are still unclassified — on the first
# version the notice sat below an early `exit 1`, so it was unreachable in exactly the
# state the checker is usually in.
rm -f "$DECL"
printf '# test\ndisp-does-not-exist-xyz|declared for the test\n' > "$DECL"
bash "$GATE" >"$TMP/stale.out" 2>&1
if grep -q 'no longer appear as candidates' "$TMP/stale.out" && grep -q 'disp-does-not-exist-xyz' "$TMP/stale.out"; then
  say ok "a declaration referring to nothing is reported by id, even alongside unclassified candidates"
else
  say no "a stale declaration is not reported: $(tail -3 "$TMP/stale.out" | tr '\n' ' ')"
fi

echo
echo "── 6. nothing is hidden: declared entries still print with their reason ──"
# The point is a headline that is no longer permanently non-zero, NOT suppression. Every
# declared candidate must still be shown with the reason that excused it.
bash "$GATE" --classify >/dev/null 2>&1
python3 - "$DECL" <<'PY'
import sys
p=sys.argv[1]
out=[]
for l in open(p,encoding='utf-8'):
    s=l.rstrip('\n')
    if not s or s.startswith('#'):
        out.append(s); continue
    out.append(s if not s.endswith('|') else s+'declared for the test: the reason must be visible')
open(p,'w',encoding='utf-8').write('\n'.join(out)+'\n')
PY
bash "$GATE" >"$TMP/show.out" 2>&1
if grep -q 'Declared and skipped' "$TMP/show.out" && grep -q 'the reason must be visible' "$TMP/show.out"; then
  say ok "declared candidates are still printed, each with its reason"
else
  say no "declared candidates are not shown with their reasons"
fi

restore
echo
echo "passed=$PASS failed=$FAIL"
[ "$FAIL" -eq 0 ]
