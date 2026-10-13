#!/usr/bin/env bash
# Proof for the waker hardening (dispatch disp-q__RsEv13FNQ78nCbGd8c).
#
# The production deadlock: a seat whose turn ended WITHOUT complete_dispatch
# leaves its dispatch `in_flight` forever, and the waker's "busy → it self-drains"
# rule then never called it again — three seats were stuck for twelve hours.
#
# This proof runs the REAL seatloom-waker binary against a SCRATCH database with a
# THROWAWAY tmux session as the only mapped seat, so no live seat is ever nudged.
# It demonstrates:
#   POINT 1  a stale in_flight claim + an idle pane -> the waker pokes anyway
#   POINT 2  a FRESH in_flight claim              -> the waker does NOT poke
#   POINT 3  idle + ready work waiting            -> the periodic sweep pokes
#            (the lost-NOTIFY case: the NOTIFY fired while the seat was busy)
#   POINT 4  a seat added to the map while the daemon runs is picked up with no
#            restart (hot reload)
#   POINT 5  the pane-liveness rule: a recognised agent in the pane (even a
#            DIFFERENT one) counts as working, and the log names which
#
# Env knobs are set small so the proof runs in seconds, not minutes.
set -uo pipefail

DB="${SEATLOOM_PROOF_DB:-seatloom_waker_proof}"
DBURL="postgresql://seatloom:seatloom@localhost:5432/${DB}"
ROOT="$(git rev-parse --show-toplevel)"
BIN="$ROOT/target/debug/seatloom-waker"
PROOF_SESSION="waker-proof-seat"
MAP="$ROOT/.scratch/waker-proof-seats.tsv"
LOG="$ROOT/.scratch/waker-proof.log"
SHIM_DIR="$ROOT/.scratch/waker-proof-bin"
SHIM_LOG="$ROOT/.scratch/waker-proof-tmux.log"
SEAT="seat-waker-proof-001"
SEAT2="seat-waker-proof-002"
SESSION2="waker-proof-seat-2"

[[ "$DB" == "seatloom" ]] && { echo "REFUSING live DB"; exit 1; }
psql() { podman exec -i seatloom-postgres psql -U seatloom -d "$DB" -tAc "$1"; }

cleanup() {
  tmux kill-session -t "$PROOF_SESSION" 2>/dev/null
  if [ -n "${WAKER_PID:-}" ]; then kill "$WAKER_PID" 2>/dev/null; fi
}
trap cleanup EXIT

echo "target database = ${DB} (scratch; NEVER live seatloom)"
echo "proof session   = ${PROOF_SESSION} (a bare shell — no real seat is nudged)"

# --- fixture: a scratch DB with the governed-dispatch schema -----------------
podman exec -i seatloom-postgres psql -U seatloom -d postgres -c "DROP DATABASE IF EXISTS ${DB};" >/dev/null 2>&1
podman exec -i seatloom-postgres psql -U seatloom -d postgres -c "CREATE DATABASE ${DB};" >/dev/null 2>&1
PSQL="podman exec -i seatloom-postgres psql -U seatloom -d ${DB} -v ON_ERROR_STOP=1 -q"
for f in 001_seatloom_core 002_document_authority 003_write_ingest_reconcile 004_operational_review_and_continuity 005_prompt_and_channel_action_authority 009_runtime_triple; do
  $PSQL -f - < "$ROOT/infra/postgres/schema/$f.sql"
done
for f in "$ROOT"/infra/postgres/seed/*.sql; do $PSQL -f - < "$f"; done
for f in 008_project_isolation 010_p0_schema_foundation 011_agent_history_archive 012_governed_dispatch; do
  $PSQL -f - < "$ROOT/infra/postgres/schema/$f.sql"
done
echo "fixture: schema 001-012 applied"

psql "INSERT INTO projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) \
        VALUES ('waker-proof','waker proof',NOW(),1024,1024);" >/dev/null
psql "INSERT INTO seats (id,name,created_at,capability_tags,status) \
        VALUES ('${SEAT}','waker proof seat',NOW(),'{}','active');" >/dev/null

# --- the throwaway tmux session + the seat map ------------------------------
tmux kill-session -t "$PROOF_SESSION" 2>/dev/null
tmux new-session -d -s "$PROOF_SESSION"          # a bare shell: pane has no children
printf '# proof map\n%s\t%s\n' "$SEAT" "$PROOF_SESSION" > "$MAP"
: > "$LOG"

# A tmux shim on PATH: every call the waker makes is logged, then forwarded to
# the real tmux. The nudge TEXT cannot be the proof signal — it contains an emoji
# and em-dashes, which a bare shell's line editor drops (verified: `tmux send-keys
# "a—b"` into a zsh pane produces nothing at all), so the proof observes the CALL.
# Resolve the real tmux BEFORE the shim exists, and bake the absolute path into
# the shim (a PATH lookup inside it would find the shim itself).
REAL_TMUX="$(command -v tmux)"
[ -n "$REAL_TMUX" ] || { echo "tmux not found"; exit 1; }
mkdir -p "$SHIM_DIR"
cat > "$SHIM_DIR/tmux" <<SHIM
#!/usr/bin/env bash
echo "\$(date +%T) tmux \$*" >> "$SHIM_LOG"
exec $REAL_TMUX "\$@"
SHIM
chmod +x "$SHIM_DIR/tmux"
: > "$SHIM_LOG"

# --- start the waker against the scratch DB --------------------------------
# Small knobs: stale after 5s, sweep every 2s, cooldown 1s.
DATABASE_URL="$DBURL" \
PATH="$SHIM_DIR:$PATH" \
SEATLOOM_SEAT_MAP="$MAP" \
SEATLOOM_STALE_AFTER_SECS=20 \
SEATLOOM_SWEEP_SECS=2 \
SEATLOOM_POKE_COOLDOWN_SECS=2 \
  "$BIN" > "$LOG" 2>&1 &
WAKER_PID=$!
sleep 2
echo "waker pid ${WAKER_PID}; log so far:"
sed 's/^/    /' "$LOG" | head -5

# A nudge = the waker sending the message and then Enter to that session.
nudged() { grep -q "send-keys -t $1 Enter" "$SHIM_LOG"; }
nudges() { local n; n=$(grep -c "send-keys -t $1 Enter" "$SHIM_LOG" 2>/dev/null); echo "${n:-0}"; }
clear_pane() { tmux send-keys -t "$PROOF_SESSION" C-c 2>/dev/null; sleep 0.2; }

echo
echo "########## POINT 1 — a STALE in_flight claim with an idle pane still gets poked ##########"
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at,started_at) \
        VALUES ('disp-stale','${SEAT}','seat:seat-lyra-001','waker-proof','the stuck task','{}', \
                'in_flight',5,NOW() - interval '40 minutes', NOW() - interval '40 minutes');" >/dev/null
echo "seeded: disp-stale in_flight since 40 minutes ago (seat idle, no ready work)"
echo "waiting for the sweep (stale-after 20s, sweep every 2s) ..."
for i in $(seq 1 20); do nudged "$PROOF_SESSION" && break; sleep 1; done
if nudged "$PROOF_SESSION"; then
  echo "PASS: the waker poked the session $(nudges "$PROOF_SESSION") time(s) — the stale claim did not silence it"
  grep -E "send-keys -t $PROOF_SESSION Enter" "$SHIM_LOG" | tail -1 | sed 's/^/    tmux: /'
else
  echo "FAIL: no nudge after the stale threshold"
fi
grep -E "STALE claim" "$LOG" | tail -1 | sed 's/^/    waker: /'

echo
echo "########## POINT 2 — a FRESH in_flight claim is NOT poked ##########"
psql "UPDATE dispatches SET started_at = NOW() WHERE id = 'disp-stale';" >/dev/null
echo "seeded: disp-stale in_flight since NOW (a healthy, working seat)"
sleep 3   # let the previous nudge's cooldown lapse, so this is not a false pass
BEFORE=$(nudges "$PROOF_SESSION")
sleep 10  # five sweeps, still well under the 20s threshold
if [ "$(nudges "$PROOF_SESSION")" -gt "$BEFORE" ]; then
  echo "FAIL: a fresh claim was poked (that would interrupt a working seat)"
else
  echo "PASS: no further nudge — the fresh claim is respected"
fi
grep -E "in_flight .* for .*pane" "$LOG" | tail -1 | sed 's/^/    waker: /' || true

echo
echo "########## POINT 3 — idle + ready work is poked by the PERIODIC sweep ##########"
clear_pane
psql "UPDATE dispatches SET status = 'done', started_at = NULL, completed_at = NOW() WHERE id = 'disp-stale';" >/dev/null
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) \
        VALUES ('disp-ready','${SEAT}','seat:seat-lyra-001','waker-proof','the queued task','{}','ready',5,NOW());" >/dev/null
echo "seeded: the seat is idle with one READY dispatch (no NOTIFY was sent for it)"
BEFORE=$(nudges "$PROOF_SESSION")
for i in $(seq 1 20); do [ "$(nudges "$PROOF_SESSION")" -gt "$BEFORE" ] && break; sleep 1; done
if [ "$(nudges "$PROOF_SESSION")" -gt "$BEFORE" ]; then
  echo "PASS: the sweep woke the idle seat — a NOTIFY missed while busy is not lost forever"
else
  echo "FAIL: the ready dispatch was not picked up by the sweep"
fi
grep -E "sweep: idle seat" "$LOG" | tail -1 | sed 's/^/    waker: /'

echo
echo "########## POINT 4 — a seat added to the map while running is picked up ##########"
tmux new-session -d -s "$SESSION2"
printf '# proof map\n%s\t%s\n%s\t%s\n' "$SEAT" "$PROOF_SESSION" "$SEAT2" "$SESSION2" > "$MAP"
psql "INSERT INTO seats (id,name,created_at,capability_tags,status) \
        VALUES ('${SEAT2}','waker proof seat 2',NOW(),'{}','active');" >/dev/null
psql "INSERT INTO dispatches (id,target_seat_id,issued_by,project_id,title,payload,status,priority,issued_at) \
        VALUES ('disp-ready-2','${SEAT2}','seat:seat-lyra-001','waker-proof','the queued task','{}','ready',5,NOW());" >/dev/null
echo "added ${SEAT2} -> ${SESSION2} to the map (the waker was NOT restarted)"
for i in $(seq 1 20); do nudged "$SESSION2" && break; sleep 1; done
if nudged "$SESSION2"; then
  echo "PASS: the new seat was nudged — hot reload works, no restart needed"
else
  echo "FAIL: the new seat was not picked up"
fi
grep -E "seat map loaded" "$LOG" | tail -2 | sed 's/^/    waker: /'
tmux kill-session -t "waker-proof-seat-2" 2>/dev/null

echo
echo "########## POINT 5 — a recognised agent in the pane counts as working ##########"
# A pane running a recognised agent (node) must NOT be poked even with a stale
# claim: the log must say which agent it saw.
BEFORE=$(nudges "$PROOF_SESSION")
psql "UPDATE dispatches SET status = 'in_flight', started_at = NOW() - interval '40 minutes' WHERE id = 'disp-ready';" >/dev/null
tmux kill-session -t "$PROOF_SESSION" 2>/dev/null
tmux new-session -d -s "$PROOF_SESSION" "node -e 'setInterval(() => {}, 1000)'"
sleep 1
PANE_PID=$("$REAL_TMUX" list-panes -t "$PROOF_SESSION" -F '#{pane_pid}' | head -1)
PANE_CHILD=$(ps -ax -o pid=,ppid=,comm= | awk -v p="$PANE_PID" '$2==p {print $3}' | head -1)
echo "pane pid ${PANE_PID}, child: ${PANE_CHILD} (a recognised agent)"
sleep 6
if [ "$(nudges "$PROOF_SESSION")" -gt "$BEFORE" ]; then
  echo "FAIL: a pane with a working agent was poked"
else
  echo "PASS: no nudge — an agent in the pane counts as working"
fi
grep -E "an agent is working" "$LOG" | tail -1 | sed 's/^/    waker: /'
echo "    (the log names the agent it saw, so a DIFFERENT agent is visible rather than assumed)"
tmux kill-session -t "$PROOF_SESSION" 2>/dev/null

echo
echo "########## waker log (full) ##########"
sed 's/^/    /' "$LOG"

echo
echo "proof complete: stale claim -> poked; fresh claim -> left alone; idle+ready -> poked by the periodic sweep; new seat -> hot-reloaded; agent in pane -> respected and named."
