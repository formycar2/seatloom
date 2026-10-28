#!/usr/bin/env bash
# refresh-sandbox-db.sh — 把正式库的一份快照灌进沙箱库,让沙箱尽量对齐正式库。
#
# 为什么需要它(2026-10-21 张先生的裁定):
#   「不需要清掉,我需要沙箱尽可能对齐正式库。」
#
#   这和 build-sandbox-db.sh 的主张不同,而他说得对:
#   一个空沙箱显示「没有项目」,人就没法在沙箱里验任何真实行为 ——
#   点下去是空的,看不出这批改动在真实数据上是对是错。
#   沙箱该验的是**行为**,数据要真;只是**不能写回真账本**。
#
# 做法:pg_dump 从主库出裸数据 -> 灌进沙箱库。同一实例,库不同。
#
# 🔴 三条必须守住的边界,少一条沙箱就从「不敢点」变成「点了毁东西」:
#
#   1. 方向只能是 主库 -> 沙箱库,**绝不反向**。
#      脚本里把两个库名写死成参数,并断言目标不是 seatloom ——
#      把 dump 灌回主库会当场清掉真账本。
#
#   2. 沙箱库会被整库替换(先 drop 再建)。
#      这是唯一能「对齐」的办法,所以 --drop 由 refresh 自己内部完成,
#      但**目标永远只能是沙箱库**。
#
#   3. 灌完必须验证:表数、各表行数与主库一致。
#      一个灌了一半的沙箱比空沙箱更糟 —— 它会显示一半数据,
#      让人以为那就是真实情况。
#
# 用法:
#   bash scripts/refresh-sandbox-db.sh [沙箱库名] [--apply]
#   默认 dry-run:只报会做什么、主库现在多少行。
#   --apply:真的灌。
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
LIVE_DB="seatloom"

APPLY=0; TARGET=""
for a in "$@"; do
  case "$a" in
    --apply) APPLY=1;;
    -*) fail "不认识的旗标 $a";;
    *) [ -z "$TARGET" ] && TARGET="$a";;
  esac
done
TARGET="${TARGET:-seatloom_sandbox}"

# 🔴 边界第一道:目标绝不能是主库。写成断言而不是信任调用方 ——
#    一个能把 dump 灌回主库的脚本,比没有脚本危险得多。
if [ "$TARGET" = "$LIVE_DB" ]; then
  fail "目标库不能是 $LIVE_DB(正式库)。这个脚本只往沙箱库灌,绝不反向。"
fi

podman exec "$CONTAINER" pg_dump --version >/dev/null 2>&1 \
  || fail "容器里没有 pg_dump"

q(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d "$1" -tAc "$2" 2>/dev/null | tr -d ' '; }

line
if [ "$APPLY" = "1" ]; then say "刷新沙箱库 — 应用模式"; else say "刷新沙箱库 — 预演模式(什么都不动)"; fi
line
say "  方向        : $LIVE_DB  ──▶  $TARGET   (单向,绝不反向)"
say "  主库现状    : $(q "$LIVE_DB" 'select count(*) from dispatches') 张派单 / $(q "$LIVE_DB" 'select count(*) from canonical_events') 条事件 / $(q "$LIVE_DB" 'select count(*) from memories') 条记忆"
say "  沙箱现状    : $(q "$TARGET" 'select count(*) from dispatches' 2>/dev/null || echo '库不存在') 张派单"
say "  做法        : 整库替换(drop -> create -> pg_dump 灌入)"
say ""
say "  ⚠️ 灌完之后:沙箱有和正式库一样的数据,但**写它不影响正式库**。"
say "     在沙箱里点「接受」只会改沙箱自己那份。"

[ "$APPLY" = "1" ] || { say ""; say "预演到此为止。要真灌加 --apply。"; exit 0; }

line; say "一 — 从正式库导出(只读,不写它)"; line
DUMP="$(mktemp)"
trap 'rm -f "$DUMP"' EXIT
# pg_dump 在容器里跑,要连的是容器内的 localhost;--format=plain 才能用 psql 灌回去
if ! podman exec -e PGPASSWORD=seatloom "$CONTAINER" pg_dump -U seatloom -d "$LIVE_DB" > "$DUMP" 2>/dev/null; then
  fail "pg_dump 失败。正式库只读访问没成功 —— 不动任何东西。"
fi
[ -s "$DUMP" ] || fail "导出文件是空的"
say "  导出大小    : $(wc -c < "$DUMP" | tr -d ' ') 字节"

line; say "二 — 替换沙箱库"; line
if ! podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -qAc \
     "select pg_terminate_backend(pid) from pg_stat_activity where datname='$TARGET' and pid<>pg_backend_pid();" >/dev/null 2>&1; then
  say "  (没有别的连接占着 $TARGET,或清理失败 — 继续)"
fi
say "  先断开可能连着沙箱库的会话(否则 drop 会失败)"
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -qAc \
  "drop database if exists $TARGET;" || fail "删不掉旧沙箱库 $TARGET"
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -qAc \
  "create database $TARGET;" || fail "建库失败 $TARGET"
say "  ✅ 空库建好:$TARGET"

line; say "三 — 灌入快照"; line
if ! podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d "$TARGET" \
       -v ON_ERROR_STOP=1 -q < "$DUMP" >/dev/null 2>&1; then
  fail "灌入失败。沙箱库可能处于灌了一半的状态 —— 重跑本脚本,不要手动接着灌。"
fi
say "  ✅ 快照灌入完成"

line; say "四 — 验证对齐(不是一个数,全部要对上)"; line
NTAB="$(q "$TARGET" "select count(*) from pg_tables where schemaname='public'")"
LTAB="$(q "$LIVE_DB" "select count(*) from pg_tables where schemaname='public'")"
say "  表数        : 沙箱 $NTAB / 正式 $LTAB"
[ "$NTAB" = "$LTAB" ] || fail "表数不一致($NTAB vs $LTAB)—— 沙箱没对齐,别拿它验东西。"

BAD=0
for t in dispatches canonical_events memories acceptances projects seats archived_sessions; do
  s="$(q "$TARGET" "select count(*) from $t")"
  l="$(q "$LIVE_DB" "select count(*) from $t")"
  if [ "$s" = "$l" ]; then say "  ✅ $t: $s"
  else say "  ❌ $t: 沙箱 $s / 正式 $l"; BAD=1; fi
done
[ "$BAD" = "0" ] || fail "上面有表行数不一致 —— 沙箱不是正式库的忠实副本。"

line; say "结果"; line
say "  ✅ 沙箱库 $TARGET 现在和正式库 $LIVE_DB 数据一致"
say ""
say "  用这个库起沙箱(DATABASE_URL 指向沙箱库,不是正式库):"
say "    SEATLOOM_SANDBOX=1 \\"
say "    SEATLOOM_SANDBOX_BRANCH=\$(git -C \"$REPO\" rev-parse --abbrev-ref HEAD) \\"
say "    SEATLOOM_SANDBOX_COMMIT=\$(git -C \"$REPO\" rev-parse --short HEAD) \\"
say "    DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/$TARGET \\"
say "    open -a /Applications/SeatLoom-Sandbox.app"
say ""
say "  从现在起在沙箱里点任何东西,只改沙箱这一份。"
say "  想重新对齐(正式库又前进了),再跑一次本脚本。"
exit 0
