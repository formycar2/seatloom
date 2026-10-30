---
title: "A mistyped branch name and a genuinely absent file return the same empty output"
category: knowledge
topic: measurement-discipline
tags: [git-cat-file, silent-empty, typo, fourth-time, exit-code, false-absence]
created: 2026-10-30T11:20:00+08:00
source_kind: canonical_event
source_ref: ev-dry-run-accurate-and-the-five-groups-are-not-equally-hard
confidence: inferred
curation: authored
---

# 拼错的分支名和真的不存在的文件,返回一模一样的空输出

## 2026-10-30 11:00

我告诉张先生:`023_memory_dedup.sql` **不在 track 上**,并据此派了 Onyx 一件
优先级 99 的活(解撞号 + 登记 + 给 live 命令)。

他跑了 dry-run,脚本第一行就是:

```
migration 023 is ON TRACK (track/infa-foundation:infra/postgres/schema/023_memory_dedup.sql)
```

**我错了。** `023_memory_dedup.sql` 从 `ee2d03b`(10-28)就在 track 上。

## 我怎么错的

我查的命令是:

```bash
git ls-tree --name-only track/infa-foundation infra/postgres/schema/
```

分支名打错了:`infa` 而不是 `infra`。

`git ls-tree` 对一个不存在的 ref **静默返回空,退出码非零**。我把空输出读成了
"文件不在"。

## 为什么这个错误特别难发现

```
拼错分支名        : []   (退出码非零)
文件真的不存在    : []   (退出码非零)
文件存在          : []   (退出码 0)
```

**`git cat-file -e` 和 `git ls-tree` 都是"静默 + 退出码"设计。**
三种情况的标准输出完全相同 —— 都是空。

而我用的判断方式是 `&& echo 在 || echo 不在`,那**只反映退出码,
不区分"文件不存在"和"我连 ref 都拼错了"**。

更要紧的是:**我的错误方向是"不在"**,这会让整条下游推理继续跑下去 ——
"不在 track 所以没登记所以 live 没效果"听起来完全自洽。

## 第四次了

同一天,同一个形状:

| # | 我测了什么 | 实际 |
|---|---|---|
| 1 | Mira 的测试,在主检出跑 | 她分支上是 10 passed |
| 2 | "有缺陷的修复被合并了" | 那张卡绑另一个 verdict,干净 |
| 3 | "Nimbus 的修复丢失了" | 我站在 `worktrees/nimbus`,它在另一个分支 |
| 4 | "023 不在 track" | 分支名打错,静默空输出 |

**四次全是"坏消息",而错的测量给出的坏消息看起来和对的测量一样合理。**

## 规矩

### 1. 空输出必须被验证,不能被解读

任何"查了,结果是空/没有"的结论,都要先证明**那个查询本身能返回非空**:

```bash
# 错的:直接信空输出
git ls-tree track/infa-foundation infra/postgres/schema/ | grep 023

# 对的:先证明 ref 存在
git rev-parse --verify track/infra-foundation >/dev/null || { echo "分支名拼错了"; exit 1; }
```

**一个返回空的查询,和"那里没有东西",是两件事。**

### 2. 拼写不可猜的分支名,先 `rev-parse` 再查内容

分支名是权威来源,不是我的记忆。写死之前先:

```bash
git branch --show-current          # 我在哪
git rev-parse --verify <ref>       # 那个 ref 存在吗
```

### 3. "不在"这种结论比"在"危险

"在"会被下游验证(合进去会冲突、能 grep 到)。
**"不在"会让下游推理自洽地跑完**,而且它通常正好支持一个我想说的故事
(这次:"迁移没落地,所以 live 没效果"—— 那个结论本身是对的,
但我给的理由是错的,而我据错理由派了一件 99 优先级的活)。

### 4. 静默失败是设计缺陷,不是我的疏忽

`git cat-file -e` 的静默是有理由的(它就是个谓词)。
**但把它包进 `&& echo 在 || echo 不在` 就等于把"ref 不存在"和"文件不存在"合并了。**

正确做法是把退出码 128(ref 错误)和 1(对象不存在)分开 —— git 的文档明确区分了。
我的包装层把这个区分抹掉了。

## 结果

- 我给张先生的一条信息是错的,已在同一轮更正
- 据错理由派的 Onyx 那件活(99 优先级)**不是白派** —— 它真的抓到了
  `026_topic_close.sql` 的值域清单会静默毁掉空库,那是比编号撞车严重得多的缺陷
- 但它的**第二项(登记 023_memory_dedup)**是基于我错误的前提:那个迁移早就在 track 上,
  只是 `apply-db.sh` 没登记它 —— 这个结论仍然对,理由是"apply-db 是显式登记制",
  不是"迁移不在 track"

**结论对、理由错,比结论错更危险** —— 下一个人会照错的理由做判断。
