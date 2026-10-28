# Supervisor Layer「意图入口」建设简报

**派单方**:Aegis(seat-aegis-001)
**委托人**:张先生,2026-10-13 原话 —— 「这算是 Seatloom 新功能,需要你让 Seatloom 项目中的各 seat 参与进来建设,分工各司则,协同推进」
**关联派单**:`disp-supervisor-intent-layer-decompose`(产品负责人)、`disp-supervisor-intent-mcp-surface-v2`(后端)、`disp-supervisor-intent-ui-v2`(界面)

---

## 1. 要建的是什么

PRD v0.5 §2 对 Supervisor Layer 的定义,原文:

> 让用户通过**意图**而不是填表来驱动系统,并在需要时**按需索取足够证据**。
> 该给用户的东西:**Command Bar、建议卡、routing rationale、decision-answer 卡片、continuity delta 卡片**。

翻译成人话:**你打一句话说想干什么,系统替你拆成结构化操作;你追问"为什么这么派"时,它给得出理由。**

## 2. 为什么现在建(痛点,查证过的)

| 事实 | 证据 |
|---|---|
| 入口是空壳 | `ui/src/components/SupervisorCommandBar.tsx` 385 行,**真数据调用 0 处** |
| 面板半通 | `ui/src/app-v2/panel/SupervisorPanel.tsx` 575 行,4 处真调用 |
| 启动器极小 | `ui/src/supervisor/SupervisorApp.tsx` 45 行,1 处 |

**后果**:2026-10-13 张先生问「你在 Seatloom 中体现在哪里?」—— 答案是**不在**。拆活、派单、核门、裁决现在靠一个终端里的 agent(我)在跑,不是产品的能力。他今天一直是"拉我一把",而不是"跟 SeatLoom 说一句话"。

## 3. 五个模块的现状(决定优先级)

| 模块 | 状态 |
|---|---|
| 账本 + handoff 闭环 | ✅ 厚:708 事件、99 派单、验收卡、唤醒服务 |
| 确定性 routing/validation/audit | ✅ 厚:依赖边是数据、门事件、Flux 判定 |
| Seat 三层架构 | ✅ 厚:6 席位 + worktree/分支/MCP 身份 |
| Artifact 审阅系统 | ✅ 厚:验收卡 + 血缘 + 结构化方案(已验收通过 1 张) |
| 可复用知识层 | 🟡 刚落地:112 记忆 / 25 主题 / 三层披露 / 行号证据 —— **只有面板能看,没有检索入口** |
| **Supervisor Layer 意图入口** | ❌ **本次要建** |
| Playbook 系统 | ❓ 未查证落地 |

## 4. 三条硬规则(PRD v0.5 定下,至今有效)

1. **任何功能必须同时存在于三层**:一个用户故事 + 一条交互流程 + 一个屏幕/UI surface 定义。**缺一层不许进实现。**
2. **任何功能必须写明**:解决什么痛点 / 不做会怎样 / 上线后用户行为怎么变。写不出来就不该留在范围里。
3. 检查按文件粒度跑,碰一个本来就脏的文件会让它变红 —— **交付"零新增问题"的实测结果,绝不顺手重排版**。

## 5. 分工(我派的,附理由)

| 派单 | 席位 | 前置 |
|---|---|---|
| `disp-supervisor-intent-layer-decompose` | 产品负责人 | 无(第一张) |
| `disp-supervisor-intent-mcp-surface-v2` | 后端/架构 | 事件 `ev-supervisor-intent-contract-landed` |
| `disp-supervisor-intent-ui-v2` | 界面 | 同上 |

**后端和界面的单已经挂着依赖边,现在是 blocked 状态** —— 产品契约不落地,它们领不动。这是故意的,不是故障。

### 产品负责人要交付的

1. 用户故事:谁、想干什么、现在怎么被卡住
2. 交互流程:从输入意图到看到结果,一步一步,**含失败和歧义路径**
3. 屏幕定义:Command Bar 放哪、建议卡长什么样、decision-answer 卡片给什么、continuity delta 是什么
4. 三个必答:解决什么痛点 / 不做会怎样 / 上线后用户行为怎么变
5. 拆成可派子单,每张写清目标席位、前置依赖边(**挂事件 id,不要用散文描述前置**)、验收怎么判

**完成后必须发出事件,id 固定为 `ev-supervisor-intent-contract-landed`** ——
后端和界面两张单的依赖边指的是这个 id,不是你的派单 id。用 `emit_event` 发,
类型建议 `ArtifactCreated`,payload 里带 `contract` 字段说明三层契约落在哪个文件。

## 6. 边界(我划的,可反对但要说明理由)

- **不重做现有界面骨架** —— 张先生明确要求过
- **「意图说不清时系统不许猜」必须写进交互流程并在界面体现** ——
  我们整个项目的病就是猜:伪造血缘、自己编方案、把交付公告当成门事件。
  入口层再犯一次,前面所有的纪律都白付
- **接线状态要么真测,要么不显示** ——
  知识面板那次我在 UI 里留了一个硬编码开关说"命令未上线",而命令其实在。
  那句假话让张先生对着屏幕看了半天。不要在别处重复这个形状
- **交付说明要能转成张先生能读懂的三段**:变了什么 / 你会看到什么 / 需要你做什么。
  不要用自造缩写,不要打印派单标题给他看

## 7. 已有的地基(后端和界面可以直接用,不用从零造)

- **账本与派单机制完全工作**:`enqueue_dispatch` / `get_next_dispatch` / `complete_dispatch` /
  `query_ledger`;依赖边是数据(`dispatch_dependencies` 表),派单 ready iff 每个 referent 存在
- **召回层刚落地**:`recall` 带 topic 祖先链、按需取 L1/L2,全程零 psql;112 条记忆、25 个主题
- **门事件**:`emit_event` + `SupervisorMessage`,条件是「live 可用」不是「代码合并」
- **验收卡系统**:带血缘 + 结构化方案,张先生已经成功点过一次

## 8. 我怎么协同

- 我盯依赖边:契约事件一落地,两张 blocked 单会自动转 ready,唤醒服务会去叫对应席位
- 我核门:任何"能力可用"的断言,按「能不能真的跑成功」核,不按「代码合没合进来」核
- 我给张先生的汇报只有三段,并且只讲他需要决策/授权的事
