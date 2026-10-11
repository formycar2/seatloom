# Vela — disp-onight-writein-v3 交付说明(S5 真实入库)

- **派单**:`disp-onight-writein-v3`(夜间 step 3,三次派,图门禁)—— S5 真实入库
- **席位**:`seat-vela-001`(Vela,语料清洗)
- **分支 / 提交**:`packet/vela/washing-pipeline` @ 本提交
- **依赖边**:`dep_kind=event, dep_value=ev-live-019-applied` —— 关单前已确认 referent 存在
- **门事件**:`ev-Fq5oiUH9PgCLaVk_6xlBb`(ArtifactCreated,subject `art-writein`,actor `seat-vela-001`,context `disp-onight-writein-v3`)

---

## 0. 先说一件必须说的事:这一单不是「补一次 complete_dispatch」就完事的

唤醒消息让我按「交付早做成、只是没结单」来收尾,引的证据是 10-10 17:58 的
ArtifactCreated、commit `e72a250`、Flux 已验绿 `ev-flux-verdict-wash-p2`。

**那批证据属于 `disp-wash-vela-p2`(S3 抽取 / S4 分层 / S5 暂存,88 条定型记录),
不属于这一单。** `e72a250` 的提交信息写得很清楚:`feat(wash): D5 Phase 2 Lane C
Part 2 — S3 extraction / S4 layering / S5 staged storage (88 typed records)`。
它是「记录已定型、等待入库」,不是「记录已入库」。

关单前我实测了 live 库的状态,证据(都可复跑):

- `get_knowledge_tree(project=seatloom)`:24 个 `wash-*` topic 全部存在,
  但 **13 个 segment topic 的 memoryCount 全是 0** —— 一条记忆都没进去;
  全项目唯一有记忆的 topic 是 `topic-aegis-fleet-operations`(1 条,是 019 应用后
  那次实证 remember 留下的)。
- 账本里没有 subject 为 `art-writein` 的 ArtifactCreated;
  `ev-vela-writein-v2-artifact-20261011`(10-11 10:44)的 payload 自己写着
  `inserted: 0`、`gate_event_ev_writein_landed_emitted: false`。

所以这一单的活**没有做完**。派单 payload 第 9 条写得很死:「Only after real rows
exist … Never without a real success」。拿 p2 的绿 verdict 给这一单结单,就是一次
虚假门声明 —— 比不结单更糟。**我把入库真正做完了,而不是把单结了就完了。**

## 1. 做了什么

按 payload 的操作契约逐条执行:

1. **重验工具面**:本进程广告 `open_topic` + 018 型 `remember`,两者均实证可用
   (24 次 open_topic 全部成功、87 次 remember 全部成功,见 §3)。
2. **24 个 topic**:`open_topic` 逐个调用(父先于子)。结果:24 个全部
   `already open — unchanged, no event`(019 应用后的那次实证已经建过),
   幂等,零事件。
3. **87 条记忆**:全部经 `remember` 写入(每行 + 事件同一事务)。逐条对照
   `writein_manifest.json`,一条不多一条不少。
4. **1 条丢弃**(凑不出诚实行号,计数如实上报):`56ccb8df`
   「Codex history is 247 files / 14GB on disk …」—— 该断言横跨四段互不相干的
   raw 区域(336 行的 14G du、484-501 的 rollout 计数、572-579 的 stepgo 启动目录、
   7024-7044 的席位真实写入位置),没有任何单一行号区间能承载它。丢弃,不硬锚。
5. **脱敏**:入库文本(标题 + l0 + l1)在写库前重跑 S1 同构规则,0 命中。
6. **门事件**:入库成功后才发 `ev-Fq5oiUH9PgCLaVk_6xlBb`。

## 2. 给张先生的Plain-language:洗出了多少,丢了多少,为什么

- **原料**:seatloom 一个 project,16 个会话,58,017 行 / 167.6 MiB
  (2026-04-24 → 2026-10-10)。
- **真正送进 LLM 的**:25,156 个 model turn / 23.6 MiB —— **保留 14.1%**。
  剪掉的是 tool_result 正文、重试轮次、元数据行;每条 canonical turn 都带
  `src_file + src_line` 指回 `raw/`,剪枝不碰这些指针。
- **产出**:87 条定型记忆(42 decision / 26 failure / 13 fact / 4 preference /
  2 unresolved),挂在 24 个 topic(1 根 + 10 会话 + 13 片段)下。
- **脱敏**:1,026 次命中,0 存活;47 个不同的「类型+指纹」对
  (样本 `⟪REDACTED:intranet_domain:7944ea⟫` 出现 363 次)。同一密钥只露指纹,
  不露内容。
- **丢了 1 条**(上面 §1.4),丢的原因是它的一条断言需要四段证据,任何单一
  行号区间都是假锚。宁可少一条,不留一条不能回溯的。
- **诚实的代价**:另外 5 条做了证据充分的重新锚定(3 条原本按 live transcript
  编号而非 raw/,digest 是空输入 sha256 的特征值;2 条幂等键过期)——
  详见 `s5_writein.py` 的 repair log 与 `writein_manifest.json`。

## 3. 怎么验证的(都可复跑)

| 门 | 命令 / 动作 | 结果 |
|---|---|---|
| 写集校验 | `python3 s5_writein.py --check` | 0 problems;87 to insert / 1 dropped / 5 repaired |
| 单测 | `python3 -m unittest tests.test_l2_pointers tests.test_s0_adapters tests.test_s1_gate2 tests.test_s2_segment tests.test_s5_writein` | **37 tests OK** |
| L2 锚点 | 驱动对全部 87 条逐一重读 `raw/` 并重算 sha256(不是抽样) | 87/87 解析且 digest 重合 |
| topic 落地 | `get_knowledge_tree` | 24/24 topic,13 个 segment topic 计数 = manifest(87 条合计) |
| 逐条读取 | `get_topic_memories(depth=l2)` | l2 带 `filePath` + `lineStart/End` + `bodyDigest`,指回 `~/Documents/GitHub/AgentHistory/raw/...` |
| 门事件 | 账本查询 | `ev-Fq5oiUH9PgCLaVk_6xlBb` 在库,subject `art-writein` |

复跑入口(只读,不重写):

```
get_knowledge_tree(project_id='seatloom')   # 13 个 wash-seg-* topic 的 memoryCount 合计必须为 87
```

## 4. 必须上报的三件事(都是我的缺陷,不是别人的)

1. **门事件 id 与派单点名的不同。** payload 第 9 条点名 `ev-writein-landed`;
   我调 `emit_event` 时**没有传 `event_id`**,服务端生成了
   `ev-Fq5oiUH9PgCLaVk_6xlBb`。事件不可改,我不补发第二个(同一 artifact 两个
   ArtifactCreated 比 id 不一致更糟)。下游若要按字面 id 找,请认这个实际 id +
   subject `art-writein`。教训已写进记忆:派单点名了事件 id,就必须显式传。
2. **我在验证时自造了一条重复行。** 为了拿 remember 的返回结构,我用同一
   `record_id` 改了标题再调了一次 remember —— **remember 没有服务端内容去重**,
   于是多出一行(`mem-XTAYQ6Cx1939qUZV5tDkz`,topic 从 11 条变 12 条)。已用
   `forget` 软删除(原因记录在案,行不硬删)。后果:这一单**不是工具级幂等的**,
   payload 里「idempotent, resumable」在 remember 这层不成立;幂等只存在于驱动
   的记录键上。**任何人不得重跑入库** —— 重跑会再插 87 条重复。已写 checkpoint
   `tools/wash/.local/wash/s5/writein_checkpoint.json` 标注 complete。
3. **payload 引的验收计划包不在我分支上**:
   `docs/coordination/acceptance/2026-10-11-lyra-vela-writein-acceptance-plan-v1.md`
   (我分支上 `docs/coordination/acceptance/` 无此文件)。所以第 10 步的
   carding(按计划 §3 参数、`flux_verdict_ref`、pre-declared verdict id)
   **我没有做**:这一单目前**没有属于自己的 Flux 绿 verdict**,
   `ev-flux-verdict-wash-p2` 是 p2 的,不能借。谁发 verdict、谁来 card,请派单。

## 5. 改动文件

- `docs/coordination/tasks/vela/disp-onight-writein-v3-delivery.md`(本文件)
- `tools/wash/.local/wash/s5/writein_checkpoint.json`(gitignored scratch,标注入库完成)

记忆行与 `TopicOpened`/`MemoryCreated` 事件在 live 库与本条派单的门事件里,
不在文件堆里 —— `raw/` 全程只读,没有另起事实源。

## 6. 前向记录(已写入 `topic-aegis-fleet-operations`)

- failure:remember 无服务端内容去重,「探测一下」= 多插一行(含 forget 处置)
- failure:派单点名了事件 id 却没显式传,门事件落到生成 id 上
- decision:拿别人的证据结单 = 虚假门声明;活没做完就去做完
- unresolved:唤醒消息把本单归到 p2 的已交付证据上 —— 是一次性口误,还是舰队的
  派单状态上报系统性地串单?若为后者,别的席位也可能被示意「拿错证据结单」
