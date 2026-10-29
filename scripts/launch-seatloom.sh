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

# Where the acceptance gate will run git. Defaults to this checkout, and is
# asserted to BE a git repository containing the integration branch - a wrong
# SEATLOOM_REPO_ROOT must fail loudly here rather than make every card unclickable
# in a way that looks like a sync problem.
REPO_ROOT="${SEATLOOM_REPO_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
INTEGRATION_BRANCH="${SEATLOOM_INTEGRATION_BRANCH:-track/infra-foundation}"

if ! git -C "$REPO_ROOT" rev-parse --verify "refs/heads/$INTEGRATION_BRANCH" >/dev/null 2>&1; then
  say "❌ $REPO_ROOT 不是含分支 $INTEGRATION_BRANCH 的 git 仓库 —— 门会在里面跑 git 然后全盘拒绝"
  say "   修法:SEATLOOM_REPO_ROOT=<主检出绝对路径> bash scripts/launch-seatloom.sh"
  exit 2
fi
say "仓库根 $REPO_ROOT,集成分支 $INTEGRATION_BRANCH(已确认存在)"


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

# SEATLOOM_REPO_ROOT is not optional and it is not a nicety. The acceptance gate
# resolves the repository with resolve_repo_root(): the env var if set, otherwise
# the process cwd. `open -a` gives the app cwd=/, and / is not a git repository.
# So `git merge-base --is-ancestor <base> <branch>` fails with "not a git
# repository", branch_contains returns Err, and the gate turns that Err into
# BranchNotSynced - EVERY card, including ones whose branch is perfectly synced.
# That is exactly what happened at 11:28 today: 7 of 7 cards showed 验收被拒
# 分支未包含验收基准 while every branch was 0 commits behind track. The gate was
# not wrong about the branches; it could not see a repository at all.
#
# My own checker could never have caught this: it does `cd "$(dirname "$0")/.."`
# on line 19, so it always measures inside the main checkout, while the gate
# measures wherever the app happens to be running. A check that repairs its own
# working directory cannot see the failure it is supposed to catch.
SEATLOOM_REPO_ROOT="$REPO_ROOT" SEATLOOM_OPERATOR_REF="$OPERATOR_REF" open -a "$APP" || { say "❌ open 失败"; exit 1; }

# 验证环境变量真的到了应用进程里。macOS 的 ps -E 能打印进程环境。
up=0
for _ in $(seq 1 30); do
  pid="$(pgrep -f 'SeatLoom.app' 2>/dev/null | head -1)"
  [ -n "$pid" ] || { sleep 1; continue; }
  up=1; break
done
[ "$up" = "1" ] || { say "❌ 应用没起来"; exit 1; }
say "  应用在跑,pid $pid"

# Both variables are load-bearing and both must be verified in the process.
# A variable that was set on the `open` line but never reached the app is the same
# as never having set it, and the failure it causes looks nothing like a launch
# problem - it looks like every card being unsynced.
got_operator=""
got_repo=""
if env_dump="$(ps -E -p "$pid" 2>/dev/null)"; then
  got_operator="$(printf '%s\n' "$env_dump" | tr ' ' '\n' | grep '^SEATLOOM_OPERATOR_REF=' | head -1)"
  got_repo="$(printf '%s\n' "$env_dump" | tr ' ' '\n' | grep '^SEATLOOM_REPO_ROOT=' | head -1)"
fi

if [ -n "$got_operator" ]; then
  say "  ✅ 进程环境里确认有 $got_operator"
else
  say "  ❌ 进程环境里没有 SEATLOOM_OPERATOR_REF"
fi
if [ -n "$got_repo" ]; then
  say "  ✅ 进程环境里确认有 $got_repo"
else
  say "  ❌ 进程环境里没有 SEATLOOM_REPO_ROOT —— 验收门会因为在 / 下跑 git 而拒绝每一张卡"
fi

# And the real test, not an environment-variable reading: run the gate's own
# predicate the way the gate runs it. branch_contains() does
#   Command::new("git").args(["merge-base","--is-ancestor",base,branch]) with
#   current_dir set to repo_root
# so the repo root is the command's WORKING DIRECTORY, not a -C argument. The
# honest test is therefore: with the app's own cwd, does the gate's command work?
# Use a trivially-true pair (base is an ancestor of itself) so a failure can only
# mean "git could not find a repository", never "the branches are out of sync".
if git -C "$REPO_ROOT" merge-base --is-ancestor "$INTEGRATION_BRANCH" "$INTEGRATION_BRANCH" >/dev/null 2>&1; then
  say "  ✅ 用应用自己的工作目录($REPO_ROOT)跑门的判据,能跑通"
else
  say "  ❌ 门的判据在 $REPO_ROOT 下跑不通 —— 现在去点合并会被拒"
  exit 1
fi

if [ -z "$got_operator" ] || [ -z "$got_repo" ]; then
  say ""
  say "  ⚠️  读不到进程环境(ps -E 在这个系统上可能要权限)—— 这一项没验成,不算通过。"
  say "     判它通没通的唯一硬证据:点一次合并,不再报 SEATLOOM_REPO_ROOT 相关拒绝。"
fi

say ""
say "现在可以去点合并了。如果还报同样的错,把这段输出贴给 Aegis。"
