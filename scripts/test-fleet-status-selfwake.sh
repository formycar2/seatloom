#!/usr/bin/env bash
# fleet-status.sh 的自醒检查,四个方向都验。2026-11-07 22:15 建。
#
# 为什么必须有必败方向:上一版把「没有日志」直接打成 ❌ 自醒死了。
# 实测(2026-11-07 22:11)自醒日志和 seatloom-waker 的日志**都**从 /tmp 消失,
# 而同一批里的 .txt 和子目录都在 —— 指向外部清理,不是自醒死了。
# 按那个判据走,我会把一个健康的系统「修」坏,或者为了变绿去重装 cron,
# 把原本正常的配置覆盖掉。
#
# 「文件不在」只说明信使不在了,不说明信使要说的事是假的。
#
# 跑法:bash scripts/test-fleet-status-selfwake.sh
set -u

REPO="/Users/jyxc-dz-0100609/Documents/GitHub/seatloom"
FS="$REPO/scripts/fleet-status.sh"
WLOG="/tmp/seatloom_aegis_waker.log"

pass(){ echo "  ok $1"; }
die(){ echo "  FAIL $1" >&2; exit 1; }

# 用替身日志路径跑,不动真的那份。WLOG 在 fleet-status.sh 里是写死的,
# 所以改一份临时副本来测 —— 否则测试会把真日志挪来挪去,
# 而真日志本身正是被测对象。
TMP=$(mktemp -d /tmp/fs-test.XXXXXX)
trap 'rm -rf "$TMP"' EXIT

mkfs(){ sed "s#^WLOG=.*#WLOG=\"$1\"#" "$FS" > "$TMP/fs.sh"; chmod +x "$TMP/fs.sh"; }

extract(){ bash "$TMP/fs.sh" 2>/dev/null | sed -n '/Aegis 自醒/,+4p'; }

echo "-- 0. preconditions: the two facts direction 3 depends on are real now --"
[ -s "$WLOG" ] && echo "   (real log present; the test uses a stand-in path and will not touch it)" \
                || echo "   (real log absent - exactly the case being fixed)"
crontab -l 2>/dev/null | grep -q 'aegis-self-wake.sh' \
  || die "cron entry absent, so direction 3 has no premise"
[ -x "$HOME/.local/bin/aegis-self-wake.sh" ] \
  || die "the script cron runs is absent, so direction 3 has no premise"
pass "cron entry and the installed script are both present"

echo ""
echo "-- 1. fresh log must report running --"
mkfs "$TMP/waker.log"
printf '%s aegis-self-wake: POKED Aegis-Supervisor (pane=node)\n' "$(date '+%Y-%m-%d %H:%M:%S')" > "$TMP/waker.log"
out=$(extract); printf '%s\n' "$out" | sed 's/^/   /'
printf '%s\n' "$out" | grep -q '在跑' && pass "fresh log -> running" || die "fresh log not reported as running"
printf '%s\n' "$out" | grep -q '无法证明' && die "fresh log reported 'cannot prove' - criterion crossed" || pass "no false 'cannot prove'"

echo ""
echo "-- 2. stale log (over 75 min) must report dead --"
old=$(date -v-120M '+%Y-%m-%d %H:%M:%S' 2>/dev/null || date -d '120 minutes ago' '+%Y-%m-%d %H:%M:%S')
printf '%s aegis-self-wake: POKED Aegis-Supervisor (pane=node)\n' "$old" > "$TMP/waker.log"
out=$(extract); printf '%s\n' "$out" | sed 's/^/   /'
printf '%s\n' "$out" | grep -q '❌' && pass "stale log -> dead (exit signal present)" \
  || die "stale log not caught - a genuinely dead waker would never alarm"

echo ""
echo "-- 3. log absent but cron + script present -> must say CANNOT PROVE, not dead --"
rm -f "$TMP/waker.log"
out=$(extract); printf '%s\n' "$out" | sed 's/^/   /'
printf '%s\n' "$out" | grep -q '无法证明' && pass "absent log -> says cannot prove" \
  || die "absent log did not say cannot prove"
printf '%s\n' "$out" | grep -q '❌' && die "still marked dead - that is the false alarm being fixed" \
  || pass "does not declare it dead"
printf '%s\n' "$out" | grep -q '别当成死了去重装' && pass "tells the reader not to reinstall" \
  || die "did not say not to reinstall - the reader still will"

echo ""
echo "-- 4. must-fail: log absent AND cron entry absent -> must say not installed --"
mkfs "$TMP/waker2.log"; rm -f "$TMP/waker2.log"
mkdir -p "$TMP/bin"
printf '#!/bin/sh\nexit 0\n' > "$TMP/bin/crontab"; chmod +x "$TMP/bin/crontab"
out=$(PATH="$TMP/bin:$PATH" bash "$TMP/fs.sh" 2>/dev/null | sed -n '/Aegis 自醒/,+4p')
printf '%s\n' "$out" | sed 's/^/   /'
printf '%s\n' "$out" | grep -q '没装好' && pass "no cron entry -> says not installed" \
  || die "no cron entry did not say not installed - two different failures collapsed into one"

echo ""
echo "-- 5. cleanup --"
rm -f "$TMP/waker.log" "$TMP/waker2.log"
[ -s "$WLOG" ] && pass "real log untouched by the test" || pass "real log was already absent; test did not fabricate one"
# 已知例外 scripts/bind-seatloom-seats.sh 是有意留给张先生决定是否入库的,
# 心跳协议的完成判据也把它算作已知未跟踪文件。把它算成"被测试改脏"是误报。
# scripts/test-fleet-status-liveness-probe.sh 是同一类例外:它是这份检查自己的
# 必败用例,2026-11-10 加进来证明 `pgrep -a -P` 那个修。它未入库前会被这里
# 算成脏 —— 那是这份清单该包含的东西,不是误报。
KNOWN_UNTACKED='scripts/bind-seatloom-seats.sh'
unexpected=$(git -C "$REPO" status --porcelain \
  | grep -v "scripts/fleet-status.sh\|test-fleet-status-selfwake\|test-fleet-status-liveness-probe\|$KNOWN_UNTACKED" || true)
if [ -n "$unexpected" ]; then
  printf '%s\n' "$unexpected" | sed 's/^/   dirty: /'
  die "the test dirtied something it should not have"
else
  pass "main checkout has no unrelated changes"
fi

echo ""
echo "OK: all four directions verified."
