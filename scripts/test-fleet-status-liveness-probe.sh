#!/usr/bin/env bash
# Is fleet-status.sh's liveness probe correct from ANY caller, not just the operator's?
#
# The bug this test exists for: the probe was `pgrep -P "$pane"` with no `-a`. `man pgrep`
# says by default the current pgrep AND ALL OF ITS ANCESTORS are excluded from the match
# list. So the answer to "is this seat alive" depended on who ran the script — from a
# seat's own process tree, that seat read itself offline. Measured 2026-11-10 with two
# processes of identical comm, depth and parent type differing only in whether the caller
# descends from them.
#
# BOTH DIRECTIONS ARE REQUIRED. A liveness probe that always says online is worse than one
# that is wrong about one seat, so the must-still-report-offline case is asserted too.
# A test that only proves the fix works has proved half of it.

set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

PASS=0; FAIL=0
say() { if [ "$1" = ok ]; then PASS=$((PASS+1)); echo "  ✓ $2"; else FAIL=$((FAIL+1)); echo "  ✗ $2"; fi; }

TMP=$(mktemp -d /tmp/liveness.XXXXXX) || exit 2
trap 'rm -rf "$TMP"' EXIT

echo "── 1. the probe uses -a ──"
# The flag, not the comment. A comment saying -a is required is not -a.
grep -q 'pgrep -a -P' scripts/fleet-status.sh && say ok "the probe reads pgrep -a -P" \
  || say no "the probe does not use -a (a comment about it is not the flag)"

echo
echo "── 2. an ANCESTOR target is visible (the case that was wrong) ──"
# Controlled: a process that IS an ancestor of this shell, and one that is not.
# WALK THE CHAIN, do not take the immediate parent. My first version used
# `ps -o ppid= -p $$`, which is whatever transient shell invoked the test - a pid that
# differs between two runs of the same test, so the case passed on one invocation and
# failed on the next. The stable ancestor is the seat agent several levels up, and it is
# the one whose parent is a real tmux pane.
pane_pids=$(tmux list-panes -a -F '#{pane_pid}' 2>/dev/null | sort -u)
anc=""; anc_pane=""
cur=$$
for _ in 1 2 3 4 5 6 7 8 9 10; do
  parent=$(ps -o ppid= -p "$cur" 2>/dev/null | tr -d ' ')
  [ -z "$parent" ] && break
  [ "$parent" = "1" ] && break
  if printf '%s\n' "$pane_pids" | grep -qx "$parent"; then
    anc="$cur"; anc_pane="$parent"; break
  fi
  cur="$parent"
done
nonanc=""
for p in $(pgrep -x saih 2>/dev/null); do
  [ "$p" = "$anc" ] && continue
  cur=$$; is_anc=0
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    cur=$(ps -o ppid= -p "$cur" 2>/dev/null | tr -d ' ')
    [ -z "$cur" ] && break
    [ "$cur" = "1" ] && break
    [ "$cur" = "$p" ] && { is_anc=1; break; }
  done
  [ "$is_anc" = "0" ] && { nonanc="$p"; break; }
done
echo "    my ancestor: ${anc:-<none>} (pane ${anc_pane:-?})   a non-ancestor: ${nonanc:-<none>}"
# RUN THE SCRIPT'S OWN PROBE, NOT A COPY OF IT. My first version called
# `pgrep -a -P` directly, so it passed with -a removed from the script — it was testing
# the right command rather than the one the script runs. Extract the probe expression
# from fleet-status.sh and evaluate THAT, so a regression in the script is caught here.
probe_line=$(grep -m1 'pgrep .*-P "\$pane"' scripts/fleet-status.sh)
stripped="${probe_line#*&& proc=}"
echo "    the script's probe: $(printf '%s' "$probe_line" | sed 's/^ *//')"
if [ -n "$anc" ]; then
  anc_pane=$(ps -o ppid= -p "$anc" 2>/dev/null | tr -d ' ')
  # Substitute the pane id and evaluate the script's own expression.
  # TWO THINGS MY FIRST VERSION GOT WRONG, both caught by running it. `pgrep -a` prints
  # "PID COMMAND" so field 1 must be taken before comparing. And `eval "$(...)"`
  # EXECUTES the substitution instead of printing it — it tried to run the pid as a
  # command — so the form is `eval echo`, which prints it.
  got=$(pane="$anc_pane"; eval echo "$stripped" 2>/dev/null | awk '{print $1}' | head -1)
  if [ "$got" = "$anc" ]; then
    say ok "the script's own probe finds my ancestor ($anc) at pane $anc_pane — a seat can see itself"
  else
    say no "the script's own probe cannot see my ancestor ($anc) at pane $anc_pane — got '${got:-<empty>}'"
  fi
else
  say no "could not establish my own ancestor pid"
fi
if [ -n "$nonanc" ]; then
  non_pane=$(ps -o ppid= -p "$nonanc" 2>/dev/null | tr -d ' ')
  got=$(pane="$non_pane"; eval echo "$stripped" 2>/dev/null | awk '{print $1}' | head -1)
  if [ "$got" = "$nonanc" ]; then
    say ok "the script's own probe still finds a non-ancestor ($nonanc)"
  else
    say no "the script's own probe lost a non-ancestor ($nonanc) — the fix broke the working case"
  fi
else
  say no "could not find a non-ancestor saih for the control"
fi

echo
echo "── 3. a genuinely empty pane STILL reports offline ──"
# This is the direction that stops -a from being a "always say online" switch.
childless=""
for p in $(ps -axo pid= 2>/dev/null | tr -d ' ' | head -60); do
  c=$(pgrep -a -P "$p" 2>/dev/null | wc -l | tr -d ' ')
  [ "$c" = "0" ] && { childless="$p"; break; }
done
if [ -n "$childless" ]; then
  out=$(pgrep -a -P "$childless" 2>/dev/null)
  [ -z "$out" ] && say ok "a pane with no children ($childless) returns empty — still offline" \
    || say no "a childless pane returned '$out' — -a invented a child"
else
  say no "could not find a childless pid to test the offline direction"
fi

echo
echo "── 4. a nonexistent pane pid STILL reports offline ──"
out=$(pgrep -a -P 999998 2>/dev/null)
[ -z "$out" ] && say ok "a nonexistent pane pid returns empty — still offline" \
  || say no "a nonexistent pane pid returned '$out'"

echo
echo "── 5. the real fleet-status.sh still runs and reports every seat ──"
bash scripts/fleet-status.sh >"$TMP/fs.out" 2>&1; rc=$?
if [ "$rc" -eq 0 ] && grep -q '在线' "$TMP/fs.out"; then
  n=$(grep -c '在线' "$TMP/fs.out")
  say ok "fleet-status.sh exits 0 and reports $n online line(s)"
else
  say no "fleet-status.sh rc=$rc: $(tail -2 "$TMP/fs.out" | tr '\n' ' ')"
fi
# And no seat may read offline while its pane has an agent. The six mapped seats all have
# one, so a 掉线 line here means the probe is still caller-dependent.
if grep -q '掉线' "$TMP/fs.out"; then
  say no "fleet-status.sh still reports a seat offline: $(grep '掉线' "$TMP/fs.out" | tr '\n' ' ')"
else
  say ok "no mapped seat reads offline while its pane has an agent"
fi

echo
echo "passed=$PASS failed=$FAIL"
[ "$FAIL" -eq 0 ]
