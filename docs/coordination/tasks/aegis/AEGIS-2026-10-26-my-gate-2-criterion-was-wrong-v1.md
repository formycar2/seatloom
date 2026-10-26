# 更正:我给门2 定的判据是错的,Vela 拒绝执行是对的

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-my-gate-2-criterion-was-wrong-v1 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |
| 涉及 | 派给 Vela 的 `disp-2o_BDY1tb3b92ZQwaYXEC` |
| 结论 | **我的判据错,Vela 拒绝写 live 库并上报,是对的** |

## 我错在哪

我看到 `archived_sessions` 有 89 行 `message_count IS NULL`,就把它当成"洞",
派 Vela 把它降到 0。

**但那 89 行不是缺陷。** `021_history_sessions_corpus.sql` 的列注释写明:

> NULL = no extracted conversation exists for this row — that is what
> distinguishes the 183-row corpus from the 271-row archive inventory ...
> a project-grouped read must filter on this column being NOT NULL to avoid
> double counting the 89 inventory-only rows

**NULL 是区分「语料(183)」和「清单(271)」的哨兵值。** 我让她填 0,
会把两者混为一谈、让每个项目被重复计数 —— 正是我同时禁止她做的事
("不许填 0 冒充")。**我的两个要求不可能同时满足,因为正确答案是"不动"。**

## 重新实测(门2 的真实条件,现已满足)

```
语料(message_count 非空)  183   ← 021 注释预言 183  ✅
清单(message_count 为空)   89   ← 021 注释预言 89   ✅
消息总数                 36680  ← 021 注释预言 36680 ✅
```

三个数字与迁移注释里写的完全一致。且:

- 后端 `crates/seatloom-core/src/history/read.rs:970/1018` **已 filter
  `message_count IS NOT NULL`**(防重复计数的关键)
- 后端类型是 `Option<i64>`(不是 i64)
- UI `HistorySessionsPanel.tsx:287/373` 在 null 时**跳过不计**并单独提示,
  **不显示成 0**

**代码三层都正确处理了 NULL,门的前置条件本来就满足 —— 是我的测量方法错了。**

## Vela 做对的两件事(要记下)

1. **拒绝写 live 库**:那是迁移/DDL 类动作,不是她的权限,她按红线升级
2. **不停在拒绝**:她做了只读调查,把"为什么不能填"的完整依据带回来,
   让这个升级是可执行的,不是一句"我不能做"

## 因此

`disp-2o_BDY1tb3b92ZQwaYXEC` 的正确结论是 **delivered(调查+升级)**,不是 failed。
Vela 不需要补任何东西。
