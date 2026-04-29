# Task: v2 前端数据覆盖率审计

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | LYRA-2026-04-30-v2-data-coverage-audit-v1 |
| status | issued |
| author | aegis |
| date | 2026-04-30 |
| to | lyra |
| priority | P0 |
| owner | Lyra |
| acceptance owner | Aegis |

## 背景

我们正在从零构建 SeatLoom v2 前端（`ui/src/app-v2/`）。目前 Supervisor IM 面板、频道页面、DAG 工作流等核心交互已初步成型，但使用的是 mock 数据。需要确认：

1. v2 前端的数据模型是否覆盖了产品需要的所有数据对象
2. 真实协作中产生的数据文件是否都有 UI 表达

## 任务

### 第一步：检查数据对象覆盖率

读取以下文件，梳理产品中涉及的所有数据对象（相当于 PostgreSQL 中的表/字段）：

- `docs/architecture-design.md` — 数据模型定义
- `docs/architecture-decisions.md` — AD-008 到 AD-012 的类型定义
- `ui/src/types/index.ts` — 当前 TypeScript 类型定义
- `crates/seatloom-core/src/types/` — Rust 端类型定义

然后对照 v2 前端的 mock 数据和组件：

- `ui/src/app-v2/AppV2.tsx` — ChatContact, ChatMessage, ProjectChannelData 等结构
- `ui/src/app-v2/dag-model.ts` — WorkNode, StageWorkflow 等结构

产出一份对照表：

| 数据对象 | 架构定义中存在 | v1 types 中存在 | v2 前端覆盖 | 缺失说明 |
|---|---|---|---|---|
| Seat | ✓ | ✓ | 部分（联系人列表） | 缺 Seat Card 详情 |
| Session | ✓ | ✓ | ？ | ？ |
| WorkItem | ✓ | ✓ | ？ | ？ |
| ... | | | | |

重点关注：
- Seat（身份、角色绑定、能力卡片、委派）
- Session（状态、运行时、checkpoint、prompt 状态）
- WorkItem（状态流转、Gate、审查 tier）
- Handoff（状态条、发送/接收/完成）
- Artifact（template+subtype 双键、元数据、Review Rail）
- CanonicalEvent（事件类型、审计链）
- InboxItem（优先级、动作类型）
- Pipeline / PipelineRun
- Checkpoint
- Playbook / SeatSkill
- 预算（Budget）
- 委派（Delegation overlay）

### 第二步：检查真实协作数据的 UI 表达覆盖

读取 `docs/coordination/` 下的真实协作产出物，按 template+subtype 分类统计：

```
docs/coordination/tasks/       → T3 (task, fix, verification)
docs/coordination/reviews/     → T4 (gap_review, benchmark, design_proposal, process_mapping)
docs/coordination/acceptance/  → T5 (acceptance_review, gate_decision)
docs/coordination/memory/      → T6 (daily_log)
docs/coordination/roles/       → T2 (seat_role)
docs/coordination/COORDINATION_RULES.md 等 → T7 (governance)
```

对照 `docs/coordination/DOCUMENT_TEMPLATES.md` 中定义的 template+subtype 全集，产出：

| Template | Subtype | 真实文件数量 | v2 前端有 UI 表达 | 缺失说明 |
|---|---|---|---|---|
| T3 | task | N 个 | ？ | ？ |
| T3 | verification | N 个 | ？ | ？ |
| T4 | design_proposal | N 个 | ？ | ？ |
| ... | | | | |

### 第三步：产出差距分析

基于以上两步，写一份差距分析，回答：

1. v2 前端的数据模型还缺哪些核心对象？
2. 哪些 template+subtype 的文档在真实协作中已经大量产出，但 v2 前端完全没有展示入口？
3. 哪些数据流（例如：WorkItem 创建 → 签发 → 交付 → 验证 → Gate 决策）在 UI 中还没有表达？
4. 建议的优先补全顺序是什么？

## 产出物

写入：
`docs/coordination/reviews/2026-04-30-v2-data-coverage-audit.md`

## Done Definition

- [ ] 数据对象覆盖对照表完成
- [ ] Template+subtype 覆盖对照表完成
- [ ] 差距分析和优先补全建议完成
- [ ] 产出物写入指定路径

## 约束

- 这是只读审计任务，不需要写代码
- 重点是发现差距，不是修复差距
- 如果某些数据对象在架构设计中定义但目前还不需要在 v2 前端展示，标注"P1/P2 延后"即可
