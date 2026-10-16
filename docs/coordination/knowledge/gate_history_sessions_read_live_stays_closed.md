---
memory_id: gate_history_sessions_read_live_stays_closed
project_id: seatloom
topic_id: aegis-gate-rulings
kind: decision
confidence: quoted
status: active
tier: hot
source_kind: canonical_event
source_ref: ev-flux-history-sessions-verdict-artifact
l2_ref_kind: file
l2_file_path: docs/coordination/knowledge/gate_history_sessions_read_live_stays_closed.md
l2_line_start: 1
l2_line_end: 60
curation: authored
l0_summary: 门事件 ev-history-sessions-read-live 在 2026-10-16 判定保持关闭。Flux 的结论原文写明它"只有 (a)(b)(c) 之后才变成可发",而 (a) 修复未做、(b) 未重量、(c) 张先生未跑 apply-schema-to-live.sh,live 库连 message_count 列都没有。已派单给 Nimbus 做 (a)(b),门仍关着。
---

## 裁决(2026-10-16 11:58,Aegis 心跳第 3 步)

**`ev-history-sessions-read-live` 不发。** `disp-history-sessions-ui`(mira)继续 blocked。

这不是"再等等看",是核过之后确定不够格。判据是 Flux 结论的原文,
它把顺序写死了,而且我逐条核了 live 现状。

## Flux 结论要求的顺序(原文要点)

> Required to reach PASS (in order):
> (a) Nimbus adds the `message_count IS NOT NULL` filter to the two project-grouped reads
>     and fixes the `dated` flag;
> (b) the six numbers are re-measured against a **live-shaped** scratch DB
>     (271-row inventory + 021), not a fresh one;
> (c) the operator runs `apply-schema-to-live.sh` and the counts are confirmed on the live DB;
> (d) **only then** does `ev-history-sessions-read-live` become emittable and Mira's UI
>     slice unblockable.

## 我核到的 live 现状(2026-10-16 11:5x,逐项实测)

| 条件 | 状态 | 证据 |
|---|---|---|
| (a) 加过滤 + 修 dated | **未做** | nimbus 分支 `crates/seatloom-core/src/db/repositories.rs` 的 `list_history_projects` 仍是 `LEFT JOIN archived_sessions a ON a.project = p.id` + `HAVING count(a.id) > 0`,没有 `message_count IS NOT NULL` |
| (b) live 形态重量 | **未做** | proof 脚本只在全新空库上跑过,Flux 明确指出这正是它一直通过而产品是错的原因 |
| (c) 021 落到 live | **未做** | live:`archived_sessions` 271 行、**没有 `message_count` 列**、`projects` 只有 1 行;track 的 schema 只到 `019`,`021` 不在 track 上 |
| (d) 发门事件 | **关闭** | 以上三条一条都没满足 |

另外核到:正在跑的二进制自报 19 个工具,**一个历史读取工具都没有**
(`list_history_projects` / `list_history_sessions` / `get_history_session_turns` 都不在)。

## 为什么会改变未来动作

- **这道门的形状是" measured,不是 announcement"**。派单原文就写明:
  "your only edge is ev-history-sessions-read-live — the MEASURED gate, not the announcement.
  You call this capability, so an announcement is not enough for you."
  所以"代码合了""工具注册了""二进制自报有它"全都不够 —— 要能真的调成功一次。
- **Flux 已经给出精确到 SQL 子句的缺陷定位**,不需要任何人重新判断:
  项目分组计数把 89 行清单行重复算了,`sum(sessionCount)=261` 应为 183,
  38 个 project 里 10 个错(`sf-code/storage` 报 58 真实 2、`seatloom` 报 16 真实 14、
  `(root)` 报 13 真实 8)。附带缺陷:`dated = first_ts.is_some() || session_date.is_some()`
  把带 manifest `session_date` 的清单行标成 `dated:true`,而 SQL 把它们排进未分组块。
- **已派单**:`disp-LpG-versCbWNywWoM7cK3` → `seat-nimbus-001`,status=ready,priority 70,
  max_retries 2,无依赖边。覆盖 (a)(b),规格逐条引自 Flux 结论,没有自行发挥。
  账本事件 `ev-Huk7Nfar2elGNmBqDovvb`。
- **(c) 是张先生的动作**(`scripts/apply-schema-to-live.sh` 写 live 库,我被权限封顶)。
  门在他确认 live 计数之后才开,由 Aegis 发。**在 (a)(b)(c) 全部完成前,
  任何会话都不必再讨论这道门开不开** —— 答案就是关着,除非出现了新的实测证据。

## 证据

- Flux 结论:`docs/coordination/tasks/flux/FLUX-2026-10-14-history-sessions-verdict-v1.md`
  @ `f5bbe50`(在 `packet/flux/intent-verdict` 上,track 上没有,按共享 ref 命名空间读)
- 账本事件 `ev-flux-history-sessions-verdict-artifact`(Flux,2026-10-14 11:48)
- live 实测:`select count(*) from archived_sessions` = 271;
  `information_schema` 无 `message_count` 列;`select count(*) from projects` = 1
- 二进制实测:`tools/list` 19 个,无 history 工具
- nimbus 分支 SQL 实测:`git show packet/nimbus/gate-vs-announcement:crates/seatloom-core/src/db/repositories.rs`
