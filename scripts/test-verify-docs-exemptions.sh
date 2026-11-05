#!/usr/bin/env bash
# test-verify-docs-exemptions.sh — 证明 verify-docs.sh 的豁免没有把该查的文件一起吃掉。
#
# 为什么需要这个测试(2026-10-31,Lyra 发现闸门红,我修的时候连踩三个坑):
#
# 坑一,我发明的 template 值(T-knowledge)不在 T1–T7 里。闸门报
#   「template not in T1–T7」,29 个违规一个没修掉。
# 坑二,两次改名顺序错:先 -v1 后 .zhs 得到 X-v1.zhs.md(正则不认),
#   反过来得到 X.zhs-v1.md(后缀跑到中间)。两次都让前一步归零,
#   而两次在干跑里都看不出来,只有跑完再跑一遍闸门才看得见。
# 坑三,**最险的一个**:我把 §4 的分发从 $f(完整路径)改成了 $base(文件名),
#   于是 docs/coordination/tasks/* 这种路径模式永远匹配不上,
#   §4 成了死代码 —— 而闸门照样全绿。
#
# 然后变异测试第二次存活才把它揪出来。可变异测试能证明「删掉它会红」,
# 证明不了「它现在真的在跑」。所以要有这个测试:
# **直接断言 §4 在该跑的文件上跑,在不该跑的文件上不跑。**
#
# 一个不会失败的检查比没有检查更糟。这个测试的每一行都必须能失败。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

GATE="$REPO/scripts/verify-docs.sh"
[ -f "$GATE" ] || { echo "❌ 找不到 $GATE"; exit 1; }

FAIL=0
pass() { echo "  ✓ $1"; }
die()  { echo "  ✗ $1"; FAIL=1; }

# 从闸门里把 §4 的判定逻辑原样抽出来,不在这个测试里重写一遍 ——
# 重写就等于测了我自己的复制品,而不是闸门里的东西。
# 做法:source 闸门会执行它的主流程,所以改用 grep 断言它包含的关键判定。
echo "── 1. §4 必须按完整路径分发,不能按文件名 ──"
# 这一条对应坑三。按 $base 分发时,路径模式永远匹配不上,§4 就是死代码。
if grep -qE '^\s*case "\$f" in' "$GATE"; then
  pass "§4 按 \$f(完整路径)分发"
else
  die "§4 不是按 \$f 分发 —— 路径模式匹配不上文件名,§4 是死代码(坑三)"
fi

echo
echo "── 2. §4 必须容忍 .zhs 后缀 ──"
# 这一条对应坑一/坑二:中文任务文档必须同时满足 §10(.zhs.md)和 §4(-vN.md)。
if grep -qE '\-v\[0-9\]\+\(\\\.zhs\)\?\\\.md\$' "$GATE"; then
  pass "§4 接受 ROLE-YYYY-MM-DD-topic-vN.zhs.md"
else
  die "§4 不接受 .zhs 形式 —— 中文任务文档不可能同时满足 §10 和 §4"
fi

echo
echo "── 3. 常驻规程文档的豁免必须按「没有日期段」判 ──"
# 这一条对应变异测试第二次存活:按文件名里有 HEARTBEAT 判,
# 会把我 16 份心跳交付一起吃掉,而闸门照样全绿。
if grep -qE "standing=0" "$GATE" && grep -qE "grep -qE '\[0-9\]\{4\}-\[0-9\]\{2\}-\[0-9\]\{2\}'" "$GATE"; then
  pass "豁免按有无 YYYY-MM-DD 判"
else
  die "豁免不是按日期段判的 —— 它会把带日期的心跳交付一起豁免掉"
fi

echo
echo "── 4. 真的跑一遍:豁免不能吃掉带日期的交付物 ──"
# 上面三条都是读文本。这一条**执行**:拿一个名字不合规但带日期的心跳交付,
# 闸门必须报它;再拿一个真规程文档,闸门必须放过它。
# 这是唯一能区分「豁免正确」和「豁免一切」的办法 ——
# 变异测试证明不了这件事,因为两者都给出 0 违规。
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
mkdir -p "$scratch/docs/coordination/tasks/aegis" "$scratch/.gitlab" "$scratch/scripts"
# 闸门必须放在临时仓库的 scripts/ 下,和真实仓库里同一个相对深度。
# 它第一行就 cd "\$(dirname \$0)/.." —— 我第一版把它放在临时仓库根目录,
# 它就 cd 到了 /tmp,于是 git rev-parse 看不见我给的基线,
# 基线退化成空树,闸门说「没有改过的文档」。
# 看着像闸门失效,其实是我把它放错了地方。
cp "$GATE" "$scratch/scripts/verify-docs.sh"
: > "$scratch/.gitlab/docs-grandfather.txt"
# 闸门靠 git 找「改过的文档」,所以这个临时目录必须是个 git 仓库。
(cd "$scratch" && git init -q . && git add -A \
  && git -c user.email=t@t -c user.name=t commit -qm base)

hdr() {
  printf '| Field | Value |\n|---|---|\n| template | T3 |\n| subtype | task_delivery |\n| id | %s |\n| status | delivered |\n| author | aegis |\n| date | 2026-10-31 |\n| version | v1 |\n| tags | test |\n' "$1"
}

# 4a. 带日期、缺 -vN、中文 → 必须被 §4 抓到(豁免不许碰它)
# 两个条件必须同时满足,少一个这个检查就测不到它声称要测的东西:
#   (1) 文件名里带 heartbeat —— 和真实的那 16 份交付物同形,
#       否则豁免从来不适用于它,豁免太宽也测不出来;
#   (2) 文件名**缺** -vN —— 否则 §4 本来就该放过它,检查永远绿。
# 我第一版两个条件都没满足:先用了不含 heartbeat 的名字,
# 改成含 heartbeat 的之后又顺手带了 -v1,于是这个检查
# 在干净闸门下也失败、在坏闸门下也失败 —— 两种情况下它都在喊,
# 但喊的不是同一件事。**一个在干净状态下就失败的检查,没人会去看它的输出。**
bad="$scratch/docs/coordination/tasks/aegis/AEGIS-2026-10-31-1300-heartbeat-probe.zhs.md"
{ printf '# 测试\n\n'; hdr "AEGIS-2026-10-31-1300-heartbeat-probe"; } > "$bad"
# 4b. 不带日期、真规程文档 → 必须被放过
good="$scratch/docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.zhs.md"
{ printf '# 心跳协议\n\n'; hdr "AEGIS-HEARTBEAT"; } > "$good"
# 提交一次基线,再改这两个文件,闸门才认它们是「改过的文档」。
(cd "$scratch" && git add -A && git -c user.email=t@t -c user.name=t commit -qm new)

# 基线 ref 必须在临时仓库**里面**算。
# 我第一版写的是 out="$(cd $scratch && bash verify-docs.sh "$(git rev-parse HEAD~1)" 2>&1)" ——
# 内层 $(git rev-parse) 在**真实仓库**里求值,把真实仓库的 HEAD~1
# 传给了临时仓库的闸门,它 rev-parse 不到就退化成空树,于是
#  「没有改过的文档」。看着像闸门失效,其实是我给它指错了基线。
out="$(cd "$scratch" && base=$(git rev-parse HEAD~1) && bash scripts/verify-docs.sh "$base" 2>&1)"
# 空输出必须被证明:先确认闸门真的检查了这两个文件,再解读它的结论。
# 否则「没报错」和「根本没看」在这一行里长得一模一样。
if ! printf '%s' "$out" | grep -q '1300-heartbeat-probe.zhs.md'; then
  if printf '%s' "$out" | grep -q 'nothing to check'; then
    die "闸门说『没有改过的文档』—— 测试没让它看到这两个文件,结论无效"
  fi
fi

if printf '%s' "$out" | grep -q "1300-heartbeat-probe.zhs.md: task filename must be"; then
  pass "带日期的交付物缺 -vN 被抓住了(豁免没有吃掉它)"
else
  die "带日期的交付物缺 -vN 没被抓到 —— 豁免太宽,16 份心跳交付都在被它放过"
fi

if printf '%s' "$out" | grep -q "AEGIS-HEARTBEAT.zhs.md: task filename"; then
  die "常驻规程文档被 §4 误报了 —— 它没有日期,不该有"
else
  pass "常驻规程文档被放过了"
fi

echo
if [ "$FAIL" = "0" ]; then
  echo "✅ 闸门的豁免只豁免了该豁免的。"
else
  echo "❌ 闸门的豁免有问题 —— 上面每一条都是真的会失败的那一种。"
fi
exit "$FAIL"
