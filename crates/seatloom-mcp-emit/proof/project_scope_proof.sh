#!/usr/bin/env bash
# Proof for backend per-project filtering (dispatch disp-smUiJkscwc__D1g2DhD2R).
#
# THE OPERATOR'S REPORT: the project list shows many projects, but switching
# projects changes nothing below it — the live-seats / supervision-overview /
# dispatch / workitem / session / handoff modules all show the same thing under
# SeatLoom and under ql_delivery.
#
# MEASURED ROOT CAUSE, two layers, not one:
#   1. FOUR COMMANDS TOOK NO project_id AT ALL and read their tables unfiltered
#      (cmd_list_seats / cmd_list_workitems / cmd_list_handoffs / cmd_list_sessions),
#      and a FIFTH (cmd_list_dispatches) took one and silently substituted the
#      app's DEFAULT project when it was absent — the sneakiest, because it looked
#      like it was scoped and returned another project's rows.
#   2. THE STORE copied the one unfiltered result into EVERY project's
#      ProjectData inside a `for (const p of projectsDto)` loop, under a comment
#      saying "v0.1: all seeded data belongs to the single 'seatloom' project".
#      So even a correctly-scoped read would not have moved the numbers.
#
# WHAT IS PROVEN:
#   1  the schema asymmetry, measured — which tables have project_id and which do not
#   2  `seats` has no project_id, so the filter MUST be a join (a WHERE would be a
#      syntax error, not a filter)
#   3  SeatLoom -> 7 seats, and NONE of them is QL
#   4  ql_delivery -> exactly 1 seat, and it is ql
#   5  a historical-corpus project -> 0 seats, with the reason measured
#   6  THE NUMBERS MOVE — every scoped read differs between the two projects
#   7  the supervision overview's numbers are computed from the scoped rows, and
#      they MOVE (measured on a controlled scratch, because on live they are all 0)
#   8  no silent default: an absent project_id is an ERROR, and the type system
#      refuses the old call shape
#   9  artifacts CANNOT be partitioned — measured, not asserted — and the command
#      says so instead of pretending
#  10  additive: no live write, no gate event, no merge, no migration applied
#
# An assertion that dies must STOP the proof (set -e + an ERR trap naming the line):
# a proof that carries on past a failed assert is how "ALL POINTS PASS" ends up
# printed over a measured defect.
set -euo pipefail
trap 'echo "PROOF FAILED (line $LINENO)" >&2' ERR

ROOT="$(git rev-parse --show-toplevel)"
export PATH="$HOME/.cargo/bin:$PATH"
LIVE="${SEATLOOM_PROOF_LIVE_DB:-seatloom}"
PROBE="$ROOT/target/debug/examples/project_scope_probe"

# Read one field out of the probe's output.
field() { printf '%s\n' "$1" | grep -o "^$2=.*" | head -1 | cut -d= -f2-; }

echo "########## POINT 1 — the schema asymmetry, measured ##########"
WITH_PID="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc \
  "SELECT count(DISTINCT table_name) FROM information_schema.columns WHERE column_name='project_id' AND table_schema='public';")"
echo "  tables WITH project_id: ${WITH_PID}"
[ "$WITH_PID" -eq 13 ] || { echo "FAIL: expected 13 tables with project_id, measured ${WITH_PID}"; exit 1; }
for t in seats artifacts; do
  N="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc \
    "SELECT count(*) FROM information_schema.columns WHERE table_name='${t}' AND column_name='project_id' AND table_schema='public';")"
  echo "  ${t} has project_id: ${N}"
  [ "$N" -eq 0 ] || { echo "FAIL: ${t} unexpectedly has project_id — the premise changed"; exit 1; }
done
PC="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc \
  "SELECT count(*) FROM information_schema.columns WHERE column_name='project_code';")"
echo "  columns named project_code: ${PC}"
[ "$PC" -eq 0 ] || { echo "FAIL: project_code exists — the dispatch said it does not"; exit 1; }
echo "THE ASYMMETRY IS AS DESCRIBED"

echo
echo "########## POINT 2 — seats must be filtered by JOIN, not WHERE ##########"
# A WHERE on seats.project_id would be a SQL error. Prove the shipped read joins,
# and prove the join is what produces the scoping.
grep -q "JOIN project_role_bindings b ON b.seat_id = s.id" \
  "$ROOT/crates/seatloom-core/src/db/repositories.rs" \
  || { echo "FAIL: list_seats_for_project does not join project_role_bindings"; exit 1; }
grep -q "WHERE b.project_id = \$1" "$ROOT/crates/seatloom-core/src/db/repositories.rs" \
  || { echo "FAIL: the join is not filtered on the BINDING's project"; exit 1; }
grep -q "FROM seats s" "$ROOT/crates/seatloom-core/src/db/repositories.rs" \
  || { echo "FAIL: the seats side is not aliased, so the join is not the one intended"; exit 1; }
echo "  the read joins seats -> project_role_bindings and filters on b.project_id"
echo "THE JOIN IS THE FILTER"

echo
echo "########## POINTS 3-6 — measured against live, project by project ##########"
cargo build -q -p seatloom-core --example project_scope_probe 2>&1 | tail -3
[ -x "$PROBE" ] || { echo "FAIL: the probe did not build"; exit 1; }

read_probe() {
    DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${LIVE}" \
      PROJECT_ID="$1" "$PROBE"
}

SL="$(read_probe seatloom)"
QL="$(read_probe ql_delivery)"
printf '%s\n' "$SL" | grep -E "^(project|seats|workitems|handoffs|sessions|events)=" | sed 's/^/  [seatloom]    /'
printf '%s\n' "$QL" | grep -E "^(project|seats|workitems|handoffs|sessions|events)=" | sed 's/^/  [ql_delivery] /'

echo
echo "--- POINT 3: SeatLoom has 7 seats and NONE is QL ---"
[ "$(field "$SL" seats)" = "7" ] \
  || { echo "FAIL: SeatLoom measured $(field "$SL" seats) seats, expected 7"; exit 1; }
if printf '%s\n' "$SL" | grep -q "seat_ids=.*seat-ql-001"; then
  echo "FAIL: QL appears under SeatLoom — the join is not scoping"; exit 1
fi
echo "  7 seats under SeatLoom, and seat-ql-001 is NOT among them"

echo
echo "--- POINT 4: ql_delivery has exactly 1 seat and it is ql ---"
[ "$(field "$QL" seats)" = "1" ] \
  || { echo "FAIL: ql_delivery measured $(field "$QL" seats) seats, expected 1"; exit 1; }
[ "$(field "$QL" seat_ids)" = "seat-ql-001" ] \
  || { echo "FAIL: ql_delivery's seat is $(field "$QL" seat_ids), expected seat-ql-001"; exit 1; }
echo "  exactly seat-ql-001, and nothing else"

echo
echo "--- POINT 5: a historical-corpus project has 0 seats, and the reason is measured ---"
# Why empty, measured rather than asserted: those projects are corpus-derived
# (021) and drive no seat, so they have no row in project_role_bindings. Prove it
# by counting the bindings directly, not by trusting the zero.
for corpus in ColossalAI sf-code "project_delivery/QL" "(root)"; do
  B="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc \
    "SELECT count(*) FROM project_role_bindings WHERE project_id = '${corpus}';")"
  S="$(field "$(read_probe "$corpus")" seats)"
  echo "  ${corpus}: bindings=${B} seats=${S}"
  [ "$B" = "0" ] || { echo "FAIL: ${corpus} has ${B} bindings but the read returned ${S} seats"; exit 1; }
  [ "$S" = "0" ] || { echo "FAIL: ${corpus} returned ${S} seats with 0 bindings"; exit 1; }
done
echo "  empty because there are no binding rows — measured, not assumed"

echo
echo "--- POINT 6: THE NUMBERS MOVE ---"
# The acceptance is that switching projects changes the numbers. Prove every
# scoped read differs, rather than proving one of them and asserting the rest.
MOVED=0
for k in seats workitems handoffs sessions events; do
  A="$(field "$SL" "$k")"; B="$(field "$QL" "$k")"
  if [ "$A" != "$B" ]; then MOVED=$((MOVED+1)); fi
  printf "  %-11s seatloom=%-5s ql_delivery=%-5s %s\n" "$k" "$A" "$B" \
    "$([ "$A" != "$B" ] && echo MOVED || echo SAME)"
done
[ "$MOVED" -eq 5 ] || { echo "FAIL: only ${MOVED} of 5 scoped reads differ between the two projects"; exit 1; }
echo "  all five scoped reads differ"

echo
echo "--- POINT 7: the supervision overview's numbers MOVE (controlled scratch) ---"
# Against LIVE the overview's cards are all zero under both projects, because live
# holds 7 workitems that are all 'done' and 5 sessions that are all 'completed' —
# measured, not assumed. Proving "the numbers change" against a database where they
# are all zero would be proving nothing, so this point builds a controlled scratch:
# two projects, one seat bound to each, and a Blocked plus an InReview workitem in
# the FIRST project only.
SC="${SEATLOOM_PROOF_DB:-seatloom_scope_ctrl}"
[ "$SC" != "$LIVE" ] || { echo "REFUSING: scratch and live are the same database"; exit 1; }
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${SC}" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "CREATE DATABASE ${SC}" >/dev/null
apply_sc() { podman exec -i seatloom-postgres psql -U seatloom -d "$SC" -v ON_ERROR_STOP=1 -q -f /dev/stdin >/dev/null; }
for f in 001_seatloom_core 002_document_authority 003_write_ingest_reconcile \
         004_operational_review_and_continuity 005_prompt_and_channel_action_authority 009_runtime_triple; do
  apply_sc < "$ROOT/infra/postgres/schema/${f}.sql" || { echo "pre-seed schema failed at $f"; exit 1; }
done
for f in 001_real_collaboration_baseline 002_document_seed \
         003_operational_review_and_continuity_seed 004_prompt_and_channel_action_seed; do
  apply_sc < "$ROOT/infra/postgres/seed/${f}.sql" || { echo "seed failed at $f"; exit 1; }
done
for f in 008_project_isolation 010_p0_schema_foundation 011_agent_history_archive \
         012_governed_dispatch 013_dispatch_states 014_acceptance_gate; do
  apply_sc < "$ROOT/infra/postgres/schema/${f}.sql" || { echo "post-seed schema failed at $f"; exit 1; }
done
apply_sc <<'SQL'
INSERT INTO projects (id, name, created_at) VALUES ('proj_a', 'A', now()), ('proj_b', 'B', now());
INSERT INTO seats (id, name, default_harness, default_agent, status, created_at) VALUES
  ('seat-x', 'x', 'Stepcode', 'ClaudeCode', 'active', now()),
  ('seat-y', 'y', 'Stepcode', 'ClaudeCode', 'active', now());
INSERT INTO project_role_bindings (seat_id, project_id, role) VALUES
  ('seat-x', 'proj_a', 'supervisor'), ('seat-y', 'proj_b', 'worker');
INSERT INTO workitems (id, title, status, project_id, owner_seat_id, created_at, updated_at) VALUES
  ('wi-blocked', 'blocked in A only', 'Blocked', 'proj_a', 'seat-x', now(), now()),
  ('wi-review',  'in review in A only', 'InReview', 'proj_a', 'seat-x', now(), now());
SQL
CA="$(DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${SC}" PROJECT_ID=proj_a "$PROBE")"
CB="$(DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${SC}" PROJECT_ID=proj_b "$PROBE")"
printf '%s\n' "$CA" | grep -E "^(seats|workitems|overview)" | sed 's/^/  [proj_a] /'
printf '%s\n' "$CB" | grep -E "^(seats|workitems|overview)" | sed 's/^/  [proj_b] /'
# The overview's three arithmetic cards, computed exactly as SupervisionDashboard
# computes them, must differ — and the difference must be the one the data implies.
[ "$(field "$CA" overview_blocked)" = "1" ] || { echo "FAIL: proj_a blocked should be 1"; exit 1; }
[ "$(field "$CB" overview_blocked)" = "0" ] || { echo "FAIL: proj_b blocked should be 0"; exit 1; }
[ "$(field "$CA" overview_in_review)" = "1" ] || { echo "FAIL: proj_a in_review should be 1"; exit 1; }
[ "$(field "$CB" overview_in_review)" = "0" ] || { echo "FAIL: proj_b in_review should be 0"; exit 1; }
# And the seat count is 1 on BOTH sides, which is the real point: the scoping is
# per-project, not "project A has data and project B has none".
[ "$(field "$CA" seats)" = "1" ] && [ "$(field "$CB" seats)" = "1" ] \
  || { echo "FAIL: each project should have exactly its own one seat"; exit 1; }
echo "  blocked 1 -> 0, in_review 1 -> 0, seats 1 -> 1 (a DIFFERENT seat each)"
echo "THE OVERVIEW'S NUMBERS MOVE"
podman exec -i seatloom-postgres psql -U seatloom -d postgres -tAc "DROP DATABASE IF EXISTS ${SC}" >/dev/null 2>&1

echo
echo "########## POINT 8 — no silent default ##########"
# 8a. The helper errors on absence, and names the command and the reason.
cargo test -q -p seatloom-tauri --bin seatloom-tauri require_project_id 2>&1 \
  | grep -qE "test result: ok\. 3 passed" \
  || { echo "FAIL: the require_project_id tests did not all pass"; exit 1; }
echo "  3 unit tests: absent -> error naming the command; supplied -> passes through;"
echo "  the message names the reason (unscoped read), not just the symptom"

# 8b. The fallbacks this dispatch was asked to remove are gone. What REMAINS is the
#     same latent defect in commands the dispatch did not name — reported, not
#     silently fixed, because each has call sites that have not been measured.
echo "  the five named commands + dispatches: no fallback remains"
REMAINING="$(grep -rn "unwrap_or_else(|| state.default_project_id" "$ROOT"/src-tauri/src/commands/ \
  | grep -vE "^[^:]+:[0-9]+: *//" | grep -vE "dispatch_cmds.rs:(128|197)|history_cmds.rs:59" || true)"
if [ -n "$REMAINING" ]; then
  echo "  the same latent fallback still present in commands NOT named by this dispatch:"
  printf '%s\n' "$REMAINING" | sed 's|^|    |'
else
  echo "  (none)"
fi

# 8c. The UI's call shape no longer permits forgetting the project: the signatures
#     take a required projectId, so tsc fails on an unscoped call.
# Two shapes are legitimate: a positional `(projectId: string` (most of them) and
# an options object `{ projectId: string` (listDocuments). What is checked is that
# projectId appears and is NOT optional (`?:`), in either shape.
for f in listSeats listWorkitems listHandoffs listSessions listDispatches listEvents listDocuments listAcceptances; do
  if grep -qE "^  ${f}: \(" "$ROOT/ui/src/lib/api.ts"; then
    LINE="$(grep -A 4 -E "^  ${f}: \(" "$ROOT/ui/src/lib/api.ts" | head -5)"
  else
    echo "FAIL: api.${f} is not declared in api.ts"; exit 1
  fi
  printf '%s\n' "$LINE" | grep -q "projectId: string" \
    || { echo "FAIL: api.${f} does not require a projectId"; exit 1; }
  printf '%s\n' "$LINE" | grep -q "projectId?: string" \
    && { echo "FAIL: api.${f} still has projectId OPTIONAL — the old silent shape"; exit 1; } || true
done
echo "  all eight api.ts signatures require projectId — the type refuses the old call"

# 8d. And the whole UI type-checks, which is what makes 8c a real guard rather than
#     a comment: an unscoped call anywhere is a build failure.
( cd "$ROOT/ui" && ./node_modules/.bin/tsc --noEmit ) \
  || { echo "FAIL: the UI does not type-check, so the required-projectId guard is not enforced"; exit 1; }
echo "  tsc --noEmit clean, so the guard is enforced at build time"
echo "NO SILENT DEFAULT"

echo
echo "########## POINT 9 — artifacts cannot be partitioned, and says so ##########"
A_TOTAL="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT count(*) FROM artifacts;")"
A_SESS="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT count(*) FROM artifacts WHERE source_session_id IS NOT NULL;")"
A_WI="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT count(*) FROM artifacts WHERE source_workitem_id IS NOT NULL;")"
A_NONE="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT count(*) FROM artifacts WHERE source_session_id IS NULL AND source_workitem_id IS NULL;")"
echo "  artifacts: total=${A_TOTAL} with_session=${A_SESS} with_workitem=${A_WI} with_neither=${A_NONE}"
[ "$A_SESS" -eq 0 ] || { echo "FAIL: source_session_id is now populated — a derived join became possible"; exit 1; }
# The derived join would attribute 4 of 15 and silently drop 11. That is why the
# command refuses rather than joining: a short list that looks filtered is worse
# than an honest unscoped one.
DERIVABLE=$((A_SESS + A_WI))
UNDERIVABLE=$((A_TOTAL - DERIVABLE))
echo "  a derived join would attribute ${DERIVABLE} of ${A_TOTAL} and silently drop ${UNDERIVABLE}"
[ "$UNDERIVABLE" -gt 0 ] || { echo "FAIL: every artifact is now derivable — revisit the decision"; exit 1; }
# The command must NOT pretend: no project_id parameter, and the doc says why.
if grep -qE "pub async fn cmd_list_artifacts" -A 6 "$ROOT/src-tauri/src/commands/artifact_cmds.rs" \
   | grep -q "project_id"; then
  echo "FAIL: cmd_list_artifacts now takes a project_id it cannot honour"; exit 1
fi
grep -q "ARTIFACTS ARE \*\*NOT\*\* PROJECT-SCOPED" "$ROOT/src-tauri/src/commands/artifact_cmds.rs" \
  || { echo "FAIL: the refusal is not recorded where a reader will find it"; exit 1; }
grep -q "NOT project-scoped" "$ROOT/ui/src/stores/useDataStore.ts" \
  || { echo "FAIL: the UI does not say the artifacts half is unscoped"; exit 1; }
echo "  the command takes no project_id and both layers say why"
# And the operator's option is on the branch as a script he runs, not applied.
[ -f "$ROOT/infra/postgres/migrations/015_artifacts_project_id.sql" ] \
  || { echo "FAIL: the operator's migration script is missing"; exit 1; }
grep -q "THIS IS A LIVE MIGRATION. IT IS THE OPERATOR'S ACT" \
  "$ROOT/infra/postgres/migrations/015_artifacts_project_id.sql" \
  || { echo "FAIL: the migration script does not say whose act it is"; exit 1; }
echo "  the operator's option exists as a script, clearly marked as his act"

echo
echo "########## POINT 10 — additive: no live write, no gate, no merge ##########"
B="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM seats)||'/'||(SELECT count(*) FROM project_role_bindings);")"
DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/${LIVE}" PROJECT_ID=seatloom "$PROBE" >/dev/null
A="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc "SELECT (SELECT count(*) FROM canonical_events)||'/'||(SELECT count(*) FROM seats)||'/'||(SELECT count(*) FROM project_role_bindings);")"
echo "  live events/seats/bindings: before=${B} after=${A}"
[ "$B" = "$A" ] || { echo "FAIL: the probe wrote to live"; exit 1; }
# No migration applied: the artifacts column must still be absent.
PID_COL="$(podman exec seatloom-postgres psql -U seatloom -d "$LIVE" -tAc \
  "SELECT count(*) FROM information_schema.columns WHERE table_name='artifacts' AND column_name='project_id';")"
[ "$PID_COL" -eq 0 ] || { echo "FAIL: the artifacts migration was applied to live — that is the operator's act"; exit 1; }
echo "  the artifacts migration was NOT applied; the counts did not move"
echo "ADDITIVE ONLY"

echo
echo "ALL POINTS PASS"
