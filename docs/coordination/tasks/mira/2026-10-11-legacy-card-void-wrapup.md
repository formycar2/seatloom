# 收尾记录 — 撤销遗留待验收卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY`(Aegis 人类通道交办,非派单)

**Seat:** Mira (seat-mira-001) · **Date:** 2026-10-11
**Branch:** `packet/mira/knowledge-panel` · **Base:** `track/infra-foundation`(`85bcc06`)
**交办来源:** Aegis,经人类通道转达(本单**不是** governed dispatch;队列实测无我的 ready/in_flight 派单)

---

## 0. 结论(先说重要的)

**卡仍未撤掉。三次调用全部被拒,第三次(对着 16:48 重建的新二进制)仍被同一个 CHECK 挡住。**

我 10-10 11:50 上报的那个阻塞(`acceptances_status_vocab` 不含 `cancelled`)**确实已经修好了** ——
017 落地,live 库现在是 4 值。但同一个 UPDATE 现在撞上**另一个**约束:016 的
`acceptances_plan_required`。剩余动作是一个 live-DB 回填,只能由 Aegis 做(见 §4)。

按 Aegis 要求,拒绝原文原样抄录,未绕过、未改用裸 SQL。

---

## 1. 三次调用与拒绝原文

全部走受治理的 stdio 兜底(同一二进制、同一库、同一套 guard、本席位自己的
`SEATLOOM_ACTOR_REF`、cwd=本 worktree)。**guard 每次都 PASS** —— 错误都来自数据库,在 guard 之后。

### 第一次 10-10 11:48(我上报的那个阻塞)

```
dispatch rejected: db error | ERROR: new row for relation "acceptances" violates check constraint
"acceptances_status_vocab"
DETAIL: Failing row contains (..., pending, ...). | sqlstate=23514
constraint=Some("acceptances_status_vocab")
```

### 第二次 10-11 09:01(vocabulary 已修,换成 plan CHECK)

```
dispatch rejected: db error | ERROR: new row for relation "acceptances" violates check constraint
"acceptances_plan_required"
DETAIL: Failing row contains (..., cancelled, Bootstrap-installed, not accepted. This card was
written by the ..., null, null, seat:seat-mira-001, 2026-10-11 09:01:54.382828+00,
2026-10-09 15:49:37.444118+00, {}). | sqlstate=23514
constraint=Some("acceptances_plan_required")
```

### 第三次 10-11 17:02(Aegis 提示二进制已重建后,对着 16:48 的新二进制重跑)

```
dispatch rejected: db error | ERROR: new row for relation "acceptances" violates check constraint
"acceptances_plan_required"
DETAIL: Failing row contains (acc-disp-me6Nm8e9D6ZRe4fKyVokY, disp-me6Nm8e9D6ZRe4fKyVokY,
seatloom, packet/mira/dispatch-ui, track, 在 SeatLoom UI 待验收区点接受 → 看到合并
sha;点打回 → 看到打回徽标+反馈+跳转返工单, ev-UDrRZZqr8LUwfcgEeFdN5, cancelled,
Bootstrap-installed, not accepted. This card was written by the ..., null, null,
seat:seat-mira-001, 2026-10-11 09:02:41.55187+00, 2026-10-09 15:49:37.444118+00, {}).
| sqlstate=23514 constraint=Some("acceptances_plan_required") table=Some("acceptances")
```

(错误里的 `how_to_verify` 与 `reason` 是 Postgres 自己截断的,原样照抄。)

---

## 2. 根因(只读实证)

```
acceptances 行:            acc-disp-me6Nm8e9D6ZRe4fKyVokY | pending | base=track
acceptance_plan:           {}            ← 就是它
acceptances_plan_required: CHECK (acceptance_plan <> '{}'::jsonb) NOT VALID   (convalidated = f)
acceptances_status_vocab:  4 值,含 cancelled   ← 我第一次上报的,已修
canonical_events 词表:     无 TopicOpened        ← 见 §5
```

**`NOT VALID` 只豁免「既有行的首次校验」,不豁免后续 UPDATE。** `cancel_acceptance` 的
`UPDATE acceptances SET status='cancelled', feedback=…, decided_by=…, decided_at=NOW()` 会重写
这一行,CHECK 于是对新行版本生效 —— 而 `acceptance_plan` 仍是 `{}`,违反
`acceptance_plan <> '{}'`。

所以顺序是:先有人给这行填上真 plan,撤销才能通过。这不是工具问题、不是 guard 问题、
不是词汇表问题,是**那一行的数据**问题。

---

## 3. 我做过但没做的事

- ✅ 用了受治理的 stdio 兜底路径(同一二进制/库/guard),guard PASS。
- ✅ 用了 Aegis 给的 reason 原文,一字未改。
- ✅ base 用真实分支名(本工具路径不涉及,但既往调用均如此)。
- ❌ **没有**改用裸 SQL 直接 UPDATE(明确禁止,也没做)。
- ❌ **没有**自己回填 plan 或 VALIDATE 约束 —— 那是 Aegis 车道的 live-DB 动作,且 016 的迁移
  注释白纸黑字写着这个回填归 Aegis。
- ❌ **没有**为了让调用通过而改弱任何东西。

---

## 4. 剩余动作(只能 Aegis 做;做完无需任何代码改动,我再调一次即成功)

Lyra 的亲笔方案早已写好并可直接用:
`docs/coordination/acceptance/2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1.md` §2
(shape v1 已复验:4 steps / 4 expected / 4 preconditions,恰 5 个键)。

```sql
-- 1) 回填 PO 亲笔方案(§2 JSON 逐字,单行形式 1184 字节)
UPDATE acceptances
   SET acceptance_plan = '<§2 JSON 逐字>'::jsonb
 WHERE id = 'acc-disp-me6Nm8e9D6ZRe4fKyVokY';

-- 2) 补上 016 注释里点名欠的那一步
ALTER TABLE acceptances VALIDATE CONSTRAINT acceptances_plan_required;
```

之后 `cancel_acceptance` 用**同样的参数**立刻成功,并正常落 `AcceptanceCancelled` 事件
(词表已就位)。我可以在一轮内重做。

**当前风险仍可控**:该卡 base 记为 `track`(不是分支),`cmd_accept_delivery` 在 checkout 处直接
拒绝,走不到合并 —— 「整分支合并删掉 track 上 8 个文件」的场景已经不可能。剩下只是一张挂在
运营者待验收列表里、点下去只会报错的过期卡。这正是它值得撤掉的原因,但它不会造成数据损失。

---

## 5. 前向记录(remember):跑了,失败,原文照录

按 Aegis 交代跑一次、失败写明、不卡住。两次都失败,且**都在数据库层**:

```
remember(kind+confidence 齐全):
  memory tool rejected: postgres error: db error
remember(缺 kind,即 018 前的旧形状):
  memory tool rejected: bad arguments: missing field `kind`
open_topic(topic_id 齐全):
  open_topic rejected: event write rejected: postgres error: db error
```

与 Aegis 的判断一致:`open_topic` 挂在事件写入上 —— live 库的
`canonical_events_event_type_vocab` **没有 `TopicOpened`**(019 已落 track 未应用到 live)。
没有主题就没有 `topic_id`,`remember` 于是也无从落地(且 `topics` 表现 0 行)。

**解锁**:019 应用到 live 库。届时 `open_topic` 可建主题、`remember` 可落记忆,前向记录协议
才算真正可执行。在那之前我不写旁路文件代替记忆(协议明令禁止)。

---

## 6. 顺带:二进制工具面在 16:48 又变了(第三次观察到)

`target/debug/seatloom-mcp-emit` 16:48 重建后广告 **18 个工具**,新增
`open_topic`、`get_knowledge_tree`、`get_topic_memories`;且 `remember` 现在**要求**
`kind` 与 `confidence`(018 的 typed 字段已进工具面)。而 `track/infra-foundation` @ `85bcc06`
的源码里没有这些实现。

**对我的知识面板是好消息**:`get_knowledge_tree` / `get_topic_memories` 的 MCP 读工具已在二进制里,
说明 Nimbus 的 read-model lane 已推进。但面板的 `KNOWLEDGE_CMDS_REGISTERED` 仍须为 false,
因为它守的是 **Tauri 命令**(`cmd_get_knowledge_tree` / `cmd_get_topic_memories`)是否注册在
**跑起来的 app 构建**里 —— 那要等 src-tauri 侧落地 + 重打包,不是 MCP 二进制的事。我没有去翻这个
开关,围栏与「命令未注册就显式渲染该事实」的纪律不变。

工具面 / track 源码 / live schema 三者仍不同步,建议 Aegis 明确二进制来源分支。

---

## 7. complete_dispatch:无对象可完成

队列实测(`get_next_dispatch`)返回 "nothing ready for seat-mira-001 (idle, not a stall)";
数据库实测亦无我的 ready/in_flight 派单。本次是 Aegis 经人类通道交办,不是 governed dispatch,
**因此没有 dispatch_id 可 complete**。我没有虚构一个去调。

如果 Aegis 希望这次撤销作为一次受治理的交付入账,请派一个 dispatch 给我;参数已全部就绪
(§4 的回填 + 一次同样参数的 `cancel_acceptance`),我可以立刻做完并照协议收尾。

---

## 8. 边界确认

只做被交代的事:重试撤销调用、只读查证根因、跑一次前向记录、写明失败。未写 live 库、未改
schema、未合并分支、未重启服务、未写旁路记忆文件。`crates/**`、`infra/**`、`src-tauri/**`、
`lib/types-dto.ts`、`lib/api.ts` 均未动。
