#!/usr/bin/env bash
# switch-aegis-to-pi.sh — 把 Aegis 从 Claude Code 切到 pi + step-5-preview。
#
# 为什么必须切:harness 合规迁移要求 2026-10-15 起席位不再用 Claude Code。
# 今天是 10-12。Aegis 是最后一个还在 Claude Code 上的。
#
# 为什么写成脚本:切换**执行完这一刻,旧的 Claude Code 会话就该关掉**。
# 在一个即将消失的会话里手工做这件事,任何一个中间步骤失败都不会有人接着修。
# 脚本可重复跑:每次跑之前都重新验一遍前置,失败了原地停,不留半切换状态。
#
# 它做五件事,顺序不能换:
#   零  把 .pi/mcp.json 建出来(刻意不入库,所以只存在于盘上,是唯一恢复途径)
#   一  前置检查(配置、程序、MCP 可达)—— 缺一项就停
#   二  修 seats 表里 Aegis 那个过期模型名(它还是 Claude Code 时代的)
#   三  在 Aegis-Supervisor 里起 pi
#   四  验证 pi 真的加载了 MCP(**看子进程,不看面板**)
#
# 用法: bash scripts/switch-aegis-to-pi.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

SESS="Aegis-Supervisor"
PI="$HOME/.local/bin/pi"
MODEL="stepplan-anthropic/step-5-preview"
BIN="$REPO/target/debug/seatloom-mcp-emit"
CFG="$REPO/.pi/mcp.json"
CTR="seatloom-postgres"

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }

# ══ 零:把 .pi/mcp.json 建出来(缺了就建,已在就跳过) ═══════════════
line
say "零 — 确保 .pi/mcp.json 在"
line
# 这个文件**刻意不入库**(.pi 在 .gitignore 里,和各个席位一样)。
# 所以它只存在于盘上,一旦丢了不会有任何地方能恢复它 —— 而这个脚本是唯一恢复途径。
# 因此这里不假设它已在:缺了就按席位的形状建一份,只改 SEATLOOM_ACTOR_REF。
# 主检出原先没有 .pi/ 目录,那正是静默失效的形状:pi 起得来、面板正常,
# 但 MCP 从未加载,队列无声停摆。
mkdir -p "$REPO/.pi"
if [ -r "$CFG" ] && python3 -c "import json;json.load(open('$CFG'))" 2>/dev/null; then
  say "  ⏭  已在且是合法 JSON,跳过(可重复跑)"
else
  cat > "$CFG" <<'JSON'
{
  "mcpServers": {
    "seatloom": {
      "command": "/Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit",
      "env": {
        "SEATLOOM_ACTOR_REF": "seat:seat-aegis-001",
        "DATABASE_URL": "postgresql://seatloom:seatloom@localhost:5432/seatloom",
        "SEATLOOM_PROJECT_ID": "seatloom"
      }
    }
  }
}
JSON
  if python3 -c "import json;json.load(open('$CFG'))" 2>/dev/null; then
    say "  ✅ 已重建并验证是合法 JSON"
  else
    say "  ❌ 建出来了但不是合法 JSON —— 停"; exit 1
  fi
fi
say ""

# ══ 一:前置检查 ════════════════════════════════════════════════════
line
say "一 — 前置检查(缺任何一项都停,不进入切换)"
line
fail=0
chk(){ if [ "$2" = "1" ]; then printf "  %-34s ✅\n" "$1"; else printf "  %-34s ❌\n" "$1"; fail=1; fi; }

[ -r "$CFG" ]            && chk ".pi/mcp.json 存在" 1 || chk ".pi/mcp.json 存在" 0
[ -x "$BIN" ]            && chk "MCP 程序在盘上" 1  || chk "MCP 程序在盘上" 0
[ -x "$(readlink -f "$PI" 2>/dev/null)" ] && chk "pi 可执行" 1 || chk "pi 可执行" 0
[ -r "$REPO/AGENTS.md" ] && chk "AGENTS.md 在(pi 要读它)" 1 || chk "AGENTS.md 在(pi 要读它)" 0
python3 -c "import json,sys;json.load(open('$CFG'))" 2>/dev/null && chk "mcp.json 是合法 JSON" 1 || chk "mcp.json 是合法 JSON" 0
podman exec "$CTR" true 2>/dev/null && chk "数据库容器在跑" 1 || chk "数据库容器在跑" 0
tmux has-session -t "$SESS" 2>/dev/null && chk "会话 $SESS 在" 1 || chk "会话 $SESS 在" 0

# MCP 以 aegis 身份到底通不通 —— 这是静默失效的唯一判据,不看面板
n_tools="$(printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"aegis-switch-check","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | SEATLOOM_ACTOR_REF="seat:seat-aegis-001" \
    DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/seatloom" \
    SEATLOOM_PROJECT_ID="seatloom" \
    "$BIN" 2>/dev/null | python3 -c "
import sys, json
for l in sys.stdin:
    l = l.strip()
    if not l: continue
    try: m = json.loads(l)
    except Exception: continue
    if m.get('id') != 2: continue
    if 'error' in m: break
    print(len(m['result']['tools'])); break
" 2>/dev/null)"
if [ "${n_tools:-0}" -ge 18 ]; then
  printf "  %-34s ✅ (%s 个工具)\n" "MCP 以 aegis 身份可达" "$n_tools"
else
  printf "  %-34s ❌ (只报到 %s 个工具)\n" "MCP 以 aegis 身份可达" "${n_tools:-0}"
  fail=1
fi

if [ "$fail" != "0" ]; then
  say ""
  say "❌ 前置没过 —— 不切换。把上面这段贴给 Aegis。"
  exit 1
fi
say ""

# ══ 二:修 seats 表里那个过期模型名 ════════════════════════════════
line
say "二 — 修 seats 表里 Aegis 的模型名"
line
# 它现在是 claude-opus-5[1m] —— Claude Code 时代的名字,不是 pi 的 provider/model 格式。
# 不改也不会让启动失败(seat-launch.sh 的 model_for 会校验形状并回退),
# 但账本上留着一个假值,以后没人分得清 Aegis 到底该跑什么。
cur="$(podman exec -e PGPASSWORD=seatloom "$CTR" psql -U seatloom -d seatloom -tAc \
        "select coalesce(default_model,'') from seats where id='seat-aegis-001'" 2>/dev/null | tr -d ' ')"
say "  现值:'${cur:-<查不到>}'"
if [ "$cur" = "$MODEL" ]; then
  say "  ⏭  已经是目标值,跳过(可重复跑)"
elif [ -z "$cur" ]; then
  say "  ⚠️  seat-aegis-001 这行不存在 —— 跳过。MCP 的席位 FK 若需要它,另外处理。"
else
  if podman exec -e PGPASSWORD=seatloom "$CTR" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 -q \
       -c "UPDATE seats SET default_model='$MODEL' WHERE id='seat-aegis-001'"; then
    now="$(podman exec -e PGPASSWORD=seatloom "$CTR" psql -U seatloom -d seatloom -tAc \
            "select default_model from seats where id='seat-aegis-001'" 2>/dev/null | tr -d ' ')"
    [ "$now" = "$MODEL" ] && say "  ✅ 已改为 '$MODEL'(重新读库核对过)" || say "  ❌ 改了但读回来不对:'$now'"
  else
    say "  ❌ 更新失败 —— 继续切换(这个值不阻断启动),但账本上仍是假值"
  fi
fi
say ""

# ══ 三:起 pi ═════════════════════════════════════════════════════
line
say "三 — 在 $SESS 里起 pi"
line
# 启动契约三条缺一不可(AGENTS.md 第 4 条):
#   跑 **pi**(不是 Claude Code —— 前者读 .pi/mcp.json,后者读 .mcp.json)
#   cwd 是**它自己的工作区**。Aegis 的工作区就是主检出(AGENTS.md 明说我不在 packet worktree 里)
#   带 **--approve**(否则停在 "Trust project folder?" 弹窗,永远不加载 MCP)
# 探针会话已实测过这条命令能起来,并且 MCP 子进程会生成。
pane="$(tmux list-panes -t "$SESS" -F '#{pane_pid}' 2>/dev/null | head -1)"
say "  目标 pane pid=$pane"
say "  发出启动命令…"
tmux send-keys -t "$SESS" "cd $REPO && $PI --approve --model $MODEL" Enter
say ""

# ══ 四:验证 MCP 真的加载了 ════════════════════════════════════════
line
say "四 — 验证(等 pi 起来,看子进程,不看面板)"
line
say "  等 30 秒…"
sleep 30
mcp_ok=0
for attempt in 1 2 3; do
  for k in $(pgrep -P "$pane" 2>/dev/null); do
    for g in $(pgrep -P "$k" 2>/dev/null); do
      case "$(ps -o command= -p "$g" 2>/dev/null)" in
        *seatloom-mcp-emit*)
          say "  ✅ MCP 子进程 pid=$g  $(ps -o lstart= -p "$g")"
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
if [ "$mcp_ok" = "1" ]; then
  say "  ✅ pi 已起来,并且**真的加载了 MCP**(子进程是证据,不是面板)"
  say ""
  say "  pane 尾部:"
  tmux capture-pane -p -t "$SESS" | grep -v '^$' | tail -4 | sed 's/^/    /'
else
  say "  ❌ pi 起来了但看不到 MCP 子进程 —— 这就是静默失效,不要当成成功。"
  say "     最可能的原因:cwd 不对、漏了 --approve、或 .pi/mcp.json 没被读到。"
  say "     把这段贴给 Aegis,不要手动重试第三、四段。"
  exit 1
fi
say ""
line
say "接下来(必须做,否则新 Aegis 没有叫醒服务)"
line
say "  1. 到 $SESS 那个面板,把下面这段发给新的 Aegis:"
say ""
cat <<'HANDOFF'
────────────────────────────────────────────────────────────
你是 Aegis,刚刚从 Claude Code 切到 pi + step-5-preview。**第一件事:重建自醒任务。**
你的自醒 cron 是会话内的,旧会话一关它就死了。现在就建:
  CronCreate(cron="11,41 * * * *", recurring=true, prompt=<下面这段>)
  「Aegis 自醒心跳。按顺序做,每一步都要能报错:
   1. bash scripts/fleet-status.sh —— 看队列、席位存活(pgrep -P pane_pid,不要裸 grep pi)、领单年龄(用库内时钟 round(extract(epoch from (now()-started_at))/60))。
   2. 有派给 seat-aegis-001 的 ready 派单就领来做。
   3. 有席位卡在等门事件,就按「live 可用」核真实条件(能不能真跑成功),条件满足才发门事件,绝不谎报。
   4. 有 stale in_flight 就处理 —— 它会让 waker 永久跳过那个席位。
   5. 有需要张先生决策/授权的,汇总成「变了什么 / 你会看到什么 / 需要你做什么」三段,不用自造缩写,被权限拦住的动作写成 scripts/ 下一条可审计命令给他。」
建完先自报一次 cron id,然后照常跑一轮心跳。

然后按当前状态接手(都已在账本里核实过):
  - **唯一解锁点**:张先生跑 `bash scripts/land-knowledge-live.sh`。
    019 未应用到 live、topics/memories 都是 0、坏卡方案仍为空 ——
    6 件失败过的派单全卡在这条命令的第一、五、六段上。
  - 全队 6 个席位只有 lyra 和 vela 换上了带新工具的程序(18:05 那次部分换动);
    mira/nimbus/flux/onyx 四个的工具子进程还是 10-10 的旧镜像。
    重启守卫已修好(以前把未跟踪草稿误判成"没存盘"),现在六个席位全部合格,
    那条命令的第五段会一次换完。
  - 我(旧 Aegis)刚修了自己的三个错:门事件号发错、回填用了自己编的方案、
    以及那个造成部分换动的守卫。提交在 b64b19b。
  - 有待你裁决的两件:Lyra 问的「v3 单发射者归谁」,以及它要的舰队进程普查。
────────────────────────────────────────────────────────────
HANDOFF
say ""
say "  2. 让新的 Aegis 确认收到后,**关掉这个 Claude Code 会话**。"
say "     两个 Aegis 同时在账本上动作会互相干扰 —— 重叠窗口越短越好。"
say ""
say "全部完成。把上面这段贴给张先生。"
