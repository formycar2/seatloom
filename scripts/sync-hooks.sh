#!/usr/bin/env bash
# sync-hooks.sh — 把 .githooks/ 里的钩子装到 git 的公共目录,并校验两者一致。
#
# 为什么需要这个脚本(2026-11-01 05:00,Lyra 发现):
#
#   她 mutation-tested 我 04:00 写的「glob 不等于所有权」检查,两个方向都过了:
#     方向一:复刻她那次 glob 失误 → 检查触发、点名文件、给出该怎么做。✅
#     方向二:只提交我自己的文件、knowledge/、scripts/ → 干净通过。✅
#   然后她找到缺口:**那个检查没有装在她犯错的 worktree 里。**
#
#   我把 core.hooksPath 设成 .githooks(相对路径)。worktree 共享 git dir,
#   所以配置也共享 —— 但 .githooks/ 这个**目录只存在于主检出的工作区**,
#   它在我这条分支上,不在她的分支上。而 **git 对「钩子文件不存在」的处理是
#   静默跳过**:她在一个临时仓库里把 core.hooksPath 指向一个不存在的目录,
#   暂存别人的文件,提交 —— 提交成功,退出 0,无警告无报错。
#
#   她的结论:「我为阻止我犯的那个错而写的守卫,**不会在我犯那个错的地方阻止它**。
#   它保护主检出。在 .githooks/ 出现在那条分支上之前,它不保护任何席位工作区。」
#
#   🔴 而这是同一族的第十三次:钩子在有它的那台机器上验过了,
#      「glob 失误现在有守卫了」这个说法对**一个**检出成立、对**五个**不成立,
#      而两种状态的 git 输出完全一样:没输出,退出 0。
#
# 设计:
#   源头放 .githooks/(进版本库,可追可复盘);
#   安装副本放 git 公共目录的 hooks/ 下(所有 worktree 共享,包括席位的);
#   core.hooksPath **不设** —— 设了相对的,在别人的 worktree 里就是静默跳过。
#
# 用法: bash scripts/sync-hooks.sh           # 干跑:只报差异,不动任何东西
#       bash scripts/sync-hooks.sh --apply   # 装上,并校验一致

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

SRC="$REPO/.githooks"
if [ ! -d "$SRC" ]; then
  echo "❌ 找不到 $SRC —— 没有可装的钩子。" >&2
  exit 1
fi

# 公共目录:所有 worktree 共享它。装在别处(比如某个 worktree 的 .git/hooks)
# 只保护那一个检出,而席位的工作区全是 linked worktree。
COMMON="$(git rev-parse --git-common-dir 2>/dev/null)"
if [ -z "$COMMON" ]; then
  echo "❌ 取不到 git 公共目录 —— 这里不是 git 仓库?" >&2
  exit 1
fi
# rev-parse 可能返回相对路径,统一成绝对路径,免得后面 cd 之后失效。
case "$COMMON" in
  /*) : ;;
  *)  COMMON="$REPO/$COMMON" ;;
esac
DEST="$COMMON/hooks"

echo "源头:   $SRC"
echo "安装到: $DEST"
echo

# core.hooksPath 必须是空的。设成相对的路径,在席位的 worktree 里会静默跳过 ——
# 而那正是这次要修的洞。
cur="$(git config core.hooksPath 2>/dev/null || true)"
if [ -n "$cur" ]; then
  if [ "$APPLY" = "1" ]; then
    git config --unset core.hooksPath
    echo "✓ 已取消 core.hooksPath(原来是 '$cur')"
    echo "  相对路径的 hooksPath 在别人的 worktree 里指向一个不存在的目录,"
    echo "  而 git 对钩子缺失的处理是静默跳过。"
  else
    echo "⚠️  core.hooksPath 现在是 '$cur' —— 需要取消。"
    echo "   相对路径的 hooksPath 在别人的 worktree 里指向不存在的目录,"
    echo "   而 git 对钩子缺失静默跳过:提交成功、退出 0、无警告。"
  fi
  echo
fi

changed=0
for f in "$SRC"/*; do
  [ -f "$f" ] || continue
  name="$(basename "$f")"
  if [ -x "$f" ]; then mode="可执行"; else mode="不可执行(装了也不会跑)"; fi
  if [ -f "$DEST/$name" ] && cmp -s "$f" "$DEST/$name"; then
    printf "  ✓ %-14s 已一致(%s)\n" "$name" "$mode"
  else
    printf "  ✗ %-14s %s\n" "$name" "$([ -f "$DEST/$name" ] && echo '内容不一致' || echo '未安装')"
    if [ "$APPLY" = "1" ]; then
      mkdir -p "$DEST"
      cp "$f" "$DEST/$name"
      chmod +x "$DEST/$name"
      changed=$((changed + 1))
      printf "      → 已装上\n"
    fi
  fi
done
echo

if [ "$APPLY" = "0" ]; then
  echo "干跑结束。加 --apply 真的装。"
  exit 0
fi

echo "── 校验:装完之后,每一个钩子都必须真的会失败 ──"
# 一个装了但跑不起来的钩子,比没装更糟:它让「有守卫」变成一个说法。
# 所以这里不是 cmp 一遍就完事,是真的跑一次,看它能不能返回非零。
probe="$(mktemp -d)"
trap 'rm -rf "$probe"' EXIT
mkdir -p "$probe"
(
  cd "$probe" || exit 1
  git init -q .
  # 🔴 必须把刚装好的钩子拷进这个探针仓库。
  # 我第一版没拷,于是在一个**根本没有钩子**的新仓库里跑,
  # 量到「拒绝别人的文件: exit 0」还以为是自己装错了。
  # 一个在没装钩子的仓库里跑的钩子自检,量的是 git 的默认行为,不是钩子的行为。
  mkdir -p .git/hooks
  cp "$DEST"/pre-commit .git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit
  # 造一个「别人的文件」,看钩子会不会拒绝。
  mkdir -p docs/coordination/tasks/mira
  echo "someone was mid-edit" > docs/coordination/tasks/mira/probe.md
  git add -A
  git -c user.email=probe@probe -c user.name=probe commit -qm probe >/dev/null 2>&1
  echo "  拒绝别人的文件: exit $? (要 1)"
  # 🔴 两次测试之间必须把别人的文件**删掉**,不只是 reset。
  # 我第一版用 git reset + git checkout -- .,但那个文件本来是未跟踪的,
  # checkout 不会删未跟踪文件;于是第二次 git add -A 又把它扫进去了,
  # 量到「放行我自己的文件: exit 1」。
  # 一个探针的残留会让下一个探针测到上一个探针的东西。
  git reset -q
  rm -f docs/coordination/tasks/mira/probe.md
  mkdir -p docs/coordination/tasks/aegis
  echo "mine" > docs/coordination/tasks/aegis/mine.md
  # 只点名我自己的文件,不用 -A —— 那正是这条规则本身。
  git add docs/coordination/tasks/aegis/mine.md
  git -c user.email=probe@probe -c user.name=probe commit -qm probe2 >/dev/null 2>&1
  echo "  放行我自己的文件: exit $? (要 0)"
)

echo
if [ "$changed" -gt 0 ]; then
  echo "✅ 装了 $changed 个钩子。所有 linked worktree(含五个席位工作区)现在都受保护。"
else
  echo "✅ 全都已一致,没有要装的。"
fi
