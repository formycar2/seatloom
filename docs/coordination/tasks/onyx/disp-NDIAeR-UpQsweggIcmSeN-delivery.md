# disp-NDIAeR-UpQsweggIcmSeN — delivery

Onyx. 2026-10-17. 迁移后第一条派单,兼第四层验真的那块单。

**结论:MCP 真连着。四层全过。可以迁其余六个。**

---

## 1. 你要的原文 —— `saih mcp list`

在我自己的工作区跑(`cd ~/Documents/GitHub/seatloom-worktrees/onyx`),原文如下:

```
$ ~/Documents/GitHub/seatloom/saih/bin/saih mcp list
seatloom: connected, 22 tools (codemode, project)
  /Users/jyxc-dz-0100609/Documents/GitHub/seatloom/target/debug/seatloom-mcp-emit
  tools: emit_event, enqueue_dispatch, get_next_dispatch, complete_dispatch, cancel_dispatch, request_acceptance, cancel_acceptance, get_acceptance_lineage, recall, remember, promote, forget, resolve, get_dispatch, query_ledger, open_topic, get_knowledge_tree, get_topic_memories, plan_intent, list_history_projects, list_history_sessions, get_history_session_turns
```

exit code 0。`seatloom: connected, 22 tools` 正是你要看到的那行。
工具数我单独数了一遍:**22**,和报的对得上。

## 2. 四层验真,逐层的实测结果

| 层 | 判据 | 结果 | 证据 |
|---|---|---|---|
| 1 | 面板里跑的是 saih | ✅ | pane 3408 的子进程是 `saih`(pid 58238),它的子进程是 `seatloom-mcp-emit`(58411) |
| 2 | 从自己工作区 `mcp list` 读到工具 | ✅ | 见上面原文,22 tools,exit 0 |
| 3 | 唤醒服务进程名门槛含 saih | ✅ | `scripts/aegis-self-wake.sh:70` = `node\|bun\|pi\|saih` |
| 4 | 账本上一条派单 ready → in_flight | ✅ | `disp-NDIAeR-UpQsweggIcmSeN`:`SeatDispatchDelivered` 08:47:28Z,dispatches 行 = `in_flight` |

第 1 层我特意**没有**用 `pgrep -P`。上一次派单我发现 `scripts/saih-verify-seat.sh:56`
踩了 macOS pgrep 看不见调用者祖先那个坑,而我自己 bash 的祖先链正是
`saih(58238) ← pane(3408)`,所以 `pgrep -P 3408` 会返回空。
改用 `ps -axo pid,ppid,comm | awk '$2==3408'`,一次就拿到了。
**那条坑在你从席位自己面板跑的时候必然踩到**,迁移后自查也会踩 —— 现在有多一个现场证据了。

## 3. 一处对不上,报给你(不替你圆)

你说「记忆也接上了(369 条,0 丢弃)」。我数出来的 live 库是:

```
memories = 233     (active 231 / forgotten 2)
```

差 136。我查了是不是有批量搬入:按小时分组看 `memories.created_at`,
没有突增,最近一小时才 2 条,全是自然增长(133 canonical_event / 90
archived_session / 6 manual / 4 document)。所以不是「搬进来了但我没看到」。

**我没有证据说 369 是错的**,只能说我量到 233。可能是你数的口径不同
(比如把某个关联表或事件一起数了),也可能是别处看到的数字。
我没有据此做任何判断,也没有改任何东西 —— 只是不想在交付里复述一个
我没能复现的数字。

## 4. 这条派单的边界(按你 🔴 的要求)

- 不改代码 ✅ —— 本分支无任何代码改动,只加了这份交付说明
- 不动 live 库 ✅ —— 全程只读;上面的 SELECT 都是 `pg_database` 侧的只读查询,
  MCP 侧只发生了派单自身的翻牌(那是账本自己的事,不是我写的)
- 不开验收卡 ✅
- 不发门事件 ✅ —— 只发了一条 `ArtifactCreated` 告知(完成协议要求),无 `verified` 字段
