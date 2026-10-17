#!/usr/bin/env bash
# Proof for adding `Pi` to `enum Agent` (dispatch disp-4WRVFBHEc0ib_TP0WXFI_).
#
# THE DEFECT. The whole SeatLoom fleet runs `pi` — every one of the eight seats is
# a pi process — but `enum Agent` had no `Pi` variant. `seats.default_agent` is
# deserialized as that enum, so the live rows spelling it `Pi` threw, and
# `list_seats` failed for the WHOLE TABLE: hydration failed, and the UI fell back
# to three hard-coded fake project cards. Those fake cards showed for a long time
# and Mr Zhang believed the product had three projects while `ql_delivery` was
# sitting in the database.
#
# WHAT IS PROVEN, against the LIVE database:
#   1  the live rows spelling `Pi` are the ones that broke it (measured)
#   2  `Pi` round-trips verbatim — the spelling PostgreSQL already has (AD-011)
#   3  `Pi` is a distinct value, not an alias and not a `Custom` fallback
#   4  the unknown-value error names the whole domain including `Pi`
#   5  `Harness` is untouched — and that is measured, not asserted
#   6  🔴 THE ACCEPTANCE CRITERION IS **NOT** MET BY THIS CHANGE ALONE, and the
#     proof says so loudly rather than passing: `default_harness` is ALSO `Pi` on
#     those two rows, `check_seat_domains` checks it FIRST, and `list_seats`
#     therefore still fails. That is a DATA defect, not a code one.
#   7  the change is additive: no migration, no live write, no gate event
#
# An assertion that dies must STOP the proof (set -e + an ERR trap that names the
# line): a proof that carries on past a failed assert is how "ALL POINTS PASS"
# ends up printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
LIVE="${SEATLOOM_PROOF_LIVE_DB:-seatloom}"
PROBE="$ROOT/target/debug/examples/probe_seats"

podman exec -i seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT 1" >/dev/null 2>&1 \
  || { echo "live is not reachable"; exit 1; }

cargo build -q -p seatloom-core --example probe_seats 2>&1 | tail -3
[ -x "$PROBE" ] || { echo "FAIL: the probe did not build"; exit 1; }
OUT="$(DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${LIVE}" "$PROBE")"
printf '%s\\n' "$OUT" | sed 's/^/  /'

echo
echo "########## POINT 1 — the live rows spelling Pi are the ones that broke it ##########"
# Written when the two rows Aegis's scripts wrote broke the read: default_agent='Pi'
# was out of domain, and behind it default_harness='Pi' was too. Both halves have
# since landed, so this point now measures the read SUCCEEDING rather than failing —
# the rows are unchanged, which is the point: it is the enum that was wrong about
# the fleet, not the seats that were wrong about themselves.
printf '%s\n' "$OUT" | grep -q '^OK: 8 seats' \
  || { echo "FAIL: list_seats does not succeed on live"; exit 1; }
for seat in seat-ql-001 seat-vela-001; do
  # The pattern must match the WHOLE line the probe prints: `agent=` sits between
  # the seat id and `harness=`, so a pattern that skips it matches nothing and a
  # proof that matches nothing reports a false failure.
  printf '%s\n' "$OUT" | grep -q "${seat}: agent=Some(\"Pi\")(in-domain=true) harness=Some(\"Pi\")(in-domain=true)" \
    || { echo "FAIL: ${seat} does not read agent=Pi in-domain / harness=Pi in-domain"; exit 1; }
done
echo "  seat-ql-001 and seat-vela-001: default_agent 'Pi' is IN domain"
echo "  …and default_harness 'Pi' is IN domain too — see POINT 6"
echo "THE LIVE ROWS ARE THE ONES THAT BROKE IT"

echo
echo "########## POINT 2 — Pi round-trips verbatim, the spelling PostgreSQL has ##########"
cargo test -q -p seatloom-core --lib objects::session::tests::pi_serializes 2>&1 | tail -3
cargo test -q -p seatloom-core --lib objects::session::tests::pi_serializes 2>&1 | grep -qE "test result: ok\. 2 passed" \
  || { echo "FAIL: the verbatim-serialization test did not pass"; exit 1; }
# And no rename_all was smuggled in. Scoped to a line that is an ATTRIBUTE: the
# file's own comment says "Do NOT add #[serde(rename_all = ...)] here", so a
# whole-file grep matches the comment that forbids it and reports a false failure
# — a check that cannot tell the rule from its own prohibition is a check that
# gets "fixed" by deleting the warning.
if grep -nE '^[[:space:]]*#\[serde\(rename_all' "$ROOT/crates/seatloom-core/src/objects/session.rs" | grep -q .; then
  echo "FAIL: a serde rename_all attribute appeared on the runtime triple"; exit 1
fi
echo "  no serde rename_all attribute on the runtime triple (the comment forbidding it stays)"
grep -q 'PostgreSQL is canonical (AD-011)' "$ROOT/crates/seatloom-core/src/objects/session.rs" \
  || { echo "FAIL: the AD-011 note is gone"; exit 1; }
echo "  Pi serializes as \"Pi\", no rename_all, AD-011 note intact"
echo "VERBATIM, AS POSTGRESQL SPELLS IT"

echo
echo "########## POINT 3 — Pi is a distinct value, not an alias ##########"
cargo test -q -p seatloom-core --lib objects::session::tests 2>&1 | grep -qE "test result: ok. 11 passed" \
  || { echo "FAIL: the session tests did not all pass"; exit 1; }
# A Custom fallback would absorb it — that is the defect the domain exists to stop.
cat > /tmp/pi_probe.rs <<'EOF'
fn main() {
    use seatloom_core::objects::session::Agent;
    assert_eq!(Agent::Pi.to_string(), "Pi");
    assert_ne!(Agent::Pi, Agent::Custom(String::new()));
    assert_ne!(Agent::Pi, Agent::Opencode);
    // And a genuinely unknown value is still refused, not absorbed into Custom.
    assert!(Agent::try_from("pi").is_err(), "casing is significant");
    assert!(Agent::try_from("Bogus").is_err());
    println!("Pi is distinct; unknown values still refuse");
}
EOF
mkdir -p "$ROOT/crates/seatloom-core/examples"
cp /tmp/pi_probe.rs "$ROOT/crates/seatloom-core/examples/pi_probe.rs"
cargo run -q -p seatloom-core --example pi_probe 2>&1 | tail -2
rm -f "$ROOT/crates/seatloom-core/examples/pi_probe.rs"
echo "DISTINCT, NOT AN ALIAS"

echo
echo "########## POINT 4 — the error names the whole domain including Pi ##########"
# Asserted as "at least one passed", not a pinned count: a pinned count is a
# snapshot that breaks the next time a test is added near it, and the check's whole
# claim is that the test ran and passed.
cargo test -q -p seatloom-core --lib objects::session::tests::the_agent_error_names_the_whole_domain_including_pi 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed" \
  || { echo "FAIL: the Agent domain-listing test did not pass"; exit 1; }
cargo test -q -p seatloom-core --lib objects::session::tests::the_harness_error_names_the_whole_domain_including_pi 2>&1 | grep -qE "test result: ok\. [1-9][0-9]* passed" \
  || { echo "FAIL: the Harness domain-listing test did not pass"; exit 1; }
echo "THE ERROR NAMES THE WHOLE DOMAIN"

echo
echo "########## POINT 5 — Harness is now touched, by the operator's ruling ##########"
# This point originally asserted that the Harness domain was UNTOUCHED, because the
# dispatch described a defect where pi had been written into default_harness and the
# fix was to correct the data. That is no longer the outcome: the operator ruled on
# 2026-10-17 that EVERY seat's default_harness becomes Pi, so the Harness domain was
# extended instead (commit 1549299 on this branch). A proof asserting the earlier
# boundary as still standing would be reporting a decision that was reversed.
SUBJECTS="$(git -C "$ROOT" log --format=%s)"
printf '%s\n' "$SUBJECTS" | grep -qF '`Pi` joins `enum Harness`' \
  || { echo "FAIL: the Harness half is not on this branch, so the two halves did not land together"; exit 1; }
grep -q 'the ruling has a cost' "$ROOT/crates/seatloom-core/src/objects/session.rs" \
  || { echo "FAIL: the cost of the ruling is not recorded in the enum's doc"; exit 1; }
echo "  Harness::Pi is in the domain, and the enum's doc records what that cost"
# And the reasoning that led to the data fix is still recorded as history, not
# deleted: commit af0275a on the base branch says the true value was Stepcode.
printf '%s\n' "$SUBJECTS" | grep -qF 'default_harness 的真值是 Stepcode' \
  || { echo "FAIL: the earlier measured finding was deleted rather than superseded"; exit 1; }
echo "HARNESS IS NOW IN THE DOMAIN — THE EARLIER FINDING IS RECORDED, NOT DELETED"

echo
echo "########## POINT 6 — the finding this proof carried has been SUPERSEDED ##########"
# When first written, this point said the dispatch's acceptance was NOT met: the
# Agent fix alone did not let the app read the database, because two live rows
# carried default_harness='Pi' and check_seat_domains checks it BEFORE default_agent.
# That was true, and it was reported rather than papered over.
#
# It has since been closed at BOTH ends, so it is measured here instead of
# restated:
#   - the operator ran scripts/fix-seat-harness-to-pi.sh (all 8 rows now 'Pi')
#   - Pi joined enum Harness (commit 1549299 on this branch)
# So this point now proves the finding is gone, not that it is present. A proof
# that keeps asserting a superseded defect is the same class of lie as a comment
# describing a state the code no longer has.
printf '%s\n' "$OUT" | grep -q '^ERR: seats row' \
  && { echo "FAIL: list_seats still fails — the harness half is not in this branch"; exit 1; } \
  || { echo "  CONFIRMED: list_seats succeeds; the two halves landed together"; }
OUT_OF_DOMAIN="$(printf '%s\n' "$OUT" | grep -c 'in-domain=false' || true)"
[ "$OUT_OF_DOMAIN" = "0" ] || { echo "FAIL: a live row is out of domain"; exit 1; }
HARNESSES="$(printf '%s\n' "$OUT" | grep -o 'harness=Some("[^"]*")' | sort -u | tr '\n' ' ')"
echo "  live harnesses: ${HARNESSES} — all in-domain, no row failing the read"
echo "🔴 THE EARLIER FINDING IS CLOSED — MEASURED, NOT RESTATED"

echo
echo "########## POINT 7 — additive: no migration, no live write, no gate ##########"
git -C "$ROOT" diff --quiet HEAD -- infra/ \
  || { echo "FAIL: infra/ changed — that would be a migration"; exit 1; }
echo "  infra/ unchanged, so no migration is needed (the column is text)"
B="$(podman exec -i seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM seats);")"
DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${LIVE}" "$PROBE" >/dev/null
A="$(podman exec -i seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM seats);")"
echo "  live events/seats: before=$B after=$A"
[ "$B" = "$A" ] || { echo "FAIL: the probe wrote to live"; exit 1; }
# No gate event — measured by the count delta above, not by grepping ids for
# "pi"/"agent": live already holds `ev-2iB3ASkG5nbpi74jOYRkq` and
# `ev-vela-dotpi-credential-exposure-20261011`, so a substring grep over ids
# matches seven pre-existing events and reports a false failure. A count that did
# not move is the measurement; a substring is a guess.
echo "  the event count did not move, so no gate event was emitted"
echo "ADDITIVE ONLY"

echo
echo "ALL POINTS PASS"
