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
# 🔴 来源 provenance 每行都要有,而且**不能凭 SQL 猜**。这一版改对了原来写死的
#    'source+runtime':一次数据库读只知道账本这么断言,不知道它有没有到达操作者
#    的应用。没有证据就写 'source',并明说没有 runtime 证据。写死 runtime 等于
#    伪造一条没做过的测量 —— 删除守卫就是 source 真、runtime 假的那个例子。
#
# 🔴 隔离(2026-11-07 第二轮,Aegis 的 11-02 契约):
#    - **必须显式给项目**。不给、给空、给不存在的项目,都拒绝。没有默认项目 ——
#      一个默认项目会把「你忘了选」显示成「这个项目没有待办」。
#    - **操作者身份走已有的两条受信通道**(~/.seatloom/operator 文件、然后
#      SEATLOOM_OPERATOR_REF 环境变量),和 cmd_accept_delivery 同一条规则。
#      **没有 --actor 参数**:命令行上能传的东西就不是受信上下文。
#      两条通道都不说身份 → 拒绝,并点名那个文件和那个变量。绝不默认
#      human:supervisor —— 那正是契约点名的两种禁止行为之一。
#    - **一次 REPEATABLE READ 事务读完全部来源**,不是五个各自为政的快照。
#      五个查询之间项目可以变、行可以变,拼出来的一份清单没有一个是真的。
#    - **不静默截断**。超过展示上限就说清「显示 N 条,共 M 条」。
#    - **未知 kind 是待复核候选,不是自动的人工义务**。认不出的分类不能默认成
#      「欠操作者一件事」;契约原话:未知类别必须明确报,不能静默漏掉。
#
# 诚实的局限(契约自己写明、这里照抄):命令行上设的环境变量和「受信启动器设的」
#    无法区分。所以本版的文件通道是强的那一条,环境通道是弱的那一条,拒绝文案里
#    说清用的是哪一条。不声称解决共享 OS 凭据的隔离问题。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
DB="${SEATLOOM_PG_DB:-seatloom}"
PODMAN_BIN="${PODMAN_BIN:-podman}"
STALL_MINUTES="${WORK_LIST_STALL_MINUTES:-30}"

# 已知的派单 kind 词表。**只用于识别「认不出的分类」**,不用于决定谁欠什么 ——
# 一个 kind 不在表里,说明它是新类别或拼错了,那样它该被当成待复核,而不是被
# 猜成产品活或验收活。认不出还照样算成人工义务,就是契约禁止的那件事。
KNOWN_KINDS="implementation verification"
# 展示上限。超了不截断了事:说出显示几条、一共几条。
MAX_ROWS="${WORK_LIST_MAX_ROWS:-50}"

# 每行的字段,固定顺序,便于人和脚本都能读。
#   来源表 / id / 主题 / 归属 / 年龄 / 动作 / provenance / 是否未决
FMT='%-16s %-34s %-38s %-14s %-18s %-8s %-22s %-9s %s\n'

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

# ── 参数:只有一个,而且它是范围,不是权限 ────────────────────────────────
# --project 说的是「看哪个项目」,不是「以谁的身份看」。身份只能来自受信通道,
# 所以这里**没有** --actor:一个能在命令行上传的身份不是受信上下文。
PROJECT=""
usage(){
  say "用法:bash scripts/work-list.sh --project <项目 id>"
  say ""
  say "  --project <id>   必填。要看哪个项目的待办。不给、给空、或给不存在的"
  say "                   项目,本命令拒绝并退出非零 —— 没有默认项目。"
  say ""
  say "  操作者身份不通过参数传,从这两条已有通道读(与验收命令同一条规则):"
  say "    1. ~/.seatloom/operator 文件(强通道:操作者自己写进去的)"
  say "    2. SEATLOOM_OPERATOR_REF 环境变量(弱通道:命令行上设的和受信启动器"
  say "       设的无法区分,拒绝文案会说明用了哪一条)"
  say "  两条都不说身份 → 拒绝。绝不默认 human:supervisor。"
}
while [ $# -gt 0 ]; do
  case "$1" in
    --project)
      [ $# -ge 2 ] || { usage; fail "--project 后面要跟一个项目 id"; }
      PROJECT="$2"; shift 2;;
    --project=*) PROJECT="${1#--project=}"; shift;;
    -h|--help) usage; exit 0;;
    *) usage; fail "不认识的参数:$1";;
  esac
done

# 项目是范围,必须先定。空项目 id 不是「全部项目」,是「没选」—— 把它当成全部
# 会显示一堆别的项目的活,那正是契约里项目隔离要防的事。
[ -n "$PROJECT" ] || { usage; fail "必须显式给 --project <项目 id>。没有默认项目:一个默认项目会把「忘了选」显示成「这个项目没有待办」。"; }

# ── 操作者身份:两条已有通道,和验收命令同一条规则 ────────────────────────
# 读文件的实现与 crates/seatloom-core/src/config.rs 一致:一条非空行,# 注释和
# 空行忽略;两条以上非空行是拒绝,不是挑一条 —— 挑一条就是猜。
OPERATOR=""
OPERATOR_BASIS=""
OPERATOR_FILE="$HOME/.seatloom/operator"
if [ -n "${HOME:-}" ] && [ -f "$OPERATOR_FILE" ]; then
  FILE_LINES="$(grep -v '^[[:space:]]*$' "$OPERATOR_FILE" 2>/dev/null | grep -v '^[[:space:]]*#' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' || true)"
  N="$(printf '%s\n' "$FILE_LINES" | grep -c . || true)"
  if [ "$N" -gt 1 ]; then
    fail "操作者身份文件 $OPERATOR_FILE 有 $N 行非空内容,而一个身份是一个 token —— 你想说的是哪一个?$(printf '%s\n' "$FILE_LINES" | sed 's/^/    /' | tr '\n' ' ')"
  fi
  if [ "$N" = "1" ]; then
    OPERATOR="$FILE_LINES"
    OPERATOR_BASIS="file:$OPERATOR_FILE"
  fi
fi
# 文件没说,才看环境变量。顺序和 config.rs 一样:文件优先。
if [ -z "$OPERATOR" ] && [ -n "${SEATLOOM_OPERATOR_REF:-}" ]; then
  case "$(printf '%s' "$SEATLOOM_OPERATOR_REF" | tr -d '[:space:]')" in
    "") fail "SEATLOOM_OPERATOR_REF 设了但是空的 —— 一个空串不是身份。";;
    *)  OPERATOR="$(printf '%s' "$SEATLOOM_OPERATOR_REF" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
        OPERATOR_BASIS="env:SEATLOOM_OPERATOR_REF";;
  esac
fi
# 两条通道都不说。拒绝,并告诉他要写哪个文件 —— 一个不能执行的清单不如没有。
[ -n "$OPERATOR" ] || fail "操作者身份没有配置:~/.seatloom/operator 不存在(或没有非空行),SEATLOOM_OPERATOR_REF 也未设。\n  把身份写进 $OPERATOR_FILE(一行,例如 human:supervisor),或设 SEATLOOM_OPERATOR_REF。\n  本命令不默认任何身份 —— 猜一个操作者会把别人的活显示成你的。"

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
# 现在每一次查询都送进**同一个只读事务会话**(见上面 SQLSCRIPT 的构造),
# 于是五次读取落在同一个快照上。失败时 Q_FAILED 带上 psql 自己的话,由
# require() 在主 shell 里 fail() —— 调用形状不是命令替换,所以 exit 1 真的结束。
q(){
  local sql label
  sql="$1"
  label="${2:-}"
  Q_OUT=""
  Q_FAILED=""
  # 用 \echo 把段标签打出来,好在一次输出里把各段切开。
  printf '\\echo ====%s====\n%s\n' "$label" "$sql" >> "$SQLSCRIPT"
}

# 把累积的查询一次性交给 psql,在同一个 REPEATABLE READ 只读事务里跑完。
# ON_ERROR_STOP 保证任何一条失败整批停,退出码非零 —— 不存在「三条成功一条失败」
# 还照样输出的部分成功。
run_snapshot(){
  printf 'COMMIT;\n' >> "$SQLSCRIPT"
  if ! Q_OUT="$("$PODMAN_BIN" exec -i -e PGPASSWORD=seatloom "$CONTAINER" \
        psql -X -U seatloom -d "$DB" -tA -f - 2>&1 < "$SQLSCRIPT")"; then
    Q_FAILED="$Q_OUT"
    return 1
  fi
  return 0
}

# 从一次快照输出里取一段。段标签由 q() 写进去,不是猜的。
# 为什么不用 sed 的区间:最后一段后面没有下一个标记,sed 的 /end/ 永远等不到,
# 于是最后一段恒定取空(实测踩到:verif 段取出空,看起来像「0 张 verification 单」)。
# awk 的写法在遇到下一个标记时收段,输出结束也收段。
section(){ printf '%s\n' "$Q_OUT" | awk -v tag="====$1====" '$0==tag{f=1;next} f && /^====/{f=0} f && $0!="COMMIT" && $0!="BEGIN"'; }

# 读一张表。失败时 fail() 在主 shell 里执行,exit 1 真的结束脚本。
require(){
  local table sql label
  table="$1"; sql="$2"; label="${3:-$1}"
  if ! q "$sql" "$label"; then
    fail "读不到 $table —— 这不是「没有待办」,是「不知道有没有待办」。psql 说:${Q_FAILED:-?}"
  fi
}

# 读一张表。失败时 fail() 在主 shell 里执行,exit 1 真的结束脚本。

# 年龄:用数据库自己的时钟算,不拿库里的时间戳跟宿主机比( podman 虚拟机的时钟
# 会在宿主机休眠后停摆,那样算出来的「挂了 10 小时」是假的)。
age_of(){ printf "round(extract(epoch from (now() - %s))/60)" "$1"; }

line
say "待办清单 — 操作者当前要动的每一件"
say "项目:${PROJECT}   操作者:${OPERATOR}(${OPERATOR_BASIS})   停滞界线:${STALL_MINUTES} 分钟"
line

ROWS=0
# 这些变量在快照执行后才被赋值,但 set -u 会在赋值前就检查它们。先声明为空。
PROJ_NAME=""; OD=""; AC=""; DS=""; UNC=""; UNC_TOTAL=""; UNC_KINDS=""; VN=""

# ── 一次 REPEATABLE READ 事务,不是五个各自为政的快照 ────────────────────
# 原来的写法是五次独立的 podman exec,每次一条查询。五个查询之间项目可以变、
# 行可以被别的进程改,拼出来的一份清单**没有一个时刻是真的**。契约 §3.1 的原话:
# 禁止先数后查两个实时结果。这里的做法是开一个只读事务,把所有查询送进同一个
# 会话,事务结束前它们看到的是同一个快照。
#
# 做法:把所有 SQL 写进一个临时文件,一次性交给 psql,输出按段落分隔。psql 的
# 退出码仍然代表整批的成败,所以 q() 的失败检测继续有效。
SQLSCRIPT="$(mktemp)"
trap 'rm -f "$SQLSCRIPT"' EXIT
{
  printf '\\set ON_ERROR_STOP on\n'
  printf 'BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;\n'
} > "$SQLSCRIPT"

# ── 查询排队(不执行) ──────────────────────────────────────────────────────
# 🔴 排队和执行必须分开。第一版把执行放在第一个查询之后立刻做,结果只有项目存在性
#    那一段是真的,其余四段在主 shell 里取到空串 —— 一份「0 obligations」的报告,
#    而库里明明有活(实测:od-1、disp-ok 都在,报告说没有)。
#    形状:先把七条查询都 require 进同一个事务,再一次 run_snapshot,然后才渲染。
require projects "select name from projects where id='$PROJECT';"
# 项目隔离 + 操作者隔离:**decider 必须精确等于受信操作者**,不是「以 human: 开头」,
# 也不是「谁建的」。契约 TODO-04:只看 human 前缀或让客户端指定 actor,都必须失败。
require operator_decisions "select id||'|'||coalesce(project_id,'')||'|'||coalesce(target,'')||'|'||coalesce(consequence,'')||'|'||$(age_of created_at) from operator_decisions where status='pending' and project_id='$PROJECT' and decider='$OPERATOR' order by created_at;"
require acceptances "select a.id||'|'||coalesce(d.target_seat_id,'')||'|'||coalesce(d.title,'')||'|'||$(age_of a.created_at) from acceptances a left join dispatches d on d.id=a.dispatch_id where a.status='pending' and a.project_id='$PROJECT' order by a.created_at;"
require dispatches "select id||'|'||coalesce(target_seat_id,'')||'|'||coalesce(title,'')||'|'||$(age_of issued_at) from dispatches where status='ready' and project_id='$PROJECT' and issued_at < now() - interval '${STALL_MINUTES} minutes' order by issued_at;" stalled
# 🔴 kind='verification' 仍然要排除 —— 第一轮判据 6 靠的就是这条。第二版为了给
#    认不出的 kind 让路,把 `<> 'verification'` 整个删了,结果 verification 单
#    回到列表里(实测:disp-verif 出现在「未验收」段,判据 6 红)。
#    正确的形状是三档,不是两档:
#      verification        → 排除,并说出为什么(它本身就是一次验收)
#      已知产品 kind        → 要求验收它
#      认不出               → 待复核候选(请人看),仍然可见
require dispatches "select id||'|'||coalesce(target_seat_id,'')||'|'||coalesce(title,'')||'|'||coalesce(payload->>'kind','')||'|'||$(age_of completed_at)
  from dispatches
  where status='done'
    and project_id='$PROJECT'
    and completed_at is not null
    and coalesce(payload->>'kind','') <> 'verification'
    and not exists (select 1 from acceptances a where a.dispatch_id = dispatches.id)
  order by completed_at desc;" done
require dispatches "select count(*) from dispatches where status='done' and project_id='$PROJECT' and completed_at is not null and coalesce(payload->>'kind','') <> 'verification' and not exists (select 1 from acceptances a where a.dispatch_id = dispatches.id);" done_total
require dispatches "select coalesce(payload->>'kind','') || '|' || count(*) from dispatches where status='done' and project_id='$PROJECT' and completed_at is not null and coalesce(payload->>'kind','') <> 'verification' and not exists (select 1 from acceptances a where a.dispatch_id = dispatches.id) group by coalesce(payload->>'kind','') order by 1;" done_kinds
require dispatches "select count(*) from dispatches
  where status='done' and project_id='$PROJECT' and completed_at is not null
    and coalesce(payload->>'kind','') = 'verification'
    and not exists (select 1 from acceptances a where a.dispatch_id = dispatches.id);" verif

# ── 一次性执行,然后取段 ──────────────────────────────────────────────────
if ! run_snapshot; then
  fail "快照读取失败 —— 这不是「没有待办」,是「不知道有没有待办」。psql 说:${Q_FAILED:-?}"
fi
PROJ_NAME="$(section projects)"
[ -n "$PROJ_NAME" ] || fail "项目 '$PROJECT' 不存在(ProjectNotFound)。不是「这个项目没有待办」—— 是没有这个项目。"
OD="$(section operator_decisions)"
AC="$(section acceptances)"
DS="$(section stalled)"
UNC="$(section done)"
UNC_TOTAL="$(section done_total)"
UNC_KINDS="$(section done_kinds)"
VN="$(section verif)"

# ── 渲染 ────────────────────────────────────────────────────────────────
# ── 1. operator_decisions:他该裁的 ───────────────────────────────────────
say ""
say "▸ 一 — operator_decisions(该你裁的)"
if [ -z "$OD" ]; then
  say "  (没有待你裁的决定)"
else
  printf "$FMT" "来源表" "id" "主题" "kind" "归属" "年龄" "动作" "provenance" "未决"
  printf "$FMT" "----------------" "----------------------------------" "------------------------------------" "--------------" "------------------" "--------" "----------------------" "---------" "----"
  while IFS='|' read -r id proj target conseq age; do
    [ -n "$id" ] || continue
    printf "$FMT" "operator_decisions" "$id" "裁:${target:-?} → ${conseq:-?}" "decision" "$proj" "$age" "裁决它" "source" "是"
    ROWS=$((ROWS+1))
  done <<< "$OD"
fi

# ── 2. acceptances:他该点接受/打回的 ─────────────────────────────────────
say ""
say "▸ 二 — acceptances(该你点接受或打回的)"
if [ -z "$AC" ]; then
  say "  (没有等你的卡)"
else
  printf "$FMT" "来源表" "id" "主题" "kind" "归属" "年龄" "动作" "provenance" "未决"
  printf "$FMT" "----------------" "----------------------------------" "------------------------------------" "--------------" "------------------" "--------" "----------------------" "---------" "----"
  while IFS='|' read -r id seat title age; do
    [ -n "$id" ] || continue
    printf "$FMT" "acceptances" "$id" "${title:-?}" "acceptance" "$seat" "$age" "点接受或打回" "source" "是"
    ROWS=$((ROWS+1))
  done <<< "$AC"
fi

# ── 3. dispatches:超时的 ready 单(系统问题,不是他的活)──────────────────────
say ""
say "▸ 三 — dispatches(躺了 ${STALL_MINUTES} 分钟没人领 —— 系统问题,不是你的活)"
# 只纳入停滞的。一张 ready 单是席位的活,永远不渲染成操作者欠的东西。
if [ -z "$DS" ]; then
  say "  (没有停滞的单)"
else
  printf "$FMT" "来源表" "id" "主题" "kind" "归属" "年龄" "动作" "provenance" "未决"
  printf "$FMT" "----------------" "----------------------------------" "------------------------------------" "--------------" "------------------" "--------" "----------------------" "---------" "----"
  while IFS='|' read -r id seat title age; do
    [ -n "$id" ] || continue
    printf "$FMT" "dispatches" "$id" "${title:-?}" "ready>${STALL_MINUTES}m" "$seat" "$age" "无需你动(席位该领)" "source" "是"
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
# 这一段不再按「非 verification 就是人工义务」分类。它分三档:
#   kind 在已知词表里          → 未验收的产品活,动作「要求验收它」
#   kind 认不出                → **待复核候选**,动作「认不出分类,请人看」——
#                                不是自动的人工义务。契约:未知类别必须明确报,
#                                不能静默漏掉,也不能猜成某一类。
#   kind=verification          → 排除,并说出为什么(它本身就是一次验收)
# 全量计数单独查,于是「显示 N 条 / 共 M 条」是真的,不是拿展示长度当全量。
say "  (已排除 ${VN:-?} 张 done 的 verification 单 —— kind 值是 'verification',它们本身就是验收,再要求被验收是循环)"
# 认不出的 kind 单独数出来说,不静默。契约:未知类别必须明确报。
UNKNOWN_KINDS=""
while IFS='|' read -r k n; do
  [ -n "$k" ] || continue
  case " $KNOWN_KINDS " in
    *" $k "*) ;;
    *) UNKNOWN_KINDS="${UNKNOWN_KINDS}${k}(${n}) ";;
  esac
done <<< "$UNC_KINDS"
if [ -n "$UNKNOWN_KINDS" ]; then
  say "  ⚠️ 认不出的 kind:$UNKNOWN_KINDS —— 不在已知词表里,当作**待复核候选**列出,不自动算成欠你的一件事。"
fi

if [ -z "$UNC" ]; then
  say "  (除此之外,没有完成却从没被验收的单)"
else
  printf "$FMT" "来源表" "id" "主题" "kind" "归属" "年龄" "动作" "provenance" "未决"
  printf "$FMT" "----------------" "----------------------------------" "------------------------------------" "--------------" "------------------" "--------" "----------------------" "---------" "----"
  SHOWN=0
  TRUNCATED=0
  while IFS='|' read -r id seat title kind age; do
    [ -n "$id" ] || continue
    if [ "$SHOWN" -ge "$MAX_ROWS" ]; then
      # 不静默截断。说出显示几条、一共几条 —— 拿展示长度当全量是契约明令禁止的。
      TRUNCATED=$(( TRUNCATED + 1 ))
      continue
    fi
    case " $KNOWN_KINDS " in
      *" $kind "*)
        ACTION="要求验收它";;
      *)
        ACTION="认不出分类,请人看";;
    esac
    # provenance 只写 source。账本这么断言,不等于它已经到达操作者的应用 ——
    # 那需要另一条证据,而这个命令没有做那条测量。写死 source+runtime 是伪造。
    printf "$FMT" "dispatches" "$id" "${title:-?}" "${kind:-?}" "$seat" "$age" "$ACTION" "source" "是"
    ROWS=$((ROWS+1)); SHOWN=$((SHOWN+1))
  done <<< "$UNC"
  if [ "$TRUNCATED" -gt 0 ]; then
    say ""
    say "  ⚠️ 这一段共 ${UNC_TOTAL:-?} 条,上面只显示 ${SHOWN} 条(上限 ${MAX_ROWS})。"
    say "     还有 ${TRUNCATED} 条没显示 —— 这不是「没有了」。用 WORK_LIST_MAX_ROWS 提高上限。"
  fi
fi

line
if [ "$ROWS" = "0" ]; then
  say "结果:0 obligations"
  say "三张来源都读过了:operator_decisions / acceptances / dispatches(同一个只读快照)"
  exit 0
fi
say "结果:$ROWS obligations"
exit 0
