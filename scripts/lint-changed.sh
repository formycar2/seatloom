#!/usr/bin/env bash
# lint-changed.sh — ratchet eslint + prettier for the frontend.
#
# Runs eslint and prettier ONLY on ui/src TypeScript files changed relative to a
# base ref. The existing tree has 143 eslint errors / 87 unformatted files that
# predate the standard; gating the whole tree would block every MR. New/changed
# files must be clean. tsc --noEmit is run separately as a FULL gate (it already
# passes tree-wide).
#
# Usage: lint-changed.sh [BASE_REF]   (BASE defaults to CI MR base, else origin/main)
# Needs: git, node/pnpm with the ui toolchain installed. Run from repo root.
# Exit 0 = clean or nothing changed; 1 = lint/format violations.

set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

BASE="${1:-${CI_MERGE_REQUEST_DIFF_BASE_SHA:-${CI_DEFAULT_BRANCH:+origin/$CI_DEFAULT_BRANCH}}}"
BASE="${BASE:-origin/main}"
git rev-parse --verify --quiet "$BASE" >/dev/null || BASE="$(git hash-object -t tree /dev/null)"

# Changed ui/src TS files (committed vs base + unstaged + untracked), path
# relative to ui/ so eslint/prettier (run in ui/) resolve them. Use a directory
# pathspec + extension filter: the glob `ui/src/**/*.ts` misses files directly
# under ui/src/ (git's `**/` needs an extra slash).
CHANGED="$( {
  git diff --name-only --diff-filter=ACMR "$BASE" HEAD -- ui/src 2>/dev/null
  git diff --name-only --diff-filter=ACMR -- ui/src 2>/dev/null
  git ls-files --others --exclude-standard -- ui/src 2>/dev/null
} | grep -E '\.(ts|tsx)$' | sort -u | sed 's#^ui/##' )"

if [ -z "$CHANGED" ]; then
  echo "lint-changed: no changed ui/src TS files vs $BASE — nothing to lint."
  exit 0
fi

echo "lint-changed: checking $(printf '%s' "$CHANGED" | grep -c .) changed file(s) vs $BASE"
cd ui

FAIL=0
# shellcheck disable=SC2086
printf '%s\n' "$CHANGED" | xargs pnpm exec eslint || FAIL=1
# shellcheck disable=SC2086
printf '%s\n' "$CHANGED" | xargs pnpm exec prettier --check || FAIL=1

[ "$FAIL" -eq 0 ] && echo "lint-changed: all changed frontend files clean." \
                   || echo "lint-changed: violations above. Fix with: cd ui && pnpm lint:fix && pnpm format"
exit "$FAIL"
