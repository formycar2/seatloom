---
title: "Accepted cards merged two migrations that apply-db.sh has never heard of"
category: knowledge
topic: deployment-integrity
tags: [migration, apply-db, registration, number-collision, accepted-but-ineffective]
created: 2026-10-30T11:00:00+08:00
source_kind: canonical_event
source_ref: ev-accepted-migrations-never-registered-in-apply-db
confidence: inferred
curation: authored
---

# 被接受的卡合进了两个 apply-db.sh 从没听说过的迁移

## 2026-10-30 10:35 量到

张先生接受了 2 张卡(Nimbus 的镜像 clamp 修复 + 去重测试)。
merge `980e490` 确实是 track 的 tip,代码真在 track 上。这一半没问题。

但这次合入还带进两个迁移:

```
infra/postgres/schema/023_memory_dedup.sql
infra/postgres/schema/024_topic_close.sql
```

live 实测:

| | |
|---|---|
| `memories_dedup_key` 索引 | **不存在**(pg_indexes 0 行) |
| live 重复组 | **仍是 5 组**(去重迁移存在的理由) |
| 非 open 主题 | **0 条**(024 没跑) |
| `apply-db.sh` 里出现 | **一次都没有** |

**两张卡被接受、合并、代码在 track 上,而在跑着的库里效果是零。**

## 编号还撞了

schema 目录里同时有:

```
023_gate_criterion.sql
023_memory_dedup.sql
024_topic_close.sql
```

`023_gate_criterion.sql` 自己的注释说它"RENUMBERED from 020",理由是
track 和一个 packet 分支各自落了一个编号 020 的迁移,一个文件只能占一个号,
所以它取"下一个空号"。

**但 023 不空 —— 去重迁移早就占了。** 那条改号注释关于"下一个空号"的说法本身是错的。

## 为什么这比"二进制旧"严重

二进制旧是**可见的**:文件时间摆在那,我会主动报。

这个不一样:

- 迁移在 track 上 ✅
- 卡被张先生接受了 ✅
- 代码在 track 上 ✅
- **在跑着的库里效果是零** ❌
- **而产品里没有任何一处说明这件事** ❌

一个席位读账本,会得出"去重已生效、主题可以关闭"的结论。**两个都不真。**

## 根因:`apply-db.sh` 是显式登记制

每个迁移都要人手写进 `PRE_SEED_SCHEMA` 或 `POST_SEED_SCHEMA` 数组,
并且按要求写明排序理由、为什么放这一组。

这个设计的意图是好的 —— 现有条目里那些注释解释了真实的顺序依赖
(008 必须在 010 前,否则 010 的 G5 守卫会因为缺 `project_id` 而 abort)。

**但它的副作用是:合并没有任何机制会提醒你登记。**
git 不知道 `apply-db.sh` 存在,`apply-db.sh` 不知道 schema 目录多了文件。

## 规矩

### 1. 接受了带迁移的卡之后,必须查三件事,不是一件

| 查什么 | 命令 |
|---|---|
| 迁移在 track 上吗 | `git ls-tree track/infa-foundation infra/postgres/schema/` |
| `apply-db.sh` 登记它了吗 | `grep -c '<filename>' scripts/apply-db.sh` |
| live 上它的效果在吗 | 按它造的对象查(索引/列/约束/视图) |

**只查第一件,会得出"已完成"的错误结论。**

### 2. 改号时必须查目标号是否真的空着

`023_gate_criterion.sql` 的注释展示了正确的做法(只改文件名和自引用,
不重命名任何 SQL 对象),但它改号时没验证目标号空着。

**"取下一个空号"这个动作本身要被执行和验证,不能只在注释里声明。**

### 3. 去重的死锁现在更要紧,不是更不要紧

迁移终于上了 track,但它建的唯一索引在有重复行时建不了:

```
有重复 → 索引被拒 → 去重不生效 → 继续产生重复
```

live 有 5 组重复,所以**迁移就算被登记、被跑,也会被它自己的守卫拒绝**。
`scripts/land-023-memory-dedup.sh` 就是为这个写的:逐组列出两行的完整 substance,
让张先生决定哪条活。**这不是可选的收尾,是迁移能落地的唯一门槛。**

### 4. 我没查,所以它溜过去了

我验了 merge 在 track 上、验了 `clamp_backfill_to_pane` 在 track 上,
**然后停了。** 我应该问一句"这次合入还带了什么",再看那些东西在 live 上有效吗。

**验到第一个绿灯就停,和没验的区别只在心里好受。**
