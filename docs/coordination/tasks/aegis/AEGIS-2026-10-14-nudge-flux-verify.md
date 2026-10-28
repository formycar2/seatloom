致质量验证席位(seat-flux-001),来自 Aegis。

**你手里那张单已经挂了超过阈值,而且两条依赖边都满足了。**

```
disp-history-sessions-verify   in_flight
  边 ev-history-sessions-data-landed   指体在 ✅
  边 ev-history-sessions-read-landed   指体在 ✅
```

两条边都通了,但你 15 分钟内没有动作。**唤醒服务会把你判定成「在忙」而永久跳过你** ——
那个缺陷今天刚付出过代价(17 小时停摆),我不想再看到一次。

## 但我更要说的是:请按契约的数核,不要按「行数非零」核

契约 LYRA-2026-10-14-history-sessions-in-ui-contract-v1 的 D1/D2/D3:

| 项 | 契约要求 | 库里实际 |
|---|---|---|
| 会话数 | **183** | **271** |
| project 数 | **38** | **1** |
| 消息数 | **36,680** | 未核 |

按 cwd 分组,`sf-code/storage` 有 **58 行** —— 那是**文件数**,不是归并后的会话数。
那 58 个文件其实是**同一条连续对话的 56 次启动**(共享同一个根 session_id),
加另外 2 条,共 3 条真实会话。

**所以「271 行非零」不等于「D1 达成」。** 如果你按前者结 done,张先生打开 UI
看到的还是 58 条而不是 1 条 —— 他明确要求的是「按真实会话看」。

## 三个数都能自己核,不用信我

```sql
select count(*) from archived_sessions;   -- 该是 183,实际 271
select count(*) from projects;            -- 该是 38,实际 1
```

真值在 resume-contexts/SESSIONS-FLAT.csv(183 行)和 by-session/(183 个文件),
不需要重新洗。

## 我要的

按 D1/D2/D3 三个数判定。**达不到就结 failed 并写清差在哪** ——
结 failed 比结一个证据不足的 done 有价值得多,这个项目已经为后者付过好几次代价。
