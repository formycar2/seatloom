#!/usr/bin/env bash
# Proof for adding `Pi` to `enum Harness` (dispatch disp-_4-4LQz9T_ienR34isCtD).
#
# THE RULING. Mr Zhang, 2026-10-17: "现在要求所有的 Harness 都改成 PI". He knew the
# cost and chose anyway — recorded in the enum's own doc comment and in the
# operator's script, not written up as a pure improvement.
#
# WHY BOTH HALVES. `seats.default_harness` is deserialized as `enum Harness`. Data
# alone leaves it throwing; code alone leaves six seats on `Stepcode` and the
# ruling unimplemented. The Agent half (1547219) is preserved and cherry-picked
# onto the clean branch this dispatch required.
#
# WHAT IS PROVEN:
#   1  the CURRENT live rows still read — no regression from adding the variant
#   2  the POST-SCRIPT state reads: every row's default_harness='Pi' is in-domain
#      and `list_seats` really succeeds (measured on a live-shaped scratch, since
#      the operator has not run the script against live yet)
#   3  `Pi` round-trips verbatim on Harness, no rename_all
#   4  `Pi` is a distinct value on Harness, not a `Custom` fallback
#   5  the error names the whole domain including `Pi`
#   6  the Agent half is preserved and untouched by this change
#   7  the cost is recorded in the source, not hidden
#   8  additive: no migration, no live write, no gate
#
# An assertion that dies must STOP the proof (set -e + an ERR trap that names the
# line): a proof that carries on past a failed assert is how "ALL POINTS PASS"
# ends up printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
LIVE="${SEATLOOM_PROOF_LIVE_DB:-seatloom}"
SCRATCH="${SEATLOOM_PROOF_DB:-seatloom_pi_harness}"
SRC="$ROOT/crates/seatloom-core/src/objects/session.rs"
PROBE="$ROOT/target/debug/examples/probe_seats"

[ "$SCRATCH" != "$LIVE" ] || { echo "REFUSING: scratch and live are the same database"; exit 1; }

echo "########## POINT 1 — the current live rows still read (no regression) ##########"
cargo build -q -p seatloom-core --example probe_seats 2>&1 | tail -3
[ -x "$PROBE" ] || { echo "FAIL: the probe did not build"; exit 1; }
OUT="$(DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${LIVE}" "$PROBE")"
printf '%s\\n' "$OUT" | sed 's/^/  /'
printf '%s\\n' "$OUT" | grep -q '^OK: 8 seats' \
  || { echo "FAIL: list_seats does not succeed on live as it stands"; exit 1; }
printf '%s\\n' "$OUT" | grep -c 'harness=Some("Stepcode")(in-domain=true)' | grep -q '^8$' \
  || { echo "FAIL: the eight live rows are not all reading with Stepcode in-domain"; exit 1; }
echo "  all 8 live rows read; every harness=Stepcode is in-domain"
echo "NO REGRESSION ON THE CURRENT DATA"

echo
echo "########## POINT 2 — the post-script state reads, really ##########"
# The operator has not run scripts/fix-seat-harness-to-pi.sh against live yet (the
# script itself says the order is: script -> this change verified and merged ->
# the app reads). So the post-script state is reproduced on a LIVE-SHAPED scratch.
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${SCRATCH}" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "CREATE DATABASE ${SCRATCH}" >/dev/null
apply() { podman exec -i seatloom-postgres psql -U seatloom -d "$SCRATCH" -v ON_ERROR_STOP=1 -q -f /dev/stdin >/dev/null; }
S="$ROOT/infra/postgres/schema"; D="$ROOT/infra/postgres/seed"
for f in 001_seatloom_core 002_document_authority 003_write_ingest_reconcile \
         004_operational_review_and_continuity 005_prompt_and_channel_action_authority 009_runtime_triple; do
  apply < "$S/${f}.sql" || { echo "pre-seed schema failed at $f"; exit 1; }
done
for f in 001_real_collaboration_baseline 002_document_seed \
         003_operational_review_and_continuity_seed 004_prompt_and_channel_action_seed; do
  apply < "$D/${f}.sql" || { echo "seed failed at $f"; exit 1; }
done
for f in 008_project_isolation 010_p0_schema_foundation 011_agent_history_archive \
         012_governed_dispatch 013_dispatch_states 014_acceptance_gate; do
  apply < "$S/${f}.sql" || { echo "post-seed schema failed at $f"; exit 1; }
done
# The script's effect, reproduced.
apply <<'SQL'
UPDATE seats SET default_harness = 'Pi';
SQL
echo "  scratch ${SCRATCH}: every seat's default_harness = 'Pi' (the script's effect)"
OUT2="$(DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${SCRATCH}" "$PROBE")"
printf '%s\\n' "$OUT2" | sed 's/^/  /'
printf '%s\\n' "$OUT2" | grep -q '^OK: ' \
  || { echo "FAIL: list_seats failed on the post-script state — the ruling cannot be implemented"; exit 1; }
# Every row must read with Pi in-domain on BOTH columns where applicable.
if printf '%s\\n' "$OUT2" | grep -q 'in-domain=false'; then
  echo "FAIL: some row is still out of domain on the post-script state"
  printf '%s\\n' "$OUT2" | grep 'in-domain=false' | sed 's/^/    /'
  exit 1
fi
echo "  list_seats really succeeds with every default_harness='Pi' in-domain"
echo "THE POST-SCRIPT STATE READS"

echo
echo "########## POINT 3 — Pi round-trips verbatim on Harness ##########"
cargo test -q -p seatloom-core --lib objects::session::tests::pi_serializes_verbatim_on_both_dimensions 2>&1 | grep -q "1 passed" \
  || { echo "FAIL: the both-dimensions verbatim test did not pass"; exit 1; }
cargo test -q -p seatloom-core --lib objects::session::tests::harness_domain_round_trips 2>&1 | grep -q "1 passed" \
  || { echo "FAIL: the harness round-trip did not pass"; exit 1; }
# No rename_all attribute — the file's own comment forbids it.
if grep -nE '^[[:space:]]*#\[serde\(rename_all' "$SRC" | grep -q .; then
  echo "FAIL: a serde rename_all attribute appeared on the runtime triple"; exit 1
fi
grep -q 'PostgreSQL is canonical (AD-011)' "$SRC" || { echo "FAIL: the AD-011 note is gone"; exit 1; }
echo "  Harness::Pi serializes as \"Pi\", no rename_all, AD-011 note intact"
echo "VERBATIM, AS POSTGRESQL SPELLS IT"

echo
echo "########## POINT 4 — Pi is distinct on Harness, not a Custom fallback ##########"
cat > /tmp/hpi.rs <<'EOF'
fn main() {
    use seatloom_core::objects::session::Harness;
    assert_eq!(Harness::Pi.to_string(), "Pi");
    assert_ne!(Harness::Pi, Harness::Custom(String::new()));
    assert_ne!(Harness::Pi, Harness::Stepcode);
    assert_ne!(Harness::Pi, Harness::Direct);
    // Unknown values still refuse, never absorbed into Custom.
    assert!(Harness::try_from("pi").is_err(), "casing is significant");
    assert!(Harness::try_from("Bogus").is_err());
    println!("Harness::Pi is distinct; unknown values still refuse");
}
EOF
mkdir -p "$ROOT/crates/seatloom-core/examples"
cp /tmp/hpi.rs "$ROOT/crates/seatloom-core/examples/hpi_probe.rs"
cargo run -q -p seatloom-core --example hpi_probe 2>&1 | tail -2
rm -f "$ROOT/crates/seatloom-core/examples/hpi_probe.rs"
echo "DISTINCT, NOT AN ALIAS"

echo
echo "########## POINT 5 — the error names the whole domain including Pi ##########"
cargo test -q -p seatloom-core --lib objects::session::tests::the_harness_error_names_the_whole_domain_including_pi 2>&1 | grep -q "1 passed" \
  || { echo "FAIL: the Harness domain-listing test did not pass"; exit 1; }
cargo test -q -p seatloom-core --lib objects::session::tests 2>&1 | grep -qE "test result: ok. 11 passed" \
  || { echo "FAIL: the session tests did not all pass"; exit 1; }
echo "THE ERROR NAMES THE WHOLE DOMAIN"

echo
echo "########## POINT 6 — the Agent half is preserved ##########"
# The dispatch requires both halves. The Agent half is 1547219, cherry-picked onto
# this clean branch; verify it is an ancestor and its markers are present.
# Capture first, then grep: `set -o pipefail` plus `grep -q` on a long stream is a
# FALSE failure — grep exits the moment it matches, git gets SIGPIPE and the
# pipeline reports non-zero even though the match was found. That cost me one
# debug cycle on this very point.
SUBJECTS="$(git -C "$ROOT" log --format=%s)"
printf '%s\n' "$SUBJECTS" | grep -qF '`Pi` joins `enum Agent`' \
  || { echo "FAIL: the Agent half is not in this branch's history"; exit 1; }
grep -q 'Pi,' "$SRC" || { echo "FAIL: the Agent Pi variant is gone"; exit 1; }
grep -q '"Pi" => Ok(Self::Pi)' "$SRC" || { echo "FAIL: the Agent FromStr arm is gone"; exit 1; }
# And exactly two Pi arms exist — one per dimension, not a duplicate.
PI_ARMS="$(grep -c '"Pi" => Ok(Self::Pi)' "$SRC")"
echo "  FromStr Pi arms: ${PI_ARMS} (expect 2 — Agent and Harness)"
[ "$PI_ARMS" -eq 2 ] || { echo "FAIL: expected one Pi arm per dimension"; exit 1; }
echo "  the Agent half is present and unchanged"
echo "BOTH HALVES ARE HERE"

echo
echo "########## POINT 7 — the cost is recorded in the source, not hidden ##########"
# The operator ruled against the measured fact and chose anyway. The enum's own doc
# comment must say so, so a future reader does not "fix" it back.
grep -q 'the ruling has a cost' "$SRC" || { echo "FAIL: the cost note is missing from the Harness doc"; exit 1; }
grep -q 'default_model' "$SRC" || { echo "FAIL: the surviving proxy clue is not named"; exit 1; }
grep -q 'vocabulary boundary is his' "$SRC" || { echo "FAIL: the ruling's ownership is not recorded"; exit 1; }
echo "  the Harness doc comment names the cost, the surviving clue, and whose call it was"
echo "THE COST IS RECORDED"

echo
echo "########## POINT 8 — additive: no migration, no live write, no gate ##########"
git -C "$ROOT" diff --quiet "${SEATLOOM_PROOF_BASE:-track/infra-foundation}" -- infra/ \
  || { echo "FAIL: infra/ changed — that would be a migration"; exit 1; }
echo "  infra/ unchanged, so no migration is needed (the columns are text)"
B="$(podman exec -i seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM seats);")"
DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${LIVE}" "$PROBE" >/dev/null
A="$(podman exec -i seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM seats);")"
echo "  live events/seats: before=$B after=$A"
[ "$B" = "$A" ] || { echo "FAIL: the probe wrote to live"; exit 1; }
echo "  the event count did not move, so no gate event was emitted"
echo "ADDITIVE ONLY"

echo
echo "ALL POINTS PASS"
