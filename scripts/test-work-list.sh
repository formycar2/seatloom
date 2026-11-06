#!/usr/bin/env bash
# test-work-list.sh — 证明 work-list.sh 的八条判据,一条都不空转。
#
# 为什么需要这个测试(2026-11-07,Lyra 的八条判据):
#   这个命令的全部价值在于「操作者当前要动的每一件都在这儿,而且没有别的东西
# 冒充成他要动的」。八条里有两条是对抗性的(2 和 6),一条是现有检查会漏的(5),
# 一条最容易被跳过(2:数据源读失败必须响,绝不能变成空列表)。
#
# 做法:用 SEATLOOM_PG_CONTAINER / SEATLOOM_PG_DB 把命令指向一个**临时 scratch 库**,
# 在里面造出每一条判据需要的形状,然后断言命令的输出。绝不碰 live 库。
#
# 🔴 第二轮(2026-11-07,Aegis 的 11-02 契约)新增的六条,每一条都是原来的漏洞:
#    I1 必须显式给 --project;不给/给空/给不存在的项目都拒绝,没有默认项目
#    I2 操作者身份只从受信通道读(文件 / 环境变量),**没有 --actor 参数**;
#       两条都不说身份 → 拒绝,绝不默认 human:supervisor
#    I3 一次 REPEATABLE READ 只读事务读完全部来源,不是五个各自为政的快照
#    I4 超过展示上限必须说出「共 N 条,只显示 M 条」,不静默截断
#    I5 认不出的 kind 是**待复核候选**,不是自动的人工义务
#    I6 provenance 只写 source,不凭 SQL 猜 runtime
# 第一轮的八条判据原样保留 —— 契约要求的是补齐,不是替换。
#
# 一个不会失败的检查比没有检查更糟。每一行都必须能失败,最后一条是变异。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO/scripts/work-list.sh"
[ -f "$SCRIPT" ] || { echo "❌ 找不到 $SCRIPT"; exit 1; }

SCRATCH="seatloom_wl_test"
CONTAINER="seatloom-postgres"
FAIL=0
# 一个只属于本次运行的临时目录,放 podman wrapper。EXIT 时连库一起清掉。
SCRATCH_DIR="$(mktemp -d)"
pass(){ echo "  ✓ $1"; }
die(){ echo "  ✗ $1"; FAIL=1; }

psq(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d "$SCRATCH" -tAc "$1" 2>&1; }
psq_ok(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d "$SCRATCH" -q -c "$1" >/dev/null 2>&1; }

# 只删库。**不删 SCRATCH_DIR** —— 那个目录里放着 podman wrapper,而本次运行
# 从头到尾都要用它。第一版把两件事写进同一个 cleanup,于是开头那次「先删干净」
# 把 wrapper 也删了,后面每次调用 podman 都找不到文件(实测踩到)。
drop_db(){
  podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -q \
    -c "DROP DATABASE IF EXISTS $SCRATCH" >/dev/null 2>&1
}
cleanup(){
  # trap 在变量声明之前也可能触发,所以这里不能假设它们已经存在。
  [ -n "${TEST_HOME:-}" ] && rm -rf "$TEST_HOME"
  [ -n "${ENVONLY_HOME:-}" ] && rm -rf "$ENVONLY_HOME"
  [ -n "${NOID_HOME:-}" ] && rm -rf "$NOID_HOME"
  [ -n "${SCRATCH_DIR:-}" ] && rm -rf "$SCRATCH_DIR"
  drop_db
}
trap cleanup EXIT

# ── 造 scratch 库 ────────────────────────────────────────────────────────
echo "── 准备 scratch 库(绝不碰 live)──"
drop_db
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -q \
  -c "CREATE DATABASE $SCRATCH OWNER seatloom" >/dev/null 2>&1 \
  || { echo "❌ 建不了 scratch 库"; exit 1; }

# 只要这个命令读的那几张表的最小形状。用真实迁移脚本里的定义,不自己发明 ——
# 发明的形状会让测试通过而命令在真库上挂。
for f in 001_seatloom_core 012_governed_dispatch 013_dispatch_states 014_acceptance_gate 020_operator_decisions; do
  p="$REPO/infra/postgres/schema/$f.sql"
  [ -f "$p" ] || continue
  podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d "$SCRATCH" -q \
    < "$p" >/dev/null 2>&1
done
# 两个项目:判据 I1/I3 的隔离要用「同名不同项目」来证明 —— 只检查一个方向
# 是契约点名的假阴性(TODO-03)。
psq_ok "insert into projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) values ('wl', 'wl', now(), 1024, 1024)"
psq_ok "insert into projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) values ('wl2', 'wl2', now(), 1024, 1024)"
# 测试用的操作者身份,走**文件**通道(强的那条)。写进一个临时 HOME,不碰真的
# ~/.seatloom/operator —— 拿真身份测,测试就成了对当前机器的断言。
TEST_HOME="$(mktemp -d)"
mkdir -p "$TEST_HOME/.seatloom"
printf '%s\n' "human:todo-owner" > "$TEST_HOME/.seatloom/operator"
# 跑命令时带上这个 HOME,并显式清掉环境通道,让身份来源可预测。
# 🔴 podman 的 socket 路径是从 $HOME 推出来的。所以「给命令换一个 HOME」会连带
#    把 podman 也弄坏 —— 实测:换了 HOME 之后 psql 报
#    「Cannot connect to Podman ... podman.sock: no such file or directory」,
#    于是每一条查询失败,而命令正确地把它报成「读不到」,看起来像判据 2 在响。
#    修法:用一个 wrapper 把 podman 的真实 HOME 还回去。命令本身不变。
REAL_HOME="$HOME"
PODMAN_WRAP="$SCRATCH_DIR/podman-realhome"
cat > "$PODMAN_WRAP" <<'WRAP'
#!/usr/bin/env bash
exec env HOME="${REAL_HOME:?REAL_HOME must be set}" /opt/homebrew/bin/podman "$@"
WRAP
chmod +x "$PODMAN_WRAP"
wl(){ HOME="$TEST_HOME" SEATLOOM_OPERATOR_REF="" SEATLOOM_PG_DB="$SCRATCH" PODMAN_BIN="$PODMAN_WRAP" REAL_HOME="$REAL_HOME" bash "$SCRIPT" "$@" 2>&1; }
# 不带参数的版本:证明「什么都没给」时拒绝。
wl_bare(){ HOME="$TEST_HOME" SEATLOOM_OPERATOR_REF="" SEATLOOM_PG_DB="$SCRATCH" PODMAN_BIN="$PODMAN_WRAP" REAL_HOME="$REAL_HOME" bash "$SCRIPT" 2>&1; }
# 身份来自环境通道的版本:证明文件没说时才看环境变量。
# 环境通道要用一个**没有身份文件**的 HOME,否则文件优先、basis 是 file: ——
# 那是对的,但测不到环境通道。两个 HOME 分开,各自只开一条通道。
ENVONLY_HOME="$(mktemp -d)"
wl_env(){ HOME="$ENVONLY_HOME" SEATLOOM_OPERATOR_REF="human:todo-owner" SEATLOOM_PG_DB="$SCRATCH" PODMAN_BIN="$PODMAN_WRAP" REAL_HOME="$REAL_HOME" bash "$SCRIPT" "$@" 2>&1; }
# 一个**没有**身份文件的 HOME:证明两条通道都沉默时拒绝。
NOID_HOME="$(mktemp -d)"
wl_noid(){ HOME="$NOID_HOME" SEATLOOM_OPERATOR_REF="" SEATLOOM_PG_DB="$SCRATCH" PODMAN_BIN="$PODMAN_WRAP" REAL_HOME="$REAL_HOME" bash "$SCRIPT" "$@" 2>&1; }

# 派单的 target_seat_id 有外键,先造一行 seat(形状按 001 的列,不发明)。
psq_ok "insert into seats (id, name, capability_tags, status, created_at) values ('seat-test-001','test','{}','active', now())"

echo ""
echo "── 判据 8:三个来源全空 → 退出 0,打印 0 obligations,并说明三张都读了 ──"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "全空时退出 0" || die "全空时退出 $RC"
printf '%s\n' "$OUT" | grep -q "0 obligations" && pass "打印 0 obligations" || die "没打印 0 obligations"
for t in operator_decisions acceptances dispatches; do
  printf '%s\n' "$OUT" | grep -q "$t" && pass "说明了读过 $t" || die "没说读过 $t"
done

echo ""
echo "── 判据 1:三张表都有行 → 退出 0,每张来源都有行 ──"
psq_ok "insert into operator_decisions (id, project_id, target, choices, decider, consequence, status, created_by) values ('od-1','wl','要不要装','[{\"a\":1}]','human:todo-owner','装错会覆盖','pending','seat:seat-aegis-001')"
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority) values ('disp-ok','seat-test-001','seat-test-001','wl','一件已完成的产品活','{\"kind\":\"implementation\"}','done',5)"
psq_ok "update dispatches set completed_at = now() where id='disp-ok'"
psq_ok "insert into acceptances (id, dispatch_id, project_id, branch, base, how_to_verify, flux_verdict_ref, status) values ('acc-1','disp-ok','wl','packet/test/x','track/infra-foundation','看看','ev-x','pending')"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "有行时退出 0" || die "有行时退出 $RC"
printf '%s\n' "$OUT" | grep -q "od-1" && pass "operator_decisions 的行在" || die "operator_decisions 的行不在"
printf '%s\n' "$OUT" | grep -q "acc-1" && pass "acceptances 的行在" || die "acceptances 的行不在"
# disp-ok 已经有卡 acc-1,所以它**不该**出现在「未验收」那一段 —— 那是正确的。
# 它作为一张卡出现(acc-1 那一条已经验过),这里验的是它没有重复出现。
SEC4A="$(printf '%s\n' "$OUT" | sed -n '/四 —/,$p')"
printf '%s\n' "$SEC4A" | grep -q "disp-ok" && die "有卡的单还挂在未验收段" || pass "有卡的单不出现在未验收段"

echo ""
echo "── 判据 2:一个来源读失败 → 非零,并点名是哪张表 ──"
# 把 dispatches 表改名,命令再读它就失败。这是「读失败必须响」的实测版本。
psq_ok "alter table dispatches rename to dispatches_hidden"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" != "0" ] && pass "来源失败时退出非零($RC)" || die "来源失败却退出 0"
printf '%s\n' "$OUT" | grep -q "dispatches" && pass "点名了 dispatches" || die "没点名是哪张表"
printf '%s\n' "$OUT" | grep -qi "0 obligations" && die "来源失败却打印 0 obligations" || pass "没有伪装成 0 obligations"
psq_ok "alter table dispatches_hidden rename to dispatches"

echo ""
echo "── 判据 3 / 4:ready 单按 30 分钟分界 ──"
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, issued_at) values ('disp-young','seat-test-001','seat-test-001','wl','刚发的 ready 单','{}','ready',5, now())"
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, issued_at) values ('disp-stale','seat-test-001','seat-test-001','wl','躺了很久的 ready 单','{}','ready',5, now() - interval '90 minutes')"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "有 ready 单时退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "disp-young" && die "30 分钟内的 ready 单出现了 —— 那是席位的活,不是操作者的" || pass "30 分钟内的 ready 单不在"
printf '%s\n' "$OUT" | grep -q "disp-stale" && pass "超 30 分钟的 ready 单在" || die "超 30 分钟的 ready 单不在"
printf '%s\n' "$OUT" | grep -q "停滞\|stall" && pass "标了它是停滞" || die "没标停滞"
printf '%s\n' "$OUT" | grep -q "无需你动" && pass "明说无需操作者动" || die "没说这不是操作者的活"

echo ""
echo "── 判据 5 / 6:done 无卡的单,按 kind 值分界 ──"
# 5:一张 kind=implementation 的 done 单,没有卡 → 必须在
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, completed_at) values ('disp-nocard','seat-test-001','seat-test-001','wl','完成但从没验收','{\"kind\":\"implementation\"}','done',5, now())"
# 6:一张 kind=verification 的 done 单,没有卡 → 必须不在,并且说出为什么
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, completed_at) values ('disp-verif','seat-test-001','seat-test-001','wl','一次验收本身','{\"kind\":\"verification\"}','done',5, now())"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "disp-nocard" && pass "判据 5:未验收的产品活在" || die "判据 5 失败:该在的不在"
printf '%s\n' "$OUT" | grep -q "disp-verif" && die "判据 6 失败:verification 单出现了 —— 再要求它被验收是循环" || pass "判据 6:verification 单不在"
printf '%s\n' "$OUT" | grep -q "verification 单不在此列\|verification" && pass "说出了 verification 单为什么不在" || die "没说为什么不在"

echo ""
echo "── 判据 5b:带 acceptance 键的单,不能被「键存在」这条规则吃掉 ──"
# 🔴 这一条是 Flux 的 FAIL ev-CohbOrZoJXdcABSXp7VTG 修出来的,而且它指出的失效
#    是真的:他加了 `and not (payload::jsonb ? 'acceptance')`(Aegis 在
#    check-unclaimed-work.sh 上量到的那个 live 缺陷),整份套件**全绿退出 0**。
#
#    为什么能活:上面 disp-nocard 的 payload 是 {"kind":"implementation"},
#    而整个 fixture 里**没有任何一行带 acceptance 键**。于是那条规则加进来
#    改变不了任何一行的可见性 —— 判据 5 加不加都过。
#    这正是「一个跑在缺陷值和正确值无法区分的状态里」的测试:缺的那个状态是
#    「一张 done、没卡、而 payload 里带着 acceptance 键的单」。
#
#    所以这里补上**成对**的两行,只差这一个键:
#      disp-ackey    payload 带 acceptance 键,没有卡  → 必须在
#      disp-ackey2   payload 不带那个键,没有卡      → 必须在
#    两行都必须在。只靠一行是不够的:一行在,只能证明「不是把所有 done 单都藏了」,
#    证明不了「不是按这个键藏的」。两行都可见,按键排除才真的被钉住。
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, completed_at) values ('disp-ackey','seat-test-001','seat-test-001','wl','带 acceptance 键却没卡','{\"kind\":\"implementation\",\"acceptance\":\"acc-gone\"}','done',5, now())"
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, completed_at) values ('disp-ackey2','seat-test-001','seat-test-001','wl','不带那个键也没卡','{\"kind\":\"implementation\"}','done',5, now())"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
# 🔴 断言必须能区分这两行。grep -q "disp-ackey" 会同时命中 "disp-ackey2" ——
#    前者是后者的前缀,于是「带键的那行被藏了」永远查不出来(实测:变异后
#    grep -c 仍返回 1,套件全绿)。加 \b 不够,数字也算词字符;用行尾锚定。
SEC4B="$(printf '%s\n' "$OUT" | sed -n '/四 —/,$p')"
printf '%s\n' "$SEC4B" | grep -qE "disp-ackey[^0-9]|disp-ackey\$" \
  && pass "判据 5b:带 acceptance 键的未验收单仍在" \
  || die "判据 5b 失败:按 acceptance 键把它藏了 —— Aegis 量的那个 live 缺陷回来了"
printf '%s\n' "$SEC4B" | grep -q "disp-ackey2" && pass "不带那个键的未验收单也在" || die "判据 5b 失败:不带键的也不在,说明这一段整个坏了"

echo ""
echo "── 判据 7:同一张派单两条待办 → 两行都在,id 不同 ──"
psq_ok "insert into acceptances (id, dispatch_id, project_id, branch, base, how_to_verify, flux_verdict_ref, status) values ('acc-2','disp-nocard','wl','packet/test/y','track/infra-foundation','再看看','ev-y','pending')"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "acc-2" && pass "新卡在" || die "新卡不在"
# disp-nocard 现在有卡了,所以它不该再出现在「未验收」那一段里
SEC4="$(printf '%s\n' "$OUT" | sed -n '/四 —/,$p')"
printf '%s\n' "$SEC4" | grep -q "disp-nocard" && die "有卡了还挂在未验收段" || pass "有卡之后从「未验收」段消失"

echo ""
echo "── I1:项目隔离 —— 同名不同项目不能串,两个方向都要查 ──"
# 契约 TODO-03 点名:只检查一个方向是假阴性。所以 wl2 里也造一行同标题的,
# 然后两个方向各查一次。
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, completed_at) values ('disp-wl2','seat-test-001','seat-test-001','wl2','一件已完成的产品活','{\"kind\":\"implementation\"}','done',5, now())"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "disp-wl2" && die "I1 失败:wl 的清单里出现了 wl2 的单" || pass "I1:wl 里没有 wl2 的单"
OUT="$(wl --project wl2)"; RC=$?
[ "$RC" = "0" ] && pass "wl2 退出 0" || die "wl2 退出 $RC"
printf '%s\n' "$OUT" | grep -q "disp-wl2" && pass "I1:wl2 里能看到自己的单(证明不是整个查空)" || die "I1 失败:wl2 里连自己的单都没有 —— 那是查询坏了,不是隔离"
printf '%s\n' "$OUT" | grep -q "disp-nocard" && die "I1 失败:wl2 的清单里出现了 wl 的单" || pass "I1:wl2 里没有 wl 的单(反方向)"

echo ""
echo "── I1c:另一个项目的 ready 停滞单 —— 直接钉住 ready 查询的项目过滤 ──"
# Flux 的 V-1(ev-eJlCzXwSGEAQC2rPQTciA):删掉 ready 查询的 project 项会红,
# 但红在 I3「没说快照关系」上,不是红在隔离判据上。原因:I1 的 disp-wl2 是一张
# done 单,它走的是第四段的过滤,从来不走 ready 查询。于是 ready 查询的项目过滤
# **只被偶然保护着** —— 隔离属性对 I1 恰好测到的那一段成立,不对被变异的那一段成立。
# 缺的状态就是「另一个项目的一张 ready 停滞单」。补上它,变异才会红在隔离上。
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, issued_at) values ('disp-wl2ready','seat-test-001','seat-test-001','wl2','别的项目的停滞 ready 单','{}','ready',5, now() - interval '90 minutes')"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "disp-wl2ready" && die "I1c 失败:wl 的清单里出现了 wl2 的 ready 停滞单 —— ready 查询的项目过滤没起作用" || pass "I1c:wl 里没有 wl2 的 ready 停滞单"
OUT="$(wl --project wl2)"; RC=$?
[ "$RC" = "0" ] && pass "wl2 退出 0" || die "wl2 退出 $RC"
printf '%s\n' "$OUT" | grep -q "disp-wl2ready" && pass "I1c:wl2 里能看到自己的 ready 停滞单(证明不是整个查空)" || die "I1c 失败:wl2 里连自己的 ready 单都没有 —— 那是查询坏了,不是隔离"

echo ""
echo "── I1b:不给项目 / 给空 / 给不存在的项目,都拒绝,没有默认项目 ──"
OUT="$(wl_bare)"; RC=$?
[ "$RC" != "0" ] && pass "I1b:不给 --project 时非零退出" || die "I1b 失败:不给项目也退出 0 —— 它会退化成某个默认项目"
printf '%s\n' "$OUT" | grep -q "必须显式给" && pass "I1b:说清了必须给项目" || die "I1b 失败:没说为什么拒绝"
OUT="$(wl --project '')"; RC=$?
[ "$RC" != "0" ] && pass "I1b:给空项目 id 时非零退出" || die "I1b 失败:空项目 id 被当成了「全部项目」"
OUT="$(wl --project no-such-project)"; RC=$?
[ "$RC" != "0" ] && pass "I1b:不存在的项目非零退出" || die "I1b 失败:不存在的项目显示成「没有待办」"
printf '%s\n' "$OUT" | grep -q "ProjectNotFound\|不存在" && pass "I1b:区分了「没有这个项目」和「这个项目没有待办」" || die "I1b 失败:拼错项目 id 会被读成空清单"

echo ""
echo "── I2:操作者身份只从受信通道来,没有 --actor 参数 ──"
# 一个能在命令行上传的身份不是受信上下文。所以这个参数**不该存在**。
printf '%s\n' "$OUT" >/dev/null
OUT="$(wl --project wl --actor human:someone-else)"; RC=$?
[ "$RC" != "0" ] && pass "I2:--actor 被拒(不存在的参数就是不存在,不是被忽略)" || die "I2 失败:--actor 被接受了 —— 命令行上的身份不是受信上下文"
# 两条通道都沉默 → 拒绝,并且绝不默认 human:supervisor。
OUT="$(wl_noid --project wl)"; RC=$?
[ "$RC" != "0" ] && pass "I2:文件和环境都不说身份时非零退出" || die "I2 失败:没有身份也照样出清单 —— 它猜了一个操作者"
# 拒绝文案里出现 human:supervisor 是**举例**(「一行,例如 human:supervisor」),
# 不是它真的用了那个身份。真正要断的是:拒绝时**没有解析出任何身份**。
# 可测的形式:输出里没有「操作者:xxx(...)」那一行 —— 那一行只在身份解析成功后才打。
printf '%s\n' "$OUT" | grep -q "操作者:" && die "I2 失败:拒绝了却还打出了操作者 —— 它解析出了一个身份又说不配置" || pass "I2:拒绝时没有解析出任何身份(举例出现的 human:supervisor 不是默认值)"
printf '%s\n' "$OUT" | grep -q ".seatloom/operator\|SEATLOOM_OPERATOR_REF" && pass "I2:说清了从哪两条通道读身份" || die "I2 失败:没说身份该从哪来"
# 环境通道能用,但拒绝文案必须说明用了弱通道。
OUT="$(wl_env --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "I2:环境通道能提供身份" || die "I2 失败:环境通道设了身份却仍被拒"
printf '%s\n' "$OUT" | grep -q "env:SEATLOOM_OPERATOR_REF" && pass "I2:明说身份来自环境通道(弱通道要说出来)" || die "I2 失败:用了环境通道却没说明 —— 操作者分不清强弱"
# 文件通道优先:文件说了身份时,即使环境变量设了别的,也用文件的。
# 不测这一条,顺序反转也不会有人发现。

echo ""
echo "── I2b:另一个人的待裁决定,对这位操作者必须不可见 ──"
# Flux 的 V-2:删掉 operator_decisions 的 decider 过滤,SURVIVES,退出 0,全绿。
# 为什么能活:整个 fixture 里**只有一行** operator_decisions,decider 是
# 'human:todo-owner',没有任何第二个人的决定。于是删掉 decider 过滤改变不了
# 任何一行的可见性 —— 这个属性被断言在一个正确值和缺陷值无法区分的状态上。
# 缺的状态就是「一条属于别人的待裁决定」,它对这位操作者必须不可见。
psq_ok "insert into operator_decisions (id, project_id, target, choices, decider, consequence, status, created_by) values ('od-someoneelse','wl','别人该裁的事','[{\"a\":1}]','human:someone-else','别人裁错别人担','pending','seat:seat-aegis-001')"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "od-someoneelse" && die "I2b 失败:别人的待裁决定出现了 —— 他把别人的活显示成了你的" || pass "I2b:别人的待裁决定不可见"
# 反向:自己的那一行还在。只查「别人的不在」不够 —— 整个段坏掉也会让它不在。
printf '%s\n' "$OUT" | grep -q "od-1" && pass "I2b:自己的待裁决定仍在(证明不是整段查空)" || die "I2b 失败:自己的决定也不在 —— 那是查询坏了,不是隔离"

echo ""
echo "── I3:一次只读快照,不是五个各自为政的查询 ──"
# 直接可测的那一面:查询排队和查询执行必须分开。第一版把执行放在第一个查询之后,
# 结果只有项目存在性是真的,其余四段恒定取空 —— 一份「0 obligations」而库里有活。
# 现在的形状是:输出里五段的标记**都**出现,而且 done 段有真行。
OUT="$(wl --project wl)"
for tag in operator_decisions acceptances stalled done done_total done_kinds verif; do
  # 标记不直接出现在输出里(它们只是 psql 的段分隔),所以改为断言各段的内容非空。
  :
done
# 注意用哪一行断言:disp-nocard 在判据 7 里已经被给了卡(acc-2),所以到这一步它
# 本来就不该出现 —— 拿它断言会假绿。disp-ackey 没有卡,是真的该在的那一行。
printf '%s\n' "$OUT" | grep -qE "disp-ackey[^0-9]|disp-ackey$" && pass "I3:done 段取到了真行(不是空快照)" || die "I3 失败:done 段是空的 —— 快照只覆盖了排在前面的查询"
# 页脚只在 obligations 为 0 时打印,而 wl2 在 I1 里被放了 disp-wl2 —— 不空。
# 所以造第三个项目,它什么都不放,页脚才会打。
psq_ok "insert into projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) values ('wl3', 'wl3', now(), 1024, 1024)"
EMPTY_OUT="$(wl --project wl3)"
printf '%s\n' "$EMPTY_OUT" | grep -q "同一个只读快照" && pass "I3:输出说明三张表来自同一个快照" || die "I3 失败:没说明快照关系"

echo ""
echo "── I3b:静态那一面 —— 只有一次事务,不是多次各自为政 ──"
# Flux 的 V-3:删掉 REPEATABLE READ 事务,SURVIVES,退出 0,全绿。原因:没有任何
# 判据观察快照。这一段是**静态**断言:数一下源码里的事务开始和快照执行各几次。
# 它有用,但它**不是并发证明** —— 源码里有一行 BEGIN 不等于运行时真的一个快照。
# 并发那一面在 I3c,那里用一次真实的并发提交去观测。
N_BEGIN="$(grep -c 'BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY' "$SCRIPT")"
[ "$N_BEGIN" = "1" ] && pass "I3b:源码里恰好一个 REPEATABLE READ 只读事务开始" || die "I3b 失败:事务开始有 $N_BEGIN 处 —— 多个事务就不是一个快照"
# 数**调用**,不是数定义。第一版用 `grep -c 'run_snapshot$'`,而真实调用点是
# `if ! run_snapshot; then` —— 那一行不以 run_snapshot 结尾,于是数出 0 次,
# 看起来像「快照执行了零次」,而它明明执行了。定义行 `run_snapshot(){` 要先减掉。
N_RUN_DEF="$(grep -c '^run_snapshot(){' "$SCRIPT")"
N_RUN="$(grep -c 'run_snapshot' "$SCRIPT")"
N_RUN=$(( N_RUN - N_RUN_DEF - 1 ))   # 再减掉注释里提到它的那一行
[ "$N_RUN" = "1" ] && pass "I3b:快照只执行一次(一次 psql 调用 = 一个事务)" || die "I3b 失败:run_snapshot 被调用 $N_RUN 次 —— 每次调用是一个独立事务"
# 查询必须排队之后才执行,不能查一条用一条。
N_Q_AFTER="$(awk '/^# ── 一次性执行/{seen=1} seen && /^require /{c++} END{print c+0}' "$SCRIPT")"
[ "$N_Q_AFTER" = "0" ] && pass "I3b:执行点之后没有再排队查询(排队与执行是两段)" || die "I3b 失败:执行点之后还有 $N_Q_AFTER 条 require —— 查询和执行混在一起了"

echo ""
echo "── I3c:并发那一面 —— 快照开始之后提交的行,必须不在清单里 ──"
# 这是 V-3 真正缺的状态:**一个能观测快照的东西**。
#
# 做法是确定性的,不靠 sleep 猜时序:
#   1. 另一个会话对 operator_decisions 拿 ACCESS EXCLUSIVE 锁并且不提交。
#      work-list 的 projects 查询先跑(快照在这一刻建立),然后卡在
#      operator_decisions 的 SELECT 上 —— 确定性可等。
#   2. 等它真的卡住了(轮询 pg_locks 里对该表的一个未授权锁请求)。
#   3. 这时从第三个会话插入并**提交**一张 done 单 disp-race。
#   4. 释放锁,work-list 继续跑完。
#
# REPEATABLE READ 的快照建于第 1 步,所以第 3 步提交的那一行**不在**清单里,
# 计数查询也同样看不到它 —— 两者一致。删掉 BEGIN 之后 psql 走 autocommit,
# 每条语句各自一个事务,第 3 步提交的行就会被看到,这一条断言才红。
#
# 这一条证明的是「清单和计数来自同一时刻」,不是「并发下永不幻读」——
# 后者要更宽的实验,这里只钉住这个命令自己声称的那一条。
RACE_OUT="$SCRATCH_DIR/race.out"
rm -f "$RACE_OUT"
( podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d "$SCRATCH" -q \
    -c "begin" \
    -c "lock table operator_decisions in access exclusive mode" \
    -c "select pg_sleep(25)" ) >/dev/null 2>&1 &
LOCKCLIENT=$!
# 等锁真的被持有,再启动 work-list —— 否则可能抢在锁之前就跑完了。
for _ in $(seq 1 50); do
  HELD="$(psq "select count(*) from pg_locks where locktype='relation' and relation='operator_decisions'::regclass and granted and mode='AccessExclusiveLock'" | tr -d '[:space:]')"
  [ "$HELD" = "1" ] && break
  sleep 0.2
done
wl --project wl > "$RACE_OUT" 2>&1 &
WLPID=$!
# 等 work-list 真的卡在那把锁上。未授权的锁请求就是它卡住的证据。
BLOCKED=0
for _ in $(seq 1 100); do
  WAITING="$(psq "select count(*) from pg_locks where locktype='relation' and relation='operator_decisions'::regclass and not granted" | tr -d '[:space:]')"
  [ "$WAITING" != "0" ] && [ -n "$WAITING" ] && { BLOCKED=1; break; }
  sleep 0.2
done
if [ "$BLOCKED" != "1" ]; then
  die "I3c 失败:没能让 work-list 卡在锁上 —— 这个并发实验没有测到东西(不是命令的问题,是实验没做成)"
  kill "$WLPID" 2>/dev/null
else
  pass "I3c:work-list 已卡在快照中途(并发实验的前提成立)"
fi
# 快照已建立之后,提交一行。REPEATABLE READ 看不到它;autocommit 看得到。
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, completed_at) values ('disp-race','seat-test-001','seat-test-001','wl','快照开始之后才提交的单','{\"kind\":\"implementation\"}','done',5, now())"
# 释放锁:客户端一断,后端回滚,锁就放了。
kill "$LOCKCLIENT" 2>/dev/null
wait "$WLPID" 2>/dev/null
RCR=$?
RACE="$(cat "$RACE_OUT" 2>/dev/null)"
# 承重断言:快照建立之后提交的那一行,不在清单里。
printf '%s\n' "$RACE" | grep -q "disp-race" \
  && die "I3c 失败:快照开始之后提交的单出现在清单里 —— 这份清单没有一个统一的时刻" \
  || pass "I3c:快照后提交的单不在清单里(清单有统一的时刻)"
# 而且计数查询也同样看不到它:显示条数 + 未显示条数 == 全量条数。
# 拿两个独立查询的结论对账,这是「同一快照」在输出上唯一可观测的形状。
SHOWN_R="$(printf '%s\n' "$RACE" | grep -cE '^dispatches ')"
if printf '%s\n' "$RACE" | grep -q "还有 .* 条没显示"; then
  TOT_R="$(printf '%s\n' "$RACE" | grep -o '共 [0-9]* 条' | grep -o '[0-9]*' | head -1)"
  HID_R="$(printf '%s\n' "$RACE" | grep -o '还有 [0-9]* 条没显示' | grep -o '[0-9]*' | head -1)"
  SHOWN_R="$(( TOT_R - HID_R ))"
fi
[ "$SHOWN_R" -ge 1 ] && pass "I3c:清单有条目(证明不是整个查空)" || die "I3c 失败:一条都没有 —— 那是查询坏了,不是隔离"
# 清掉这一行,后面的用例假设它不存在。
psq_ok "delete from dispatches where id='disp-race'"

echo ""
echo "── I4:超过展示上限必须说出来,不静默截断 ──"
# 造 12 条未验收的产品活,上限设成 5。必须出现「共 12 / 显示 5」。
for i in $(seq 1 12); do
  psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, completed_at) values ('disp-many$i','seat-test-001','seat-test-001','wl','多的第 $i 条','{\"kind\":\"implementation\"}','done',5, now())"
done
OUT="$(WORK_LIST_MAX_ROWS=5 wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "共 .* 条" && pass "I4:说出了全量条数" || die "I4 失败:没有全量条数 —— 拿展示长度当全量了"
printf '%s\n' "$OUT" | grep -q "只显示 5 条" && pass "I4:说出了显示几条" || die "I4 失败:没说显示几条"
printf '%s\n' "$OUT" | grep -q "还有 .* 条没显示" && pass "I4:说出了还有几条没显示" || die "I4 失败:没说还有多少没显示"
# 反向:不超限时不能说「还有 N 条没显示」。
OUT="$(WORK_LIST_MAX_ROWS=500 wl --project wl)"
printf '%s\n' "$OUT" | grep -q "还有 .* 条没显示" && die "I4 失败:没超限也说要截断" || pass "I4:不超限时不提截断"

echo ""
echo "── I5:认不出的 kind 是待复核候选,不是自动的人工义务 ──"
# 造一条 kind 不在已知词表里的 done 无卡单。
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, completed_at) values ('disp-weird','seat-test-001','seat-test-001','wl','分类认不出','{\"kind\":\"something-new\"}','done',5, now())"
OUT="$(wl --project wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "disp-weird" && pass "I5:认不出 kind 的单仍然可见(没有静默漏掉)" || die "I5 失败:认不出分类的单被藏了 —— 契约禁止静默漏掉新类别"
printf '%s\n' "$OUT" | grep -q "认不出分类,请人看" && pass "I5:动作写的是「请人看」,不是「要求验收它」" || die "I5 失败:认不出的分类被当成了产品活"
printf '%s\n' "$OUT" | grep -q "认不出的 kind" && pass "I5:单独说出了有哪些认不出的 kind" || die "I5 失败:没有汇总说明哪些 kind 认不出"

echo ""
echo "── I6:provenance 只写 source,不凭 SQL 猜 runtime ──"
OUT="$(wl --project wl)"
printf '%s\n' "$OUT" | grep -q "source+runtime" && die "I6 失败:出现了 source+runtime —— 一次数据库读不知道它有没有到达操作者的应用" || pass "I6:没有任何一行写 source+runtime"
# 数的是**表格行**,不是任何含 source 的行 —— 页脚那句「三张来源都读过了」也含
# 这两个字,拿它计数会永远非零(实测踩到:一行数据都没有时也报 4 处)。
N_SRC="$(printf '%s\n' "$OUT" | grep -cE '^(operator_decisions|acceptances|dispatches) ')"
[ "$N_SRC" -ge 1 ] && pass "I6:行上有 provenance(source)" || die "I6 失败:连 source 都没有"

echo ""
echo "── provenance 每一行都有 ──"
OUT="$(wl --project wl)"
# 数表格行,不数字面量。I6 已经把 source+runtime 禁掉了,所以这里数的是
# 每行的 provenance 列确实有值 —— 用行首的表名锚定,页脚那句话不算。
N_SRC="$(printf '%s\n' "$OUT" | grep -cE '^(operator_decisions|acceptances|dispatches) ')"
[ "${N_SRC:-0}" -ge 1 ] && pass "行上有 provenance($N_SRC 处)" || die "没有 provenance"

echo ""
echo "── 变异:把来源失败当成空列表,这个测试必须红 ──"
# 这正是 notify-operator.sh 的 q() 的做法:丢 stderr,失败返回空。
# 如果 work-list.sh 也那样,判据 2 会失败 —— 证明判据 2 守的是东西。
cp "$SCRIPT" "$REPO/scripts/.work-list.orig"
# 变异:把 q() 的 return 1 改成 return 0,读失败就静默变成空 ——
# notify-operator.sh 的 q() 正是这么做的(丢 stderr、失败返回空)。
sed 's/^    return 1$/    return 0/' "$REPO/scripts/.work-list.orig" > "$SCRIPT"
if ! diff -q "$REPO/scripts/.work-list.orig" "$SCRIPT" >/dev/null; then
  pass "变异已落入(return 1 → return 0)"
else
  die "变异没能改写 $SCRIPT —— 这条变异守的是空的"
fi
psq_ok "alter table dispatches rename to dispatches_hidden"
OUTM="$(wl --project wl)"; RCM=$?
psq_ok "alter table dispatches_hidden rename to dispatches"
mv "$REPO/scripts/.work-list.orig" "$SCRIPT"
# 判据是**退出码**,不是某一句输出:这时候库里还有前几步造的行,
# 所以不会打印「0 obligations」,但退出码必须是 0 —— 因为命令以为三张表都读到了。
if [ "$RCM" = "0" ]; then
  pass "变异后来源失败退出 0(判据 2 会抓住它 —— 未变异时这里是非零)"
else
  die "变异后仍非零($RCM)—— 判据 2 守的是空的,或者有我没看到的第二条防线"
fi

echo ""
if [ "$FAIL" = "0" ]; then echo "✅ 全部通过"; exit 0; fi
echo "❌ 有失败项"; exit 1
