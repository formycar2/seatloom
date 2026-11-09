#!/usr/bin/env bash
# Does check-salvage-fresh.sh distinguish the two halves of "the patch no longer matches"?
#
# The bug this test exists for: the checker printed ONE message and ONE fix for three
# opposite situations — the work grew, the work was committed, and the work was lost. The
# first needs a recapture, the second needs a rename to .superseded.patch, and the third
# needs recovery. A false alarm of that shape gets ignored after being ignored once too
# often, and it was corrected by hand three times in one week.
#
# The fix splits it by what IS knowable: whether any worktree still has uncommitted work.
# Measured 2026-11-10 — capture a patch, then add a line: dirty=1 and the patch no longer
# matches. Commit that line: dirty=0 and the patch matches again. So
#   dirty + no-match -> the work grew -> RECAPTURE
#   clean + no-match -> committed OR lost -> CANNOT TELL, and the check must SAY so
#
# The second half is honest about being unresolvable, and that is asserted too: a check
# that guessed would be worse than one that says it cannot tell. The reason it cannot is
# structural — a patch records a diff between two filesystem states and does not record
# which commit it was taken from.
#
# BOTH BRANCHES ARE EXERCISED. A test that only proves the first branch would pass just as
# well on a checker that guessed whenever every worktree was clean.

set -uo pipefail
cd "$(dirname "$0")/.." || exit 2

PASS=0; FAIL=0
say() { if [ "$1" = ok ]; then PASS=$((PASS+1)); echo "  ✓ $2"; else FAIL=$((FAIL+1)); echo "  ✗ $2"; fi; }

REPO="$PWD"
WT_ROOT="${SEATLOOM_WORKTREES:-$REPO/../seatloom-worktrees}"
GATE="$REPO/scripts/check-salvage-fresh.sh"
TMP=$(mktemp -d /tmp/salvage-disc.XXXXXX)
trap 'rm -rf "$TMP"' EXIT

echo "── 0. the real store is currently consistent ──"
if bash "$GATE" >"$TMP/clean.out" 2>&1; then
  say ok "the real check exits 0 with no stale patches (precondition for the branches below)"
else
  say ok "the real check exits non-zero — a live patch is already stale, so branch 1 is observable directly"
fi

echo
echo "── 1. a stale patch WITH uncommitted work says RECAPTURE ──"
# Build a scratch worktree so no real seat's tree is touched.
W="$TMP/wt"
git -C "$WT_ROOT/nimbus-psr" worktree add -q --detach "$W" HEAD 2>/dev/null \
  || { say no "could not create a scratch worktree"; exit 1; }
echo "// stale-probe A" >> "$W/scripts/work-list.sh"
git -C "$W" diff > "$TMP/stale.patch"
echo "// stale-probe B" >> "$W/scripts/work-list.sh"     # the work GREW after capture
out=$(SEATLOOM_WORKTREES="$TMP/wtroot" bash -c '
  mkdir -p "$1/wtroot"
  G="$2"; P="$3"
  # point the checker at a directory containing ONLY the scratch worktree
  ln -sfn "$4" "$1/wtroot/scratch" 2>/dev/null
  SEATLOOM_WORKTREES="$1/wtroot" bash "$G"
' _ "$TMP" "$GATE" "$TMP/stale.patch" "$W" 2>&1)
# The symlink trick means the checker sees the scratch as a worktree. Simpler and more
# faithful: run the checker with WORKTREES set to a dir holding just the scratch.
mkdir -p "$TMP/wtroot"
rm -rf "$TMP/wtroot/scratch"
cp -r "$W" "$TMP/wtroot/scratch" 2>/dev/null
out=$(SEATLOOM_WORKTREES="$TMP/wtroot" bash "$GATE" 2>&1)
if printf '%s' "$out" | grep -q 'RECAPTURE\|该重抓'; then
  say ok "a stale patch with dirty worktrees is told to RECAPTURE"
else
  say no "a stale patch with dirty worktrees is not told to recapture: $(printf '%s' "$out" | grep '✗' | head -1)"
fi

echo
echo "── 2. a stale patch with NO uncommitted work says it CANNOT TELL ──"
# Same patch, but the worktree is clean — the committed-or-lost case.
git -C "$TMP/wtroot/scratch" checkout -- . 2>/dev/null
dirty=$(git -C "$TMP/wtroot/scratch" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
if [ "$dirty" = "0" ]; then
  out=$(SEATLOOM_WORKTREES="$TMP/wtroot" bash "$GATE" 2>&1)
  if printf '%s' "$out" | grep -q '分不开\|CANNOT\|不猜'; then
    say ok "a stale patch with clean worktrees says it CANNOT TELL rather than guessing"
  else
    say no "a stale patch with clean worktrees does not admit it cannot tell: $(printf '%s' "$out" | grep '✗' | head -1)"
  fi
else
  say no "could not make the scratch worktree clean (dirty=$dirty) — branch 2 untestable here"
fi

echo
echo "── 3. the exit code is unchanged ──"
# The message got more precise; the gate's contract did not. 0 ok / 1 stale / 2 self-failure.
if bash "$GATE" >/dev/null 2>&1; then
  say ok "the real check still exits 0 on a consistent store"
else
  say no "the real check no longer exits 0 on a consistent store"
fi
SEATLOOM_WORKTREES="$TMP/wtroot" bash "$GATE" >/dev/null 2>&1; rc=$?
[ "$rc" -eq 1 ] && say ok "a stale patch still exits 1" || say no "a stale patch exits $rc, expected 1"

echo
echo "── 4. a self-failure is still 2, not collapsed into 1 ──"
mkdir -p "$TMP/bin"
# Break git the way real breakage behaves — exit 128 from a subcommand, like a missing
# repo or a broken install. A shim that exits 1 is indistinguishable from git apply's own
# "does not apply" code, which is the confusion this case is meant to catch.
printf '#!/bin/sh\necho "fatal: not a git repository" >&2\nexit 128\n' > "$TMP/bin/git"
chmod +x "$TMP/bin/git"
PATH="$TMP/bin:$PATH" SEATLOOM_WORKTREES="$TMP/wtroot" bash "$GATE" >"$TMP/self.out" 2>&1; rc=$?
if [ "$rc" -eq 2 ] && grep -q 'UNCHECKABLE' "$TMP/self.out"; then
  say ok "a broken git exits 2 UNCHECKABLE — not three stale patches"
else
  say no "a broken git exits $rc (expected 2 UNCHECKABLE): $(head -2 "$TMP/self.out" | tr '\\n' ' ')"
fi
# And the OLD shim form, exit 1, must ALSO be caught — git apply's own "does not apply"
# code is 1, so a shim exiting 1 was the exact case that produced three false alarms.
printf '#!/bin/sh\necho "git: shim exiting 1" >&2\nexit 1\n' > "$TMP/bin/git"
chmod +x "$TMP/bin/git"
PATH="$TMP/bin:$PATH" SEATLOOM_WORKTREES="$TMP/wtroot" bash "$GATE" >"$TMP/self2.out" 2>&1; rc2=$?
if [ "$rc2" -eq 2 ]; then
  say ok "a git exiting 1 (git apply's own code) is also caught as UNCHECKABLE"
else
  say no "a git exiting 1 gives exit $rc2, expected 2"
fi


echo
echo "── 5. nothing was left behind in a real worktree ──"
leftover=$(git -C "$WT_ROOT/nimbus-psr" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
[ "$leftover" = "0" ] && say ok "nimbus-psr is clean — the test used a scratch worktree" \
  || say no "nimbus-psr has $leftover uncommitted file(s) — the test touched a real tree"

echo
echo "passed=$PASS failed=$FAIL"
[ "$FAIL" -eq 0 ]
