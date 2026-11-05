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
# 一个不会失败的检查比没有检查更糟。每一行都必须能失败,最后一条是变异。

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO/scripts/work-list.sh"
[ -f "$SCRIPT" ] || { echo "❌ 找不到 $SCRIPT"; exit 1; }

SCRATCH="seatloom_wl_test"
CONTAINER="seatloom-postgres"
FAIL=0
pass(){ echo "  ✓ $1"; }
die(){ echo "  ✗ $1"; FAIL=1; }

psq(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d "$SCRATCH" -tAc "$1" 2>&1; }
psq_ok(){ podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -X -U seatloom -d "$SCRATCH" -q -c "$1" >/dev/null 2>&1; }

cleanup(){
  podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d postgres -q \
    -c "DROP DATABASE IF EXISTS $SCRATCH" >/dev/null 2>&1
}
trap cleanup EXIT

# ── 造 scratch 库 ────────────────────────────────────────────────────────
echo "── 准备 scratch 库(绝不碰 live)──"
cleanup
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
psq_ok "insert into projects (id,name,created_at,worker_budget_tokens,supervisor_budget_tokens) values ('wl', 'wl', now(), 1024, 1024)"

wl(){ SEATLOOM_PG_DB="$SCRATCH" bash "$SCRIPT" 2>&1; }

# 派单的 target_seat_id 有外键,先造一行 seat(形状按 001 的列,不发明)。
psq_ok "insert into seats (id, name, capability_tags, status, created_at) values ('seat-test-001','test','{}','active', now())"

echo ""
echo "── 判据 8:三个来源全空 → 退出 0,打印 0 obligations,并说明三张都读了 ──"
OUT="$(wl)"; RC=$?
[ "$RC" = "0" ] && pass "全空时退出 0" || die "全空时退出 $RC"
printf '%s\n' "$OUT" | grep -q "0 obligations" && pass "打印 0 obligations" || die "没打印 0 obligations"
for t in operator_decisions acceptances dispatches; do
  printf '%s\n' "$OUT" | grep -q "$t" && pass "说明了读过 $t" || die "没说读过 $t"
done

echo ""
echo "── 判据 1:三张表都有行 → 退出 0,每张来源都有行 ──"
psq_ok "insert into operator_decisions (id, project_id, target, choices, decider, consequence, status, created_by) values ('od-1','wl','要不要装','[{\"a\":1}]','human:supervisor','装错会覆盖','pending','seat:seat-aegis-001')"
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority) values ('disp-ok','seat-test-001','seat-test-001','wl','一件已完成的产品活','{\"kind\":\"implementation\"}','done',5)"
psq_ok "update dispatches set completed_at = now() where id='disp-ok'"
psq_ok "insert into acceptances (id, dispatch_id, project_id, branch, base, how_to_verify, flux_verdict_ref, status) values ('acc-1','disp-ok','wl','packet/test/x','track/infra-foundation','看看','ev-x','pending')"
OUT="$(wl)"; RC=$?
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
OUT="$(wl)"; RC=$?
[ "$RC" != "0" ] && pass "来源失败时退出非零($RC)" || die "来源失败却退出 0"
printf '%s\n' "$OUT" | grep -q "dispatches" && pass "点名了 dispatches" || die "没点名是哪张表"
printf '%s\n' "$OUT" | grep -qi "0 obligations" && die "来源失败却打印 0 obligations" || pass "没有伪装成 0 obligations"
psq_ok "alter table dispatches_hidden rename to dispatches"

echo ""
echo "── 判据 3 / 4:ready 单按 30 分钟分界 ──"
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, issued_at) values ('disp-young','seat-test-001','seat-test-001','wl','刚发的 ready 单','{}','ready',5, now())"
psq_ok "insert into dispatches (id, target_seat_id, issued_by, project_id, title, payload, status, priority, issued_at) values ('disp-stale','seat-test-001','seat-test-001','wl','躺了很久的 ready 单','{}','ready',5, now() - interval '90 minutes')"
OUT="$(wl)"; RC=$?
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
OUT="$(wl)"; RC=$?
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
OUT="$(wl)"; RC=$?
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
OUT="$(wl)"; RC=$?
[ "$RC" = "0" ] && pass "退出 0" || die "退出 $RC"
printf '%s\n' "$OUT" | grep -q "acc-2" && pass "新卡在" || die "新卡不在"
# disp-nocard 现在有卡了,所以它不该再出现在「未验收」那一段里
SEC4="$(printf '%s\n' "$OUT" | sed -n '/四 —/,$p')"
printf '%s\n' "$SEC4" | grep -q "disp-nocard" && die "有卡了还挂在未验收段" || pass "有卡之后从「未验收」段消失"

echo ""
echo "── provenance 每一行都有 ──"
OUT="$(wl)"
N_SRC="$(printf '%s\n' "$OUT" | grep -c 'source+runtime' || true)"
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
OUTM="$(wl)"; RCM=$?
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
