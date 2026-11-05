#!/usr/bin/env bash
# Does the clickability gate now ask what the operator's ruling says to ask?
#
# THE RULING (dec-clickable-criterion-level-with-base, resolved 2026-11-05
# 03:43:40 UTC by human:supervisor): clickability asks only whether the merge
# would DROP the mainline's commits or DELETE files tracked on the base; it no
# longer requires the branch to be LEVEL with the mainline. The level-with-base
# requirement STAYS at request_acceptance (dispatch.rs:1815) and does not block
# clicking.
#
# WHY THIS TEST EXISTS. Before the ruling, the gate's predicate was
# `merge-base --is-ancestor <base> <branch>` — the level check itself — which
# refused every branch that was merely behind. That cost the operator five hours
# a day (Lyra's re-priced figure: windows of 61.5 / 58.9 / 61.1 / 119.4 min).
#
# THE DEFECT THIS CLOSES. A gate that stops refusing without starting to check
# anything is WORSE than one that refuses on the wrong ground. Deleting the
# level check would have made the gate exit 0 for every card, including one
# whose merge deletes merged work. So the replacement must actually check.
#
# 🔴 THE DIRECTION A TEST WILL NOT NATURALLY COVER. The real repository has ZERO
# pending cards, so `check-cards-clickable.sh` prints "no pending cards" and
# exits 0 no matter what its predicate is. A test that only ran the script
# against the live ledger would pass on a gate that had been gutted. So this
# test builds its own branches and drives the gate's predicate directly, with a
# stubbed row source.
#
# Usage: scripts/test-clickability-criterion.sh
# Exit 0 = both directions proven. Non-zero = the gate does not do what the
# ruling says, and the output names which direction failed.

set -u

cd "$(dirname "$0")/.." || exit 2

BASE=track/infra-foundation

PASS=0
FAIL=0
SCRATCH=""

ok()  { PASS=$((PASS + 1)); echo "  ✓ $1"; }
bad() { FAIL=$((FAIL + 1)); echo "  ✗ $1"; }

cleanup() {
  git checkout -q "$BASE" 2>/dev/null
  for b in probe/mainline-advanced probe/behind-only probe/deletes-base-file probe/behind-work probe/deleting; do
    git branch -D "$b" >/dev/null 2>&1
  done
  git checkout -q -- README.md 2>/dev/null
  [ -n "$SCRATCH" ] && rm -rf "$SCRATCH"
  rm -rf "$PWD/.scratch/clickability-mutations" 2>/dev/null
  return 0
}
trap cleanup EXIT

# Start from a known state: no leftover probe branch, and on the base.
cleanup
git checkout -q "$BASE" || { echo "FAIL: cannot check out $BASE"; exit 2; }
HERE=$(git branch --show-current)
[ "$HERE" = "$BASE" ] || { echo "FAIL: expected to be on $BASE, on $HERE"; exit 2; }
ok "starting on $BASE with no leftover probe branch"
BASE_SHA=$(git rev-parse "$BASE")

GATE=scripts/check-cards-clickable.sh
[ -f "$GATE" ] || { echo "FAIL: $GATE not found"; exit 2; }

echo
echo "── the clickability criterion matches the operator's ruling ──"

# What must be GONE is the refusal keyed on being behind. Note that
# `--is-ancestor` cannot be the predicate at all — a merely-behind branch also
# fails it, so it refuses behind and diverged identically. The ruling's question
# is "would the merge drop the mainline's commits", and the predicate that
# answers it is `merge-base base branch == base`.
# The ruling names two questions. Only one of them is a property of the MERGE,
# so only that one belongs here; the other lives at request_acceptance where he
# ruled it stays. A gate that answered both at click time would re-implement the
# check he just moved.
if grep -q 'merge-tree --write-tree' "$GATE"; then
  ok "the gate previews the merge, the one click-time question that is about the merge"
else
  bad "the gate does not preview the merge, so it cannot know what would be deleted"
fi
if grep -q 'DELETION_EXEMPTIONS\|ui/src/app-v2/AppV2.tsx' "$GATE"; then
  ok "the gate subtracts the operator-ruled deletion exemptions"
else
  bad "the gate ignores DELETION_EXEMPTIONS, so it would refuse a deletion he already ruled on"
fi
if grep -qE '^[[:space:]]*behind=.*git rev-list --count' "$GATE" && \
   grep -A1 'behind=$(git rev-list --count' "$GATE" | grep -q 'bad=$((bad + 1))'; then
  bad "the gate still refuses on being behind — the ground the ruling dropped"
else
  ok "the gate no longer refuses on being behind"
fi

echo
echo "── building scenario branches ──"

# 🔴 THE SCENARIO MUST BE BUILT IN THE RIGHT ORDER, and the first version of
# this test got it wrong. "Merely behind" means the branch CONTAINS the base's
# history and lacks only its LATER commits. If the delivery branch and the
# advanced mainline are both cut from the same tip, the branch does not contain
# the mainline at all — that is a refusal for a DIFFERENT reason, and it would
# make the "behind must be allowed" direction fail for the wrong reason. So: the
# mainline advances first, every delivery branch is cut from the ADVANCED
# mainline, and only then does the mainline move forward again.

git checkout -q -b probe/mainline-advanced "$BASE" 2>/dev/null || git checkout -q probe/mainline-advanced || exit 2
git commit -q --allow-empty -m "probe: mainline advanced" || exit 2
MAINLINE=probe/mainline-advanced

git checkout -q -b probe/behind-only "$MAINLINE" 2>/dev/null || git checkout -q probe/behind-only || exit 2
git commit -q --allow-empty -m "probe: behind work, deletes nothing" || exit 2

git checkout -q "$MAINLINE" || exit 2
git checkout -q -b probe/deletes-base-file "$MAINLINE" 2>/dev/null || git checkout -q probe/deletes-base-file || exit 2
git rm -q README.md || exit 2
git commit -q -m "probe: deletes a base-tracked file" || exit 2

git checkout -q "$MAINLINE" || exit 2
git commit -q --allow-empty -m "probe: mainline advanced once more" || exit 2
git checkout -q "$BASE" || exit 2

# The delivery branches are cut from the ADVANCED mainline and then the mainline
# moves forward once more, so they are genuinely BEHIND: they do not contain the
# mainline's newest tip. That is correct and intended — it is the state the
# ruling says must be allowed. What must NOT be true is that they contain none
# of the mainline's history at all, which would be a real divergence.
if git merge-base --is-ancestor "$BASE_SHA" probe/behind-only 2>/dev/null; then
  ok "behind-only contains the base's history (behind, not diverged)"
else
  bad "behind-only does not contain the base's history — the scenario is wrong, not the gate"
fi
if git merge-base --is-ancestor "$BASE_SHA" probe/deletes-base-file 2>/dev/null; then
  ok "deletes-base-file contains the base's history (destructive, not diverged)"
else
  bad "deletes-base-file does not contain the base's history — the scenario is wrong, not the gate"
fi
if git merge-base --is-ancestor probe/mainline-advanced probe/behind-only 2>/dev/null; then
  bad "behind-only CONTAINS the mainline tip — it is not behind, so the scenario proves nothing"
else
  ok "behind-only really is behind the mainline tip"
fi

BEHIND_COUNT=$(git rev-list --count probe/behind-only..probe/mainline-advanced)
echo "  probe/behind-only       behind ${BEHIND_COUNT}, deletes nothing"
echo "  probe/deletes-base-file  deletes README.md"

# Prove the scenarios are what they claim BEFORE reading any answer from the
# gate. An empty answer must never be interpreted as a fact.
A_DEL=$(git diff --name-only --diff-filter=D probe/mainline-advanced \
        "$(git merge-tree --write-tree probe/mainline-advanced probe/behind-only | head -1)" 2>/dev/null)
B_DEL=$(git diff --name-only --diff-filter=D probe/mainline-advanced \
        "$(git merge-tree --write-tree probe/mainline-advanced probe/deletes-base-file | head -1)" 2>/dev/null)

echo
echo "── scenario sanity (a scenario that is not what it claims proves nothing) ──"
if [ -z "$A_DEL" ]; then ok "behind-only's merge really deletes nothing"; else bad "behind-only deletes [${A_DEL}] - the scenario is wrong"; fi
if [ -n "$B_DEL" ]; then ok "deletes-base-file's merge really deletes [${B_DEL}]"; else bad "deletes-base-file deletes nothing - the scenario is wrong"; fi
if [ "$BEHIND_COUNT" -gt 0 ]; then ok "behind-only really is behind"; else bad "behind-only is not behind - the scenario is wrong"; fi

# ── Drive the gate ITSELF, with a stubbed row source ────────────────────────
# The gate reads its rows from podman/psql. We stub podman so the gate sees
# exactly the one scenario we want. Driving the gate itself — not a
# reimplementation of it — is what makes the mutation below meaningful.
# The mutated copies must live INSIDE the repository, because the gate's own
# second line is `cd "$(dirname "$0")/.."` — running a copy from /tmp makes it
# cd into /tmp, where there is no repository and every ref check fails. The
# first version of this test put them in a mktemp dir and every mutation
# reported exit 2 "branch track/infra-foundation does not exist", which is the
# test failing, not the gate.
SCRATCH=$(mktemp -d)
STUB="$SCRATCH/bin"
mkdir -p "$STUB"
WORKDIR="$PWD/.scratch/clickability-mutations"
mkdir -p "$WORKDIR"
cp "$GATE" "$WORKDIR/gate-original.sh"

serve_row() {  # serve_row <branch> <base>
  cat > "$STUB/podman" <<STUBEOF
#!/usr/bin/env bash
cat <<ROW
$1|$2|acc-probe-mutation
ROW
STUBEOF
  chmod +x "$STUB/podman"
}

run_gate() {  # run_gate <script> <branch> <base> ; sets GATE_OUT / GATE_RC
  serve_row "$2" "$3"
  GATE_OUT=$(PATH="$STUB:$PATH" SEATLOOM_REPO_ROOT="$(pwd)" bash "$1" 2>&1)
  GATE_RC=$?
}

echo
echo "── the gate's own answer per scenario ──"

run_gate "$GATE" probe/behind-only probe/mainline-advanced
A_OUT="$GATE_OUT"; A_RC="$GATE_RC"
echo "  behind-only       -> exit ${A_RC}  $(printf '%s' "$A_OUT" | grep -E 'FAIL|OK|no pending' | head -1)"

run_gate "$GATE" probe/deletes-base-file probe/mainline-advanced
B_OUT="$GATE_OUT"; B_RC="$GATE_RC"
echo "  deletes-base-file -> exit ${B_RC}  $(printf '%s' "$B_OUT" | grep -E 'FAIL|OK|no pending' | head -1)"

echo
echo "── the two directions the ruling demands ──"

# Direction 1: MERELY BEHIND must NOT be refused. This is what the operator
# paid five hours a day for, and no test would naturally cover it, because the
# live ledger has zero pending cards.
if [ "$A_RC" -eq 0 ]; then
  ok "a branch that is merely behind is NOT refused (exit 0)"
else
  bad "a branch that is merely behind is REFUSED (exit ${A_RC}) — the ground he ruled invalid"
  echo "      gate said: $(printf '%s' "$A_OUT" | grep -E 'behind|UNCLICKABLE' | head -2 | tr '\n' ' ')"
fi

# Direction 2: a merge that DELETES must be refused. This protects merged work,
# and is the reason the check was not simply deleted.
if [ "$B_RC" -ne 0 ] && printf '%s' "$B_OUT" | grep -q 'would delete'; then
  ok "a branch whose merge deletes a base-tracked file IS refused"
else
  bad "a branch whose merge deletes a base-tracked file is NOT refused — merged work is unprotected"
  echo "      gate said: $(printf '%s' "$B_OUT" | grep -E 'FAIL|OK|no pending' | head -1)"
fi

# ── Mutations: prove this test can fail ─────────────────────────────────────
# A test that cannot fail is worse than no test. The mutations are done with
# python3 rather than sed: the gate's code contains pipes, dollars and
# backslashes, and every sed delimiter collided with one of them. A mutation
# that silently fails to apply is worse than no mutation at all, because the
# test then reports a pass it did not earn.
echo
echo "── mutation: a gate that does not check the merge must fail this test ──"

mutate() {  # mutate <old-literal> <new-literal> <outfile>
  python3 - "$GATE" "$1" "$2" "$3" <<'PYEOF'
import sys
src, old, new, out = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
text = open(src, encoding='utf-8').read()
if old not in text:
    sys.stderr.write("MUTATION TARGET NOT FOUND\n")
    sys.exit(3)
open(out, 'w', encoding='utf-8').write(text.replace(old, new, 1))
PYEOF
}

OLD_PREVIEW='merge_tree=$(git -C "$GATE_REPO_ROOT" merge-tree --write-tree "$base" "$branch" 2>/dev/null | head -1)'
NEW_PREVIEW='merge_tree="$base"  # MUTATION: no preview, so nothing is ever deleted'

if mutate "$OLD_PREVIEW" "$NEW_PREVIEW" "$WORKDIR/gate-no-preview.sh"; then
  ok "the mutation applied (the gate no longer previews the merge)"
else
  bad "the mutation did not apply — this test is not testing anything"
fi

run_gate "$WORKDIR/gate-no-preview.sh" probe/deletes-base-file probe/mainline-advanced
# 🔴 THE ASSERTION'S DIRECTION MATTERS, and the first version had it backwards.
# A gutted gate that stops previewing MUST let the deleting branch through —
# that is what the mutation does. So exit 0 here is the PROOF that the preview
# is load-bearing, and it must be read as a pass. Asserting the opposite would
# have demanded that a gate with its check removed still refuses, which is a
# test that can only pass when the mutation fails to apply.
if [ "$GATE_RC" -eq 0 ]; then
  ok "with the preview removed the gate lets a deleting branch through — the preview is load-bearing"
else
  bad "with the preview removed the gate still refuses (exit ${GATE_RC}) — the preview may be dead code"
fi
echo "  gutted gate says: $(printf '%s' "$GATE_OUT" | grep -E 'FAIL|OK|no pending' | head -1)"

# Mutation 2: the level check restored as the refusal ground — the exact code
# the ruling removed. It must refuse the merely-behind scenario.
OLD_BEHIND='  behind=$(git rev-list --count "$branch".."$base" 2>/dev/null)'
NEW_BEHIND='  behind=$(git rev-list --count "$branch".."$base" 2>/dev/null); if [ "$behind" != "0" ]; then bad=$((bad + 1)); declare_bad="${declare_bad}${branch}|${card}|behind ${behind}\n"; fi'

if mutate "$OLD_BEHIND" "$NEW_BEHIND" "$WORKDIR/gate-level-restored.sh"; then
  ok "the level-check mutation applied (the pre-ruling ground is back)"
else
  bad "the level-check mutation did not apply — the test cannot tell the two criteria apart"
fi

run_gate "$WORKDIR/gate-level-restored.sh" probe/behind-only probe/mainline-advanced
if [ "$GATE_RC" -ne 0 ] && printf '%s' "$GATE_OUT" | grep -q 'behind'; then
  ok "the restored level check refuses a merely-behind branch — this test detects the ground he dropped"
else
  bad "the level check does NOT refuse a merely-behind branch (exit ${GATE_RC}) — the test is blind to the ruling"
fi
echo "  level-restored gate says: $(printf '%s' "$GATE_OUT" | grep -E 'FAIL|OK|no pending' | head -1)"

# ── The live state must stay honest ─────────────────────────────────────────
echo
echo "── the live gate still runs and still reports honestly ──"
LIVE_OUT=$(bash "$GATE" 2>&1)
LIVE_RC=$?
if [ "$LIVE_RC" -eq 0 ] && printf '%s' "$LIVE_OUT" | grep -q 'no pending cards'; then
  ok "with zero pending cards the gate says so and exits 0 — no false clean, no false alarm"
elif [ "$LIVE_RC" -eq 1 ]; then
  ok "with pending cards the gate reports them and exits 1"
else
  bad "the live gate returned an unexpected exit ${LIVE_RC}"
fi

echo
echo "──────────────────────────────────────────"
echo "  ${PASS} passed, ${FAIL} failed"
[ "$FAIL" -eq 0 ] || exit 1
echo "  Both directions proven: behind is allowed, deleting is refused."
