# 交付说明 — disp-dZSlGCRUzUs6S3pnaSvMu(撤第三张卡 + 两个问题的回答)

## 1. 撤卡:已经撤了,没有重复撤

`acc-disp-reemit-stalled-announcements` 在上一轮(`disp-R_LETuejKcd2rdMsy304X`)已经撤掉,
账本实测:

```
acc-disp-reemit-stalled-announcements | cancelled | seat:seat-nimbus-001 | 2026-10-28 18:04:48
```

撤卡理由已逐字记录在 `acceptances.feedback` 里(见该行)。`cancel_acceptance` 是终态的,
再调一次只会被守卫拒绝,所以本单没有第二次撤卡动作。

三张卡现在的状态:

| 卡 | 状态 |
|---|---|
| `acc-disp-ibABeNk-nkEDAkI6Qj2sG` | cancelled |
| `acc-disp-b5VbRor1fHhP3nN4YZI_l` | cancelled |
| `acc-disp-reemit-stalled-announcements` | cancelled |

撤卡后 `scripts/check-verdict-refs.sh` → `good=5 bad=0`,`check-cards-clickable.sh` → `OK: all 5 pending cards are clickable`。

## 2. 问题一:重发这件活到底要不要一张卡?

**我的回答:按今天生效的规则,要。而且我不建议为它开豁免。**

先看我这一件实际改了什么(实测,不是回忆):

```
$ git show --stat 94f7585
 ...NIMBUS-2026-10-17-reemit-stalled-announcements-v1.md | 162 +++++++++++++++++++++
 1 file changed, 162 insertions(+)
```

**一件产品代码都没动。** 唯一的 diff 是那份交付说明;真正的产物是两条账本事件
(`ev-blocked-on-whom-dto-landed` 和 `ev-reemit-stalled-announcements-landed`)。

规则原文(`2026-10-27-verification-gate-class-rule-decision.md`):

- 第 3 条:**`implementation`(any diff)opens a card**,且需要非本席位的独立 GREEN verdict;
- 第 5 条:**`review`(investigates, reports, commits no diff)opens no card**。

而这个派单的 `acceptance_class` 实测是 **`implementation`**(发行方在派单里声明的,
不是我选的)。

所以按字面:它有 diff(那份说明),类别是 implementation → **要卡**。

你的直觉是对的 —— 它的内容确实贴近 review。但我**不建议**用"没有产品 diff所以不用卡"这个理由,
因为那个论证的形状你今天刚被驳回过一次:你论证"docs-only 提交不需要在 track 上",
而验收门是 `merge-base --is-ancestor`,一个字都不看提交里是什么文件,23 张卡照样全点不动。
同一个形状搬到卡上,就是拿"没有产品 diff"去驳一条写着 **any diff** 的规则。
一条能被这样拉伸的规则,下次就会被人拿去盖代码。

**如果你要它不开卡,正确的动作是改类别,不是解释类别**:发行方把
`acceptance_class` 从 `implementation` 改判为 `review`。这是 Lyra/Aegis 的裁量,不是我的。
改判的理由是现成的:规则自己写的 review 判据是 "no diff ⇒ no merge candidate",
而这一件唯一的 diff 就是交付说明本身,正是 review 的产物;两条事件已经在账本里、可审计。

## 3. 问题二:如果开卡,谁来验?

**不能是我。** 我自己验自己的活不算验,这是规则第 1 条,也是今天这三张卡出问题的同一条边界。

**应该给 Flux,一件一单、预声明 id** —— 和另两件已经派下去的一模一样。实测另两件的派单已入队:

| 派单 | 状态 | 内容 |
|---|---|---|
| `disp-Hm22QwdbFAvkTSUwOIC1_` | ready | 验洞 2(request_acceptance 不验结论指向) |
| `disp-zcTsJyKKG0fTnFbwVV-A8` | ready | 验洞 1(阻塞派单不记录卡在什么上) |

**第三件(重发)目前没有对应的 Flux 派单** —— 我查了,标题和 payload 里含"重发/reemit"的派单
都不指向它。这是缺口,需要补一张。

**但先说清这一件的验活能验到什么**,免得派下去又是一张空卡:

这次重发的全部主张是"边上声明的那个 id 现在存在了"。独立验活就是三条查询:

```sql
select count(*) from canonical_events where id='ev-reemit-stalled-announcements-landed';  -- 应为 1
select id, status from dispatches where id in ('disp-blocked-on-whom-ui','disp-first-screen-verify');
select count(*) from canonical_events where id='ev-blocked-on-whom-dto-delivered';         -- 应为 1,旧 id 未被删
```

外加核 `actor_ref` 是发布者本人(不是别人代发)。**这是真检查,但是薄的一条** ——
它验的是一次 ledger 写入的结果,不是一个行为。我如实说:如果 Lyra 认为这不足以构成
一次独立 verdict,那更该走的是第 2 节的改判 review,而不是硬派一张单。

## 4. 本单没有做的事

- 没有重开任何一张卡(明确被要求不要用原来的 ref 重开);
- 没有合并任何东西(那是操作者的 Accept);
- 没有动别人的卡;
- 没有改任何判据 —— 三张卡的 `acceptance_plan` 和 `how_to_verify` 在撤卡前是什么,撤卡后还是什么。
