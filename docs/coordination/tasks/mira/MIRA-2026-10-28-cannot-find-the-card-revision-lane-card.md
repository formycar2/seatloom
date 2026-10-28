# 关于「卡片改版 lane 的验收卡重派」—— 我核不到这个对象

POKE 说:

> 卡片改版 lane 的验收卡重派 —— 你两次失败都是因为 `acceptance_plan` 列不存在、
> vocab 只有 3 个值,这三样现在都在 live 了。但 verdict 要重新量,别假定 10-10
> 那条还指向这个派单。

**我在自己的库和工作区里核了一遍,找不到这张卡,也核不到它描述的状态。
我没有开任何卡。逐条列出我量到什么。**

---

## 一、我名下现在一张 pending 卡都没有

```
select id, status, dispatch_id from acceptances
where status='pending' and dispatch_id in
  (select id from dispatches where target_seat_id='seat-mira-001');
→ 空
```

而且我那三张卡**已经被张先生接受了**:

```
acc-disp-B4Y5oAohMQwtXm5Vva1fw   | accepted
acc-disp-HlxnnSKW2lX-rMR02AFk2   | accepted
acc-disp-CACpK8qlLm96WQY08itl-   | accepted
```

其中后两张是上一轮 Lyra 抓到拼写错误、我撤了重开的那两张。**它们已经走完了。**

## 二、 cancelled 的卡有四张,没有一张是"卡片改版 lane"

| 卡 | 派单内容摘要 |
|---|---|
| `acc-disp-me6Nm8e9D6ZRe4fKyVokY` | D2 里程碑:在界面里验收一个里程碑并闭环 |
| `acc-disp-supervisor-intent-ui-wiring` | 把 UI 接到 live 的 intent 工具上 |
| `acc-disp-qR65V3mi0m-4yYl4J8mmk` | 补派 Lyra 计划里漏掉的三件(界面侧) |
| `acc-disp-RUnXBR3uICuWmmFizJ7vX` | 切项目时下面每个模块都不过 |

**没有一张提到 lane、vocab、或"卡片改版"。**

## 三、POKE 描述的技术状态,我逐条核了

### 3.1 `acceptance_plan` 列 —— 存在

```sql
select column_name from information_schema.columns
where table_name='acceptances' and column_name='acceptance_plan';
→ acceptance_plan
```

这一条 POKE 说对了。

### 3.2 "vocab 只有 3 个值" —— 核不到

`acceptances` 表的全部列:

```
id, dispatch_id, project_id, branch, base, how_to_verify,
flux_verdict_ref, status, feedback, rework_dispatch_id,
merge_commit, decided_by, decided_at, created_at, acceptance_plan
```

**没有 lane 列。** 也没有 `%lane%` 或 `%card%` 的表:

```sql
select table_name from information_schema.tables
where table_schema='public' and (table_name like '%lane%' or table_name like '%card%');
→ 空
```

`fluxVerdict` 里确实有一个 `lane` 字段,但真实调用一次拿到的是 `None`:

```
get_acceptance_lineage(acc-disp-CACpK8qlLm96WQY08itl-)
  fluxVerdict = {refId, verdict, evidence, evidenceDetail, targetBranch, lane, occurredredAt}
  fluxVerdict.lane = None
```

Rust 侧也搜不到 lane 的枚举定义:

```
grep -rn 'lane' crates/seatloom-core/src/db/*.rs | grep -i 'enum|const|=>'
→ 空
```

**所以"vocab 只有 3 个值"这个前提,在我这边的 live 库里不存在对应物。**

## 四、"两次失败"我也核不到

我名下没有 rejected 状态的卡,只有上面四张 cancelled。
cancelled 的原因我逐条读过 payload,都与 lane/vocab 无关。

如果"两次失败"指的是别的事情(比如我某两次 `request_acceptance` 被工具拒绝),
那两次的类型化拒绝消息里提到的是 `acceptance_plan` 需要 `how_to_verify` 键、
以及 `refs.1.role` 不在允许值里 —— 都不是 lane。

## 五、我为什么不开卡

三个理由,每一个都足够:

1. **找不到这张卡对应的派单。** 派单要求"dispatch id 从账本抄",而我名下
   没有符合"卡片改版 lane"描述的派单。凭 POKE 的三个关键词去猜一个 id,
   正是我这两天连错两次的那类动作(拼写、漏推)。
2. **POKE 的技术前提我核不到。** 它说"三样现在都在 live 了",我核到一样
   (`acceptance_plan` 列)、核不到两样(lane 列/vocab)。**照一个我核不到的前提
   开工,是把猜测当下一步动作的输入。**
3. **我名下已经无 pending 卡。** 没有"重派"的对象 —— 重派是被拒/取消的卡
   重新开,而我那三张是 accepted。

## 六、我给 POKE 方的三个问题,任一个答了我就能动

1. **这张卡的 dispatch id 是什么?** 从账本抄,我从 `get_dispatch` 拿。
2. **"卡片改版 lane"指的是哪个功能?** 是 `fluxVerdict.lane` 字段,还是别的东西?
   我这边它的值是 `None`。
3. **"vocab 只有 3 个值"是哪三个值、在哪个表/哪个枚举里?** 我在
   `acceptances` 表和 `seatloom-core/src/db/` 下都搜不到。

## 七、我现在在做什么

我手里 `disp-CRUP2vSYJFV18TbV4z3tz`(血缘面板)仍 in_flight,已经完成的部分:

- **merge track 完成**:落后 147 个提交(Vela 的 washing-pipeline 合进来了),
  合并干净,`110bad3` / behind 0 / SYNCED
- **在自己工作区量到能力在**:`crates/seatloom-core/src/db/lineage.rs` 485 行、
  `cmd_get_acceptance_lineage` 已注册(`main.rs:191`)
- **真实调用成功**(不是 grep):通过 `target/debug/seatloom-mcp-emit` 走 stdio 调
  `get_acceptance_lineage`,返回 `acceptance` / `dispatch` / `events[9]` /
  `fluxVerdict` / `products` / `dependencies` / `rework` 七个部分

**POKE 说"别假定 10-10 那条 verdict 还指向这个派单"—— 这一条我认。**
我这次量的是当下,没有引用任何历史结论。

UI 侧的接线也已核到:`ui/src/lib/api.ts:158` 有
`getAcceptanceLineage: (acceptanceId) => invoke('cmd_get_acceptance_lineage', {acceptanceId})`。

血缘面板本身还没开始建 —— 我停下来先回这条 POKE,因为它在队列里指向一件
我核不到的事。
