#!/usr/bin/env bash
# clear-stale-node-options.sh — 清掉 tmux 服务器里那个指向不存在文件的 NODE_OPTIONS。
#
# 什么坏了(2026-10-17 18:41 实测,不是推测):
#   tmux 服务器(pid 2193)的环境里有
#     NODE_OPTIONS=--require=/Users/.../.agentd/runtime/cache/probe-runtime-node/loader.cjs
#   而 ~/.agentd/ **整个目录已经被删**。node 于是给每个经 tmux 起的子进程都要求
#   预加载一个不存在的文件,报:
#     Error: Cannot find module '.../loader.cjs'  Require stack: - internal/preload
#   症状:saih mcp list 坏、codemode 工具沙箱坏 —— 一切新起的 node 命令全坏。
#
# 为什么七个席位没坏:
#   他们的 saih 进程是 16:53–17:16 起的,环境里没有这个变量(ps -E 实测计数 0)。
#   seatloom MCP 也不坏 —— 它是 Rust 二进制(Mach-O arm64),不经 node。
#   所以坏的是「新起的 node 命令」这一层,不是席位,也不是 MCP。
#
# 为什么 .zshrc 那两行不是源头:
#   .zshenv 和 .zshrc 都是 `[ -f ... ] && . ...` —— 文件不存在时静默跳过,
#   根本不设变量。实测我这条 bash 的父链:tmux(2193) 有 → zsh(2194) 没有 → saih 没有。
#   所以变量是从 tmux 服务器继承的,它起来时 ~/.agentd 还在。
#
# 为什么是脚本:动 tmux 服务器的全局环境会影响所有席位,这一步归张先生。
#
# 先干跑看状态,再真修。
#
# 用法:
#   bash scripts/clear-stale-node-options.sh            # 干跑,只报不动
#   bash scripts/clear-stale-node-options.sh --apply    # 真修
set -uo pipefail

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

TMUX_PID="$(pgrep -x tmux 2>/dev/null | head -1)"
[ -n "$TMUX_PID" ] || { echo "❌ 找不到 tmux 服务器进程"; exit 1; }

echo "=== tmux 服务器 pid=$TMUX_PID ==="
echo "--- 它环境里的 NODE_OPTIONS ---"
ps -E -p "$TMUX_PID" 2>/dev/null | tr ' ' '\n' | grep "NODE_OPTIONS" | sed 's/^/  /' || echo "  (没有)"

echo "--- 它指的那个文件在吗 ---"
LEFTOVER="$(ps -E -p "$TMUX_PID" 2>/dev/null | tr ' ' '\n' | grep "NODE_OPTIONS" | sed 's/.*--require=//; s/[]"'"'"']//g' | head -1)"
if [ -z "$LEFTOVER" ]; then
  echo "  ✅ tmux 环境里没有 NODE_OPTIONS —— 没东西要清"
  exit 0
fi
echo "  指向: $LEFTOVER"
if [ -e "$LEFTOVER" ]; then
  echo "  ✅ 文件存在,不用清(但值得看一眼是不是该有)"
  exit 0
fi
echo "  ❌ 文件不存在 —— 这就是 node 全坏的原因"

if [ "$APPLY" != "1" ]; then
  cat <<NOTE

干跑结束,没动任何东西。真修要重启 tmux 服务器,而那会断开所有席位的面板。
真修方式(二选一,都不是我能替你决定的):

  一、只影响新开的窗口(不动席位),在你自己的终端里:
       unset NODE_OPTIONS
       然后从那个终端起 node 命令。**已存在的 tmux 窗口仍然带着旧变量。**

  二、彻底清掉,重启 tmux 服务器:
       tmux kill-server
       然后重跑 scripts/seat-launch.sh --all 把七个席位拉起来。
      🔴 这会断开所有席位面板。他们跑在 saih 上,重启 = 重新起进程,
         当前那一步可能丢。**挑全队空闲的时候做。**
      重启后席位会继承干净的环境。

NOTE
  exit 0
fi

echo
echo "🔴 真修需要重启 tmux 服务器,这一步我没在这个脚本里做 —— 那会断开所有席位。"
echo "   方式见上面的「二」。确认全队空闲后再动。"
