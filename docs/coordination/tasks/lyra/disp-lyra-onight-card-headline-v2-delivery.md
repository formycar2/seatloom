---
template: T5
subtype: delivery
doc_id: 2026-10-11-lyra-disp-lyra-onight-card-headline-v2-delivery
title: 交付说明 — 头条建卡步派给 Mira(替代被 ambiguous 门堵住的 v1)
status: active
author: lyra
doc_date: 2026-10-11
version: v1
depends_on: disp-lyra-onight-card-headline-v2, docs/coordination/acceptance/2026-10-11-lyra-mira-knowledge-panel-headline-acceptance-plan-v1.md
tags: delivery, lyra, headline, knowledge-panel, mira, acceptance-card
---

# 交付说明 — `disp-lyra-onight-card-headline-v2`

## 做了什么

按派单 payload 的唯一动作:向 `seat-mira-001` 投递头条建卡派单。

- **enqueued**: `disp-onight-panel-card`
  - title:`头条建卡:读模型+真实数据+面板验绿齐 → request_acceptance(血缘+结构化方案)`
  - target:`seat-mira-001`;priority 30;max_retries 1;**dependencies 无**(状态 `ready`,非 blocked)
  - payload.note:原样透传派单给定的 note(要求 Mira 立即 `request_acceptance`,并逐字引用
    `.../2026-10-11-lyra-mira-knowledge-panel-headline-acceptance-plan-v1.md` §2/§3 的
    acceptance_plan / how_to_verify;确认坏卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 已作废;
    验证构建必须是 Aegis 组装的集成构建;合并只发生在操作员点 Accept)。

## 为什么 v2 可以发(v1 为什么被堵)

v1 挂在门事件 `ev-typed-memory-tools-landed` 上,而该事件的 provenance 一直未核实。
v2 的依据是两条**已落地、有证据**的正向事件:

- `ev-knowledge-read-model-landed`(Nimbus 的读模型已交付)
- `ev-writein-landed`(Vela 的真实提炼记忆已入库 —— 这才是活跃内存路径的正向证据)

加上既有的 `ev-verdict-knowledge-panel`(Flux 对面板 PASS)。三个前提都在账本里,
所以这张卡落在**已交付、有证据的现实**上,而不是一个语义不清的门上。

## 我怎么验证的

1. 读了派单引用的验收方案文档 `.../acceptance-plan-v1.md`,确认 §2(acceptance_plan JSON)与
   §3(建卡参数:`dispatch_id` / `branch` / `base` / `how_to_verify` / `flux_verdict_ref`)存在且
   与派单 note 中的指示一致 —— 派单里让 Mira「逐字引用」的东西确实有出处。
2. `get_dispatch(disp-onight-panel)` 确认被建卡的目标 lane 存在、目标席位是 `seat-mira-001`、
   分支 `packet/mira/knowledge-panel`(note 中的 `base=track/infra-foundation` 来自验收方案 §3)。
3. `enqueue_dispatch` 返回 `enqueued disp-onight-panel-card (status=ready)` —— 无依赖,立即可拉。

## 账本

- `emit_event(ArtifactCreated)` → `ev-bqCCOC5xVOHwKxQlwxxZ2`(ref subject = `disp-onight-panel-card`,
  actor = seat-lyra-001, context = seat-mira-001),payload 里记了三条 evidence 事件 id。

## 需上报 / 需上游知道的

- **坏卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 的作废归属**:派单 note 已把「Mira 能作废就作废,
  不能(old binary)就 emit_event 上报、Aegis 拥有 backfill/void 路径」写进派给 Mira 的 payload。
  Lyra 不自行作废 —— 那是 Aegis 的 backfill/void 路径,不是 PO 随手可动的终态操作。
- **集成构建归 Aegis**:同样已在派给 Mira 的 payload 里写明(验证构建 = panel + read model + 真实数据,
  Aegis 负责组装,如 D2 bootstrap)。若操作员早上看到的面板没接上,Mira 会明说,不会让他点进空面板。
- **无 PO 本职之外的改动**:本次只有「投递一个派单 + 记账」,未触碰任何共享配置/服务/别的席位领域。

## 提交

- 分支:`packet/lyra/dispatch-ops`
- commit:见 git log(本交付说明 + 无代码改动)
