---
title: "My verification checker reported shell-based=0 while four cards told the operator to INSERT into the ledger"
category: knowledge
topic: tooling-integrity
tags: [acceptance, checker-bug, false-negative, ifs, newline, shell-verification]
created: 2026-10-29T04:20:00+08:00
source_kind: canonical_event
source_ref: ev-checker-reported-clean-while-four-cards-ran-shell
confidence: inferred
curation: authored
---

# 我的检查工具报了 shell-based=0,而四张卡正在让操作者往账本里 INSERT

## 发生了什么

2026-10-29 11:00 心跳。四项机械检查全绿,`check-card-verifications.sh` 打印:

```
total=7  shell-based=0  never-names-a-surface=0
```

我手工去读那 7 张卡(而不是只信工具),发现:

| 卡 | 验收方案在要求什么 |
|---|---|
| `acc-disp-accplan-lane-c-onyx` | 跑两遍 `apply-db.sh`,四条 `psql`,**其中两条是 `INSERT INTO acceptances`** |
| `acc-disp-d5-lane-a-onyx` | 跑两遍 `apply-db.sh`,四条 `psql` 自省 |
| `acc-disp-019-onyx` | 一条可选只读 `psql` |
| `acc-disp-bc20UzOznJQ8hCgbHw876` | 一条可选只读 `psql` |

工具说 0,实际 4。而其中一张让张先生**写 `acceptances` 表本身** —— 那正是记录"他批准过什么"的表。

## 两个 bug,都是我的,而且是叠在一起的

### Bug 1:检查读错了列

```sql
select a.id, a.dispatch_id, coalesce(a.how_to_verify,'') from acceptances ...
```

`how_to_verify` 是席位填的那一栏,写得很干净(全是"打开应用…")。
shell 命令在 **`acceptance_plan::text`** 里,而 `acceptance_plan` 才是产品在卡面上渲染的那一栏。

**一个只读席位填得整齐的那一栏、不读产品真正渲染的那一栏的检查,不是检查卡,是检查席位的打字习惯。**

### Bug 2:换行把行切碎了

把 `acceptance_plan` 接进来之后,`shell-based` 仍然报 0。原因是连接符用了 `chr(10)`:

```sql
coalesce(a.how_to_verify,'') || chr(10) || coalesce(a.acceptance_plan::text,'')
```

`while read` 按行读。**payload 里有换行,它就不是一行。** JSON 落到下一行,
被 `read` 当成新的 `card`,于是 `case "$how"` 看到的是空的或者第一行的残片。

同理,`IFS='|'` 也不行 —— JSON 里全是 `|`。分隔符必须是**不可能出现在 payload 里的字符**,
`0x1f`(unit separator)才是;而 payload 里不能有换行,所以连接符用空格。

```bash
US=$(printf '\037')
psql ... -F"$US" -c "select id, dispatch_id, how_to_verify || ' ' || acceptance_plan::text ..."
while IFS="$US" read -r card dispatch how; do
```

修完:`total=7 shell-based=4 never-names-a-surface=0`。

## 为什么这个 bug 特别危险

它是**假阴性**,而且假阴性长什么样都行 —— 它不报错、不打叉、打印的是 `0`。
一个报"0"的检查和真的"0"在输出上完全一样。

我在 10-28 写这个工具时,还专门给它加了一段说明:"两项计数都不是判定,都是问题"。
那段说明让工具显得谨慎,而它当时就在漏报。**免责声明不替代正确性。**

## 规矩

### 1. 计数为 0 要先问"这个 0 有没有可能是我读错了"

尤其是自己写的检查工具。`0` 是最容易被当成"没问题"的数字,也最容易是
"我没看那一列"或者"行被切碎了"。

验证办法:**找一个确定应该命中的样本,看它是否命中。**
我修好之后立刻能用 `acc-disp-accplan-lane-c-onyx` 复现 `SHELL` 命中 ——
如果它还不命中,说明工具仍然是坏的,而不是卡变干净了。

### 2. payload 里有换行/竖线时,`while read` 的 IFS 和连接符都要重新想

`IFS='|'` + JSON = 行被切碎。`chr(10)` 连接 + `while read` = 一行变多行。
**读结构化字段的 shell 循环,要么用不可能出现的分隔符,要么不要用 read。**

### 3. 工具必须读产品渲染的那一栏,不是人写得好看的那一栏

`how_to_verify` 是人写的摘要,`acceptance_plan` 是结构化的、卡面上真正显示的。
检查要对着**用户会看到的东西**,不是对着**最容易被 grep 的那一栏**。

## 顺带查清的(这些卡本身没问题)

同一轮我核了另外几张卡的断言,用真实数据而不是读码:

- **10 个席位**:live `seats` 表确实是 10 行;vela 的 `capability_tags` 精确等于
  `{corpus-ingest,extraction,redaction,data-washing}`,与卡上写的四个一致。
- **019 的三个主题**:`mira-testing-discipline` / `mira-ui-states` / `mira-verification-method`
  都在、都 `open`;`open_topic` 工具在 `topics.rs:127` 真在。
- **7 张卡全都有结构化方案**:`acceptance_plan is null` 全为 false,`steps` 都带 input/action。
- **"这个会话提炼出了什么"**:产品源码 `HistorySessionsPanel.tsx:900` 真有这段文案,
  三种状态在 916 / 925 / 1034 行有真实分支。**测试不是自证的。**

但发现一个真空缺:**`seat-networkorchestrator-001` 和 `seat-stepaicloud-001` 的
`capability_tags` 是 `{}`**,种子 005 没给它们声明。Onyx 的卡里甚至写了一句
"空标签的席位就是真空,没有假装有"——诚实,但操作者点开这两个席位会看到空白。
