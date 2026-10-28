#!/usr/bin/env bash
# merge-ql-projects.sh — 把 ql_delivery 并进 project_delivery/QL,一个目录只留一个项目。
#
# 为什么需要它(2026-10-23 张先生选 A):
#   10-16 Aegis 建 QL 时留了两条项目记录,指向**同一个目录**:
#     ql_delivery          home_path=.../project_delivery/QL   绑了 seat-ql-001
#     project_delivery/QL  home_path=.../project_delivery/QL   **没绑席位**
#   张先生在 UI 里选 project_delivery/QL,下面没有 Seat —— 因为席位绑在另一条上。
#   **同一个目录两条记录,界面按项目 id 分,于是"有席位的那个"他看不到。**
#
# 做什么:
#   1. 把 seat-ql-001 的绑定从 ql_delivery 搬到 project_delivery/QL(原样保留列)
#   2. ql_delivery 名下若有数据(派单/事件/归档),改指到 project_delivery/QL —— 不删历史
#   3. 删掉 ql_delivery 这条空壳项目记录
#
# 🔴 这是改 live 账本,Aegis 被权限封顶,必须由张先生执行。
#    默认 dry-run:只打印会做什么、影响哪些行,一行不改。
#
# 用法:
#   bash scripts/merge-ql-projects.sh          # 预演
#   bash scripts/merge-ql-projects.sh --apply   # 真改
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
OLD="ql_delivery"
NEW="project_delivery/QL"
APPLY=0; for a in "$@"; do [ "$a" = "--apply" ] && APPLY=1; done

q(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d seatloom -tAc "$1" 2>/dev/null; }
x(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d seatloom -qAc "$1" 2>&1; }

line
if [ "$APPLY" = "1" ]; then say "合并 QL 项目 — 应用模式(真改账本)"; else say "合并 QL 项目 — 预演模式(一行不改)"; fi
line

# ── 前置:两条记录都必须在,且指向同一目录 ──
OLD_PATH="$(q "select home_path from projects where id='$OLD'" | tr -d ' ')"
NEW_PATH="$(q "select home_path from projects where id='$NEW'" | tr -d ' ')"
[ -n "$OLD_PATH" ] || fail "找不到项目 $OLD"
[ -n "$NEW_PATH" ] || fail "找不到项目 $NEW —— 你 UI 里那个必须存在"
if [ "$OLD_PATH" != "$NEW_PATH" ]; then
  fail "两条记录的 home_path 不同($OLD_PATH vs $NEW_PATH)—— 不是同一个目录,先人工确认再合"
fi
say "  ✅ 两条都指向同一目录:$NEW_PATH"

# ── 前置:NEW 下不能已有席位绑定(否则会撞车) �
NEW_BIND="$(q "select seat_id from project_role_bindings where project_id='$NEW'" | tr -d ' ')"
if [ -n "$NEW_BIND" ]; then
  fail "$NEW 下已经绑了席位 $NEW_BIND —— 先人工决定留哪个,别让脚本覆盖"
fi
say "  ✅ $NEW 下当前无绑定,可以接收"

# ── 前置:OLD 下必须确有那条绑定 ──
OLD_BIND="$(q "select seat_id||'/'||role from project_role_bindings where project_id='$OLD'" | tr -d ' ')"
[ -n "$OLD_BIND" ] || fail "$OLD 下没有席位绑定 —— 没什么可搬的"
say "  ✅ 要搬的绑定:$OLD_BIND"

# ── OLD 名下各表有多少行(决定要不要改指) ──
say ""
say "  $OLD 名下数据(会被改指到 $NEW,不删):"
TOTAL=0
for t in dispatches canonical_events acceptances workitems handoffs sessions topics memories prompt_instances channel_action_receipts review_threads; do
  n="$(q "select count(*) from $t where project_id='$OLD'" | tr -d ' ')"
  n="${n:-0}"
  [ "$n" != "0" ] && say "    $t: $n 行"
  TOTAL=$((TOTAL + n))
done
[ "$TOTAL" = "0" ] && say "    (全是 0 行 —— 这是个空壳,搬绑定就够了)"
# archived_sessions 的 project 列不是 project_id,单独看
ARCH="$(q "select count(*) from archived_sessions where project='$OLD'" | tr -d ' ')"
ARCH="${ARCH:-0}"
say "    archived_sessions.project='$OLD': $ARCH 行"

line; say "将要做"; line
say "  1. 绑定:seat-ql-001 从 $OLD 搬到 $NEW(原样保留 role 等列)"
say "  2. 上表有行的全部改指 project_id: $OLD → $NEW"
say "  3. archived_sessions.project: $OLD → $NEW"
say "  4. 删除项目记录 $OLD(空壳)"

[ "$APPLY" = "1" ] || { say ""; say "预演到此为止。要真改加 --apply。"; exit 0; }

# ── 1. 搬绑定(先插新的,成功再删旧的;顺序反了会丢绑定) ──
say ""; say "执行…"
if ! x "insert into project_role_bindings (seat_id, project_id, role, authority_doc_refs, constraints, collaboration_template_ref, active_delegation_id) select seat_id, '$NEW', role, authority_doc_refs, constraints, collaboration_template_ref, active_delegation_id from project_role_bindings where project_id='$OLD';" >/dev/null; then
  fail "插入新绑定失败 —— 什么都没改,重跑或人工看错误"
fi
x "delete from project_role_bindings where project_id='$OLD';" >/dev/null
say "  ✅ 绑定已搬到 $NEW"

# ── 2/3. 改指数据 ──
for t in dispatches canonical_events acceptances workitems handoffs sessions topics memories prompt_instances channel_action_receipts review_threads; do
  x "update $t set project_id='$NEW' where project_id='$OLD';" >/dev/null 2>&1 || say "  ⚠️  update $t 失败(该表可能无 project_id 列,跳过)"
done
x "update archived_sessions set project='$NEW' where project='$OLD';" >/dev/null 2>&1 || true
say "  ✅ 数据已改指"

# ── 4. 删空壳项目 ──
# 若还有行指着它(理论上已改指完),删会被外键挡 —— 那就报出来,不硬删
if ! x "delete from projects where id='$OLD';" >/dev/null 2>&1; then
  say "  ⚠️  删不掉 $OLD —— 还有外键引用。手动查:"
  say "     select * from dispatches where project_id='$OLD';(以及其它表)"
  say "     清干净后再 delete from projects where id='$OLD';"
else
  say "  ✅ 空壳项目 $OLD 已删"
fi

# ── 验证(不看「应该成功了」,看实际状态) ──
line; say "验证"; line
NB="$(q "select seat_id||'/'||role from project_role_bindings where project_id='$NEW'" | tr -d ' ')"
say "  $NEW 的绑定: ${NB:-无}"
[ "$NB" = "seat-ql-001/custom(worker)" ] || fail "绑定不对:'$NB' —— 合并没成功"
STILL="$(q "select count(*) from projects where id='$OLD'" | tr -d ' ')"
say "  $OLD 还在吗: $([ "$STILL" = "0" ] && echo '已删 ✅' || echo '仍在(上面有原因)')"
say ""
say "  现在去 UI 选 $NEW,应该能看到 seat-ql-001。"
say "  但注意:**那是条从没启动过的席位记录** —— 看得见不等于能干活。"
say "  要它真辅助你,还得在 QL 目录把 QL agent 拉起来(下一步,Aegis 会另说)。"
exit 0
