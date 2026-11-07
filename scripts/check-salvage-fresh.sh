#!/usr/bin/env bash
# check-salvage-fresh.sh — 抢救存档里的补丁,还对应得上工作区现在的状态吗?
#
# 为什么需要这个检查(2026-11-02 11:00):
#
#   10-31 15:00 我在 docs/coordination/salvage/README 里写下「这些补丁会过期」,
#   附了理由、附了怎么验。**然后五个小时没回来验。**
#   11:00 去验,Lyra 那份已经 stale —— 不是她的工作丢了,
#   是**我对它的描述过期了**:那行 v2→v3 已经被她提交,
#   所以那份补丁里有一块不再属于「未提交」,而清单里还写着它「有效」。
#
#   **一个过期却还标着「有效」的补丁,比没有补丁更糟。**
#   没有补丁:我知道那份工作没被保护,我会去问。
#   有过期却标着有效的补丁:我以为它被保护着,而恢复出来的是一份旧状态。
#
# 判据:每个 .patch(除 .superseded.patch)都必须能 `git apply --reverse` 贴回
# 它声明的那个工作区。贴不回去 = 它已经不对应现在的状态 = 清单在说谎。
#
# 退出码 0 = 全部对得上;1 = 有补丁过期(清单在说谎);2 = 检查自己跑不了。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SALVAGE="$REPO/docs/coordination/salvage"
WORKTREES="${SEATLOOM_WORKTREES:-$REPO/../seatloom-worktrees}"

[ -d "$SALVAGE" ] || { echo "❌ 找不到 $SALVAGE"; exit 2; }

# 从补丁内容里推它是哪个工作区的:diff 的 a/ b/ 行不带路径,
# 但每个工作区的分支名不同。改用更直接的办法 —— 逐个工作区试贴。
# 不猜,试完在输出里说清是哪一个,这样「对得上」是一个量出来的结论。
FAIL=0
CHECKED=0

shopt -s nullglob
for patch in "$SALVAGE"/*.patch; do
  name="$(basename "$patch")"
  case "$name" in
    *.superseded.patch)
      echo "  ⊘ $name —— 已标为取代,不检查"
      continue
      ;;
  esac

  CHECKED=$((CHECKED + 1))
  matched=""
  UNREADABLE=""

  # 在每一个 worktree 里试一次反向应用。
  # 🔴 必须能区分两种失败,而我 2026-11-07 16:00 正好把它们混过一次:
  #    git apply 打不开补丁文件 = 退出 128("can't open patch")
  #    补丁贴不上当前树   = 退出 1("patch does not apply")
  #    我拿一个相对路径配 git -C 跑,于是文件根本找不到,读出 128,
  #    就写下「这份快照不忠实」—— 补丁本身是好的,是我的调用方式是坏的。
  #    **打不开文件不等于贴不上。** 把两者混同,会让一份忠实的快照被判成过期。
  for d in "$WORKTREES"/*; do
    [ -d "$d/.git" ] || [ -f "$d/.git" ] || continue
    if git -C "$d" apply --check --reverse "$patch" >/dev/null 2>&1; then
      matched="$(basename "$d")"
      break
    elif [ ! -r "$patch" ]; then
      UNREADABLE="$d"
      break
    fi
  done

  if [ -n "$matched" ]; then
    echo "  ✓ $name → 贴回 $matched 的工作区(对得上)"
  elif [ -n "$UNREADABLE" ]; then
    echo "  ✗ $name → 读不到这个补丁文件本身。这不是「贴不回去」,是检查自己跑不了。"
    FAIL=2
  else
    echo "  ✗ $name → 在任何一个工作区都贴不回去。它已经过期,而清单还写着它有效。"
    echo "      修:cd 对应工作区 && git diff > 新补丁,然后把这份改名成 .superseded.patch"
    FAIL=1
  fi
done
shopt -u nullglob

# 反过来也要查:有没有哪个席位工作区有未提交的改动、却没有对应补丁?
# 只查这一半会漏掉「新出现的未提交工作」—— 而它是最该被保护的那一类。
echo
echo "── 反向:有未提交工作但没有补丁的工作区 ──"
UNCOVERED=0
for d in "$WORKTREES"/*; do
  [ -d "$d/.git" ] || [ -f "$d/.git" ] || continue
  wt="$(basename "$d")"
  # 只数「被修改的已跟踪文件」+「新增的未跟踪文件」,而且要排掉构建产物。
  #
  # 🔴 我第一版用 git status --porcelain | wc -l,于是把
  # `d2preview` 工作区里的两个 node_modules **符号链接**算成「未提交工作」,
  # 报了一个假阳性。它们指向主检出的 node_modules,是构建残留,不是任何人的活。
  # **一个会把符号链接和构建产物算成「未提交工作」的检查,会训练人忽略它。**
  n="$(git -C "$d" status --porcelain 2>/dev/null \
        | grep -vE '^\?\? +(node_modules|ui/node_modules|dist|target|\.next)/?$' \
        | wc -l | tr -d ' ')"
  [ "$n" = "0" ] && continue
  # 有没有任何补丁能贴回这个工作区?
  covered=0
  shopt -s nullglob
  for patch in "$SALVAGE"/*.patch; do
    case "$(basename "$patch")" in *.superseded.patch) continue ;; esac
    git -C "$d" apply --check --reverse "$patch" >/dev/null 2>&1 && { covered=1; break; }
  done
  shopt -u nullglob
  if [ "$covered" = "0" ]; then
    echo "  ✗ $wt 有 $n 个未提交文件,但没有任何补丁覆盖它。"
    UNCOVERED=$((UNCOVERED + 1))
  else
    echo "  ✓ $wt 的 $n 个未提交文件有补丁覆盖"
  fi
done

echo
if [ "$FAIL" = "0" ] && [ "$UNCOVERED" = "0" ]; then
  echo "✅ 抢救存档是当前状态:查了 $CHECKED 个补丁,未提交工作全部有着落。"
  exit 0
fi

if [ "$FAIL" != "0" ]; then
  if [ "$FAIL" = "2" ]; then
    echo "❌ 检查自己跑不了:有补丁读不到。这不是「补丁过期」,是这一轮没有量到东西。"
  else
    echo "❌ 有补丁过期而清单仍标着有效。"
  fi
fi
if [ "$UNCOVERED" != "0" ]; then
  echo "❌ 有 $UNCOVERED 个工作区的未提交工作没有任何补丁覆盖。"
fi
# 🔴 检查自己跑不了要报 2,不能报 1。1 的含义是「清单在说谎」,
#    而 2 的含义是「这一轮什么都没量到」。把 2 压成 1,
#    会让一次自己失败被读成一次外部发现 —— 而我 16:00 刚犯过反过来的错。
[ "$FAIL" = "2" ] && exit 2
exit 1
