# disp-fwdrec-aegis — 交付说明

| 字段 | 值 |
|---|---|
| template | T3 |
| subtype | delivery |
| id | AEGIS-2026-10-10-fwdrec-delivery-v1 |
| status | delivered |
| author | aegis |
| date | 2026-10-10 |
| 派单 | disp-fwdrec-aegis(D5 Phase 2 Lane B 前向记录协议) |

## 改了什么

- **新建 `AGENTS.md`**(主检出根目录,track 分支)。Aegis 此前**没有工作协议文档** ——
  六个席位各有 AGENTS.md,只有总控没有。这本身就是它反复成为流程缺口的一部分原因。
- 其中 `### 前向记录(complete_dispatch 之前,D5 Phase 2 Lane B)` 一节
  **逐字抄自** `docs/coordination/FORWARD_RECORDING_PROTOCOL.md` §5,未改一字。
- 并在完成协议里把它排在第 4 步(提交 → 交付说明 → emit_event → **前向记录** → complete_dispatch)。
- Aegis 额外承担两条:张先生纠正时记 `preference`(恒 quoted);裁决完跨席位升级后记 `failure`(根因+修法)。

## 顺带固化了今天的教训(全部复发过,写进 AGENTS.md 以免再犯)

1. Aegis 没有 waker(唤醒服务日志明说 `NOTIFY for unmapped seat seat-aegis-001 — ignoring`)——
   这是 2026-10-10 第三次停摆的根因;替代方案是 cron 自醒,**会死于会话结束,换会话要重建**。
2. 查席位存活要查 pane 子进程,裸 `ps|grep pi` 会误报全队宕机(pi 是 22 字节符号链接);
   且 pane 里可能跑着别的 agent(当日 Onyx 被 `gemini --resume` 占据,卡在 OAuth 超时)。
3. 验证靠账本:`ready`→`in_flight` 是 MCP 真通的唯一证据。
4. 席位连 MCP 需三条件全满足:pi + 自己的 worktree + `--approve`,缺一静默失效。
5. **条件没满足绝不发门事件** —— 向账本谎称某事已发生,比卡住更糟。
6. 集成席位分支**提交前必须先编译**,且必须取跨 crate 的完整连贯变更集;
   整分支合并会删掉 track 上已有文件。
7. 汇报纪律:禁用自造缩写;每项检查都必须能报错(曾写出 `&& echo OK || echo OK`);
   变量紧跟全角标点要 `${var}`。

## 怎么验证的

- `AGENTS.md` 的前向记录段落与 `FORWARD_RECORDING_PROTOCOL.md` §5 代码块逐字一致(抄录,非改写)。
- 完成协议的步骤顺序与其它席位 AGENTS.md 一致(提交 → 交付说明 → emit_event → 前向记录 → 结单)。
- 本文件即第 2 步要求的交付说明。

## 需上报的

🔴 **本协议的第 4 步此刻无法执行**:`remember` 需要 `memories` 表,而该表**尚未落到 live 数据库**
(实测 `select count(*) from pg_tables where tablename='memories'` = 0)。015 在 Onyx 分支上、018 还没写出来。
所以本次交付**只安装协议,未能自用** —— 这是个诚实的 `unresolved`,等表落库后补记。

⚠️ **Aegis 被权限分类器封顶,无法自行把迁移落到 live**(当日被拦 4 次,每次都拦对了;
连改自己的权限文件也被拦 —— 这条尤其正确:能自发权限的 agent 等于没有权限系统)。
因此 015/018 落库必须走席位交付 + 张先生执行,不能由 Aegis 代办。

⏸ **当前全队串行在 Onyx 身上**:nimbus 与 lyra 被 018 挡住,mira 等 017,flux 无待验之物。
空闲不花钱,但墙上时间被单一席位卡住 —— 若需提速,应考虑把 schema 工作再拆,或接受串行。
