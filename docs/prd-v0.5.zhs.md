# SeatLoom PRD v0.5

| 字段 | 值 |
|---|---|
| 产品 | SeatLoom |
| 文档 | PRD v0.5 |
| 状态 | 草案 - 待 Lyra、Aegis 与 Mr. Zhang 评审 |
| 更新日期 | 2026-04-28 |
| 语言 | 简体中文（ZHS） |
| 规范入口 | `docs/PRODUCT_TRUTH.md` |
| 上一版契约 | `docs/archive/product-history/prd-v0.4.md` |
| 配套规格 | `docs/interaction-spec-v1.1.md`, `docs/ux-spec-v1.1.md`, `docs/acceptance-spec-v1.1.md` |
| 支持文档 | `docs/architecture-decisions.md`, `docs/architecture-design.md` |

## 0. 为什么需要 v0.5

这一版修订将 4 月 28 日关于核心价值的深度分析转化为可执行的产品契约，并在此基础上纳入另外两个核心设计输入：

- `docs/coordination/reviews/2026-04-28-playbook-vs-skills-and-ecosystem.md`
- `docs/coordination/reviews/2026-04-28-context-and-retrieval-design.md`

本轮的关键规则是严格的：

> 任何功能在进入 implementation 前，必须同时存在于以下三个层面：
> 1. 一个用户故事，
> 2. 一个交互流程，
> 3. 一个屏幕或 UI surface 定义。

第二条规则现在同样严格：

> 如果 PRD 没有明确写出某项功能解决什么痛点、不做会怎样、以及上线后用户行为会发生什么变化，那么该功能不得继续保留在交付范围内。

v0.5 围绕以下五个模块扩展了产品契约，它们现在共同构成产品核心：

1. Data Engine（数据引擎）
2. Seat three-layer architecture（Seat 三层架构）
3. Playbook system（Playbook 系统）
4. Supervisor Layer（监督层）
5. Artifact review system（Artifact 审阅系统）

目标不是增加抽象能力描述。目标是明确：用户会看到什么、用户能做什么、产品在 P0/P1/P2 中如何表现，以及为什么每个功能值得建设。

## 1. 产品论点

SeatLoom 是一个面向 human-plus-agent 项目协作的本地优先连续性系统。

它的核心价值不是“更多 AI agents”。它的核心价值是在成本可控前提下获得更高产出，方式包括：

- 在不需要 LLM 推理的地方，用确定性的 routing、validation、retrieval 与 audit 来完成工作，
- 内置一个 supervisor，将用户意图转化为结构化操作，
- 提供可复用的 seat、skill 与 playbook 系统，让团队不必为相同上下文重复付费，
- 提供可审计的 artifact 审阅与 handoff 闭环，使工作可恢复、可治理，
- 通过硬性 budget 与隔离控制，让 token ROI 由产品行为来强制执行，而不是依赖团队自觉。

## 2. 产品架构

SeatLoom 由五个互相连接的产品模块组成：

| 模块 | 目的 | 用户可见的主要界面 |
|---|---|---|
| Data Engine（数据引擎） | 以确定性方式在不同 runtime 间组装、隔离、预算、验证、召回并审计结构化项目数据 | Inbox 摘要、Timeline 标签、context-tier pack 预览、budget 仪表、evidence 搜索结果、gate 原因、audit 链接 |
| Seat three-layer architecture（Seat 三层架构） | 将持久的 seat 身份与项目角色、协作模式分离，同时让 seat capability truth 可见 | Seat Registry、Seat Card 面板、Project Role Drawer、Collaboration Template 面板、Execution Template 检视器、delegation 徽标 |
| Playbook system（Playbook 系统） | 将“学习得到的复用经验”与“编写好的 seat 指导”分开，并在合适时机渐进注入两者 | LaunchPack 中的 playbook 槽位、seat-skill 槽位、Playbook Library、publish 流程、节省计数器 |
| Supervisor Layer（监督层） | 让用户通过意图而不是填表来驱动系统，并在需要时按需索取足够证据 | Command Bar、建议卡、routing rationale、decision-answer 卡片、continuity delta 卡片 |
| Artifact review system（Artifact 审阅系统） | 让团队能够在 SeatLoom 内部消费、搜索、审阅、标注并关闭 artifacts | Markdown 阅读器、review rail、artifact 内搜索、评论、resolve/dispute 控件、版本列表 |

这些模块不是相互独立的附加项。它们就是产品核心。只要任一模块尚未在 stories、interactions 与 UX 定义中出现，implementation 就不得开始。

## 3. 优先级边界

| 优先级 | 用户承诺 | 纳入的产品范围 | 暂不纳入 |
|---|---|---|---|
| P0 | 用户可以开始一天的工作、创建并路由任务、查看 seat capability truth、切换 runtime、恢复 continuity、审阅输出并找到精确证据，而无需重新解释项目。 | Data Engine 核心、Supervisor command bar、seat identity 与 project role 绑定、collaboration template 选择、scoped delegation overlay、Seat Card 可见性、seat-skill 槽位可见性、context tiers 0-2、structured checkpoints、LaunchPack 预览、Layer 1 structured retrieval、Layer 2 full-text retrieval、budget enforcement、gate 可见性、reissue 流程、应用内 artifact review、手动评论。 | Autonomous child-work creation、semantic recall、用户可编辑的 execution-template forking、cloud-first orchestration。 |
| P1 | 用户可以复用团队已经学到的内容，以更低上下文成本恢复工作，并用更少轮询来查看执行进度。 | Playbook publish/apply 流程、collaboration template 克隆、supervisor-assisted artifact comments、更丰富的 supervisor continuity packs、semantic retrieval layer、带 delta context 的 session suspend/resume、Handoff `working` 状态与可选 live activity mode、Execution Template inspector、版本历史与 diff 审阅、token-savings 可见性。 | Full autonomy、自我变更的治理、始终在线的后台编排。 |
| P2 | 用户可以允许受限自动化提案，并比较哪些模式真正节省时间或 token，同时不丢失人工审批与可审计性。 | 由 seats 起草的 handoffs/artifacts/work proposals、template recommendations、由 review 推导的 action suggestions、高级 ROI dashboards、execution-behavior 版本历史对比。 | 无边界的 agent autonomy、静默状态变更、把 cloud-first coordination 作为必需前提。 |

## 4. 进入 implementation 的契约规则

一项功能只有在以下五个条件都满足时，才算 implementation-ready：

1. 它有一个带可见结果的用户故事，
2. 它有一个触发-成功-失败的交互定义，
3. 它有一个可见该行为的屏幕或 UI 容器，
4. 它有明确的优先级归属与非目标说明，
5. 它通过了下面的功能价值测试。

### 功能价值测试

本 PRD 中列出的每个功能都必须以书面形式回答以下三个问题：

1. 这个功能解决什么用户痛点？
2. 如果我们不做它，会发生什么？
3. 它上线后，用户行为会如何变化？

如果团队无法清楚回答这些问题，该功能就从交付范围中移除，不得进入 implementation。这对 P0、P1、P2 都是硬门槛。P1 和 P2 不允许继续以含糊的“能力桶”形式存在。

## 5. 按优先级划分的故事地图

| Story ID | 优先级 | 模块 | 用户结果 | 解决的痛点 | 如果省略 | 行为变化 | 交互引用 | UX 引用 |
|---|---|---|---|---|---|---|---|---|
| US-P0-01 | P0 | Supervisor, Data Engine | 我打开 SeatLoom 后，立刻就能看到什么需要行动、为什么重要，以及 SeatLoom 推荐我下一步做什么。 | 每天开始时的监督工作很慢，因为状态分散在聊天、终端和记忆里。 | 用户将继续手动轮询各个 seats，错过 blockers，并花时间重建紧急程度。 | 用户从 Overview 和 Inbox 开始，信任排序后的 next actions，只在指导表明有必要时介入。 | `INT-01`, `INT-04` | `UX-01`, `UX-03` |
| US-P0-02 | P0 | Supervisor, Seat, Data Engine | 我用自然语言告诉 SeatLoom 创建并路由一个任务，审阅结构化提案，并在无需填写空白表单的情况下确认它。 | 把意图转成干净的 task packet 很重复，容易描述不足，并且在聊天来回中成本很高。 | 任务创建会继续在产品外进行，结构逐渐漂移，seat 执行从不完整 brief 开始。 | 用户只表达一次意图，审阅一个提案，只修改例外项，并在产品内确认结构化路由。 | `INT-02` | `UX-02`, `UX-05` |
| US-P0-03 | P0 | Seat | 我在不同项目中使用同一个 seat identity，但每个项目都会显示正确的角色、authority docs 和当前协作模式。 | 如果每个项目都要重新创建或重新解释 seats，团队就会失去连续性。 | Capability truth 会漂移，setup 会重复，assignment 会在看不见 authority 边界的情况下发生。 | 用户复用已知 seats，检查项目级 role bindings，并在明确 authority 上下文的前提下分派工作。 | `INT-03` | `UX-04` |
| US-P0-04 | P0 | Seat, Supervisor | 我可以把一小段有边界的工作临时从一个 seat 委托给另一个 seat，并在 UI 和 timeline 中清晰看到这次委托。 | 临时 seat 替代很常见，但代理执行通常被隐藏，或者会重写所有权历史。 | Acting-seat 的工作会保持模糊，audit trail 会断裂，review 反馈会发给错误的 owner。 | 用户发起一个有范围限制的 delegation overlay，清晰地跟踪它，并在不改变原始 seat identity 的前提下关闭它。 | `INT-03`, `INT-04` | `UX-04`, `UX-05`, `UX-11` |
| US-P0-05 | P0 | Data Engine, Supervisor | 当一次交付审阅失败时，SeatLoom 会显示失败原因、关联证据以及 reissue 路径，而不会逼我手动重建上下文。 | 审阅失败目前会产生模糊的返工循环，并迫使人工重新陈述上下文。 | 返工会变慢、带有责怪氛围且不一致，因为失败原因没有绑定到证据上。 | 用户在一个与证据关联的审阅状态中直接 reject 或 reissue，而不是从零重新写一个纠正性 brief。 | `INT-05` | `UX-05`, `UX-07` |
| US-P0-06 | P0 | Data Engine | 当我切换 runtime 或恢复一个被中断的 session 时，SeatLoom 会生成正确的 continuity pack，向我展示其中内容，并给出确定性的 fallback。 | Runtime 变化和中断目前会迫使昂贵的手动 rehydration。 | 用户会重新解释项目，失去连续性，恢复质量依赖记忆而非系统事实。 | 用户通过可见的 pack previews 和 fallback 步骤重新 launch，而不是在终端里手动重建上下文。 | `INT-06` | `UX-06` |
| US-P0-07 | P0 | Artifact review | 我可以在应用内打开完整 artifact，对某一段落发表评论，并把这条评论重新路由回工作闭环。 | 当 artifacts 活在外部工具里、反馈又被手工复制回来时，审阅上下文会丢失。 | 评论会变得不可追踪，审批难以审计，迭代闭环会泄漏到 SeatLoom 之外。 | 用户在 SeatLoom 内阅读、评论、resolve，并把 artifact 反馈作为正常工作闭环的一部分重新路由。 | `INT-07` | `UX-07` |
| US-P0-08 | P0 | Seat, Data Engine, Supervisor | 在我分派或 launch 工作之前，我可以查看一个 seat 的能力、可接受输入、约束和 budget，从而依据 capability truth 而不是角色传说来路由。 | 单纯的角色标签过于粗糙，因此工作经常被错配，seats 也被过度信任。 | 用户会继续凭记忆分配工作，seat mismatch 将保持不可见，token budget 事后才被打破。 | 用户查看 Seat Card，看到某个 seat 为什么符合条件，并在明确约束和 budgets 的前提下确认路由。 | `INT-02`, `INT-03` | `UX-02`, `UX-04`, `UX-05` |
| US-P0-09 | P0 | Data Engine, Playbook | 当 SeatLoom 构建 continuity 时，我可以在 launch 前看到 context tiers、匹配到的 seat skills、匹配到的 playbooks，以及 budget 消耗。 | 整包注入成本高且不透明，因此用户无法信任或调节上下文成本。 | Runtime 切换会持续浪费 tokens，而 continuity 质量仍然不可预测。 | 用户基于可见的分层预览来 launch、裁剪或扩展 continuity，而不是接受一个黑箱 pack。 | `INT-06` | `UX-06` |
| US-P0-10 | P0 | Data Engine, Artifact review, Supervisor | 我可以搜索精确的项目证据，而不必让模型重新通读整个项目历史。 | 当历史只能以聊天记录或原始文件形式阅读时，查找已知文件、blocker 或 checkpoint 会很慢。 | 人类和 Supervisor 都会浪费时间和 tokens 去扫描历史以寻找精确引用。 | 用户优先搜索结构化对象和全文证据，然后直接打开相关源。 | `INT-13` | `UX-07`, `UX-10` |
| US-P1-01 | P1 | Playbook, Data Engine | 当我一次性解决了某个 blocker 后，我可以把这个经验发布为 playbook，并在下次相关上下文中自动收到推荐。 | 相同的 runtime 与 workflow 问题会被重复解决，并反复消耗新的 token 成本。 | 团队将持续为重新发现已知修复方案而付费，无法复利积累操作经验。 | 用户把验证过的修复发布一次，并期待未来的 continuity packs 自动呈现可复用 playbooks。 | `INT-09` | `UX-06`, `UX-08` |
| US-P1-02 | P1 | Seat | 我可以从另一个项目克隆 collaboration template，让新项目一开始就拥有已知的路由和治理规则。 | 新项目会因为从零重建 seat 规则和治理方式而浪费 setup 时间。 | 每个项目都会在不一致的 routing、隐藏假设和重复协调开销中启动。 | 用户从一个被验证过的 template 启动新项目，而不是手动重建协作规则。 | `INT-10` | `UX-04` |
| US-P1-03 | P1 | Supervisor, Artifact review | 我可以请 Supervisor 把口述或松散的审阅反馈转成结构化评论，再由我确认。 | 高上下文的审阅反馈通常说出来比改写成正式评论更快。 | 有价值的审阅洞察会困在临时笔记里，或者因为转录成本太高而被跳过。 | 用户口述或自由输入审阅意图，检查一个结构化草稿，并在不手工重写的前提下确认评论。 | `INT-08` | `UX-02`, `UX-07` |
| US-P1-04 | P1 | Supervisor, Data Engine | 当 supervisor 在间隔后回来时，SeatLoom 会重建一个适合该角色的 continuity pack，而不是只给 worker 使用的 LaunchPack。 | Supervisors 和 sponsors 需要与 worker seats 不同的 continuity，尤其是在中断之后。 | 管理者恢复时会处于盲态、重新阅读原始历史，或复用遗漏决策级摘要的 worker context。 | 用户通过 supervisor 专用 continuity pack 重新进入，并基于凝练且角色适配的证据做决策。 | `INT-09` | `UX-03`, `UX-06` |
| US-P1-05 | P1 | Data Engine | 我可以挂起一个 session，并在之后只用 delta context 恢复它，而不是为完整重启付出全部成本。 | 重复的 runtime 切换和恢复会把 token 花在 runtime 已经见过的上下文上。 | 即使只有很小的增量变化，上下文重建仍然昂贵、重复且有损。 | 用户把暂停与恢复当成一等操作，而不是每次都关闭后再重建。 | `INT-06` | `UX-06`, `UX-11` |
| US-P1-06 | P1 | Seat, Data Engine | 我可以看到一个 handoff 进入 `working`，并通过轻量 live activity 跟踪进展，而不必不断轮询终端。 | handoff 一旦被接受，进度在完成或失败之前都是不可见的。 | Supervisors 会继续手动轮询 seats，并对长时间运行的工作失去信心。 | 用户监控一个可选的 live activity layer 和一个可见的 handoff state strip，同时保留 replay 语义。 | `INT-12` | `UX-05`, `UX-11` |
| US-P1-07 | P1 | Seat, Data Engine | 在把某个 seat 用于重复性工作之前，我可以检查它的 Execution Template。 | Seat 的行为像黑箱，因此用户无法审计 token 花在哪里，或者工作是如何分阶段执行的。 | 团队会继续在不理解流程是确定性的、LLM-heavy 的还是可验证的情况下使用 seats。 | 用户在选择某个 seat 承担重复性工作之前，会先检查执行步骤、budgets 和 verification points。 | `INT-03`, `INT-12` | `UX-04`, `UX-11` |
| US-P1-08 | P1 | Data Engine, Supervisor, Playbook | 我可以询问某个过去决策为什么发生，并得到召回出来的证据，而不是一大段原始历史转储。 | 当相关证据分散在 checkpoints、artifacts 和过往 sessions 中时，决策理由很难恢复。 | 用户会继续反复阅读大段历史，或者向模型提出范围过大的问题，导致 grounding 很差。 | 用户提出聚焦的 why-questions，并收到一个由链接证据和 playbooks 支撑的简明回答。 | `INT-14` | `UX-02`, `UX-10` |
| US-P2-01 | P2 | Supervisor, Seat | seat 可以为我起草一个 handoff 或 artifact proposal，但在我批准之前，没有任何内容会生效。 | 人类仍然在为显而易见的下一步提案做包装，而系统本可以安全地预先准备这些内容。 | 即使下一步已经可以预测，低价值的 orchestration 工作仍然需要人工处理，从而拖慢吞吐。 | 用户审阅排队中的 draft proposals，并批准或编辑它们，而不是从零编写每个 handoff。 | `INT-11` | `UX-09` |
| US-P2-02 | P2 | Playbook, Data Engine, Seat | 我可以比较哪些 playbooks、seats 和流程在不同项目中真正节省了时间或 tokens。 | 如果团队看不到什么真正降低了时间、token 成本或返工，就无法可靠优化。 | 优化将继续停留在观点层面，成功模式也很难被证明或放大。 | 用户基于 evidence dashboards 而不是轶事来调整 seats、playbooks 与 routing。 | `INT-15` | `UX-08`, `UX-11` |

## 6. 模块契约

### 6.1 Data Engine（数据引擎）

#### 用户承诺

SeatLoom 不会把模型 tokens 浪费在那些可以用确定性方式完成的 formatting、routing、validation、retrieval 或 audit 工作上。

#### 子系统

| 子系统 | P0 契约 | P1 扩展 | P2 扩展 |
|---|---|---|---|
| Template Engine | 使用事件模板生成可读的 Inbox 与 Timeline 标签，零 LLM 依赖 | 增加更丰富的多对象模板与本地化措辞规则 | 增加可由操作方调节的模板包 |
| Pack Engine | 基于 context tiers 0-2、structured checkpoints、精确 evidence 命中、seat skills 与 playbook 匹配来构建 continuity packs | 增加 supervisor continuity packs、仅 delta 的 resume packs，以及可选的更深层 evidence 加载 | 为受限自治流程增加 proposal packs |
| Gate Engine | 在提交前校验状态流转、必填字段、角色约束、关系完整性与 budget 阈值 | 增加更丰富的规则解释与步骤级 verification 摘要 | 增加预测性 preflight warnings |
| Route Engine | 将规范对象变更投影为 Inbox items、priority bands 与具备 capability-awareness 的 seat suggestions | 增加实时进度投影与由 playbook 触发的 route hints | 为自治草稿增加 proposal routing |
| Retrieval Engine | 在任何 semantic 或 LLM 步骤之前，先查询 Layer 1 structured index 与 Layer 2 full-text search | 增加用于 why-questions 与相似度匹配的 Layer 3 semantic retrieval | 增加跨项目排序与召回调优 |
| Budget Enforcer | 在 session、handoff 与 pipeline 范围内执行硬性输入/输出 budget，并在超限时优雅停止 | 增加 warnings、delta 成本估算与节省计数器 | 增加策略调优与对比式 budget 优化 |
| Isolation Layer | 执行 need-to-know 访问控制，使 worker seats 只能看到 pack 中包含或被显式加载的证据 | 增加外部工具访问映射与基于角色的 retrieval 扩展 | 增加操作方审查与策略模拟 |
| Audit Engine | 将每个状态变更动作绑定到持久 evidence 与 ledger events | 增加 reuse counters、token-savings 估算与 review-resolution 链接 | 增加跨项目 analytics 导出 |

#### 用户会看到什么

- 可读的 Inbox 行，而不是原始事件名，
- 可读的 Timeline 行，而不是原始 ledger 代码，
- 在 WorkItems、Handoffs、comments 与 launches 上可见的 gate-failure 原因，
- 按可见 context tiers 拆分的 continuity previews，
- 在任何 LLM 摘要之前就先给出精确 evidence 搜索结果，
- 在 launch 前就显示 budget 仪表与超预算警告，
- 当 injection 或 route 失败时给出确定性的 fallback 指引。

#### 用户故事

作为用户，当 SeatLoom 创建、路由、验证、召回或恢复结构化工作时，我能直接看到可用的摘要、原因、精确证据以及 budget 后果，而不需要等待 LLM 去复述系统本来就知道的事实。

#### 交互契约覆盖

- `INT-01`, `INT-04`, `INT-05`, `INT-06`, `INT-12`, `INT-13`, `INT-14`, `INT-15`

#### UX surface 覆盖

- `UX-01`, `UX-03`, `UX-05`, `UX-06`, `UX-10`, `UX-11`

#### 规则

1. 如果结构化数据已经存在，Data Engine 必须先组装并渲染它，之后才允许考虑 LLM 调用。
2. Checkpoints 必须是结构化的一等对象，而不能只是摘要型自由文本。
3. Retrieval 顺序固定：先 structured index，再 full-text，第三是 semantic retrieval，最后才是 LLM explanation。
4. Worker seats 只能在策略限制内访问 pack 中已包含或被显式加载的证据。
5. Budget 耗尽时，必须以结构化 stop、escalation 或 partial-result handoff 收尾，而不是静默截断。
6. 每个改变项目状态的 engine 动作都必须生成 audit record 和关联 evidence。

### 6.2 Seat three-layer architecture（Seat 三层架构）

#### 用户承诺

Seat 不只是一个项目本地的角色标签。它拥有稳定的 identity、项目特定角色、协作模式，以及可见的 operating contract。

#### 层级

| 层级 | 范围 | 用户可见字段 | P0/P1/P2 边界 |
|---|---|---|---|
| Seat Identity | 全局 | name、default runtime、capability tags、preferred budgets | P0 可编辑 registry |
| Project Role Binding | 每项目 | role、authority docs、constraints、current owner、delegation badge | P0 必需 |
| Collaboration Template | 每项目 | template name、routing rules 摘要、prohibited paths 摘要 | P0 可见且可选；P1 可克隆/编辑；P2 可推荐 |

#### 附属 operating contracts

| 契约 | 目的 | P0/P1/P2 边界 |
|---|---|---|
| Seat Card | 为 routing 声明 capabilities、accepted input types、output types、budget limits 与 constraints | P0 为可信 assignment 的必需项 |
| Seat Skill | 人工编写的指导包，用来教 seat 如何在这个环境中工作 | P0 可附加，并在 pack preview 中可见 |
| Execution Template | 可检查的步骤序列，展示 engine steps、LLM steps 与 verification points | P1 可检查；P2 支持版本对比与 fork 建议 |

#### 用户会看到什么

- 一个全局 Seat Registry，同一个 seat 只需存在一次，并可在不同项目中复用，
- 一个 Seat Card 面板，展示 capabilities、input/output types、budgets、constraints 与已附加的 seat skills，
- 一个 Project Role Drawer，显示当前项目中的 active role，
- 在分派工作前就可见的 authority documents 与 constraints，
- 类似 `Flux acting for Mira on WI-009` 的 delegation badges，而不会改写 Flux 的底层 identity，
- 一个 Execution Template inspector，揭示某个 seat 如何编排确定性工作、LLM 工作与 verification。

#### 用户故事

作为用户，我可以在不同项目之间切换而无需重建 seats，同时仍然能看到当前项目中每个 seat 正确的角色、authority、skill guidance、budgets 与 execution guardrails。

#### 交互契约覆盖

- `INT-02`, `INT-03`, `INT-10`, `INT-12`

#### UX surface 覆盖

- `UX-02`, `UX-04`, `UX-05`, `UX-11`

#### 规则

1. Seat identity 在跨项目场景下保持稳定。
2. Project role binding 可以因项目而不同，但不能改写全局 identity。
3. Delegation 是一个带 scope、issuer 与 expiry 的 overlay；它不得改写历史所有权。
4. Routing 必须使用 Seat Card truth，而不能只看角色标签。
5. Seat Skill 是编写好的指导，必须与 Playbook 这种“学习到的复用”保持区分。
6. Collaboration templates 在 P0 中可见，在 P1 中可克隆/编辑，在 P2 中可被推荐。

### 6.3 Playbook system（Playbook 系统）

#### 用户承诺

当团队把某个 runtime 问题、workflow 问题或可重复的设计问题解决过一次后，SeatLoom 以后可以复用这份经验，而不必再支付相同的 token 成本。

#### Playbook 与 Seat Skill 契约

| 概念 | 性质 | 作者 | 触发方式 | 用户可见价值 |
|---|---|---|---|---|
| Seat Skill | 关于某个 seat 应如何工作的编写型指导 | Human | 显式附加到某个 seat 或 runtime context | 让 seat 的行为与预期在执行前就清晰可读 |
| Playbook | 从真实项目执行中提炼出的学习型复用 | 基于证据自动或半自动生成 | 在 continuity、routing 或 review 期间进行上下文匹配 | 防止重复犯错与重复消耗 token |

#### P0/P1/P2 边界

| 优先级 | 契约 |
|---|---|
| P0 | 在 continuity previews 中保留独立的 seat-skill 与 playbook 附着位，并说明各自为何被纳入。 |
| P1 | 从已解决 blocker 或成功 recipe 中发布 playbooks，按项目或全局范围存储，按 runtime/role/context 进行匹配，并显示节省预估与复用次数。 |
| P2 | 跨项目排序和推荐 playbooks，自动建议候选复用，并长期比较 ROI。 |

#### 用户会看到什么

- continuity previews 中的 `Seat skills` 区块，
- 与 seat guidance 分离的 `Relevant playbooks` 区块，
- 一个 Playbook Library，显示 type、trigger、scope、saved-effort estimate、reuse count 与 source evidence，
- playbook candidate 的 publish 与 accept 界面，
- 对 global playbooks 的跨项目可见性。

#### 用户故事

作为用户，我能区分“关于某个 seat 应该如何工作”的编写型指导，与“团队已经验证过什么有效”的学习型复用，并且我期待两者以不同方式共同降低上下文成本。

#### 交互契约覆盖

- `INT-06`, `INT-09`, `INT-14`, `INT-15`

#### UX surface 覆盖

- `UX-06`, `UX-08`, `UX-10`

#### 规则

1. 每个 playbook 都必须回链到源证据：session、work item、artifact 或 blocker resolution。
2. Playbooks 可以是项目局部的，也可以是全局的。
3. Playbook 应用先做确定性匹配，之后才可选用 Supervisor 做解释。
4. Saved-effort 报告是近似值，必须明确标注为 estimate。
5. Seat Skill 与 Playbook 绝不能合并成一个不透明的统一“guidance”对象。

### 6.4 Supervisor Layer（监督层）

#### 用户承诺

操作 SeatLoom 的主要方式，是表达意图，然后确认一个结构化提案或一个有证据支撑的答案，而不是从空白表单开始。

#### Supervisor 是什么

Supervisor 是内置的 orchestration layer，而不是用户创建的 seat。

它负责：

- 意图解析，
- 提案生成，
- 结构化动作拆解，
- seat routing suggestions，
- 基于证据的回答草拟，
- 在状态变更前请求确认。

#### P0/P1/P2 边界

| 优先级 | 契约 |
|---|---|
| P0 | Command Bar、suggestion cards、具备 capability awareness 的 WorkItems/Handoffs/comments 创建、Inbox quick-action suggestions、exact-evidence assist，以及所有写操作前的显式确认。 |
| P1 | 批量 proposals、supervisor continuity packs、更丰富的 confirm 前编辑流程、review-assist drafting，以及基于分层 retrieval 的 why-decision answers。 |
| P2 | 按策略调优的 orchestration proposals、多步骤 draft plans，以及需要审批的受限自动化提案。 |

#### 用户会看到什么

- 应用 shell 中显著的 Command Bar，
- 展示 proposed objects、owners、seat rationale、budgets 与 constraints 的 suggestion cards，
- 确认前的 inline edit controls，
- 带 source links 的 exact-evidence 与 why-decision answer cards，
- Inbox 行与 artifact review threads 上的 quick-action suggestions，
- 独立的 Supervisor 上下文与历史，而不是和 worker seat identity 混在一起。

#### 用户故事

作为用户，我可以说 `Create a task for Nimbus to implement OAuth with Google and GitHub` 或 `Why did we abandon passport.js?`，然后 SeatLoom 会在任何重要状态改变之前，先返回一个可审阅、由证据支撑的响应。

#### 交互契约覆盖

- `INT-01`, `INT-02`, `INT-05`, `INT-08`, `INT-09`, `INT-11`, `INT-13`, `INT-14`

#### UX surface 覆盖

- `UX-02`, `UX-03`, `UX-05`, `UX-06`, `UX-07`, `UX-09`, `UX-10`

#### 规则

1. 在 P0 和 P1 中，Supervisor 可以起草结构化动作或答案，但所有会改变状态的操作都必须由用户确认。
2. Supervisor 的 token budget 与 worker seat budgets 分离。
3. Supervisor 的输入必须是最小化上下文、角色适配且基于 retrieval grounding 的。
4. Supervisor 的输出应以结构化数据优先、自由文本次之。
5. 当 Supervisor 回答 why-question 时，它必须展示使用了哪些 evidence sources。

### 6.5 Artifact review system（Artifact 审阅系统）

#### 用户承诺

SeatLoom 在 artifact 产出之后并未结束。用户必须能够读取它、搜索它、评论它、关闭反馈，并把这轮审阅重新连接回工作执行闭环。

#### P0/P1/P2 边界

| 优先级 | 契约 |
|---|---|
| P0 | 在应用内打开完整 artifact 内容、支持手动评论、搜索精确文本、显示各 section 的 comment counts、允许 resolve/reply/dispute、将 comments 关联到 Inbox 与 Timeline，并支持 project-scope 或 multi-WorkItem 的 artifact 关联。 |
| P1 | 增加 Supervisor-assisted comment drafting、版本历史、artifact diff view，以及由 comments 触发的 work-draft suggestions。 |
| P2 | 增加 review analytics、batch review bundles，以及跨 artifacts 与 teams 的趋势视图。 |

#### 用户会看到什么

- 主面板中的全文阅读器，
- 带 comment threads 的 review side rail，
- 显示评论数量的 inline anchors 与 section badges，
- 支持精确匹配的 artifact 级搜索，
- 每个 thread 上的 resolve、reply 与 dispute controls，
- artifact detail 中关联的 WorkItems 与版本历史。

#### 用户故事

作为用户，我可以在 SeatLoom 内打开一份产品评审文档，快速找到相关段落，对其发表评论，并看到这条反馈回到对应负责 seat 那里，成为一个可执行、可审计的工作信号。

#### 交互契约覆盖

- `INT-05`, `INT-07`, `INT-08`, `INT-13`, `INT-14`

#### UX surface 覆盖

- `UX-05`, `UX-07`, `UX-10`

#### 规则

1. Artifact review 是主工作闭环的一部分，而不是只能跳转到外部编辑器的逃生口。
2. 手动评论与 Supervisor-assisted 评论必须产生相同底层 comment object 与 audit trail。
3. Comment resolution 必须在 Inbox、Timeline 与 artifact detail 中可见。
4. Artifact 关联可以是单 WorkItem、多 WorkItem，或项目级范围。
5. 精确的 artifact 搜索必须在不依赖 LLM 的情况下完成。

## 7. 集成后的 P0 用户旅程

用于 implementation 的基线用户旅程如下：

1. 用户打开 SeatLoom，看到 Overview 与 Inbox guidance。
2. 用户要求 Supervisor 创建或路由工作。
3. Seat identity、Seat Card、project role binding、collaboration mode 与 budget limits 决定谁可以接收这项工作。
4. Data Engine 以确定性方式生成 summaries、gates、routes、retrieval hits 与 continuity previews。
5. 一个 worker seat 在外部 runtime 或被包装的 session 中执行。
6. 用户在 SeatLoom 内审阅产生的 artifact，必要时发表评论，然后选择接受或 reissue 工作。
7. 如果发生 runtime 变化或中断，SeatLoom 会通过 structured checkpoints、context tiers、exact evidence retrieval 与确定性 fallback 来重建上下文。

如果这条旅程中的任何一步缺少 story、interaction contract、UX definition 或 value-test answer，implementation 就必须停止，直到缺口被补齐。

## 8. 与生态的对齐决策

SeatLoom 现在将以下生态经验纳入产品契约：

| 来源输入 | 已采纳的产品决策 | 范围 |
|---|---|---|
| Agent Skills | 使用渐进式加载逻辑与独立的编写型 `Seat Skill` 概念，而不是把所有复用都视作一个大杂烩 | P0 |
| MCP | 在 Data Engine 内采用 capability declaration、仅 pack 内可见的隔离，以及分层加载 | P0 |
| A2A | 使用 Seat Card 结构与明确的 Handoff progress states，而不是非正式的 seat 描述 | P0/P1 |
| Harness engineering | 将 budget 与轻量 verification 视为产品强制执行的控制，而不是可选纪律 | P0 |
| Context/retrieval design | 采用 structured checkpoints、context tiers、layered retrieval 与 delta resume，以最大化 token ROI | P0/P1 |

SeatLoom 认可以下想法，但暂时不将其纳入当前交付范围：

| 延后项 | 延后原因 |
|---|---|
| 将 SeatLoom 暴露为面向第三方 runtimes 的通用 MCP Server | 这是重要的架构选项，但在成为交付承诺前，需要先做单独的价值与安全决策 |
| 用户可编辑地 fork seat Execution Templates | 后续有价值，但当前优先级是“可检查与建立信任”，而不是“可编辑行为分叉” |
| 默认始终开启 Live Timeline mode | Live mode 有价值，但 replay 清晰性仍是首要目标，噪音必须保持可选 |

## 9. 非目标

v0.5 契约仍然明确排除：

- 静默提升为 autonomous work，
- 无限制的跨-seat 数据访问，
- 把 cloud-first coordination 作为必需条件，
- 取代 IDE 或 model-native chat tools，
- 将产品定位为通用 issue-board 项目管理工具，
- 在没有人工审批的前提下进行无限制的 agent-to-agent orchestration，
- 在核心用户价值尚未被清晰写入契约前，就交付面向生态的协议暴露能力。

## 10. 进入 implementation 的检查清单

只有在以下条件全部满足后，implementation 才可以开始：

- `docs/prd-v0.5.md`、`docs/interaction-spec-v1.1.md` 与 `docs/ux-spec-v1.1.md` 之间不存在未解决的 P0 冲突，
- 每个 P0 功能都至少映射到一个用户故事、一个交互流程和一个 UX surface，
- 每个 P1 与 P2 功能都被显式定界，并以故事形式可见，
- 每个在范围内的功能都明确写出了所解决的用户痛点、省略后的后果，以及交付后用户行为的变化，
- Aegis stage-gate review 将这组契约标记为 `GO`。
