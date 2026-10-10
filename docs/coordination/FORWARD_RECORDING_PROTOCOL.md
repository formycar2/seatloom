# Forward-Recording Protocol — 席位完成派单时的前向记忆记录(Lane B,D5 Phase 2)

**Owner:** Lyra (PO) · **Status:** ACTIVE — effective immediately. **Date:** 2026-10-10.
**Authority:** dispatch `disp-jwWHSX8Q2OeN66pnV3teY`(D5 Phase 2); design
`AEGIS-2026-10-10-washing-pipeline-design-v2.md`(张先生批准)。
**入口:** 统一走 MCP `remember` 工具(已交付、Flux 已验绿;typed 字段随 018 工具面升级到位)。
**禁止旁路:** 不得以文件代替记忆落库 —— 提炼产物一律落 SeatLoom,旁路文件不成事实源。

---

## 1. 为什么(O1/O2/O3 立刻生效,不必等 168MB 洗完)

验收目标:**O1** 派单 payload 不再重复内联已有事实,改为 recall 引用;**O2** 已记录的
failure 不再同根因复发;**O3** 张先生不再是系统最后一道记忆。前向记录让三条从**下一个
派单**就开始积累,清洗流水线(Lane C)只是回补历史。

## 2. 记录什么(每类 0–3 条,宁缺勿滥)

席位 complete 一个派单时,除交付说明外,用 `remember` 写本次真正产生的：

| kind | 记什么 | confidence 判据 |
|---|---|---|
| `decision` | 本次拍板的裁决(以及它约束什么) | 原话是你的 → quoted;你从证据推的 → inferred |
| `failure` | 什么坏了 + 根因 + 修法(实证:今天写入 memory 的多数是 failure 且全部复发过 —— 优先级最高) | 根因有实据(日志/测试输出)→ quoted;推测 → inferred |
| `unresolved` | 留给未来的开放问题/阻塞(谁会在哪再撞上) | 同上 |
| `preference` | **仅张先生纠正 Aegis 时**记(他说的话) | 恒 `quoted` |
| `artifact` | 产出的重要产物指针(路径 + 一句话为什么重要) | quoted |
| `fact` | 学到的稳定事实(环境/工具/契约的坑) | 视来源 |

Aegis 另有职责:解决跨席位升级后,把根因+修法记 `failure`。
`source_kind='canonical_event'`,`source_ref` 指向本次派单的生命周期事件
(SeatDispatchCompleted 的事件 id,或更具体的那条事件)。

## 3. 「宁缺勿滥」判据(三条全满足才记;存疑不记)

1. **会改变未来动作**:这条记录能让未来的某个决定/排障不同(约束性决策、有根因的失败、
   会被再撞上的开放问题)。纯粹的过程流水不记。
2. **不在别处逐字已有**:先 recall/查文档;派单 payload、交付说明、契约文档里已有的事实
   不重复记(O1 的整个目的就是消灭重复)。
3. **说得出证据**:能命名 L2 证据(对 archived_session 来源:文件 + 行号区间;对
   canonical_event 来源:事件 id)。说不出证据的记录不进库。

噪声是召回质量的税:每条都记 = 谁也别想 recall 到东西。

## 4. 写入门禁(随 018 工具面升级生效;此前按当前 remember 形状记,kind 进 tags 过渡)

- `kind`/`confidence` 必填(typed 词表);`source_kind='canonical_event'` +
  `source_ref=事件 id`;l2 行号区间在 file+archived_session 场景必填。
- 一次 complete 最多约 9 条(3 类 × 3);超量说明在记日记,不是在记记忆。

## 5. 各席位 AGENTS.md 增补(完成协议一节加入以下段落)

```markdown
### 前向记录(complete_dispatch 之前,D5 Phase 2 Lane B)

完成派单时,除交付说明外,按 `docs/coordination/FORWARD_RECORDING_PROTOCOL.md`
用 `remember` 记录本次真正产生的 decision/failure/unresolved(每类 0–3 条,宁缺勿滥:
会改变未来动作 + 别处没有 + 说得出证据,三条全满足才记;存疑不记)。
source_kind='canonical_event',source_ref 指向本派单生命周期事件;confidence 如实
(quoted=原话,inferred=推断)。张先生的纠正(preference)由 Aegis 记,恒 quoted。
入口只走 remember 工具,不得旁路写文件。
```

## 6. 生效与依赖

- **协议即时生效**(本单完成即写入各席位 AGENTS.md,成本近零)。
- typed 字段(kind/confidence/l2 行号)随 Lane A2(Nimbus,018 门事件后)生效;此前用当前
  remember 形状记录,kind 暂记 tags —— 018 工具面到位后的第一次记录起用完整形状
  (旧行不重写,confidence 回填 inferred 的既有纪律适用)。
