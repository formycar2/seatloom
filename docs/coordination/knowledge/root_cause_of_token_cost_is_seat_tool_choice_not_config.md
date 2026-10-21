---
id: root_cause_of_token_cost_is_seat_tool_choice_not_config
source_kind: canonical_event
source_ref: AEGIS-2026-10-21-seat-token-hotspots-v1
session_scope: seatloom
title: token 消耗的大头是席位用 bash heredoc 写代码,不是压缩/thinking 配置
domain: cost
modality: decision
status: active
curation: authored
confidence: inferred
owner: aegis
created_by: aegis
created_at: 2026-10-21
supersedes: null
evidence_refs:
  - docs/coordination/tasks/aegis/AEGIS-2026-10-21-seat-token-hotspots-v1.md
  - docs/coordination/ENGINEERING_STANDARDS.md
---

## 实测(11+ 个席位 session,2026-10-21)

全队 token 构成:**toolCall 占 36%,高于 thinking 的 17%。**
我此前判断「压缩阈值 / thinking 是主因」是错的 —— 那两个已修,但只占两成量级。

toolCall 里 bash 占一半,bash 里 **58% 是把代码塞进 `python3 - <<'EOF'` heredoc 改文件**。

**同一份代码付两次**:一次当命令参数发出去,一次之后 read/diff/上下文里再看回来。

`edit` 只要改动的 old+new 片段,自带 diff、可回退、可 dry-run。走 edit 省掉第一笔全额。

按席位(heredoc 占该席位改代码手段的比例):
mira 61% · nimbus 45% · flux 29% · lyra 30% · onyx 34% · **vela 14%**(最低,且 edit 用量全场最高)。

**vela 最低证明这是习惯不是天性** —— 而 mira/nimbus 从没被给过「改代码用 edit」这条指令。

## 判据(已写进 ENGINEERING_STANDARDS.md「Tool use and token cost」)

1. 改代码用 `edit`,不用 bash heredoc 包代码。heredoc 只留给测试夹具、一次性数据处理、非代码文件。
2. 不写 `cd <绝对路径> &&`(席位 bash 已在自己 worktree 起,cwd 本就对)。全队这一项 ~75k token,纯浪费。
3. 每条派单都要带这三句(派单模板是每个席位每次干活前必读的东西,杠杆最大)。

## 附带:压缩机制一个已证实的 bug

`prepareCompaction` 在 `keepRecentTokens` 非零时几乎必返回 undefined,
而 `_runAutoCompaction` 对此静默 return false(不发事件不报错)——
**上下文很小的 session 永不压缩**。已在 saih 树上修,与截图 token 估算 bug 同一棵树。

## Aegis 自己犯过的错(记下来免得重犯)

我曾派一张 token 卫生单给 Nimbus,瞄准「别重复 cd、别重读文件」——
但 cd 只占 bash 的 8.1%(小鱼),heredoc 才是 58%(大鱼)。
**打小鱼放大鱼。** 教训:派优化单前,先量清占比,再决定打哪里,别凭直觉挑看得见的那个。
