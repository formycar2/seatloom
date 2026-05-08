# SeatLoom Product Value Review — Real Collaboration Pain Points

| Field | Value |
|---|---|
| doc | 2026-04-30-seatloom-value-review |
| scope | SeatLoom v0.5 product value vs. team's actual daily collaboration mode |
| basis | PRODUCT_TRUTH.md · prd-v0.5.md · architecture-decisions.md · COORDINATION_RULES.md · recent 3-day collaboration reality |
| status | issued |
| author | aegis |
| date | 2026-04-30 |

---

## 0. Premise: What our collaboration currently looks like

Over the past 72 hours, this team has collaborated on SeatLoom itself using the very pattern SeatLoom is meant to solve. The current reality:

**How we work today:**
- **Task packets** → written as markdown files in `docs/coordination/tasks/<role>/`, hand-edited, dispatched via terminal summary to each role
- **Reviews** → written as markdown files in `docs/coordination/reviews/`, passed through terminal, no machine-readable structure
- **Acceptance** → written as markdown files in `docs/coordination/acceptance/`, verdict stated in prose
- **Decisions** → scattered across terminal transcripts, PRs, and markdown files, with no single queryable registry
- **Role trust** → based on filename conventions, human memory, and terminal reputation
- **Context continuity** → every new agent session must re-read all markdown from scratch; context is rebuilt every time
- **Evidence retrieval** → `grep` or manual directory scanning; no structured index, no FTS, no semantic layer
- **Session state** → ephemeral terminal output; once output scrolls past, the state is gone
- **Git/MR lifecycle** → review comments, CI artifacts, and decisions are locked in PR threads, not surfaced systematically
- **Audit trail** → git log is the only durable history; cross-session cross-file event causation is reconstructed manually

**What SeatLoom promises to replace this with:**
A structured, local-first, deterministic system where intent → structured proposal → Inbox → WorkItem → Handoff → Artifact → Gate → Ledger forms a closed, auditable loop.

---

## 1. Pain Point Analysis and SeatLoom's Value Map

### Pain 1 — Task Dispatch Is Manual and Has No State Machine

**Current pain:**
- Task packets are authored by PO (Lyra/Aegis) as markdown files, dispatched via terminal summary to each role
- Each role reads the file, states acceptance, and begins work in terminal output with no formal "delivered" trigger
- WorkItem status (Ready → Active → Blocked → InReview → Done) is tracked implicitly, not as a machine-readable state transition
- No central WorkItem registry; status drift across concurrent MRs and sessions is common

**SeatLoom value:**
| Concept | Pain → Value |
|---|---|
| `WorkItem` | turns implicit status into explicit, durable states |
| `WorkItemStatus` enum (Draft / Ready / Active / Blocked / InReview / Verified / Done / Reopened / Drifted) | replaces ad-hoc status tracking |
| `InboxItem` | provides the PO's decision queue instead of scattered PR + terminal messages |
| `Gate Engine` | validates transitions before commit (AD-010): InReview → ReviewVerdictIssued → Blocked → Ready on reissue |
| `depends_on` | hard state-machine lock prevents InReview when prior WorkItems are not Done |

**Coverage assessment:** v2 前端当前完全没有 InboxItem 和 WorkItem 视图（见数据覆盖率审计报告）。Supervisor IM 中 WorkItem 消息只以静态卡片形式展示，没有状态机或操作入口。**SeatLoom 的核心价值承诺在 v2 中尚未体现。**

---

### Pain 2 — Context Is Lost Between Sessions and Across Users

**Current pain:**
- MR 讨论被冻结在 PR thread 里，新对话上下文须手动重建
- 每个 agent session 启动时，角色（Lyra/Mira/Nimbus/Flux）须从头阅读最近的任务文件
- Session 中断后恢复须重新解释上下文，无法从上次断点续接
- Aegis 作为 Supervisor 指导多个并行角色，但每个角色看到的上下文不同

**SeatLoom value:**
| Concept | Pain → Value |
|---|---|
| `Session` + `launch_pack_ref` | 斯托续包（LaunchPack）结构化，在启动时注入 Session 上下文 |
| `Checkpoint` | 第一个结构化检查点（Checkpoint），不仅仅是总结自由文本 |
| `SeatSkill` | 为每个席位附加结构化运营指南，让下一个 agent 接手时可见 |
| `Playbook` | 解决问题一次，复用自动加载 |
| `PromptState` (AD-012) | interactive prompt（菜单/向导）被分类为机器可处理状态，不再需要人工观看滚动输出 |

**Coverage assessment:** v2 完全没有 Session 或 Checkpoint 视图（同样在 P0 缺失列表）。Supervisor IM 似乎是"上下文管理中心"的替代品，但目前是手动状态，不是 Ledger 记录的结构化状态。**一半的价值承诺尚未被 v2 体现。**

---

### Pain 3 — Artifacts (Reviews, Gates, Acceptances) Are Just Markdown Files

**Current pain:**
- Reviews, gate decisions, and acceptances 都只是 markdown 文件，没有内部结构或索引
- Finding a specific gate decision from 3 days ago requires manual file scanning or `grep`
- The template+subtype system (T1-T7) is defined on paper but not enforced or indexed in any productive tool
- Artifact review happens by reading the whole markdown file; there is no typed metadata first card, no review rail, no linkage to specific WorkItems

**SeatLoom value:**
| Concept | Pain → Value |
|---|---|
| `Artifact.template + subtype` (AD-008) | T1-T7 双键驱动 Detail Pane 布局，不仅是更多 markdown |
| `Artifact.subtype_valid` | 无效类型时自动降级 + 可见警告 |
| Layer 1 (PostgreSQL structured index) | 精确字段查询在 T1 字段（id, status, workitem_id, template, subtype, tags）上运行 |
| Layer 2 (FTS) | tag + 正文全文检索 |
| `CanonicalEvent.ArtifactCreated` | Artifact 产出自动上链至 Ledger |
| Artifact review rail | 结构化批注 + 线程化讨论，而不是标记文本 |
| `ReviewVerdictIssued` event (AD-010) | Gate 决策绑定证据，而不是孤立文件 |

**Coverage assessment:** v2 完全没有 Artifact 视图（P0 缺口）。T1/T2/T4/T6/T7 模板在 v2 中不通过索引，\~58 个此类文件（占文档数的约 26%）完全不可通过 v2 检索。T3 + T5 仅在消息气泡中被碎片化访问。**Artifact 系统的价值承诺在 v2 中基本未兑现。**

---

### Pain 4 — Handoffs Across Seats Have No Traceable State

**Current pain:**
- Work 在一个席位完成并通过 terminal summary 转交给另一个席位
- Handoff 状态（Drafted / Sent / Received / Accepted / Working / Returned / Completed）在 terminal 中口头描述
- 如果接收席位遇到问题，原发起者不知情，直到主动询问
- 争议时没有可追溯的 handoff evidence chain

**SeatLoom value:**
| Concept | Pain → Value |
|---|---|
| `Handoff` with `status` enum | Drafted → Sent → Received → Accepted → Working → Returned → Completed 状态机 |
| `artifact_ids` on Handoff | 自动将 Handoff 产出绑定到 Artifact |
| `required_receipt: bool` | 强制签署方确认接收，否则 Handoff 不是完成的 |
| `CanonicalEvent.Handoff*` 事件系列 | 完整 Ledger 审计轨迹 |
| `HandoffWorking` (P1, US-P1-06) | 可选的实时活动可见性层 |

**Coverage assessment:** v2 完全没有 Handoff 视图（P1 缺口）。**Handoff 机制的设计在工作，但价值未被用户可见。**

---

### Pain 5 — Budget / Trust / Authority Is Implicit and Easy to Violate

**Current pain:**
- token 使用由开发者注意，无系统强制
- 席位能做/不能做的边界基于口头规则或 CI 误判
- PO/Supervisor 的权限 upheld 靠人工监督
- 一个席位的上下文 cost（$）独立于结构管理
- 实时产出是否被信任（例如经 Nimbus 初始化基础设施是否经 Aegis 审核）取决于人工跟进

**SeatLoom value:**
| Concept | Pain → Value |
|---|---|
| `PackEngine` Tier 0/1/2 结构注入 | 上下文分为 "总是包含" 和 "预算批准" 两部分 |
| `BudgetEnforcer` | 硬上限，不是团队礼仪 |
| `GateEngine` | 状态转换前强制执行字段 + 角色 + 预算资格 |
| `IsolationLayer` | Need-to-know 访问控制 → 席位只看到其包包含的证据 |
| `SeatCard` | 能力、预算、约束在分配前可见性 |
| `Delegation` | 临时代理不重写原始身份（AD-009）|

**Coverage assessment:** v2 完全没有 Budget 或 SeatCard 视图（P2 缺口）。Seat 卡片只显示简化信息（avatar/name/role/color）。**核心信任/授权架构在设计中有定义，但 v2 中尚未展示。**

---

## 2. Quantifying the "Help" — Current Collaboration Coverage

```
Current team collaboration reality:
  任务分发              → Markdown 文件 + 终端总结           [手工，无状态机]
  Review/Acceptance     → Markdown 文件                      [手工，碎片化]
  Status tracking       → 人工记忆 + PR thread                [手工，易漂移]
  Artifacts             → 约 220 个 Markdown 文件              [可 indexed,Grep]
  Evidence retrieval    → grep / 手动目录扫描                     [低效]
  Audit trail           → git log                            [单向，无事件关联]
  Session continuity    → 重建上下文，人工重新阅读               [重复开销每 session]
  Seat/role trust       → 人工监督，CI 误判                      [脆弱]
  Handoff state         → 口头转达 + terminal 描述              [无结构]
  Budget/enforcement    → 人工注意，CI 误判                    [自愿遵守]

SeatLoom product promises:
  → 5 个模块: Data Engine · Seat 三层 · Playbook · Supervisor · Artifact Review
  → P0: 15 个用户故事 (US-P0-01 ~ US-P0-15)
  → P1: 8 个用户故事 (US-P1-01 ~ US-P1-08)
  → P2: 2 个用户故事 (US-P2-01, US-P2-02)
  → P3: 1 个用户故事 (US-P3-01)

v2 frontend current state (post-Copilot modularization):
  ✅ Partially (T3+T5 in IM bubbles)
  ⚠️ Partially (Seat simplified, Timeline flat, DAG workItemRef)
  ❌ Not in v2: Session, Artifact, Inbox, Handoff, CanonicalEvent full,
                Budget, SeatCard full, Playbook, PromptState

v2 data coverage: ~22% of architecture types have UI representation
v2 template coverage: ~25% of coordination docs (only T3+T5) surfaced
```

---

## 3. What SeatLoom Actually Solves (When Built)

### 3.1 Solves — Task Dispatch and Review Loop

SeatLoom 将"任务生成 → 路由 → WorkItem 创建 → Inbox → 审查者查看 → 验证 → Gate 决策"的循环转为机器可读状态机。在 v0.5 P0 范围内，**`InboxView` + `WorkItemsView` + `Gate Engine` 是最核心的价值路径**。这部分在 v2 中完全缺失。

### 3.2 Solves — Artifact Evidence Traceability

T1-T7 双键 + PostgreSQL structured index + FTS + canonical events 将"220 个 markdown 文件"转化为"可精确检索、可引用、可审计的项目真相"。v2 的 T1+T4+T7 文档无 UI 入口，意味着这部分价值在 v2 中未被用户可见。

### 3.3 Partially Solves — Context Continuity

Session → Checkpoint → LaunchPack + SeatSkill 是 SeatLoom 最有吸引力的特性（连续工作体验）。v2 在 AppV2.mock data 中有 continuity pack 的概念引用，但没有 Session 视图让用户实际查看、比较、选择。

### 3.4 Partially Solves — Delegation Trust

Seat Delegation Overlay（AD-009）让 Flux 代表 Mira 工作而不改写 Mira 身份。ChagInput 中 `isDirectWorker → routingGuard → via PO` 的逻辑是这一精神的基础 UI 体现。**v2 这一部分做得最好**（相对其他核心模块而言），但在 Ledger / Artifact 中不可追溯到 Delegation 实体。

### 3.5 Does Not Yet Solve in v2 — Budget Enforcement / Trust / Authority

核心信任架构（`BudgetEnforcer` · `SeatCard` · `AuthorityDocRefs` · `GateEngine`）是 P0 PP，但 v2 没有可见的 UI，没有可以让用户交互的"验证"或"路由"流程。

---

## 4. How Much Does This Help Our Actual Collaboration Mode?

### 4.1 Quantified answer

| 协作痛点 | 当前严重程度 | SeatLoom 帮助程度 | v2 已覆盖 |
|---|---|---|---|
| 任务分发和状态机 | 高（手工，易漂移）| P0: **核心价值** | ❌ 未覆盖 |
| Review/Gate 闭环 | 高（手工，碎片化）| P0: **核心价值** | ⚠️ 部分（静态卡片）|
| Artifact 检索 | 中高（grep + 手动扫描）| P0: **核心价值** | ❌ 未覆盖 |
| Context 连续性 | 中高（每 session 重建代价大）| P1: **高价值** | ❌ 未覆盖 |
| Handoff 链路 | 中（口头转达易丢失）| P1: **高价值** | ❌ 未覆盖 |
| Seat 信任和授权 | 中（依赖人工监督）| P0: **架构承诺** | ⚠️ 部分（简化卡片）|
| 委派覆盖 | 低（在 SeatLoom 内很少跨席位）| P0: **已部分设计但无 UI** | ⚠️ 部分（routing guard）|
| Playbook 复用 | 低（本地解决，尚未出现）| P1: **累积价值** | ❌ 未覆盖 |
| Budget 执行 | 低（尚未出现 token 爆炸危机）| P0: **架构承诺** | ❌ 未覆盖 |
| Prompt 状态分类 | 低（不适用，目前 agent 角色没有 blocking interactive）| P0: **架构承诺** | ❌ 未覆盖 |

**总体评估：** SeatLoom v0.5 P0 承诺解决的 10 个主要痛点中，大约 **4 个有部分实现（routing guard / IM 消息卡片 / DAG / 简化 seat 卡片）**，但其余 **6 个 P0 核心价值（Inbox、WorkItem 状态机、Artifact 检索、Session/Checkpoint、Gate 决策闭环、Budget/Authority 验证）在 v2 中完全没有用户可见的 UI**。

**v2 当前在本质上只是一个"工作台"，还没有变成"操作系统"。它能**展示**工作状态，还不能**执行**工作流。**

### 4.2 Why the gap matters

我们的团队协作模式（人工 task packet + review + acceptance 文件 + terminal 通讯）正是 SeatLoom 的 P0 目标受众：AI 原点项目团队的本地-first协作。

**如果 SeatLoom 按照 PRD 承诺交付完整的 P0 用户故事，对我们这个团队的具体帮助：**

1. **Aegis (Supervisor)** 不再需要为每个角色写 markdown 任务文件 → 通过自然语言 → Inbox → WorkItem 状态机追踪每个角色的进展
2. **Lyra (PO)** 在 Supervisor Command Bar 就能掌握所有项目的 Gate 状态，不用翻遍 reviews/ 和 acceptance/ 目录
3. **Mira (UX)** 的连续工作上下文不会在每次重启后丢失 → SeatSkill + LaunchPack 自动续接
4. **Nimbus (Arch)** 的交付物通过 Artifact 系统而不是独立 markdown 文件追溯
5. **Flux (QA)** 的验证结果（review 报告）成为 Structured Event 而不是纯人工报告

**这 5 个核心变革的真值在 SeatLoom 中是有架构设计定义的，但 v2 还没有 UI 实现。**

---

## 5. The "Distance" to Full Value

```
SeatLoom v0.5 value (P0 promise):
├── Data Engine: deterministic routing, validation, retrieval, audit
│   ├── InboxItem decision queue          ← v2: ❌ 完全无 UI
│   ├── WorkItem state machine            ← v2: ⚠️ 静态展示（DAG 节点 ID）
│   ├── Gate Engine logic                 ← v2: ❌ 无 UI
│   ├── Retrieval Engine (L1+L2）         ← v2: ❌ 无 UI
│   ├── BudgetEnforcer                    ← v2: ❌ 无 UI
│   └── Audit/CanonicalEvent              ← v2: ⚠️ 扁平（无事件全量）
├── Seat three-layer:
│   ├── Seat three-layer identity         ← v2: ⚠️ 扁平（ChatContact）
│   ├── Seat Card (capabilities/budgets）← v2: ❌ 无 UI
│   ├── Delegation overlay                ← v2: ⚠️ 原型（routing guard logic only）
│   └── Collaboration template            ← v2: ❌ 无 UI
├── Playbook system:
│   ├── SeatSkill attachment               ← v2: ❌ 无 UI
│   └── Playbook discovery/publish         ← v2: ❌ 延迟至 P1
├── Supervisor Layer:
│   ├── Command Bar (intent → proposal）   ← v2: ⚠️ 原型（IM 是最好的演绎）
│   └── Continuity pack preview            ← v2: ❌ 无 UI
└── Artifact review:
    ├── T1-T7 template dual-key            ← v2: ❌ 无 UI（消息卡片不引用结构化模板）
    ├── ReviewRail                         ← v2: ❌ 无 UI
    ├── Artifact family/subtype filters     ← v2: ❌ 无 UI
    └── Comment/feedback loop              ← v2: ❌ 无 UI

v2 P0 完整覆盖               ~22% / 15 个数据实现对象有 some UI
v2 P0+P1 完整覆盖（含 Artifact）~44%
v2 P0+P1+P2 完整覆盖          ~67%
```

---

## 6. Key Insight: We Are Building the Tool We Need

There is a notable alignment: the collaboration patterns SeatLoom is designed to solve (task packets → WorkItems → review → Gate → Reconcile) mirror the exact collaboration patterns we are using to build SeatLoom right now.

| Building SeatLoom using... | Currently doing... | SeatLoom would automated... |
|---|---|---|
| Task packets as markdown | `COPILOT-*.md` in `tasks/` | Inbox → WorkItem → state machine |
| Reviews as markdown | `reviews/` files | Review → Artifact → canonical events |
| Acceptances as markdown | `acceptance/` files | Gate Engine → structured verdict |
| Terminal → manual dispatch | terminal summary + handoff | Handoff state machine |
| Context re-read on session start | every agent reads all fules | LaunchPack / SeatSkill |
| Git log → audit trail only | git log + manual reconstruction | CanonicalEvent + Ledger |
| Route via human memory | PO decides routing | Route Engine + SeatCard truth |

**This means SeatLoom is being built by a team currently suffering from the exact pain SeatLoom is designed to solve.** That is both a strength (the team deeply understands the pain) and a risk (the team may underestimate how much manual workflow has been absorbed into muscle memory and thus how transformative the product would be).

---

## 7. Prioritized Value Delivery

If the team can only ship P0, the **top 3 value moments** (ordered by daily impact):

1. **InboxView + WorkItemsView (P0 fix #1-2 in audit)** — Users (Aegis + Lyra) immediately see what needs attention, routed from WorkItems and artifacts, without hunting across files. This is the "start-of-day supervision" pain (US-P0-01).

2. **Artifact System + L1/L2 Retrieval (P0 fix #3)** — ~220 markdown documents become queryable by template+subtype+status+author. The supervisor (Aegis) can find the exact gate decision from 2 days ago in seconds, not minutes. This is US-P0-10.

3. **Gate Engine + WorkItem State Machine (implicit in InboxView)** — WorkItem transitions become auditable and deterministic, not manual status updates in markdown. This is US-P0-05.

These three together replace: manual file scanning + grep + PR thread scrolling + terminal memory + markdown status updates with a single structured, queryable workflow.

---

## 8. What "Not Building" Would Cost

Per PRD §3:

| Omitted module | Direct cost | Given current work mode |
|---|---|---|
| Data Engine + Inbox/WorkItem | Task dispatch regresses to markdown | High — role executions become uncoordinated |
| Artifact system + retrieval | All reviews/gates/acceptances stay unstructured | High — review findings cannot be traced or replayed |
| Session + Checkpoint | Context rebuild costs are paid every session | High — agents re-read all context from scratch |
| Handoff state machine | Corridor handoffs lose audit trail | Medium — disputes become harder to resolve |
| Seat identity | Role trust remains folklore | Medium — routing remains by memory, not capability |

---

## 9. Conclusion

**SeatLoom's fundamental value proposition is real and directly addresses the team's current collaboration pain.** The architecture covers the full stack:

- **Task/state machine layer** (WorkItem → Inbox → Gate) — solves dispatch and status tracking
- **Continuity layer** (Session → Checkpoint → LaunchPack → SeatSkill) — solves context rebuild cost
- **Artifact layer** (T1-T7 templates → Artifact → Retrieval) — solves evidence traceability
- **Audit/Compliance layer** (Ledger → CanonicalEvent) — replaces git log + manual reconstruction

**The implementation gap is not in architecture; it is in v2 UI delivery.** The data coverage audit shows only ~22% of data objects and ~25% of document templates have v2 UI representation. The P0 value (Inbox → WorkItem → Artifact → Gate loop) is fully architected but not yet surfaced.

**The retrofit cost of building these gaps incrementally is low** — the types, storage schemas, and event models are already defined. The remaining work is UI surface construction, which can be done in parallel with backend completion. The risk is not rebuilding v2's harder path: the v2 IM-style Supervisor panel is now a product, but it needs the structured backend layers to fulfill the P0 promise.

**SeatLoom is the tool the team needs to stop building SeatLoom manually.**

---

---

## 10. L1/L2 Priority Layer Impact (chan-06/chan-08 补充, 2026-05-08)

本审查最初以"模块完整性"视角评估 v2 覆盖率。2026-04-30 讨论产出 chan-06 洞察后，评估框架需要修正：

### 10.1 对本文 §7 优先级排序的影响

| 本文原始排序 | L1/L2 修正后理解 |
|---|---|
| #1 InboxView + WorkItemsView | **L2** — 仍然重要，但不再是"第一个做" |
| #2 Artifact System + L1/L2 Retrieval | **L2** — 检索是按需行为 |
| #3 Gate Engine + WorkItem State Machine | **L2** — 管理视图 |

**修正后的优先级视角**：L1 surface（Supervisor IM + GlobalDashboard + ProjectDashboard）的加固优先于 L2 data views 的新建。chan-03（GlobalDashboard）和 chan-09（viewMode 正交状态）已经按这一原则执行。

### 10.2 对本文 §4.1 表格的补充标签

| 协作痛点 | Layer | 说明 |
|---|---|---|
| 任务分发和状态机 | L2 | 需要独立 WorkItem 视图 |
| Review/Gate 闭环 | L2 | 需要 Artifact + Gate 视图 |
| Artifact 检索 | L2 | 需要 Artifacts 检索界面 |
| Context 连续性 | L1 | Supervisor IM 自动续接 = L1 核心体验 |
| Seat 信任和授权 | L2 | SeatCard 是管理视图 |
| Playbook 复用 | L2 | 库管理是低频操作 |

### 10.3 结论修正

> 原文结论"实现差距在 v2 UI 交付"仍然成立，但**交付顺序**应按 L1→L2 排列。L1 surface 的加固（让 Supervisor IM 更强、ProjectDashboard 信息密度更高）优先于 L2 独立视图（InboxView/WorkItemsView/ArtifactsView）的全新建设。

---

*Value Review by Aegis · 2026-04-30 · L1/L2 supplement added 2026-05-08 · Basis: PRODUCT_TRUTH.md + prd-v0.5.md + architecture-decisions.md + COORDINATION_RULES.md + 3-day collaboration reality analysis*
