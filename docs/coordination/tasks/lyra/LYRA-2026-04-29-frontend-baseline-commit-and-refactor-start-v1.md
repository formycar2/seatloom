# Task: 前端基线提交 + 重构启动

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-04-29-frontend-baseline-commit-and-refactor-start-v1 |
| status | issued |
| author | aegis |
| date | 2026-04-29 |
| version | v1 |
| to | lyra |
| priority | P0 |
| deadline | 2026-04-29 |
| depends_on | `docs/PRODUCT_TRUTH.md`, `docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md` |
| tags | lyra, frontend, commit, refactor, design-direction |
| owner | Lyra |
| acceptance owner | Aegis / Mr. Zhang |

## 背景

Mr. Zhang 已与 Aegis 完成 UI/UX 设计方向对齐。核心结论：

1. 当前 Mira 产出的前端界面存在两个根本问题：**难看/易用性低** 和 **不支持移动端**。
2. Mr. Zhang 决定 **设计先行**——先把交互设计和页面样式做到能完全体现产品价值，再进一步做后端 implementation。
3. 新的设计方向文档已写入：`docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md`

## Objective

分两步执行：

### Step 1：提交当前代码基线

将当前工作目录中的所有变更（包括已接受的 UI 工作、Nimbus Rust 工程、协调文档更新等）作为一次 baseline commit 提交到 main 分支。

提交说明建议：`chore: baseline commit before frontend refactoring`

这次提交的目的是保存当前所有已接受的工作成果，确保后续重构有一个干净的回退点。

### Step 2：启动前端重构

基于新的设计方向文档 `docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md`，开始规划和执行前端重构。

重构的核心变化：
- 默认首屏从当前布局改为 **监督仪表盘**（数据驱动 dashboard + 工作流全景 + 行动队列）
- 导航从 full sidebar 改为 **48px 图标窄轨**
- 信息架构从"平铺所有信息"改为 **三层渐进式展开**（L1 扫描 / L2 决策 / L3 深度）
- 色彩系统按新文档约束（浅色画布 + 手术刀式彩色重点）
- 新增 **Mobile 端**支持（四 Tab 架构：概览/行动/活动/成果）

重构方式：Mira 的当前产出可以被推翻，按新设计方向重新实现。

## 设计方向文档

必须读取并作为执行约束：

- `docs/coordination/reviews/2026-04-29-ui-design-direction-v1.md`

## Done Definition

- [ ] 当前所有变更已提交为一个 baseline commit
- [ ] 重构计划已制定（可以是后续的 task packet 分拆）

## Non-Goals

- 不在本 packet 内完成全部重构实现
- 不修改已接受的 Nimbus Rust 工程代码
- 不修改产品合约文档（PRD/INT/UX/Acceptance）

## Constraints

- 提交时不要包含敏感文件（.env, credentials 等）
- 重构方向必须严格遵循设计方向文档，Mira 不允许自由发挥视觉方向
