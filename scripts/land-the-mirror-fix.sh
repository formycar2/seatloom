#!/usr/bin/env bash
# After the operator accepts the mirror-fix card, put the fix into the app he is
# actually running, and PROVE it is in there.
#
# WHY ONE COMMAND. The mirror fix (`9a67ea5`, `pipe_guard.rs`) has been verified
# and carded since 10-27 20:26. The running binary is still 10-27 14:03. The gap
# is not knowledge - it is that "accept the card" and "the fix is running" are
# six manual steps apart, and six manual steps is where the fix stops living.
#
# WHAT THIS DOES, in order, each step able to fail:
#   1. confirm the fix's commit is an ancestor of track (the card was accepted)
#   2. build the frontend + the tauri bundle
#   3. verify the bundle CONTAINS the frontend, by decompressing its blobs and
#      content-matching ui/dist. Never by listing Resources, never by grepping the
#      executable - both are wrong in both directions (see verify-tauri-bundle.sh)
#   4. verify the built binary contains the fix's symbol
#   5. install over /Applications/SeatLoom.app, keeping the old one
#   6. report the new binary's mtime and pid, so the next heartbeat can compare
#
# It does NOT restart the app: that would close the operator's windows. It prints
# the command to do it.
#
# Usage:
#   scripts/land-the-mirror-fix.sh            # dry run: measure, build nothing
#   scripts/land-the-mirror-fix.sh --apply    # build, verify, install

set -u

cd "$(dirname "$0")/.." || exit 2

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

TRACK="track/infra-foundation"
FIX_COMMIT="9a67ea5"
APP="/Applications/SeatLoom.app"
BIN="$APP/Contents/MacOS/seatloom-tauri"

echo "== 1. is the fix on track yet? =="
if ! git rev-parse --verify -q "$TRACK" >/dev/null; then
  echo "FAIL: branch $TRACK does not exist here" >&2
  exit 2
fi
if git merge-base --is-ancestor "$FIX_COMMIT" "$TRACK" 2>/dev/null; then
  echo "  yes - $FIX_COMMIT is an ancestor of $TRACK"
else
  echo "  NO - the fix is not on track yet."
  echo "  That means the card acc-disp-jZg89gwy8YzVOOjdAsxTL has not been accepted."
  echo "  Nothing to install. The operator's Accept is the gate, not this script."
  exit 1
fi

echo "== 2. what is running now =="
if [ -x "$BIN" ]; then
  echo "  binary mtime: $(stat -f '%Sm' -t '%Y-%m-%d %H:%M:%S' "$BIN")"
else
  echo "  FAIL: $BIN is missing" >&2
  exit 2
fi
pid=$(pgrep -f "$BIN" 2>/dev/null | head -1)
if [ -n "$pid" ]; then
  echo "  running pid: $pid (started $(ps -p "$pid" -o lstart= | sed 's/^ *//'))"
else
  echo "  not currently running"
fi

if [ "$APPLY" != "1" ]; then
  echo
  echo "dry run only. Re-run with --apply to build, verify and install."
  exit 0
fi

echo "== 3. build the frontend =="
( cd ui && pnpm build ) || { echo "FAIL: frontend build failed" >&2; exit 1; }

echo "== 4. build the tauri bundle =="
( cd src-tauri && pnpm tauri build ) || { echo "FAIL: tauri build failed" >&2; exit 1; }

echo "== 5. prove the bundle contains the frontend (content match, not listing) =="
bash scripts/verify-tauri-bundle.sh || {
  echo "FAIL: the built bundle does not provably contain the frontend." >&2
  echo "Do NOT install it. A bundle that passes a directory listing but fails a" >&2
  echo "content match is exactly the false-deployment case recorded twice." >&2
  exit 1
}

echo "== 6. prove the binary contains the fix =="
NEW=$(find target/release/bundle/macos -name 'seatloom-tauri' -type f 2>/dev/null | head -1)
if [ -z "$NEW" ]; then
  echo "FAIL: could not find the built seatloom-tauri under target/release/bundle/macos" >&2
  exit 1
fi
echo "  built binary: $NEW ($(stat -f '%Sm' -t '%Y-%m-%d %H:%M:%S' "$NEW"))"
# strings is unreliable on a stripped release binary, so this is a SOFT check:
# it can only fail-open. Say so rather than let a green line mean proof.
if strings "$NEW" 2>/dev/null | grep -q 'pipe_guard\|mirror-lost\|mirrorLost'; then
  echo "  soft check: found the fix's symbol (this can only fail-open; the real"
  echo "             proof is the behaviour test, which this script does not run)"
else
  echo "  soft check: symbol not found by strings - NOT proof of absence."
  echo "             Release builds strip and mangle names. Verify by behaviour:"
  echo "             run tmux pipe-pane from outside and watch the app stop showing"
  echo "             that seat with a re-attach banner."
fi

echo "== 7. install (keeping the old one) =="
cp "$APP/Contents/MacOS/seatloom-tauri" \
   "$APP/Contents/MacOS/seatloom-tauri.bak-$(date +%Y%m%d-%H%M%S)" || {
  echo "FAIL: could not back up the running binary" >&2
  exit 1
}
cp "$NEW" "$BIN" || { echo "FAIL: install failed" >&2; exit 1; }
echo "  installed. new mtime: $(stat -f '%Sm' -t '%Y-%m-%d %H:%M:%S' "$BIN")"

echo
echo "== done =="
echo "The fix is in the binary but NOT in the running process - a running process"
echo "keeps the image it started with. To pick it up:"
echo
echo "  1. quit SeatLoom (Cmd+Q)"
echo "  2. open -a /Applications/SeatLoom.app"
echo "  3. re-attach the live seats (接入实时镜像) for any seat showing the button"
echo
echo "Then the behaviour test: have another process run tmux pipe-pane on a seat's"
echo "pane. The app must stop showing that seat and offer a re-attach button."
