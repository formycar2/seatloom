| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-11-lyra-nimbus-open-topic-acceptance-plan-v1 |
| title | Acceptance plan — open_topic:受治理的主题创建工具(记忆链的唯一前置) |
| status | active |
| author | lyra |
| doc_date | 2026-10-11 |
| version | v1 |
| depends_on | AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md (裁决一), VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md |
| tags | acceptance-plan, open-topic, nimbus, d5-phase2 |

# Acceptance plan — open_topic

**对象**:seat-nimbus-001 的 open_topic lane(Aegis 派单,进行中)· 分支
`packet/nimbus/open-topic`。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(为什么它是唯一卡点)

memories.topic_id NOT NULL + FK → topics,而 topics 0 行、全仓没有任何生产路径能建
topic —— 每一次 remember 都会外键失败,前向记录义务与 Vela 的 S5 全被它堵死。
open_topic 把「开启一个主题」变成有意的、受治理的动作(裁决一:不隐式创建、不手工播种)。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认「开启主题」成为受治理的工具动作:幂等、身份服务端解析、四类破坏性调用全部 typed 拒绝且零写入、行与事件同事务落账本 —— 记忆链的唯一前置打通,且作用域(project+topic)不会被拼写错误意外撑大。",
  "preconditions": [
    "open_topic lane 已交付并 Flux 验绿(commit-pinned;判定事件 id 按契约预声明)",
    "调用方的 MCP 进程是重启后的新二进制(自报工具清单含 open_topic);live 库已 apply 015-018"
  ],
  "steps": [
    { "action": "开一个主题", "input": "MCP open_topic(topic_id='acc-ot-demo-parent', title='验收演示-父主题', project_id='seatloom')" },
    { "action": "同 id 再开一次(验幂等)", "input": "MCP open_topic(topic_id='acc-ot-demo-parent', title='随便什么标题')" },
    { "action": "开一个子主题", "input": "MCP open_topic(topic_id='acc-ot-demo-child', title='验收演示-子主题', parent_topic_id='acc-ot-demo-parent')" },
    { "action": "负例:父主题不存在", "input": "MCP open_topic(topic_id='acc-ot-bad-1', title='坏父', parent_topic_id='no-such-topic')" },
    { "action": "负例:自环", "input": "MCP open_topic(topic_id='acc-ot-demo-parent', title='自环', parent_topic_id='acc-ot-demo-parent')" },
    { "action": "负例:跨 project 父子(若实现支持指定 project)", "input": "MCP open_topic(topic_id='acc-ot-bad-2', title='跨项目', project_id='<另一个 project>', parent_topic_id='acc-ot-demo-parent')" },
    { "action": "查账本:主题开启事件在,subject 是 topic,actor 是调用席位", "input": "MCP query_ledger(project_id='seatloom', event_types=['<Nimbus 选定的事件类型>'])" }
  ],
  "expected": [
    { "output": "首次 open_topic 成功,返回 topic 行", "expect": "opened_by = 调用席位的 SEATLOOM_ACTOR_REF(服务端解析,绝非入参)" },
    { "output": "第二次同 id 调用成功且不改既有行", "expect": "幂等(ON CONFLICT DO NOTHING 语义):标题不被打乱,返回现有行" },
    { "output": "子主题创建成功,parent 指向正确", "expect": "同 project;树形可表达" },
    { "output": "三个负例全部 typed 拒绝", "expect": "拒绝信息指出所违规则;canonical_events 计数不变(零写入);不落半行" },
    { "output": "账本里每个成功的 open_topic 恰一条事件", "expect": "行 + 事件同一事务;refs subject 为 ref_type=topic;事件类型若是新增值,Nimbus 交付说明写明了「复用 vs 新增」两路代价与选择理由(裁决要求,不默默选)" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-nimbus-001)

```
dispatch_id=<Nimbus 的 open_topic lane dispatch id>
branch=packet/nimbus/open-topic
base=track/infra-foundation
how_to_verify=open_topic 幂等且 opened_by 服务端解析;坏父/自环/跨项目三类 typed 拒绝零写入;行+事件同事务,subject=topic
flux_verdict_ref=<该 lane 的预声明判定事件 id>
acceptance_plan=<§2 JSON 逐字>
```

## 4. 备注(分支路径,已在派单中写明)

若 Nimbus 选择**新增** `TopicOpened` 事件类型:PG 值域加宽 = Onyx 迁移 + Lyra 契约签署,
Nimbus 必须先上报(Lyra 出契约,双方 packet 逐字携带)再动 schema 侧;工具代码可先落 Rust。
若选择**复用**既有类型:在交付说明里论证为什么那个类型的语义装得下「一个主题被开启」。
