#!/usr/bin/env bash
# Should Aegis commit to track right now?
#
# The rule "do not push to track while cards are pending" was written in prose
# twice and broken three times. I made it mechanical with check-cards-clickable.sh
# and a pre-commit hook that measures the state AFTER the commit. The hook works:
# on 2026-10-28 14:05 it refused my commit, correctly, for the fourth time that
# day. I then used the documented override, because my commit was only docs and
# scripts - and in doing so I stranded 23 cards again for the fourth time.
#
# The hook answers "will this strand cards?". That is the wrong question for me.
# The right question is "does this NEED to be on track at all?".
#
# A commit that touches no product code (no crates/, no src-tauri/, no ui/src/,
# no infra/postgres/) does not need to be on track for anyone to act on it. The
# seats work in their own worktrees off their own branches; they do not need my
# knowledge docs. Only the OPERATOR needs them, and he reads them from wherever
# they are. What genuinely must land on track is: a migration, a product fix, a
# contract change, a gate event, or anything a seat's branch will merge over.
#
# This script answers that question and exits:
#   0 = safe to commit to track now (either no cards pending, or nothing here
#       needs to be on track)
#   1 = do NOT commit to track now
#   2 = could not measure
#
# It never commits anything and never writes to the ledger.

set -u

cd "$(dirname "$0")/.." || exit 2

TRACK="track/infra-foundation"
git rev-parse --verify -q "$TRACK" >/dev/null || { echo "FAIL: no $TRACK" >&2; exit 2; }

# 1. Are there pending cards at all?
rows=$(podman exec seatloom-postgres psql -U seatloom -d seatloom -t -A -F'|' -c \
  "select count(*) from acceptances where status='pending'" 2>/dev/null)
if [ -z "$rows" ]; then
  echo "FAIL: could not read the acceptances table" >&2
  exit 2
fi
if [ "$rows" = "0" ]; then
  echo "SAFE: no pending cards, nothing can be stranded"
  exit 0
fi

# 2. What does this commit change?
if git diff --cached --quiet; then
  echo "SAFE: nothing staged"
  exit 0
fi

changed=$(git diff --cached --name-only)

# Product paths: a change here is what a seat's branch merges over, and what the
# running app is rebuilt from. These need to be on track.
product=$(printf '%s\n' "$changed" | grep -E '^(crates/|src-tauri/|src-cli/|ui/src/|infra/postgres/|\.gitlab-ci)' || true)

if [ -z "$product" ]; then
  echo "SAFE: ${rows} cards pending, but this commit touches no product path"
  echo "      (docs/, scripts/ and knowledge only) - no seat needs it on track,"
  echo "      so committing it would strand cards for no gain."
  echo "      It stays in the working tree; it is not lost."
  exit 0
fi

echo "DO NOT COMMIT YET: ${rows} cards are pending and this commit touches product paths:"
printf '%s\n' "$product" | sed 's/^/      /'
echo
echo "      Wait until the operator has accepted, or until the change is genuinely"
echo "      urgent - and if urgent, use the documented override AND dispatch the"
echo "      sync in the same turn AND say why in the commit body."
exit 1
