# Delivery: v2 Frontend Phase 1 (模块化) + Phase 2 (P0 数据视图)

**日期**: 2026-04-30  
**执行者**: Copilot  
**状态**: ✅ 交付完毕 — `tsc --noEmit` 零错误

---

## 背景

- **Phase 1** 任务来源：`docs/coordination/tasks/copilot/COPILOT-2026-04-30-frontend-modularization-v1.md`
- **Phase 2** 任务来源：Aegis 数据覆盖率审计报告 `docs/coordination/reviews/2026-04-30-v2-data-coverage-audit.md`

---

## Phase 1 — AppV2.tsx 模块化拆分

### 目标
将单文件 `AppV2.tsx`（2093 行）物理拆分成职责单一的模块，零行为变更，不引入新库。

### 执行结果

| 新建文件 | 职责 |
|----------|------|
| `ui/src/app-v2/types.ts` | v2 专属类型：ChatContact、ChatMessage、PlanPhase、TimelineEntry、ProjectChannelData |
| `ui/src/app-v2/mock-data.ts` | MOCK_CONTACTS、MOCK_MESSAGES、MOCK_CHANNEL_DATA |
| `ui/src/app-v2/components/Avatar.tsx` | SeatLoomLogo、StatusDot、Avatar |
| `ui/src/app-v2/components/ChatInput.tsx` | ChatInput（含路由治理逻辑） |
| `ui/src/app-v2/components/ContactRow.tsx` | ContactRow |
| `ui/src/app-v2/components/MessageBubble.tsx` | TYPE_BADGES + MessageBubble |
| `ui/src/app-v2/dashboard/BlockersSection.tsx` | 阻塞项面板 |
| `ui/src/app-v2/dashboard/ActiveWorkSection.tsx` | 活跃工作项面板 |
| `ui/src/app-v2/dashboard/TimelineSection.tsx` | 时间线/活动日志 |
| `ui/src/app-v2/dashboard/StageProgressSection.tsx` | DAG 阶段进度头部行 |
| `ui/src/app-v2/dashboard/GoalsSection.tsx` | 目标 + 阶段列表 |
| `ui/src/app-v2/dashboard/ProjectDashboard.tsx` | 项目频道完整看板（含 hover popup、数据投影、混合 mock+truth） |
| `ui/src/app-v2/panel/SupervisorPanel.tsx` | 可拖拽 IM 面板（localStorage 位置持久化） |

| 修改文件 | 变更 |
|----------|------|
| `ui/src/app-v2/AppV2.tsx` | 从 2093 行精简至 110 行，仅保留根组件 |

**不动文件**（任务文档要求）：`dag-model.ts`、`DagWorkflow.tsx`、`styles/tokens.css`

---

## Phase 2 — P0 数据覆盖视图

### 目标
补全 Aegis 审计发现的三个 P0 缺口：InboxView / WorkItemsView / ArtifactsView，集成进 ProjectDashboard tab 导航。

### 架构决策

**Props-down 模式**（非直接访问 store）  
P0 视图全部接收来自 ProjectDashboard 的数据切片作为 props，原因：store 的原有 `removeInboxItem` 依赖 `state.activeProjectId`（v1 全局概念），与 v2 的 `projectId` prop 不兼容。

**新增 store action**：`removeInboxItemFromProject(projectId: string, id: string)` — project-scoped 归档，解除全局 `activeProjectId` 耦合。

**Tab 状态重置**：`SupervisorPanel` 中的 `<ProjectDashboard>` 加了 `key={activeContactId}`，切换联系人时强制 remount 重置 tab。

**Tab 栏位置**：固定在 scroll 容器外部（`flexShrink: 0`），不随内容滚动。

### 执行结果

| 新建文件 | 职责 |
|----------|------|
| `ui/src/app-v2/views/InboxView.tsx` | InboxItem 决策队列；Critical→Normal→Low 排序；关联 Artifact badge；归档按钮 |
| `ui/src/app-v2/views/WorkItemsView.tsx` | WorkItem 全生命周期；状态筛选 tabs（进行中/待处理/阻塞/审阅中/已完成）；AC 预览；委派信息；审阅级别 |
| `ui/src/app-v2/views/ArtifactsView.tsx` | Artifact 文档库；T1-T7 Template 筛选 chips（仅显示数据中出现的模板）；按创建时间倒序 |

| 修改文件 | 变更 |
|----------|------|
| `ui/src/app-v2/dashboard/ProjectDashboard.tsx` | 新增 `activeTab` state；tab 导航栏（看板/待办/工作项/文档）；tab badge 计数；接收 store 数据后传 props 给各 view |
| `ui/src/app-v2/panel/SupervisorPanel.tsx` | `<ProjectDashboard key={activeContactId} .../>` |
| `ui/src/stores/useDataStore.ts` | 新增 `removeInboxItemFromProject(projectId, id)` action |

### WorkItem 状态映射（显式约定）

| Tab 标签 | 对应 WorkItem.status 值 |
|---------|------------------------|
| 进行中 | `Active`, `Reopened` |
| 待处理 | `Draft`, `Ready` |
| 阻塞 | `Blocked` |
| 审阅中 | `InReview` |
| 已完成 | `Done`, `Verified` |
| 全部 | 全部（含 `Drifted`） |

---

## 编译验证

```
cd ui && npx tsc --noEmit
# exit code 0，零错误
```

---

## 遗留 / 未启动项（Aegis P1+）

依据 Aegis 审计报告优先级排序，以下工作**未在本次执行**：

| 优先级 | 缺口 | 说明 |
|--------|------|------|
| P1 | SessionView | Session 对象无任何 v2 展示 |
| P1 | HandoffView / Handoff 集成 | Handoff 对象无 v2 展示 |
| P1 | Timeline 深度详情 | CanonicalEvent 的 EventType/evidence/payload 字段未深度展示 |
| P2 | Budget/SeatSkill/Pipeline/Review/Governance 视图 | 见 Aegis 审计报告 §P2 |
| P2 | T1/T2/T4/T6/T7 文档模板完整展示 | 目前 ArtifactsView 列表可见，但无详情展开 |
