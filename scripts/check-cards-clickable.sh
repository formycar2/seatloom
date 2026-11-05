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

# ============================================================================
# 0. THE GATE'S OWN WORKING DIRECTORY, NOT OURS.
#
# The line above cd's into the main checkout, which is where a human runs this
# script. The acceptance gate does NOT run there. It runs inside the app process,
# and resolves the repository with resolve_repo_root(): SEATLOOM_REPO_ROOT if
# set, otherwise the process cwd. `open -a` gives the app cwd=/, and / is not a
# git repository.
#
# On 2026-10-29 11:28 that made all 7 pending cards render as 验收被拒 / 分支未
# 包含验收基准, while every branch was 0 commits behind. The gate was not wrong
# about the branches - it could not see a repository at all. git failed, Err
# became BranchNotSynced, and 7 cards went grey.
#
# This script said OK: all 7 pending cards are clickable at 11:26, one minute
# before he screenshotted it. It could never have caught this, because it cd's
# itself into a valid repo. A check that repairs its own working directory is
# blind to the failure it exists to catch.
#
# So: measure the gate's predicate in the gate's working directory FIRST. If that
# fails, say so and exit - the per-branch numbers below are meaningless.
# ----------------------------------------------------------------------------
APP_PID="$(pgrep -f 'SeatLoom.app' 2>/dev/null | head -1)"
if [ -n "$APP_PID" ]; then
  APP_CWD="$(lsof -p "$APP_PID" 2>/dev/null | awk '$4=="cwd"{print $NF; exit}')"
else
  APP_CWD=""
fi
GATE_REPO_ROOT="${SEATLOOM_REPO_ROOT:-}"

# What the gate would actually resolve to: the env var if the app has it,
# otherwise the app's cwd.
if [ -z "$GATE_REPO_ROOT" ] && [ -n "$APP_PID" ]; then
  GATE_REPO_ROOT="$(ps -E -p "$APP_PID" 2>/dev/null | tr ' ' '\n' \
                     | grep '^SEATLOOM_REPO_ROOT=' | head -1 | cut -d= -f2-)"
fi
if [ -z "$GATE_REPO_ROOT" ]; then
  GATE_REPO_ROOT="${APP_CWD:-/}"
fi

if ! git -C "$GATE_REPO_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  echo "FAIL: the acceptance gate resolves its repository to '$GATE_REPO_ROOT', which is NOT a git repository."
  echo
  echo "  Every card will read 验收被拒 / 分支未包含验收基准 no matter how synced the branches are."
  echo "  app pid      : ${APP_PID:-<not running>}"
  echo "  app cwd      : ${APP_CWD:-<unknown>}"
  echo "  SEATLOOM_REPO_ROOT in the app : ${SEATLOOM_REPO_ROOT:-<unset>}"
  echo
  echo "  The gate runs: git -C \$repo_root merge-base --is-ancestor <base> <branch>"
  echo "  In a non-repo that fails, branch_contains returns Err, and Err becomes BranchNotSynced."
  echo
  echo "  Fix: relaunch the app through scripts/launch-seatloom.sh, which sets and VERIFIES"
  echo "       SEATLOOM_REPO_ROOT. Do not launch it from the Dock or Finder - neither passes it."
  exit 1
fi
echo "gate repo root: $GATE_REPO_ROOT (is a git repository - the gate can see branches)"
echo

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
behind_note=""
declare_bad=""

while IFS='|' read -r branch base card; do
  [ -n "$branch" ] || continue
  total=$((total + 1))

  if ! git rev-parse --verify -q "$branch" >/dev/null; then
    bad=$((bad + 1))
    declare_bad="${declare_bad}${branch}|${card}|branch missing in this checkout\n"
    continue
  fi

  # Measured in the gate's repository, not in this script's working directory.
  # They are the same refs (worktrees share refs/heads) but the question "can the
  # gate see a repository at all" is not, and that question came first.
  #
  # 🔴 2026-11-05: the predicate CHANGED, by the operator's ruling.
  # dec-clickable-criterion-level-with-base, resolved 03:43:40 UTC by
  # human:supervisor: clickability asks only whether the merge would DROP the
  # mainline's commits or DELETE files tracked on the base; it no longer requires
  # the branch to be LEVEL with the mainline. The level-with-base requirement
  # STAYS at request_acceptance (dispatch.rs:1815 BranchNotSynced) and does not
  # block clicking.
  #
  # What used to be here was the level check itself — merge-base --is-ancestor —
  # which refused every branch that was merely behind. That is exactly the
  # ground he ruled invalid, and it cost him five hours a day (Lyra's re-priced
  # figure: windows of 61.5 / 58.9 / 61.1 / 119.4 minutes).
  #
  # 🔴 A gate that stops refusing without starting to check anything is WORSE
  # than one that refuses on the wrong ground. So this does not delete the
  # check; it replaces it with the two questions he named. Both are already
  # used by the real gate: branch_contains (dispatch.rs:1810) and
  # merge_would_delete (dispatch.rs:1822, exemptions applied).
  #
  # Both directions are proven by scripts/test-clickability-criterion.sh:
  # a branch that is merely BEHIND must NOT be refused, and a branch whose
  # merge DELETES must be refused. The second protects the merge; the first is
  # what he paid five hours a day for.
  #
  # Being behind is now only a NOTE. It is not a refusal, because a behind
  # branch merges cleanly: the merge keeps the mainline's later commits.
  behind=$(git rev-list --count "$branch".."$base" 2>/dev/null)
  [ "$behind" = "0" ] || behind_note="behind ${behind}"

  # (1) Would the merge DELETE files the base still tracks? This is the one
  # that protects merged work, so it is asked first.
  merge_tree=$(git -C "$GATE_REPO_ROOT" merge-tree --write-tree "$base" "$branch" 2>/dev/null | head -1)
  if [ -z "$merge_tree" ]; then
    # Not a conflict message we can read. Say so rather than guessing: an
    # unreadable preview must not become "nothing would be deleted".
    bad=$((bad + 1))
    declare_bad="${declare_bad}${branch}|${card}|merge preview unreadable\n"
  else
    deleted=$(git -C "$GATE_REPO_ROOT" diff --name-only --diff-filter=D "$base" "$merge_tree" 2>/dev/null)
    if [ -n "$deleted" ]; then
      # Subtract the operator-ruled exemptions, the same list the gate uses.
      # The remainder is what the defence actually protects.
      undeleted=""
      while IFS= read -r path; do
        [ -n "$path" ] || continue
        case "$path" in
          ui/src/app-v2/AppV2.tsx) ;;   # DELETION_EXEMPTIONS, ruled by the operator
          *) undeleted="${undeleted}${path} " ;;
        esac
      done <<< "$deleted"
      if [ -n "$undeleted" ]; then
        bad=$((bad + 1))
        declare_bad="${declare_bad}${branch}|${card}|merge would delete: ${undeleted}\n"
      fi
    fi
  fi

  # (2) There is deliberately NO second predicate here, and the first version of
  # this script got this wrong twice.
  #
  # The ruling says clickability asks whether the merge "would drop the
  # mainline's commits or delete files". The deletion half is above. The
  # drop-commits half is NOT a merge property: a git merge keeps both sides'
  # history, so no ordinary merge can drop a mainline commit. The only thing it
  # can map onto is "the branch does not contain the base"
  # (`merge-base --is-ancestor base branch` = false) — and that is exactly
  # `request_acceptance`'s BranchNotSynced at dispatch.rs:1815, which the
  # operator EXPLICITLY KEPT at card-open time and EXPLICITLY SAID does not
  # block clicking. Re-adding it here would re-implement the check he just
  # moved, which is the same defect wearing different clothes.
  #
  # So the click-time gate asks the one question that is genuinely about the
  # merge: would it delete files the base still tracks. The containment
  # question is answered where he ruled it belongs.
done <<< "$rows"

if [ "$bad" -gt 0 ]; then
  echo "FAIL: ${bad} of ${total} pending cards are UNCLICKABLE - the operator's Accept is blocked"
  echo
  echo "which branch / which card / why"
  printf "$declare_bad" | sort -u
  echo
  if [ -n "${behind_note:-}" ]; then
    echo "note (not a refusal): some branches are merely behind — ${behind_note}"
    echo "  being behind is no longer a ground to refuse. dec-clickable-criterion-level-with-base"
    echo "  resolved that on 2026-11-05 03:43:40 UTC. They may merge to tidy up; they do not have to."
    echo
  fi
  echo "the affected seats must fix the merge itself, not the calendar:"
  echo "  a branch whose merge would delete base-tracked files must stop deleting them"
  echo "  a branch that lacks the base must merge track into their packet branch, then push"
  echo
  echo "Do NOT merge their branches into track yourself: that is the operator's Accept."
  exit 1
fi

[ "$QUIET" = "1" ] || echo "OK: all ${total} pending cards are clickable"
exit 0
