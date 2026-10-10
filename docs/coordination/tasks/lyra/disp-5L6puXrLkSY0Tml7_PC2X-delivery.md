# Delivery — disp-5L6puXrLkSY0Tml7_PC2X(验收卡 closeout:同步 + containment 防线 + 6 方案建卡)

**Seat:** Lyra (PO) · **Dispatch:** `disp-5L6puXrLkSY0Tml7_PC2X` · **Date:** 2026-10-10
**Branch:** `packet/lyra/milestone-summary` @ `3a269e2`

## 做了什么

Aegis 阻塞裁决驱动完毕。两件事的处置:① 链条断点(5 条已验绿 lane 零卡)——**验收方案作者是
PO,6 份方案已全部产出**(shape v1,JSON 均校验合法);② 结构缺陷(接受按钮会删已合并工作)——
拆成「分支同步(数据事实)+ containment 防线(产品行为)」两步派发。

**产物(commit `3a269e2`,9 文档零代码):**

| 产物 | 路径 |
|---|---|
| closeout 拆解 | `docs/coordination/tasks/lyra/LYRA-2026-10-10-acceptance-cards-closeout.md` |
| 方案 1(Onyx 015) | `docs/coordination/acceptance/2026-10-10-lyra-onyx-015-knowledge-layer-acceptance-plan-v1.md` |
| 方案 2(Onyx 016) | `docs/coordination/acceptance/2026-10-10-lyra-onyx-016-plan-carrier-acceptance-plan-v1.md` |
| 方案 3(Onyx 017,surfaced) | `docs/coordination/acceptance/2026-10-10-lyra-onyx-017-cancel-widening-acceptance-plan-v1.md` |
| 方案 4(Nimbus 血缘) | `docs/coordination/acceptance/2026-10-10-lyra-nimbus-lineage-readmodel-acceptance-plan-v1.md` |
| 方案 5(Nimbus recall) | `docs/coordination/acceptance/2026-10-10-lyra-nimbus-d5-recall-acceptance-plan-v1.md` |
| 方案 6(Mira 卡片) | `docs/coordination/acceptance/2026-10-10-lyra-mira-card-ui-acceptance-plan-v1.md` |
| Onyx closeout packet | `docs/coordination/tasks/onyx/ONYX-2026-10-10-acceptance-closeout-v1.md` |
| Nimbus closeout packet | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-10-acceptance-closeout-v1.md` |
| Mira closeout packet | `docs/coordination/tasks/mira/MIRA-2026-10-10-acceptance-closeout-v1.md` |

**派发结果(enqueue 返回值即证据):**

| Dispatch | 席位 | 状态 | 内容 |
|---|---|---|---|
| `disp-acclose-onyx` | seat-onyx-001 | **ready**(p40) | 017 落地 + 015/016/017 三卡 |
| `disp-acclose-nimbus` | seat-nimbus-001 | **ready**(p35) | 两分支同步 + containment 防线 + 两卡 |
| `disp-acclose-mira` | seat-mira-001 | **ready**(p30) | 一卡(同步+验绿已就绪) |

## 现状实测(派单依据)

- Onyx 两分支、Mira 一分支**已含 track b6751a1** 且 Flux commit-pinned 验绿 —— 步骤 1 在其
  三侧已完成;Nimbus 两分支 stale(mb b84684a2)—— 步骤 1 残留派给他。
- containment 防线全栈未建(任何分支无 merge-base/删除校验)—— 步骤 2 派给 Nimbus,与他的
  base 校验同一处合并实现。
- 旧空卡结清是 Aegis 的动作(cancel_acceptance 已在二进制),本单只保证其 schema 前置被驱动。

## 两个 surfaced 缺口(已 emit SupervisorMessage 上报 Aegis,未自行其是)

1. **017 不存在**:cancel_acceptance 的 schema 支撑(acceptances status +cancelled、
   event_type +AcceptanceCancelled)在任何分支/迁移里都没有(Nimbus 仅在 .scratch 起草过),
   live 词汇表停在 25 —— **Aegis 作废旧卡会被 DB CHECK 拒**。已派 Onyx 落地 017(全量清单
   26 值,逐字取自 Nimbus 钉死的 Rust 枚举)+ 自请 Flux + 建卡。
2. **知识层无 topic 创建工具**:14 个 MCP 工具里没有;topics 表 live 0 行;proof 级 bootstrap
   走 psql。remember/recall 的「照着做」以 topic 存在为前提 —— recall 方案 precondition 如实
   写明,topic 规模 populated 归 Phase 2(Aegis);建议 Phase 2 含 topic 引导或未来 P1 开
   open_topic(需新事件类型,另一次契约加宽)。

## 怎么验证

- Lane 状态:上表(enqueue 返回值)。
- 6 份方案 JSON:python 解析 + 形状断言全过(plan_version=1 / scenario / steps[action] /
  expected[output,expect])。
- 分支事实:`git merge-base --is-ancestor track/infra-foundation <branch>` 逐分支实测
  (onyx×2、mira 含;nimbus×2 不含)。
- 26 值清单:与 Nimbus recall 分支 `EventType::ALL` 逐字比对一致。

## 边界确认

只做派单要求的事:拆解 + 6 方案 + 3 closeout packet + 3 dispatch。未合并任何分支、未写 live、
未重启服务、未替席位做实现(017 的 schema、防线、建卡调用都在对应席位车道)。旧空卡未碰
(Aegis 动作)。D5 两 lane 的建卡在本单框架内由各席位执行,未重复派单。

## 产物索引

- Commit `3a269e2`(9 文档)。
- Dispatch:`disp-acclose-onyx` / `disp-acclose-nimbus` / `disp-acclose-mira`(均 ready)。
- 账本事件:ArtifactCreated(6 方案 + 3 packet 捕获)、SupervisorMessage ×2(017 缺口、topic 接缝)。
