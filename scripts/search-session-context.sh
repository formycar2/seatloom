#!/usr/bin/env bash
# search-session-context.sh — 在全部历史会话上下文里按关键词检索,返回位置。
#
# 为什么需要它:270 个 session 的完整上下文已经抽成 261 个文件(354MB)。
# 但「问某个做法当时怎么做的」需要一个入口 —— 否则一个席位得翻 261 个文件。
# 这个脚本就是那个入口:给它关键词,它返回**哪个项目、哪个会话、原文件第几行**。
#
# 它不复制任何东西:命中结果带的是原文件行号,原文只有一份(在磁盘上)。
# 这是 B 方案的检索半边 —— 抽出来的文件是缓存,原文件是唯一真相。
#
# 判据的诚实边界:这是**字面匹配**,不是语义检索。它答不出「当时怎么解决 OOM 的」
# 如果原文用的是「显存不足」。语义检索是以后的事,这个先解决「按原词定位」。
#
# 用法:
#   bash scripts/search-session-context.sh <关键词> [更多关键词...]
#   默认任一关键词命中即返回;加 --all 要求全部命中
#
# 输出:每个命中带 项目 / 会话文件 / 原文件行号 / 上下文前后各 2 行
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$REPO/resume-contexts"

# --all 可能出现在任何位置(用户不会记住只能放最前面),所以扫描全部参数,
# 而不是只查 $1。第一版只查 $1,放在末尾就静默失效。
ALL=0
_args=(); for a in "$@"; do [ "$a" = "--all" ] && ALL=1 || _args+=("$a"); done
set -- "${_args[@]}"
[ $# -gt 0 ] || { echo "用法: bash scripts/search-session-context.sh <关键词> [更多...] [--all]"; exit 1; }

[ -d "$DIR" ] || { echo "❌ 上下文目录不在:$DIR —— 先跑 extract-session-context.sh"; exit 1; }

SEARCH_ALL="$ALL" SEARCH_DIR="$DIR" python3 - "$@" <<'PY'
import sys, os, re, glob

# 关键词只来自命令行;ALL / DIR 走环境变量,不占位置参数
# (第一版把它们也塞进 argv,于是 '0' 和目录路径本身成了关键词,命中 15 万轮)
terms = [a for a in sys.argv[1:] if not a.startswith('--')]
all_mode = os.environ.get('SEARCH_ALL') == '1'
d = os.environ.get('SEARCH_DIR', '.')

if not terms:
    print("没有关键词"); sys.exit(1)

files = sorted(glob.glob(os.path.join(d, '*.md')))
hits = []
for f in files:
    try:
        text = open(f, encoding='utf-8').read()
    except Exception:
        continue
    blocks = re.split(r'\n## ', text)
    header = blocks[0]
    proj = ''
    m = re.search(r'# 完整会话上下文 · (.+)', header)
    if m: proj = m.group(1).strip()
    src = ''
    m = re.search(r'来源文件\s+(\S+)', header)
    if m: src = m.group(1)
    for b in blocks[1:]:
        lines = ('## ' + b).split('\n')
        head = lines[0]
        body = '\n'.join(lines[1:])
        low = body.lower()
        ok = all(t.lower() in low for t in terms) if all_mode else any(t.lower() in low for t in terms)
        if not ok: continue
        lm = re.search(r'原文件第 (\d+) 行', head)
        line_no = lm.group(1) if lm else '?'
        hits.append((proj, os.path.basename(f), line_no, src, body.strip()))

print(f"关键词 {terms} ({'全部命中' if all_mode else '任一命中'})")
print(f"扫描 {len(files)} 个上下文文件,命中 {len(hits)} 轮\n")
if not hits:
    print("没有命中。换更接近原文的词 —— 这是字面匹配,不是语义检索。")
    sys.exit(0)

# 按项目聚合,先看分布
import collections
by_proj = collections.Counter(h[0] for h in hits)
print("按项目分布:")
for p, n in by_proj.most_common(12):
    print(f"  {n:4d}  {p}")
print()

LIMIT = int(os.environ.get('SEARCH_LIMIT', '8'))
for proj, fn, ln, src, body in hits[:LIMIT]:
    print("─" * 78)
    print(f"项目     {proj}")
    print(f"会话     {fn}")
    print(f"原文件第 {ln} 行")
    print(f"原文位置 {src}")
    snippet = body if len(body) <= 700 else body[:700] + ' …'
    print(f"\n{snippet}\n")
if len(hits) > LIMIT:
    print(f"…另 {len(hits)-LIMIT} 轮命中。SEARCH_LIMIT=50 可看更多。")
PY
