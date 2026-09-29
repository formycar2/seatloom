# SeatLoom Product Design Insight — Supervisor IM as L1 Surface

| Field | Value |
|---|---|
| doc | 2026-04-30-supervisor-im-as-l1-insight |
| scope | Analyze why Supervisor IM / routing governance is the highest-frequency user surface and how this shifts product design priorities |
| basis | User feedback on 2026-04-30, v2 implementation reality, prd-v0.5.md, architecture-design.md, data-coverage-audit |
| status | issued |
| author | aegis |
| date | 2026-04-30 |

---

## The Misunderstanding

My value review (2026-04-30-seatloom-value-review.md) structured the product in **backend capability layers**
(Data Engine · Seat · Playbook · Supervisor · Artifact Review), and measured progress by how many module
surfaces had any UI at all. This produced the table:

```
Inbox / WorkItem / Artifact / Session / Handoff → ❌ not yet surfaced
Supervisor IM → ⚠️ partially covered
```

That framing was **wrong** because it treats all "P0 modules" as equal-value surfaces. They
are not. The user explicitly corrected this:

> "Supervisor IM（routing governance）已经成为事实的 level1 重要的模块，使用频率角度远远
> 大于 v1 和 v2 规划的 InboxItem / WorkItem 状态机, Artifact (T1-T7)...

这指明了 **L1 与 L2 的真正区分依据**：
**L1 是用户实际操作的频率最高的入口，不是架构上 P0 的功能。** 而 L1 只需要提供"那时那刻最重要的信息"，
不是全部信息。

---

## Why Supervisor IM is the True L1 Surface

### What Aegis/Supervisor actually does in practice

从最近 72 小时的合作模式可以看出真实的 Aegis 工作流：

```
Aegis 的日常工作流：
  ① 查看每个角色是否有阻塞 → 在频道/DAG 上直接看到
  ② 批准/拒绝 Gate 决策 → 在消息气泡上直接看到和操作 (GO/HOLD)
  ③ 路由消息到合适的 PO/Worker → routing guard 机制
  ④ 签发任务包 via 项目频道 → 直接发消息给 Lyra/Nimbus/Mira/Flux
  ⑤ 查看当前状态 → 打开频道 DAG 即可看到全局状态
  ⑥ 审查代码/输出 → 直接在频道中看到 review 卡片
```

这一动作系列的共同特征：
- **高频**：每天打开 20+ 次，每次会话都反复使用
- **决策导向**：每项操作都是一个决策（批准、路由、查看、修正）
- **context-sensitive**：不需要手动去各个视图拼凑状态，信息在 Supervisor 中汇聚
- **直接交互**：点击 → 操作 → 完成，无需跳转、无需翻页

### Supervisor IM 解决的核心痛点

| 目前协作模式 | Supervisor IM 的价值 |
|---|---|
| 每个 Seat 的消息在各自终端中 | 所有对话统一在一个浮层中，不分 Seat |
| 总要在不同终端/文件间切换 | IM 内的 channel 就涵盖了 seat conversation + project dashboard |
| 路由决策要人工思考 | routing guard 自动做决策提示 |
| Gate 决策要看多个 PR/review | IM 消息气泡直接展示 Gate 状态和 actions |
| 状态更新要手动追踪 | Channel dashboard + DAG 实时展示 |

Supervisor IM 的本质是 **AI Native 的核心交互模式**：它把决策、信息、操作合并为一个连续的 IM 会话，
让 Aegis 作为 Orchestrator（总指挥）在一个统一的视野中工作，而不是像传统管理工具那样
切换视图来查找信息。

**这就是"Auto Pilot 不需要驾驶员关注所有，只要关注那时那刻最重要的"的具体实现：**
- Supervisor 通过 routing 机制自动做 Gate/Handoff 决策，人只在最需要注意的阻断处介入
- 信息被 push 到 IM，不需要用户 pull 去 Inbox/WorkItems/Artifacts 视图逐一扫描
- Supervisor IM 的每个卡片都是"即得即决策"的瞬态接口，不需要完整视图

---

## Why Traditional Data Views are L2 (not L1)

传统的"数据视图"（Inbox/WorkItem/Artifact/Session/Handoff）仍是必要的，但它们的角色是：

1. **详细审查/背景研究的来源**，不是高频操作入口
2. **结构化存储层**的 UI 面，提供完整视图以便做决定时参考完整信息
3. **传统 CRUD 的搜索和过滤需求**——在 IM 上下文之外需要 进行大规模过滤或批量操作时使用
4. **合规/审计/追溯的入口**，不是日常操作路径

这些 ALL 有价值，但使用频率低得多：
| 视图 | 预期日使用次数 | 典型使用场景 |
|---|---|---|
| Supervisor IM | 30-50 次 | 所有 Seat 的消息、gate decision、routing |
| Inbox | 5-10 次 | 批量清理待决项、处理积压 |
| WorkItems | 3-8 次 | 复杂 WorkItem 详情查看、状态变更 |
| Artifacts | 2-6 次 | 找特定文档、review 证据链 |
| Sessions | 1-5 次 | 连接/断开、Setup session |
| Handoff | 1-3 次 | 发送/接收委派时 |

L1 日使用 30-50 次 vs L2 日使用 1-10 次，L1 使用频率是 L2 的 8-30 倍。

**AI Native 产品的核心设计原则**：
- L1 = 最高频、最短路径、最少的步骤
  → Supervisor IM 担负 L1，大量使用 = 立即响应 + 交互流畅
  → L1 不需要强的状态自动追踪或高信息密度，但要高速度
- L2 = 功能完备、操作完整但频率较低
  → Inbox/WorkItems/Artifacts/Sessions/Handoffs 在 L2，需要完整的 CRUD/检索/过滤/深度展示，但不需要高频刷新
- L3 = 系统级操作/批量处理
  → Budget/Governance/Pipeline/Ledger 在 L3，按需访问

---

## How This Changes Product Priorities

### Incorrect (before correction)
```
Priority: Inbox → WorkItem → Artifact → Session → Handoff → Supervisor IM
```
把 Supervisor IM 视作 P1 模块（"feasible to deliver later"），
优先建设那些"结构性"模块。

### Correct (design insight)
```
Priority: Supervisor IM (L1, core) → Inbox → WorkItem → Artifact → Session → Handoff
```
Supervisor IM 是 L1 产品中心，必须作为第一优先级完善。

**这不是说其他模块不重要，而是它们的优先级是基于使用频率和用户价值密度，而不是基于架构深度。**

### Priority 1: Supervisor IM 深度

| 优先级 | 课程 | 工作 |
|---|---|---|
| P0-L1-① | **routing flow** | Supervisor IM 的 routing + gate 机制必须最稳定 |
| P0-L1-② | **阻塞项展示** | BlockersSection + DAG blocking nodes + ActiveWork Section |
| P0-L1-③ | **高性能 push** | "最重要" 的信息在 IM 里即时推送 |
| P0-L1-④ | **query 完成度** | ⌘K 频道切换、快速查询、预览、行动 |
| P0-L1-⑤ | **移动端** | Mobile Companion 在 L1 |

完成这些意味着 80% 的使用场景已经解决。

### Priority 2: L2 数据视图（不是完全不优先，但优先级在 L1 后）
```
Inbox → WorkItem → Artifact → Session → Handoff → Timeline 深度
```
这些视图不需 L1 的"即时可用"，更强调功能完整性和耐久性，
但不需要调度 L1 的资源。

---

## Why the Insight is Hard to See

1. **架构思维 vs. 产品思维**：架构上的 P0 模块（Data Engine / Seat / Playbook）承诺的完整度很高，
   但这些是后端能力，不直接等于 UI 表面。

2. **隐含需求未被显式表达**：Supervisor IM 不是 PRD 里的"几号 Screen"，它是用户实际工作中
   无意识地反复使用的实践，很难用文档或截图来衡量。

3. **L1 vs L2 的误判**：如果按照 P0/P1/P2 判断 "所有 P0 都同等重要"D 仍然是一个频次和速度的决定因素，
   这里的区分是 P0 操作频次 vs P0 功能完整性。

---

## Conclusion: L1 Principle for AI-Native Products

**L1 设计原则**：最高价值 UI = 最高使用频率 + 最短操作路径 + 最少的决策步骤

```
SeatLoom L1 核心交互循环：
  [Aegis 打开 Supervisor IM] → [查看当前最重要的 Blockers/Gates/Messages]
  → [在 IM 内直接路由/批准/纠正]
  → [动作完成，无需离开 IM]
  → [回到下一个决策]

停留时间：2-5 分钟
操作步骤：0-3 步
信息密度：高（最有价值的，不是全部）

对比传统工作流（Inbox → WorkItems → Artifacts → Handoffs → Sessions）:
停留时间：5-30 分钟
操作步骤：5-15 步
信息密度：全量（有价值 + 无价值）
```

**建议**：所有 v2 改进和后续版本决策，首先问：
> "这会缩短 Aegis 在 Supervisor IM 里的操作步骤，还是会提供更多 IM 外的详情？

如果答案是"它能让 IM 内操作更顺畅、更少步骤"，它就是 L1，应该在最高优先级。

如果答案是"它让其他视图更全更强大"，它是 L2，可以在 L1 稳定之后渐进交付。

**SeatLoom 的突破性价值不是"有一个 Artifact 数据库"，而是"Supervisor IM 把最高价值的操作压缩到最少的步骤和最高频的交互中。"</p>