#!/usr/bin/env bash
# aegis-self-wake.sh — Aegis 的自醒 waker(pi 版)。
#
# 为什么存在:每个席位都有 seatloom-waker 叫醒,只有 Aegis 没有 ——
# 唤醒服务的日志明说 `NOTIFY for unmapped seat seat-aegis-001 — ignoring`。
# 于是任何经过 Aegis 的环节都会卡到张先生出现为止。2026-10-10 / 10-11
# 三次停摆全是这一条,不是别的原因。
#
# 为什么是系统 cron 而不是会话内定时任务:会话内的定时任务随会话死掉
# (旧 Aegis 的 CronCreate 7d114e8e 就是这么死的,切到 pi 之后更是没有这个工具)。
# 系统 crontab 活在会话之外:换 harness、换会话、pi 崩了重启,它都还在。
#
# 它只做一件事:按 fleet waker 的既定通道(tmux send-keys)给 Aegis 的面板
# 发一条**短的**读文件指令。真正的心跳协议在
# docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md,改协议改那个文件,不用改这里。
# 之所以不直接发长字符串:send-keys 长字面量会被截断,
# 席位收到过半条消息(scripts/seat-dispatch.sh 第 4 行,实测过)。
#
# 它不碰账本、不碰数据库、不改任何权限文件 —— 只是"去叫一声"。
# 被叫醒之后做什么,是 Aegis 自己的判断,和 seatloom-waker 对别的席位一样。
#
# crontab(每 :11/:41):
#   11,41 * * * * /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/scripts/aegis-self-wake.sh >> /tmp/seatloom_aegis_waker.log 2>&1
set -uo pipefail

REPO="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom"
SESS="Aegis-Supervisor"
HEARTBEAT="$REPO/docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md"
TS="$(date '+%Y-%m-%d %H:%M:%S')"

log(){ printf '%s aegis-self-wake: %s\n' "$TS" "$*"; }

# cron 的 PATH 极简(实测只有 /usr/bin:/bin),tmux 在 /opt/homebrew/bin,
# 光靠 command -v 找不到。按已知位置逐个找;都找不到就明说,不要静默成功 ——
# 一个永远不报错的自醒,比没有自醒更有害。
TMUX_BIN=""
for c in "$(command -v tmux || true)" /opt/homebrew/bin/tmux /usr/local/bin/tmux /usr/bin/tmux; do
  [ -n "$c" ] && [ -x "$c" ] && { TMUX_BIN="$c"; break; }
done
if [ -z "$TMUX_BIN" ]; then
  log "ERROR — 找不到 tmux(PATH=$PATH);自醒这一轮是空转,不是成功"
  exit 1
fi

# 只有 pane 里真的跑着 agent 才叫。pane 停在 shell 上时把长文本送进去,
# 它会当成 shell 命令执行 —— 那不是叫醒,那是乱按键盘。
pane_cmd="$("$TMUX_BIN" list-panes -t "$SESS" -F '#{pane_current_command}' 2>/dev/null | head -1)"
if [ -z "$pane_cmd" ]; then
  log "SKIP — 会话 $SESS 不存在(Aegis 没起来,叫也无用)"
  exit 0
fi
case "$pane_cmd" in
  node|bun|pi) ;;  # pi 是 node 进程;bun 是备用运行时
  *)
    log "SKIP — pane 里跑的是 '$pane_cmd',不是 agent(不往 shell 里塞文本)"
    exit 0;;
esac

if [ ! -r "$HEARTBEAT" ]; then
  log "ERROR — 心跳协议文件不在:$HEARTBEAT(叫了也没东西可读)"
  exit 1
fi

# 和 seat-dispatch.sh 同一套手法:先清掉输入行里可能残留的半截文本,
# 再发一条短指令,payload 走文件,不走在 tmux 输入行上。
"$TMUX_BIN" send-keys -t "$SESS" C-u 2>/dev/null || true
sleep 0.3
"$TMUX_BIN" send-keys -t "$SESS" -l "Aegis 自醒心跳 — Read $HEARTBEAT in full and act on it."
sleep 0.3
"$TMUX_BIN" send-keys -t "$SESS" Enter
log "POKED $SESS (pane=$pane_cmd) — 心跳指令已入队"
