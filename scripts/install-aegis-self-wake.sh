#!/usr/bin/env bash
# install-aegis-self-wake.sh — 把 Aegis 的自醒装到 cron 真的能执行的位置。
#
# 为什么需要一个安装器,而不是直接把 crontab 指向仓库里的脚本:
#   macOS 的 TCC 拦住 cron 对 ~/Documents 的一切访问。cron 到点执行仓库里的
#   scripts/aegis-self-wake.sh,报 `Operation not permitted`,脚本一行都没跑 ——
#   而日志里只会留下一行 bash 的报错,不看日志就会以为自醒活着。
#   2026-10-12 就是这么发现的:crontab 装好了、cron 也到点跑了,自醒其实是死的。
#
# 它做四件事,可重复跑:
#   一  把仓库里的原本拷到 ~/.local/bin/(TCC 不拦的位置)
#   二  装 crontab 行(幂等:先删掉所有指向旧位置的行)
#   三  读回 crontab 核对
#   四  用 cron 的裸环境实跑一次装好的副本,确认日志长出 POKED
#
# 第四步是关键:不验证"脚本能执行",验证"在 cron 会用的那个环境里真的叫到了"。
# 装完不看日志,等于没装。
#
# 用法: bash scripts/install-aegis-self-wake.sh [--check]
#       --check  只报状态(原本/副本/crontab/日志),不改任何东西
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$REPO/scripts/aegis-self-wake.sh"
DEST="$HOME/.local/bin/aegis-self-wake.sh"
LOG="/tmp/seatloom_aegis_waker.log"
MARK="# seatloom — Aegis 自醒 waker"
CRON_LINE="11,41 * * * * $DEST >> $LOG 2>&1"

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
CHECK_ONLY=0; [ "${1:-}" = "--check" ] && CHECK_ONLY=1

# ── 状态 ────────────────────────────────────────────────────────────
line; say "自醒状态"; line
[ -r "$SRC" ]   && say "  原本(仓库,人审这份)   ✅ $SRC"   || say "  原本(仓库,人审这份)   ❌ 不在"
if [ -r "$DEST" ]; then
  if cmp -s "$SRC" "$DEST"; then say "  副本(cron 跑这份)     ✅ 与原本一致"
  else say "  副本(cron 跑这份)     ⚠️  与原本不一致 —— 原本改过但没重装"; fi
else say "  副本(cron 跑这份)     ❌ 不在 —— cron 到点只会报 Operation not permitted"; fi

cur="$(crontab -l 2>/dev/null | grep -F 'aegis-self-wake' | grep -v '^#' || true)"
if [ -n "$cur" ]; then
  if [ "$cur" = "$CRON_LINE" ]; then say "  crontab               ✅ 已装且指向副本"
  else say "  crontab               ⚠️  装着,但指向别处:"; say "      $cur"; fi
else say "  crontab               ❌ 没有 —— 自醒是死的"; fi

if [ -s "$LOG" ]; then
  say "  日志                  最近 3 行:"
  tail -3 "$LOG" | sed 's/^/      /'
  if grep -q "Operation not permitted" "$LOG"; then
    say "  ⚠️  日志里有 Operation not permitted —— 那个位置 cron 进不去(TCC)"
  fi
else say "  日志                  (还没有)"; fi
say ""

if [ "$CHECK_ONLY" = "1" ]; then
  # --check 也要能报错:任何一项不合格就非零退出
  ok=1
  [ -r "$DEST" ] || ok=0
  cmp -s "$SRC" "$DEST" 2>/dev/null || ok=0
  [ "$cur" = "$CRON_LINE" ] || ok=0
  grep -q "Operation not permitted" "$LOG" 2>/dev/null && ok=0
  [ "$ok" = "1" ] && say "✅ 自醒健康" || say "❌ 自醒不健康 —— 跑 bash scripts/install-aegis-self-wake.sh 修"
  [ "$ok" = "1" ] && exit 0 || exit 1
fi

# ── 一:拷贝到 TCC 不拦的位置 ───────────────────────────────────────
line; say "一 — 装副本到 $DEST"; line
if [ ! -r "$SRC" ]; then say "  ❌ 原本不在:$SRC —— 停"; exit 1; fi
mkdir -p "$HOME/.local/bin"
if cp "$SRC" "$DEST" && chmod +x "$DEST"; then
  cmp -s "$SRC" "$DEST" && say "  ✅ 已拷贝且内容一致" || { say "  ❌ 拷完了但内容不一致 —— 停"; exit 1; }
else say "  ❌ 拷贝失败 —— 停"; exit 1; fi
say ""

# ── 二:装 crontab(幂等) ────────────────────────────────────────────
line; say "二 — 装 crontab"; line
existing="$(crontab -l 2>/dev/null || true)"
# 去掉所有指向旧位置/旧副本的行,以及旧标记行,避免重复累积
filtered="$(printf '%s\n' "$existing" | grep -v 'aegis-self-wake' | grep -vF "$MARK" || true)"
if printf '%s\n' "$filtered" | crontab -; then
  say "  ✅ 已清掉旧行(可重复跑)"
else
  say "  ❌ 清旧行失败 —— 停,不要在不干净的状态上叠加"; exit 1
fi
if printf '%s\n%s\n%s\n' "$filtered" "$MARK" "$CRON_LINE" | crontab -; then
  say "  ✅ 已写入新行"
else say "  ❌ 写 crontab 失败 —— 停"; exit 1; fi
say ""

# ── 三:读回核对 ────────────────────────────────────────────────────
line; say "三 — 读回核对"; line
back="$(crontab -l 2>/dev/null | grep -F 'aegis-self-wake' | grep -v '^#' || true)"
if [ "$back" = "$CRON_LINE" ]; then say "  ✅ crontab 读回一致"; else
  say "  ❌ 读回不符:"; say "     期望 $CRON_LINE"; say "     实际 $back"; exit 1; fi
say ""

# ── 四:用 cron 的裸环境实跑一次 ────────────────────────────────────
line; say "四 — 用 cron 的裸环境实跑(不是'脚本能执行',是'真的叫到了')"; line
before=0; [ -s "$LOG" ] && before="$(wc -l < "$LOG" | tr -d ' ')"
if env -i HOME="$HOME" PATH=/usr/bin:/bin /bin/sh -c "$DEST >> $LOG 2>&1"; then
  say "  ✅ 裸环境执行成功(exit 0)"
else
  say "  ❌ 裸环境执行失败 —— 自醒仍然是死的,不要当成装好了"; exit 1
fi
after=0; [ -s "$LOG" ] && after="$(wc -l < "$LOG" | tr -d ' ')"
if [ "$after" -gt "$before" ]; then
  say "  ✅ 日志长出了 $((after-before)) 行:"
  tail -1 "$LOG" | sed 's/^/     /'
else
  say "  ❌ 日志没长 —— 执行了但没留下任何痕迹,这比失败更难发现"; exit 1
fi
if grep -q "Operation not permitted" <(tail -1 "$LOG"); then
  say "  ❌ 最后一行是 Operation not permitted —— 位置还是不对"; exit 1
fi
say ""
line
say "✅ 自醒装好了。判据:crontab 读回一致 + 裸环境实跑 + 日志长出 POKED。"
say "   以后随时可以跑 bash scripts/install-aegis-self-wake.sh --check 自己核。"
say "   心跳协议在 docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md(改它不用重装)。"
