#!/usr/bin/env bash
# apply-020-operator-decisions.sh — 把 020(OperatorDecision)落到 live 库。
#
# 为什么要有这个(2026-10-27,张先生截图报的):
#   SeatLoom UI 的「等你裁定」面板在跑,但显示 `postgres error: db error`。
#   根因:`operator_decisions` 表在 live 库里**不存在**。
#   020_operator_decisions.sql 在 track 上、也在 apply-db.sh 里分类好了,
#   但**从来没有人对 live 库跑过它**。
#
#   这正是 Aegis 早上记下的那条教训的又一例:
#   「迁移在 track 上 ≠ 迁移在跑着的库里」。
#   不同的是这次有可见后果 —— 屏幕上一条红字。
#
# 🔴 为什么由张先生跑而不是 Aegis:Aegis 被自动权限分类器封顶,
#    不能写 live 数据库。规矩是把动作写成可审计、可重复跑的脚本,
#    然后给张先生一条命令。不绕、不换工具、不让席位代跑。
#
# 🔴 默认 dry-run:什么都不写,只核前提 + 打印将要跑的文件。
#    一条会改 live 库的命令,默认必须是「不做事」的那一档。
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
DB="${SEATLOOM_DB:-seatloom}"
USER="${SEATLOOM_DB_USER:-seatloom}"
PASS="${SEATLOOM_DB_PASS:-seatloom}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SQL="$HERE/infra/postgres/schema/020_operator_decisions.sql"

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

say(){ printf '%s\n' "$*"; }
line(){ printf '%s\n' "────────────────────────────────────────────────────────────"; }

line; say "  把 020(OperatorDecision)落到 live 库"; line
say "  模式  : $([ "$APPLY" = 1 ] && echo 'APPLY — 真的写库' || echo 'DRY-RUN — 什么都不写')"
say "  文件  : $SQL"
say "  目标  : $CONTAINER / $DB"
say ""

# ── 0. 前提:不满足就停,不做半截 ────────────────────────────────────
line; say "0 — 前提条件"; line
fail=0
[ -r "$SQL" ] && say "  ✅ 迁移文件在" || { say "  ❌ 读不到 $SQL"; fail=1; }
if podman exec "$CONTAINER" true 2>/dev/null; then
  say "  ✅ 数据库容器在跑"
else
  say "  ❌ 连不上容器 $CONTAINER"; fail=1
fi
# 020 依赖 projects(id) —— 不在就不能跑
if [ "$fail" = 0 ]; then
  have_p="$(podman exec -e "PGPASSWORD=$PASS" "$CONTAINER" \
    psql -X -U "$USER" -d "$DB" -tAc \
    "select count(*) from information_schema.tables where table_name='projects';" 2>/dev/null | tr -d '[:space:]')"
  if [ "${have_p:-0}" -ge 1 ]; then
    say "  ✅ 依赖的 projects 表在"
  else
    say "  ❌ projects 表不在 —— 020 的外键会失败"; fail=1
  fi
fi
[ "$fail" != 0 ] && { say ""; say "❌ 前提不满足,停。修好再跑。"; exit 1; }
say ""

# ── 1. 现状:表在不在(决定要不要跑) ─────────────────────────────────
line; say "1 — live 库现状"; line
have="$(podman exec -e "PGPASSWORD=$PASS" "$CONTAINER" \
  psql -X -U "$USER" -d "$DB" -tAc \
  "select count(*) from information_schema.tables where table_name='operator_decisions';" 2>/dev/null | tr -d '[:space:]')"
say "  operator_decisions 表:$([ "${have:-0}" -ge 1 ] && echo '已存在' || echo '不存在 ← 这就是 UI 报 postgres error 的原因')"
say ""

# ── 2. 迁移自己声称的性质(照抄它的头部,不替它担保) ──────────────────
line; say "2 — 这个迁移声称的性质"; line
sed -n '1,12p' "$SQL" | sed 's/^/  /'
say ""

if [ "$APPLY" != 1 ]; then
  line; say "这是 DRY-RUN,什么都没改。确认后跑:"; line
  say ""
  say "    bash scripts/apply-020-operator-decisions.sh --apply"
  say ""
  exit 0
fi

# ── 3. 真的跑(stdin 喂进去:容器没挂载仓库) ────────────────────────
line; say "3 — 应用迁移"; line
if podman exec -i -e "PGPASSWORD=$PASS" "$CONTAINER" \
     psql -X -v ON_ERROR_STOP=1 -U "$USER" -d "$DB" < "$SQL" >/tmp/apply-020.log 2>&1; then
  say "  ✅ psql 跑完(ON_ERROR_STOP=1,出错会非零退出)"
else
  say "  ❌ psql 报错,看 /tmp/apply-020.log"
  tail -15 /tmp/apply-020.log | sed 's/^/    /'
  exit 1
fi
say ""

# ── 4. 读回核对:不看"执行成功",看库真的变了 ───────────────────────
line; say "4 — 读回核对"; line
now="$(podman exec -e "PGPASSWORD=$PASS" "$CONTAINER" \
  psql -X -U "$USER" -d "$DB" -tAc \
  "select count(*) from information_schema.tables where table_name='operator_decisions';" 2>/dev/null | tr -d '[:space:]')"
say "  operator_decisions 表现在:$([ "${now:-0}" -ge 1 ] && echo '存在 ✅' || echo '还是不存在 ❌')"
idx="$(podman exec -e "PGPASSWORD=$PASS" "$CONTAINER" \
  psql -X -U "$USER" -d "$DB" -tAc \
  "select count(*) from pg_indexes where tablename='operator_decisions';" 2>/dev/null | tr -d '[:space:]')"
say "  它的索引数:${idx:-0}(020 建了 2 个)"
[ "${now:-0}" -ge 1 ] || { say "❌ 读回对不上,停。"; exit 1; }
say ""
say "完成。UI 那个面板应该不再报 postgres error。"
say "如果还报,把现象告诉 Aegis —— 那说明还有别的迁移没落到 live 库。"
