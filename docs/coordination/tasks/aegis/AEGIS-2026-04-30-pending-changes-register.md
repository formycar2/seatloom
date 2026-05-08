# Changes Register — SeatLoom v2 Architecture Alignment (2026-04-30)

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-04-30-pending-changes-register |
| status | issued |
| author | aegis |
| date | 2026-04-30 |
| to | aegis |
| priority | P0 |
| acceptance owner | aegis |

---

## 背景

在 2026-04-30 的探讨中，对齐了多个结构性理解，产出了若干设计洞察。这些洞察需要同步到：
1. **设计文档**（architecture-design.md, architecture-decisions.md, prd-v0.5.md 等）
2. **实现层面**（v2 frontend + Rust data model）

本文档是所有待变更的**集中登记表**，不执行任何变更，只记录"变更什么、为何变更、为何优先级"。

---

## 讨论产出物（已生成的审查文档）

| 文档 | 用途 |
|---|---|
| `docs/coordination/reviews/2026-04-30-v2-data-coverage-audit.md` | v2 数据对象覆盖率审计（15 对象 + T1-T7 模板覆盖） |
| `docs/coordination/reviews/2026-04-30-seatloom-value-review.md` | SeatLoom 产品价值与实际协作痛点对照 |
| `docs/coordination/reviews/2026-04-30-supervisor-im-as-l1-insight.md` | Supervisor IM 为 L1 最高频交互的设计洞察 |

---

## 待变更清单

### chan-01 · [共识] Q3 Seats 不在 Project 外

- **来源**：Q3 对齐
- **状态**：✅ 共识，无需文档变更（确认 Company 不是实体）
- **影响范围**：无（仅为理解对齐）

---

### chan-02 · [共识] Q4 Person-Supervisor 1:1

- **来源**：Q4 对齐
- **状态**：✅ 共识
- **影响范围**：`architecture-design.md §6.1 Seat store`（建议更新 Person 与 Supervisor 的绑定描述，但 Person 不是实体，不引入 Person 实体）

---

### chan-03 · [共识] Q4-Q5 Supervisor 两层上下文模型

- **来源**：Q4-Q5 对齐，Mr. Zhang 描述
- **状态**：✅ **CLOSED — 2026-05-07**（acceptance: `docs/coordination/acceptance/2026-05-07-lyra-app-v2-supervisor-context-mode-acceptance.md`）
- **核心内容**：
  - Supervisor 在一个 project 中只在一个 context 下工作
  - Global context：跨项目全局摘要，各 project 状态一览
  - Project context：某个 project 内部细节视图，不混入其他 project context
  - 设计意图：避免跨 project context 污染，同时支持全局健康监控

- **已交付**：
  - `architecture-decisions.md` → AD-013 记录上下文层级设计（含 mutex 不变式）
  - `architecture-design.md` → §7.1 supervisorStore 类型定义与 localStorage 键
  - `SupervisorPanel.tsx` → `currentContextMode` local state + breadcrumb + stale fallback
  - `mock-data.ts` → `MOCK_GLOBAL_SUMMARY`（project health cards）
  - `dashboard/GlobalDashboard.tsx` → 全量 project 摘要视图（new, 193 lines）
  - 实现 commit: `f5b8423`；delivery commit: `6038ba6`

---

### chan-04 · [共识] Q3 Seat vs Session vs Runtime 三层关系

- **来源**：Q3 对齐
- **状态**：✅ 共识，**需要写入设计文档**
- **核心内容**：
  - **Seat** = 身份（L1Identity + L2RoleBind），跨 project 稳定
  - **Session** = 具体运行中的工作单元（有 status/pid/workspace_path/launch_pack_ref）
  - **Runtime** = Session 的属性，不是 Seat 的属性
  - **sandbox/path/console 资源 = Session 启动参数**（不是 Seat 的常驻字段）

- **待变更文档**：
  - `architecture-design.md §3` → 更新 `workspace_path` 所在位置（从 Seat 更正为 Session），并注意目前 `Session` struct 已有此字段

- **待变更实现**：
  - `ui/src/types/index.ts` → 增加 `SandboxConfig / ConsoleConfig` 等 Session 启动资源字段（如尚无）

---

### chan-05 · [共识] Q4 Playbook 作用域精确性

- **来源**：Q4 对齐，Mr. Zhang 描述："Playbook 可以灵活共享，也可以精确赋能到 runtime"
- **状态**：✅ 共识，**需要写入设计文档**
- **核心内容**：
  - Playbook 作用域不应钉死在单一维度，而应是可配置的 binding scope
  - 支持四级 binding：
    - Company 全局（所有 project 所有 seat 所有 runtime）
    - 按 Project
    - 按 Seat
    - 按 Project+Seat+Runtime 精确匹配

- **待变更文档**：
  - `architecture-decisions.md` → 更新或新增 AD-004 关于 Playbook 的作用域定义
  - `architecture-design.md §12` → 更新 Playbook binding 数据结构

- **待变更实现**：
  - `crates/seatloom-core/src/objects/playbook.rs`（若存在）→ 增加 `PlaybookBinding { project_id?, seat_id?, runtime? }`

---

### chan-06 · [洞察] Supervisor IM 是 L1，传统数据视图是 L2

- **来源**：Mr. Zhang 2026-04-30 反馈
- **状态**：✅ 洞察（已写入 `supervisor-im-as-l1-insight.md`）
- **影响范围**：重塑整个 v2 优先级判断逻辑、所有优先级排序重构

- **核心影响**：
  - **传统 L2 视图的定位**：Inbox/WorkItems/Artifacts/Sessions/Handoffs → 必要但不紧急，使用频率低 10-30 倍
  - **L1 工作流的定义**：Supervisor IM 意图 → 结构化 Proposal → 即时确认 → 无需切换视图
  - **阶段收到的建设顺序**应该 reverse 回 L1 目标

- **待变更文档**：
  - `prd-v0.5.md` → 更新优先级定义，新增 L1/L2 分层概念
  - `docs/coordination/reviews/priorities.md`（新建）→ 统一记录优先级判断原则
  - `docs/coordination/deliveries/` → 阶段 3/4/5 的预期成果按 L1→L2 重新描述

---

### chan-07 · [待确认] v2 Copilot Phase 1-2 设计意图

- **来源**：Copilot Phase 2 交付说明
- **状态**：⚠️ 需要确认是否符合 L1 原则
- **现有设计**：
  - Phase 1：AppV2.tsx 模块化拆分 ✅
  - Phase 2：InboxView + WorkItemsView + ArtifactsView（P0 数据覆盖）
- **质疑**：Phase 2 把 P0 资源投入传统 L2 视图，而非继续加固 L1 Supervisor IM
- **待确认**：Inbox/WorkItems/Artifacts 是否应该作为"正文章节"，还是继续作为L2 完成任务甚至在完成 L1 后再完成

- **需要等待 Mr. Zhang 决策后再决定文档/实现的调整**

---

### chan-08 · [共识] Mr. Zhang 对价值认知的修正

| 之前我的认知 | 修正后的理解 |
|---|---|
| P0 = 架构上承诺的功能 | P0 = **L1 最高频 + 最短操作路径 |
| 所有 P0 模块同等重要 | P0 = L1 (Supervisor IM) > P0-L2 (Inbox/Artifact) > P1 |
| 数据覆盖率是模块完整性 | = **那个模块让 Supervisor IM 更强** |
| DAG/Blockers 是功能 | 这些只 Ｌ是信息 **密度** 的展示，不是功能的完整实现 |

- **待变更文档**：
  - `docs/coordination/reviews/2026-04-30-seatloom-value-review.md` → 补充以上洞察对评估的影响（注意，Chan-06 才是正反馈）
  - `docs/coordination/reviews/2026-04-30-v2-data-coverage-audit.md` → 补充 L1/L2 标签，标记传统数据视图的优先级

---

### chan-09 · [共识] Supervisor viewMode 正交状态（AD-013 v2）

- **来源**：Mr. Zhang 2026-05-07 设计讨论；Lyra review 确认
- **状态**：✅ **CLOSED — 2026-05-08**（acceptance: `docs/coordination/acceptance/2026-05-08-lyra-app-v2-viewmode-chan-09-acceptance.md`）
- **核心内容**：
  - 引入 `SupervisorViewMode = 'dashboard' | 'chat'` 与 `contextMode` 正交的状态维度
  - `contextMode` 专司 dashboard 数据隔离（AD-013 v1 不变量严格保留）
  - `viewMode` 专司右侧 pane 路由 + 面包屑渲染（4 种情况决策表）
  - `switchContact` for seat 不再自动同步 `contextMode`（解耦 chat 与 dashboard）
  - 面包屑中间层在 chat 模式下变为 active link（`enterProjectFromBreadcrumb` 同步 activeContactId）
  - 跨 project 分叉态（dashboard p-1 + chat p-2）为合法 UI 状态

- **已交付**：
  - `architecture-decisions.md` → AD-013 v2（commit `4f051b3`）
  - `architecture-design.md` → §7.1 supervisorStore v2 接口（commit `4f051b3`）
  - `ui/src/app-v2/types.ts` → `SupervisorViewMode` 类型
  - `ui/src/app-v2/panel/SupervisorPanel.tsx` → viewMode state + breadcrumb 4-case + `enterProjectFromBreadcrumb` + 解耦 switchContact
  - 实现 commit: `cadf36e`；delivery commit: `eb0ddaa`

---

## 文档更新顺序（设计先于实现）

```
Step 1 — 生成文档层
  docs/architecture-decisions.md
    → 新增 AD-AEGIS-01: Supervisor 两层上下文模型 (chan-03)
    → 新增 AD-AEGIS-02: Playbook binding scope 自由度 (chan-05)
  docs/architecture-design.md
    → §3 Session struct 说明更正为归属 Session 而非 Seat (chan-04)
    → §12 Playbook 结构更新 (chan-05)
  docs/prd-v0.5.md
    → §3 优先级层新增 L1/L2 定义 (chan-06)
    → Module contracts 更新 Supervisor Layer 的 L1 定位
  docs/coordination/reviews/priorities.md [新建]
    → 统一记录 L1 vs L2 设计原则 (chan-06)

Step 2 — 实现层（文档更新完成后启动）
  ui/src/app-v2/panel/SupervisorPanel.tsx    ← context mode (chan-03)
  ui/src/app-v2/dashboard/GlobalDashboard.tsx ← chan-03
  ui/src/app-v2/mock-data.ts                 ← 全局摘要数据 (chan-03)
  crates/seatloom-core/src/objects/session.rs ← 确认 sandbox/console 字段 (chan-04)
  rust/src/objects/playbook.rs               ← binding scope (chan-05)

Step 3 — 最终优先级排序重新确认
  待 Step 1+2 完成后，重新评估 Phase 2/3 交付顺序是否需要调整（chan-07）
```

---

## Done Definition

- [x] `architecture-decisions.md` 新增 AD-013（上下文层级，chan-03）— CLOSED 2026-05-07
- [x] `architecture-design.md` §7.1 supervisorStore 定义（chan-03）— CLOSED 2026-05-07
- [x] `SupervisorPanel.tsx` context mode 扩展实现（chan-03）— CLOSED 2026-05-07
- [x] `GlobalDashboard` 组件实现（chan-03）— CLOSED 2026-05-07
- [x] AD-013 v2 viewMode 正交状态（chan-09）— CLOSED 2026-05-08
- [x] `SupervisorPanel.tsx` viewMode 实现（chan-09）— CLOSED 2026-05-08
- [ ] `architecture-decisions.md` 新增 AD-AEGIS-02（Playbook binding scope，chan-05）
- [ ] `architecture-design.md` §3 Session Resources 字段归属更正（chan-04）
- [ ] `prd-v0.5.md` §3 优先级层新增 L1/L2 定义（chan-06）
- [ ] `playbook.rs` binding scope 数据结构更新（chan-05）
- [ ] Chan-07（Copilot Phase 2 优先级重评）得到明确结论

---

*Changes Register by Aegis · 2026-04-30 · Status: open — design stage ongoing*
