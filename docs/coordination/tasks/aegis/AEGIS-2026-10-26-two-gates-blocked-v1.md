# 补两个洞:卡了 10 天的三张单,前置条件实测不满足

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-two-gates-blocked-v1 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |
| 为什么 | 三张单(`disp-session-key-ui` / `disp-first-screen-verify` / `disp-integration-round-verify-v2`)从 10-16/17 卡到现在,等的两个门事件从没发 |
| Aegis 的核法 | 按「门的条件是能不能真的跑成功」逐条实测,不按代码合没合 |

## 实测结果(2026-10-26,两个门都还不满足)

**门1 `ev-session-key-readmodel-live` — 不满足**

```
select count(*) from v_memory_session_join;
→ ERROR: relation "v_memory_session_join" does not exist
```

public schema 下只有 `v_archived_sessions_normalized` 一个视图,
没有 session↔work 桥接视图。**表都没建,门不能发。**

**门2 `ev-history-sessions-read-live` — 不满足**

```
archived_sessions 总行数          272
message_count IS NULL             89
按 harness:codex 60 / stepcode 19 / gemini 9 / opencode 1
```

语料在,但**89 行没有消息数**。按已知约定(types-dto.ts 注明):
`messageCount` 为 NULL 表示语料迁移未应用,**不是 fabricated zero**。
所以这 89 行现在读不出内容,门不能发。

## 要做的两件事(分开做,分开交付)

### A. Nimbus — 建 session↔work 桥接视图

契约在 `docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md`
(§11.2 slice 4 是被卡那单的 done 定义)。

判据(可重跑):
1. `v_memory_session_join`(或契约里定的名字)**真的存在**,`select count(*)` 能跑
2. 视图能把 session 和它的工作关联起来 —— 给出**实际行数**和 1-2 行样例
3. 迁移写成可重复跑(幂等),不要 drop 历史数据
4. 应用后 `sessions` 表现有 5 行仍是 5 行(不许清空)

### B. Vela — 补齐 89 行缺失的 message_count

判据(可重跑):
1. 交付时给出:`select count(*) from archived_sessions where message_count is null`
   的结果(**要从 89 降下来,目标 0**;若某些 harness 天生读不出消息数,
   **如实说明是哪几行、为什么读不出**,不许填 0 冒充)
2. 已有 272 行不许被清空或重算(抽 3 行 before/after 对比行数不变)
3. 说明这 89 行分别属于哪些 harness、每个为什么之前是 NULL

## 明确不做

- **不许为了让门能发而造数据** —— 填 0 或编一个数比 NULL 更糟
- 不许改那三张 blocked 单的状态(改账本不是席位的权限)
- 不许合并代码

## 做完之后

A 和 B 都交付并通过验绿,Aegis 才发那两个门事件,
那三张单才有机会继续走。**在 A/B 真的能跑之前,Aegis 不会发门。**
