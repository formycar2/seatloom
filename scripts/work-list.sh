#!/usr/bin/env bash
# work-list.sh — 一件活在一处:操作者当前要动的每一件,连同它从哪来。
#
# 为什么必须有(2026-11-07 Lyra 三优先级研究里的第二项):
#   操作者的可行动义务散在**三张已经存在的表**里,而现有检查各看一张、各说各话:
#     - operator_decisions(pending)  他该裁的
#     - acceptances(pending)          他该点接受/打回的
#     - dispatches(ready 且超 30 分钟) 该被推动而没被推动的
#   于是「我现在欠什么」这个问题没有单一答案,而每一张单都自称自己那张最重要。
#
# 边界(派单定死的,这个脚本不越界):
#   - 不是 read model:不建新表、不做快照服务、不分页(那是下一片)
#   - 不是 UI:CLI 可以是诊断片,不是最终界面的替代品
#   - 不是重新分类器:分类只读值,不猜形状
#
# 🔴 纳入规则用的是**停滞规则**,不是 ready 规则。一张 ready 单是席位的活。
#    notify-operator.sh:166-168 早就弄对了这件事:只有超过 30 分钟才响,并且明说
#    「这是系统问题,不是你的事」。本脚本照抄那条界线 —— 一张 ready 单永远不会
#    被渲染成操作者欠的东西。
#
# 🔴 数据源读失败时,必须非零退出并点名是哪一张表,**绝不能不打印**。
#    一个空列表不是健康的证据。派单的原话:「source failure must be uncheckable
#    or non-zero, NEVER empty」。三张表现在都有行(实测 operator_decisions 0 /
#    acceptances 0 / dispatches 1),所以拿空数据源测不出任何东西 —— 正因为如此,
#    读失败必须响。
#
# 🔴 来源 provenance 每行都要有:'source' = 账本这么断言;
#    'source+runtime' = 它已经到达操作者的应用。这两者实测会分叉 ——
#    删除守卫就是 source 真、runtime 假。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
DB="${SEATLOOM_PG_DB:-seatloom}"
PODMAN_BIN="${PODMAN_BIN:-podman}"
STALL_MINUTES="${WORK_LIST_STALL_MINUTES:-30}"

# 每行的字段,固定顺序,便于人和脚本都能读。
#   来源表 / id / 主题 / 归属 / 年龄 / 动作 / provenance / 是否未决
FMT='%-18s %-38s %-46s %-22s %-9s %-30s %-14s %s\n'

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

# ── 严格的查询 helper ────────────────────────────────────────────────────
# 和 notify-operator.sh 的 q() 的区别,也就是这个脚本存在的理由之一:
# 那个 helper 把 stderr 丢掉、失败时返回空。于是「表读不到」和「表里没有行」
# 长得一模一样。这里失败时返回空字符串**并且设置 QUERY_FAILED**,调用方必须检查。
# 读失败必须和「没有行」可区分。notify-operator.sh 的 q() 把 stderr 丢掉、
# 失败时返回空,于是这两种情况长得一模一样 —— 那正是这个脚本要避免的。
#
# 三个实测踩到的坑,每一个都让读失败伪装成空列表:
#   坑一:`local sql out rc` 之后 `rc=$?` 拿到的是 `local` 自己的退出状态。
#   坑二:`out="$(q "$sql")"` 也是命令替换,`$?` 拿到的是**替换**的状态。
#   坑三(最隐蔽):`X="$(require …)"` 里 fail() 的 exit 1 只退出那个子 shell,
#        主脚本照旧往下走,还把失败那一行当成数据打印出来。实测输出里出现过
#        「❌ 读不到 dispatches」紧接着「结果:9 obligations」并退出 0 ——
#        一个会说谎的报告。
#
# 修法:不用命令替换。q() 直接把结果写进一个**全局**变量,失败时也写进全局变量,
# 于是失败状态活在主 shell 里,检查它的人也在主 shell 里。
Q_OUT=""
Q_FAILED=""
q(){
  local sql
  sql="$1"
  Q_OUT=""
  Q_FAILED=""
  if ! Q_OUT="$("$PODMAN_BIN" exec -e PGPASSWORD=seatloom "$CONTAINER" \
        psql -X -U seatloom -d "$DB" -tAc "$sql" 2>&1)"; then
    Q_FAILED="$Q_OUT"        # psql 自己的话,操作者才知道为什么读不到
    return 1
  fi
  return 0
}

# 读一张表。失败时 fail() 在主 shell 里执行,exit 1 真的结束脚本。
require(){
  local table="$1" sql="$2"
  if ! q "$sql"; then
    fail "读不到 $table —— 这不是「没有待办」,是「不知道有没有待办」。psql 说:${Q_FAILED:-?}"
  fi
}

# 年龄:用数据库自己的时钟算,不拿库里的时间戳跟宿主机比( podman 虚拟机的时钟
# 会在宿主机休眠后停摆,那样算出来的「挂了 10 小时」是假的)。
age_of(){ printf "round(extract(epoch from (now() - %s))/60)" "$1"; }

line
say "待办清单 — 操作者当前要动的每一件"
say "生成于:$(date '+%Y-%m-%d %H:%M:%S')  停滞界线:${STALL_MINUTES} 分钟"
line

ROWS=0

# ── 1. operator_decisions:他该裁的 ───────────────────────────────────────
say ""
say "▸ 一 — operator_decisions(该你裁的)"
require operator_decisions "select id||'|'||coalesce(project_id,'')||'|'||coalesce(target,'')||'|'||coalesce(consequence,'')||'|'||$(age_of created_at) from operator_decisions where status='pending' order by created_at;"
OD="$Q_OUT"
if [ -z "$OD" ]; then
  say "  (没有待你裁的决定)"
else
  printf "$FMT" "来源表" "id" "主题" "归属" "年龄(分)" "动作" "provenance" "未决"
  printf "$FMT" "------------------" "------------------------------------" "--------------------------------------------" "--------------------" "---------" "----------------------------" "-------------" "----"
  while IFS='|' read -r id proj target conseq age; do
    [ -n "$id" ] || continue
    printf "$FMT" "operator_decisions" "$id" "裁:${target:-?} → ${conseq:-?}" "$proj" "$age" "裁决它" "source+runtime" "是"
    ROWS=$((ROWS+1))
  done <<< "$OD"
fi

# ── 2. acceptances:他该点接受/打回的 ─────────────────────────────────────
say ""
say "▸ 二 — acceptances(该你点接受或打回的)"
require acceptances "select a.id||'|'||coalesce(d.target_seat_id,'')||'|'||coalesce(d.title,'')||'|'||$(age_of a.created_at) from acceptances a left join dispatches d on d.id=a.dispatch_id where a.status='pending' order by a.created_at;"
AC="$Q_OUT"
if [ -z "$AC" ]; then
  say "  (没有等你的卡)"
else
  printf "$FMT" "来源表" "id" "主题" "归属" "年龄(分)" "动作" "provenance" "未决"
  printf "$FMT" "------------------" "------------------------------------" "--------------------------------------------" "--------------------" "---------" "----------------------------" "-------------" "----"
  while IFS='|' read -r id seat title age; do
    [ -n "$id" ] || continue
    printf "$FMT" "acceptances" "$id" "${title:-?}" "$seat" "$age" "点接受或打回" "source+runtime" "是"
    ROWS=$((ROWS+1))
  done <<< "$AC"
fi

# ── 3. dispatches:超时的 ready 单(系统问题,不是他的活)──────────────────────
say ""
say "▸ 三 — dispatches(躺了 ${STALL_MINUTES} 分钟没人领 —— 系统问题,不是你的活)"
# 只纳入停滞的。一张 ready 单是席位的活,永远不渲染成操作者欠的东西。
require dispatches "select id||'|'||coalesce(target_seat_id,'')||'|'||coalesce(title,'')||'|'||$(age_of issued_at) from dispatches where status='ready' and issued_at < now() - interval '${STALL_MINUTES} minutes' order by issued_at;"
DS="$Q_OUT"
if [ -z "$DS" ]; then
  say "  (没有停滞的单)"
else
  printf "$FMT" "来源表" "id" "主题" "归属" "年龄(分)" "动作" "provenance" "未决"
  printf "$FMT" "------------------" "------------------------------------" "--------------------------------------------" "--------------------" "---------" "----------------------------" "-------------" "----"
  while IFS='|' read -r id seat title age; do
    [ -n "$id" ] || continue
    printf "$FMT" "dispatches" "$id" "${title:-?}" "$seat" "$age" "无需你动(席位该领)" "source+runtime" "是"
    ROWS=$((ROWS+1))
  done <<< "$DS"
fi

# ── 4. 已完成的活,从没被要求验收 ────────────────────────────────────────
# 这一行是现有检查会漏掉的那一行:一张 done 的单,没有 pending 卡,却带着一个
# acceptance 键(说明曾经有过卡)。那意味着它上线了而操作者从没被要求看过它。
#
# 🔴 判据读**值**,不读形状。payload 的形状和键的存在都不是权威的任务分类。
#    所以 (a) 不因为「有 acceptance 键」就排除它 —— 那正是会漏掉它的写法;
#    (b) kind 是 payload 里的一个**值**,读它是允许的,而且必须读:一张
#    kind='verification' 的单本身就是一次验收,再要求它被验收是循环。
#    这一条读的是值(payload->>'kind'),不是形状(键存在与否)。
say ""
say "▸ 四 — 完成但从未验收的活(现有检查漏掉的一行)"
require dispatches "select id||'|'||coalesce(target_seat_id,'')||'|'||coalesce(title,'')||'|'||$(age_of completed_at)
  from dispatches
  where status='done'
    and completed_at is not null
    and coalesce(payload->>'kind','') <> 'verification'
    and not exists (select 1 from acceptances a where a.dispatch_id = dispatches.id)
  order by completed_at desc
  limit 50;"
UNC="$Q_OUT"
# 说出 verification 单为什么不在 —— **无论这一段是不是空**。只在这一段空时说,
# 等于「有未验收产品活的时候就不解释排除规则」,而那时最需要知道规则是什么。
require dispatches "select count(*) from dispatches
  where status='done' and completed_at is not null
    and coalesce(payload->>'kind','') = 'verification'
    and not exists (select 1 from acceptances a where a.dispatch_id = dispatches.id);"
VN="$Q_OUT"
say "  (已排除 ${VN:-?} 张 done 的 verification 单 —— kind 值是 'verification',它们本身就是验收,再要求被验收是循环)"
if [ -z "$UNC" ]; then
  say "  (除此之外,没有完成却从没被验收的单)"
else
  printf "$FMT" "来源表" "id" "主题" "归属" "年龄(分)" "动作" "provenance" "未决"
  printf "$FMT" "------------------" "------------------------------------" "--------------------------------------------" "--------------------" "---------" "----------------------------" "-------------" "----"
  while IFS='|' read -r id seat title age; do
    [ -n "$id" ] || continue
    printf "$FMT" "dispatches" "$id" "${title:-?}" "$seat" "$age" "要求验收它" "source+runtime" "是"
    ROWS=$((ROWS+1))
  done <<< "$UNC"
fi

line
if [ "$ROWS" = "0" ]; then
  say "结果:0 obligations"
  say "三张来源都读过了:operator_decisions / acceptances / dispatches"
  exit 0
fi
say "结果:$ROWS obligations"
exit 0
