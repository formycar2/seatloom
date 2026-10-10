| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-nimbus-lineage-readmodel-acceptance-plan-v1 |
| title | Acceptance plan — 血缘读模型(cmd_get_acceptance_lineage + MCP get_acceptance_lineage + request_acceptance 计划强制) |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1 |
| tags | acceptance-plan, nimbus, lineage, read-model, plan-enforcement |

# Acceptance plan — 血缘读模型 + 计划强制

**对象**:dispatch `disp-accplan-lane-b-nimbus`(done)· 分支 `packet/nimbus/acceptance-lineage-readmodel`
@ `32a3369`(同步 + 重验后以新 tip 为准)· Flux PASS(同步前 pin da64fcc;同步后重验)。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(血缘速览)

张先生阻塞反馈的「纯展示缺口」半边:从一个验收卡 id 一次拉出上游派单(标题/issuer/时间/
原始 payload)、完整事件链、Flux 判定与证据、产物/交付文档入口、依赖边、返工血缘 ——
零新存储,全部从账本组装。同批交付:request_acceptance 的 acceptance_plan typed 强制
(缺方案不许建卡)。与 D5 recall 同形(对象 + 上下文链,渐进披露),一个 lineage 模块两个组装器。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认「血缘」能力:对着任意一张待验收卡,一次拿到它的前因后果(谁下的单、原始任务要求、谁做的、产物在哪、Flux 怎么判的、依赖什么),并且验收方案缺失时建卡被明确拒绝 —— 卡片信息不足的问题从后端根上解决。",
  "preconditions": [
    "运行中的构建含该命令(track b6751a1 或其后的集成构建;seatloom-waker/MCP 服务器按既有部署)",
    "live 库存在至少一张 acceptance(在候卡 acc-disp-me6Nm8e9D6ZRe4fKyVokY 即可)",
    "MCP get_acceptance_lineage 对调用席位可用(identity 来自 SEATLOOM_ACTOR_REF,不接受参数伪造)"
  ],
  "steps": [
    { "action": "调血缘命令查在候卡", "input": "cmd_get_acceptance_lineage(acceptance_id='acc-disp-me6Nm8e9D6ZRe4fKyVokY')" },
    { "action": "逐块核对返回:dispatch 块", "input": "无(看 title / issued_by / issued_at / payload 原文)" },
    { "action": "逐块核对返回:events 块", "input": "无(看是否按时间序含 Issued→Delivered→ArtifactCreated→Completed→AcceptanceRequested→ReviewVerdictIssued)" },
    { "action": "逐块核对返回:fluxVerdict / products / dependencies / rework 块", "input": "无" },
    { "action": "用 MCP 读工具对同一 id 再查一次", "input": "MCP get_acceptance_lineage(acceptance_id='acc-disp-me6Nm8e9D6ZRe4fKyVokY')" },
    { "action": "演示计划强制负例:不带 acceptance_plan 调 request_acceptance", "input": "dispatch_id=<任意自己的 done dispatch>, branch, base='track/infra-foundation', how_to_verify, flux_verdict_ref, 不传 acceptance_plan" },
    { "action": "演示计划强制负例二:plan 形状不符(如 steps 为空数组)", "input": "acceptance_plan={'plan_version':1,'scenario':'s','steps':[],'expected':[{'output':'o','expect':'e'}]}" }
  ],
  "expected": [
    { "output": "dispatch 块含上游派单全貌", "expect": "标题「D2/Mira: 派单页「待验收」区 UI(接受即合/打回即重派)」、issued_by=seat:seat-lyra-001、issued_at=2026-10-09 12:50、payload 原文(1253 字符,原始任务说明)" },
    { "output": "events 块六类事件齐全且有序", "expect": "每条带 actor + occurred_at + payload;顺序即 dispatch 生命周期" },
    { "output": "fluxVerdict 块指向判定事件", "expect": "flux_verdict_ref=ev-UDrRZZqr8LUwfcgEeFdN5,verdict=pass,evidence/evidence_detail/target_branch 可见" },
    { "output": "dependencies 块显示该卡 deps=0;rework=null", "expect": "机制在(依赖边存在性可查);空态显式,不静默省略" },
    { "output": "MCP 与 Tauri 两条路径返回同一数据", "expect": "共用一个 repo 函数的证据;全程零 psql" },
    { "output": "两个负例都被 typed 拒绝", "expect": "拒绝信息指出所缺/所违规则;canonical_events 计数不变(零写入);带合法 plan 才建得出卡(本卡的存在即正例)" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-nimbus-001)

```
dispatch_id=disp-accplan-lane-b-nimbus
branch=packet/nimbus/acceptance-lineage-readmodel(同步后 tip)
base=track/infra-foundation
how_to_verify=对 acc-disp-me6Nm8e9D6ZRe4fKyVokY 调血缘命令一次返回上游派单/事件链/Flux 判定/产物/依赖边;缺 plan 的 request_acceptance typed 拒绝且零写入
flux_verdict_ref=<同步后 Flux 重验的 ReviewVerdictIssued 事件 id>
acceptance_plan=<§2 JSON 逐字>
```
