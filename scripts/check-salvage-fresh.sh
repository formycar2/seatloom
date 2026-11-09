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

# 自检:git 到底能不能用。没有这一探,一个坏掉的 git 会让**每一份**补丁都"贴不回去",
# 于是检查报「有三份补丁过期」—— 而那三份都是好的,坏的是仪器。
# 已实测(2026-11-10):git rev-parse --git-dir 在好 git 上退 0,在坏 git 上退 128,
# 而 git apply --check --reverse 在「贴得上」时也退 1,在「贴不上」时退 128 ——
# 单看 apply 的退出码分不出「它说不行」和「它跑不了」。先探一次,把后者变成 exit 2。
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "UNCHECKABLE: git rev-parse --git-dir 失败 —— 检查自己跑不了,这不是『补丁过期』。" >&2
  echo "              一个坏掉的 git 会让每一份补丁都贴不回去,然后被报成过期。" >&2
  exit 2
fi

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
    # 贴不回去有三种完全相反的含义,而旧版对三种打印同一条修法:
    #   1. 工作长大了/还在写  → 该重抓补丁
    #   2. 工作已经提交       → 该改名成 .superseded.patch
    #   3. 工作真的丢了       → 该恢复
    # 第一种是「现在有未提交的活」,后两种是「工作区干净」。这一半是可判的,
    # 已实测:抓一份补丁后再改一行 → dirty=1 且补丁贴不回;把那行提交 →
    # dirty=0 且补丁又能贴回。所以 dirty + 贴不回 = 第一种,clean + 贴不回 =
    # 第二或第三种。
    # 后两种分不开,原因是补丁**不记录它取自哪个提交**:它只记录两个文件
    # 系统状态的差异,而那个状态不可寻址。要分开就得让补丁带基线提交,
    # 那是改一个别的席位也在写的数据格式,不是这次能替张先生定的。
    # 所以这里只说能说的那一半,剩下的明说分不开,不猜。
    dirty_list=""
    for d in "$WORKTREES"/*; do
      [ -d "$d/.git" ] || [ -f "$d/.git" ] || continue
      n=$(git -C "$d" status --porcelain 2>/dev/null | wc -l | tr -d ' ')
      [ "$n" -gt 0 ] && dirty_list="$dirty_list $(basename "$d"):$n"
    done
    echo "  ✗ $name → 在任何一个工作区都贴不回去。它已经过期,而清单还写着它有效。"
    if [ -n "$dirty_list" ]; then
      echo "      可判的那一半:还有未提交的活(${dirty_list# })—— 工作长大了,该重抓:"
      echo "        cd 对应工作区 && git diff > 新补丁,然后把这份改名成 .superseded.patch"
    else
      echo "      分不开的那一半:所有工作区都干净。它可能是「工作已提交」(该改名"
      echo "        .superseded.patch),也可能是「工作真的丢了」(该恢复) —— 补丁不记录"
      echo "        它取自哪个提交,所以这两种我分不开,不猜。请人看一眼。"
    fi
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
