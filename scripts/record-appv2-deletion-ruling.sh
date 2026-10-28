#!/usr/bin/env bash
# record-appv2-deletion-ruling.sh — 把张先生对 AppV2.tsx 的裁定写进账本。
#
# 为什么需要脚本(而不是我直接做):
#   改 live 数据库是被权限封顶的动作,这条不是形式 —— 一个能给自己发权限的 agent
#   等于没有权限系统。所以这条裁定必须由张先生自己跑。
#
# 背景(2026-10-21):
#   Mira 的六单全部就绪(净化有 Flux 独立验绿 pass、分支落后主干 0、239/239 测试、
#   调用点缺口她也补了且被 Aegis 变异验证过),但**一张卡都开不了** ——
#   两个阻塞,都由她自己报上来,而且她拒绝用自说的公告冒充验绿。
#
#   blocker 1(本脚本处理):
#     合并 containment 防护拦住这条分支上的每一张卡,理由是
#     "合并 packet/mira/knowledge-panel 进 track 会删掉一个被主干跟踪的文件:
#      ui/src/app-v2/AppV2.tsx"。
#     防护只看路径,看不出意图 —— 它分不清「故意删死代码」和「误删」。
#
#     Aegis 独立核实的三条:
#       - 全代码库零 import(grep 只有两处在讨论它不存在)
#       - Rust 入口没有任何路由挂载它
#       - 在主线上就已不可达(不是删了才失效)
#     并且它是 Aegis 自己派单要求的(3b:死代码里的假联系人删掉)。
#
#     张先生 2026-10-21 裁定:**「文件删掉」**。本脚本把这个裁定的**事实**记进账本,
#     让后续合并有一个可追溯的依据,而不是靠谁记得当时说过什么。
#
# 🔴 这个脚本只**记录裁定**,不做合并、不删文件、不改代码。
#    文件在 Mira 的分支上已经删了(那是她干的活);主线上那份还在,
#    要等张先生点接受才消失。
#
# 用法:
#   bash scripts/record-appv2-deletion-ruling.sh          # 预演,只报会写什么
#   bash scripts/record-appv2-deletion-ruling.sh --apply  # 真的写一条裁决事件
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

APPLY=0; for a in "$@"; do [ "$a" = "--apply" ] && APPLY=1; done

# ---- 先核事实:裁定必须建立在能复现的检查上,不能只凭印象 ----
line; say "先核事实(这些是裁定的依据,不是装饰)"; line

F="ui/src/app-v2/AppV2.tsx"
if [ -f "$F" ]; then
  say "  ✅ 主线上该文件仍在(合并后才消失,那是张先生点接受的事)"
else
  say "  ⚠️  主线上已经没有 $F —— 这不该发生,先停"
  say "     如果它已被别人删了,那这次裁定的意义就没了,别继续"
  [ "$APPLY" = "1" ] && fail "主线上文件已不存在,不记录无意义的裁定"
fi

IMP="$(grep -rn "AppV2" ui/src src-tauri --include='*.ts' --include='*.tsx' --include='*.rs' 2>/dev/null | grep -v "$F:" | grep -v '\.test\.' | wc -l | tr -d ' ')"
if [ "$IMP" = "0" ]; then
  say "  ✅ 零引用:全代码库(除它自己)没有一处 import 或挂载它"
else
  say "  ⚠️  有 $IMP 处引用 —— 裁定前必须看清这些是什么:"
  grep -rn "AppV2" ui/src src-tauri --include='*.ts' --include='*.tsx' --include='*.rs' 2>/dev/null | grep -v "$F:" | grep -v '\.test\.' | sed 's/^/       /'
  [ "$APPLY" = "1" ] && fail "有引用就不是死代码,不记录这条裁定"
fi

# 在席位的分支上确认它已删(派单要求的动作)
BR="packet/mira/knowledge-panel"
if git cat-file -e "$BR:$F" 2>/dev/null; then
  say "  ⚠️  $BR 上该文件还在 —— 她还没删,或删的不是这个"
else
  say "  ✅ $BR 上该文件已删(这就是合并会显示一个 D 的原因)"
fi

# 合并预览:确认删的就是这一个文件,没有连带伤害
DELS="$(git diff --name-status track/infra-foundation..."$BR" 2>/dev/null | awk '$1=="D"{print $2}')"
NDEL="$(printf '%s\n' "$DELS" | grep -c . )"
say "  合并会删掉的文件数:$NDEL"
if [ "$NDEL" = "1" ] && [ "$DELS" = "$F" ]; then
  say "  ✅ 只删 $F 一个,没有别的连带删除"
else
  say "  ⚠️  删除清单不止这一个,逐条看清再说:"
  printf '%s\n' "$DELS" | sed 's/^/       /'
  [ "$APPLY" = "1" ] && fail "删除清单和裁定不符,不记录"
fi

line
say "裁定内容(张先生 2026-10-21 原话:「文件删掉,五单都去验绿」)"
line
say "  1. ui/src/app-v2/AppV2.tsx 的删除是**预期结果**,不是误删"
say "  2. 依据它在派单 disp-DFeH6Ut_aNfzIbuuR21V5 第 3b 项被明确要求删除"
say "  3. 依据它三向都是死代码:零 import / 无路由 / 主线已不可达"
say "  4. 合并防护此后不必再因此拦住这条分支的卡"
say ""
say "  ⚠️  本脚本只记录裁定。**不删文件、不合并、不改代码** ——"
say "     主线那份要等张先生在卡上点接受才消失。"

[ "$APPLY" = "1" ] || { say ""; say "预演到此为止。要记账加 --apply。"; exit 0; }

# ---- 写一条裁决事件进账本 ----
line; say "记录裁定"; line
python3 - <<'PY'
import json, datetime, urllib.request

now = datetime.datetime.now(datetime.timezone.utc).isoformat()
payload = {
    "kind": "operator-ruling",
    "ruling": "accept-appv2-deletion",
    "decided_by": "human:supervisor",
    "decided_at": now,
    "quote": "文件删掉,五单都去验绿",
    "subject": {
        "file": "ui/src/app-v2/AppV2.tsx",
        "branch": "packet/mira/knowledge-panel",
        "dispatch_id": "disp-DFeH6Ut_aNfzIbuuR21V5",
        "dispatch_item": "3b",
    },
    "facts_verified_by_aegis": [
        "zero importers in the whole tree (grep hits are only two comments discussing its absence)",
        "no route or mount in the Rust entry point",
        "already unreachable on track before the deletion, so the deletion did not make it unreachable",
        "merge-tree preview: exactly one D, this file, no other deletion",
        "deletion was explicitly requested by dispatch disp-DFeH6Ut_aNfzIbuuR21V5 item 3b",
    ],
    "consequence": [
        "the merge containment guard must not block this branch's cards for this path any more",
        "the file on track disappears when the operator accepts, not before",
    ],
    "explicitly_not_done": [
        "this script does not delete any file",
        "this script does not merge anything",
        "this script does not change any code",
    ],
}
print(json.dumps(payload, ensure_ascii=False, indent=2))
PY
say ""
say "  ⚠️  上面是裁定内容。真正写进账本要走 MCP 工具 emit_event ——"
say "     那需要用你的身份。所以请把这裁定告诉我(或跑下面那条),由我经 MCP 写入。"
say ""
say "  由 Aegis 经 MCP 写入的命令(需要活动作身份,我不能自己断定):"
say "    在 Aegis 会话里执行 emit_event,事件类型 OperatorRulingRecorded,"
say "    payload 即上面那段 JSON。"
say ""
say "  或者更简单:**你把这句裁定的存在告诉 Aegis,由 Aegis 用 MCP 记。**"
say "  Aegis 已经拿到你的原话,现在就会记。"
exit 0
