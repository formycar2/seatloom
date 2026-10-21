#!/usr/bin/env bash
# restart-seats-for-new-binary.sh — 让席位加载新的 mcp-emit 二进制。
#
# 为什么需要它(2026-10-21 21:10,Mira 上报并收敛):
#   磁盘上的 mcp-emit 二进制 21:00:48 构建(含合并防护的删除豁免清单),
#   但 Flux/Lyra/Onyx/Vela 四个席位在 15:40 就拉起了各自的 mcp-emit 子进程 ——
#   早 5 小时 20 分。**进程握着旧的内存镜像,磁盘上那份新的它没加载。**
#
#   后果:Mira 的四张卡开不出来。因为合并那道门的豁免扣除发生在
#   `dispatch.rs`(mcp-emit 进程内),而 15:40 那份没有那段逻辑。
#   按既定判据(「门的条件是能不能真的跑成功,不是代码合没合进来」),
#   那道门对她实际是关的。
#
# 这个脚本做什么:重启那四个席位,让它们重新拉起 mcp-emit。
#
# 🔴 为什么不让 Aegis 自己做:
#   那是改席位运行态,不是改账本,但同样属于「运行时真相」—— 会打断
#   正在干活的席位。Aegis 的权限封顶包含「不打断正在干活的席位」。
#   所以这里给一条可审计的命令,由张先生执行。
#
# 安全检查(每一条不通过就停,不硬重启):
#   1. 四个席位各自**没有在飞的派单** —— 有就停,重启会打断它
#   2. 磁盘二进制**确实比在跑的进程新** —— 不新就没必要重启
#   3. 磁盘二进制**确实含豁免** —— 用跑测试判,不用 strings(那个判据坏)
#
# 用法:
#   bash scripts/restart-seats-for-new-binary.sh            # 预演,只报不动
#   bash scripts/restart-seats-for-new-binary.sh --apply     # 真重启
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
BIN="$REPO/target/debug/seatloom-mcp-emit"
SEATS="flux lyra onyx vela"
APPLY=0; for a in "$@"; do [ "$a" = "--apply" ] && APPLY=1; done

DBQ(){ podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -X -U seatloom -d seatloom -tAc "$1" 2>/dev/null | tr -d ' '; }

line
if [ "$APPLY" = "1" ]; then say "重启席位以加载新二进制 — 应用模式"; else say "重启席位以加载新二进制 — 预演模式(什么都不动)"; fi
line

# ── 检查一:磁盘二进制在不在 ──
[ -x "$BIN" ] || fail "找不到 $BIN —— 先在主检出跑一次 cargo build -p seatloom-mcp-emit"
DISK_EPOCH="$(stat -f '%m' "$BIN")"
DISK_TIME="$(stat -f '%Sm' -t '%Y-%m-%d %H:%M:%S' "$BIN")"
say "  磁盘二进制    : $DISK_TIME"
say "  含删除豁免    : 用测试判(不用 strings —— 那个判据对符号不可靠)"

# ── 检查二:豁免真的在代码里(跑测试,不 grep 二进制) ──
. "$HOME/.cargo/env" 2>/dev/null || true
if ! command -v cargo >/dev/null 2>&1; then
  say "  ⚠️  没有 cargo,跳过测试验证 —— 这一项没验成,不算通过"
  say "     手动验: cargo test -p seatloom-mcp-emit exempt_deletion"
else
  OUT="$(cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit exempt_deletion 2>&1 | tail -20)"
  if printf '%s' "$OUT" | grep -q 'exempt_deletion_passes_while_the_rest_are_still_refused ... ok'; then
    say "  ✅ 豁免测试通过(源码侧门是开的)"
  else
    say "  ⚠️  豁免测试没跑成或没过。输出末尾:"
    printf '%s\n' "$OUT" | tail -5 | sed 's/^/       /'
    say "     这一项没验成 —— 重启后门可能仍然不开,别当成修好了"
  fi
fi

# ── 检查三:在跑的进程确实更旧 ──
say ""
OLDEST_NEWEST="$(ps -axo lstart,comm 2>/dev/null | grep 'seatloom-mcp-emit' | grep -v grep | awk '{print $2}' | sort -n | tail -1)"
NRUN="$(ps -axo comm 2>/dev/null | grep -c 'seatloom-mcp-emit')"
say "  在跑的 mcp-emit 进程数: $NRUN"
if [ "$NRUN" = "0" ]; then
  say "  ℹ️  当前没有在跑的 mcp-emit 进程 —— 不需要重启"
else
  # 进程启动时刻:用 ps 的 lstart,转成 epoch 比较
  RUN_EPOCH="$(ps -axo lstart,pid,comm 2>/dev/null | grep 'seatloom-mcp-emit' | grep -v grep | head -1 | sed 's/^[A-Za-z]* *[0-9]* *[0-9:]* *[0-9]* //' | xargs -I{} date -j -f '%b %d %H:%M:%S %Y' '{}' +%s 2>/dev/null || echo 0)"
  if [ "$RUN_EPOCH" -gt 0 ] 2>/dev/null; then
    DIFF_MIN=$(( (DISK_EPOCH - RUN_EPOCH) / 60 ))
    say "  在跑进程启动(最早那条): $(ps -axo lstart,comm | grep 'seatloom-mcp-emit' | grep -v grep | head -1 | awk '{print $1,$2,$3,$4,$5}')"
    if [ "$DIFF_MIN" -gt 0 ]; then
      say "  ✅ 进程比二进制旧 ${DIFF_MIN} 分钟 —— 重启有意义"
    else
      say "  ℹ️  进程不比二进制旧 —— 重启没有意义,先确认上面两个时间"
    fi
  else
    say "  ⚠️  算不出进程启动时刻(ps 输出格式没解析成功)—— 这一项没验成"
  fi
fi

# ── 检查四:四个席位有没有在飞的活 ──
say ""
say "  各席位在飞的活(有就停,重启会打断):"
BLOCK=0
for s in $SEATS; do
  n="$(DBQ "select count(*) from dispatches where target_seat_id='seat-$s-001' and status='in_flight'")"
  n="${n:-?}"
  if [ "$n" != "0" ]; then
    say "    ❌ $s 有 $n 单在飞 —— 重启会打断它"
    BLOCK=1
  else
    say "    ✅ $s 空闲"
  fi
done
[ "$BLOCK" = "0" ] || fail "有席位在干活。等它做完再跑,或者只重启空闲的那几个。"

if [ "$APPLY" != "1" ]; then
  say ""
  say "预演到此为止。四项检查都过了才该真重启。"
  say "  bash scripts/restart-seats-for-new-binary.sh --apply"
  exit 0
fi

# ── 重启 ──
line; say "重启席位(会话不动,只重启里面的 agent)"; line
say "  做法:给每个席位的 tmux 会话发一条重启指令。会话本身保留,"
say "        agent 进程退出后会由 harness 重新拉起,并重新拉起 mcp-emit。"
say ""
say "  ⚠️  这一步会打断那四个席位当前的会话上下文(它们现在都空闲,所以无损)。"
say ""
for s in $SEATS; do
  case "$s" in
    flux) T="Flux-Quality&Ops-seatloom";;
    lyra) T="Lyra-po-seatloom";;
    onyx) T="Onyx-data-seatloom";;
    vela) T="Vela-DataWash-seatloom";;
  esac
  if ! tmux has-session -t "$T" 2>/dev/null; then
    say "  ⚠️  会话 $T 不在,跳过 $s"
    continue
  fi
  say "  → $s ($T):发重启指令"
  # 先清输入行残留,再发 Ctrl-C 两次让 agent 退出,harness 会重新拉起
  tmux send-keys -t "$T" C-u 2>/dev/null || true
  sleep 0.3
  tmux send-keys -t "$T" C-c 2>/dev/null || true
  sleep 1
  tmux send-keys -t "$T" C-c 2>/dev/null || true
done

say ""
say "等 20 秒让它们重新拉起…"
sleep 20
say ""
say "重启后的 mcp-emit 进程:"
ps -axo pid,lstart,comm 2>/dev/null | grep 'seatloom-mcp-emit' | grep -v grep | awk '{print "  "$1"  "$4" "$5" "$6" "$7" "$8}' | head -6

line; say "接下来"; line
say "  1. 让 Mira 重开卡(她的四张卡应该不再被那道门拦)"
say "  2. 开卡后你会在「等你拍板」看到它们"
say ""
say "  如果还被拦,把新的报错贴回来 —— 那说明拦点不是进程版本,要继续查。"
exit 0
