#!/usr/bin/env bash
# apply-schema-to-live.sh — 把 track 上已验绿的表结构落到正在跑的数据库。
#
# 为什么要脚本:Aegis 在"改 live 数据库"上被权限分类器封顶(当日被拦 4 次,每次都拦对了),
# 所以这一步必须由张先生执行。
#
# 为什么不直接用 scripts/apply-db.sh:那个脚本要宿主机的 psql(这台没装),而且它会连 seed
# 一起重跑 —— 在一个已有 60 条派单、真实数据的库上重跑 seed 是不必要的风险。
# 这里只做一件事:**按编号顺序把缺的迁移 apply 进去**,不碰 seed、不建库、不删任何东西。
#
# 这些迁移都是 additive + 幂等(第二遍 apply 零 catalog 写入),所以重复跑是安全的。
# 跑完会打印验证结果 —— 把输出贴给 Aegis。
#
# 用法: bash scripts/apply-schema-to-live.sh
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
SCHEMA_DIR="$REPO/infra/postgres/schema"

run_sql(){ podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 -q "$@"; }
query(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -tAc "$1" 2>/dev/null; }

# 文件必须走 stdin 管给容器里的 psql,**不能用 -f 传宿主机路径** ——
# psql 跑在容器里,会去容器自己的文件系统找那个路径,必然 "No such file or directory"。
# (2026-10-11 第一版就是这个 bug。同一个脚本家族里的第二次宿主机/容器路径混淆:
#  第一次是 podman exec 漏了 -i,heredoc 到不了 psql。)
apply_file(){ run_sql < "$1"; }

if ! query "select 1" | grep -q 1; then
  echo "❌ 数据库连不上。先跑: podman machine start && podman start $CONTAINER"
  exit 1
fi

echo "数据库连上了。现在的表结构状态:"
printf "  memories 表          %s\n"         "$([ "$(query "select count(*) from pg_tables where tablename='memories'")"         = "1" ] && echo 已有 || echo 缺)"
printf "  acceptance_plan 列   %s\n"         "$([ "$(query "select count(*) from information_schema.columns where table_name='acceptances' and column_name='acceptance_plan'")" = "1" ] && echo 已有 || echo 缺)"
echo

# 按编号顺序逐个 apply。都是幂等的,已经有的会原地跳过。
failed=0
for f in 015_knowledge_layer.sql 016_acceptance_plan.sql 017_acceptance_cancel.sql 018_memory_hardening.sql; do
  p="$SCHEMA_DIR/$f"
  if [ ! -r "$p" ]; then echo "❌ 找不到 $f —— 仓库是不是不在 track 分支?"; failed=1; break; fi
  printf "apply %-28s " "$f"
  if apply_file "$p" >/tmp/apply-$f.log 2>&1; then echo "ok"
  else echo "失败 —— 详情: /tmp/apply-$f.log"; tail -5 "/tmp/apply-$f.log" | sed 's/^/    /'; failed=1; break; fi
done
echo

echo "验证结果:"
printf "  memories 表                %s\n" "$([ "$(query "select count(*) from pg_tables where tablename='memories'")" = "1" ] && echo ✅建好了 || echo ❌还是没有)"
printf "  memories.kind 列           %s\n" "$([ "$(query "select count(*) from information_schema.columns where table_name='memories' and column_name='kind'")" = "1" ] && echo ✅有 || echo ❌缺)"
printf "  memories 行级溯源列        %s\n" "$([ "$(query "select count(*) from information_schema.columns where table_name='memories' and column_name like 'l2_line%'")" -ge 1 ] && echo ✅有 || echo ❌缺)"
printf "  项目层级列                 %s\n" "$([ "$(query "select count(*) from information_schema.columns where table_name='projects' and column_name='parent_project_id'")" = "1" ] && echo ✅有 || echo ❌缺)"
printf "  验收方案列                 %s\n" "$([ "$(query "select count(*) from information_schema.columns where table_name='acceptances' and column_name='acceptance_plan'")" = "1" ] && echo ✅有 || echo ❌缺)"
printf "  作废卡片状态               %s\n" "$(query "select pg_get_constraintdef(oid) from pg_constraint where conname='acceptances_status_vocab'" | grep -q cancelled && echo ✅支持 || echo ❌不支持)"
echo
printf "  现有数据没被动:派单 %s 条,验收卡 %s 张\n" "$(query "select count(*) from dispatches")" "$(query "select count(*) from acceptances")"
echo
[ "$failed" = "0" ] && echo "全部完成。把上面这段贴给 Aegis,他会解封等着的派单。" \
                    || echo "有步骤失败了 —— 把上面这段贴给 Aegis,不要自己重试。"
exit "$failed"
