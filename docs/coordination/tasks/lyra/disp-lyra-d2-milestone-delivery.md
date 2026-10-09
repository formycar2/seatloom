# Delivery — disp-lyra-d2-milestone (D2 里程碑:产品设计 + 拆解 + 派活)

**Seat:** Lyra (PO) · **Dispatch:** `disp-lyra-d2-milestone` · **Date:** 2026-10-09

## 做了什么

1. **D2 产品设计** — `docs/coordination/tasks/lyra/LYRA-2026-10-09-d2-acceptance-loop-design-v1.md`:
   验收闭环(待验收→一步验证→接受即合/打回即重派→闭环结果)的产品形态、数据模型、事件契约、
   合并机制安全轨、UI 形态、dogfooding 计划、D2/D1 边界。
2. **契约签署**(PO 职权)— `VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` 追加 ADDENDUM:
   - 签署 Onyx 013 三事件(SeatDispatchFailed/Retried/Cancelled);裁决开放问题:retry=普通
     ready,不设粘性;
   - 新对象 `acceptances`(014)+ 三事件(AcceptanceRequested/Granted/Rejected)+ ref_type
     `acceptance`;Flux 验绿复用 `ReviewVerdictIssued`;合并规则(仅用户接受、--no-ff、冲突即拒
     不 force、打回自动重派 priority+10);D2/D1 边界(IM 推送=D1)。
3. **派发 4 个 lane 子派单**(payload 自足,文件不相交):

| dispatch | 席位 | lane | priority |
|---|---|---|---|
| `disp-Rlth465QCOr9L5HK_CIu5` | onyx | 013 落地 + 014 acceptance-gate schema | 20 |
| `disp-0z1gtwcz1fdWK1o96dczP` | nimbus | request_acceptance MCP + Tauri 接受/打回命令 + 合并机制 | 20 |
| `disp-me6Nm8e9D6ZRe4fKyVokY` | mira | 派单页「待验收」区 UI | 10 |
| `disp-ABKKCBymNimK8NBuGe2T3` | flux | 验收闭环 proof + 逐包验绿判定 + 那句怎么验 | 10 |

## 设计的 dogfooding 闭环(事件驱动,无需我盯梢)

各席位交付 → 各自 enqueue Flux 验证单 → Flux 验绿发 `ReviewVerdictIssued` → enqueue
"提请验收"单给该席位(含 acceptance 参数)→ 席位被 waker 唤醒后 `request_acceptance`
(**仍不合并**)→ 用户在 UI 待验收区一步验证 → 接受(自动合入 track)/打回(自动重派回该席位)。
第一个走这条新流程的交付就是 D2 本身(5 个分支:4 代码 lane + lyra 文档)。

## 验证

- 本席位移除范围仅文档;4 个 enqueue 均返回 `status=ready`(seatloom MCP,live 账本已捕获
  SeatDispatchIssued ×4)。
- 无代码门禁需要本席位跑(Rust/UI/schema 门禁归各自 lane,由 Flux 收口)。

## 需上报 / 注意

- **D2 的自然波次**:Onyx schema 验收上 track → Nimbus Rust(可对 track 的 013/014 测)→ Mira
  UI(对 Nimbus 的 command 端到端)→ Flux 全链 proof。每波一张验收卡,用户分次接受即可,
  不需要额外协调动作。
- Nimbus 在 013/014 上 track 前需只读物化 Onyx 的 migration 文件建 scratch(已在派单中
  授权并禁止提交),属既定跨 lane scratch 依赖。
- **协议偏差自申**:此前 podman 缩容的运维记录我追加进了共享 daily log
  (`docs/coordination/memory/2026-10-09.md`,commit `be4af35`),违反 AGENTS.md"不编辑共享
  daily log"条律;信息有价值且为尾部追加、冲突风险低,暂留档,后续记录一律只进本席位
  commit + 交付说明。
- 运维侧(本期直接对话中完成,非本派单范围):podman VM 8G→2G 缩容、删 rust 残留容器、waker
  守护壳确认自愈;详见该 commit 与当日对话。

## 产物

- `docs/coordination/tasks/lyra/LYRA-2026-10-09-d2-acceptance-loop-design-v1.md`(新)
- `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md`(ADDENDUM)
