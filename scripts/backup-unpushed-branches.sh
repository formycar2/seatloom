#!/usr/bin/env bash
# backup-unpushed-branches.sh — 把只存在于这一个 clone 里的提交打成本地 bundle。
#
# 为什么要这个脚本(2026-10-31 的事故):
#   Lyra 清空了 live 数据库,2946 行事件账本、68 张已接受的卡、约 400 件派单
#   全部不可恢复(archive_mode 关着、无 pg_dump、WAL 已覆盖)。
#   **git 从此是这个项目唯一耐久的记录** —— 哪件派单产出哪个提交、
#   哪条 verdict 验过它、哪张卡被接受,只剩提交信息里那几句。
#
#   而实测:Flux 的 packet/flux/intent-verdict **在两个远端上都不存在**
#   (origin 没有这个分支,gitlab 落后 29 个提交),那 29 个提交里有 11 份
#   verdict 报告。那就是下一个「一处都没有了」。
#
# 这个脚本只做一件事:对每个「领先两个远端」或「在任何远端上都不存在」的
# 分支,打一个 bundle 到 backups/。**不 push、不连任何远端、不碰数据库。**
# 它是纯本地读 + 写一个新文件,所以不需要张先生批准。
#
# 为什么 bundle 而不是 cp -r .git:bundle 是单个文件、可验证、可被
# `git bundle verify` 检查,而且可以整体搬去另一台机器后 `git fetch` 回来。
#
# 用法: bash scripts/backup-unpushed-branches.sh
#       默认写到 <repo>/backups/git-<timestamp>.bundle,并打印每个分支的状态。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

STAMP=$(date '+%Y%m%d-%H%M%S')
OUTDIR="$REPO/backups"
mkdir -p "$OUTDIR"
OUT="$OUTDIR/git-$STAMP.bundle"

# 只备份「有丢失风险」的分支:领先任一远端,或在所有远端上都不存在。
# 一个已经推齐的分支打进 bundle 只是浪费空间,而清单太长人就懒得看。
at_risk=()
while IFS= read -r b; do
  [ -n "$b" ] || continue
  om=$(git rev-list --count "origin/$b".."$b" 2>/dev/null || echo "?")
  gm=$(git rev-list --count "gitlab/$b".."$b" 2>/dev/null || echo "?")
  if [ "$om" != "0" ] || [ "$gm" != "0" ]; then
    at_risk+=("$b")
  fi
done < <(git for-each-ref --format='%(refname:short)' refs/heads/packet/ refs/heads/track/)

if [ "${#at_risk[@]}" -eq 0 ]; then
  echo "没有领先远端的分支 —— 不需要备份。"
  exit 0
fi

echo "有丢失风险的分支 ${#at_risk[@]} 个:"
# 「远端没有这个分支」和「远端落后 N 个」是两回事,而且前者更糟。
# rev-list --count 对一个不存在的 ref 会失败,我原来把它打成「?」——
# 而 ? 和 0 在视觉上一样安静,于是最该被看见的情况最不显眼。
for b in "${at_risk[@]}"; do
  if git rev-parse --verify -q "origin/$b" >/dev/null; then
    om="+$(git rev-list --count "origin/$b".."$b" 2>/dev/null || echo '?')"
  else
    om="不存在"
  fi
  if git rev-parse --verify -q "gitlab/$b" >/dev/null; then
    gm="+$(git rev-list --count "gitlab/$b".."$b" 2>/dev/null || echo '?')"
  else
    gm="不存在"
  fi
  printf "  %-46s origin=%-8s gitlab=%-8s\n" "$b" "$om" "$gm"
done
echo
# track 本身落后,是这个项目最静默的风险:它是集成线,是所有席位的 base。
# 2026-10-31 实测 origin 停在 09-30(落后 897),gitlab 停在 10-09(落后 743)。
# 这台机器一丢,两周的集成就没了,而没有任何一处会显示这件事。
if git rev-parse --verify -q "origin/track/infra-foundation" >/dev/null; then
  _t=$(git rev-list --count "origin/track/infra-foundation".."track/infra-foundation" 2>/dev/null || echo "?")
  if [ "$_t" != "0" ] && [ "$_t" != "?" ]; then
    echo "⚠️  track/infra-foundation 领先 origin ${_t} 个提交。"
    echo "    它是集成线,是所有席位分支的 base。这台机器一丢,这些提交只剩这一份。"
    echo "    修:git push origin track/infra-foundation  (要你执行,这是共享状态)"
    echo
  fi
fi

# --all 会连已经推齐的分支一起打。这里显式列出有风险的那些,
# 这样 bundle 小而准,而且 verify 的输出能直接对着上面的清单看。
if ! git bundle create "$OUT" "${at_risk[@]}" 2>&1 | tail -3; then
  echo "❌ bundle 创建失败"
  exit 1
fi

echo
echo "── 验证 ──"
# 一个建出来但不验证的备份,和没有备份的区别只在心里好受。
# git bundle verify 会真的去解一遍对象,打不开会报错。
if git bundle verify "$OUT" 2>&1 | tail -4; then
  echo
  echo "✅ $OUT ($(du -h "$OUT" | cut -f1))"
  echo
  echo "怎么用:把它拷到另一台机器,然后"
  echo "  git fetch <path-to-bundle> '<branch>:<branch>'"
  echo "或者在本机:"
  echo "  git fetch $OUT '<branch>:<branch>'"
else
  echo "❌ bundle 验证失败 —— 这个备份不可用,不要当成已经有了。"
  exit 1
fi
