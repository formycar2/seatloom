#!/usr/bin/env bash
# aegis-self-wake.sh — Aegis 的自醒 waker。
#
# 为什么存在:每个席位都有 seatloom-waker 叫醒,只有 Aegis 没有 ——
# 唤醒服务的日志明说 `NOTIFY for unmapped seat seat-aegis-001 — ignoring`。
# 于是任何经过 Aegis 的环节(门事件、跨席位裁决、迁移落库)都卡到张先生出现为止。
# 2026-10-10 / 10-11 三次停摆全是这一条。
#
# 为什么是系统 cron 而不是会话内定时任务:会话内的定时任务随会话死掉
# (旧 Aegis 的 CronCreate 7d114e8e 就是这么死的,切到 pi 之后这个工具根本不存在)。
# 系统 crontab 活在会话之外:换 harness、换会话、pi 崩了重启,它都还在。
#
# 🔴 为什么这个文件不能放在 ~/Documents 下面(2026-10-12 实测,不是推测):
#    macOS 的 TCC 会拦住 cron 对「文档」目录的一切访问。cron 到点执行仓库里的
#    同名脚本,报 `Operation not permitted`,**脚本一行都没跑**。同一个脚本放到
#    ~/.local/bin/ 立刻正常。判据是探针:两个只 echo 一次的脚本,一个在仓库里
#    一个在 ~/.local/bin/,cron 每分钟跑一遍 —— 只有外面的那个输出。
#    所以:**仓库里这份是给人审的原本,cron 跑的是安装器装到 ~/.local/bin 的那份。**
#    改行为改这份,然后重跑 scripts/install-aegis-self-wake.sh。
#
# 🔴 它也刻意不读仓库里任何文件(连 cd 进仓库都没有):心跳协议是它读给我听的,
#    不是 cron 读的。cron 读不到 ~/Documents,而这个脚本要在 cron 的上下文里活。
#    心跳协议在 docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md,由我(agent)去读。
#
# 它只做一件事:按 fleet waker 的既定通道(tmux send-keys)给 Aegis 的面板发一条
# **短的**指令。不直接发长字符串 —— send-keys 长字面量会被截断,
# 席位收到过半条消息(scripts/seat-dispatch.sh 第 4 行,实测过)。
#
# 它不碰账本、不碰数据库、不改任何权限文件 —— 只是"去叫一声"。
# 被叫醒之后做什么,是 Aegis 自己的判断,和 seatloom-waker 对别的席位一样。
set -uo pipefail

SESS="Aegis-Supervisor"
# 短指令。payload 不走 tmux 输入行;路径只是文本,不访问文件系统。
#
# 🔴 行首那个 `#` 是**构造保证**,不是装饰。质量验证席位 2026-10-12 实测发现:
#    捅进 pane 的文本,如果那个 pane 恰好停在裸 shell 提示符上,会被 shell 当场执行
#    (它观察到 zsh: number expected / bash: syntax error near unexpected token `(')。
#    今天的消息只是碰巧因为含有括号才变成语法错误 —— 那是运气,不是保证。
#    一旦消息文本里长出 shell 元字符,或者 pane 的 shell 里压着半行没输完的输入,
#    它就可能真的做成某件事。
#    行首 `#` 让整行在 bash 和 zsh 里都是**注释**,无论后面是什么都不执行;
#    而 pi 收到的只是一条以 # 开头的提示,无害。
#    上面的 pane 守卫是第二道防线(只在 pane 跑着 agent 时才发),两道都要在。
MSG="# Aegis 自醒心跳 — Read /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md in full and act on it."
TS="$(date '+%Y-%m-%d %H:%M:%S')"

log(){ printf '%s aegis-self-wake: %s\n' "$TS" "$*"; }

# cron 的 PATH 极简(实测只有 /usr/bin:/bin),tmux 在 /opt/homebrew/bin,
# 光靠 command -v 找不到。按已知位置逐个找;都找不到就明说 ——
# 一个永远不报错的自醒,比没有自醒更有害。
TMUX_BIN=""
for c in "$(command -v tmux || true)" /opt/homebrew/bin/tmux /usr/local/bin/tmux /usr/bin/tmux; do
  [ -n "$c" ] && [ -x "$c" ] && { TMUX_BIN="$c"; break; }
done
if [ -z "$TMUX_BIN" ]; then
  log "ERROR — 找不到 tmux(PATH=$PATH);这一轮是空转,不是成功"
  exit 1
fi

# 只有 pane 里真的跑着 agent 才叫。pane 停在 shell 上时把文本送进去,
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

# 和 seat-dispatch.sh 同一套手法:先清掉输入行里可能残留的半截文本,再发短指令。
"$TMUX_BIN" send-keys -t "$SESS" C-u 2>/dev/null || true
sleep 0.3
"$TMUX_BIN" send-keys -t "$SESS" -l "$MSG"
sleep 0.3
"$TMUX_BIN" send-keys -t "$SESS" Enter
log "POKED $SESS (pane=$pane_cmd) — 心跳指令已入队"
