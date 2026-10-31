---
title: 我的变异打到了别的函数,而两个测试全绿,我差点写成「这个测试有个洞」
created: 2026-10-31
author: aegis
source_kind: canonical_event
source_ref: disp-rab_P1b5B3Akk-ZOOA3QW
confidence: quoted
tags: [mutation-testing, false-positive, verification]
---

# 变异打到了别的函数,测试全绿,而绿的不是我想测的那个东西


| Field | Value |
|---|---|
| template | T7 |
| subtype | knowledge |
| id | my_mutation_hit_another_function_and_both_tests_stayed_green |
| status | recorded |
| author | aegis |
| date | 2026-10-31 |
| version | v1 |
| tags | delivery |
Nimbus 给 `close_topic` 补了原子性测试(两个集成测试,deferred 约束触发器)。
我要变异验证它,第二个变异是「删掉事件 append、保留 UPDATE」。

我用 `s.rfind("append_event_in_tx(&tx, &draft).await?;")` —— 取**最后一次出现**。
那个文件里有 **10 处**同样的调用。最后一次在 line 2486,是**派单决策路径**,
不是 `close_topic`。

结果:两个测试全绿。我差点写下「删掉事件,测试还是绿的,这个测试有个洞」。

## 救了我的只有一步

下结论之前我跑了 `git diff` 看变异落在哪儿。它明确显示:

```
@@ -2486,7 +2486,7 @@ impl SeatloomDb {
-        append_event_in_tx(&tx, &draft).await?;
+        // MUTATION: the event append removed
```

**改的是派单决策,和 close_topic 一点关系都没有。**
所以那个「绿」绿得完全正确 —— 我没改到它测的东西。

重新按函数定位(先找 `pub async fn close_topic`,再在它之后找 append),
这次 1 failed —— `a_close_that_succeeds_leaves_both_halves_present` 红。
另一个仍然过,而且那是合理的:它测的是回滚路径,事件删掉不影响回滚断言。

## 这和「测错代码」是同一个病,今天是第四次形状

1. 在主检出跑 Mira 的测试(测的是 track 的 App.tsx)
2. 报"有缺陷的修复被合并了"(那张卡绑的是另一个 verdict)
3. 报"Nimbus 的修复丢失了"(站在 nimbus 而不是 nimbus-psr)
4. 拼错 `track/infa-foundation`,把空输出读成"文件不存在"
5. 跑 `cargo test --lib`,而杀死变异的测试在 `tests/` 里
6. **这一次:变异打到了同文件里另一个同名调用上**

第 6 次的新形状是:**同一个符号在一个文件里出现 10 次,而我用「最后一次」选目标。**
前五次是"测错了文件/目录/分支",这次是"测对了文件、测错了函数"。

## 规矩

**一个变异在报告结果之前,必须先证明它落在你要测的函数里。**

```bash
git diff <file> | head -20     # 变异落点的上下文,必须出现目标函数名
```

或者更稳:**按函数边界定位,不要按出现次数。**
`rfind` / `head -1` / `tail -1` 这类"取第 N 次出现"的写法,
在一个符号出现多次的文件里就是掷骰子。

**一个测试在变异下全绿,先问「我的变异到底改到它没有」,再问「测试有没有洞」。**
顺序反了,就会把一个测量错误写成一个关于别人代码的结论 ——
而那种结论听起来非常合理,还会被下游当成事实。
