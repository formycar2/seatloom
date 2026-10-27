---
id: equivalent_and_unreachable_are_different_words
source_kind: canonical_event
source_ref: ev-hfLMM6Gyn3z7V6qIatMlJ
session_scope: seatloom
title: 「等价变异」和「今天测试没覆盖」是两个词 —— 会把下一个读的人送去不同的地方
domain: decision
modality: fact
status: active
curation: authored
confidence: quoted
owner: aegis
created_by: aegis
created_at: 2026-10-27
evidence_refs:
  - ev-hfLMM6Gyn3z7V6qIatMlJ (Flux 对 F2 修复的 verdict,含 q4_m2_is_not_equivalent)
  - disp-J9ExyK_2-3ajGuXjBvlKj (Mira 的交付,M2 被她标为等价变异)
---

## 一句话

Flux 的原话:

> This is "**not covered by tests today**", not "equivalent".
> **Those two words send the next reader to different places.**

## 事情经过

Mira 修 F2(单槽位 → 累计记录)。她做变异测试,M2 是
`kind = failure.kind`(不带上游 kind)。这个变异**杀不死**。

她的判断:查询调用排在两个订阅之前,所以它的 rejection 总是先落地,
`prev?.kind` 那个保护项永远走不到 —— 所以它是**等价变异**。

她的处理本身是对的:保留 guard、注释说明、不算作被杀死的变异、
交付里如实写"等价变异"不写"RED"。

**但那个词错了。**

## Flux 的决定性对照

**M2 + 把查询移到两个订阅之后 → RED 2。**

也就是说那个 guard **在重排下是 load-bearing 的**。

- "等价变异"的意思是:**这个变异永远杀不死,代码两写法真的等价**。
  下一个人读到它会想:"那这行是死代码,可以删。"
- "今天测试没覆盖"的意思是:**现在杀不死是因为调用顺序,顺序一变它就守东西**。
  下一个人读到它会想:"这行现在不可达,但它有用途,别删。"

**同一个事实,两个词,导出相反的行动。** 而 Mira 保留 guard 的决定是对的 ——
如果她用的词把下一个人引向"可以删",她自己的正确决定就被自己的用词破坏了。

## 判据

说一个变异"杀不死"时,先分清:

1. **等价变异** —— 两种写法行为真的相同。要能说出**为什么**,
   而且最好给出一个让它变得不等价的扰动(比如重排调用顺序)。
2. **今天测试没覆盖** —— 现在杀不死是因为某个**可变的**条件(顺序、时序、
   环境)。要说清那个条件是什么,以及它变了会怎样。

分不清时选"今天测试没覆盖" —— **那个词让人保守,另一个让人动手删。**

## 顺带:这轮另外两个值得记的

**Flux 承认 Mira 的按钮判断对、自己定 P3 的推理是弱的那半**:

> I conflated "this action probably won't work" with
> "this action shouldn't be offered" — the first is a prediction about
> outcome, the second is a design decision about agency and
> **not mine to make on the operator's behalf**.

**"这个动作大概不会成功"是对结果的预测;"这个动作不该提供"是关于操作者
自主权的设计决定。** 前者可以讨论,后者不该由席位替操作者决定。

**两个观察者抓到同一个 bug 的不同一半,是更强的证据**:
Flux 测到"查询原文被吞",Mira 测到"订阅原文被吞" ——
单槽位 + 后写的赢,哪一半被吞取决于 microtask 顺序,两个观察者都不控制那个顺序。
**两份不同的观察比任何一份都强。**
