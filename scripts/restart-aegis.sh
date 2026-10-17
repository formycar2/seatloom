#!/usr/bin/env bash
# restart-aegis.sh — 让 Aegis 换上磁盘上那个新版本的 pi。
#
# 为什么需要它:`pi update` 只改磁盘。启动器 `~/.pi/agent/bin/pi` 在 exec 那一刻
# 读一次 `install/current-version`,然后 exec 对应 release 里的程序。进程起来之后
# 代码就在内存里了 —— 磁盘换成 1.3.0 对一个 11:24 起的进程没有任何影响。
# 这就是 2026-10-16 的实际情况:全队 6 个席位重启换上了 1.3.0,Aegis 自己没换。
#
# 为什么必须由张先生跑,不能让 Aegis 自己跑:
#   重启 Aegis = 杀掉 Aegis 正在运行的进程。而这个脚本本身是 Aegis 用 bash 工具
#   拉起的子进程 —— 杀皮的瞬间这条命令也死了,后面几步一步都不会执行,
#   留下一个「pi 退了、pane 停在 shell、没有新的起来」的中间态。
#   而自醒 waker 刻意不往 shell 里塞文本(pane 不是 agent 它就 SKIP),
#   所以没有任何东西会把 Aegis 拉回来。那正是「项目不能中断」的反面。
#   和 scripts/restart-seats-for-new-tools.sh 同一条理由:杀一个正在干活的 agent
#   可能丢掉它那一轮,这件事归张先生决定。
#
# 用法(从 Aegis 面板之外的终端跑):
#   bash scripts/restart-aegis.sh
#
# 安全性:
#   - 主检出有**已跟踪文件**的未提交改动就拒绝重启(那一轮的工作只在进程里)。
#     未跟踪的草稿不算 —— 杀掉进程不会让盘上的文件消失。
#   - 每一步失败都原地停并报错,不留半切换状态。可重复跑。
#
# 🔴 为什么这里用 ps+awk 而不用 pgrep -P(2026-10-16 实测,不是推测):
#    macOS 的 pgrep **静默看不见调用者的祖先进程**。Aegis 的 bash 命令是 pi 的子进程,
#    所以 `pgrep -P <aegis 的 pane_pid>` 永远返回空 —— 而 pane 里明明跑着 pi。
#    从另一个 tmux 会话里跑同一条命令就正常返回。查席位时不会踩到(席位不是 Aegis 的祖先),
#    但查 Aegis 自己必然踩到,而且症状是"查不到",不是报错 —— 一个静默的假阴性。
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

SESS="Aegis-Supervisor"
PI="${SEATLOOM_PI_BIN:-$HOME/.local/bin/pi}"
# 2026-10-17: pin an EXISTING session id so a transferred session is continued
# instead of orphaned. Same gap seat-launch.sh had: a fresh session starts empty
# while the transferred transcript sits under the OLD id, so "migrate the memory"
# silently does nothing. Set SEATLOOM_RESUME_UUID to the uuid from the transferred
# file name <timestamp>_<uuid>.jsonl — the part after the underscore.
RESUME_UUID="${SEATLOOM_RESUME_UUID:-}"
MODEL="stepplan-anthropic/step-5-preview"
BIN="$REPO/target/debug/seatloom-mcp-emit"
VERFILE="$HOME/.pi/agent/install/current-version"

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; say "   把上面这段贴给 Aegis。不要手动重试后面的步骤。"; exit 1; }

# pane 的直接子进程 pid 列表。刻意不用 pgrep -P,原因见文件头。
pane_children(){
  ps -axo pid,ppid,command 2>/dev/null | awk -v P="$1" '$2==P {print $1}'
}
# 某个 pid 的直接子进程 pid 列表
children_of(){
  ps -axo pid,ppid,command 2>/dev/null | awk -v P="$1" '$2==P {print $1}'
}
cmdof(){ ps -o command= -p "$1" 2>/dev/null | tr -d '[:space:]'; }
startof(){ ps -o lstart= -p "$1" 2>/dev/null | sed 's/^ *//;s/ *$//'; }

# ══ 一:前置 ══════════════════════════════════════════════════════
line
say "一 — 前置检查"
line

pane="$(tmux list-panes -t "$SESS" -F '#{pane_pid}' 2>/dev/null | head -1)"
[ -n "$pane" ] || fail "找不到会话 $SESS"
panecmd="$(tmux list-panes -t "$SESS" -F '#{pane_current_command}' 2>/dev/null | head -1)"
say "  ✅ 会话在,pane pid=$pane,里面跑着 '$panecmd'"

diskver="$(cat "$VERFILE" 2>/dev/null | tr -d '[:space:]')"
[ -n "$diskver" ] || fail "读不到 $VERFILE"
say "  ✅ 磁盘上的版本:$diskver"

# 旧 Aegis 的 pid:pane 的子进程里那个 agent(pi 是 node 进程,ps 报的名字是 pi)
oldpid=""
for k in $(pane_children "$pane"); do
  case "$(cmdof "$k")" in
    pi|node|bun) oldpid="$k";;
  esac
done
# 反查:如果 pane 明摆着跑着 agent 却一个 pid 都没找到,那是检查本身坏了,不是"Aegis 没在跑"
if [ -z "$oldpid" ]; then
  case "$panecmd" in
    node|pi|bun)
      say "  pane 里跑的是 agent('$panecmd'),但子进程一个都没枚举到:"
      say "    ps -axo pid,ppid,command | awk -v P=$pane '\$2==P' 的输出:"
      ps -axo pid,ppid,command 2>/dev/null | awk -v P="$pane" '$2==P' | sed 's/^/      /'
      fail "枚举 pane 子进程失败 —— 检查本身坏了,不是 Aegis 没在跑。不要当成'不用重启'。"
      ;;
    *)
      fail "pane 里没有 agent 进程(现在是 '$panecmd')—— Aegis 没在跑,不用重启"
      ;;
  esac
fi
say "  ✅ 旧 Aegis:pid=$oldpid,起于 $(startof "$oldpid")"
say "     (它起于 current-version 还是旧版的时候,所以它跑的是旧代码)"

[ -x "$BIN" ] || fail "找不到工具程序 $BIN"

# 主检出有没有没存盘的活
dirty="$(git status --porcelain 2>/dev/null | grep -E '^[ MADRCU]{1,2}[^ ]' | wc -l | tr -d ' ')"
if [ "$dirty" != "0" ]; then
  say "  ⛔ 主检出有 $dirty 个已跟踪文件的未提交改动 —— 不重启"
  git status --porcelain | grep -E '^[ MADRCU]{1,2}[^ ]' | head -5 | sed 's/^/       /'
  fail "先让 Aegis 把那轮工作提交了再来"
fi
say "  ✅ 主检出已跟踪文件全部已提交(另有 $(git status --porcelain | grep -c '^??' | tr -d ' ') 个未跟踪草稿,不影响)"

# 别把自己降级 —— 和 Vela 那次同一个坑
dbmodel="$(podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc \
  "select coalesce(default_model,'') from seats where id='seat-aegis-001'" 2>/dev/null | tr -d ' ')"
say "  ℹ️  账本里 Aegis 的模型:'${dbmodel:-<查不到>}'"
if [ -n "$dbmodel" ] && [ "$dbmodel" != "$MODEL" ]; then
  say "  ⚠️  和本脚本要用的 $MODEL 不一致。"
  say "      不想静默改模型的话,现在就停下,把这段贴给 Aegis 让它判断。"
  fail "账本模型 ($dbmodel) 与启动模型 ($MODEL) 不一致"
fi
say "  ✅ 模型一致,不会被静默降级"

# DRY_RUN=1 只跑到这里:把前置检查真跑一遍,不杀不停。
# 前置检查是整条命令里最容易静默出错的一段(见文件头 pgrep 那条),必须能单独验。
if [ "${DRY_RUN:-0}" = "1" ]; then
  say ""
  say "✅ DRY_RUN —— 前置全部通过,没有杀任何进程。去掉 DRY_RUN 才会真重启。"
  exit 0
fi
say ""

# ══ 二:杀掉旧 Aegis ══════════════════════════════════════════════
line
say "二 — 停掉旧 Aegis (pid=$oldpid)"
line
say "  这一秒之后,新的还没起来 —— 中间态是预期的,第三步立刻接上。"
kill "$oldpid" 2>/dev/null

# 等 pane 回到 shell。pane_current_command 从 node 变回 zsh 才算真的退了。
waited=0
back=0
for _ in $(seq 1 30); do
  c="$(tmux list-panes -t "$SESS" -F '#{pane_current_command}' 2>/dev/null | head -1)"
  case "$c" in
    zsh|bash|sh) back=1; break;;
  esac
  sleep 1; waited=$((waited+1))
done
[ "$back" = "1" ] || fail "等了 ${waited} 秒,pane 还没回到 shell(现在是 '$(tmux list-panes -t "$SESS" -F '#{pane_current_command}' | head -1)')。旧 Aegis 可能没退干净,不要进行第三步"
say "  ✅ 旧 Aegis 已退,pane 回到 shell(等了 ${waited} 秒)"
say ""

# ══ 三:起新的 ════════════════════════════════════════════════════
line
say "三 — 在 $SESS 里起新 Aegis (版本 $diskver)"
line
# 启动契约三条缺一不可(AGENTS.md 第 4 条):
#   跑 **pi**(不是 Claude Code —— 前者读 .pi/mcp.json,后者读 .mcp.json)
#   cwd 是**它自己的工作区**。Aegis 的工作区就是主检出(AGENTS.md 明说不在 packet worktree 里)
#   带 **--approve**(否则停在 "Trust project folder?" 弹窗,永远不加载 MCP)
if [ -n "$RESUME_UUID" ]; then
  tmux send-keys -t "$SESS" "cd $REPO && $PI --approve --model $MODEL --session-id $RESUME_UUID" Enter
  say "  已钉会话 id:$RESUME_UUID(转过去的记忆会接上)"
else
  tmux send-keys -t "$SESS" "cd $REPO && $PI --approve --model $MODEL" Enter
  say "  ⚠️ 没给 SEATLOOM_RESUME_UUID —— 起的是全新会话,转过去的记忆不会接上"
fi
say "  启动命令已发出。"
say ""

# ══ 四:验证(看子进程,不看面板)═══════════════════════════════════
line
say "四 — 验证 MCP 真的加载了(子进程是证据,面板不是)"
line
say "  等 30 秒…"
sleep 30
mcp_ok=0
newpid=""
for attempt in 1 2 3; do
  for k in $(pane_children "$pane"); do
    case "$(cmdof "$k")" in pi|node|bun) newpid="$k";; esac
    for g in $(children_of "$k"); do
      case "$(cmdof "$g")" in
        *seatloom-mcp-emit*)
          say "  ✅ MCP 子进程 pid=$g  $(startof "$g")"
          mcp_ok=1;;
      esac
    done
  done
  [ "$mcp_ok" = "1" ] && break
  say "  第 ${attempt} 次没看到 MCP 子进程,再等 15 秒…"
  sleep 15
done

say ""
line
say "结果"
line
if [ "$mcp_ok" != "1" ]; then
  say "  ❌ 新 Aegis 起来了但看不到 MCP 子进程 —— 这就是静默失效,不要当成成功。"
  say "     最可能的原因:cwd 不对、漏了 --approve、或 .pi/mcp.json 没被读到。"
  say "     把这段贴给张先生,不要手动重试第三、四段。"
  exit 1
fi
say "  ✅ MCP 已加载(新 Aegis pid=$newpid,起于 $(startof "$newpid"))"

# 新进程必须晚于新 release 目录 —— 否则等于没换上,而这一项必须能报错
relepoch="$(stat -f %m "$HOME/.pi/agent/install/releases/$diskver" 2>/dev/null)"
newstart="$(startof "$newpid")"
newepoch="$(date -j -f '%a %b %d %H:%M:%S %Y' "$newstart" +%s 2>/dev/null)"
if [ -z "$newepoch" ]; then
  say "  ⚠️  算不出新进程的起始时间(ps 输出 '$newstart')—— 这项没验,不算通过"
elif [ "$newepoch" -ge "$relepoch" ]; then
  say "  ✅ 新进程晚于 $diskver 的 release 目录 —— 真的换上了(晚 $((newepoch-relepoch)) 秒)"
else
  say "  ❌ 新进程比起 $diskver 还早 $((relepoch-newepoch)) 秒 —— 没换上,面板正常但跑的是旧代码"
  exit 1
fi

say ""
say "  pane 尾部:"
tmux capture-pane -p -t "$SESS" 2>/dev/null | grep -v '^$' | tail -4 | sed 's/^/    /'
say ""
line
say "接下来(给新的 Aegis 的第一条消息)"
line
cat <<'HANDOFF'
你刚刚被重启,换上了新版本的 pi。先确认三件事,每件都要能报错:
  1. 进程起始时间要晚于 install/releases/<新版本> 目录的 mtime —— 脚本第四段已经核过并在上面报了结果。
     不要用 `pi --version` 当证据:那会新起一个进程,报的是磁盘版本,不是你正在跑的版本。
  2. MCP 通:工具子进程活着只是必要条件。按仓库规矩,唯一硬证据是派单从 ready 翻成 in_flight,
     那要等你接到活才有。接到之前不要说"已验证可用"。
  3. 自醒 cron 还在(crontab -l | grep aegis-self-wake)。它在会话之外,本该活着 ——
     确认一下,别假设。
然后按当前状态接手:全队 6 个席位已在 11:31 换上 1.3.0,Aegis 是最后一个。
HANDOFF
say ""
say "全部完成。"
exit 0
