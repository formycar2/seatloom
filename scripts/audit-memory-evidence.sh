#!/usr/bin/env bash
# audit-memory-evidence.sh — 每一条记忆的「行号证据」到底支不支持它的说法。
#
# 为什么需要它:2026-10-12 我只抽样 5 条,结果关键词命中从 1/6 到 6/6 不等。
# 「行号开得出原文」和「那段原文支撑这句话」是两件事,而后者才是验收卡第 7 步
# 要张先生判断的东西。抽样不足以回答,必须全量。
#
# 它回答三个问题,全部只读:
#   1. 每条记忆的证据区间能不能打开(路径存在 + 行号不越界)
#   2. 区间里的文本,和这条记忆的摘要有没有实质词汇重叠(支撑的弱信号)
#   3. 分布如何 —— 特别是「开得出但零命中」的那批,那是真问题
#
# 它不写任何东西:不改记忆、不改库、不重跑洗数据。审计只报。
#
# 判据的诚实边界:词汇重叠是**弱信号**,不是语义判断。零命中只说明「这段话的用词
# 在那段原文里没出现」,可能是提炼时换了说法(合理),也可能是证据指错(缺陷)。
# 这个脚本把可疑的挑出来给人看,不替人下结论。
#
# 🔴 这个指标对一类记忆**系统性低估**:标题/摘要是中文、证据是英文源码或英文会话的。
#    匹配用中文 2-gram + 英文 6 字母以上词,中文散文对英文代码结构上不可能命中 ——
#    2026-10-12 实测:修完 3 条缺陷记忆后零命中从 11 涨到 14,新增的 3 条里有 2 条
#     正好是这个形状,而它们的证据是逐字支撑的。所以零命中数**不能**当成缺陷数,
#     只能当成「需要人看的候选」。判缺陷的黄金标准是打开那一段人读。
#
# 用法: bash scripts/audit-memory-evidence.sh [--full]
#       --full  连零命中的样本也打印前几条(默认只打印统计)
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FULL=0; [ "${1:-}" = "--full" ] && FULL=1

CTR="seatloom-postgres"
PSQL(){ podman exec -e PGPASSWORD=seatloom "$CTR" psql -U seatloom -d seatloom -tAc "$1" 2>/dev/null; }

echo "审计时间 $(date '+%Y-%m-%d %H:%M:%S')"
echo

# 没有 psql 通道就明说,不要报一堆 0 假装健康
if ! PSQL "select 1" | grep -q 1; then
  echo "❌ 数据库连不上 —— 审计做不了,不要当成 0 问题"; exit 1
fi

total="$(PSQL "select count(*) from memories")"
with_l2="$(PSQL "select count(*) from memories where l2_file_path is not null and l2_line_start is not null")"
no_l2="$(PSQL "select count(*) from memories where l2_file_path is null or l2_line_start is null")"

echo "── 总量 ──"
printf "  记忆总数            %s\n" "$total"
printf "  带行号证据的        %s\n" "$with_l2"
printf "  不带行号证据的      %s\n" "$no_l2"
echo "  (不带行号的那批:席位自己的运营记忆,来源是账本事件而非历史会话文件,本就不需要行号)"
echo

python3 - <<'PY'
import os, subprocess, re, sys, unicodedata

def psql(q):
    return subprocess.run(["podman","exec","-e","PGPASSWORD=seatloom","seatloom-postgres",
        "psql","-U","seatloom","-d","seatloom","-tAc",q],
        capture_output=True,text=True).stdout

rows = psql("""select id, title, l0_summary, l2_file_path, l2_line_start, l2_line_end, source_kind
               from memories where l2_file_path is not null and l2_line_start is not null""").strip().splitlines()

def words(s):
    # 中文按 2-gram 切,英文按 6+ 字母的词。都用小写。
    s = unicodedata.normalize("NFKC", s or "").lower()
    en = set(w for w in re.findall(r"[a-z_]{6,}", s))
    zh = "".join(re.findall(r"[\u4e00-\u9fff]", s))
    grams = set(zh[i:i+2] for i in range(len(zh)-1)) if len(zh) >= 2 else set()
    return en | grams

buckets = {"open_strong":0, "open_weak":0, "open_zero":0, "not_open":0}
zero_hits = []
weak_hits = []

for r in rows:
    p = r.split("|")
    if len(p) < 7: continue
    mid, title, summ, fp, s, e, sk = p[0], p[1], p[2], p[3], int(p[4]), int(p[5]), p[6]
    path = os.path.expanduser(fp)
    if not os.path.isfile(path):
        buckets["not_open"] += 1; continue
    try:
        lines = open(path, encoding="utf-8", errors="ignore").readlines()
    except Exception:
        buckets["not_open"] += 1; continue
    if e > len(lines) or s < 1 or e < s:
        buckets["not_open"] += 1; continue
    body = "".join(lines[s-1:e])
    kws = words(title) | words(summ)
    if not kws:
        buckets["open_weak"] += 1; continue
    body_l = body.lower()
    hit = sum(1 for k in kws if k in body_l)
    ratio = hit / len(kws)
    if ratio >= 0.5:
        buckets["open_strong"] += 1
    elif ratio >= 0.15:
        buckets["open_weak"] += 1
        weak_hits.append((mid, title, hit, len(kws)))
    else:
        buckets["open_zero"] += 1
        zero_hits.append((mid, title, hit, len(kws), fp, s, e))

n = len(rows)
print("── 证据审计(全量 %d 条) ──" % n)
print("  区间开得出 + 支撑强(命中率 ≥50%%)   %3d  %s" % (buckets["open_strong"], "█"*int(40*buckets["open_strong"]/max(n,1))))
print("  区间开得出 + 支撑弱(15%%–50%%)       %3d  %s" % (buckets["open_weak"], "█"*int(40*buckets["open_weak"]/max(n,1))))
print("  区间开得出 + 零命中(<15%%)           %3d  %s" % (buckets["open_zero"], "█"*int(40*buckets["open_zero"]/max(n,1))))
print("  区间开不出(路径不在/行号越界)        %3d  %s" % (buckets["not_open"], "█"*int(40*buckets["not_open"]/max(n,1))))
print()

if zero_hits:
    print("── 开得出但零命中:这些话的用词在它指向的原文里没出现 ──")
    print("  (可能是提炼时换了说法,也可能证据指错 —— 脚本不判,只挑出来)")
    for mid, title, hit, tot, fp, s, e in zero_hits[:12]:
        print("   · %s" % title[:66])
        print("     %s : %d-%d  (命中 %d/%d)" % (os.path.basename(fp), s, e, hit, tot))
    if len(zero_hits) > 12: print("     …另 %d 条" % (len(zero_hits)-12))
    print()

if weak_hits:
    print("── 支撑弱的(命中率 15%%–50%%),共 %d 条,前 8 条 ──" % len(weak_hits))
    for mid, title, hit, tot in weak_hits[:8]:
        print("   · %s  (命中 %d/%d)" % (title[:66], hit, tot))
    print()

# 结论必须能报错
if buckets["not_open"] > 0:
    print("❌ 有 %d 条的证据区间开不出 —— 这是缺陷,不是噪音" % buckets["not_open"]); sys.exit(1)
if buckets["open_zero"] > 0:
    print("⚠️  %d 条开得出但零命中 —— 这是**需要人看的候选**,不是缺陷数" % buckets["open_zero"])
    print("    其中标题中文+证据英文的那批,是匹配指标系统性低估,不是证据有问题;")
    print("    判缺陷请打开那一段人读。详见脚本头部说明。")
    sys.exit(2)
print("✅ 全部开得出,且全部有实质词汇重叠")
PY
