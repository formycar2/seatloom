# v2 前端数据覆盖率审计报告

| Field | Value |
|-------|-------|
| doc | 2026-04-30-v2-data-coverage-audit |
| scope | `ui/src/app-v2/` 全部文件（含拆分后 15 个文件） |
| basis | `architecture-design.md` §3 · `architecture-decisions.md` AD-008–AD-012 · `ui/src/types/index.ts` · `crates/seatloom-core/src/objects/` |
| status | issued |
| author | aegis |
| priority | P0 |

---

## 一、数据对象覆盖对照表

### 1.1 架构层完整对象清单

```
架构来源：
  architecture-design.md §3  Rust structs（seat/session/workitem/artifact/handoff/checkpoint）
  architecture-decisions.md AD-008（ArtifactTemplate/Subtype）
                            AD-009（SeatDelegation）
                            AD-010（ReviewVerdictIssued/WorkItemRescoped event）
                            AD-011（CanonicalEvent）
                            AD-012（PromptState/PromptKind/PromptPolicy/PromptAction/AssistBudget）
  ui/src/types/index.ts       镜像 TS 类型
  crates/seatloom-core/src/objects/  Rust 源文件
```

### 1.2 逐对象审计结果

| 数据对象 | 架构定义 | v1 types 中有 | v2 前端覆盖 | 说明 |
|---|---|---|---|---|
| **Seat** | ✓ `SeatIdentity`+`ProjectRoleBind`+`SeatDelegation` | ✓ | ⚠️ 部分 | ChatContact 是 Identity+RoleBind 的 flatten；`SeatDelegation` overlay 无 v2 UI；seat_card / seat_skill 未实现 |
| **Session** | ✓ Session+PromptState+Runtime+SessionStatus | ✓ | ✗ 无 UI | v2 无 Session 管理视图。v1 plan 中有 SessionView，v2 未实现 |
| **WorkItem** | ✓ Status/Priority/AC/DependsOn/Parent/ReviewChangeRecord/DelegationRecord | ✓ | ⚠️ 部分 | DAG node 展示 workItemRef ID，但不展开 WorkItem 完整字段；无 WorkItem 管理视图 |
| **Artifact** | ✓ Template/Subtype/SubtypeValid/SystemKind | ✓ | ✗ 无 UI | ArtifactsView 在 v1 plan 中，v2 未复刻；DAG node 中 artifactRefs 存在但无展示逻辑 |
| **Handoff** | ✓ Status含 Working/Expired | ✓ | ✗ 无 UI | v1 types 全量字段；v2 无 Handoff 管理视图 |
| **CanonicalEvent** | ✓ EventType 全量 + ObjectRef + Payload | ✓ | ⚠️ 部分 | v2 TimelineSection 展示 flat 文本条目，不展示 EventType 全量枚举、evidence_refs、payload |
| **InboxItem** | ✓ Priority/Type/Actor/ObjectRef | ✓ | ✗ 无 UI | v1 types 全量字段；v2 未实现 Inbox 视图 |
| **Pipeline / PipelineRun** | ✓ Pipeline 定义 + runs | ✓ | ✗ 无 UI | v2 无 Pipeline 运行视图 |
| **Checkpoint** | ✓ CheckpointTrigger/SummaryQuality | ✓ | ✗ 无 UI | v2 mock 数据中有引用，无独立 UI 视图 |
| **Playbook / SeatSkill** | ✓ `seat_skills`/`playbook_matches` | ✓ | ✗ 无 UI | PlaybookView 在 v1 plan 中，v2 未复刻 |
| **Budget** | ✓ PackEngine Budget + AssistBudget | ✓ | ✗ 无 UI | 无预算/配额可视化 |
| **Delegation overlay** | ✓ SeatDelegation 三层分离 | ✓ | ✗ 无 UI | ChatContact 无 delegation 字段；无 Delegation 管理视图 |
| **ReviewChangeRecord** | ✓ Tier/Reason/Impact/AckMode | ✓ | ⚠️ 部分 | ActiveWorkSection 有 reviewTier 展示，ReviewChangeRecord 完整结构无 UI |
| **PromptState** (AD-012) | ✓ PromptKind/Policy/Actions/AssistBudget | ✓ | ✗ 无 UI | v2 未处理 SessionStatus.PromptBlocked；无 prompt assist 机制 |

---

## 二、Template+Subtype 覆盖对照表

### 2.1 真实文件统计（按 DOCUMENT_TEMPLATES.md 分类）

数据来源：`docs/coordination/` 全部 `.md` 文件（不包含 `.gemini/` 目录）。

| Template | Subtype | 真实文件数量（约） | 文件目录 |
|---|---|---|---|
| T1 Authority Doc | `prd` | 2 | `docs/prd-*.md` |
| T1 | `architecture_design` | 1 | `docs/architecture-design*.md` |
| T1 | `architecture_decisions` | 1 | `docs/architecture-decisions*.md` |
| T2 Role Profile | `seat_role` | 4 | `docs/coordination/roles/` |
| T3 Task Packet | `task` | ~140 | `docs/coordination/tasks/` |
| T3 | `fix` | ~8 | `docs/coordination/tasks/` |
| T3 | `verification` | ~10 | `docs/coordination/tasks/` |
| T4 Review | `gap_review` | ~8 | `docs/coordination/reviews/` |
| T4 | `benchmark` | ~2 | `docs/coordination/reviews/` |
| T4 | `process_mapping` | ~2 | `docs/coordination/reviews/` |
| T4 | `design_proposal` | ~12 | `docs/coordination/reviews/` |
| T5 Acceptance | `acceptance_review` | ~28 | `docs/coordination/acceptance/` |
| T5 | `gate_decision` | ~12 | `docs/coordination/acceptance/` |
| T6 Daily Memory | `daily_log` | 3 | `docs/coordination/memory/` |
| T7 Governance Doc | `coordination_rules` 等 4 种 | 4 | `docs/COORDINATION_RULES.md` 等 |
| **合计** | | **~220** | |

### 2.2 v2 前端模板家族展示状态

| Template | Subtype（允许列表） | v2 前端有 UI 表达 | 覆盖状态 |
|---|---|---|---|
| T3 Task Packet | `task` / `fix` / `verification` | ✓（SupervisorPanel 消息卡片 + DAG workItemRef） | **已覆盖（基础）** |
| T5 Acceptance | `acceptance_review` / `gate_decision` | ✓（gate/verification 卡片展示于消息气泡） | **已覆盖（基础）** |
| T1 Authority Doc | 6 种 | ✗ | **缺失** |
| T2 Role Profile | `seat_role` | ✗ | **缺失** |
| T4 Review | 4 种 | ✗ | **缺失** |
| T6 Daily Memory | `daily_log` | ✗ | **缺失** |
| T7 Governance Doc | 4 种 | ✗ | **缺失** |

> 共 5 个 Template 家族、17 个 Subtype 在 v2 中无展示入口。

---

## 三、差距总结

### 3.1 v2 数据模型缺口（按 P0→P2）

| 优先级 | 数据对象 | 缺口描述 |
|---|---|---|
| P0 | **InboxItem** | 决策队列无独立视图 |
| P0 | **WorkItem 完整字段** | DAG 仅展示 workItemRef ID；无 WorkItem 管理视图 |
| P0 | **Artifact 管理视图** | 证据链无独立入口；DAG artifactRefs 只展示 ID |
| P1 | **Session 管理视图** | 工作流部署状态不可见；PromptBlocked 无处理层 |
| P1 | **Handoff 追踪视图** | 跨席位委派链路不可追溯 |
| P1 | **CanonicalEvent 全量** | Timeline flat 展示，无 EventType/evidence/payload 深度视图 |
| P1 | **Seat + Delegation Overlay** | Seat 卡片未展示能力标签；Delegation 状态不可见 |
| P2 | **Budget / SeatSkill / Playbook** | 无预算/配额/能力可视化 |
| P2 | **Pipeline / PipelineRun** | Pipeline 运行指标无展示窗口 |
| P3 | **ReviewThread / ReviewComment** | Artifact review rail 未实现 |

### 3.2 数据流缺口

| 数据流 | v2 当前状态 | 缺口 |
|---|---|---|
| WorkItem → 签发 → 交付 → 验证 → Gate 决策 | 仅 Gate 卡片静态展示于 IM 气泡 | 无独立 WorkItem 卡片视图；无法操作流转 |
| Artifact → Review Rail → 证据上链 | DAG artifactRefs 存在但仅 ID | Artifact 元数据视图缺失；证据链不可追溯 |
| Handoff → 跨席位委派状态条 | 无 | 完整 Handoff tracker 类型存在但无 UI |
| Session → Running → PromptBlocked → Checkpoint | 无 | 完整 Session 管理视图缺失 |
| Reconcile → PostgreSQL 变更检测 | 无 | 运行期数据同步无可观测 UI |

### 3.3 真实协作数据与 v2 展示对比

当前 v2 仅处理 IM 消息气泡和项目频道卡片：
- ✅ **T3 task/verification + T5 gate_decision** — IM 气泡有基础卡片展示
- ⚠️ 约 **220 个真实文档** 在生产中产出，其中：
  - T1/T2/T4/T6/T7 共 ~58 个文件完全无 v2 展示入口
  - T3 中 work 任务的 ~158 个文件仅通过 IM 气泡得到碎片化展示

---

## 四、优先补全顺序

> **L1/L2 分层修正（2026-05-08, chan-06/chan-08）**：原始排序按"模块完整性"视角。修正后按 L1→L2 原则重新标注。L1 surface 加固优先于 L2 独立视图新建。

### Phase 0（L1 加固，已交付）

```
✅ chan-03  GlobalDashboard（跨项目健康摘要）         [L1]
✅ chan-09  viewMode 正交状态 + 面包屑重构            [L1]
```

### Phase 1（v2 P0，补全数据入口）— L2

```
顺序 ①  InboxView（InboxItem 决策队列）              [L2]
        ②  WorkItemsView（WorkItem 全生命周期视图）   [L2]
            ③  ArtifactsView（Artifact+模板筛选+证据链）[L2]
        ④  ArtifactDetail（全字段详情，含 T1-T7 家族展示）[L2]
```

**理由**：Inbox → WorkItem → Artifact 是 SeatLoom 业务核心链路（路由 → 调度 → 交付 → 证据），任意一段缺失都导致用户无法在 v2 中执行完整工作流。**但这些都是 L2 视图，在 L1 稳定后再建设。**

### Phase 2（P1，连通性）— L2

```
顺序 ⑤  SessionsView（Session 状态面板，含 PromptBlocked 入口）[L2]
        ⑥  Handoff → 集成进 WorkItemDetail                     [L2]
        ⑦  Timeline → 深度详情 Modal（EventType+evidence+payload）[L2]
           ⑧  Seat Card 完善（Delegation overlay）               [L2]
```

**理由**：Session 和 Handoff 补全后，工作流五项核心迁移（脉）的 UI 链路才能闭合。CanonicalEvent 深度视图依赖于 SessionView 和 ArtifactView 同时存在。

### Phase 3（P2，体验增强）— L2

```
顺序 ⑨  Budget / SeatSkill / Playbook 视图    [L2]
        ⑩  Pipeline / PipelineRun 视图         [L2]
        ⑪  T4 Review / T6 Daily Memory 聚合视图 [L2]
        ⑫  T7 Governance Doc 展示入口           [L2]
```

**理由**：难度相对低，无状态流转影响，可作为插补项渐进完成。

---

*Audited by Aegis · 2026-04-30*
