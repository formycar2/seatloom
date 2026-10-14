# request_acceptance 加可选 acceptance_id(让作废后能重开)

**派单方**:Aegis · **执行**:后端席位 seat-nimbus-001
**派单 id**:`disp-acceptance-reissue-id-param`
**触发**:洗数据席位 2026-10-14 撞墙(见下)

## 它撞到的(verbatim)

> 人话重写完成并通过 request_acceptance 形状校验,两张旧卡已作废;但「重开」被工具面缺陷堵死,
> verbatim: `duplicate key value violates unique constraint "acceptances_pkey"`
> `Key (id)=(acc-disp-wash-vela-p1/p2) already exists` (sqlstate 23505)。
> 根因:`request_acceptance` 无 id 入参、卡 id 由 dispatch_id 确定性推导,
> `cancel_acceptance` 置终态不删行 —— 「不得有 pending 卡」守卫放行、确定性主键堵死。

它**没有绕**:无 raw SQL、无旁路文件、未改 live 库。结 `failed` 不结 `done`,理由是
「成功条件(张先生有可验的新卡)不存在,标 done 即无证据的门式声明」。

它还提醒:界面那张同形派单会撞同一个 PK,修好前别作废。

## 根因(两层守卫互相打架)

| 守卫 | 行为 |
|---|---|
| 「不得有 pending 卡」 | 作废后**放行**重开 |
| `acceptances_pkey`(确定性 id) | 因为 `cancel_acceptance` **不删行**,同一个 id 还在,**堵死**重开 |

作废只改 status,不删行 —— 这是对的(终态不可改是 014 的规矩)。
但 id 由 dispatch_id 确定性推导,导致"同一个 dispatch 只能提请一次,终身"。

## 修法:加一个可选入参

`request_acceptance` 增加可选 `acceptance_id`:

- **不传** → 行为完全不变(仍是 dispatch_id 推导),现有调用方零影响
- **传了** → 用调用方给的 id,让"作废后用同一个 id 重开"走得通

席位置身性守卫**不放宽**:
- 只能以本席位身份提请(`SEATLOOM_ACTOR_REF`,不接受调用方传 seat_id)
- 只能重开**本席位**作废过的 id
- 传一个已被 **pending** 卡占用的 id → 拒绝,且不改任何行

## 验收判据

1. 不传 id 时,现有全部调用方行为不变(**回归测试覆盖**)
2. 传一个已作废卡的同 id → 重开成功
3. 传一个 pending 卡占用的 id → 拒绝,且不改任何行
4. 传别的席位提请过的 id → 拒绝(席位置身性守卫没被绕过)
5. **一条真实路径跑通**:洗数据席位用它重开 `acc-disp-wash-vela-p1` / `p2`

## 为什么这个修法优先

洗数据席位已经把两份人话计划**逐字**写好了
(`docs/coordination/tasks/vela/disp-acceptance-cards-plain-language-vela-delivery.md` §6),
机制一好它就能一键重开,不用再写一遍。

界面那张(`acc-disp-supervisor-intent-ui-wiring`)已经重开成功了 ——
说明它在洗数据之前派单,撞墙之前完成。**修好之后两张一起重开,一次清完。**
