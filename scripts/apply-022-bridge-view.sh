#!/usr/bin/env bash
# apply-022-bridge-view.sh — 把 session↔work 桥接视图应用到 live 库。
#
# 为什么需要它(2026-10-26):
#   Nimbus 写好了 `022_session_work_bridge_view.sql`(分支
#   `packet/nimbus/project-scoped-reads`,提交 f7bfe23),但**没应用到 live 库**。
#   实测:live 库里 `v_memory_session_join` 不存在,
#   `select count(*) from v_memory_session_join` → ERROR: relation does not exist。
#
#   这正是「迁移在 track 上 ≠ 迁移在跑着的库里」。三门里唯一真缺的就是这门
#   (`ev-session-key-readmodel-live`),三张单从 10-16 卡到现在就是因为它在。
#
# 🔴 这是改 live 数据库,Aegis 被权限封顶,必须由张先生执行。
#    默认 dry-run:只报会做什么、现在缺什么,一行不改。
#
# ⚠️ 这个迁移不是幂等的:它用裸 `CREATE VIEW`(不是 CREATE OR REPLACE),
#    重跑会报 "already exists"。所以本脚本第二次跑会**报错退出**,
#    不静默跳过、不 DROP 后重建(DROP 会短暂让读它的人看不到视图)。
#    真要重建,先 `DROP VIEW v_memory_session_join;` 再跑 —— 那是明摆着的动作,
#    不是脚本偷偷做的。
#
# 用法:
#   bash scripts/apply-022-bridge-view.sh            # 预演
#   bash scripts/apply-022-bridge-view.sh --apply     # 真应用
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
DB="${SEATLOOM_DB:-seatloom}"
APPLY=0; for a in "$@"; do [ "$a" = "--apply" ] && APPLY=1; done

# 迁移文件在 Nimbus 的分支上,还没合并到 track。允许用环境变量指过去,
# 默认找 worktree 里那份(存在就用,不存在就报错,不猜)。
SQL="${SEATLOOM_022_SQL:-$REPO/../seatloom-worktrees/nimbus/infra/postgres/schema/022_session_work_bridge_view.sql}"

line
if [ "$APPLY" = "1" ]; then say "应用 022 桥接视图 — 应用模式(真改 live 库)"; else say "应用 022 桥接视图 — 预演模式(一行不改)"; fi
line

[ -f "$SQL" ] || fail "找不到迁移文件:$SQL
  它在 Nimbus 的分支上(packet/nimbus/project-scoped-reads,提交 f7bfe23),
  还没合并到 track。用 SEATLOOM_022_SQL=<路径> 指定。"

say "  迁移文件: $SQL"
sha="$(shasum -a 256 "$SQL" | awk '{print $1}')"
say "  sha256:    ${sha:0:16}…"

q(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d "$DB" -tAc "$1" 2>&1; }

# ── 前置1:库里现在有没有(有就不该再建) ──
exists="$(q "select count(*) from information_schema.views where table_name='v_memory_session_join';" | tr -d ' ')"
if [ "$exists" = "1" ]; then
  say "  ℹ️  live 库里视图已存在 —— 不需要应用。若要重建,先手工 DROP VIEW。"
  say "     (本脚本不 DROP:DROP 会让正在读它的人短暂看不到视图)"
  exit 0
fi
say "  ✅ 当前 live 库里没有这个视图,可以应用"

# ── 前置2:它依赖的表在不在 ──
for t in memories archived_sessions; do
  n="$(q "select count(*) from information_schema.tables where table_name='$t';" | tr -d ' ')"
  [ "$n" = "1" ] || fail "依赖的表 $t 不存在 —— 先弄清为什么"
  say "  ✅ 依赖表 $t 在"
done

[ "$APPLY" = "1" ] || { say ""; say "预演到此为止。要真应用加 --apply。"; exit 0; }

# ── 应用 ──
say ""; say "应用…"
# psql 在容器里跑,所以要把文件内容喂进去(podman exec -i)
if ! podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" \
     psql -X -U seatloom -d "$DB" -v ON_ERROR_STOP=1 -q < "$SQL" 2>&1 | tail -5; then
  fail "psql 报错 —— 什么都没建成。把上面的错误贴给 Aegis。"
fi

# ── 验证:按迁移文件尾部写的数字核(不只看"建成了") ──
line; say "验证(按 022 自己写的期望数字核)"; line
run="$(q "select count(*) from v_memory_session_join;" | tr -d ' ')"
[ "$run" = "90" ] || fail "视图行数是 $run,022 期望 90 —— 建成了但数字不对,别当成成功"
say "  ✅ v_memory_session_join 行数 = $run(022 期望 90)"

for pair in "both|87" "a1_path|3" "a2_source_ref|0" "unresolved|0"; do
  k="${pair%%|*}"; want="${pair##*|}"
  got="$(q "select count(*) from v_memory_session_join where resolved_by='$k';" | tr -d ' ')"
  if [ "$got" = "$want" ]; then say "  ✅ resolved_by=$k → $got(期望 $want)"
  else say "  ⚠️  resolved_by=$k → $got,期望 $want —— 数字不符,要查"; fi
done

a1="$(q "select count(*) from v_memory_session_join where not a1_resolves;" | tr -d ' ')"
a2="$(q "select count(*) from v_memory_session_join where not a2_resolves;" | tr -d ' ')"
say "  not a1_resolves = $a1(022 期望 0)"
say "  not a2_resolves = $a2(022 期望 3 —— 这是要暴露的 drift,不是错误)"

line; say "接下来"; line
say "  视图真的能跑了,门 `ev-session-key-readmodel-live` 的条件现在满足。"
say "  Aegis 会派 Flux 独立审计(他已认错一次,不能自证),通过后才放行那三张单。"
say "  重跑本脚本会报『already exists』而退出 —— 这个迁移不是幂等的,这是预期的。"
exit 0
