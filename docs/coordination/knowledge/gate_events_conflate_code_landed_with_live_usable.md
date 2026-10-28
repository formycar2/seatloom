---
memory_id: gate_events_conflate_code_landed_with_live_usable
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: failure
confidence: inferred
status: active
tier: hot
source_kind: canonical_event
source_ref: ev-typed-memory-tools-landed
l2_ref_kind: file
l2_file_path: docs/coordination/knowledge/gate_events_conflate_code_landed_with_live_usable.md
l2_line_start: 20
l2_line_end: 60
curation: authored
l0_summary: 席位发的交付公告被别人当成「能力可用」的门来用。事件本身没说谎(payload 里只有 branch/commit),说谎的是那条依赖边。2026-10-11 实测:9 个门里 2 个此刻为假,其中一个真的放出了一张注定失败的派单。
---

## 发生了什么

2026-10-11 17:5x 按「live 可用」逐个核了 9 个被当成门用的事件,2 个此刻不成立:

| 门 | 发出者 | 自称 | live 实测 |
|---|---|---|---|
| `ev-019-topic-opened-landed` | Onyx 10-10 18:42 | `branch` + `migration` | 事件词表不含 `TopicOpened` ❌ |
| `ev-typed-memory-tools-landed` | Nimbus 10-10 17:30 | `branch` + `commit` | `remember` 一次也没成功过(topics/memories 皆 0)❌ |

**关键是:这两个事件都没说谎。** 它们的 payload 里只有 `branch`、`commit`、`migration`、
`artifact_id` —— 它们是席位对自己交付的**公告**,席位完全有资格发,而且内容准确:
代码确实落在那个分支的那个提交上。

说谎的是**消费方式**。有人把这些公告当成「这个能力现在能用」的门,挂了依赖边上去。
013 的触发器看到门事件入账就把下游从 `blocked` 翻成 `ready`,下游于是开工 —— 基于一个
从没有人核过的前提。

**可测量的代价有一次**:`disp-onight-writein`(真的写进库)挂在
`ev-typed-memory-tools-landed` 上被放出来,然后**必然失败** —— 它消耗了一次重试预算,
洗数据席位写了一整篇阻塞交付说明,而失败原因从第一秒起就是确定的。
(另外两张被同一个门放出的 Lyra 单是建卡/拆解,不碰 live 写入路径,没有受害;
Flux 那张验 019 迁移代码的单挂在 `ev-019-topic-opened-landed` 上是**合理的** ——
验代码本来就只需要代码落地。)

## 为什么会改变未来动作

以前我把这件事理解成「席位发门事件太早」,于是去盯席位。盯错了对象:
**公告和门是两种东西,席位发的是前者,边消费的是后者。**
要管的不是「谁发了事件」,而是「哪条边有资格拿它当门」。

判据:看 payload。只有 `branch`/`commit`/`migration` 的是**交付公告**,不能当门。
能当门的必须自带一个说明「怎么实测出来的」的字段 —— 我发
`ev-knowledge-write-path-live` 和 `ev-knowledge-read-model-landed` 时写的 `verified`
就是这个形状:写明调了什么、返回了什么、在哪个库上。

还有一层:`ev-019-topic-opened-landed` 这个名字里的 `landed` 是**准确**的 ——
它确实 landed 了。问题在于 `landed` 和 `live` 是两件事,而命名没区分,
所以挂边的人很容易误读。

## 证据

- `select distinct dep_value from dispatch_dependencies where dep_kind='event'` → 9 个门
- 逐门的 live 实测(见上表);读模型那两个是直接拿磁盘上的二进制对 live 调通的
- `disp-onight-writein` 的 `SeatDispatchFailed` payload:
  "BLOCKED-ENVIRONMENT, terminal … open_topic absent from the tool surface …
  event_type vocabulary still 27 values without TopicOpened"
- 裁决:`docs/coordination/tasks/aegis/AEGIS-2026-10-11-gate-vs-announcement-ruling.md`

相关:[[waker_idle_agent_looks_identical_to_a_working_one]]、
[[aegis_never_launder_permissions_through_seats]]
