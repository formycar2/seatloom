# Close-out — 作废陈旧待验收卡:第 6 次 cancel_acceptance 成功(撤卡 saga 收口)

**执行 actor(账本落款):** `seat:seat-mira-001` · **指令来源:** 张先生,经 Aegis 通道 · **Date:** 2026-10-11
**收口对象:** `disp-lyra-void-stale-d2-card`(Lyra → Mira,max_retries=0)+ 前 5 次被同因拒绝的 `cancel_acceptance`
**目标卡:** `acc-disp-me6Nm8e9D6ZRe4fKyVokY`(gate `disp-me6Nm8e9D6ZRe4fKyVokY`)

> 身份说明:本会话以 Aegis 通道接收张先生指令,但 MCP 解析出的 actor 是 `seat:seat-mira-001`
> (guard 因此通过、账本落款亦如此)。故本条记录归 `tasks/mira/`,按真相落,不按标签落。

---

## 1. 结论

第 6 次 `cancel_acceptance`(参数不变)**成功**,卡进入终态 `cancelled`。
前 5 次拒绝的共同根因——`acceptances_plan_required` 在 UPDATE 时对**整行新版本**求值,而遗留行
`acceptance_plan='{}'`——已由回填消除,本条不再复现。

## 2. 动作(仅受治理路径,无裸 SQL、无绕过、无弱化约束)

```
cancel_acceptance(
  acceptance_id = 'acc-disp-me6Nm8e9D6ZRe4fKyVokY',
  reason        = <之前那版:Lyra 逐字原文>
)
```
经 seatloom MCP 受治理工具发出。成功返回(逐字):

> cancelled acceptance acc-disp-me6Nm8e9D6ZRe4fKyVokY for disp-me6Nm8e9D6ZRe4fKyVokY
> (reason: base recorded as nonexistent track; content already bootstrap-merged by Aegis;
> voiding per ruling — the new feature gets a new card with lineage + structured plan)
> — the card is closed; no merge, no rework

## 3. 账本验证(只读回读,不信成功串)

- 行:`status=cancelled`;`decidedBy=seat:seat-mira-001`;`decidedAt=2026-10-11T17:49:41.863734Z`;
  `feedback`=reason 逐字;`mergeCommit=null`;`reworkDispatchId=null` → 干净撤销,无合并、无返工。
- 事件链末条:**`AcceptanceCancelled`(ev-9h_cUS-Etb6QnyHUGiMOV,2026-10-11T17:49:41.841813Z,
  by seat:seat-mira-001,payload 含 reason)**。这是全账本**第一条** AcceptanceCancelled ——
  Lyra 此前点出的「作废路径端到端从未有人走通」,至此走通。

## 4. 前提:回填已由 supervisor 完成(非 Aegis / 非张先生署名)

发前只读核验发现 `acceptance_plan` 已被回填为 **Lyra §2 逐字原文**(4 steps / 4 expected /
4 preconditions,`plan_version=1`),并附 `backfill_note`:

> Backfilled by the supervisor on 2026-10-12 to clear 016's acceptances_plan_required CHECK …
> The plan content above is Lyra's section 2 verbatim from
> docs/coordination/acceptance/2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1.md —
> the PO's own authorship, not written by Aegis.

这正是张先生坚持的原则落到实处:**方案须产品席位(Lyra)亲笔,不得由他或我署名,否则伪造血缘**;
而空-plan 遗留行也由此拿到「真承诺」而非「洗白的假数据」才过了 CHECK。

## 5. 边界与收尾状态

- 只做受治理 `cancel_acceptance` 一次 + 只读核验。未裸 SQL、未改 schema、未动词表、未合并、未重启。
- `complete_dispatch` **无目标**:`disp-lyra-void-stale-d2-card` 已 `done`,本会话无 in_flight 派单
  (本次为人通道请求,非在役派单);未调用 `get_next_dispatch` 拽新活。
- 前向记录:见 `remember`(topic-aegis-fleet-operations,decision)。

## 6. 附:本单产物

- 本收尾说明(唯一文件产物)。
- 账本事件:`AcceptanceCancelled` ev-9h_cUS-Etb6QnyHUGiMOV(见 §3)。
