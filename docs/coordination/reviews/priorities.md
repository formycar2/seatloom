# SeatLoom Priority Principles — L1 / L2 Framework

| Field | Value |
|---|---|
| doc | priorities |
| scope | Unified priority judgment principles for SeatLoom v2 development |
| basis | prd-v0.5.md §3.1 · chan-06 insight · chan-08 value correction |
| status | issued |
| author | aegis |
| date | 2026-05-08 |

---

## 1. Core Insight (chan-06)

> Supervisor IM 是 L1 — 传统数据视图是 L2。

Mr. Zhang (2026-04-30) 确认：Supervisor IM（意图 → 结构化 Proposal → 即时确认 → 无需切换视图）是用户最高频交互路径。传统管理视图（Inbox/WorkItems/Artifacts/Sessions/Handoffs）的使用频率比 L1 低 10–30 倍。

---

## 2. Layer Definitions

| Layer | 频率 | 路径长度 | 代表性 surface | 设计原则 |
|---|---|---|---|---|
| **L1** | 最高频 | 最短路径（0–1 次切换） | Supervisor IM + GlobalDashboard + ProjectDashboard | 每个 sprint 优先加固 L1 |
| **L2** | 中频 | 需要切换视图（1–3 次导航） | InboxView, WorkItemsView, ArtifactsView, SessionView, HandoffView | L1 稳定后再建设 L2 |

---

## 3. Priority Ordering Rule

```
Within same P-tier:
  L1 items → implement first
  L2 items → implement after L1 stable

Cross-tier rule:
  P0-L1 > P0-L2 > P1-L1 > P1-L2 > P2
```

---

## 4. Application to v2 Architecture

| Component | Layer | Rationale |
|---|---|---|
| SupervisorPanel (chat mode) | L1 | 意图入口，最高频 |
| GlobalDashboard | L1 | 跨项目健康一览，启动点 |
| ProjectDashboard (overview tab) | L1 | 项目内健康视图 |
| InboxView | L2 | 决策队列，按需查看 |
| WorkItemsView | L2 | 状态机管理，按需查看 |
| ArtifactsView | L2 | 证据链检索，按需查看 |
| SessionsView | L2 | 运行时管理，低频 |
| HandoffView | L2 | 跨席位追踪，低频 |

---

## 5. Value Correction (chan-08)

| 之前的认知 | 修正后的理解 |
|---|---|
| P0 = 架构上承诺的功能 | P0 = L1 最高频 + 最短操作路径 |
| 所有 P0 模块同等重要 | P0-L1 (Supervisor IM) > P0-L2 (Inbox/Artifact) > P1 |
| 数据覆盖率 = 模块完整性 | 数据覆盖率 = **那个模块让 Supervisor IM 更强** |
| DAG/Blockers 是功能 | DAG/Blockers 是信息**密度**的展示，不是功能的完整实现 |

---

## 6. Practical Decision Guidance

当面对"下一步做什么"的决策时：

1. **先问**：这个工作是加固 L1 还是扩展 L2？
2. **如果是 L1**：直接执行，不需要论证。
3. **如果是 L2**：确认 L1 当前是否稳定（无 known gap/regression），否则优先修 L1。
4. **如果是跨层依赖**：L2 功能需要作为 L1 prerequisite 时，按 L1 优先级对待。

---

*Priority Principles by Aegis · 2026-05-08 · Basis: chan-06 insight + chan-08 value correction + Mr. Zhang confirmation*
