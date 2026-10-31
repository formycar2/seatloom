#!/usr/bin/env bash
# fix-doc-headers.sh — 给缺 §2 通用头的文档补上,而不是把它们塞进豁免名单。
#
# 背景(2026-10-31,Lyra 发现,我复核):
#   scripts/verify-docs.sh 是个棘轮闸门,只检查相对 merge-base 改过的文件。
#   以真实 CI 基线(HEAD 与 track/infra-foundation 的 merge-base)跑,
#   退出码 1,**29 个文件违规**,全部是我这两天写的:
#   16 份 Aegis 心跳交付、12 份 knowledge 文档、INDEX.md。
#
#   而 verify-docs.sh 在 .gitlab-ci.yml 里是 docs 阶段的最后一步、
#   没有 allow_failure —— 也就是说**任何往 gitlab 发的合并请求都会红**。
#   gitlab 从这台机器连不上(Nimbus 量过 12 次全部超时),
#   于是这个闸门红在一个没人看得见的地方,已经红了一个月。
#
#   我把自己的文档写成违反自己项目的闸门,连写了 28 份。
#   这就是 Lyra 说的「一个大家都学会跨过去看的红闸门不是闸门」——
#   而这次跨过去的人就是我,写的人也是我。
#
# 为什么补头而不是加豁免:
#   这些文件不是历史欠账。DATA-LOSS-INCIDENT.md 是 10-31 写的,
#   AEGIS-HEARTBEAT.md 是 10-28 写的,16 份心跳交付全是这两天。
#   **把新写的文件塞进豁免名单,是把「我违规了」改成「我豁免了」。**
#   Lyra 也是这么做的 —— 她本来可以把 LYRA-HEARTBEAT.md 加进豁免,
#   但她回退了自己的编辑,因为「我的文档不是历史欠账」。
#
# 用法: bash scripts/fix-doc-headers.sh           # 干跑,只打印会改什么
#       bash scripts/fix-doc-headers.sh --apply   # 真的改文件
#
# 幂等:已经合规的文件不动;跑第二遍什么都不改。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

# 用 CI 真实用的基线,不要用「全部文件」—— 那样会把一个月的历史欠账
# 一起算进来,而这个脚本只该修我这几天写的东西。
BASE="$(git merge-base HEAD track/infra-foundation 2>/dev/null)"
if [ -z "$BASE" ]; then
  echo "找不到与 track/infra-foundation 的 merge-base —— 中止。" >&2
  exit 1
fi

echo "基线(CI 同款) = $(git rev-parse --short "$BASE")"
echo

CHANGED="$( {
  git diff --name-only --diff-filter=ACMR "$BASE" HEAD -- docs 2>/dev/null
  git diff --name-only --diff-filter=ACMR -- docs 2>/dev/null
  # 未跟踪的新文件也要算进来。闸门自己的 CHANGED 有这一条
  # (git ls-files --others),我第一版没有,于是刚写完还没提交的文档
  # 每次都漏掉,下一轮心跳又报同样的违规 ——
  # 而那正是我刚花一整轮修的东西。
  git ls-files --others --exclude-standard -- docs 2>/dev/null
} | grep -E '\.md$' | sort -u )"
if [ -z "$CHANGED" ]; then
  echo "没有改过的文档。"
  exit 0
fi

# 每个文件要补的字段:template subtype id status author date version tags。
# 值和文件内容对齐,不是编的。
#
# template 的取值不是名字,是**位置码**:闸门规定 T1–T7 各自对应一个目录,
# 而且会检查文件是否真的住在那个目录下。我第一版自己发明了 T-knowledge /
# T-acceptance,闸门报「template not in T1–T7」—— 把一个我不知道取值范围的值
# 当成了取值范围。第二次犯错是同一个形状:我按自己想的格式写了一篇文档,
# 而没有先读规定。
#   T3 = docs/coordination/tasks
#   T7 = docs/coordination(含 knowledge/、INDEX.md)
#
# subtype 沿用同目录里既有文档用过的值,不发明。
# id 取文件名去扩展名。
# status:交付说明是 delivered,knowledge 是 recorded。
# version 固定 v1。
# tags 从标题里取实词,宁少勿编。
fixed=0
skipped=0

while IFS= read -r f; do
  [ -f "$f" ] || continue

  # 只处理文档。.patch 是机器产物,不是文档 ——
  # 我 15:00 那一版把两个 .patch 也改名加了头,因为它们 diff 里的
  # 上下文行含中文。补丁文件的**名字和内容都必须一个字节都不能动**,
  # 否则 git apply 就贴不回去了。抢救一份工作的产物被自己改坏,
  # 比没有这份产物更糟。
  case "$f" in
    *.md) : ;;
    *) continue ;;
  esac

  # 已经有全部 8 个字段的,不动。
  if grep -qE '^\| *template *\|' "$f" && grep -qE '^\| *subtype *\|' "$f" \
     && grep -qE '^\| *id *\|' "$f" && grep -qE '^\| *status *\|' "$f" \
     && grep -qE '^\| *author *\|' "$f" && grep -qE '^\| *date *\|' "$f" \
     && grep -qE '^\| *version *\|' "$f" && grep -qE '^\| *tags *\|' "$f"; then
    skipped=$((skipped+1))
    continue
  fi

  base="$(basename "$f" .md)"
  case "$f" in
    docs/coordination/tasks/*)     template="T3"; subtype="task_delivery"; status="delivered" ;;
    docs/coordination/acceptance/*) template="T5"; subtype="acceptance_note"; status="accepted" ;;
    docs/coordination/salvage/*)    template="T7"; subtype="salvage";         status="recorded" ;;
    *)                             template="T7"; subtype="knowledge";      status="recorded" ;;
  esac

  # 日期:文件名里找 YYYY-MM-DD,找不到再找正文里第一个。
  d="$(printf '%s' "$base" | grep -oE '20[0-9]{2}-[0-9]{2}-[0-9]{2}' | head -1)"
  [ -z "$d" ] && d="$(grep -oE '20[0-9]{2}-[0-9]{2}-[0-9]{2}' "$f" | head -1)"
  [ -z "$d" ] && d="$(date '+%Y-%m-%d')"

  # tags:从标题里抓几个实词,不给空、不编。
  ttl="$(grep -m1 '^# ' "$f" | sed 's/^# *//')"
  tags="$(printf '%s' "$ttl" | tr '[:upper:]' '[:lower:]' | grep -oE '[a-z][a-z-]{4,}' | head -3 | tr '\n' ',' | sed 's/,$//')"
  [ -z "$tags" ] && tags="delivery"

  author="aegis"
  case "$f" in
    docs/coordination/knowledge/*) author="aegis" ;;
  esac

  header="| Field | Value |
|---|---|
| template | $template |
| subtype | $subtype |
| id | $base |
| status | $status |
| author | $author |
| date | $d |
| version | v1 |
| tags | $tags |"

  echo "  ✗ → ✓ $f"
  echo "        id=$base date=$d tags=$tags"

  # §10 必须先做,§4 后做。
  #
  # 我在临时仓库里跑第一版,29 个违规降到 2 个,剩下的两个是
  # AEGIS-2026-10-31-DATA-LOSS-INCIDENT 和 AEGIS-HEARTBEAT ——
  # 先加了 -v1,又在后面接了 .zhs.md,得到 X-v1.zhs.md,
  # 而闸门的 §4 正则要求以 .md 结尾,于是 -vN 白加了。
  # 顺序反了会让前半段的工夫归零,而这个归零在干跑里看不出来,
  # 只有跑完再跑一遍闸门才看得见。所以下面每次改名之后都复核。
  #
  # §10 先判定,不改名 —— 只记下「这篇是中文」,名字最后统一改。
  # 闸门已经为中文文档留了位置:acceptance/ 下已经有 5 个 .zhs.md。
  # 我给张先生写的交付说明必须是中文(他要能看懂),knowledge 记的是我和席位
  # 之间的判断,用中文才写得准。所以走 .zhs.md 这条路,
  # 而不是把中文删掉换成英文 —— 那等于为了让检查过,把内容变得他看不懂。
  zhs=0
  # already_zhs 记的是**改名之前**名字里有没有 .zhs。
  # §4 会把名字改成 X.zhs-v1,那时它不再以 .zhs 结尾,
  # 所以最后一步不能靠改名后的名字判断 —— 我第一版那么写,
  # 结果一个本来就叫 .zhs.md 的文件被接了两次后缀。
  # 判断必须用改动前的状态,而改动前的状态只会在这里原样存在。
  already_zhs=0
  if printf '%s' "$base" | grep -q '\.zhs$'; then
    already_zhs=1
  fi
  if ! printf '%s' "$base" | grep -q '\.zhs$'; then
    if python3 -c "import sys,re; sys.exit(0 if re.search('[一-鿿]', open(sys.argv[1],encoding='utf-8').read()) else 1)" "$f" 2>/dev/null; then
      zhs=1
    fi
  else
    zhs=1
  fi

  # §4:任务类文件名必须是 ROLE-YYYY-MM-DD-<topic>-vN.md。
  # 我的心跳交付写成了 ...-1200-heartbeat-xxx.md,少了 -vN。
  # 闸门不是抱怨,它是对的:同一主题下没有版本号,第二版就只能靠时间戳区分。
  # 正则要容许 .zhs 后缀:中文的任务文档同时要满足 §10 和 §4,
  # 而闸门自己的 §4 正则没有为 .zhs 开口子 —— 这是个真问题,
  # 但改闸门不是这一轮的事,先把名字写成它能接受的形式。
  # .zhs 是**最后一步**才接上的后缀。
  #
  # 我在临时仓库里跑出过 AEGIS-HEARTBEAT.zhs-v1.md ——
  # 因为我把 base 改成了 X.zhs,然后 §4 又在后面接了 -v1,
  # 后缀跑到了中间。第一次的顺序错是 -v1 之后接 .zhs,第二次是反的,
  # 两次都让前一步的工夫归零,而两次在干跑里都看不出来,
  # 只有跑完再跑一遍闸门才看得见。
  # 所以现在用两个变量:base 只管主名,zhs 单独记着,最后统一接上。
  #
  # 🔴 而 16:00 又抓到第三次,这次是我的**验法是错的**,不是脚本。
  # 我在临时仓库里验「X.zhs-v1.md 能不能过闸门」,只测了 §4 的正则
  # (`.+` 贪婪匹配,所以 .zhs 在中间它也认),就写了「三种输入都验过」。
  # 真的闸门还跑 §10:文件名必须以 .zhs.md **结尾**。
  # X.zhs-v1.md 于是被报「含中文」。
  # **拿闸门的一条规则冒充整个闸门,是最像验过了的一种没验。**
  # 所以 -v1 必须插在 .zhs **前面**:X.zhs → X-v1.zhs,而不是 X.zhs-v1。
  if [ "$template" = "T3" ]; then
    if ! printf '%s' "$base" | grep -qE '^[A-Z]+-[0-9]{4}-[0-9]{2}-[0-9]{2}-.+-v[0-9]+$'; then
      if [ "$already_zhs" = "1" ]; then
        # 名字本来以 .zhs 结尾:把 -v1 插到它前面。
        newbase="${base%.zhs}-v1.zhs"
      else
        newbase="${base}-v1"
      fi
      newf="$(dirname "$f")/$newbase.md"
      echo "        rename → $newbase.md (§4)"
      if [ "$APPLY" = "1" ]; then
        if [ -e "$newf" ]; then
          echo "        !! 目标已存在,不改名:$newf" >&2
        else
          git mv "$f" "$newf" 2>/dev/null || mv "$f" "$newf"
          # §4 改名同样要回头改索引里的链接。
          idx="$REPO/docs/coordination/knowledge/INDEX.md"
          [ -f "$idx" ] && grep -q "${base}.md" "$idx" && {
            sed -i '' "s|${base}.md|${newbase}.md|g" "$idx" 2>/dev/null \
              || sed -i "s|${base}.md|${newbase}.md|g" "$idx"
          }
          f="$newf"
          base="$newbase"
        fi
      fi
    fi
  fi

  # 最后统一接 .zhs。中文的任务文档要同时满足 §10 和 §4,
  # 而闸门的 §4 正则只认 .md 结尾 —— 它自己没有为 .zhs 开口子,
  # 这是个真问题,但改闸门不是这一轮的事。
  #
  # 只在后缀还不在的时候接。我第一版无条件接,于是**一个本来就叫
  # .zhs.md 的文件会被接两次**:X.zhs.md → §4 加 -v1 → X.zhs-v1.md
  # → 再接 .zhs → X.zhs-v1.zhs.md,而 §4 正则只认 .md 结尾,于是
  # 它又不合规了。这个 bug 是我在临时仓库里复现 Lyra 的变异时撞见的 ——
  # 她 mutation-tested 我的修复(三个方向,证明那一行是承重的),
  # 而我自己复现时用了不同的探针文件名,撞出了另一个洞。
  # **同一个工具,换一个输入就露出下一个洞。**
  # 只在「本来没有 .zhs、而且内容是中文」时才接。
  # 用 already_zhs(改动前的状态),不用当前 $base ——
  # §4 已经把它改成 X.zhs-v1,那时它不以 .zhs 结尾,
  # 照当前名字判断就会把一个本来就合规的文件再接一次后缀。
  if [ "$zhs" = "1" ] && [ "$already_zhs" = "0" ]; then
    newf="${f%.md}.zhs.md"
    echo "        rename → $(basename "$newf") (§10 中文文档)"
    if [ "$APPLY" = "1" ]; then
      if [ -e "$newf" ]; then
        echo "        !! 目标已存在,不改名:$newf" >&2
      else
        git mv "$f" "$newf" 2>/dev/null || mv "$f" "$newf"
        idx="$REPO/docs/coordination/knowledge/INDEX.md"
        if [ -f "$idx" ] && grep -q "$base.md" "$idx"; then
          sed -i '' "s|$base.md|$base.zhs.md|g" "$idx" 2>/dev/null \
            || sed -i "s|$base.md|$base.zhs.md|g" "$idx"
          echo "        INDEX.md 里的链接也跟着改了"
        fi
        f="$newf"
      fi
    fi
  fi

  if [ "$APPLY" = "1" ]; then
    # 插在第一个一级标题之后、第一个空行之后。
    # 用 python 而不是 sed:要按行号插多行内容,macOS 的 sed -i 对多行插入很难看。
    python3 - "$f" "$header" <<'PYEOF'
import sys
path, header = sys.argv[1], sys.argv[2]
lines = open(path, encoding='utf-8').read().split('\n')
# 找第一个 '# ' 标题行,插到它后面第一个空行之后;没有空行就紧接标题后。
idx = next((i for i, l in enumerate(lines) if l.startswith('# ')), None)
if idx is None:
    lines = header.split('\n') + [''] + lines
else:
    j = idx + 1
    while j < len(lines) and lines[j].strip() == '':
        j += 1
    lines[j:j] = [''] + header.split('\n')
open(path, 'w', encoding='utf-8').write('\n'.join(lines))
PYEOF
    fixed=$((fixed+1))
  fi
done <<EOF
$CHANGED
EOF

echo
if [ "$APPLY" = "0" ]; then
  echo "干跑结束。加 --apply 真的改。"
else
  echo "改完 $fixed 个,另有 $skipped 个本来就合规。"
  echo
  echo "── 复核:再用 CI 的基线跑一遍闸门 ──"
  bash scripts/verify-docs.sh "$BASE" 2>&1 | tail -3
  echo "退出码 $?"
fi
