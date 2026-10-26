#!/usr/bin/env bash
# set-compaction-keep-recent.sh — 把 compaction.keepRecentTokens 调到 20000。
#
# 为什么需要它(2026-10-24,张先生反馈「Aegis 心跳一把撑满 22 万,体验太差」):
#
# 实测数据(Aegis session 01a142bc,132 次心跳):
#   常规心跳(6 条消息)      ~1,400 token
#   稍忙的心跳(12 条)        ~2,500 token
#   干活的心跳(30-60 条)    7,000-11,600 token   <- bash 参数占大头(单次 18,002 字符)
#
# 所以「一把撑满」不是真的 —— 常规心跳只有 1,400 token。真正的原因是压缩不清仓:
#
#   reserveTokens     780,000  -> 阈值 220,000
#   keepRecentTokens  120,000  -> 压缩后仍保留 12 万原文 + ~6,500 摘要 = 126,500
#                                     距离阈值只剩 93,500
#
#   上游默认 keepRecentTokens 是 20,000:压缩后剩 ~26,500,腾出 95 万。
#   我(2026-10-21)同时把阈值砍到 1/4.5、keepRecent 抬到 6 倍,两个错误相乘,
#   结果 4 分钟压一次且几乎不省。
#
# 这个脚本只改 keepRecentTokens。阈值已在 2026-10-24 改成 440,000(reserveTokens 560,000)。
# 改完:
#   阈值 440,000 / 压缩后剩 ~26,500 / 可增长 ~413,500
#   按常规心跳 1,400 tok 算 ≈ 295 小时才压一次;按干活的心跳 5,000 算 ≈ 83 小时。
#
# ⚠️ 改的是全局 ~/.saih/agent/settings.json,七个会话共用。
#    已运行的进程不重读它(SettingsManager 构造时一次性 merge),所以改完要重启才生效。
#
# 用法:
#   bash scripts/set-compaction-keep-recent.sh          # 预演,只报不动
#   bash scripts/set-compaction-keep-recent.sh --apply  # 真改
set -uo pipefail

SETTINGS="${HOME}/.saih/agent/settings.json"
TARGET="${SEATLOOM_KEEP_RECENT:-20000}"
APPLY=0
for a in "$@"; do
  case "$a" in
    --apply)    APPLY=1 ;;
    -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
    *) echo "unknown option: $a" >&2; exit 2 ;;
  esac
done

if [ ! -f "$SETTINGS" ]; then
  echo "❌ 找不到 $SETTINGS" >&2
  exit 1
fi

python3 - "$SETTINGS" "$TARGET" "$APPLY" <<'PY'
import json, os, sys
path, target, apply_ = sys.argv[1], int(sys.argv[2]), sys.argv[3] == "1"
d = json.load(open(path))
c = d.get("compaction", {})
old_keep = c.get("keepRecentTokens")
old_reserve = c.get("reserveTokens", 16384)

# 从 models.json 推 contextWindow:阈值 = contextWindow - reserveTokens。
# 席位在用的三个模型窗口都是 1,000,000,这里取它们在用的最大值做估算。
cw = 1000000
mp = os.path.expanduser("~/.pi/agent/models.json")
try:
    md = json.load(open(mp))
    wins = [m.get("contextWindow", 0) for v in md.get("providers", {}).values()
            if isinstance(v, dict) for m in v.get("models", [])
            if m.get("contextWindow")]
    if wins:
        cw = max(wins)
except Exception:
    pass

thr = cw - old_reserve
after = target + 6500  # 摘要实测约 6,500 token
space = thr - after

print(f"文件            : {path}")
print(f"contextWindow   : {cw:,}  (取 models.json 里的最大值)")
print(f"reserveTokens   : {old_reserve:,}  ->  压缩阈值 = {thr:,}")
print()
print(f"keepRecentTokens: {old_keep:,}  ->  {target:,}")
print(f"压缩后仍在上下文 : ~{old_keep+6500:,}  ->  ~{after:,}")
print(f"可增长空间       : ~{max(thr-(old_keep+6500),0):,}  ->  ~{max(space,0):,}"
      f"   ({space/max(thr-(old_keep+6500),1):.1f}x)")
print()
if thr <= target * 2:
    print(f"❌ 阈值 {thr:,} 不健康(<= keepRecentTokens 的 2 倍)。先调 reserveTokens。停。")
    sys.exit(1)
print("✅ 阈值健康")

if not apply_:
    print()
    print("预演结束,未改动。真改加 --apply。")
    sys.exit(0)

c["keepRecentTokens"] = target
d["compaction"] = c
json.dump(d, open(path, "w"), indent=2, ensure_ascii=False)
open(path, "a").write("\n")
print()
print("已写入:")
print(json.dumps(d["compaction"], indent=2))
print()
print("⚠️ 已运行的七个会话不重读此文件,需重启才生效。")
PY
