#!/usr/bin/env bash
# 两个方向都验 pre-commit 的卡片检查。2026-11-07 22:00 建。
#
# 为什么必须有必败样本:上一版守卫问的是被张先生推翻的判据
# («分支是否包含 FUTURE»),而它打印的理由关于 accept 路径是假的。
# 一个从不失败的守卫,比没有守卫更糟 —— 它把「我没查」变成「我查过」。
#
# 方向一:两张真卡必须放过(真实合并干净)。
# 方向二:一个真会冲突的分支必须被拦。
#
# 跑法:bash scripts/test-precommit-card-check.sh
set -u

REPO="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom"
HOOK="$REPO/.git/hooks/pre-commit"
FIX=$(mktemp -d /tmp/precommit-test.XXXXXX)

pass(){ echo "  ✅ $1"; }
die(){ echo "  ❌ $1" >&2; }

# 🔴 清理必须在定义 trap 之前就把逻辑写对,而且要在**任何**改动之前先验锚点。
#    上一版我把锚点检查放在改动之后,于是它一失败,已经 add 的改动就留在工作区,
#    下一次运行读到的是上一轮的残留,于是同一个保护连续两次误报。
#    **一个测试自己留下的脏状态,会让下一轮看起来像是守卫的错。**
# 🔴 顺序:必须先 `git reset`(把索引退回 HEAD),再 checkout。
#    上一版先 checkout 再 reset —— checkout 从**索引**恢复,而那一刻索引里
#    正是被改的那份,于是它把修改"恢复"回了工作区;随后的 reset 只动索引
#    不动工作区,改动留下。等 EXIT trap 再跑一次才干净,所以第 4 步的检查
#    失败、而测试退出后工作区反而是干净的 —— 一个只在退出后才生效的清理。
#    直接从 HEAD 恢复,不经过索引,就没有这个先后问题。
cleanup(){
  cd "$REPO" 2>/dev/null || return 0
  git reset -q HEAD 2>/dev/null
  git checkout -q HEAD -- scripts/fleet-status.sh 2>/dev/null
  git worktree list --porcelain 2>/dev/null | awk '/^worktree /{print $2}' | grep -F '/precommit-test.' \
    | while read -r w; do git worktree remove --force "$w" 2>/dev/null; done
  git worktree prune 2>/dev/null
  git branch --list 'precommit-*' 2>/dev/null | while read -r b; do git branch -D "$b" -q 2>/dev/null; done
  rm -rf "$FIX"
}
trap cleanup EXIT

echo "── 0. 前置:守卫还在,判据是裁定问的那个,而且锚点行没被上一轮改坏 ──"
[ -f "$HOOK" ] || { die "守卫文件不在: $HOOK"; exit 1; }
grep -q 'merge-tree --write-tree' "$HOOK" || { die "守卫已经不按裁定问的那个问题判断了"; exit 1; }
grep -q 'is-ancestor "$FUTURE" "$br"' "$HOOK" \
  && { die "守卫还在用被推翻的判据(分支是否包含 FUTURE)"; exit 1; } \
  || pass "判据是「合并会不会冲突」,不是被推翻的「是否齐平」"

# 锚点:这一行必须含「快照时间」。不逐字比整行 —— 整行里有引号和 $(),
# 单双引号嵌套在测试脚本里极易被 bash 吃掉,于是保护本身变成误报。
sed -n '32p' scripts/fleet-status.sh | grep -q '快照时间' \
  || { die "锚点行 32 不是预期内容 —— 工作区可能被上一轮改脏了,先恢复再跑"; exit 1; }
pass "锚点行 32 是预期的,样本有意义"

echo ""
echo "── 1. 两张真卡:必须放过 ──"
# 这两张是活库上真实存在的 pending 卡。真实合并已由 Aegis 量过:干净、删 0、丢 0。
if out=$(cd "$REPO" && bash "$HOOK" 2>&1); then
  printf '%s\n' "$out" | sed 's/^/     /'
  printf '%s\n' "$out" | grep -q 'merges clean' \
    && pass "两张真卡都报「merges clean」并放行" \
    || { die "放行了,但没报告合并是干净的 —— 那可能是漏检,不是通过"; exit 1; }
else
  printf '%s\n' "$out" | sed 's/^/     /'
  die "守卫拦下了两张本来点得动的真卡 —— 这正是 22:00 那次误拦"; exit 1
fi

echo ""
echo "── 2. 必败样本:一个真会冲突的分支必须被拦 ──"
# 🔴 第一版这个样本是错的,不是守卫错。我当时让分支在文件**末尾追加**一行,
# 而 FUTURE 里那一行不存在 —— 追加一个对方没有的东西不是冲突,是干净合并。
# 于是守卫正确地报「clean」,而我的测试报「守卫没守」。
# **一个造不出来的必败样本,会让你以为守卫是坏的。**
# 真冲突的形状:同一个基线行,两侧改成不同的内容。
sed -i '' '32s/.*/echo "HOOK_TEST_SIDE_A"/' scripts/fleet-status.sh
git add scripts/fleet-status.sh

(cd "$REPO" && git worktree add -q --detach "$FIX/wt" HEAD) || { die "建不了临时工作区"; exit 1; }
CONFLICT_BR="precommit-mustfail-$$"
(
  cd "$FIX/wt" || exit 1
  git checkout -q -b "$CONFLICT_BR"
  sed -i '' '32s/.*/echo "HOOK_TEST_SIDE_B"/' scripts/fleet-status.sh
  git -c user.email=t@t -c user.name=t commit -qam "must-fail fixture: same line, different content"
) || { die "造不出必败分支"; exit 1; }

FIXTURE="$FIX/cards.txt"
{
  printf '%s|%s|%s\n' "$CONFLICT_BR" "track/infra-foundation" "acc-mustfail"
  printf '%s|%s|%s\n' "branch/that/does/not/exist" "track/infra-foundation" "acc-missing"
} > "$FIXTURE"
echo "  控制样本:fixture 2 行,一行是必败分支,一行是不存在的分支"
[ "$(wc -l < "$FIXTURE" | tr -d ' ')" = "2" ] || { die "fixture 行数不对"; exit 1; }
git rev-parse --verify -q "$CONFLICT_BR" >/dev/null || { die "必败分支没建成 —— 后面的拦截毫无意义"; exit 1; }
! git rev-parse --verify -q "branch/that/does/not/exist" >/dev/null || { die "缺失分支其实存在"; exit 1; }

if out=$(cd "$REPO" && SEATLOOM_HOOK_CARDS_CMD="cat $FIXTURE" bash "$HOOK" 2>&1); then
  printf '%s\n' "$out" | sed 's/^/     /'
  die "必败样本被放行了 —— 这个守卫什么都没守"; exit 1
else
  rc=$?
  printf '%s\n' "$out" | sed 's/^/     /'
  [ "$rc" = "1" ] || { die "退出码是 $rc,不是 1 —— 拦截的退出码语义坏了"; exit 1; }
  printf '%s\n' "$out" | grep -q 'CONFLICTS' \
    && pass "必败分支被拦,并且说的是「合并冲突」(exit 1)" \
    || { die "拦下了,但理由不是冲突 —— 那它拦的可能是别的东西"; exit 1; }
  printf '%s\n' "$out" | grep -q 'missing in this checkout' \
    && pass "不存在的分支单独报「missing」,没有被合并成笼统的失败" \
    || { die "缺失分支没有单独报出来 —— 两种失败混在一起了"; exit 1; }
fi

echo ""
echo "── 3. 干净分支在同一个 fixture 里必须放过(证明不是「有卡就拦」) ──"
# 主检出仍是 SIDE_A;分支改成同一个 SIDE_A 再追加一行 —— 那才是干净合并。
(cd "$REPO" && git worktree add -q --detach "$FIX/wt2" HEAD) || { die "建不了第二个临时工作区"; exit 1; }
CLEAN_BR="precommit-clean-$$"
(
  cd "$FIX/wt2" || exit 1
  git checkout -q -b "$CLEAN_BR"
  sed -i '' '32s/.*/echo "HOOK_TEST_SIDE_A"/' scripts/fleet-status.sh
  printf '\n# 干净测试:只加新行,不动第 32 行\nCLEAN_SIDE\n' >> scripts/fleet-status.sh
  git -c user.email=t@t -c user.name=t commit -qam "clean fixture: same line plus an append"
) || { die "造不出干净分支"; exit 1; }
printf '%s|%s|%s\n' "$CLEAN_BR" "track/infra-foundation" "acc-clean" > "$FIXTURE"
if out=$(cd "$REPO" && SEATLOOM_HOOK_CARDS_CMD="cat $FIXTURE" bash "$HOOK" 2>&1); then
  printf '%s\n' "$out" | sed 's/^/     /'
  pass "干净分支被放过 —— 所以第 2 条的拦截确实来自冲突,不是来自「有卡」"
else
  printf '%s\n' "$out" | sed 's/^/     /'
  die "干净分支被拦 —— 这个守卫在乱拦"; exit 1
fi

echo ""
echo "── 4. 收尾:临时工作区和分支都清掉,主检出干净 ──"
cleanup
leftover=$(cd "$REPO" && git worktree list | grep -c precommit || true)
[ "$leftover" = "0" ] || { die "临时工作区没清干净: $leftover"; exit 1; }
(cd "$REPO" && git diff --quiet && git diff --cached --quiet) \
  && pass "主检出工作树干净,没有留下变异" \
  || { die "主检出被我的测试改脏了"; exit 1; }
(cd "$REPO" && git log --oneline -1) | sed 's/^/     HEAD: /'

echo ""
echo "✅ 两个方向都验过了:真卡放过,真冲突拦住,干净卡同样放过。"
