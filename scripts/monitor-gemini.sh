#!/bin/bash
# Gemini CLI Auto-Monitor Daemon
# Watches for Gemini CLI network failures and auto-captures traces
# Usage: bash scripts/monitor-gemini.sh [interval-seconds]
# Default interval: 300 (5 minutes)
# Runs until interrupted (Ctrl+C) or stopped via .local/evidence/gemini-network-trace/MONITOR_PID

set -euo pipefail

INTERVAL="${1:-300}"
EVIDENCE_DIR=".local/evidence/gemini-network-trace"
STATE_FILE="$EVIDENCE_DIR/monitor-state"
LOG_FILE="$EVIDENCE_DIR/monitor.log"
TRACE_SCRIPT="$(dirname "$0")/trace-gemini-network.sh"

# Ensure npm global bin is in PATH
export PATH="/opt/homebrew/bin:$PATH"

mkdir -p "$EVIDENCE_DIR"

# Write PID file
echo $$ > "$EVIDENCE_DIR/MONITOR_PID"

# Log start
echo "[$(date '+%Y-%m-%d %H:%M:%S')] Monitor started (PID $$, interval ${INTERVAL}s)" >> "$LOG_FILE"

# State tracking: 1=healthy, 0=unhealthy
STATE=1

cleanup() {
  echo "[$(date '+%Y-%m-%d %H:%M:%S')] Monitor stopped (PID $$)" >> "$LOG_FILE"
  rm -f "$EVIDENCE_DIR/MONITOR_PID"
  exit 0
}

trap cleanup SIGINT SIGTERM EXIT

while true; do
  TS=$(date +%Y-%m-%d-%H-%M)
  
  # Quick health check: can we reach Gemini API with HEAD request?
  HTTP_CODE=$(curl -sI --max-time 8 -o /dev/null -w "%{http_code}" https://generativelanguage.googleapis.com/v1beta/models 2>/dev/null || echo "000")
  
  if [[ "$HTTP_CODE" == "000" ]]; then
    # Network failure
    if [[ $STATE -eq 1 ]]; then
      echo "[$(date '+%Y-%m-%d %H:%M:%S')] FAILURE DETECTED (curl $HTTP_CODE) — capturing trace" >> "$LOG_FILE"
      bash "$TRACE_SCRIPT" "$EVIDENCE_DIR/${TS}-FAIL.txt"
      STATE=0
    fi
  else
    # Network OK
    if [[ $STATE -eq 0 ]]; then
      echo "[$(date '+%Y-%m-%d %H:%M:%S')] RECOVERY DETECTED (HTTP $HTTP_CODE) — capturing trace" >> "$LOG_FILE"
      bash "$TRACE_SCRIPT" "$EVIDENCE_DIR/${TS}-RECOVERY.txt"
      STATE=1
    fi
  fi
  
  sleep "$INTERVAL"
done
