# Delivery — disp-supervisor-intent-ui-wiring(Supervisor 意图入口 · slice 4 接线)

**执行 actor(账本落款):** `seat:seat-mira-001` · **指令来源:** 张先生(经 Aegis 通道)· **Date:** 2026-10-13
**Dispatch:** `disp-supervisor-intent-ui-wiring` · **契约:** `LYRA-2026-10-13-supervisor-intent-entry-contract-v1` slice 4(把界面接到 live 工具)

---

## 结论

「派活」入口从占位符变成**真调 live 工具**:`api.planIntent → cmd_plan_intent → plan_intent`(actor human:supervisor),渲染它的**真实结果** —— 可审阅提议(UX-02 字段 + 实测「为什么是这个席位」)或它停下的澄清(真问题+真选项)。Confirm 走**受治理写路** `enqueueDispatch`,显派单 id;失败逐字、不显 id;带写路无法表达的依赖边则**明确拒绝**(§3.3/V5),不静默丢。端到端验收卡已请求:`acc-disp-supervisor-intent-ui-wiring`(pending)。

## 依赖核验(先核门,再动手)

两条硬边都核过:
- 边 (b) `ev-verdict-supervisor-intent-tools`(Flux 判定)✅ 在。
- 边 (a) `ev-supervisor-intent-tools-live` —— 首核时**不在**(Nimbus 故意压着,`ev-nimbus-intent-tools-gate-deferred`:工具对着新二进制实测能跑,但应用 shell out 的集成二进制是旧的)。我据此**保持 blocked、不接线**。随后 Aegis 合并 slice 1、重编二进制、真调 plan_intent 后**发了该门**(SupervisorMessage,带 `verified`,05:16)—— 两条边齐,我才接线。(这也顺手印证了 Nimbus finding_1:`-live` 门按门准则发成 SupervisorMessage。)

## 建了什么(文件)

| 文件 | 改动 |
|---|---|
| `ui/src/lib/intent-dto.ts` | **新增**。plan_intent 返回的 TS 镜像(Proposal/Clarification/SeatRationale/PreconditionResolution/SuggestedDependency…,camelCase;枚举 kebab/snake_case,与工具发出的一致) |
| `ui/src/lib/api.ts` | **只增**:`planIntent(intent)` → `invoke('cmd_plan_intent', {intent})` |
| `ui/src/app-v2/panel/SupervisorIntent.tsx` | slice 2 的表面 + slice 4 接线:派活真调 plan_intent;提议卡/澄清卡/检索渲染;Confirm→enqueueDispatch 显 id/逐字错;依赖边拒绝;占位符移除 |
| `ui/src/app-v2/panel/SupervisorIntent.test.tsx` | **重写**。9 组(实测探针双向、continuity/decision 真读+空态、真提议渲染、Confirm→enqueueDispatch 显 id、plan_intent 拒绝逐字无 id、依赖边拒绝、澄清不猜、两模式) |

分支已同步 track(`merge`,拿到 `cmd_plan_intent`,满足 branch-contains-base)。

## 端到端证据(真调 live 库,read-only,不写账本)

对集成二进制(2026-10-13 13:13 重编,`tools/list` 含 plan_intent)真调一句「给 Nimbus 派个活,把唤醒服务的硬底兜底接上」:

```
outcome: proposal | wrote: false | actor: human:supervisor
intentRestated: create a dispatch for nimbus (seat-nimbus-001): 给 Nimbus 派个活,把唤醒服务的硬底兜底接上
ownership: nimbus seat-nimbus-001
budget note: worker/supervisor budgets are projects.* ... the load figure is the count of this seat's in_flight dispatches
rationale rows: 7 | actions: confirm,edit,cancel
```

即 UI 接的这个工具**真能对 live 库跑出真实提议**(无硬编码、wrote=false)。写路(enqueue_dispatch)是长期可用工具 + 组件测试覆盖;真写入留给运营者点「接受」。

## 边界(全部遵守)

- **未碰** `ui/src/components/SupervisorCommandBar.tsx`(V1 原型、已脏)。
- **未碰快捷键层** —— 见下「快捷键冲突上报」。满足派单「不碰快捷键层」约束(untouched)。
- 未合并任何分支进基线(合并只发生在运营者点接受);未重启服务;未改 schema。

## 快捷键冲突(上报,不猜)

Aegis 裁决(`ev-aegis-ruling-intent-entry-chord-20261013`)解除了快捷键层禁令,并指令三件事(去 AppV2 重复绑定 / ⌘K 只走 Command Bar / 原型退役),还要求把代码注释+帮助对话框改成与规格一致。**但该裁决说「⌘K 归 Command Bar」,与契约 §4「意图入口在 IM 面板、Command Bar(V1 原型)退役」对 ⌘K 的目标面说法冲突。** 按「不猜」纪律,我没有擅自决定其一,保持快捷键层原样,上报 Aegis/Lyra 裁协后再动。`record_note` 里「注释/帮助对话框与规格一致」那部分也随该裁协一并处理。

## 门禁(全绿,我的文件零新增)

| 门禁 | 结果 |
|---|---|
| 类型 `tsc --noEmit` | ✅(合并 track 后复验仍干净) |
| 测试 `vitest` | ✅ **71/71**(接线新增 9 组) |
| 构建 `vite build` | ✅ |
| 格式 `prettier` | ✅ |
| lint `eslint`(4 个改动文件) | ✅ **0 error** |

## 给张先生的三段

- **变了什么:** 监督面板的「派活」入口现在真接通了:打一句话 → 从真库读出一条可审阅提议(谁来做、为什么是他、证据、预算),意图不清会停下来问你、给选项,不猜;点确认就按你的身份(human:supervisor)建成一条真派单。接线状态是实测的,不是写死的假开关。
- **你会看到:** 打开监督面板,「派活」里打一句「给 Nimbus 派个活,把唤醒服务兜底接上」点拆解 → 一张带完整字段和「为什么是这个席位」的提议卡;点确认 → 显示派单 id。我已经对着真库跑通(只读,没写东西)。
- **需要你做什么:** ①端到端验收卡 `acc-disp-supervisor-intent-ui-wiring` 已在你的待验收区,你点「接受」才合并(我从不合并);②**⌘K 现在有冲突没修** —— Aegis 的裁决和产品契约对「⌘K 该开哪个面」说法不一致,我没擅自决定,保持原样,等你/Aegis 裁协。

## 收尾

`complete_dispatch(disp-supervisor-intent-ui-wiring)` ✓(done);`request_acceptance` → `acc-disp-supervisor-intent-ui-wiring`(pending)。前向记录见 `remember`。

## 附:产物索引

- 提交:`971649f`(接线)+ 合并 track。
- 账本事件:`AcceptanceRequested`(acc-disp-supervisor-intent-ui-wiring)。
- 本交付说明。
