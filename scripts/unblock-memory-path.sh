#!/usr/bin/env bash
# unblock-memory-path.sh — 一条命令打通记忆写入路径。
#
# 为什么要脚本:Aegis 被权限分类器拦住两类动作 —— 改 live 数据库(发事件/派单都是写账本)
# 和 `git checkout <分支> -- <路径>`(覆盖工作树)。今天各拦一次,**都拦对了**。
# 按 AGENTS.md 的规矩:不绕路、不拆小块、不换工具、不让子 agent 代跑,写成脚本交给张先生。
#
# 它分三段,后一段不被前一段的失败阻断(派单的必要性与编译结果无关):
#   第零段  提交 Aegis 侧的工作(裁决文档 + 这两个脚本)—— Aegis 连 git commit 都被拦
#   第一段  把 Nimbus 的 memory-tools-018 集成进 track:先编译,过了才提交;不过就原样回滚
#   第二段  写账本:发一个**条件真实成立**的门事件,派一张单给 Nimbus
#
# 它**刻意不做**的事:不发 ev-typed-memory-tools-landed。
# 理由见下方第二段的打印说明,以及
# docs/coordination/tasks/aegis/AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md
#
# 全部可重复跑:门事件按固定 id 发(重复则跳过),派单按固定 id 派(重复则跳过)。
#
# 用法: bash scripts/unblock-memory-path.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

RULING="docs/coordination/tasks/aegis/AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md"
BIN="$REPO/target/debug/seatloom-mcp-emit"
export DATABASE_URL="${DATABASE_URL:-postgresql://seatloom:seatloom@localhost:5432/seatloom}"
export SEATLOOM_ACTOR_REF="seat:seat-aegis-001"
export SEATLOOM_PROJECT_ID="seatloom"

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }

# ══ 第零段:提交 Aegis 侧的工作 ═════════════════════════════════════
# Aegis 连 `git commit` 都被权限分类器拦住了,所以这几个文件只能在这里落盘。
# 刻意**不用 `git add -A`** —— 只加下面这张显式清单,内容就是已逐个扫过
# 密钥样貌串(ak-/sk-/ghp_/AKIA/Bearer/60+ 字符高熵)的那几个文件,
# 不会把你工作树里别的东西顺手卷进来。
line
say "第零段 — 提交 Aegis 侧的工作(裁决文档 + 这两个脚本)"
line
AEGIS_FILES=(
  AGENTS.md
  scripts/seat-launch.sh
  scripts/fleet-status.sh
  scripts/configure-wash-provider.sh
  scripts/register-vela-seat.sh
  scripts/integrate-memory-tools.sh
  scripts/unblock-memory-path.sh
  .seatloom/dispatch-seats.tsv
  "$RULING"
)
present=()
for f in "${AEGIS_FILES[@]}"; do [ -e "$f" ] && present+=("$f"); done
if [ "${#present[@]}" = "0" ]; then
  say "  没有文件可提交,跳过"
elif [ -z "$(git status --porcelain -- "${present[@]}")" ]; then
  say "  这些文件已全部提交过,跳过(可重复跑)"
else
  git add -- "${present[@]}" && git commit -q -F - <<'MSG'
ops(aegis): rule on the two escalations, and hand Mr. Zhang one command for each blocked act

Vela and Lyra both escalated correctly, and both land in the supervisor lane.

Vela's blocker (a) — live DB at the 014 state — is gone: the 015-018 apply
landed and verified. Blocker (b) is half-gone: the binary now reports all 15
tools, but every seat's pi process predates the rebuild, so each seat's MCP
child still holds the old image.

Checking those claims surfaced two things neither seat could see:

  1. track cannot rebuild the binary the fleet is running. remember/recall/
     promote/forget appear zero times in track's source. Same shape as the
     request_acceptance gap Lyra caught yesterday.

  2. No production code path creates a topic. memories.topic_id is NOT NULL
     with an FK to topics, topics has 0 rows, and the only INSERT INTO topics
     anywhere lives inside #[cfg(test)] fixtures. So every remember call fails
     regardless of schema or binary — which blocks Vela's S5 and every seat's
     forward-recording obligation.

Two rulings, written up in full rather than asserted:
  - topics get a governed open_topic tool. Not implicit creation in remember
    (a typo would mint an unreachable scope, and opened_by NOT NULL shows the
    schema expects a deliberate opener), and not hand-seeded SQL (the pipeline
    infers topics from the corpus, so pre-seeding overrides the one seat that
    has actually read the data).
  - Lyra's post-verdict wake gap is closed by making the pre-declared verdict
    event id a contract, not by adding a second NOTIFY site. A dependency edge
    is data in the ledger; an implicit notify is behaviour in the source. The
    premise of governed dispatch is that graph edges are data.

ev-018-memory-hardening-landed is emitted — its condition is genuinely true.
ev-typed-memory-tools-landed is deliberately withheld until one real remember
call succeeds. Adopting Vela's distinction fleet-wide: a gate asserts
live-usable, not code-merged.

Both scripts verify before they act and roll back rather than leave a half
state: integrate-memory-tools.sh builds before committing and restores the
tree on failure; unblock-memory-path.sh re-measures the gate conditions itself
and skips the event if they do not hold.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
MSG
  if [ "$?" = "0" ]; then say "  ✅ $(git log -1 --pretty='%h %s' | cut -c1-72)"
  else say "  ❌ 提交失败 —— 不影响后面两段,继续"; fi
fi
say ""

# ══ 第一段:集成到 track ════════════════════════════════════════════
line
say "第一段 — 把记忆工具的源码集成进 track"
line
integrate_rc=0
if [ -x scripts/integrate-memory-tools.sh ] || [ -r scripts/integrate-memory-tools.sh ]; then
  bash scripts/integrate-memory-tools.sh
  integrate_rc=$?
else
  say "❌ 找不到 scripts/integrate-memory-tools.sh"
  integrate_rc=1
fi
say ""
if [ "$integrate_rc" = "0" ]; then
  say "第一段结果:✅ 成功"
else
  say "第一段结果:❌ 失败(退出码 ${integrate_rc})—— 工作树已回滚,未留半集成状态"
  say "            第二段继续:派单给 Nimbus 这件事不依赖编译结果。"
fi
say ""

# ══ 第二段:写账本 ══════════════════════════════════════════════════
line
say "第二段 — 写账本(发门事件 / 派单)"
line

[ -x "$BIN" ] || { say "❌ 找不到 MCP 二进制 ${BIN}"; exit 1; }

# 条件必须真实成立才发门事件(AGENTS.md 第 5 条:向账本谎称某事已发生,比卡住更糟)。
# 这里不信任上面的打印,重新实测一遍。
say "核对 ev-018-memory-hardening-landed 的真实条件:"
q(){ podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc "$1" 2>/dev/null; }

c_mem=$(q "select count(*) from pg_tables where tablename='memories'")
c_kind=$(q "select count(*) from information_schema.columns where table_name='memories' and column_name='kind'")
c_l2=$(q "select count(*) from information_schema.columns where table_name='memories' and column_name like 'l2_line%'")
c_src=$(grep -rc '"remember"' crates/seatloom-mcp-emit/src/*.rs 2>/dev/null | awk -F: '{s+=$2} END{print s+0}')

ok=1
chk(){ # chk 名称 实测值 期望下界
  if [ "${2:-0}" -ge "$3" ]; then printf "  %-28s ✅ (%s)\n" "$1" "$2"
  else printf "  %-28s ❌ (%s,需 >= %s)\n" "$1" "${2:-空}" "$3"; ok=0; fi
}
chk "live 有 memories 表"      "$c_mem"  1
chk "live 有 kind 列"          "$c_kind" 1
chk "live 有 L2 行级溯源列"    "$c_l2"   1
chk "track 源码含 remember"    "$c_src"  1
say ""

# 把要发的调用写成 JSON-RPC,交给二进制自己执行(走受治理的工具路径,不碰裸 SQL)
CALLS="$(mktemp)"; trap 'rm -f "$CALLS"' EXIT
python3 - "$ok" "$RULING" > "$CALLS" <<'PY'
import json, sys
gate_ok = sys.argv[1] == "1"
ruling  = sys.argv[2]

calls = []

if gate_ok:
    calls.append(("emit_event", {
        "type": "SupervisorMessage",
        "event_id": "ev-018-memory-hardening-landed",
        "refs": [{"ref_type": "project", "ref_id": "seatloom", "role": "subject"}],
        "payload": {
            "gate": "ev-018-memory-hardening-landed",
            "statement": "Migration 018 is live-applied and its source is on track.",
            "verified": (
                "015-018 applied to the running seatloom-postgres and verified: memories table, "
                "memories.kind, L2 line-range columns, projects.parent_project_id, "
                "acceptances.acceptance_plan, and the cancelled status vocab all present. "
                "Existing data untouched (74 dispatches, 1 acceptance card). "
                "Source landed on track/infra-foundation."
            ),
            "note": (
                "Vela's distinction is adopted fleet-wide: a gate's condition is 'live-usable', "
                "not 'code-merged'. This gate asserts only the 018 schema, nothing about the "
                "typed memory write path."
            ),
            "issued_by": "seat-aegis-001",
        },
    }))

# open_topic 的派单。payload 自足:席位的 worktree 可能看不到 track 上的新文档
# (2026-10-10 的教训),所以要点内联,并给出取回路径。
instructions = f"""裁决全文在 track 上:{ruling}
你的 worktree 若没有这个文件,用这条取回:
  git show track/infra-foundation:{ruling}

== 为什么派这张单 ==
生产代码没有任何路径能创建 topic,所以记忆写入路径此刻完全不可用:
  - memories.topic_id 是 NOT NULL 且 FK -> topics(id)
  - topics 现有 0 行
  - 全仓 INSERT INTO topics 只有 3 处,全在 #[cfg(test)] 测试夹具里
    (dispatch.rs 第 3105/3494 行附近,cfg(test) 自第 2431 行起)
  - 15 个 MCP 工具里没有任何创建 topic 的工具
后果:schema 已落库、二进制已有工具,但每一次 remember 仍会在外键上失败。
这不只卡 Vela 的 S5,也卡每个席位的前向记录义务(完成协议第 4 步强制 remember)。

== 要求一:新增 MCP 工具 open_topic ==
入参      topic_id(必填)、title(必填)、project_id(可选,缺省取 SEATLOOM_PROJECT_ID,绝不编造)、
          parent_topic_id(可选,必须已存在)
opened_by 服务端解析 SEATLOOM_ACTOR_REF,**绝不作为入参** —— 与 get_next_dispatch 同纪律:
          席位身份永远不是参数
幂等      同 topic_id 重复调用成功、不改既有行,返回现有行
拒绝      parent 不存在 / project 不存在 / 父子跨 project / 自环或环路
          -> 一律 typed rejection + 零写入
事件      一个事务内写行 + 发事件(与 remember 的 row+event 同纪律),refs 的 subject 为 topic

需要你判定并在交付说明里写明:27 个 PascalCase 事件类型里没有 TopicOpened。
要么复用既有类型,要么新增(那会动值域契约,须逐字出现在两侧 packet,
且跨 Rust<->PG TEXT 边界手写 FromStr/Display)。**不要默默选一个** ——
把两种做法的代价写出来,选一个并说明理由。

为什么不是别的做法(已裁决,不要改方向):
  不让 remember 自动建 topic —— 一个拼写错误就凭空产生一个新作用域且无人察觉,
  记忆静默落进谁也不会召回的孤岛。schema 自己就要求有意的开启者(opened_by NOT NULL)。
  不手工 SQL 播种 topic 树 —— 洗数据流水线是从语料里推断 topic 的,
  预先钉死等于替 Vela 决定它还没读出来的东西。

== 要求二:判定事件 id 从"好习惯"升为"契约" ==
Lyra 10-10 17:34 升级:ReviewVerdictIssued 落地后没有任何机制唤醒发起席位,
这是"5 条 Flux 全绿、0 张验收卡"的根因。
裁决:**不新增 NOTIFY 站点**。现有的 blocked->ready 触发器(013)已经提供这个唤醒,
前提是存在一条指向预声明判定事件 id 的依赖边 —— Lyra 为 Part 2 做的正是这个,它是对的。
依赖边是账本里的数据(可查、可见、可审计);新增 NOTIFY 是代码里的隐式行为。
治理式派单的前提就是"图的边是数据",不拿它换便利。
要实现:请求验证的路径必须**强制**产出判定事件 id ——
由工具铸造(或校验席位提供的那个)并返回,Flux 发判定时必须用这个 id,
请求方据此挂 gated 自派单。你已交付的周期性扫描(e65d438)是兜底,不替代契约。
不追溯 Part 1(它按 Lyra 写好的手工程序建卡,不要为它加代码)。

== 完成协议 ==
按你自己 AGENTS.md 的完成协议走。注意前向记录(remember)在 open_topic 上线前仍不可用 ——
若 remember 仍失败,在交付说明里写明并照常 complete_dispatch,不要卡在那一步。"""

calls.append(("enqueue_dispatch", {
    "id": "disp-open-topic-nimbus",
    "target_seat_id": "seat-nimbus-001",
    "title": "open_topic MCP tool + verdict-event-id contract (Aegis ruling 2026-10-11)",
    "priority": 60,
    "payload": {"packet": ruling, "instructions": instructions},
}))

msgs = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize",
     "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "aegis-unblock", "version": "1"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
]
for i, (name, args) in enumerate(calls, start=2):
    msgs.append({"jsonrpc": "2.0", "id": i, "method": "tools/call",
                 "params": {"name": name, "arguments": args}})
for m in msgs:
    print(json.dumps(m, ensure_ascii=False))
PY

if [ "$ok" = "1" ]; then
  say "条件成立 → 发 ev-018-memory-hardening-landed"
else
  say "条件不成立 → **不发**门事件(只派单)。"
  say "            向账本谎称某事已发生,比卡住更糟。"
fi
say "派单          disp-open-topic-nimbus → seat-nimbus-001(优先级 60)"
say ""
say "执行中..."
"$BIN" < "$CALLS" 2>/dev/null | python3 -c "
import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    try: m = json.loads(line)
    except Exception: continue
    if m.get('id') in (None, 1): continue
    if 'error' in m:
        e = m['error']
        print('  ❌ 失败:', str(e.get('message'))[:200])
        continue
    r = m.get('result') or {}
    if r.get('isError'):
        txt = ' '.join(c.get('text','') for c in r.get('content') or [])
        # 已存在属于幂等成功,不是失败
        low = txt.lower()
        if 'already' in low or 'duplicate' in low or 'exists' in low or 'conflict' in low:
            print('  ⏭  已存在,跳过(可重复跑):', txt[:160])
        else:
            print('  ❌ 工具拒绝:', txt[:260])
        continue
    txt = ' '.join(c.get('text','') for c in r.get('content') or [])
    print('  ✅', (txt or json.dumps(r, ensure_ascii=False))[:260])
"
say ""

line
say "核对账本(这是唯一可信的证据,不看上面的打印)"
line
printf "  门事件 ev-018-memory-hardening-landed   %s\n" \
  "$([ "$(q "select count(*) from canonical_events where id='ev-018-memory-hardening-landed'")" = "1" ] && echo ✅已入账 || echo '未入账')"
printf "  派单 disp-open-topic-nimbus             %s\n" \
  "$(q "select coalesce(max(status),'未入账') from dispatches where id='disp-open-topic-nimbus'")"
printf "  ev-typed-memory-tools-landed            %s\n" \
  "$([ "$(q "select count(*) from canonical_events where id='ev-typed-memory-tools-landed'")" = "0" ] && echo '刻意未发(等一次真实 remember 成功)' || echo '已入账')"
printf "  topics 行数                             %s  (仍为 0 = 记忆写入路径仍不可用,正是本次派单要解决的)\n" "$(q "select count(*) from topics")"
say ""
say "全部完成。把上面这段贴给 Aegis。"
