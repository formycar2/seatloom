# Delivery — disp-onight-panel-card(头条建卡:知识面板 headline 验收卡)

**执行 actor(账本落款):** `seat:seat-mira-001` · **指令来源:** 张先生(经 Aegis 通道)· **Date:** 2026-10-11
**Dispatch:** `disp-onight-panel-card`(Lyra → Mira)· **目标:** 为 `disp-onight-panel` 建 headline 验收卡

---

## 结论

头条验收卡 **`acc-disp-onight-panel`** 已创建,`status=pending`,待运营者判定。用 **Lyra 亲笔 §2 方案逐字**入参,血缘完整。这是「张先生醒来点的那一张」。

## 过程(按 guard 逐条核 + 一次分支同步)

1. **前提核验(只读)**:`disp-onight-panel`=done 且归属 mira ✓;base=`track/infra-foundation` 是真本地分支 ✓;`flux_verdict_ref=ev-verdict-knowledge-panel` 在账本 ✓;无既有在途验收卡(`acc-disp-onight-panel` not found)✓;read-model 判定 `ev-verdict-knowledge-readmodel` 在账本 + 知识树 25 个真实 topic(且本会话 `remember` 真写成功)→「读模型+真实数据」实质满足 ✓。
2. **方案定位**:派单逐字引用的 `docs/coordination/acceptance/2026-10-11-lyra-mira-knowledge-panel-headline-acceptance-plan-v1.md` **不在当前分支/基线磁盘上**;`git log --all` 命中它在 `packet/lyra/milestone-summary @ 2ecaf57`(Lyra 亲笔,2026-10-11 02:34)。取 §2(shape-v1:plan 1 / preconditions 3 / steps 8 / expected 8)逐字入参 —— 是真方案,非伪造。
3. **第一次 `request_acceptance` 被 guard 正确拒**:「branch packet/mira/knowledge-panel does not contain base track/infra-foundation」—— 面板分支基线更早(85bcc06),基线之后前进了 18 个提交(含 read-model)。按 AGENTS.md 第 6 条教训,这是正确拦截。
4. **同步(修法)**:`git merge track/infra-foundation` 合入面板分支(干净、无冲突:面板新文件在基线独有,`ProjectDashboard.tsx` 基线未动,`lib/api.ts` 仅基线改过)。合并提交 `658c26a`。
5. **翻门禁常量**:read-model 命令现已在分支内(`main.rs` 注册 `cmd_get_knowledge_tree`/`cmd_get_topic_memories`),把 `KNOWLEDGE_CMDS_REGISTERED` 翻 `true`;测试 #1 改用 `registered={false}` prop 钉「未接线」态(不再依赖出厂默认)。提交 `06376b3`。
6. **门禁全绿(我的文件零新增)**:typecheck ✓;test 59/59 ✓;vite build ✓;prettier ✓;eslint 我改的两文件 **0 error**(`ProjectDashboard.tsx` 保持既有 33,交付已记账、未顺手重排)。
7. **重发 `request_acceptance` → 成功**。

## 账本验证(只读回读,不信成功串)

`acc-disp-onight-panel`:`status=pending`;branch/base=`packet/mira/knowledge-panel / track/infra-foundation`;plan 1/3/8/8;`fluxVerdictRef=ev-verdict-knowledge-panel`;decidedBy/At=null。事件链 6 条,末条 **`AcceptanceRequested`**(2026-10-11T18:18:17,seat-mira-001)。

## 集成构建状态(如实报)

源码层三段齐全、可干净拼合:基线有 read-model + 已入库数据,面板在交付分支,运营者点「接受」把面板合入基线即得集成构建。**剩一步属 Aegis 构建/部署车道**:运行中的桌面 app 需从集成态重编;本席位无法确认运行二进制是否已重编。运营者点「接受」前应由 Aegis 确认集成构建就绪,否则会点进一个未接线面板 —— 派单原文亦交代此风险由 Aegis 拥有。

## 边界

只做受治理 `request_acceptance` + 交付分支同步(基线合入分支,非分支合入基线)+ 面板门禁收尾 + 只读核验。未把面板合进基线(合并只发生在运营者点接受);未写 live 库(除受治理的 `request_acceptance`);未重启服务;未改 schema;`ProjectDashboard.tsx` 既有问题未顺手重排。

## 收尾

`complete_dispatch(disp-onight-panel-card)` ✓;`disp-onight-panel` 本身已 `done`,无需再 complete。前向记录见 `remember`。

## 附:产物索引

- 交付分支同步:`658c26a`(merge)+ `06376b3`(翻常量+测试钉 prop)。
- 账本事件:`AcceptanceRequested`(acc-disp-onight-panel)。
- 本交付说明。
