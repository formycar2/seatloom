#!/usr/bin/env bash
# Isolated row-source tests; never writes a live ledger or the checkout index.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
mkdir "$T/bin"
printf '#!/bin/bash\nif [ "${FAIL_SOURCE:-0}" = 1 ]; then exit 9; fi\nprintf "%%s\\n" "$FIXTURE"\n' > "$T/bin/podman"
chmod +x "$T/bin/podman"
# Point the checker at an isolated declarations file. Without this the real
# .gitlab/unclaimed-declared.txt is read, and a fixture whose rows match none of them
# makes every real declaration look stale - so the quiet-mode assertion below fails on
# repo state rather than on the behaviour under test.
export UNCLAIMED_DECLARED="$T/declared.txt"
: > "$UNCLAIMED_DECLARED"
run() {
  local expected="$1"; shift
  if PATH="$T/bin:$PATH" bash "$ROOT/scripts/check-unclaimed-work.sh" "$@" > "$T/out" 2>&1; then rc=0; else rc=$?; fi
  [ "$rc" -eq "$expected" ] || { printf 'FAIL expected %s got %s\n' "$expected" "$rc"; exit 1; }
}
export FIXTURE='excluded|0'
run 0
grep -q 'Excluded explicit verification.*: 0' "$T/out"
run 0 --quiet
[ ! -s "$T/out" ]
export FIXTURE='excluded|4
candidate|disp-product-with-acceptance|seat-nimbus|unknown|2026-11-06'
run 1
grep -q 'disp-product-with-acceptance' "$T/out"
grep -q 'REVIEW REQUIRED' "$T/out"
export FAIL_SOURCE=1
run 2
unset FAIL_SOURCE
export FIXTURE=''
run 2
# The SQL must not filter acceptance key presence, and must explicitly retain
# unknown kinds. Stubbed rows above verify rendering/error behavior, not SQL.
python3 - "$ROOT/scripts/check-unclaimed-work.sh" <<'PY'
import sys
s=open(sys.argv[1]).read()
assert "? 'acceptance'" not in s
assert "coalesce(d.payload->>'kind','')<>'verification'" in s
assert 'REPEATABLE READ READ ONLY' in s
PY
printf 'PASS: candidate visibility, exclusions, quiet mode, failed/empty source; SQL policy static assertions (not a live SQL fixture test).\n'
