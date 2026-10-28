#!/usr/bin/env bash
# launch-seatloom.sh — 用操作者身份启动 SeatLoom。
#
# 为什么需要它(2026-10-16 张先生点合并时撞到的):
#   点「接受」要走 cmd_accept_delivery,它第一件事是取操作者身份:
#     std::env::var("SEATLOOM_OPERATOR_REF"),取不到就整条命令拒绝,
#     理由是「验收是用户的动作,身份不许当场编」—— 这个设计是对的。
#   但项目里从来没有任何地方设置过这个变量:没有脚本、没有启动器、
#   没有文档写该设成什么。而从启动台/Finder 点的应用不带这个环境变量,
#   所以每一次点合并都必然失败。**功能在,但没有给人配置它的路。**
#
# 用法:
#   bash scripts/launch-seatloom.sh
#
# 身份取 human:supervisor —— 和账本里张先生既有的写法一致
# (dispatches.issued_by、canonical_events.actor_ref 里都是它),
# 也是代码自己往工具进程注入身份时用的同一个值(dispatch_cmds.rs)。
set -uo pipefail

OPERATOR_REF="${SEATLOOM_OPERATOR_REF:-human:supervisor}"
APP="/Applications/SeatLoom.app"

say(){ printf '%s\n' "$*"; }

[ -d "$APP" ] || { say "❌ 找不到 $APP —— 先用 prepare-verification-build.sh 装一份"; exit 1; }

# 已经在跑的那份没有这个环境变量,带着它才管用,所以先停掉重起。
if pgrep -f "SeatLoom.app" >/dev/null 2>&1; then
  say "停掉正在跑的那份(它是从启动台点的,没有操作者身份)…"
  pkill -f "SeatLoom.app" 2>/dev/null
  for _ in $(seq 1 15); do pgrep -f "SeatLoom.app" >/dev/null 2>&1 || break; sleep 1; done
  pgrep -f "SeatLoom.app" >/dev/null 2>&1 && { say "❌ 旧的那份停不掉,不启动"; exit 1; }
fi

say "以操作者身份 $OPERATOR_REF 启动…"
SEATLOOM_OPERATOR_REF="$OPERATOR_REF" open -a "$APP" || { say "❌ open 失败"; exit 1; }

# 验证环境变量真的到了应用进程里。macOS 的 ps -E 能打印进程环境。
up=0
for _ in $(seq 1 30); do
  pid="$(pgrep -f 'SeatLoom.app' 2>/dev/null | head -1)"
  [ -n "$pid" ] || { sleep 1; continue; }
  up=1; break
done
[ "$up" = "1" ] || { say "❌ 应用没起来"; exit 1; }
say "  应用在跑,pid $pid"

# ps -E 在某些 macOS 版本上要 root,拿不到就明说拿不到,不要假装验过。
if env_line="$(ps -E -p "$pid" 2>/dev/null | tr ' ' '\n' | grep '^SEATLOOM_OPERATOR_REF=' | head -1)"; then
  [ -n "$env_line" ] || env_line=""
fi
if [ -n "${env_line:-}" ]; then
  say "  ✅ 进程环境里确认有 $env_line"
else
  say "  ⚠️  读不到进程环境(ps -E 在这个系统上可能要权限)—— 这一项没验成,不算通过。"
  say "     判它通没通的唯一硬证据:点一次合并,不再报 SEATLOOM_OPERATOR_REF is not set。"
fi

say ""
say "现在可以去点合并了。如果还报同样的错,把这段输出贴给 Aegis。"
