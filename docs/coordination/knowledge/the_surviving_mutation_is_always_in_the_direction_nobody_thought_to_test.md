---
title: "Lyra found the surviving mutation is always in the direction nobody thought to test"
category: knowledge
topic: verification
tags: [mutation-testing, surviving-mutation, test-gap, lyra-standing-duty, consumptive-skip]
created: 2026-10-29T14:30:00+08:00
source_kind: canonical_event
source_ref: ev-lyra-mutation-2-survives-the-consumptive-skip
confidence: inferred
curation: authored
---

# Lyra 发现:存活的变异总是在"没人想到要测"的那个方向

## 经过

Mira 返工冷启动修复(拆开那两个 effect)。她自己做了变异测试,报了绿。

Lyra 的值班职责是验证,**她没有只重跑 Mira 的数字,而是自己又变异了三个。**

### 她的处方比 Mira 的设计弱,她自己说出来了

Lyra 原本让 Mira 用 `loadedUIProjectId === null` 当冷启动信号。
Mira 没照做,写了一个 consumptive skip:

```ts
const skipColdStart = !restoredPosition || coldStartSkip.current === activeProjectId;
if (coldStartSkip.current === activeProjectId) coldStartSkip.current = null;
if (!skipColdStart) { /* 应用已存 tab */ }
```

Lyra 记录:**这个设计更好,而且她写明为什么自己的处方更弱。**
Mira 的门不问"哪个项目",问"恢复落地了没有",而且 skip 是**消耗性的** ——
触发一次就自我清除。于是一个不变式覆盖三种情况:

- 冷启动 → skip 触发,留在派单页
- 切到项目 B → skip 已消耗,B 的 tab 恢复
- **切回被恢复的项目** → skip 已消耗,它的 tab 也恢复

她的 `loadedUIProjectId === null` 版本前两种情况能过,第三种要再加一个条件。

**一个能说出"我开的处方比对方实现弱"的验证者,比一个只报绿勾的有价值得多。**

### 变异 2 存活,而且它破坏的是操作者要的功能

| 变异 | 结果 |
|---|---|
| 1. 删掉冷启动 skip | **被抓到** —— 2 failed \| 8 passed |
| 2. 让 skip 永久化(删掉消耗那一行) | **存活** —— 10 passed |

变异 2 的后果:切项目不再恢复那个项目的 tab,切回去也永不恢复。
**每个项目的 tab 记忆静默消失,而整个测试套件全绿。**

而那正是操作者要求的功能,也正是 Lyra 在裁定里明确说"不可省略"的那一半。

### 为什么套件漏了它

测试名叫"会话内切项目仍恢复该项目的 tab",在变异 2 下**通过**。
因为覆盖到的情况是:冷启动、切一次、二次启动、后端恢复几种。
**缺的恰恰是这个 consumptive 设计唯一依赖的那种情况:切走,再切回,tab 必须还在。**

## 被命名的模式

> 这是本周第三次变异在报绿的套件里存活,而这个模式已经稳定到可以命名:
> **存活的变异总是在没人想到要测的方向,从不是修复针对的方向。**

前两次:
- Nimbus 的去重测试匹配自己的断言文本(测试朝内看,不朝外看)
- Nimbus 的镜像主修复没被钉住(测试重新实现了 `min`,而产品里是内联的)

## 规矩

### 1. 验证者要自己变异,不能只重跑作者的变异

Mira 自己变异过、报了绿。Lyra 又变异三个,抓到一个她没抓到的。
**作者变异自己代码时,变异的是她想到的方向。验证者的价值在别处。**

### 2. 消耗性/自我清除的门,必须有"第二次"测试

`skip` / `once` / `first-run` / `已消耗` 这类设计,正确性依赖**它只触发一次**。
只测"第一次对"和"切走对",测不出"它是不是变成了永久"。
**必须测:第二次、切回来、重复操作。**

### 3. 绿套件是"这些用例过了"的证据,不是"实现正确"的证据

Lyra 的原话:**A green suite is not evidence of correctness; it is evidence that the suite's cases pass.**

而存活变异的方向总在盲区 —— 所以绿套件系统性地高估正确性,尤其在被修的那个方向附近。

### 4. 验证者该说出"我的处方更弱"

Lyra 拒绝用权限压人,而是记录为什么对方的设计更好,并且采用对方的诊断
("I installed a correct gate at the wrong door" 比她的诊断更有用)。
**验证的产物不只是 pass/fail,还包括"谁的想法更好"。**

### 5. 她拒绝写那个测试,而且理由对

> "a test I write for someone else's code is a test nobody owns"

她还做了不该省略的事:每次变异后从备份恢复 `App.tsx`,
`grep -c skipColdStart` 数回 2、套件回到 10/10 和 370/370 才交给我,
**没有编辑 Mira 的分支**。
