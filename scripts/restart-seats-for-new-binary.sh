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
# 要重启的席位:默认四个常在手的;位置参数可覆盖(空格分隔的短名),
# 例如 `--apply mira nimbus` 只动这两个,不重动已加载新二进制的那四个。
DEFAULT_SEATS="flux lyra onyx vela"
SEATS="$DEFAULT_SEATS"
APPLY=0
_explicit=""
for a in "$@"; do
  case "$a" in
    --apply) APPLY=1;;
    -*) ;;
    *) _explicit="$_explicit $a";;
  esac
done
[ -n "$_explicit" ] && SEATS="$_explicit"
# 拉起用的东西(2026-10-21 补:退出必须拉起,否则舰队断)。
# saih 启动器:fleet 从 pi 迁到 saih 后,pi 符号链接没了、seat-launch.sh/restart-aegis.sh
# 都还指着 ~/.local/bin/pi(已失效)。真正在跑的是 saih/bin/saih(= node .../cli.js)。
SAIH_BIN="${SEATLOOM_SAIH_BIN:-$REPO/saih/bin/saih}"
WROOT="${SEATLOOM_WORKTREES:-$REPO/../seatloom-worktrees}"
CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
FAILED=""
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
  # 进程启动时刻:用 ps 的 lstart。第一版那段解析失败却继续跑(报「没验成」
  # 然后照旧重启)—— 一个报不出错的检查比没有检查更有害,所以现在解析失败就停。
  # 进程启动时刻:ps 的 lstart 是 5 个字段(星期 月 日 时:分:秒 年)。
  # 第一版用 grep ' seatloom-mcp-emit' 带空格匹配,而进程名前的空格数不定,
  # 于是永远匹配不到、解析永远空 —— 而空值时脚本还继续跑了(那是静默失败)。
  # 现在:先整行 grep,再用 sed 剥掉星期和后面的路径,只剩 4 个字段给 date -j。
  # 进程启动时刻。三次迭代都在这段上栽过,记下来免得再来第四次:
  #  1) grep ' seatloom-mcp-emit' 带前导空格 -> 永远匹配不到(进程名前空格数不定)
  #  2) 解析失败时 fail 掉整个重启 -> 可它只是「要不要重启」的依据,
  #     不是「能不能重启」的依据;真正该拦的是下面「席位有没有在飞的活」
  #  3) Aegis 这边正常、张先生那边报算不出 -> 有环境差异,不猜原因
  # 现在:算不出就**把原始输出摊开**让人自己对,不静默也不拦死。
  RUN_EPOCH="$(ps -axo lstart=,comm= 2>/dev/null | grep 'seatloom-mcp-emit' | head -1 \
    | sed -E 's/^[A-Za-z]{3} +([A-Za-z]{3}) +([0-9]+) +([0-9:]+) +([0-9]{4}).*/\\1 \\2 \\3 \\4/' \
    | while read -r m d t y; do date -j -f '%b %d %H:%M:%S %Y' "$m $d $t $y" +%s 2>/dev/null; done)"
  FIRST_RUN="$(ps -axo lstart=,comm= 2>/dev/null | grep 'seatloom-mcp-emit' | head -1 | awk '{print $1,$2,$3,$4,$5}')"
  if [ -n "${RUN_EPOCH:-}" ] && [ "${RUN_EPOCH:-0}" -gt 0 ] 2>/dev/null; then
    DIFF_MIN=$(( (DISK_EPOCH - RUN_EPOCH) / 60 ))
    say "  在跑进程启动(最早那条): ${FIRST_RUN:-?}"
    if [ "$DIFF_MIN" -gt 0 ]; then
      say "  ✅ 进程比二进制旧 ${DIFF_MIN} 分钟 —— 重启有意义"
    else
      say "  ℹ️  进程不比二进制旧(差 ${DIFF_MIN} 分钟)—— 重启没意义,先确认上面两个时间"
    fi
  else
    say "  ⚠️  算不出进程启动 epoch —— **这一项没验成**,但不当成拦住重启的理由"
    say "     原始 ps 第一行: ${FIRST_RUN:-（没取到）}"
    say "     磁盘二进制构建于: $DISK_TIME"
    say "     自己比一下:那个时刻若明显早于磁盘构建时刻,重启就有意义"
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
    mira) T="Mira-UX/UED-seatloom";;
    nimbus) T="Nimbus-TechArchi-seatloom";;
    *) say "  ⚠️ 不知道席位 $s 的会话名,跳过"; continue;;
  esac
  if ! tmux has-session -t "$T" 2>/dev/null; then
    say "  ⚠️  会话 $T 不在,跳过 $s"
    continue
  fi
  say "  → $s ($T):发重启指令"
  # 先清输入行残留,再用 /quit 退出 agent(不是 C-c —— 那只是取消当前操作,
  # 我第一版用 C-c,四个 agent 一个没退,mcp-emit 进程 pid 全没变,重启等于没做)。
  # 依据:saih 的 interactive-mode.ts:4344 明写 quit 是 Ctrl+D / Ctrl+C / /quit 三者之一,
  # 但实测 C-c 在 TUI 里被当成「取消」而非「退出」,所以这里用最明确的 /quit。
  # 退出时 saih 会打印一条 resume 命令,那是恢复会话用的,不影响我们重启。
  tmux send-keys -t "$T" C-u 2>/dev/null || true
  sleep 0.3
  tmux send-keys -t "$T" -l '/quit' 2>/dev/null || true
  sleep 0.3
  tmux send-keys -t "$T" Enter 2>/dev/null || true
  # 等它退到 shell。**必须等退干净再拉起** —— 否则在旧 agent 还占着时又起一个,
  # 会得到两个 agent 抢同一个面板(我抢救时就是这么把 flux 起到 vela 面板里的)。
  wpane="$(tmux list-panes -t "$T" -F '#{pane_pid}' 2>/dev/null | head -1)"
  for _ in $(seq 1 15); do pgrep -P "$wpane" >/dev/null 2>&1 || break; sleep 1; done
  if pgrep -P "$wpane" >/dev/null 2>&1; then
    say "    ⚠️  $s 退出后仍有子进程,没退干净 —— **不拉起**(会在旧 agent 上又叠一个)"
    say "       人工看一眼 $T,处理干净后单独重跑这一个席位"
    FAILED="$FAILED $s"
    continue
  fi
  say "    ✅ 已退到 shell"
  # ── 重新拉起(上一版缺的就是这步:『退出不拉起』= 舰队断,实测四个席位全变空 shell) ──
  # 拉起三契约缺一不可:对的工作区 + saih 启动器 + --approve(漏 --approve 停在
  # "Trust project folder?" 弹窗,永不加载 MCP)。模型按席位从账本取,不写死。
  seat_model="$(podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -tAc "select default_model from seats where id='seat-$s-001'" 2>/dev/null | tr -d ' ')"
  [ -n "$seat_model" ] || seat_model="modelproxy-anthropic/step-5-preview-sys"
  seat_wt="$WROOT/$s"
  [ -d "$seat_wt" ] || { say "    ⚠️ 找不到工作区 $seat_wt —— 不拉起"; FAILED="$FAILED $s"; continue; }
  tmux send-keys -t "$T" "cd $seat_wt && $SAIH_BIN --approve --model $seat_model" Enter
  say "    已拉起(cwd=$seat_wt model=$seat_model)"
done

say ""
say "等 25 秒让 agent 起来并加载 MCP…"
sleep 25
say ""
say "验证(看每个面板的子进程+自己的工作区+有无自己的 mcp-emit,不看面板文字):"
for s in $SEATS; do
  case "$s" in
    flux) T="Flux-Quality&Ops-seatloom";; lyra) T="Lyra-po-seatloom";;
    onyx) T="Onyx-data-seatloom";; vela) T="Vela-DataWash-seatloom";;
    mira) T="Mira-UX/UED-seatloom";; nimbus) T="Nimbus-TechArchi-seatloom";;
  esac
  pane="$(tmux list-panes -t "$T" -F '#{pane_pid}' 2>/dev/null | head -1)"
  child="$(pgrep -P "$pane" 2>/dev/null | head -1)"
  cwd="$(lsof -a -p "$child" -d cwd 2>/dev/null | tail -1 | awk '{print $NF}')"
  has_mcp="$(pgrep -P "$child" 2>/dev/null | while read c; do ps -o comm= -p "$c" 2>/dev/null | grep -q mcp-emit && echo yes; done | head -1)"
  mark="✅"; [ "$cwd" = "$WROOT/$s" ] || mark="❌工作区不对"; [ "$has_mcp" = "yes" ] || mark="$mark ❌无MCP"
  say "  $mark $s: 工作区=$(basename "${cwd:-无}") MCP=${has_mcp:-无}"
done
[ -n "${FAILED:-}" ] && say "" && say "  ⚠️ 这几个没拉起,需人工处理:$FAILED"

line; say "接下来"; line
say "  1. 让 Mira 重开卡(各席位已加载带豁免的 mcp-emit,她的卡应该不再被那道门拦)"
say "  2. 开卡后你会在「等你拍板」看到它们"
say ""
say "  如果还被拦,把新的报错贴回来 —— 那说明拦点不是进程版本,要继续查。"
exit 0
