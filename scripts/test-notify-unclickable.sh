#!/usr/bin/env bash
# test-notify-unclickable.sh — 证明 notify-operator.sh 第 5 类(有卡点不动)两个方向都真的会发生。
#
# 为什么需要这个文件:Lyra 派活时(disp-ZZEb6fpwXPR0zqMQjiHwV)把第三条要求说成
# 「最容易被跳过的一条」—— 这一条不能在没事的时候弹。她说要用变异证明,
# 和我们对 Nimbus 的守卫用的是同一套纪律。
#
# 一个永远不响的检查和永远响的检查一样糟:前者让张先生以为通道是活的,
# 后者教他忽略整条通道。
#
# 用法:bash scripts/test-notify-unclickable.sh   退出码 0 = 两个方向都验过了
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUBJECT="$REPO/scripts/notify-operator.sh"
WORK="$(mktemp -d /tmp/notify-test.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

fails=0
ok(){ printf '  ✓ %s\n' "$1"; }
bad(){ printf '  ✗ %s\n' "$1"; fails=$((fails+1)); }

cp "$SUBJECT" "$WORK/subject.sh"
chmod +x "$WORK/subject.sh"

# 把真的检查换成一个可控的桩,这样我们能在「点得动」和「点不动」两种状态下各跑一次,
# 而不必真的去制造一次点不动(那要动 track 或 Mira 的分支)。
stub() { # $1 = 退出码, $2 = 输出
  local rc="$1" out="$2"
  python3 - "$WORK/subject.sh" "$rc" "$out" <<'PY'
import sys
p, rc, out = sys.argv[1], sys.argv[2], sys.argv[3]
s = open(p).read()
anchor = 'click_out="$(bash "$REPO/scripts/check-cards-clickable.sh" 2>&1)"\nclick_rc=$?'
assert anchor in s, "找不到被测的调用点 —— 测试的锚失效了,该更新测试而不是跳过"
s = s.replace(anchor, 'click_out=%r\nclick_rc=%s' % (out, rc))
open(p, 'w').write(s)
PY
}

# 每条用例都清掉去重记录,否则第二次跑会被当成「弹过了」而跳过 ——
# 那会让一个真的会弹的用例看起来是安静的,也就是假阴性。
clear_seen(){ rm -f "$HOME/.seatloom-notify-seen" 2>/dev/null; }

echo "── 方向一:卡点得动(退出码 0),这一类必须完全不发声 ──"
cp "$SUBJECT" "$WORK/subject.sh"; clear_seen; stub 0 ""
out="$(bash "$WORK/subject.sh" --push --force 2>&1)"
if printf '%s' "$out" | grep -q '有验收卡现在点不动'; then
  bad "卡点得动时它弹了 —— 这正是 Lyra 要求防的那个方向(会教他忽略整条通道)"
else
  ok "卡点得动时它闭嘴"
fi

echo "── 方向二:卡真的点不动(退出码 1),这一类必须弹,并说清怎么解锁 ──"
cp "$SUBJECT" "$WORK/subject.sh"; clear_seen
stub 1 "FAIL
packet/mira/knowledge-panel|acc-disp-Ra5YbN4t78bzdbYhYkQiV|behind 1"
out="$(bash "$WORK/subject.sh" --push --force 2>&1)"
if printf '%s' "$out" | grep -q '有验收卡现在点不动'; then
  ok "点不动时它弹了"
else
  bad "点不动时它没弹 —— 这一条等于不存在"
fi
# pop() 只把标题打进输出,正文(那条命令)不进 —— 所以「给了命令」这件事要读源码,
# 不能读输出。读输出会得到一个假阴性:通知真的带着命令,只是没被打印出来。
if grep -q 'git merge track/infra-foundation' "$SUBJECT"; then
  ok "文案里带着那条能解锁的命令"
else
  bad "弹了但没说怎么解开 —— 一个不能据以行动的通知不是通知"
fi

echo "── 方向三:查不了(退出码 2),必须闭嘴并明说自己没查,不能假装干净 ──"
cp "$SUBJECT" "$WORK/subject.sh"; clear_seen; stub 2 ""
out="$(bash "$WORK/subject.sh" --push --force 2>&1)"
if printf '%s' "$out" | grep -q '有验收卡现在点不动'; then
  bad "查不了时它也弹了 —— 那是在报告一个它并不知道的状态"
else
  ok "查不了时它没弹"
fi
if printf '%s' "$out" | grep -q '没查卡的点得动性'; then
  ok "并且明说了自己没查"
else
  bad "没查却一声不响 —— 静默吞错"
fi

echo "── 变异:把「点得动就闭嘴」的守卫废掉,方向一必须转红 ──"
cp "$SUBJECT" "$WORK/subject.sh"; clear_seen; stub 0 ""
python3 - "$WORK/subject.sh" <<'PY'
import sys
p = sys.argv[1]
s = open(p).read()
old = 'if [ "$click_rc" = "0" ]; then\n  :  # 点得动,这一类不发声 —— 这是设计,不是遗漏\nelif [ "$click_rc" = "1" ]; then'
new = 'if [ "$click_rc" = "77" ]; then\n  :  # MUTATED: guard never matches\nelif [ "$click_rc" != "99" ]; then'
assert old in s, "变异锚失效 —— 被测代码变了,该更新变异而不是跳过"
open(p, 'w').write(s.replace(old, new))
PY
out="$(bash "$WORK/subject.sh" --push --force 2>&1)"
if printf '%s' "$out" | grep -q '有验收卡现在点不动'; then
  ok "变异后被抓住了:守卫是真的在承重,不是装饰"
else
  bad "变异后仍然安静 —— 说明方向一的守卫根本不起作用,这条检查是死的"
fi

echo ""
if [ "$fails" = "0" ]; then
  echo "✅ 三个方向加一个变异都验过了。"
  exit 0
fi
echo "❌ $fails 项没过。"
exit 1
