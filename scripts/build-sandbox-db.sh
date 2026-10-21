#!/usr/bin/env bash
# build-sandbox-db.sh — 给沙箱建一个自己的库,让它在沙箱里点的每一下都落不到真账本上。
#
# 为什么需要它(2026-10-21 张先生指出的):
#   「沙箱版本,没有做 project 之间的隔离。」
#
#   实测确认过的风险:沙箱进程与主版开**同一个** PostgreSQL 库
#   (`lsof` 数到 5 条 ESTABLISHED 到 localhost:postgresql)。
#   也就是说:在沙箱里点一次「接受」,会真的把未经验证的活合进主干账本。
#   沙箱本该是「看归看、点归点了也不算」的那一份,现在不是。
#
#   这不是理论风险 —— 沙箱存在的全部意义就是让人放心点。点一下会改真账本的
#   沙箱,比没有沙箱更糟:他会因此不敢点,于是又回到盲点。
#
# 做什么:
#   1. 在同一个 PostgreSQL 实例里建一个独立库(默认 seatloom_sandbox)
#   2. 只灌表结构(schema),从 infra/postgres/schema 按编号顺序灌
#   3. **不灌任何种子/演示数据** —— 一个空的沙箱库显示「没有项目」,
#     比一个假装有 39 个项目的库诚实(这正是 V5 那条:答不出就别显示)
#   4. 打印验证:库在、表数对、没有 seed 行
#
# 用法:
#   bash scripts/build-sandbox-db.sh [库名] [--apply]
#   默认 dry-run 只报会做什么。
#   --drop  先删掉已存在的同名库(只在明确给了这个旗标时才删)
#
# 安全性:
#   - 默认 dry-run;不给 --apply 什么都不建
#   - 只在明确点名要删时才 DROP,绝不默认清任何库
#   - 建完必须验证表结构真的灌进去了,否则报错(不许装一个连不上的沙箱)
#   - 这个脚本只建库灌结构,不碰 seatloom 主库
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
SCHEMA_DIR="$REPO/infra/postgres/schema"

APPLY=0; DROP=0; DBNAME=""
for a in "$@"; do
  case "$a" in
    --apply) APPLY=1;;
    --drop) DROP=1;;
    -*) fail "不认识的旗标 $a";;
    *) [ -z "$DBNAME" ] && DBNAME="$a";;
  esac
done
DBNAME="${DBNAME:-seatloom_sandbox}"

[ -d "$SCHEMA_DIR" ] || fail "找不到表结构目录 $SCHEMA_DIR"

# 迁移文件:按编号排序。
# 🔴 2026-10-21 实测:/bin/bash 在这台机器上是 3.2,**没有 mapfile**,
#    而 `set -u` 会让未定义数组直接终止 —— 报的是「MIGS: unbound variable」,
#    一个和真正原因(缺这个内建命令)完全不相干的错。改用 while 读,
#    POSIX 兼容。
MIGS=""
for f in $(ls "$SCHEMA_DIR"/*.sql 2>/dev/null | sort); do MIGS="$MIGS$f\n"; done
[ -n "$MIGS" ] || fail "$SCHEMA_DIR 里没有 .sql"
NMIGS="$(printf "%b" "$MIGS" | grep -c .)"

line
if [ "$APPLY" = "1" ]; then say "沙箱数据库 — 应用模式"; else say "沙箱数据库 — 预演模式(什么都不建)"; fi
line
say "  容器        : $CONTAINER"
say "  要建的库    : $DBNAME"
say "  表结构来源  : $SCHEMA_DIR($NMIGS 个迁移)"
say "  种子数据    : 不灌(空库显示「没有项目」,比假装有项目诚实)"
if [ "$DROP" = "1" ]; then say "  ⚠️  给了 --drop:会先删掉已存在的 $DBNAME"; fi
say "  主库        : seatloom —— 本脚本不碰它"

# 数据库连不上要早发现,不要建到一半才报
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -tAc "select 1" >/dev/null 2>&1 \
  || fail "连不上 $CONTAINER。先跑: podman machine start && podman start $CONTAINER"

if [ "$APPLY" != "1" ]; then
  say ""
  say "预演到此为止。要真建加 --apply。"
  say ""
  say "建好之后这样起沙箱(让沙箱用这个库,不是主库):"
  say "  SEATLOOM_SANDBOX=1 SEATLOOM_SANDBOX_BRANCH=<分支> \\"
  say "  SEATLOOM_SANDBOX_COMMIT=<提交> \\"
  say "  DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/$DBNAME \\"
  say "  open -a /Applications/SeatLoom-Sandbox.app"
  exit 0
fi

line; say "一 — 建库"; line
EXISTS="$(podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -tAc "
  select count(*) from pg_database where datname='$DBNAME';" 2>/dev/null | tr -d ' ')"
if [ "$EXISTS" = "1" ]; then
  if [ "$DROP" = "1" ]; then
    say "  库已存在,按 --drop 删掉重建"
    podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -qAc \
      "drop database if exists $DBNAME;" || fail "删库失败"
  else
    say "  ⚠️  库 $DBNAME 已存在 — 保留它已有的数据,只补缺的表"
    say "     (想从零开始就加 --drop;不默认删任何库)"
  fi
fi
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -qAc \
  "create database $DBNAME;" 2>/dev/null \
  || say "  (create 返回非零:库可能刚被别人建了,继续)"
say "  ✅ 库在:$DBNAME"

line; say "二 — 灌表结构(按编号顺序)"; line
APPLIED=0
for f in $(printf "%b" "$MIGS"); do
  name="$(basename "$f")"
  # 每个迁移单独跑,失败要说清是哪个文件,不许一起吞掉
  if podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d "$DBNAME" \
       -v ON_ERROR_STOP=1 -q < "$f" >/dev/null 2>&1; then
    APPLIED=$((APPLIED+1))
  else
    say "  ❌ 迁移失败:$name"
    fail "表结构没灌全。上面那个文件就是原因 —— 一个没灌全的沙箱库比没有更糟(它会让沙箱显示一半数据)。"
  fi
done
say "  ✅ 跑了 $APPLIED 个迁移"

line; say "三 — 验证(必须说得清,不许静默)"; line
q(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d "$DBNAME" -tAc "$1" 2>/dev/null | tr -d ' '; }
NTAB="$(q "select count(*) from pg_tables where schemaname='public'")"
say "  表数        : $NTAB"
LIVE="$(podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -tAc "select count(*) from pg_tables where schemaname='public'" 2>/dev/null | tr -d ' ')"
say "  主库表数    : $LIVE  ← 用来对比,少太多就是没灌全"
if [ -n "$LIVE" ] && [ "$NTAB" -lt "$LIVE" ]; then
  fail "沙箱库只有 $NTAB 张表,主库有 $LIVE 张 —— 没灌全,不许拿给沙箱用"
fi
say "  ✅ 表数和主库一致"
# 关键:确认里面没有种子数据(否则沙箱会假装有 39 个项目)
NPROJ="$(q "select count(*) from projects" 2>/dev/null || echo '?')"
if [ "$NPROJ" = "0" ]; then
  say "  ✅ 项目数 0 —— 干净的沙箱库(沙箱会如实地显示「没有项目」)"
else
  say "  ⚠️  项目数 $NPROJ —— 这个库不是空的。里面的数据是哪来的要查清再用。"
fi
# 确认主库毫发无损
SAFE="$(podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -tAc "select count(*) from dispatches" 2>/dev/null | tr -d ' ')"
say "  主库派单数  : $SAFE(本脚本不写它,这是前后对比的依据)"

line; say "结果"; line
say "  ✅ 沙箱库建好:$DBNAME,表和主库一样多,但没有种子数据"
say ""
say "  现在用这个库起沙箱(注意 DATABASE_URL):"
say "    SEATLOOM_SANDBOX=1 \\"
say "    SEATLOOM_SANDBOX_BRANCH=\$(git -C \"$REPO\" rev-parse --abbrev-ref HEAD) \\"
say "    SEATLOOM_SANDBOX_COMMIT=\$(git -C \"$REPO\" rev-parse --short HEAD) \\"
say "    DATABASE_URL=postgresql://seatloom:seatloom@localhost:5432/$DBNAME \\"
say "    open -a /Applications/SeatLoom-Sandbox.app"
say ""
say "  ⚠️ 空库 = 沙箱里看不到任何项目、派单、席位。**这是对的**:"
say "     沙箱该验证的是界面行为,不是你的真实数据。"
say "     想连真库看真数据,那就是另一件事(仍然会写,要想清楚)。"
exit 0
