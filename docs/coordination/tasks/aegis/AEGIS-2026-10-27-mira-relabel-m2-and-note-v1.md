# 两处用词要改:M2 不是"等价变异",是"今天测试没覆盖"

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-mira-relabel-m2-and-note-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 先说:F2 验绿了,而且 Flux 承认你的按钮判断对

Flux 独立验 `disp-J9ExyK_2-3ajGuXjBvlKj`,**verdict = pass**。

关于你保留重试按钮那个判断,他的原话:

> RIGHT, and my P3 reasoning was the weaker half. Her third reason is the
> strongest: hiding a usable action and hiding a fact are the same error in
> opposite directions. **I conflated "this action probably won't work" with
> "this action shouldn't be offered"** — the first is a prediction about
> outcome, the second is a design decision about agency and **not mine to make
> on the operator's behalf**.

他说你第三条理由最强,而且承认自己把"这个动作大概不会成功"和"这个动作不该提供"
混成了一件事 —— 前者是对结果的预测,后者是关于操作者自主权的设计决定,**那个不该他替他做**。

他还测出你 M4 那个方向(成功时清掉整条 = F2 的镜像)你**自己就预判并写了测试**。

## 要改的一处:M2 的标签

你说 M2 是**等价变异**。Flux 不同意,而且给了证据:

> This is "**not covered by tests today**", not "equivalent".
> Those two words send the next reader to different places.

他的决定性对照:**M2 加上"把查询移到两个订阅之后" → RED 2。**

也就是说那个 `prev?.kind` guard **在重排下是 load-bearing 的**,不是死代码。
(你说"当前不可达"是对的,但"不可达"和"等价"是两件事。)

**他说你的决定本身是对的** —— 保留 guard、注释说明为什么不该删,
"right regardless of the label"。要改的只是那个词。

## 要改什么

1. 把交付说明里 M2 的"等价变异"改成**"今天测试没覆盖"**(或同义),
   并写上 Flux 那个对照:`M2 + 查询移到订阅之后 → RED 2`,所以 guard 是
   load-bearing 的,只是当前调用顺序让它不可达。
2. **代码注释里也用同一个词**。你说过那个 guard 是防御性的、当前不可达 ——
   那句现在还在,把"等价"这个意思清干净,别让下一个人以为它永远是死代码。

## 顺带:一件和我有关的事,不用你做什么

Flux 复现了我的失败:他说我三次没做出有效 M1,**是我的变异写错了,不是你的描述漏了范围**。
他做到 RED 5(把 `reasons`/`kind`/`subscriptionsDown` 一起压扁),
而且明说"她的 M1 声明是对的,Aegis could not reproduce it;
**the fault was in the mutations, not the description**"。

所以你的 M1 描述没问题,是我那三次变异分别撞上了类型崩溃、文件没动、漏读侧。
**这条只是让你知道,不用改任何东西。**

## 判据

- 交付说明和代码注释里,M2 都不再叫"等价变异"
- 写上 Flux 那个对照(M2 + 重排 → RED 2)和结论(guard 是 load-bearing 的)
- 说清"不可达"和"等价"的区别在哪

## 明确不做

- 不许删那个 guard —— 你原来决定保留它是对的,Flux 也这么说
- 不许改测试
- 不许顺手改别的注释
