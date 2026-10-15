# Delivery — disp-supervisor-intent-ui-v2(Supervisor Layer 意图入口 · slice 2 界面)

**执行 actor(账本落款):** `seat:seat-mira-001` · **指令来源:** 张先生(经 Aegis 通道)· **Date:** 2026-10-13
**Dispatch:** `disp-supervisor-intent-ui-v2` · **契约:** `LYRA-2026-10-13-supervisor-intent-entry-contract-v1` slice 2(界面,对契约;live 工具接线是 slice 4)

---

## 结论

按契约建好意图入口的**界面层**:两模式入口 + 实测接线探针 + continuity-delta / decision-answer 两块(**接真读**)+ §3.3 「不猜」澄清态。结构化解析归 slice 1(Nimbus,未建),故结构化路径**停在一条实测澄清**,不猜方案、不造假旗。

## 依赖核验(先核门,再动手)

- 依赖边 `dep_value=ev-supervisor-intent-contract-landed`:**已在账本**(ArtifactCreated,2026-10-13T04:10:48,seat-lyra-001)—— 产品契约已落地,门满足。
- decompose dispatch 本身仍 `in_flight`,但依赖边挂在**事件**上、不挂 dispatch 状态,故交付正确。
- 读了契约全文(用户故事 / 交互流程 / 屏幕定义 / 拆解)与 brief,按 §3/§4 形状实现。

## 建了什么(文件)

| 文件 | 改动 |
|---|---|
| `ui/src/app-v2/panel/SupervisorIntent.tsx` | **新增**。两模式入口(派活/说句话,不混一次提交)+ 实测 capability 探针 + continuity-delta 卡 + decision-answer 卡 + §3.3 澄清卡(verb 不清 / owner 零匹配·多匹配·未给 / 对象不清 / 前提无法表达 / 无合格席位 / 工具不可用) |
| `ui/src/app-v2/panel/SupervisorIntent.test.tsx` | **新增**。8 组断言(见下) |
| `ui/src/app-v2/panel/SupervisorPanel.tsx` | **只增**:1 import + 1 挂载(`<SupervisorIntent onRawSend={handleSend}/>`,置于 chat pane 顶部、messages 前) |

## 接线诚实(契约 §4 / boundary 3 —  registered-flag 的坑)

- `ACCEPTANCE_CMDS_REGISTERED` 那种手编布尔**没有复用**。改为**实测探针**:挂载即调 `api.listSeats()`,据真实结果显示「后端可达(实测)」/「后端不可达(实测)」,实测错误放进 title。要么真测,要么不显示 —— 没有未测的接线断言。
- continuity-delta / decision-answer 绑**真命令**:`listAcceptances`/`listDispatches`/`listHandoffs`/`listEvents`/`listCheckpoints` 与 `getKnowledgeTree`/`getTopicMemories`。空块一律「为何空 + 下一步」,不留空框。

## 不猜(契约 §3.3 — 项目的病就是猜)

结构化「派活」的确定性解析是 slice 1 的工具、当前不可调;界面据此**停在一条实测澄清卡**(「deterministic intent parser not yet a callable command… 实测,非猜测」),并指路「说句话」或先用已接真读的两块。**没有**编造提议、**没有**假称「命令未上线」。§3.3 各歧义态在 UI 均为可达的一等卡片(带真实选项与理由),非 toast、非禁用态。

## 边界(契约 §6 / §8)

- **未碰** `ui/src/components/SupervisorCommandBar.tsx`(V1 原型、已脏、无产品收益)。
- **未碰快捷键层**:`⌘K` 冲突(V6)是 Aegis 未决问题,契约明令 slice 2 「must not touch the shortcut layer」—— 遵守,未动 `useGlobalShortcuts.ts` / `App.tsx` 键绑。
- 加法挂载,不重排骨架;`SupervisorPanel.tsx` 维持既有 lint 3 个、零新增(实测)。

## 门禁(全绿,我的文件零新增)

| 门禁 | 结果 |
|---|---|
| 类型 `tsc --noEmit` | ✅ 干净 |
| 测试 `vitest` | ✅ **67/67**(新增 8:实测探针双向、continuity 真读+空态、decision-answer 真读+空主题、不猜澄清态、两模式不混) |
| 构建 `vite build` | ✅ |
| 格式 `prettier`(两新文件) | ✅ |
| lint `eslint`(两新文件) | ✅ **0 error**;`SupervisorPanel.tsx` 维持既有 3、零新增 |

## 给张先生的三段

- **变了什么:** 监督面板里多了一块「意图入口」—— 可切「派活 / 说句话」;下面挂了「距上次以来的变化」和「为什么这么定(带出处)」两块,都接真实后端读;接线状态是实测的,不是写死的开关。
- **你会看到:** 打开监督面板(⌘K)即可见这两块,点「读取」拉真实数据;结构化「派活」目前会诚实告诉你解析器还没上线(实测),不会给你一个假提议。
- **需要你做什么:** 一件事留给你/Aegis 定 —— **⌘K 到底归意图入口还是别的**(现在它同时开监督 IM 和一个旧原型,是活的冲突)。在那个决定前,我没碰任何快捷键。其余无需你动;live 接线(slice 4)等 Nimbus 的意图工具上线。

## 收尾

`complete_dispatch(disp-supervisor-intent-ui-v2)` ✓。前向记录见 `remember`。

## 附:产物索引

- 提交:`d612b14`。
- 账本事件:见 `emit_event(ArtifactCreated)`。
- 本交付说明。
