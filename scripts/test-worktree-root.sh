#!/usr/bin/env bash
# test-worktree-root.sh — 证明 worktree-root.sh 在各种形状下都答对,并且答不出时说答不出。
#
# 为什么需要这个测试(2026-11-07):
#   席位工作区根的老写法是一句目录名猜测。它在主检出里对,在席位自己的工作区里错,
#   于是 prepare-sandbox-build.sh --build-only 从席位工作区跑不起来,报的是
#   「这条分支不是任何工作区检出的分支」—— 把人指向错误方向的失败。
#   换成一个 resolver 之后,必须有测试钉住它,否则下一个改法会静悄悄地退化。
#
# 🔴 第一版的 resolver 就退化过一次,而且退化得很隐蔽:
#    数工作区数量用的是**前缀**匹配(case "$wt" in "$parent"/*),于是
#    .../GitHub/seatloom-worktrees/flux 也匹配父目录 .../GitHub,
#    数出来 9 比 8 大,选中的是 GitHub —— 差一层,而且稳定地错。
#    所以这个测试里有一条专门钉「必须按直接父目录数」。
#
# 做法:全部用 fixture 里的假仓库,绝不碰真实的工作区布局。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RESOLVER="$REPO/scripts/worktree-root.sh"
[ -f "$RESOLVER" ] || { echo "❌ 找不到 $RESOLVER"; exit 1; }

FAIL=0
pass(){ echo "  ✓ $1"; }
die(){ echo "  ✗ $1"; FAIL=1; }
# 🔴 路径要过一次 `cd ... && pwd` 再比。macOS 的 mktemp 给出 /var/folders/...,
#    那是指向 /private/var/folders/... 的符号链接;git 会解析它,而测试里的字面量不会。
#    第一版直接比字面量,于是六条里五条红,而 resolver 其实全对 ——
#    **一个拿未解析路径当期望值的测试,会把正确的实现报成坏的。**
ROOT="$(cd "$(mktemp -d)" && pwd)"
cleanup(){ rm -rf "$ROOT"; }
trap cleanup EXIT
REAL_GIT="$(command -v git)"

echo "── 1. 从主检出跑:答出挂着席位工作区的那一个目录 ──"
# 形状和真实部署一致:主检出的父目录只挂它自己,席位根下面挂好几个。
MAIN="$ROOT/main"
SEATS="$ROOT/seatloom-worktrees"
mkdir -p "$MAIN" "$SEATS"
( cd "$MAIN" && "$REAL_GIT" init -q -b track/main . \
  && "$REAL_GIT" config user.email t@t.local && "$REAL_GIT" config user.name t \
  && echo hi > f && "$REAL_GIT" add . \
  && "$REAL_GIT" -c core.hooksPath=/dev/null commit -q --no-verify -m base ) >/dev/null 2>&1
for name in seat-a seat-b; do
  mkdir -p "$SEATS/$name"
  ( cd "$MAIN" && "$REAL_GIT" worktree add -q "$SEATS/$name" -b "packet/$name/test" ) >/dev/null 2>&1
done
# 主检出的 HEAD 在 track/main,分支不能重复检出,所以席位用各自的分支。
# same_path 两边都解析成**git 会用的那个形式**再比。
# macOS 上 /var 是指向 /private/var 的符号链接,而 git worktree list 给出的是解析后的
# 真实路径。测试里的字面量来自 mktemp(未解析),所以直接比一定不等 ——
# 第一版因此六条里五条红,而 resolver 其实全对。用 git 自己认path的方式归一:
# 让 git 告诉我们它认为这个路径是什么。
same_path(){
  local a b
  a="$(cd "$1" 2>/dev/null && pwd -P)"
  b="$(cd "$2" 2>/dev/null && pwd -P)"
  [ -n "$a" ] && [ "$a" = "$b" ]
}
OUT="$( cd "$MAIN" && . "$RESOLVER" && resolve_worktree_root "$MAIN" 2>&1 )"
RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC:$OUT"
same_path "$OUT" "$SEATS" && pass "答出 $SEATS" || die "答成了 '$OUT',期望 '$SEATS'"

echo ""
echo "── 2. 从席位自己的工作区跑:答同一个目录(老写法在这里错)──"
OUT2="$( cd "$SEATS/seat-a" && . "$RESOLVER" && resolve_worktree_root "$SEATS/seat-a" 2>&1 )"
RC2=$?
[ "$RC2" = "0" ] && pass "退出 0" || die "退出 $RC2:$OUT2"
same_path "$OUT2" "$SEATS" && pass "从席位工作区也答出 $SEATS" || die "答成了 '$OUT2' —— 老写法在这里会拼出双层目录"
# 老写法在这个形状下给出的答案,必须和 resolver 不同 —— 否则这一条没有区分力。
OLD="$(cd "$SEATS/seat-a/.." && pwd)/seatloom-worktrees"
[ "$OLD" != "$SEATS" ] && pass "老写法在这里确实会答错($OLD)—— 这一条有区分力" || die "老写法也答对了,这条用例没有区分力"

echo ""
echo "── 3. 路径里有空格:必须答对,不能被 glob 或不加引号的展开弄碎 ──"
SPACED="$ROOT/seat looms"
mkdir -p "$SPACED"
# 在新根下再造两个席位工作区。git worktree list 会同时列出旧的和新的。
( cd "$MAIN" && "$REAL_GIT" worktree add -q "$SPACED/seat-x" -b packet/seat-x/test ) >/dev/null 2>&1
( cd "$MAIN" && "$REAL_GIT" worktree add -q "$SPACED/seat-y" -b packet/seat-y/test ) >/dev/null 2>&1
export SEATLOOM_WORKTREE_ROOT="$SPACED"
OUT3="$( cd "$SPACED/seat-x" && . "$RESOLVER" && resolve_worktree_root "$SPACED/seat-x" 2>&1 )"
unset SEATLOOM_WORKTREE_ROOT
RC3=$?
[ "$RC3" = "0" ] && pass "带空格的路径退出 0" || die "带空格的路径退出 $RC3:$OUT3"
same_path "$OUT3" "$SPACED" && pass "带空格的路径答对了" || die "答成了 '$OUT3'"

echo ""
echo "── 4. 必须按**直接父目录**数,不能按前缀数 ──"
# 第一版用前缀匹配,于是 .../seatloom-worktrees/flux 也匹配父目录 ...,
# 数出来外层更大,选中的是外层 —— 差一层,而且稳定地错。
# 构造一个正好能触发它的形状:外层挂 1 个(主检出),内层挂 3 个。
INNER="$ROOT/outer/inner"
mkdir -p "$INNER"
( cd "$MAIN" && "$REAL_GIT" worktree add -q "$INNER/w1" -b packet/w1/test ) >/dev/null 2>&1
( cd "$MAIN" && "$REAL_GIT" worktree add -q "$INNER/w2" -b packet/w2/test ) >/dev/null 2>&1
( cd "$MAIN" && "$REAL_GIT" worktree add -q "$INNER/w3" -b packet/w3/test ) >/dev/null 2>&1
OUT4="$( cd "$INNER/w1" && . "$RESOLVER" && resolve_worktree_root "$INNER/w1" 2>&1 )"
# 前缀计数下,外层 $ROOT/outer 会拿到 4(inner 下的三个 + 自己),内层拿到 3,
# 于是选中外层。直接父目录计数下,内层 3 > 外层 1,选中内层。
same_path "$OUT4" "$INNER" && pass "选中直接父目录挂最多的那一个($INNER)" || die "答成了 '$OUT4' —— 像是按前缀数的,这正是第一版的 bug"

echo ""
echo "── 5. 显式指定的根优先,且不存在时必须报错 ──"
# 🔴 `VAR=x . file` 不会把 VAR 导出给被 source 的文件里的函数 ——
#    那个前缀只对**外部命令**生效。所以这里必须先 export 再 source。
#    第一版写成一行,结果 resolver 看到的是空的 SEATLOOM_WORKTREE_ROOT,
#    走了 git worktree list 那条路,于是「显式优先」这条一直红。
export SEATLOOM_WORKTREE_ROOT="$SEATS"
OUT5="$( cd "$MAIN" && . "$RESOLVER" && resolve_worktree_root "$MAIN" 2>&1 )"
unset SEATLOOM_WORKTREE_ROOT
same_path "$OUT5" "$SEATS" && pass "SEATLOOM_WORKTREE_ROOT 优先" || die "显式指定没优先:$OUT5"
export SEATLOOM_WORKTREE_ROOT="$ROOT/nope"
OUT6="$( cd "$MAIN" && . "$RESOLVER" && resolve_worktree_root "$MAIN" 2>&1 )"
RC6=$?
unset SEATLOOM_WORKTREE_ROOT
[ "$RC6" != "0" ] && pass "显式指向不存在的目录时非零退出" || die "显式指向不存在的目录却退出 0"
printf '%s\n' "$OUT6" | grep -q "不存在" && pass "明说那个目录不存在" || die "没说为什么失败"

echo ""
echo "── 6. git 读不到时必须非零,不能猜一个目录交差 ──"
# 一个不是 git 仓库的目录,而且老写法猜的路径也不存在。
NOREPO="$ROOT/plain"
mkdir -p "$NOREPO"
OUT7="$( cd "$NOREPO" && SEATLOOM_WORKTREE_ROOT="" . "$RESOLVER" && resolve_worktree_root "$NOREPO" 2>&1 )"
RC7=$?
[ "$RC7" != "0" ] && pass "读不到 git 时非零退出" || die "读不到 git 却退出 0 —— 它会猜一个目录交差"
printf '%s\n' "$OUT7" | grep -q "定不下\|没给出" && pass "明说定不下来" || die "没说为什么定不下来"

echo ""
if [ "$FAIL" = "0" ]; then
  echo "✅ 全部通过"
  exit 0
fi
echo "❌ 有失败项"
exit 1
