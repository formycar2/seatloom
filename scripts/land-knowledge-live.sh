#!/usr/bin/env bash
# land-knowledge-live.sh — 一条命令让记忆写入路径**真的能跑**。
#
# 背景:三条 Flux 已验绿的交付(Onyx 的 019 迁移、Nimbus 的 open_topic、Nimbus 的知识读模型)
# 已于 5bd500d 路径限定地落到 track,程序也已重建并自报 18 个工具(含 open_topic)。
# 但 **live 数据库还是 27 个事件类型的旧 CHECK**,所以 open_topic 一定会在数据库上失败。
# 按「门的条件是 live 可用,不是代码合并」,现在还不能发任何门事件。
#
# 为什么要脚本:Aegis 被权限分类器拦住**改 live 数据库**的一切操作(拦得对)。
# 按 AGENTS.md:不绕路、不拆小块、不换工具、不让席位代跑 —— 写成可审计、可重复跑的脚本交给张先生。
#
# 它分十段,**前一段失败就停**(后面每一段都以前一段为真实前提):
#   第一段  把缺的迁移应用到正在跑的数据库(委派 apply-schema-to-live.sh,它自动发现 015+)
#   第二段  核对 CHECK 真的变了(不信上一段的打印)
#   第三段  实证一次真实写入:一次 open_topic + 一次 remember(019 没落成也照跑 —— 要的就是那条拒绝原文)
#   第四段  只有第三段两步都成功,才发门事件
#   第五段  让全队换上带新工具的程序
#   第六段  回填那张坏卡的空方案,让界面席位能撤掉它(不依赖记忆写入路径,实证失败也会做)
#   第七段  实证知识读模型真的能调通,条件成立才发它的门事件(同样不依赖记忆写入路径)
#   第八段  把「交付公告被当成门」这个结构问题派出去修(同样不依赖记忆写入路径)
#   第九段  叫醒洗数据席位收尾它自己卡死的领单
#   第十段  把唤醒服务那个漏洞派给后端席位修
#
# 全部可重复跑:019 自带幂等守卫,open_topic 幂等,remember 用固定 id 的门事件去重。
#
# 用法: bash scripts/land-knowledge-live.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

BIN="$REPO/target/debug/seatloom-mcp-emit"
MIG="$REPO/infra/postgres/schema/019_topic_opened.sql"   # 只用于「在不在盘上」的前置检查;应用由 apply-schema-to-live.sh 负责
CTR="seatloom-postgres"

export DATABASE_URL="${DATABASE_URL:-postgresql://seatloom:seatloom@localhost:5432/seatloom}"
export SEATLOOM_ACTOR_REF="seat:seat-aegis-001"
export SEATLOOM_PROJECT_ID="seatloom"

apply_ok=1   # 019 是否真的落到 live(第一、二段实测)。只用于告诉你接下来会发生什么;
             # 真正的控制权在 proof_ok —— 门事件和换程序一律只看「有没有真的写进去」。
say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
# 库内查询。**刻意不吞 stderr** —— 2026-10-11 用 2>/dev/null 把一个列名写错的查询
# 吞成了「0 行」,据此差点误判。一个永远不会报错的检查比没有检查更有害。
q(){ podman exec -e PGPASSWORD=seatloom "$CTR" psql -U seatloom -d seatloom -tAc "$1"; }

[ -x "$BIN" ] || { say "❌ 找不到程序 ${BIN} —— 先跑 cargo build --workspace"; exit 1; }
[ -r "$MIG" ] || { say "❌ 盘上找不到 019 迁移 ${MIG} —— 仓库是不是不在 track 分支?"; exit 1; }
podman exec "$CTR" true 2>/dev/null || {
  say "❌ 数据库容器 ${CTR} 不在跑 —— 先跑 podman machine start && podman start ${CTR}"; exit 1; }

# ══ 第一段:把缺的迁移应用到正在跑的数据库 ═════════════════════════
line
say "第一段 — 把缺的迁移应用到正在跑的数据库"
line
# **委派给 scripts/apply-schema-to-live.sh,不自己手搓 psql。**
# 它本来就是「把 track 上已验绿的表结构落到正在跑的库」那一件事的唯一入口:
# 不碰 seed、不建库、不删东西,逐个幂等 apply,跑完打印一整块验证。
# 而且它现在**从目录里自动发现 015 及以后的全部迁移**(不再写死文件名),
# 所以缺的不只 019 时它也能一次补齐 —— 我这边再写一遍只会分叉出第二条真相。
# (它自己也踩过并记下了「psql 跑在容器里,不能 -f 传宿主机路径」这条,
#  所以它用 stdin 管文件,和 f00b854 修的是同一个坑。)
APPLIER="$REPO/scripts/apply-schema-to-live.sh"
if [ ! -r "$APPLIER" ]; then
  say "  ❌ 找不到 ${APPLIER}"
  apply_ok=0
elif bash "$APPLIER"; then
  say ""
  say "  ✅ 迁移应用完成"
else
  say ""
  say "  ❌ 迁移应用失败(上面那块验证结果就是证据)。"
  say "     第三段仍会实测一次 —— 它会被数据库拒绝,那条拒绝原文也贴给我。"
  say "     第四、五段(门事件 / 换程序)因此不会执行;第六~八段照跑 —— 它们不依赖迁移。"
  apply_ok=0
fi
say ""

# ══ 第二段:核对 CHECK 真的变了 ════════════════════════════════════
line
say "第二段 — 核对 live 的事件类型词表真的含 TopicOpened"
line
if vocab="$(q "select pg_get_constraintdef(oid) from pg_constraint where conname='canonical_events_event_type_vocab'")"; then
  case "$vocab" in
    *TopicOpened*) say "  ✅ live CHECK 已含 TopicOpened";;
    *) say "  ❌ live CHECK 仍不含 TopicOpened —— open_topic 还是会失败。"
       say "     第三段仍会实测并失败(那就是证据);第四、五段不执行;第六~八段照跑。"; apply_ok=0;;
  esac
else
  say "  ❌ 查询失败 —— 当作条件不成立"; apply_ok=0
fi
say "  词表值个数:$(q "select cardinality(string_to_array(regexp_replace(pg_get_constraintdef(oid),'.*ARRAY\[|\].*','','g'),','))" 2>/dev/null || echo '(数不出来,不影响判定)')"
say ""

# ══ 第三段:实证一次真实写入 ═══════════════════════════════════════
line
say "第三段 — 实证 live 写入路径:一次 open_topic + 一次 remember"
line
say "  这是整条链通不通的唯一判据。两步都成功才算通。"
say ""

# 刻意用一条**真实的**记忆,不是测试数据:今天这次停摆的根因与修法。
# 它同时履行 Aegis 的前向记录义务(完成协议第 4 步)。
CALLS="$(mktemp)"; trap 'rm -f "$CALLS"' EXIT
python3 - > "$CALLS" <<'PY'
import json

TOPIC = "topic-aegis-fleet-operations"

calls = [
    ("open_topic", {
        "topic_id": TOPIC,
        "title": "舰队运行:停摆的根因与修法",
        "project_id": "seatloom",
    }),
    ("remember", {
        "topic_id": TOPIC,
        "kind": "failure",
        "confidence": "inferred",
        "title": "三条已验绿的交付停在分支上,记忆写入路径因此整夜不可用",
        "l0_summary": (
            "门的条件按「代码合并」核会漏掉 live:019 迁移、open_topic、知识读模型三条 "
            "Flux 全绿却都停在分支上,topics 保持 0 行,于是 remember 整夜一次也没成功过。"
        ),
        "l1_overview": (
            "2026-10-10 19:18(库内时钟)之后账本再无事件。实测:track 源码里 open_topic 出现 0 次,"
            "live 的 canonical_events_event_type_vocab 仍是 27 值不含 TopicOpened,topics 0 行、memories 0 行。"
            "memories.topic_id 是 NOT NULL 且外键指向 topics,所以每一次 remember 必然在外键上失败。"
            "同时洗数据席位有一条 in_flight 的领单从未 complete——唤醒服务把它当成「在忙,会自排空」而永久跳过,"
            "它后面 4 张 ready 单因此一张都发不出去。"
            "修法:路径限定地把三条交付集成到 track(整分支合并会删掉 015-018 和六个脚本,因为分支基线更早),"
            "编译通过后提交(5bd500d),重建程序确认自报 18 个工具,再把 019 应用到 live,"
            "然后实证一次 open_topic + 一次 remember 才发门事件。"
        ),
        "l2_ref_kind": "file",
        "l2_file_path": "infra/postgres/schema/019_topic_opened.sql",
        "l2_line_start": 1,
        "l2_line_end": 80,
        "l2_body_digest": (
            "019 把事件类型词表从 27 加宽到 28(+TopicOpened),带守卫的 DROP+ADD 写全了 post-018 的完整清单。"
            "在它应用到 live 之前,open_topic 一定在数据库 CHECK 上失败。"
        ),
        "source_kind": "canonical_event",
        "source_ref": "ev-verdict-open-topic",
        "tags": ["gate", "live-vs-landed", "waker", "integration"],
        "project_id": "seatloom",
    }),
]

msgs = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize",
     "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "aegis-land-knowledge", "version": "1"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
]
for i, (name, args) in enumerate(calls, start=2):
    msgs.append({"jsonrpc": "2.0", "id": i, "method": "tools/call",
                 "params": {"name": name, "arguments": args}})
for m in msgs:
    print(json.dumps(m, ensure_ascii=False))
PY

"$BIN" < "$CALLS" 2>&1 | python3 -c "
import sys, json
names = {2: 'open_topic', 3: 'remember'}
for raw in sys.stdin:
    raw = raw.strip()
    if not raw:
        continue
    try:
        m = json.loads(raw)
    except Exception:
        # 程序写到 stderr 的东西照原样打出来,不吞
        print('  stderr:', raw[:240]); continue
    i = m.get('id')
    if i in (None, 1):
        continue
    label = names.get(i, f'调用#{i}')
    if 'error' in m:
        print(f'  ❌ {label} 协议级失败:', str(m[\"error\"].get(\"message\"))[:300]); continue
    r = m.get('result') or {}
    txt = ' '.join(c.get('text','') for c in (r.get('content') or []))
    if r.get('isError'):
        low = txt.lower()
        if 'already' in low or 'exists' in low or 'duplicate' in low:
            print(f'  ⏭  {label} 已存在,幂等跳过:', txt[:200])
        else:
            print(f'  ❌ {label} 被拒:', txt[:400])
    else:
        print(f'  ✅ {label}:', (txt or json.dumps(r, ensure_ascii=False))[:300])
"
say ""

# 账本是唯一可信的证据(AGENTS.md 第 3 条:验证靠账本,不靠面板)
n_topic="$(q "select count(*) from topics")"
n_mem="$(q "select count(*) from memories")"
say "  账本实测:topics ${n_topic} 行 / memories ${n_mem} 行"
if [ "${n_topic:-0}" -ge 1 ] && [ "${n_mem:-0}" -ge 1 ]; then
  proof_ok=1
  say "  ✅ 记忆写入路径**真的通了** —— 这是第一次有真实记忆落库"
else
  proof_ok=0
  say "  ❌ 没有真实写入 —— 停在这里,**不发门事件**。"
  say "     向账本谎称某事已发生,比卡住更糟:下游会基于假前提开工。"
fi
say ""

# ══ 第四段:只有实证通过才发门事件 ═════════════════════════════════
line
say "第四段 — 门事件"
line
if [ "$proof_ok" = "1" ]; then
  python3 - > "$CALLS" <<'PY'
import json
msgs = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize",
     "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "aegis-land-knowledge", "version": "1"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
    {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
        "name": "emit_event",
        "arguments": {
            "type": "SupervisorMessage",
            "event_id": "ev-knowledge-write-path-live",
            "refs": [{"ref_type": "project", "ref_id": "seatloom", "role": "subject"}],
            "payload": {
                "gate": "ev-knowledge-write-path-live",
                "statement": "The typed memory write path is live-usable: one real open_topic and one real remember have succeeded against the running database.",
                "verified": "019 applied to the running seatloom-postgres (event-type vocab 27 -> 28, +TopicOpened, verified by reading pg_constraint, not by trusting the apply output); open_topic created topic-aegis-fleet-operations; remember wrote one real memory row. topics and memories both non-zero in the ledger.",
                "landed": "track 5bd500d — 019_topic_opened.sql, open_topic (topics.rs), the knowledge read model (knowledge.rs + Tauri commands + UI types), taken path-limited from packet/onyx/topic-opened-019 and packet/nimbus/knowledge-readmodel after cargo build --workspace passed.",
                "note": "Emitted only after the proof, per the live-usable-not-code-merged rule. The three Flux verdicts behind it: ev-019-topic-opened-flux-verdict, ev-verdict-open-topic, ev-verdict-knowledge-readmodel.",
                "issued_by": "seat-aegis-001",
            },
        }}},
]
for m in msgs:
    print(json.dumps(m, ensure_ascii=False))
PY
  say "  条件成立 → 发 ev-knowledge-write-path-live"
  "$BIN" < "$CALLS" 2>&1 | python3 -c "
import sys, json
for raw in sys.stdin:
    raw = raw.strip()
    if not raw: continue
    try: m = json.loads(raw)
    except Exception: print('  stderr:', raw[:200]); continue
    if m.get('id') in (None, 1): continue
    if 'error' in m:
        print('  ❌ 失败:', str(m['error'].get('message'))[:240]); continue
    r = m.get('result') or {}
    txt = ' '.join(c.get('text','') for c in (r.get('content') or []))
    if r.get('isError'):
        low = txt.lower()
        print(('  ⏭  已存在,跳过:' if ('already' in low or 'exists' in low or 'duplicate' in low) else '  ❌ 被拒:'), txt[:240])
    else:
        print('  ✅', (txt or json.dumps(r, ensure_ascii=False))[:240])
"
else
  say "  跳过 —— 实证没过,门事件不发。"
fi
say ""

# ══ 第五段:让全队换上带新工具的程序 ═══════════════════════════════
line
say "第五段 — 让全队换上带新工具的程序"
line
say "  席位的工具进程是它启动那一刻的程序镜像。程序重建 ≠ 全队换上了新的。"
if [ "$proof_ok" = "1" ]; then
  if [ -r "$REPO/scripts/restart-seats-for-new-tools.sh" ]; then
    bash "$REPO/scripts/restart-seats-for-new-tools.sh"
  else
    say "  ❌ 找不到 scripts/restart-seats-for-new-tools.sh"
  fi
else
  say "  跳过 —— 实证没过,重启席位没有意义(新工具在库上还是会失败)。"
fi
say ""

# 实证没过就停在这里。后面两段的措辞里写着「019 已应用、remember 已成功」——
# 实证没过的话那是假的,而向席位谎称某事已发生,比卡住更糟:它会基于假前提开工。
# ══ 第六段:让界面席位能撤掉那张坏卡 ═══════════════════════════════
line
say "第六段 — 让界面席位能撤掉那张坏卡"
line
# 界面席位 17:09 把根因查死了,它的分析是对的,我核过:
#   坏卡 acc-disp-me6Nm8e9D6ZRe4fKyVokY 的 acceptance_plan 是 '{}',
#   而 016 加的 acceptances_plan_required CHECK (acceptance_plan <> '{}') 是 NOT VALID。
#   NOT VALID 只豁免「ALTER 那一刻的既有行不被校验」,**不豁免这一行以后被 UPDATE**。
#   cancel_acceptance 只写 status/feedback/decided_by/decided_at、根本不碰 acceptance_plan,
#   但 UPDATE 会让 Postgres 重新校验这一行 —— 所以任何参数组合都过不去。
#   它调了五次、每次都被同一个 CHECK 拒绝,然后停下来上报,没有绕过。这是对的。
# 016 当初就把这件事记在自己头上:「the one legacy pending row is exempt until
# Aegis backfills the plan and VALIDATEs」。回填是总控的 live 动作,所以在这里做。
say "  坏卡的 acceptance_plan 是空的,而 016 的 CHECK 会在任何 UPDATE 时重新校验这一行。"
say "  回填这一行 → 界面席位就能撤卡。"

cur_plan="$(q "select acceptance_plan::text from acceptances where id='acc-disp-me6Nm8e9D6ZRe4fKyVokY'")"
if [ -z "${cur_plan:-}" ]; then
  say "  ⏭  这张卡已经不在了,跳过"
elif [ "$cur_plan" != "{}" ]; then
  say "  ⏭  已回填过($(printf '%s' "$cur_plan" | wc -c | tr -d ' ') 字符),跳过(可重复跑)"
else
  # 刻意**不**把 Lyra 为别的 lane 写的方案抄到这张卡上 —— 那会伪造血缘。
  # Lyra 没有为 dispatch-ui 这张卡写过方案(夜间拆解给的是 knowledge-panel 那条),
  # 所以这里写的是**撤卡本身**的验收方案,并在 JSON 里如实说明它是什么。
  #
  # 整条 UPDATE 由 python 生成后从 stdin 喂给 psql。**不要用 `-c` 配 psql 变量** ——
  # psql 对 `-c` 的字符串不做变量插值,`:'plan'` 会原样送给服务器然后报语法错误
  # (实测过,所以换成美元引号)。JSON 里出现定界符就换一个,不硬着来。
  SQL="$(python3 <<'PY'
import json, sys
plan = json.dumps({
  'plan_kind': 'withdrawal',
  'note': '这张卡是 bootstrap 装进来的,不是 request_acceptance 调用产生的:它的 base 是字符串 track(本仓没有这个分支),所以 cmd_accept_delivery 会在 checkout 处拒绝,这张卡永远不可能被 grant。本字段不是交付的验收方案 —— Lyra 从未为 dispatch-ui 这张卡写过方案 —— 而是「撤掉这张卡」这个动作本身的验收方案。dispatch-ui 的工作本身不受影响,会带血缘和结构化方案重新建卡。',
  'steps': [
    {'action': '打开 SeatLoom 的待验收队列', 'input': '无'},
    {'action': '确认这张卡已经不在待验收列表里', 'input': '卡号 acc-disp-me6Nm8e9D6ZRe4fKyVokY'},
    {'action': '确认队列里剩下的卡都能点(base 是真实分支、方案不为空)', 'input': '无'},
  ],
  'expected': [
    {'output': '队列可打开', 'expect': '不报错'},
    {'output': '这张卡状态为 cancelled,不再出现在待验收列表', 'expect': '张先生看不到点下去只会报错的卡'},
    {'output': '剩余卡片每张都有非空方案和真实的 base 分支', 'expect': '队列里只剩人能真的判一次的卡'},
  ],
}, ensure_ascii=False)
tag = '$aegisplan$'
if tag in plan:
    sys.stderr.write('JSON 含美元引号定界符,中止\n'); sys.exit(1)
print("UPDATE acceptances SET acceptance_plan = %s%s%s::jsonb "
      "WHERE id = 'acc-disp-me6Nm8e9D6ZRe4fKyVokY' "
      "AND acceptance_plan = '{}'::jsonb;" % (tag, plan, tag))
PY
)"
  if [ -z "${SQL:-}" ]; then
    say "  ❌ 生成回填语句失败 —— 跳过"
  elif printf '%s\n' "$SQL" | podman exec -i -e PGPASSWORD=seatloom "$CTR" \
         psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 -q; then
    say "  ✅ 回填完成"
  else
    say "  ❌ 回填失败 —— 跳过本段剩余步骤"
  fi
fi

# 回填之后再 VALIDATE:让「空方案的卡」从此在数据库层面不可能存在,
# 这本来就是 016 的目的,之前只是因为那一行是空的才不得不留 NOT VALID。
if [ "$(q "select count(*) from acceptances where acceptance_plan = '{}'::jsonb")" = "0" ]; then
  if [ "$(q "select convalidated::text from pg_constraint where conname='acceptances_plan_required'")" = "true" ]; then
    say "  ⏭  acceptances_plan_required 已是 VALIDATED,跳过"
  elif podman exec -e PGPASSWORD=seatloom "$CTR" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 -q \
         -c "ALTER TABLE acceptances VALIDATE CONSTRAINT acceptances_plan_required"; then
    say "  ✅ acceptances_plan_required 已 VALIDATE —— 从此空方案的卡在库层面不可能存在"
  else
    say "  ❌ VALIDATE 失败(不影响撤卡,继续)"
  fi
else
  say "  ⏭  还有空方案的卡,不 VALIDATE(会失败)"
fi

# 回填成功才叫醒它 —— 否则等于叫它去撞第六次同一面墙
if [ "$(q "select acceptance_plan::text from acceptances where id='acc-disp-me6Nm8e9D6ZRe4fKyVokY'")" != "{}" ]; then
  MS='Mira-UX/UED-seatloom'
  if tmux has-session -t "$MS" 2>/dev/null; then
    NUDGE="$(mktemp)"
    cat > "$NUDGE" <<'EOF'
Aegis:你 17:09 的根因分析是对的,我核过了 —— 失败的 CHECK 是 acceptances_plan_required,
它作用在 acceptance_plan 列上,而 NOT VALID 只豁免「ALTER 那一刻的既有行」,不豁免这一行以后被 UPDATE。
cancel_acceptance 不碰那一列,但 UPDATE 会让 Postgres 重新校验整行,所以任何参数组合都过不去。
你调了五次、每次停下来上报而不是绕过,这是对的。

你要的那件事已经做完了(回填是总控的 live 动作,016 当初就记在总控头上):
  acc-disp-me6Nm8e9D6ZRe4fKyVokY 的 acceptance_plan 已回填为非空
  acceptances_plan_required 已 VALIDATE

**刻意没有**把 Lyra 为别的 lane 写的方案抄到这张卡上 —— 那会伪造血缘。Lyra 从没为 dispatch-ui
这张卡写过方案,所以回填的内容是「撤卡这个动作本身」的验收方案,JSON 里的 plan_kind='withdrawal'
如实写明了它是什么。

现在请重做第六次调用,参数不变:
  cancel_acceptance(acceptance_id='acc-disp-me6Nm8e9D6ZRe4fKyVokY', reason=<你之前用的那版>)
然后按完成协议收尾。如果又被拒,把拒绝原文原样抄给我,不要绕过、不要裸 SQL。
EOF
    tmux load-buffer -b aegism8 "$NUDGE" && tmux paste-buffer -b aegism8 -t "$MS" && tmux delete-buffer -b aegism8 \
      && say "  ✅ 已叫醒界面席位重做撤卡" || say "  ❌ 叫醒失败 —— 手动跟它说一声"
    rm -f "$NUDGE"
  else
    say "  ⚠️ 界面席位会话不在 —— 回填已完成,等它回来自己撤"
  fi
else
  say "  跳过叫醒 —— 回填没成,叫它只会撞第六次同一面墙"
fi
say ""

# ══ 第七段:知识读模型的门事件 ═════════════════════════════════════
line
say "第七段 — 知识读模型的门事件"
line
# Lyra 的头条建卡单(disp-lyra-onight-card-headline-v2)挂着两条依赖边:
#   event ev-knowledge-read-model-landed   ← 本段负责
#   event ev-writein-landed                ← 洗数据席位真的把 88 条记录写进去才算,不由本脚本发
# 读模型这一条**今天就已经是真的**,而且不依赖 019:它只读表,不写事件。
# 实测(2026-10-11 17:3x,用磁盘上的二进制直连 live):
#   get_knowledge_tree(project_id=seatloom) → 成功,返回 {"projectId":"seatloom","topics":[]}
#   get_topic_memories(topic_id=不存在)     → 带类型的拒绝「topic not found」(在工作,不是崩)
# 空树是**正确的空**,不是失败:库里确实还没有主题。
# 但按 AGENTS.md 第 7 条,门事件的条件一律按「能不能真的跑成功」核 ——
# 所以这里**重新实测一次**,不信上面这段注释,也不信代码合没合进来。
say "  重新实测一次读模型(不信注释、不信代码是否合并)"
RMCALLS="$(mktemp)"
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"aegis-land-knowledge","version":"1"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_knowledge_tree","arguments":{"project_id":"seatloom"}}}' \
  > "$RMCALLS"
rm_ok="$("$BIN" < "$RMCALLS" 2>/dev/null | python3 -c "
import sys, json
for raw in sys.stdin:
    raw = raw.strip()
    if not raw: continue
    try: m = json.loads(raw)
    except Exception: continue
    if m.get('id') != 2: continue
    if 'error' in m:
        print('0 协议级失败: ' + str(m['error'])[:200]); break
    r = m.get('result') or {}
    txt = ' '.join(c.get('text','') for c in (r.get('content') or []))
    if r.get('isError'):
        print('0 工具拒绝: ' + txt[:200])
    else:
        print('1 ' + txt.replace(chr(10), ' ')[:160])
    break
else:
    print('0 没有收到响应')
")"
rm -f "$RMCALLS"
case "$rm_ok" in
  1\ *) say "  ✅ get_knowledge_tree 调通:${rm_ok#1 }"; rm_proof=1;;
  *)    say "  ❌ get_knowledge_tree 没调通:${rm_ok#0 }"; rm_proof=0;;
esac

if [ "${rm_proof:-0}" = "1" ]; then
  python3 - > "$CALLS" <<'PY'
import json
msgs = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize",
     "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "aegis-land-knowledge", "version": "1"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
    {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
        "name": "emit_event",
        "arguments": {
            "type": "SupervisorMessage",
            "event_id": "ev-knowledge-read-model-landed",
            "refs": [{"ref_type": "project", "ref_id": "seatloom", "role": "subject"}],
            "payload": {
                "gate": "ev-knowledge-read-model-landed",
                "statement": "The knowledge read model is live-usable: get_knowledge_tree returns successfully against the running database, and get_topic_memories gives a typed not-found rather than failing.",
                "verified": "Called against the live database with the on-disk binary, not inferred from the merge. get_knowledge_tree(project_id=seatloom) succeeded and returned an empty topics array — the correct empty, since no topic has been opened yet. get_topic_memories with an unknown topic_id returned the typed rejection 'topic not found'. Re-measured by this script immediately before emitting.",
                "landed": "track 5bd500d — crates/seatloom-mcp-emit/src/knowledge.rs, the Tauri knowledge commands, and the UI api/types surface, taken path-limited from packet/nimbus/knowledge-readmodel after cargo build --workspace passed. Flux verdict ev-verdict-knowledge-readmodel (pass).",
                "note": "This gate asserts ONLY that the read path works. It says nothing about there being anything to read: the other edge on Lyra's headline dispatch, ev-writein-landed, belongs to the washing seat and is earned by actually writing the 88 staged records.",
                "issued_by": "seat-aegis-001",
            },
        }}},
]
for m in msgs:
    print(json.dumps(m, ensure_ascii=False))
PY
  say "  条件成立 → 发 ev-knowledge-read-model-landed"
  "$BIN" < "$CALLS" 2>&1 | python3 -c "
import sys, json
for raw in sys.stdin:
    raw = raw.strip()
    if not raw: continue
    try: m = json.loads(raw)
    except Exception: print('  stderr:', raw[:200]); continue
    if m.get('id') in (None, 1): continue
    if 'error' in m:
        print('  ❌ 失败:', str(m['error'].get('message'))[:240]); continue
    r = m.get('result') or {}
    txt = ' '.join(c.get('text','') for c in (r.get('content') or []))
    if r.get('isError'):
        low = txt.lower()
        print(('  ⏭  已存在,跳过:' if ('already' in low or 'exists' in low or 'duplicate' in low) else '  ❌ 被拒:'), txt[:240])
    else:
        print('  ✅', (txt or json.dumps(r, ensure_ascii=False))[:240])
"
else
  say "  条件不成立 → **不发**门事件。向账本谎称某事已发生,比卡住更糟。"
fi
say "  (另一条边 ev-writein-landed 不由本脚本发 —— 那是洗数据席位真的写进 88 条才算)"
say ""

# ══ 第八段:把「公告当门用」这个结构问题派出去修 ═══════════════════
line
say "第八段 — 把「交付公告被当成门」这个结构问题派出去修"
line
# 2026-10-11 逐个核了 9 个被当成门用的事件,2 个此刻为假:
#   ev-019-topic-opened-landed    (Onyx 10-10 18:42)  live 词表不含 TopicOpened
#   ev-typed-memory-tools-landed  (Nimbus 10-10 17:30) remember 一次也没成功过
# 两个事件都**没有说谎** —— payload 里只有 branch/commit/migration,是席位对自己交付的公告。
# 说谎的是**消费方式**:有人把公告当成「能力可用」的门,挂了依赖边。
# 可测量的损失一次:disp-onight-writein 被这个门放出来,然后必然失败。
# 裁决全文:docs/coordination/tasks/aegis/AEGIS-2026-10-11-gate-vs-announcement-ruling.md
say "  9 个门里 2 个此刻为假;损失已发生一次(写入库那张单被放出来然后必然失败)。"
RULING2="docs/coordination/tasks/aegis/AEGIS-2026-10-11-gate-vs-announcement-ruling.md"
python3 - "$RULING2" > "$CALLS" <<'PY'
import json, sys
ruling = sys.argv[1]

instructions = f"""裁决全文在 track 上:{ruling}
你的 worktree 若没有这个文件,用这条取回:
  git show track/infra-foundation:{ruling}

== 已测到的事实(不要重测,直接用) ==
把 dispatch_dependencies 里 dep_kind='event' 的 9 个门逐个按「live 可用」核过,2 个此刻为假:
  ev-019-topic-opened-landed    (Onyx 10-10 18:42)  payload 只有 branch/migration;live 词表不含 TopicOpened
  ev-typed-memory-tools-landed  (Nimbus 10-10 17:30) payload 只有 branch/commit;remember 一次也没成功过
两个事件都没有说谎 —— 它们是席位对自己交付的公告,内容准确。
说谎的是消费方式:有人把公告当成「这个能力现在能用」的门,挂了依赖边,
013 的触发器看到事件入账就把下游从 blocked 翻成 ready。
可测量的损失一次:disp-onight-writein 被 ev-typed-memory-tools-landed 放出来,
然后必然失败(失败原因从第一秒起就是确定的),消耗了一次重试预算。
对照:Flux 验 019 迁移代码那张单也挂在 ev-019-topic-opened-landed 上,这是合理的 ——
验代码只需要代码落地。同一个事件,一条边用对了、一条边用错了。

== 裁决(方向已定,不要改) ==
1) 凭 payload 判定一个事件能不能当门:能当门的必须有一个字段写明怎么实测出来的
   (调了什么、返回了什么、在哪个库上)。参照 ev-knowledge-write-path-live 和
   ev-knowledge-read-model-landed 里的 verified 字段。
   只有 branch/commit/migration/artifact_id 的是交付公告,不能当门。
2) 新门 id 一律 ev-<能力>-live;公告继续用 -landed。
   已有的 -landed 门不改名不撤回(账本只追加),但不再新增指向 -landed 的依赖边。
3) 「等代码落地」是正当需求,不要消灭它。判据是这张下游单会不会真的去调那个能力:
   会调 -> 必须挂 -live;只读代码/写文档/做验证 -> 挂 -landed 就对。

== 要你们判定的那件事(这是派单的核心,不要跳过) ==
机械化的**位置**我刻意没有定死,因为有一个真实的设计张力:
门事件 id 是**预声明**的 —— gated 自派单的前提就是「边先挂上、事件后到」。
所以「挂边时校验事件带不带 verified」做不到:那一刻事件还不存在。
校验只能发生在**释放时刻**,也就是 013 的 fn_dispatch_recompute_ready
把 blocked 翻成 ready 的那一刻。

于是:
  - 判据的形状(payload 里要有什么才算门)是 **Nimbus 的工具契约**
  - 释放时刻的校验在数据库函数里,改它是 **Onyx 的迁移槽**
请两边都出意见,**不要有人单方面定**。需要在交付说明里写明:
  (a) 判据具体长什么样,以及一个只有 branch/commit 的 payload 会怎样被判定
  (b) 校验放在触发器里还是别处,以及为什么;放触发器里的话,
      一个「永远等不到合格门事件」的下游单会变成什么状态(永久 blocked?
      还是要有一个能上报的终态?)—— 这个问题必须有答案,不能让单子静默悬着
  (c) 对现存那些指向 -landed 的边怎么处理(我的裁决是不动,但如果你们认为
      有更好的办法,写出来说服我)
  (d) **一张没有任何依赖边的单,算不算「已核过」?** 同日傍晚出现了这个病的镜像:
      Lyra 在自己报「没活可干」三分钟后重派 disp-onight-writein-v2 给洗数据席位,
      一条依赖边都没挂、直接 ready,而它要的前置(019 应用到 live)当时不成立 ——
      这张单从被派出的第一秒起就注定失败,烧光了对方 1/1 的重试预算,
      两次失败报告逐字相同。而对方在第一次失败里就明确写过「在 019 应用前不要重派」。
      我的裁决是「重派必须把导致失败的那个前置挂成边」(裁决五)。
      但**首次派单无边是正常的**(前置本来就成立),所以机械判据不能简单禁止无边 ——
      要能区分「首次派单」和「重派」。请给出你们的判据。

== 完成协议 ==
按你自己 AGENTS.md 的完成协议走。这张单的产出是**一个有两边签字的方案 + 实现**,
不是单方面的代码。方案和实现可以在同一次交付里给,但方案部分必须能看出两边都表过态。"""

msgs = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize",
     "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "aegis-land-knowledge", "version": "1"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
    {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
        "name": "enqueue_dispatch",
        "arguments": {
            "id": "disp-gate-vs-announcement-enforcement",
            "target_seat_id": "seat-nimbus-001",
            "title": "a delivery announcement is not a gate: make the blocked->ready release check for live-usability (Aegis ruling 2026-10-11)",
            "priority": 65,
            "payload": {"packet": ruling, "instructions": instructions},
        }}},
]
for m in msgs:
    print(json.dumps(m, ensure_ascii=False))
PY
"$BIN" < "$CALLS" 2>&1 | python3 -c "
import sys, json
for raw in sys.stdin:
    raw = raw.strip()
    if not raw: continue
    try: m = json.loads(raw)
    except Exception: print('  stderr:', raw[:200]); continue
    if m.get('id') in (None, 1): continue
    if 'error' in m:
        print('  ❌ 失败:', str(m['error'].get('message'))[:240]); continue
    r = m.get('result') or {}
    txt = ' '.join(c.get('text','') for c in (r.get('content') or []))
    if r.get('isError'):
        low = txt.lower()
        print(('  ⏭  已派过,跳过:' if ('already' in low or 'exists' in low or 'duplicate' in low) else '  ❌ 被拒:'), txt[:240])
    else:
        print('  ✅', (txt or json.dumps(r, ensure_ascii=False))[:240])
"
say ""

if [ "$proof_ok" != "1" ]; then
  line
  say "停在第八段 —— 第九、十段不执行"
  line
  say "  第九、十段要对席位说「019 已应用、记忆写入已实证成功」。实证没过,这话就是假的。"
  say "  先把上面第三段打印的那条拒绝原因贴给 Aegis。"
  exit 1
fi
say ""

# ══ 第九段:解开洗数据席位那条卡死的领单 ═══════════════════════════
line
say "第九段 — 解开洗数据席位那条卡死的领单"
line
# 实测(/tmp/seatloom_waker.log,2026-10-11):
#   seat-vela-001 in_flight disp-wash-vela-p2 for 63344s (pane running pi) — an agent is working; not poking
# 领单从 10-10 17:34 起一直 in_flight,**但那份活它早就交了**
#   (10-10 17:58 ArtifactCreated,commit e72a250;Flux 也已验绿 ev-flux-verdict-wash-p2)
# —— 只是从来没调 complete_dispatch。唤醒服务因此永久跳过它,它后面 4 张 ready 单一张都发不出去。
#
# 为什么不由脚本直接改账本:改领单行是席位自己的动作,不是总控的。
# 这里只做**叫醒 + 说清要做什么**(tmux send-keys 是既定的席位间协调通道)。
VELA_SESS="$(awk -F'\t' '$1=="seat-vela-001"{print $2}' "$REPO/.seatloom/dispatch-seats.tsv" 2>/dev/null | head -1)"
if [ -z "${VELA_SESS:-}" ]; then
  say "  ❌ 席位表里查不到 seat-vela-001 的会话名 —— 跳过"
elif ! tmux has-session -t "$VELA_SESS" 2>/dev/null; then
  say "  ❌ 会话 ${VELA_SESS} 不在 —— 跳过"
else
  stuck="$(q "select id from dispatches where target_seat_id='seat-vela-001' and status='in_flight' limit 1")"
  if [ -z "${stuck:-}" ]; then
    say "  ✅ 已经没有卡住的领单,无需处理"
  else
    age="$(q "select round(extract(epoch from (now()-started_at))/60) from dispatches where id='${stuck}'")"
    say "  卡住的领单:${stuck}(已 ${age} 分钟,年龄由数据库自己的时钟算)"
    say "  叫醒 ${VELA_SESS},让它自己收尾这条领单"
    tmux send-keys -t "$VELA_SESS" "你那条领单 ${stuck} 已 in_flight ${age} 分钟,但你的交付早就做完了(10-10 17:58 的 ArtifactCreated,commit e72a250,Flux 已验绿 ev-flux-verdict-wash-p2)—— 你只是从没调 complete_dispatch。唤醒服务因此把你当成「在忙」而永久跳过,你后面 4 张 ready 单一张都发不出去。请按完成协议收尾这一条:确认工作已提交、交付说明在位,然后 complete_dispatch(${stuck})。收尾后再按队列顺序继续;其中「真的写进库」那张单的前置是记忆写入路径 live 可用 —— 019 已应用、open_topic 与 remember 已各实证成功一次,你的工具进程也已换成带 18 个工具的新程序。" C-m
    say "  ✅ 已发出(叫醒是否生效看账本:该领单是否转为 done,不看面板)"
  fi
fi
say ""

# ══ 第十段:把唤醒服务的这个漏洞派出去修 ═══════════════════════════
line
say "第十段 — 把唤醒服务的这个漏洞派给后端席位修"
line
say "  陈旧领单检测是在跑的,它正确地算出了 17 小时;但它因为「pane 里有 pi 活着」而拒绝叫醒。"
say "  一个空闲但活着的 agent 和一个正在干活的 agent,在这个判据下长得一模一样。"
python3 - > "$CALLS" <<'PY'
import json

instructions = """== 实测到的现象 ==
/tmp/seatloom_waker.log 连续 17 小时每 90 秒打印同一行:
  seatloom-waker: seat-vela-001 in_flight disp-wash-vela-p2 for 63344s (pane running pi) — an agent is working; not poking

陈旧领单检测**本身是对的**:年龄由 Postgres 自己的 NOW() 算出(跨时钟那条教训已经吃进去了),
63344 秒远超 SEATLOOM_STALE_AFTER_SECS 的 900。它卡在第二个判据上:
「pane 里有 agent 在跑」被实现成「有个叫 pi 的进程活着」,
而**一个空闲但活着的 agent 和一个正在干活的 agent,在这个判据下长得一模一样**。
后果:全队停摆 17 小时 —— 洗数据席位后面 4 张 ready 单一张都发不出去,整条夜间链路断在这里。

== 要修什么 ==
给陈旧领单补一条**兜底的叫醒**:领单年龄超过一个更大的阈值时(建议
SEATLOOM_STALE_HARD_AFTER_SECS,缺省 4 × stale_after),**无论 pane 看起来在不在干活都叫醒一次**,
照现有的 poke cooldown 节流。

为什么这样是安全的:叫醒一个已经 in_flight 的席位不会造成损害 ——
AGENTS.md 明确写了「waking an already-in_flight seat is safe」,
而 get_next_dispatch 对一个已持有 in_flight 的席位的回答本来就是「你已经在忙了」。
所以这个兜底的最坏情况是一次多余的叫醒;而不叫醒的最坏情况,是今天这 17 小时。

**不要**去改账本:陈旧的 in_flight 不该被唤醒服务改写成 failed 或 ready
(现有注释里已经写明这条,是对的,保持)。兜底只叫醒,不写行。

== 也请顺手判定 ==
「pane 在干活」这个判据还有没有更诚实的信号(比如子进程累计 CPU 时间在两次扫描之间是否增长,
或 pane 内容哈希是否变化)。如果有,就把它作为主判据、硬阈值兜底作为次判据,
并在交付说明里写明你选的信号为什么比「进程活着」更贴近「真的在推进」。
判断不出来就只上硬阈值兜底,**把结论写出来,不要默默跳过这一问**。

== 完成协议 ==
按你自己 AGENTS.md 的完成协议走。前向记录现在可用了(019 已应用,
open_topic 与 remember 已各实证成功一次,你的工具进程也已换成带 18 个工具的新程序)。"""

msgs = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize",
     "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "aegis-land-knowledge", "version": "1"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
    {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
        "name": "enqueue_dispatch",
        "arguments": {
            "id": "disp-waker-stale-hard-floor",
            "target_seat_id": "seat-nimbus-001",
            "title": "waker: a stale claim must still be poked when the pane merely LOOKS busy (17h fleet stall)",
            "priority": 70,
            "payload": {"instructions": instructions},
        }}},
]
for m in msgs:
    print(json.dumps(m, ensure_ascii=False))
PY
"$BIN" < "$CALLS" 2>&1 | python3 -c "
import sys, json
for raw in sys.stdin:
    raw = raw.strip()
    if not raw: continue
    try: m = json.loads(raw)
    except Exception: print('  stderr:', raw[:200]); continue
    if m.get('id') in (None, 1): continue
    if 'error' in m:
        print('  ❌ 失败:', str(m['error'].get('message'))[:240]); continue
    r = m.get('result') or {}
    txt = ' '.join(c.get('text','') for c in (r.get('content') or []))
    if r.get('isError'):
        low = txt.lower()
        print(('  ⏭  已派过,跳过:' if ('already' in low or 'exists' in low or 'duplicate' in low) else '  ❌ 被拒:'), txt[:240])
    else:
        print('  ✅', (txt or json.dumps(r, ensure_ascii=False))[:240])
"
say ""

# ══ 收尾:核对账本 ═════════════════════════════════════════════════
line
say "核对账本(唯一可信的证据)"
line
printf "  live 词表含 TopicOpened      %s\n" \
  "$(case "$(q "select pg_get_constraintdef(oid) from pg_constraint where conname='canonical_events_event_type_vocab'")" in *TopicOpened*) echo ✅;; *) echo ❌;; esac)"
printf "  topics 行数                  %s\n" "$(q "select count(*) from topics")"
printf "  memories 行数                %s\n" "$(q "select count(*) from memories")"
printf "  门事件 ev-knowledge-write-path-live  %s\n" \
  "$([ "$(q "select count(*) from canonical_events where id='ev-knowledge-write-path-live'")" = "1" ] && echo ✅已入账 || echo '未入账')"
printf "  洗数据席位的卡死领单          %s\n" \
  "$(q "select coalesce(string_agg(id||'('||status||')',' '),'已清空 ✅') from dispatches where target_seat_id='seat-vela-001' and status='in_flight'")"
printf "  洗数据席位待领               %s 张\n" \
  "$(q "select count(*) from dispatches where target_seat_id='seat-vela-001' and status='ready'")"
printf "  唤醒服务修复单                %s\n" \
  "$(q "select coalesce(max(status),'未入账') from dispatches where id='disp-waker-stale-hard-floor'")"
printf "  公告当门用的修复单            %s\n" \
  "$(q "select coalesce(max(status),'未入账') from dispatches where id='disp-gate-vs-announcement-enforcement'")"
printf "  读模型门 ev-knowledge-read-model-landed  %s\n" \
  "$([ "$(q "select count(*) from canonical_events where id='ev-knowledge-read-model-landed'")" = "1" ] && echo ✅已入账 || echo '未入账')"
printf "  那张坏卡                     %s\n" \
  "$(q "select status||'(方案'||case when acceptance_plan='{}'::jsonb then '仍为空 ❌' else '已回填 ✅' end||')' from acceptances where id='acc-disp-me6Nm8e9D6ZRe4fKyVokY'")"
printf "  张先生能点的卡               %s 张\n" \
  "$(q "select count(*) from acceptances where status='pending' and acceptance_plan <> '{}'::jsonb and base <> 'track'")"
say ""
say "全部完成。把上面这段贴给 Aegis。"
