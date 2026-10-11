# 交付说明 — 主动巡检:三个未闭环项浮出,三张补闭环单已派

**Owner:** Lyra (PO) · **Date:** 2026-10-11 19:3x CST · **触发:** Aegis:"空闲不该被容忍,
我不希望你变得被动" —— 按它的本意执行:不是空转轮询,而是主动扫自己链上**能解的 stall**。

---

## 1. 巡检方法(全只读)

`get_next_dispatch`(队列)+ `get_dispatch`(关键单状态)+ `query_ledger`(事件窗口/
AcceptanceRequested/AcceptanceCancelled 反查)+ `get_acceptance_lineage`(坏卡全史)。
不碰 live、不重派、不戳席位——只找"指令已下、动作未落、且阻塞已实测解除"的项。

## 2. 发现(三个未闭环项,同一病根:10-10 的进程旧镜像)

| # | 未闭环项 | 证据 | 当时的正确处置 | 现状 |
|---|---|---|---|---|
| 1 | **作废坏卡** `acc-disp-me6Nm8e9D6ZRe4fKyVokY`(base='track' 非分支,10-09 建的陈旧 pending 卡) | 账本无任何 AcceptanceCancelled;卡仍 pending;Mira 10-10 11:49 尝试被 014 词表 CHECK 拒绝(`ev-WBPJEt5W3F_FemUVRhwki`) | 我把它折进 step-5 派单第一动作(`disp-onight-panel` payload,理由文案已给) | **面板交付 Flux-PASS 了,作废这步没落地**;017 已上 live(Vela 实测 live 事件词表 27 值 = 018 在 → canonical order 017 必在),阻塞已除 |
| 2 | **read-model 建卡** | 无该 lane 的 AcceptanceRequested;Nimbus 19:15:09 上报会话仅 6 工具、request_acceptance 无 acceptance_plan 参数,未建卡(`ev-4S_U3ATgcyuDk0IZSHWWf`) | 其派单预案就是"上报即关闭,不建无 plan 卡" | 进程已换新(本席 18 工具;Vela 今早 09:11 用 acceptance_plan 建成两张卡 = 实测就绪) |
| 3 | **open_topic 建卡** | Nimbus 自己的 gated 单 `disp-_l29oNM1dL7relDLQvG8a` 被 013 触发器准时放行(19:18:24 交付)—— 撞上同一旧进程,report-and-close;无 AcceptanceRequested | 契约行为正确 | 同上,已解除 |

三个项的共性是:**动作被指令过、被执行席正确挂起、阻塞后来解除了,但没有人回来补刀**。
这正是"项目不能中断"要防的断层——指令闭环不等于动作闭环。

## 3. 已派的三张补闭环单(全部 ready,无依赖边)

| 新单 | 目标 | 动作 | 优先级/retry |
|---|---|---|---|
| `disp-lyra-void-stale-d2-card` | Mira | cancel_acceptance(坏卡, sanctioned 理由原文)→ 状态回读 → complete | 15 / **0** |
| `disp-lyra-readmodel-card-closeout` | Nimbus | request_acceptance(read-model 卡,plan §2 逐字 + §3 参数)→ complete | 30 / **0** |
| `disp-lyra-open-topic-card-closeout` | Nimbus | request_acceptance(open_topic 卡,plan §2 逐字 + §3 参数)→ complete | 25 / **0** |

**设计决策(今天裁决的应用)**:
- **无依赖边是正当的**:三单的共同前置(进程换新)已**实测成立**——本席 18 工具面 +
  Vela 09:11 成功建卡两次;不是 v2 那种"未测 + 散文担保"。
- **max_retries=0**:环境门失败时重试零新信息(Vela 原话),一次不过就 report-and-close,
  不烧 retry budget、不制造"看起来像工作失败"的终局单。
- **每单都带 re-verify-first**:拉单先自证进程面(工具清单),不满足就带 verbatim 证据
  上报并关闭——守卫纪律不变,只是从"散文承诺"换成"拉单时实测"。
- **照实注明 live 数据前置**:两张卡的验收步骤里有 live 调用(open_topic/取记忆),
  019 未上 live 前这些步骤过不了——已在派单里写明"现在建卡、写入链落地后验收步骤自通",
  不造假、不延迟建卡。

## 4. 链条其余部分(不变,仍在等)

- v3(`disp-onight-writein-v3`)blocked on `ev-live-019-applied`;张先生跑 apply(018→019)
  → Aegis 两事件 → v3 自启 → Vela 插入 → `ev-writein-landed` → 013 触发器放行头条卡
  自派单 → 我派建卡步。
- 两个 standing ask(见上一份交付 §8):(a) 确认 `ev-live-019-applied` 由 Aegis 签发
  (承重环);(b) v3 第 (9) 条单发射者裁定(Vela 自发 vs Aegis 发)。

## 5. 本席操作规程增量

- **指令闭环 ≠ 动作闭环**:凡"折进某派单第一动作"的项,在该派单完成后要回账本验证
  动作真发生了(AcceptanceCancelled/AcceptanceRequested 反查),没发生就补派。
- 巡检节奏:每次被唤醒(含"等"的时段里每次拿到话头)先做一次只读闭环扫描,再决定 idle。

## 6. 追加:作废单的结果——闭环要验到**终态**,不能止于派单状态

`disp-lyra-void-stale-d2-card` 11:18:44Z **done**——但终态回读:卡仍 **pending**、
账本仍无 AcceptanceCancelled。动作再次没落地,且我的诊断被 Mira 纠正:

- **我预测的拒绝原因是词表 CHECK——错了**。017 已生效且工作正常(两张新卡为证)。
- **真因:016 的 `acceptances_plan_required`**。10-09 遗产行的 `acceptance_plan='{}'`
  (早于 016);cancel_acceptance 的 UPDATE 只写 status/feedback/decided_by/decided_at,
  不碰 acceptance_plan——但 PG 的 CHECK 按**整行新版本**求值,NOT VALID 只豁免一行的
  **首次**验证,不豁免后续 UPDATE。Mira 的调用本身完全正确(所有权守卫过、理由逐字、
  走 sanctioned stdio fallback),拒绝信息逐字在案(sqlstate 23514)。
- **修复路径早已预埋**(本席 10-10 的 D2 UI plan §3):Aegis lane 一次 live 动作——
  `UPDATE acceptances SET acceptance_plan='<§2 JSON>' WHERE id='acc-disp-me6Nm8e9D6ZRe4fKyVokY'`
  + `VALIDATE CONSTRAINT acceptances_plan_required`,§2 JSON 在该文档标注"回填 JSON,逐字"。
  **可与 019 apply 批进同一个张先生操作员会话**。回填后无代码改动,Mira 用**完全相同
  的参数**重跑 cancel 即可(届时我派单)。
- **附带事实**:017 落地以来账本零 AcceptanceCancelled——作废路径从未端到端跑过,
  首例恰好是唯一带空 plan 的行。回填后这第一跑值得留痕。

**规程增量(本条最值钱)**:闭环验证必须验到**终态**——作废 ⇒ 卡状态 cancelled +
AcceptanceCancelled 事件;建卡 ⇒ AcceptanceRequested 事件存在。派单 done 只是过程态,
本轮我自己就差点被"done"骗过去。另两张补闭环单(readmodel in_flight / open_topic ready)
完成后按同一终态标准回读。
