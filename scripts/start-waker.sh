#!/usr/bin/env bash
# Start the seatloom-waker daemon, the thing that nudges an idle seat when a
# dispatch becomes ready.
#
# WHY THIS SCRIPT EXISTS. On 2026-10-28 16:00 I found the waker had never been
# running: no compiled binary, no process, no log. `fleet-status.sh` was checking
# `tmux has-session -t seatloom-waker` - but the waker is a Rust resident daemon
# (crates/seatloom-mcp-emit/src/bin/waker.rs), not a tmux session. When I created
# a session by that name, the status line went green immediately while the daemon
# was still not running.
#
# Worse: I had been poking seats by hand every heartbeat, so work kept moving and
# nothing ever showed the symptom. What I did by hand hid the fact that the
# automatic mechanism was dead.
#
# The daemon takes NO command-line arguments - it reads the environment and then
# loops forever. `--help` does not print help, it STARTS the daemon. That is why
# this script exists rather than a usage line.

set -u

cd "$(dirname "$0")/.." || exit 2

TMUX_BIN=""
for c in "$(command -v tmux || true)" /opt/homebrew/bin/tmux /usr/local/bin/tmux /usr/bin/tmux; do
  if [ -n "$c" ] && [ -x "$c" ]; then TMUX_BIN="$c"; break; fi
done
if [ -z "$TMUX_BIN" ]; then
  echo "FAIL: tmux not found (PATH=$PATH)" >&2
  exit 2
fi

# Already running? Say so, do not start a second one.
# 只认 waker 二进制本身的进程。pgrep -f 'seatloom-waker' 会抓到三类:
#   外层 zsh -c ...target/debug/seatloom-waker(是壳,不是守护进程)
#   target/debug/seatloom-waker          (这才是)
#   任何命令行里提到这个名字的进程(包括查它的那条命令自己)
# 所以用 ps 抓全表,再筛 command 以 target/ 下的可执行路径开头的那条。
waker_pid=""
for p in $(pgrep -f 'seatloom-waker' 2>/dev/null); do
  c=$(ps -p "$p" -o command= 2>/dev/null)
  case "$c" in
    target/*seatloom-waker|*/target/*seatloom-waker) waker_pid="$p"; break ;;
  esac
done
if [ -n "$waker_pid" ]; then
  echo "already running (pid $waker_pid)"
  exit 0
fi

echo "== building the waker =="
cargo build -p seatloom-mcp-emit --bin seatloom-waker || {
  echo "FAIL: cargo build failed" >&2
  exit 1
}
[ -x target/debug/seatloom-waker ] || {
  echo "FAIL: build reported success but target/debug/seatloom-waker is not there" >&2
  exit 1
}

if [ ! -f .seatloom/dispatch-seats.tsv ]; then
  echo "FAIL: .seatloom/dispatch-seats.tsv is missing - the daemon maps seats to" >&2
  echo "      tmux sessions from it and will not start usefully without it" >&2
  exit 1
fi
echo "  seat map: $(grep -c '^seat-' .seatloom/dispatch-seats.tsv) seat(s)"

echo "== starting =="
"$TMUX_BIN" new-session -d -s seatloom-waker -c "$(pwd)" \
  "SEATLOOM_PROJECT_ID=seatloom target/debug/seatloom-waker 2>&1 | tee /tmp/seatloom_waker.log" || {
  echo "FAIL: could not create the tmux session" >&2
  exit 1
}

# Verify by PROCESS, not by session name. A tmux session by this name proves
# nothing about the daemon.
sleep 3
waker_pid=$(pgrep -f 'seatloom-waker' 2>/dev/null | head -1)
if [ -z "$waker_pid" ]; then
  echo "FAIL: no seatloom-waker process after start. Its own log:" >&2
  cat /tmp/seatloom_waker.log 2>/dev/null >&2
  exit 1
fi
echo "  running, pid $waker_pid"
echo "  log: /tmp/seatloom_waker.log"
echo
echo "  first lines of its own report:"
head -4 /tmp/seatloom_waker.log 2>/dev/null | sed 's/^/    /'

# The daemon's own first output is the honest signal that it is armed.
if ! grep -q 'armed\.' /tmp/seatloom_waker.log 2>/dev/null; then
  echo
  echo "  WARNING: it has not printed 'armed.' yet. It may still be connecting" >&2
  echo "           (it retries with backoff if the database is not up yet)." >&2
  echo "           Check again in a minute; do not assume it is broken." >&2
fi

exit 0
