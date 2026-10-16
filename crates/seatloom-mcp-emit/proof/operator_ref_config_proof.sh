#!/usr/bin/env bash
# Proof for the operator identity's configuration channels (dispatch
# disp-SfHOAaw2hl0EfvAZdAxFv).
#
# THE DEFECT THIS PROOF EXISTS FOR. Mr. Zhang clicked "accept" on 2026-10-16 and
# got `SEATLOOM_OPERATOR_REF is not set`. The refusal itself was correct — an
# acceptance decision is a human act and the identity must never be invented on
# the spot — but it read from the ENVIRONMENT ONLY, and an app launched from the
# macOS Launchpad, Spotlight or Finder has no environment at all. So a correct
# guard had become a wall he could never get past: every click failed, and nothing
# in the product could satisfy it from a GUI launch.
#
# It had been fixed once already, on 2026-10-12 (2339223: a single-line file at
# ~/.seatloom/operator, env > file > refuse). That fix was then LOST on 2026-10-13
# by 1e7b254, the intent integration onto track, which took acceptance_cmds.rs
# wholesale from packet/nimbus/gate-vs-announcement — a branch that forked from
# track at b6751a1, BEFORE the fix. So the integration silently reverted a
# track-side fix. This proof measures the channel, so a second silent revert is
# caught by a gate rather than by the operator.
#
# WHAT IS PROVEN, against the real filesystem and the real file:
#   1  the file at ~/.seatloom/operator resolves to the identity already there
#   2  a blank file is NOT an identity (the hard line, in its smallest form)
#   3  two stated identities is a refusal, not a pick
#   4  neither channel states one -> a refusal that names the file, its absolute
#      path, the exact command, and that no restart is needed
#   5  the env var still works as the per-launch channel
#   6  the precedence is file > env, and both are explicit configuration
#   7  there is NO third channel: no default is ever substituted
#   8  the unit tests pin every branch above, and the read writes nothing
#
# An assertion that dies must STOP the proof (set -e + an ERR trap that names the
# line): a proof that carries on past a failed assert is how "ALL POINTS PASS"
# ends up printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
PROBE="$ROOT/target/debug/examples/operator_ref_probe"
REAL_FILE="$HOME/.seatloom/operator"
BACKUP="$(mktemp -t seatloom-operator-backup)"
HAD_FILE=0
[ -f "$REAL_FILE" ] && { cp "$REAL_FILE" "$BACKUP"; HAD_FILE=1; }
restore() {
  if [ "$HAD_FILE" = "1" ]; then cp "$BACKUP" "$REAL_FILE"; else rm -f "$REAL_FILE"; fi
  rm -f "$BACKUP"
}
trap 'restore; echo "PROOF FAILED (line $LINENO)" >&2' ERR
trap 'restore' EXIT

echo "probe: ${PROBE#$ROOT/}"
echo "the real file: $REAL_FILE"
echo

# ---------------------------------------------------------------------------
# Build the probe. It is the same `read_operator_ref()` the Tauri command calls,
# driven from a process whose environment is exactly what we set — which is the
# only honest way to measure the Launchpad case (an app with no environment).
# ---------------------------------------------------------------------------
cargo build -q -p seatloom-core --example operator_ref_probe
[ -x "$PROBE" ] || { echo "FAIL: the probe did not build"; exit 1; }
echo "probe built"

# `probe` with a scrubbed environment: no SEATLOOM_OPERATOR_REF, so the only
# channel that can answer is the file. That is the Launchpad case.
probe() { env -u SEATLOOM_OPERATOR_REF "$PROBE"; }

echo
echo "########## POINT 1 — the real file resolves ##########"
[ -f "$REAL_FILE" ] || { echo "FAIL: $REAL_FILE is not present, so the happy path cannot be measured"; exit 1; }
echo "  file content: $(tr -d '\n' < "$REAL_FILE")"
probe | tee /tmp/op_proof_1.txt
grep -q '^RESOLVED value=human:supervisor source=File(' /tmp/op_proof_1.txt \
  || { echo "FAIL: the real file did not resolve"; exit 1; }
grep -q "source=File(\"$REAL_FILE\")" /tmp/op_proof_1.txt \
  || { echo "FAIL: it resolved from somewhere other than the file"; exit 1; }
echo "  the identity already on this machine is read, from the file, with no environment"
echo "REAL FILE RESOLVES"

echo
echo "########## POINT 2 — a blank file is NOT an identity ##########"
: > "$REAL_FILE"
OUT="$(probe)"
echo "$OUT" | head -1
echo "$OUT" | grep -q '^REFUSED' || { echo "FAIL: a blank file was accepted as an identity"; exit 1; }
echo "$OUT" | grep -q 'exists but is empty' \
  || { echo "FAIL: the refusal does not say the file is empty"; exit 1; }
echo "A BLANK FILE IS NOT AN IDENTITY"

echo
echo "########## POINT 3 — two stated identities is a refusal, not a pick ##########"
printf 'human:supervisor\nhuman:someone-else\n' > "$REAL_FILE"
OUT="$(probe)"
echo "$OUT" | head -1
echo "$OUT" | grep -q '^REFUSED' || { echo "FAIL: two identities were accepted"; exit 1; }
echo "$OUT" | grep -q 'one token' || { echo "FAIL: the refusal does not say a ref is one token"; exit 1; }
echo "$OUT" | grep -q 'human:supervisor' && echo "$OUT" | grep -q 'human:someone-else' \
  || { echo "FAIL: the refusal does not show both lines"; exit 1; }
echo "TWO IDENTITIES IS A REFUSAL, NOT A PICK"

echo
echo "########## POINT 4 — neither channel: the refusal is actionable ##########"
rm -f "$REAL_FILE"
OUT="$(probe)"
echo "$OUT" | head -1
echo "$OUT" | grep -q '^REFUSED' || { echo "FAIL: no identity was refused"; exit 1; }
MSG="$(probe | sed -n '/--- message as the operator sees it ---/,$p' | tail -n +2)"
printf '%s\n' "$MSG" | sed 's/^/    /'
# It names the file by its ABSOLUTE path — not a ~ the operator must expand.
printf '%s' "$MSG" | grep -qF "$REAL_FILE" \
  || { echo "FAIL: the refusal does not name the absolute path"; exit 1; }
printf '%s' "$MSG" | grep -q "echo 'human:supervisor' > $REAL_FILE" \
  || { echo "FAIL: the refusal does not give the exact command"; exit 1; }
printf '%s' "$MSG" | grep -q 'SEATLOOM_OPERATOR_REF' \
  || { echo "FAIL: the refusal does not name the env alternative"; exit 1; }
printf '%s' "$MSG" | grep -q 'no restart needed' \
  || { echo "FAIL: the refusal does not say the change needs no restart"; exit 1; }
printf '%s' "$MSG" | grep -q 'will not invent one for you' \
  || { echo "FAIL: the refusal does not state that nothing is invented"; exit 1; }
# And it must NOT claim $HOME is missing when it is set — the first cut of this
# module did exactly that, and the test caught it.
printf '%s' "$MSG" | grep -q 'cannot find your home directory' \
  && { echo "FAIL: the refusal claims HOME is unset when it is set"; exit 1; } || true
echo "THE REFUSAL NAMES THE PATH, THE COMMAND, AND THE NO-RESTART FACT"

echo
echo "########## POINT 5 — the environment is still a working channel ##########"
rm -f "$REAL_FILE"
OUT="$(SEATLOOM_OPERATOR_REF=human:supervisor "$PROBE")"
echo "$OUT"
echo "$OUT" | grep -q '^RESOLVED value=human:supervisor source=Env' \
  || { echo "FAIL: the env channel does not work"; exit 1; }
# A blank env var is not an identity either.
OUT="$(SEATLOOM_OPERATOR_REF='   ' "$PROBE")"
echo "$OUT" | head -1
echo "$OUT" | grep -q '^REFUSED' || { echo "FAIL: a blank env var was accepted"; exit 1; }
echo "THE ENV CHANNEL WORKS, AND A BLANK ONE DOES NOT"

echo
echo "########## POINT 6 — precedence is file > env, both explicit ##########"
printf 'human:supervisor\n' > "$REAL_FILE"
OUT="$(SEATLOOM_OPERATOR_REF=human:someone-else "$PROBE")"
echo "$OUT"
echo "$OUT" | grep -q '^RESOLVED value=human:supervisor source=File(' \
  || { echo "FAIL: the env var overrode the file; the documented precedence is file > env"; exit 1; }
echo "FILE > ENV, AND BOTH ARE EXPLICIT CONFIGURATION"

echo
echo "########## POINT 7 — there is NO third channel ##########"
# With both channels silent the only possible outcome is a refusal. If a default
# ever appears, this point dies.
rm -f "$REAL_FILE"
for _ in 1 2 3; do
  OUT="$(probe)"
  echo "$OUT" | grep -q '^REFUSED' || { echo "FAIL: a default was substituted"; exit 1; }
done
echo "$OUT" | grep -qi 'default' \
  && { echo "FAIL: the refusal mentions a default"; exit 1; } || true
# The source of truth is one function, so there is nowhere else to look.
grep -q 'pub fn read_operator_ref()' "$ROOT/crates/seatloom-core/src/config.rs" \
  || { echo "FAIL: read_operator_ref moved"; exit 1; }
N="$(grep -c 'SEATLOOM_OPERATOR_REF' "$ROOT/crates/seatloom-core/src/config.rs")"
echo "  SEATLOOM_OPERATOR_REF appears $N times in config.rs — one constant, one read"
echo "NO DEFAULT, NO THIRD CHANNEL"

echo
echo "########## POINT 8 — the unit tests pin every branch, and the read writes nothing ##########"
cargo test -q -p seatloom-core --lib config:: -- --test-threads=1 2>&1 | tail -3
BEFORE="$(ls -la "$REAL_FILE" 2>/dev/null | awk '{print $5}' || echo absent)"
probe >/dev/null
AFTER="$(ls -la "$REAL_FILE" 2>/dev/null | awk '{print $5}' || echo absent)"
echo "  file size before=${BEFORE} after=${AFTER} (expect EQUAL — the read never writes)"
[ "$BEFORE" = "$AFTER" ] || { echo "FAIL: the read modified the file"; exit 1; }
# And the acceptance commands hold no second implementation: one call site pair,
# both going through the core function.
grep -q 'seatloom_core::config::read_operator_ref()' "$ROOT/src-tauri/src/commands/acceptance_cmds.rs" \
  || { echo "FAIL: the Tauri command no longer calls the core reader"; exit 1; }
if grep -qE 'std::env::var\(OPERATOR_REF_ENV\)' "$ROOT/src-tauri/src/commands/acceptance_cmds.rs"; then
  echo "FAIL: a second implementation reappeared in acceptance_cmds.rs"; exit 1
fi
echo "ONE IMPLEMENTATION, AND THE READ WRITES NOTHING"

echo
echo "ALL POINTS PASS"
