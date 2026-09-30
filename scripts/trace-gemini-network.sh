#!/bin/bash
# Gemini CLI network trace — run manually when it breaks or works
# Usage: bash scripts/trace-gemini-network.sh [log-file]
# Default log: .local/evidence/gemini-network-trace/YYYY-MM-DD-HH-MM.txt

set -euo pipefail

TS=$(date +%Y-%m-%d-%H-%M)
EVIDENCE_DIR=".local/evidence/gemini-network-trace"
LOG_FILE="${1:-$EVIDENCE_DIR/$TS.txt}"

mkdir -p "$EVIDENCE_DIR"

{
  echo "=== Gemini Network Trace ==="
  echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S %Z')"
  echo "Host: $(hostname)"
  echo ""

  echo "--- 1. Public IP (Google DNS) ---"
  curl -s --max-time 5 https://ifconfig.me 2>&1 || echo "FAILED"
  echo ""

  echo "--- 2. Public IP (IP.sb) ---"
  curl -s --max-time 5 https://api.ip.sb/ip 2>&1 || echo "FAILED"
  echo ""

  echo "--- 3. DNS resolution for generativelanguage.googleapis.com ---"
  nslookup generativelanguage.googleapis.com 2>&1 || echo "nslookup FAILED"
  echo ""

  echo "--- 4. DNS resolution for www.googleapis.com ---"
  nslookup www.googleapis.com 2>&1 || echo "nslookup FAILED"
  echo ""

  echo "--- 5. HTTP connectivity test to Gemini API ---"
  curl -sI --max-time 10 https://generativelanguage.googleapis.com/v1beta/models 2>&1 | head -5 || echo "curl FAILED"
  echo ""

  echo "--- 6. HTTP timing + status ---"
  curl -s --max-time 10 -w "HTTP Status: %{http_code}\nTime Connect: %{time_connect}s\nTime Total: %{time_total}s\nRemote IP: %{remote_ip}\n\n" -o /dev/null https://generativelanguage.googleapis.com/v1beta/models 2>&1 || echo "curl FAILED"
  echo ""

  echo "--- 7. Proxy environment ---"
  env | grep -i proxy || echo "No proxy env vars set"
  echo ""

  echo "--- 8. IPv4/IPv6 check ---"
  echo "IPv4 default route:"
  netstat -rn 2>/dev/null | grep default | grep -v '::' || echo "No IPv4 default route"
  echo "IPv6 default route:"
  netstat -rn 2>/dev/null | grep '^default' | grep '::' || echo "No IPv6 default route"
  echo ""

  echo "--- 9. Google basic connectivity (www.google.com) ---"
  curl -sI --max-time 5 https://www.google.com 2>&1 | head -3 || echo "FAILED"
  echo ""

  echo "--- 10. Gemini CLI version ---"
  if command -v gemini-cli &>/dev/null; then
    gemini-cli --version 2>&1 || echo "gemini-cli present but --version failed"
  else
    echo "gemini-cli not in PATH"
  fi
  echo ""

  echo "--- 11. Gemini CLI ping test ---"
  if command -v gemini-cli &>/dev/null; then
    timeout 15 gemini-cli --help 2>&1 | head -5 || echo "gemini-cli timeout or FAILED"
  else
    echo "Skipped: gemini-cli not found"
  fi

} > "$LOG_FILE" 2>&1

echo "Trace written to: $LOG_FILE"
echo "Size: $(wc -c < "$LOG_FILE") bytes"
