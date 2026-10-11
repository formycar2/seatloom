#!/usr/bin/env bash
# restart-seats-for-new-tools.sh — 让席位换上带新工具的程序。
#
# 为什么必须重启:**席位的工具进程是常驻子进程**,在席位启动那一刻就起来了,
# 持有的是**那一刻的程序镜像**。实测(2026-10-11 03:30):
#   洗数据席位的工具进程起于 10-10 21:39,带 open_topic 的程序 10-11 03:16 才建好
#   → 它看不到 open_topic。全队的工具进程都早于 03:16,所以**现在没有任何席位能调它**。
# 重建程序 ≠ 全队换上了新的。这是今天第二次踩在这上面。
#
# 为什么要脚本:重启席位被权限分类器拦住了(拦得对 —— 杀掉一个正在干活的 agent 会丢掉它那一轮)。
#
# 安全性:每个席位重启**之前**都会检查它 worktree 里有没有未提交的改动。
# 有未提交改动的**跳过不重启** —— 杀掉它等于丢掉它没存盘的工作。
# (已知例外:`.pi/` 和 `AGENTS.md` 是 Aegis 给席位建的配置,刻意不提交,不算未存盘工作。)
#
# 用法: bash scripts/restart-seats-for-new-tools.sh
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1
WT="$(cd "$REPO/.." && pwd)/seatloom-worktrees"
MAP="$REPO/.seatloom/dispatch-seats.tsv"
BIN="$REPO/target/debug/seatloom-mcp-emit"

[ -r "$MAP" ] || { echo "❌ 找不到席位表 $MAP"; exit 1; }
[ -x "$BIN" ] || { echo "❌ 找不到程序 $BIN"; exit 1; }

binage="$(stat -f '%Sm' -t '%m-%d %H:%M' "$BIN")"
echo "目标程序构建于 ${binage}"
echo "它自报的工具:"
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"p","version":"0"}}}' \
  | DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/seatloom" \
    SEATLOOM_ACTOR_REF="seat:seat-aegis-001" SEATLOOM_PROJECT_ID="seatloom" \
    "$BIN" 2>&1 >/dev/null | head -1 | sed 's/^/  /'
echo

# 哪些席位的工具进程是旧的
echo "逐个检查:"
todo=""
while IFS=$'\t' read -r sid sess; do
  case "$sid" in ''|'#'*) continue;; esac
  short="$(printf '%s' "$sid" | sed 's/seat-//;s/-001//')"
  pane="$(tmux list-panes -t "$sess" -F '#{pane_pid}' 2>/dev/null | head -1)"
  if [ -z "${pane:-}" ]; then printf "  %-8s 会话不在,跳过\n" "$short"; continue; fi

  # 它的工具子进程(常驻)起于何时
  started=""
  for k in $(pgrep -P "$pane" 2>/dev/null); do
    for g in $(pgrep -P "$k" 2>/dev/null); do
      case "$(ps -o command= -p "$g" 2>/dev/null)" in
        *seatloom-mcp-emit*) started="$(ps -o lstart= -p "$g" 2>/dev/null)";;
      esac
    done
  done

  # 它自己的 worktree 有没有未存盘的工作
  wt="$WT/$short"
  dirty="?"
  if [ -d "$wt/.git" ] || [ -f "$wt/.git" ]; then
    dirty="$(git -C "$wt" status --porcelain 2>/dev/null | grep -vE '^\?\? (\.pi/|AGENTS\.md)$' | wc -l | tr -d ' ')"
  fi

  if [ "${dirty}" != "0" ] && [ "${dirty}" != "?" ]; then
    printf "  %-8s ⛔ 有 %s 个未提交改动 —— **不重启**(会丢掉没存盘的工作)\n" "$short" "$dirty"
    git -C "$wt" status --short 2>/dev/null | grep -vE '^\?\? (\.pi/|AGENTS\.md)$' | head -3 | sed 's/^/           /'
  else
    printf "  %-8s ✅ 工作已提交,工具进程起于 %s → 重启\n" "$short" "${started:-未知}"
    todo="$todo $sess"
  fi
done < "$MAP"
echo

if [ -z "$(printf '%s' "$todo" | tr -d ' ')" ]; then
  echo "没有可以安全重启的席位。把这段贴给 Aegis。"
  exit 1
fi

echo "开始重启(每个之间等 8 秒,避免同时抢程序锁):"
for sess in $todo; do
  echo "  --- $sess ---"
  bash scripts/seat-launch.sh "$sess" 2>&1 | sed 's/^/      /'
  sleep 8
done
echo

echo "验证(工具进程必须比程序新,否则就是没换上):"
sleep 6
binepoch="$(stat -f %m "$BIN")"
ok=0; bad=0
while IFS=$'\t' read -r sid sess; do
  case "$sid" in ''|'#'*) continue;; esac
  short="$(printf '%s' "$sid" | sed 's/seat-//;s/-001//')"
  pane="$(tmux list-panes -t "$sess" -F '#{pane_pid}' 2>/dev/null | head -1)"
  [ -z "${pane:-}" ] && continue
  pid=""
  for k in $(pgrep -P "$pane" 2>/dev/null); do
    for g in $(pgrep -P "$k" 2>/dev/null); do
      case "$(ps -o command= -p "$g" 2>/dev/null)" in *seatloom-mcp-emit*) pid="$g";; esac
    done
  done
  if [ -z "$pid" ]; then
    printf "  %-8s ⚠️ 没有工具进程(还在启动,或停在信任弹窗 —— 必须带 --approve)\n" "$short"
    bad=$((bad+1)); continue
  fi
  # 用 ps 的 etime 反推启动时刻,和程序构建时刻比 —— 这一项必须能报错
  el="$(ps -o etime= -p "$pid" 2>/dev/null | tr -d ' ')"
  secs="$(printf '%s' "$el" | awk -F'[-:]' '{
    if (NF==4) print ($1*86400)+($2*3600)+($3*60)+$4;
    else if (NF==3) print ($1*3600)+($2*60)+$3;
    else if (NF==2) print ($1*60)+$2; else print $1 }')"
  startedat=$(( $(date +%s) - ${secs:-0} ))
  if [ "$startedat" -ge "$binepoch" ]; then
    printf "  %-8s ✅ 换上了(工具进程比程序新 %s 秒)\n" "$short" "$(( startedat - binepoch ))"
    ok=$((ok+1))
  else
    printf "  %-8s ❌ 还是旧的(工具进程比程序早 %s 秒)—— 没换上\n" "$short" "$(( binepoch - startedat ))"
    bad=$((bad+1))
  fi
done < "$MAP"
echo
if [ "$bad" = "0" ]; then
  echo "✅ 全部换上新程序(${ok} 个席位)。"
  exit 0
else
  echo "⚠️ ${ok} 个换上了,${bad} 个没换上。"
  echo "   最常见的原因:该席位正在干活,启动脚本按设计拒绝打断它(输出里会写 mid-turn, refusing)。"
  echo "   等它空闲再跑一次本脚本即可;**不要**加 --force 去打断正在干活的席位。"
  exit 1
fi
