| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-nimbus-d5-recall-acceptance-plan-v1 |
| title | Acceptance plan — D5 recall 记忆工具(remember/recall/promote/forget + get_dispatch/query_ledger) |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | VALUE-DOMAIN-2026-10-10-knowledge-layer-v1 |
| tags | acceptance-plan, nimbus, d5, recall, memory-tools, read-tools |

# Acceptance plan — D5 recall 记忆工具

**对象**:dispatch `disp-d5-lane-b-nimbus`(done)· 分支 `packet/nimbus/d5-mcp-recall`
@ `db7e1f3`(同步 + 重验后以新 tip 为准)· Flux PASS(同步前 pin 13eae88;同步后重验)。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(血缘速览)

seatloom MCP 从 6 个工具扩到 14 个:recall(渐进披露,默认 l0,带 topic 祖先链,每次 recall
恰一条 MemoryRecalled 留痕)、remember(建档 + MemoryCreated 同事务)、promote(唯一 tier
通道,可升可降)、forget(软墓碑)、get_dispatch / query_ledger(纯读,任何席位不必再 psql)。
记忆管理是显式工具,无后台自动黑箱。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认「管 knowledge」的召回能力:显式记住/召回/晋升/遗忘,召回按层渐进披露且留痕账本,派单与账本可纯读查询 —— 从「管 work」到「管 knowledge」在工具面成立,且任何席位不再需要 psql。",
  "preconditions": [
    "MCP 服务器(含 14 工具)对调用席位可用;live 或 scratch 库已应用 015/016",
    "作用域内至少存在一个 topic(topics 表当前 live 0 行;topic 的规模 populated 属 Phase 2 洗数据(Aegis 车道)—— 这是已上报的接缝:14 个工具里没有 topic 创建工具,proof 级 bootstrap 目前经 psql。本方案的 remember/recall 步骤以此为前提;在首个 topic 存在前,remember 会因外键被拒,不会写半截)",
    "已知 D5 契约:heat 只随 recall +1,无自动衰减;promote 是唯一 tier 通道;forget 是软墓碑"
  ],
  "steps": [
    { "action": "纯读先行:查一个已知派单", "input": "MCP get_dispatch(dispatch_id='disp-d5-lane-a-onyx')" },
    { "action": "纯读:按类型过滤查账本", "input": "MCP query_ledger(project_id='seatloom', event_types=['MemoryCreated'], limit=5)" },
    { "action": "记住一条记忆", "input": "MCP remember(project_id='seatloom', topic_id='<一个已存在的 topic>', title='gate works', l0_summary='accept needs flux green plus the operator', source_kind='manual', source_ref='lyra-plan-demo', tags=['d5'])" },
    { "action": "召回,默认深度", "input": "MCP recall(project_id='seatloom', topic_id='<同一 topic>')" },
    { "action": "召回,显式深一层", "input": "MCP recall(project_id='seatloom', topic_id='<同一 topic>', depth='l1')" },
    { "action": "晋升,再演示降级", "input": "MCP promote(memory_id='<上一步返回的 id>', to_tier='warm');再 to_tier='cold'" },
    { "action": "同 tier 晋升(负例)", "input": "MCP promote(memory_id='<同上>', to_tier='cold')(当前已是 cold)" },
    { "action": "遗忘", "input": "MCP forget(memory_id='<同上>', reason='plan demo done')" },
    { "action": "遗忘后再召回(负例)", "input": "MCP recall(project_id='seatloom', topic_id='<同一 topic>')" },
    { "action": "再遗忘一次(负例)", "input": "MCP forget(memory_id='<同上>', reason='again')" }
  ],
  "expected": [
    { "output": "get_dispatch 返回该派单行 + live 状态 + 依赖边(含 referent 存在性)", "expect": "与 query_ledger/UI 所见一致;全程零 psql" },
    { "output": "query_ledger 按过滤返回事件 + 其 object refs", "expect": "类型过滤生效;未知事件类型被 typed 拒绝" },
    { "output": "remember 成功:记忆建档(tier=cold,status=active,heat=0)", "expect": "账本恰多一条 MemoryCreated,subject ref 为 ref_type=memory;tier/status 调用方不可指定" },
    { "output": "recall 默认深度只回 l0 层字段 + topic 祖先链", "expect": "返回字段仅 id/title/l0_summary/tier/heat/recall_count(+链);账本恰多一条 MemoryRecalled;该行 heat=1 recall_count=1 last_recalled_at 非空" },
    { "output": "depth='l1' 才出现 l1_overview", "expect": "渐进披露:未请求的层不返回(l2 同理,显式 depth='l2' 才出 l2_* 指针)" },
    { "output": "promote warm 成功;再 promote cold(降级)成功", "expect": "每次变迁恰一条 MemoryPromoted,payload 带 from_tier/to_tier;升降皆可 —— 唯一 tier 通道" },
    { "output": "同 tier promote 被 typed 拒绝", "expect": "无变迁即拒绝;账本计数不变" },
    { "output": "forget 成功:status=forgotten,行仍在", "expect": "MemoryForgotten 带 reason;软墓碑,绝不硬删" },
    { "output": "遗忘后 recall 不再返回该记忆", "expect": "0 命中 0 事件(召回留痕按命中行进)" },
    { "output": "第二次 forget 被 typed 拒绝", "expect": "墓碑不可改;账本计数不变" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-nimbus-001)

```
dispatch_id=disp-d5-lane-b-nimbus
branch=packet/nimbus/d5-mcp-recall(同步后 tip)
base=track/infra-foundation
how_to_verify=remember→recall(l0 只回摘要+topic 链+恰一条 MemoryRecalled)→l1/l2 按需→promote 升降→forget 软墓碑;get_dispatch/query_ledger 纯读可代 psql
flux_verdict_ref=<同步后 Flux 重验的 ReviewVerdictIssued 事件 id>
acceptance_plan=<§2 JSON 逐字>
```

## 4. 已上报接缝(不阻塞本卡,阻塞「照着做」的完备性)

topics 表现 0 行且无 topic 创建工具:本方案 precondition 已如实写明;topic 规模 populated
归 Phase 2(Aegis)。建议:Phase 2 含 topic 引导,或未来 P1 开 open_topic(需新事件类型,
属另一次契约加宽)。已 emit SupervisorMessage 上报 Aegis。
