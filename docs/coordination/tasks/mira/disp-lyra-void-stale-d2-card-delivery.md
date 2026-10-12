# Delivery — disp-lyra-void-stale-d2-card(作废陈旧坏卡:调用被拒,根因是 016 的 plan CHECK,非词表)

**Seat:** Mira (seat-mira-001) · **Dispatch:** `disp-lyra-void-stale-d2-card` · **Date:** 2026-10-11
**Issuer:** Lyra (seat-lyra-001) · **Branch:** `packet/mira/knowledge-panel`
**Base:** `track/infra-foundation`(`85bcc06`)

**结论:调用已按派单执行(Lyra 的 reason 逐字),被拒。拒绝**不是**派单预判的 vocabulary CHECK
—— 017 确实已应用且在正常工作。拒绝来自 **016 的 `acceptances_plan_required`**,根因是那张遗留行
自己的 `acceptance_plan` 是 `{}`。剩余动作是一条 live-DB 回填,只能由 Aegis 做。**

---

## 1. 派单要求的动作:已执行

按派单原文调用,`acceptance_id` 与 `reason` 均逐字:

```
acceptance_id = acc-disp-me6Nm8e9D6ZRe4fKyVokY
reason = base recorded as nonexistent track; content already bootstrap-merged by Aegis;
         voiding per ruling — the new feature gets a new card with lineage + structured plan
```

走受治理的 stdio 兜底(同一二进制、同一库、同一套 guard、本席位 `SEATLOOM_ACTOR_REF`)。

### 派单要求的「先重验一次」:已做(19:17:22,一次只读)

| 检查 | 实测 |
|---|---|
| 行状态 | `pending`,base=`track` |
| `acceptances_status_vocab` 含 `cancelled`(017) | **t** ✅ |
| 事件词表含 `AcceptanceCancelled`(017) | **t** ✅ |
| `acceptances_plan_required`(016) | `CHECK (acceptance_plan <> '{}') NOT VALID`,`convalidated=f` |
| 遗留行 `acceptance_plan` | **`{}`**(`plan_bytes = 2`) |

### 调用结果:被拒,原文照录

```
dispatch rejected: db error | ERROR: new row for relation "acceptances" violates check constraint
"acceptances_plan_required"
DETAIL: Failing row contains (acc-disp-me6Nm8e9D6ZRe4fKyVokY, disp-me6Nm8e9D6ZRe4fKyVokY,
seatloom, packet/mira/dispatch-ui, track, 在 SeatLoom UI 待验收区点接受 → 看到合并
sha;点打回 → 看到打回徽标+反馈+跳转返工单, ev-UDrRZZqr8LUwfcgEeFdN5, cancelled,
base recorded as nonexistent track; content already bootstrap-me..., null, null,
seat:seat-mira-001, 2026-10-11 11:17:23.006065+00, 2026-10-09 15:49:37.444118+00, {}).
| sqlstate=23514 constraint=Some("acceptances_plan_required") table=Some("acceptances")
```

按派单交代:**没有绕过、没有改用裸 SQL、没有编辑 live 词表、没有碰共享 .gitignore、没有改写历史。**

---

## 2. 派单的预判需要修正:不是 vocabulary CHECK

派单写「if cancel_acceptance is still refused by the DB vocabulary CHECK」。**实测不是**——
017 已应用且在工作,证据有二:

1. 词表实测:`acceptances_status_vocab` 含 `cancelled`;事件词表含 `AcceptanceCancelled`。
2. **更强的一条:同一张表上,另外两张卡刚被成功创建,且都带了真 plan**：

| acceptance id | status | base | plan_bytes | scenario(前 60 字) |
|---|---|---|---|---|
| `acc-disp-me6Nm8e9D6ZRe4fKyVokY` | pending | `track` | **2** | (none) |
| `acc-disp-wash-vela-p2` | pending | `track/infra-foundation` | **1508** | 张先生确认历史真正变成了「管得住的知识」… |
| `acc-disp-wash-vela-p1` | pending | `track/infra-foundation` | **1529** | 张先生确认清洗流水线前半段产出可信… |

也就是说:**016 的列在、`request_acceptance` 的 plan 强制在、新卡全部合规落地。整套验收链路是通的。**
唯一不合规的是 10-09 那张 bootstrap 写的遗留行 —— 它诞生于 016 存在之前,`acceptance_plan` 从来
没被写过,一直是 `{}`。

**所以断点非常窄:不是词表、不是工具、不是 guard(guard 每次都 PASS)、不是新卡路径,
就是那一行的一个字段。**

---

## 3. 根因

`cancel_acceptance` 的 SQL 是:

```sql
UPDATE acceptances
   SET status = 'cancelled', feedback = $2, decided_by = $3, decided_at = NOW()
 WHERE id = $1 AND status = 'pending'
```

它**不写** `acceptance_plan`。但 Postgres 的 CHECK 约束在 UPDATE 时对**整行新版本**求值,于是
`acceptance_plan <> '{}'` 被检查到这一行仍为 `{}` → 违反。

`NOT VALID` 的语义只豁免「加约束那一刻的既有行首次校验」,**不豁免之后的任何 UPDATE**。
这正是它拦住撤销的原因。

---

## 4. 剩余动作(只能 Aegis 做;做完无需任何代码改动)

```sql
-- 1) 回填 PO 亲笔方案(逐字,shape v1 已复验:4 steps / 4 expected / 4 preconditions)
UPDATE acceptances
   SET acceptance_plan = '<docs/coordination/acceptance/2026-10-10-lyra-mira-d2-ui-acceptance-plan-v1.md §2 JSON>'::jsonb
 WHERE id = 'acc-disp-me6Nm8e9D6ZRe4fKyVokY';

-- 2) 016 注释里点名欠的那一步
ALTER TABLE acceptances VALIDATE CONSTRAINT acceptances_plan_required;
```

之后同样参数一次成功,并正常落 `AcceptanceCancelled`(词表已就位)。

---

## 5. 一个值得注意的事实:作废路径从未被任何人走通过

```
canonical_events 里 AcceptanceCancelled 事件数 = 0
```

即 **017 落地至今,没有任何席位成功作废过一张卡**。这条路径端到端一次都没跑过。
上面那张遗留行是它的第一个真实用例,而它恰好是唯一带着 `{}` plan 的行。

建议 Aegis 在回填后,顺手确认这条路径能通(我可以立刻做这一次真实调用并回报),
否则下次任何一个席位要撤自己的卡,都会撞在同一面墙上。

---

## 6. 风险仍可控(但不撤有真实成本)

该卡 base 记为 `track`(不是分支),`cmd_accept_delivery` 在 checkout 处直接拒绝,走不到合并 ——
「整分支合并删掉 track 上 8 个文件」的数据损失场景已经不可能。

但派单说的成本是真的:这是张先生打开界面时唯一看得见的坏东西,一张点下去只会报错的待验收卡。
现在他的队列里有 3 张 pending,其中 2 张(Vela 的)带着真 plan、可以正常判断;**只有这一张是坏的**。

---

## 7. 边界确认

只做派单要求的事:一次重验只读、一次 `cancel_acceptance` 调用(Lyra reason 逐字)、只读查证根因。
未写 live 库、未改 schema、未合并分支、未重启服务。`crates/**`、`infra/**`、`src-tauri/**`、
`lib/types-dto.ts`、`lib/api.ts`、共享 `.gitignore` 均未动。发现的问题只记录 + 上报。

---

## 附:本单产物索引

- 本交付说明(唯一产物;按派单交代,拒绝报告即交付物)。
- Dispatch:`disp-lyra-void-stale-d2-card`(max_retries=0,派单明言拒绝即终态)。
- 账本事件:SupervisorMessage(把 verbatim 拒绝 + 修正后的根因上报 Aegis/Lyra)。
