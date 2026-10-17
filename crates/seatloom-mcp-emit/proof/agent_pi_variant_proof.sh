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
# The two rows Aegis's scripts wrote, and the read they broke.
printf '%s\\n' "$OUT" | grep -q 'ERR: seats row "seat-ql-001": column default_harness' \
  || { echo "FAIL: the measured failure is not the one the dispatch described"; exit 1; }
for seat in seat-ql-001 seat-vela-001; do
  # The pattern must match the WHOLE line the probe prints: `agent=` sits between
  # the seat id and `harness=`, so a pattern that skips it matches nothing and a
  # proof that matches nothing reports a false failure.
  printf '%s\n' "$OUT" | grep -q "${seat}: agent=Some(\"Pi\")(in-domain=true) harness=Some(\"Pi\")(in-domain=false)" \
    || { echo "FAIL: ${seat} does not read agent=Pi in-domain / harness=Pi out-of-domain"; exit 1; }
done
echo "  seat-ql-001 and seat-vela-001: default_agent 'Pi' is now IN domain"
echo "  …and default_harness 'Pi' is still OUT of domain — see POINT 6"
echo "THE LIVE ROWS ARE THE ONES THAT BROKE IT"

echo
echo "########## POINT 2 — Pi round-trips verbatim, the spelling PostgreSQL has ##########"
cargo test -q -p seatloom-core --lib objects::session::tests::pi_serializes 2>&1 | tail -3
cargo test -q -p seatloom-core --lib objects::session::tests::pi_serializes 2>&1 | grep -q "1 passed" \
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
cargo test -q -p seatloom-core --lib objects::session::tests 2>&1 | grep -qE "test result: ok. 9 passed" \
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
cargo test -q -p seatloom-core --lib objects::session::tests::the_agent_error_names_the_whole_domain_including_pi 2>&1 | grep -q "1 passed" \
  || { echo "FAIL: the domain-listing test did not pass"; exit 1; }
cargo test -q -p seatloom-core --lib db::repositories::tests::rendered_error_names_table_row_column_and_value 2>&1 | grep -q "1 passed" \
  || { echo "FAIL: the pinned operator-facing message was not updated"; exit 1; }
echo "THE ERROR NAMES THE WHOLE DOMAIN"

echo
echo "########## POINT 5 — Harness is untouched ##########"
git -C "$ROOT" diff HEAD -- crates/seatloom-core/src/objects/session.rs | grep -E "^[-+].*(Harness|Stepcode|Direct)" \
  && { echo "FAIL: the Harness domain was touched"; exit 1; } || true
echo "  no line of the Harness domain changed"
# And the dispatch's reasoning for that boundary is recorded as still true: the
# eight seats DO run under stepcode, and the six correct rows say so.
printf '%s\\n' "$OUT" | grep -c 'harness=Some("Stepcode")(in-domain=true)' \
  | grep -q '^6$' || { echo "FAIL: the six correct harness rows are not as measured"; exit 1; }
echo "  six seats carry default_harness=Stepcode and are in-domain — the boundary holds"
echo "HARNESS IS UNTOUCHED"

echo
echo "########## POINT 6 — 🔴 THE ACCEPTANCE CRITERION IS NOT MET BY THIS ALONE ##########"
# The dispatch's acceptance: "应用重新能读到数据库 —— 项目页出现 39 个真项目".
# Measured: list_seats STILL fails, because default_harness is also 'Pi' and
# check_seat_domains checks it BEFORE default_agent.
printf '%s\\n' "$OUT" | grep -q '^ERR: seats row' \
  && { echo "  CONFIRMED: list_seats still fails after the Agent fix"; } \
  || { echo "FAIL: expected list_seats to still fail — the harness defect would be gone"; exit 1; }
podman exec -i seatloom-postgres psql -U seatloom -d "$LIVE" -tAc \
  "SELECT count(*) FROM seats WHERE default_harness NOT IN ('Direct','Stepcode') AND default_harness !~ '^Custom';" \
  | grep -q '^2$' || { echo "FAIL: expected 2 rows with an out-of-domain harness"; exit 1; }
echo "  2 live rows carry default_harness='Pi', which is out of the Harness domain"
echo "  This is a DATA defect, not a code one: pi is an AGENT, not a harness, and"
echo "  the six correct rows say Stepcode — which is the harness the seats run under."
echo "  The one-line-per-row fix is:  UPDATE seats SET default_harness='Stepcode'"
echo "    WHERE default_harness='Pi';   -- touches live, so it is NOT done here."
echo "🔴 ACCEPTANCE NOT MET — REPORTED, NOT PAPERED OVER"

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
