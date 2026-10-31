#!/usr/bin/env bash
# land-aegis-packet-branch.sh — 把 Aegis 的分支合进集成线,再推到 origin。
#
# 为什么要这个脚本(2026-10-31 17:00,Lyra 发现):
#
#   集成线 track/infra-foundation 停在 052f6ff(10-30 17:37)。
#   今天所有通过验证的工作 —— 账本存在性检查、快照诚实的 4 处修正、
#   补席位脚本、未推送分支备份、抢救存档、文档闸门修复和它的测试、
#   补头工具的三个缺陷修复 —— **全在 packet/aegis/heartbeat-1800-un-gated-merges 上,
#   不在集成线上**。实测领先 44 个提交(现在是 45)。
#
#   Lyra 的原话:「我读主检出的 git log,然后把结果描述成『已经落到 track 上』。
#   **它们落在他的 packet 分支上。** 我在声称一个提交被集成之前,
#   没有先核 HEAD 在哪个 ref 上。」这是她今天第 N 次同一个测量错误,
#   而这次后果最大:集成线和工作线已经分叉。
#
#   另一件事:track 领先 origin **897** 个提交(origin 停在 09-30)。
#   合完之后会变成 942。这台机器一丢,两周的集成记录就没了 ——
#   而没有任何一处会显示这件事。
#
# 这个脚本只做两步,都是**快进**,不可能有冲突:
#   1. git merge --ff-only packet/aegis/... 进 track(已实测 track 是它的严格祖先)
#   2. git push origin track/infra-foundation
#
# 为什么默认干跑:合并集成线 + 推共享远端,是这个项目里最重的两个动作。
# 误粘贴一次就该跑的东西,不能默认就跑。
#
# 用法: bash scripts/land-aegis-packet-branch.sh          # 干跑,只打印会做什么
#       bash scripts/land-aegis-packet-branch.sh --apply  # 真的做

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

BRANCH="packet/aegis/heartbeat-1800-un-gated-merges"
TRACK="track/infra-foundation"

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

echo "── 现状 ──"
git branch --show-current | sed 's/^/  当前分支: /'
for r in "$TRACK" "$BRANCH" "origin/$TRACK"; do
  if git rev-parse --verify -q "$r" >/dev/null 2>&1; then
    printf "  %-46s %s %s\n" "$r" "$(git rev-parse --short "$r")" \
      "$(git log -1 --format='%ad' --date=format:'%m-%d %H:%M' "$r")"
  else
    printf "  %-46s %s\n" "$r" "(不存在)"
  fi
done
echo

# 前提一:分支必须存在。
if ! git rev-parse --verify -q "$BRANCH" >/dev/null; then
  echo "❌ 找不到分支 $BRANCH —— 中止。" >&2
  exit 1
fi

# 前提二:必须是快进。不是快进就说明集成线上有我不知道的提交,
# 那时候该停下来问,而不是合。
if git merge-base --is-ancestor "$TRACK" "$BRANCH"; then
  ahead=$(git rev-list --count "$TRACK".."$BRANCH")
  # origin 落后多少也一起算出来,后面干跑提示要用。
  # 不写死:17:00 我写死 897,18:00 就已经不是那个数了。
  behind_origin=0
  if git rev-parse --verify -q "origin/$TRACK" >/dev/null 2>&1; then
    behind_origin=$(git rev-list --count "origin/$TRACK".."$BRANCH")
  fi
  echo "  ✓ 可以快进:$BRANCH 领先 $TRACK $ahead 个提交,无冲突可能。"
else
  echo "  ❌ 不是快进 —— $TRACK 上有 $BRANCH 没有的提交。" >&2
  echo "     这时候该停下来弄清楚,不是合。" >&2
  exit 1
fi
echo

# 这批提交里有什么。合之前必须能说清,不能只说一个数字 ——
# 数字每天都会变(17:00 我写死「45」,18:00 已经是 47),
# 而写死的数字会和现实脱钩,然后开始骗人。
# **一个不会跟着现实变的数字,比没有数字更糟。**
echo "── 将要进入集成线的内容 ──"
git diff --stat "$TRACK".."$BRANCH" | tail -1 | sed 's/^/  /'
echo "  按目录:"
git diff --name-only "$TRACK".."$BRANCH" | sed 's|/[^/]*$||' | sort | uniq -c | sort -rn \
  | sed 's/^/    /'
echo

# 有没有产品代码。今天这批全是文档、脚本、存档,没有 crates/ 和 ui/。
# 这一条是断言:如果哪天它不再是纯文档,这里必须喊。
if git diff --name-only "$TRACK".."$BRANCH" | grep -qE '^(crates|ui|src-tauri)/'; then
  echo "  ⚠️  这批改动里有产品代码(crates/ ui/ src-tauri/)。"
  echo "     按你定的规矩,产品代码要有验收卡才进集成线。先确认这件事。"
  echo
else
  echo "  ✓ 没有产品代码 —— 只有文档、脚本、存档。"
  echo
fi

if [ "$APPLY" = "0" ]; then
  echo "── 干跑结束 ──"
  echo
  echo "将执行:"
  # 用 $ahead 和 $behind_origin,不写死 —— 上面刚算过。
  echo "  git merge --ff-only $BRANCH        # 把 $ahead 个提交带进集成线"
  echo "  git push origin $TRACK             # 推到 origin(现在落后 $behind_origin 个)"
  echo
  echo "确认无误就跑:  bash scripts/land-aegis-packet-branch.sh --apply"
  exit 0
fi

echo "── 执行 ──"
git checkout -q "$TRACK" || { echo "❌ 切不到 $TRACK" >&2; exit 1; }
if ! git merge --ff-only "$BRANCH"; then
  echo "❌ 合并失败 —— 已切到 $TRACK,但没有改动它。" >&2
  exit 1
fi
echo "  ✓ 已快进合并"

echo
echo "── 推送 ──"
if git push origin "$TRACK"; then
  echo "  ✓ 已推到 origin"
  echo
  echo "现在 origin 上的集成线和本地一致了。"
  echo "核对:git rev-list --count origin/$TRACK..$TRACK  应该输出 0"
else
  echo "❌ 推送失败(网络或权限)。合并已经在本地完成,重试推送即可:" >&2
  echo "    git push origin $TRACK" >&2
  exit 1
fi
