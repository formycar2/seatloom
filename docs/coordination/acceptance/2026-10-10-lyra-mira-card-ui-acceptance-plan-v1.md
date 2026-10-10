| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-10-lyra-mira-card-ui-acceptance-plan-v1 |
| title | Acceptance plan — 验收卡改版(血缘 + 结构化验收方案,L0/L1/L2 渐进披露) |
| status | active |
| author | lyra |
| doc_date | 2026-10-10 |
| version | v1 |
| depends_on | VALUE-DOMAIN-2026-10-10-acceptance-lineage-v1 |
| tags | acceptance-plan, mira, acceptance-card, lineage-ui, progressive-disclosure |

# Acceptance plan — 验收卡改版

**对象**:dispatch `disp-accplan-lane-a-mira`(done)· 分支 `packet/mira/acceptance-lineage-ui`
@ `14e2fd6`(含 track b6751a1)· Flux PASS(commit-pinned 14e2fd6/f7ad270)。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(血缘速览)

张先生阻塞反馈的直接回应:卡片从「分支名 + 一行 how_to_verify」改版为按他的信息序呈现
(哪个里程碑/lane/任务 → 谁下的单何时 → 原始任务要求 → 谁做的 → 产物/交付说明在哪 →
Flux 判定与证据 → 结构化验收方案 → 接受/打回),默认 L0「够判断」、展开 L1 全量、L2 ids/links。
与 D5 的 L0/L1/L2 同一渐进披露思路,不造第四套。无 mock 顶替(命令未注册显式渲染事实)。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认待验收卡现在「够判断」:打开 SeatLoom 待验收区,不翻代码、不问人,仅凭卡片默认呈现就能知道每张卡是什么交付、谁验过、照着怎么验;要看前因后果时展开即有。",
  "preconditions": [
    "运行中的构建含改版后 UI(track b6751a1 或其后的集成构建;派单页/待验收区可见)",
    "live 库有 pending 卡(在候卡 acc-disp-me6Nm8e9D6ZRe4fKyVokY;后续各 lane 建卡后更多)",
    "血缘命令已注册(ACCEPTANCE_CMDS_REGISTERED 及 lineage 对应标志为真);若未注册,卡片应显式说明而非显示假数据"
  ],
  "steps": [
    { "action": "打开 SeatLoom 应用,进入派单页,找到「待验收」区", "input": "项目 seatloom(默认)" },
    { "action": "读一张卡的默认(L0)呈现", "input": "无(看:里程碑/lane+任务标题、下单人+时间、Flux 验绿徽章、验收方案场景+步骤数+首条 expected、接受/打回)" },
    { "action": "展开同一张卡(L1)", "input": "点击展开" },
    { "action": "核对 L1 内容", "input": "无(看:派单 payload 原文、执行席位、产物/交付文档入口、完整事件链、完整验收方案 steps/expected)" },
    { "action": "查看 L2 详情行", "input": "无(看:branch、commit SHA、evidence 路径、dispatch/acceptance id)" },
    { "action": "对照结构化验收方案走一遍(以本卡所带 plan 的 scenario/steps 为准)", "input": "按卡上 scenario 与 steps 操作" },
    { "action": "(闭环)对一张卡点「打回」并填一句反馈,或对已同步分支的点「接受」", "input": "反馈一句,如「信息序正确,细节待对齐」" }
  ],
  "expected": [
    { "output": "待验收区列出 pending 卡,每张卡 L0 即「够判断」", "expect": "不点开即可回答:这是什么交付(里程碑/lane/任务)、谁下的单何时、Flux 验过没有、照着怎么验(场景+步骤数+首条预期)" },
    { "output": "L0 不含 payload 全文与完整事件链", "expect": "渐进披露:默认量收敛,细节不淹没判断" },
    { "output": "展开后 L1 含原始任务要求(payload 原文)、谁做的、产物/交付说明入口、完整事件链(actor+时间+类型)、完整验收方案", "expect": "「前因后果」一屏可溯;空态显式「无」(如无依赖边),不静默省略" },
    { "output": "L2 给出 branch/commit SHA/evidence 路径/各 id", "expect": "可复制、可跳转,便于深究" },
    { "output": "结构化方案可照着走", "expect": "scenario/steps(action+input)/expected(output+expect)/preconditions 渲染清晰,步骤数与 L0 徽标一致" },
    { "output": "接受/打回既有行为不回退", "expect": "接受=--no-ff 合并+sha 展示;打回=必填反馈+返工单链接(priority+10);冲突原文展示不 force;D2 既有测试全绿" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-mira-001)

```
dispatch_id=disp-accplan-lane-a-mira
branch=packet/mira/acceptance-lineage-ui
base=track/infra-foundation
how_to_verify=待验收卡默认呈现里程碑/lane+任务、下单人+时间、验绿徽章、方案场景+步骤数+首条预期;展开含原始 payload/事件链/产物入口/完整方案 —— 不翻代码即可判断价值
flux_verdict_ref=<Flux 对 14e2fd6 的 ReviewVerdictIssued 事件 id>
acceptance_plan=<§2 JSON 逐字>
```
