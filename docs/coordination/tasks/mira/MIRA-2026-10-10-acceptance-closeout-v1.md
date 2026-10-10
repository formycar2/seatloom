# MIRA-2026-10-10-acceptance-closeout-v1 — 卡片改版 lane 建卡收尾

**Owner:** Mira (seat-mira-001) · **Issuer:** Lyra (seat-lyra-001) · **Date:** 2026-10-10
**Dispatch:** `disp-acclose-mira` · **Milestone:** 验收卡 closeout(Aegis 阻塞裁决驱动)
**Base branch:** `track/infra-foundation`(@ b6751a1)· **Delivery branch:** `packet/mira/acceptance-lineage-ui`(已在)
**Done-definition:** §2(一卡)建出 + §3 收尾走完。

---

## 0. 定位

Aegis 阻塞裁决:5 条已验绿 lane 没有变成验收卡(断点在 PO 已补:验收方案由 Lyra 供稿)。
**你的分支已就绪**:`packet/mira/acceptance-lineage-ui` @ `14e2fd6` **含 track b6751a1**,
Flux 已 commit-pinned 验绿(7e28b2e)——步骤 1(同步)在你侧已完成。本单只剩:用 Lyra 供稿的
验收方案把你的卡建出来。

## 1. 现状(实测,不必重查)

- 分支 `packet/mira/acceptance-lineage-ui` @ 14e2fd6(含 track b6751a1);门禁与 UI 测试随
  D2 既有 + 本 lane 新增全绿(你的交付文档)。
- Flux PASS,commit-pinned 14e2fd6/f7ad270(判定事件 id 在你的 Flux 交付文档里)。
- 若期间 track 又前进(b6751a1 之后有新 commit):把 track 合进来重跑门禁 + 请 Flux 重验,
  再建卡(未同步不许建卡,Aegis 裁决;冲突解律:绝不删除 track 已追踪文件)。

## 2. 建卡(方案由 Lyra 供稿,你执行调用)

```
request_acceptance(dispatch_id="disp-accplan-lane-a-mira",
  branch="packet/mira/acceptance-lineage-ui", base="track/infra-foundation",
  how_to_verify="待验收卡默认呈现里程碑/lane+任务、下单人+时间、验绿徽章、方案场景+步骤数+首条预期;展开含原始 payload/事件链/产物入口/完整方案 —— 不翻代码即可判断价值",
  flux_verdict_ref=<Flux 对 14e2fd6 的 ReviewVerdictIssued 事件 id>,
  acceptance_plan=<docs/coordination/acceptance/2026-10-10-lyra-mira-card-ui-acceptance-plan-v1.md §2 JSON 逐字>)
```

- base 用真实分支名 **track/infra-foundation**(旧卡 base='track' 的记录错误已由 Nimbus
  缺陷单修,别再写 'track')。
- 若 Nimbus 的 containment 防线已部署:你的分支含 base、合并不删 base 文件,应正常通过 ——
  若被拒,把拒绝信息原样上报,不要改弱任何东西去迁就。

## 3. 收尾流程

1. 确认分支状态(§1);有变动则同步 + 门禁 + Flux 重验。
2. 按 §2 建卡。
3. 写交付说明 `docs/coordination/tasks/mira/MIRA-2026-10-10-acceptance-closeout-delivery-v1.md`
   (分支含 track 的 rev-parse 证据、acceptance_id、plan 来源文档)。
4. `complete_dispatch(dispatch_id="disp-acclose-mira", outcome="done")`。
5. **席位永不自己合并**;卡由张先生在 UI 接受后自动 `--no-ff` 合并。

## 4. 红线

- 验收方案逐字入参(shape v1),不改字段不改字;方案作者是 PO,你只渲染/传递。
- 无 mock 顶替真实后端值(house rule);不整分支合并;事件驱动零轮询。
