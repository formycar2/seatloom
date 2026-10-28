#!/usr/bin/env bash
# integrate-memory-tools.sh — 把 Nimbus 的 packet/nimbus/memory-tools-018 集成进 track。
#
# 为什么要脚本:Aegis 在 `git checkout <分支> -- <路径>`(覆盖工作树)上被权限分类器拦住,
# 和"改 live 数据库"属同一类。按 AGENTS.md 的规矩不绕路、不拆小块、不换工具,
# 写成一个可审计、可重复跑的脚本交给张先生。
#
# 为什么这件事非做不可:
#   舰队此刻跑的 target/debug/seatloom-mcp-emit 报告 15 个工具(含 recall/remember/promote/forget),
#   但 track 的源码里**一个都没有** —— 等于 track 重建不出正在跑的二进制。
#   这正是 2026-10-10 Lyra 抓到的 request_acceptance 那个坑的复发形状。
#
# 这一次集成一并带来三样等了很久的东西:
#   1. 记忆工具(018 typed 字段)   —— 洗数据的产物才有地方落
#   2. 唤醒服务滞留 claim 修复      —— 前三次停摆的同一个根因
#   3. request_acceptance 包含性防护 —— "分支必须包含基线",那张坏卡的根治
#
# 纪律(AGENTS.md 教训 6):**先编译,后提交**。编译不过就原样回滚,绝不留半集成状态。
# 已核对:该分支在 crates/ 上已包含 track 的全部工作(`git log 分支..HEAD -- crates/` 为空),
# 且相对 track **不删除任何文件**,所以路径限定取是安全的。
#
# 用法: bash scripts/integrate-memory-tools.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1
BRANCH="packet/nimbus/memory-tools-018"
PATHS=(crates/ src-tauri/ ui/)

say(){ printf '%s\n' "$*"; }
fail(){ say "❌ $*"; exit 1; }

# ── 0. 前置检查:每一项都必须能报错 ────────────────────────────────
say "== 前置检查 =="

cur="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)" || fail "这里不是 git 仓库"
[ "$cur" = "track/infra-foundation" ] || fail "当前在 ${cur},不是 track/infra-foundation。先切回去。"
say "  分支                 ${cur}"

git rev-parse --verify "$BRANCH" >/dev/null 2>&1 || fail "找不到分支 ${BRANCH}"
say "  来源分支             ${BRANCH} ($(git log -1 --pretty='%h %s' "$BRANCH" | cut -c1-60))"

# 该分支必须已含 track 在 crates/ 上的全部提交,否则取过来会丢 track 的工作。
behind="$(git log --oneline "${BRANCH}..HEAD" -- crates/ | wc -l | tr -d ' ')"
[ "$behind" = "0" ] || fail "该分支落后 track ${behind} 个 crates/ 提交 —— 取过来会丢工作。停。"
say "  落后 track 的提交    0(安全)"

# 工作树在这三个路径上必须干净,否则回滚会连带吃掉别人没提交的改动。
dirty="$(git status --porcelain -- "${PATHS[@]}" | wc -l | tr -d ' ')"
[ "$dirty" = "0" ] || {
  say "  ⚠️ 这些路径下有未提交改动:"
  git status --short -- "${PATHS[@]}" | sed 's/^/      /'
  fail "先提交或撤销它们,再跑本脚本 —— 否则失败回滚会一起吃掉。"
}
say "  目标路径工作树       干净"

BEFORE="$(git rev-parse HEAD)"
say "  回滚锚点             ${BEFORE}"

# dev watcher 在跑是正常的:取文件会触发它重建,这正是新代码上线的路径。
# 它和下面的 cargo build 会在 target 锁上排队,不会互相弄坏(cargo 是等锁,不是报错)。
if pgrep -f 'tauri:dev' >/dev/null 2>&1; then
  say "  dev watcher          在跑(会自动重建并热加载,属预期)"
else
  say "  dev watcher          没在跑"
fi
say ""

# ── 1. 路径限定取出 ────────────────────────────────────────────────
say "== 取出源码(路径限定,不做整分支合并)=="
git checkout "$BRANCH" -- "${PATHS[@]}" || fail "checkout 失败,工作树未改动"
changed="$(git status --porcelain -- "${PATHS[@]}" | wc -l | tr -d ' ')"
say "  改动文件数           ${changed}"
[ "$changed" = "0" ] && { say ""; say "无变化 —— track 已经是最新,无需集成。"; exit 0; }
git status --short -- "${PATHS[@]}" | sed 's/^/    /'
say ""

# 失败就回到锚点状态,不留半集成
rollback(){
  say ""
  say "回滚中(回到 ${BEFORE} 的状态)..."
  git checkout "$BEFORE" -- "${PATHS[@]}" 2>/dev/null
  # 分支新增的文件在锚点里不存在,checkout 不会删,得显式清掉
  git clean -fd -- "${PATHS[@]}" >/dev/null 2>&1
  say "已回滚。工作树回到集成前。"
}

# ── 2. 先编译,后提交(AGENTS.md 教训 6)────────────────────────────
say "== 编译 =="
say "  先编 mcp-emit(承重的那部分,快速反馈)..."
if ! cargo build -p seatloom-mcp-emit --bins >/tmp/integrate-mcp.log 2>&1; then
  say "  ❌ mcp-emit 编译失败。最后 25 行:"
  tail -25 /tmp/integrate-mcp.log | sed 's/^/      /'
  rollback
  exit 1
fi
say "  mcp-emit             ✅ 过"

say "  再编整个 workspace(core / tauri / cli 都要能过,否则 CI 必红)..."
if ! cargo build --workspace >/tmp/integrate-ws.log 2>&1; then
  say "  ❌ workspace 编译失败。最后 25 行:"
  tail -25 /tmp/integrate-ws.log | sed 's/^/      /'
  rollback
  exit 1
fi
say "  workspace            ✅ 过"

# 格式只报告不阻断:CI 会跑 fmt,但格式问题不该让这次集成回滚。
if cargo fmt --all --check >/tmp/integrate-fmt.log 2>&1; then
  say "  cargo fmt            ✅ 干净"
else
  say "  cargo fmt            ⚠️ 有格式差异($(grep -c '^Diff in' /tmp/integrate-fmt.log 2>/dev/null) 处)——CI 会卡,但不影响本次集成"
fi
say ""

# ── 3. 端到端验证:编译过 ≠ 工具真上线 ─────────────────────────────
say "== 验证(问二进制自己要工具清单,不靠 grep 猜)=="
BIN="$REPO/target/debug/seatloom-mcp-emit"
probe="$(printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"probe","version":"0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | DATABASE_URL="postgresql://seatloom:seatloom@localhost:5432/seatloom" \
    SEATLOOM_ACTOR_REF="seat:seat-aegis-001" SEATLOOM_PROJECT_ID="seatloom" \
    "$BIN" 2>/dev/null)"
tools="$(printf '%s' "$probe" | python3 -c "
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    try: m = json.loads(line)
    except Exception: continue
    ts = m.get('result', {}).get('tools')
    if ts is not None:
        print(' '.join(sorted(t['name'] for t in ts)))
        break
" 2>/dev/null)"
if [ -z "$tools" ]; then
  say "  ❌ 二进制没回工具清单 —— 它起不来。不提交。"
  rollback
  exit 1
fi
n=$(printf '%s' "$tools" | wc -w | tr -d ' ')
say "  工具数               ${n}"
missing=""
for t in recall remember promote forget request_acceptance get_acceptance_lineage resolve; do
  printf '%s' "$tools" | grep -qw "$t" || missing="$missing $t"
done
[ -n "$missing" ] && { say "  ❌ 缺工具:${missing}"; rollback; exit 1; }
say "  记忆四件套 + 验收    ✅ 全在"
# 真正的目的:track 的源码能重建出这个二进制
src_n=$(grep -rc '"remember"' crates/seatloom-mcp-emit/src/*.rs 2>/dev/null | awk -F: '{s+=$2} END{print s+0}')
[ "${src_n:-0}" -gt 0 ] || { say "  ❌ track 源码里仍然没有 remember —— 集成没真正生效"; rollback; exit 1; }
say "  track 源码可重建     ✅(源码里有 remember)"
say ""

# ── 4. 提交 ────────────────────────────────────────────────────────
say "== 提交 =="
git add -- "${PATHS[@]}" || fail "git add 失败"
git commit -q -F - <<'MSG' || fail "提交失败"
feat(d5): integrate the memory tools, the waker fix, and acceptance containment onto track

track could not rebuild the binary the fleet was running. The deployed
seatloom-mcp-emit reported 15 tools — recall/remember/promote/forget among
them — while track's source contained none of them. Same shape as the
request_acceptance gap Lyra caught on 2026-10-10: a running binary with no
reproducible source.

Taken path-scoped from packet/nimbus/memory-tools-018, which already contains
every track commit under crates/ and deletes nothing. Built before committing
(mcp-emit bins, then the whole workspace) and verified by asking the rebuilt
binary for its own tool list rather than grepping for names.

Brings three things that were each blocking something:
  - typed memory tools (018) — the washing pipeline now has somewhere to land
  - waker stale-claim fix — the root cause behind three separate stalls
  - request_acceptance containment — branch-contains-base, so Accept can no
    longer delete work already merged to track

Known gap, deliberately not papered over: no production code path creates a
topic. memories.topic_id is NOT NULL with an FK to topics, and topics has 0
rows, so every remember call still fails. Escalated separately — the gate
event for the typed memory path stays unemitted until a topic can exist.
MSG
say "  $(git log -1 --pretty='%h %s' | cut -c1-72)"
say ""

# ── 5. 让唤醒服务用上新二进制 ──────────────────────────────────────
# 唤醒服务跑在一个 `while true` 重启循环里,杀掉进程它会自己用新二进制重起。
# ⚠️ 绝不能用 `pkill -f seatloom-waker` —— 那个模式**会连外层循环一起杀掉**
#    (2026-10-10 就这么把 supervisor 的重启循环误杀过)。必须按精确命令行匹配。
say "== 唤醒服务 =="
if ! tmux has-session -t seatloom-waker 2>/dev/null; then
  say "  ⚠️ 唤醒服务会话不在 —— 新派单不会被派下去。修:见 scripts/fleet-status.sh 的提示"
else
  pids=""
  for p in $(pgrep seatloom-waker 2>/dev/null); do
    cmd="$(ps -o command= -p "$p" 2>/dev/null)"
    # 两道闸,因为误杀循环的代价是唤醒服务彻底消失(2026-10-10 真的发生过):
    #   闸一:命令行必须**以**那个二进制结尾(循环的命令行以 `done` 结尾,不会命中)
    #   闸二:命令行里只要出现 while / tmux / zsh -c,一律跳过(那就是包装层,不是本体)
    case "$cmd" in
      *while*|*tmux*|*'zsh -c'*|*'bash -c'*) continue;;
    esac
    case "$cmd" in
      *'/seatloom-waker') pids="$pids $p";;
    esac
  done
  if [ -z "$(printf '%s' "$pids" | tr -d ' ')" ]; then
    say "  ⚠️ 没找到唤醒进程本体(只有循环)—— 循环会自己重起,跳过"
  else
    say "  重启进程${pids}(循环会自动用新二进制重起)"
    # shellcheck disable=SC2086
    kill $pids 2>/dev/null
    sleep 4
    newp="$(pgrep seatloom-waker 2>/dev/null | tr '\n' ' ')"
    if [ -n "$(printf '%s' "$newp" | tr -d ' ')" ]; then
      say "  ✅ 已起来(pid ${newp})—— 滞留 claim 修复生效"
    else
      say "  ❌ 没起来。看一眼:tmux attach -t seatloom-waker"
    fi
  fi
fi
say ""

say "全部完成。把上面这段贴给 Aegis。"
