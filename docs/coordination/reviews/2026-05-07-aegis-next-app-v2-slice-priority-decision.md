# Aegis Priority Decision: Next app-v2 Slice after Phase 2 + Slice B

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | AEGIS-2026-05-07-next-app-v2-slice-priority-decision-v1 |
| status | issued |
| author | aegis |
| date | 2026-05-07 |
| version | v1 |
| basis | `docs/coordination/reviews/2026-04-30-supervisor-im-as-l1-insight.md`; `docs/coordination/tasks/aegis/AEGIS-2026-04-30-pending-changes-register.md` (chan-03, chan-06, chan-07); `docs/coordination/reviews/2026-04-30-v2-data-coverage-audit.md`; `docs/coordination/acceptance/2026-05-07-lyra-app-v2-phase2-slice-b-acceptance.md` |
| tags | decision, priority, app-v2, L1-L2, slice-planning |

## Question from Lyra

Phase 2 + Slice B 已 PASS 验收（commit `cb06ce0`）。P0 数据覆盖审计中仍未覆盖的 UI 视图：
Session / Checkpoint / Handoff / Pipeline / Delegation overlay / PromptState runtime。下一个 slice 的优先级和归属？

## Aegis Decision

**Pause the L2 data-view coverage push. Pivot to L1 Supervisor surface strengthening.**

### Next slice: `chan-03` Supervisor Context Mode (GlobalDashboard + Project separation)

| Field | Value |
|---|---|
| surface class | **L1** (Supervisor IM context) |
| ownership (design) | Aegis (write AD-AEGIS-01 first) |
| ownership (implementation) | **Nimbus** (cross-track assignment: Copilot unavailable per Mr. Zhang 2026-05-07; Nimbus takes UI/TS implementation despite main track being Rust/Postgres) |
| verification | Flux (verify-only, same model as Phase 2 slice-b verification) |
| acceptance | Lyra |
| priority | P0 |

## Why NOT any of the remaining L2 views

Per `supervisor-im-as-l1-insight.md` and chan-06:

> **L1 是用户实际操作的频率最高的入口，不是架构上 P0 的功能。**
> 传统数据视图（Inbox/WorkItems/Artifacts/Session/Handoff）是 L2 — 必要但不紧急，
> 使用频率比 L1 低 10-30 倍。

Phase 2 已经为 L2 交付了 3 个视图（Inbox/WorkItems/Artifacts）。Chan-07 明确标注了对此的质疑：
> "Phase 2 把 P0 资源投入传统 L2 视图，而非继续加固 L1 Supervisor IM"

继续直接拉下一个 L2 slice（Session/Handoff/Checkpoint 等）会与 chan-06 的优先级原则冲突，
且 chan-07 的质疑尚未通过一次 L1 加固得到对冲。

### L2 剩余视图的处置

| 对象 | 降级原因 | 触发升级的条件 |
|---|---|---|
| Session detail view | Session 的绝大多数 context 已通过 Supervisor IM 消息展示 | 当 Runtime / SandboxConfig 成为频繁决策点时（目前不是） |
| Handoff view | Handoff 的关键状态已在 Supervisor IM 消息卡片中展示（`type: 'delivery' / 'verification' / 'gate'`） | 当出现跨 Seat 的 Handoff 查找场景并产生实际痛点时 |
| Checkpoint view | Checkpoint 是历史快照，非 daily decision surface | 当需要回滚或审计特定 checkpoint 时再单独建 |
| Pipeline view | 目前没有实际 Pipeline 运行（属于 infra readiness 范畴） | 当 CI/CD 真正接入并需要可视化时 |
| Delegation overlay | ChatContact 已包含 seatType（po/worker/verifier）的路由信息 | 当出现三层分离带来的实际 UI 混淆时 |
| PromptState runtime | SessionStatus 尚未支持 `PromptBlocked`，后端也未实现 PromptKind / Policy / Actions | 当后端 prompt 机制上线后（目前属于 AD-012 的未来范围） |

这不等于"永不做"，而是：**不主动拉 slice，等待具体 pain signal 触发。**

## Why chan-03 (GlobalDashboard) is the right L1 pick

chan-03 是 pending changes register 中 **已确认共识**、**尚未实现**、**对 L1 有直接正反馈** 的唯一项：

1. **真实缺口**：Supervisor 目前只能看单个 project context，无法 cross-project 看健康度。
   Aegis 的实际工作流需要"所有 project 一览 → 下钻到某个 project"的双层切换。

2. **Mr. Zhang 已描述清楚**：
   - Global context：跨项目全局摘要，各 project 状态一览
   - Project context：某个 project 内部细节视图，不混入其他 project context

3. **实现工作量可控**：
   - 1 个新组件 `GlobalDashboard`
   - `SupervisorPanel` 增加 `currentContextMode: 'global' | 'project'` + `activeProjectId?`
   - mock-data 增加全局摘要数据源
   - `ProjectDashboard` 区分 context 路径（已存在 projectId prop，只需确保 global 路径不触发它）

4. **与 Phase 1/2 模式一致**：结构化扩展，零需求歧义，适合单 slice 完成。Nimbus 跨轨需额外结构化引导（见 Step 2）。

## Execution sequence

**Step 1 — Aegis (设计先于实现)**
产出 `AD-AEGIS-01: Supervisor 两层上下文模型` 作为架构决策记录，写入 `docs/architecture-decisions.md`。
同步更新 `architecture-design.md §6.1 Seat store` 中对 Person-Supervisor 绑定和 context mode 的描述。

**Step 2 — Nimbus (实现)**
Aegis 发包 `NIMBUS-2026-05-07-app-v2-supervisor-context-mode-v1`（路径：`docs/coordination/tasks/nimbus/`），明确：
- 新文件 `dashboard/GlobalDashboard.tsx`
- `SupervisorPanel.tsx` 增加 context mode state + global/project 切换入口
- `mock-data.ts` 增加 `MOCK_GLOBAL_SUMMARY`
- `types.ts` 增加 `SupervisorContextMode` 类型
- 零回归：所有 Phase 1/2 已交付视图在 project mode 下保持不变

**Nimbus 跨轨提示**（Nimbus 主轨为 Rust/Postgres infra，本次跨入 UI/TypeScript）：
- 任务包需要附上 Phase 1 的 14 个模块结构参考（文件列表 + 各自职责一行）
- 任务包需要附上 `GlobalDashboard.tsx` 与 `ProjectDashboard.tsx` 的对照点（context mode 切换路径、`projectId` prop 的来源与传递）
- 验收门控明确：`cd ui && npx tsc --noEmit` 零错误 + `pnpm build` PASS + 浏览器 smoke test（切换 global ↔ project 无回归）

**Step 3 — Flux (verify-only)**
同 Phase 2 + Slice B 模式：commit-pinned，`tsc --noEmit` + `pnpm build` + 浏览器 smoke test + 回归检查。

**Step 4 — Lyra (验收)**
关闭 chan-03 in AEGIS-2026-04-30-pending-changes-register。

## Concurrency rule

此 slice 进行期间：
- **不开新的 L2 slice**（Session / Handoff / Checkpoint 等一律暂缓）
- **不开新的后端 slice**（LYRA-flux-prompt-channel-authority-remote-verification 继续走既定链路，独立于此决策）
- 仅允许的例外：**具体用户 pain signal** 触发的紧急修复（需要 Lyra 确认 pain 的真实性）

## Open question to Lyra

本决策基于 chan-06 / chan-07 的 L1/L2 框架。如果 Lyra 或 Mr. Zhang 认为该框架需要修正
（例如 chan-07 得到"继续 L2 也可以"的结论），请先关闭该上游问题再开新 slice。

---

*Decision by Aegis · 2026-05-07 · Status: issued — awaits Lyra confirmation before Step 1 execution*
