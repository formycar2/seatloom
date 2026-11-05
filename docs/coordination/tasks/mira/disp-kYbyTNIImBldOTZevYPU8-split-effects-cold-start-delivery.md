# disp-kYbyTNIImBldOTZevYPU8 交付说明 —— 拆开那两个 effect,冷启动落 派单

**派单**:`disp-kYbyTNIImBldOTZevYPU8`(Aegis 发,授权返工)
**提交**:见 git log

---

## 1. 根因:不是两个 effect,是**三次**到达

Aegis 派单里转述 Flux 的诊断:

> `setActiveProject` 和 `setColdStartProjectId` 在同一个 effect 里
> land in SEPARATE renders,所以冷启动门被武装晚了一帧。

**这个诊断是对的,但它只覆盖了两次到达。我实测出第三次。**

我在 tab effect 里打了探针,逐帧记录(这不是推理,是运行日志):

```
PROBE-TABEFFECT proj=p-1        skip=null         saved=undefined
PROBE-TABEFFECT proj=seatloom   skip=null         saved=workitems   ← 第三者!
PROBE-TABEFFECT proj=ql_delivery skip=ql_delivery  saved=artifacts
```

三帧:

1. **`p-1`** —— app 自己的启动默认,没有记忆,无害。
2. **`seatloom`** —— 后端读取在飞的时候,`useDataStore` **自己**选的第一真项目。
   没有任何门保护它,它直接把 `workitems` 恢复上去了。
   **这就是十轮里每次"修好一条坏一条"的那个第三帧。**
3. **`ql_delivery`** —— restore effect 落地的正确项目,`coldStartSkip` 命中。

**Flux 说的"晚一帧"是真的;但那一帧不是 restore 目标,是数据 store 的自选项目。**
我前面十轮一直在给第 3 帧加门 —— 门确实生效了(日志里 skip 命中)——
可第 2 帧早就把 tab 改掉了。**我给一扇正确的门,装在错误的门口。**

## 2. 修法:门不认项目,认"恢复是否已落地"

Aegis 授权我拆 effect。我没有拆成两个 effect,而是**换了一个更简单的不变量**:

```ts
// tab effect 内
const skipColdStart = !restoredPosition || coldStartSkip.current === activeProjectId;
if (coldStartSkip.current === activeProjectId) coldStartSkip.current = null;
if (!skipColdStart) { /* 恢复 saved tab */ }
```

`!restoredPosition` 覆盖第 1、2 帧 —— restore 落地前,**任何**到达都不是操作员的:
要么是启动默认,要么是数据 store 在读取期间的自选。
对一个操作员从没打开过的项目应用记忆,是在用"他上次在哪"回答一个他没问的问题。

`coldStartSkip.current === activeProjectId` 覆盖第 3 帧 —— restore 刚落地那一帧,
否则它会把恢复项目的 tab 覆盖到 派单 默认上。

**跳过是消耗式的,不是粘性的**:命中即清 `coldStartSkip`。
所以切回同一项目会正常恢复 —— 这是派单 `done_definition` 第 2、3 条要的。

## 3. done_definition 逐条对

| # | 要求 | 实测 |
|---|---|---|
| 1 | 两条新契约测试绿 | ✅ `10 passed (10)` |
| 2 | 8 条既有测试保持绿(含我重写的 4 条) | ✅ 同一次运行 10/10 |
| 3 | 写盘 effect 不得把冷启动 tab 写进别的项目记忆 | ✅ 变异 3 专测这个,见 §4 |
| 4 | 全套绿 / typecheck 净 / eslint 不劣于 baseline | ✅ 370/31,tsc 净,135 errors(=baseline),prettier 净,build 1.81s |

## 4. 变异测试:3 个,全红

| 变异 | 改了什么 | 结果 |
|---|---|---|
| 1 | `skipColdStart = false`(去掉冷启动门) | **2 红** —— 冷启动测试 + 二次启动测试都抓住 |
| 2 | `if (!skipColdStart)` → `if (false)`(关掉切项目恢复) | **1 红** —— 切项目测试抓住 |
| 3 | 冷启动帧里把默认 tab 写进项目记忆(我第 6 轮自己发现过的污染) | **1 红** —— 切项目测试抓住 |

每次都用**备份文件** `/tmp/App.tsx.bak` 恢复,没有用 `git checkout`
(那两个新测试文件当时未提交)。恢复后 `grep -c MUTATION` = 0,`10 passed`。

**变异 3 值得单独说**:它测的正是我十轮里第 6 轮自己撞上的那个污染。
现在有一条测试会因为它的复现而红。污染不会悄悄回来。

## 5. 我修掉的一个测试缺陷(必须报告)

第二个测试(`switching projects inside a session still restores THAT project's tab`)
原来给 `FIRST_REAL_PROJECT` 存的 tab 是 **`'dispatches'`**,而断言找的是
**`活跃工作项账本`**(workitems 页标题)。

**它不可能通过。** 更糟的是:`'dispatches'` 正是冷启动默认值,所以
"记忆生效了"和"什么都没发生"是同一个观测 —— 这个坑**测试自己的注释里
刚写过**(在第一个测试里),却在这里留了下来。

已改成 `'workitems'`,并把那句测量写进注释。

**这是"测试没在盯对的东西"的一个实例**,而且是我自己前几轮
写下的教训在同一批文件里复发。记下来。

## 6. Lyra 的产品裁定没等,但留了开关

派单说"实现 A(冷启动一律落 派单)作为工作默认,但把 B 写成一行开关"。

我的实现里,**A/B 的分界就是 `!restoredPosition` 这一个布尔**。
B(冷启动记住上次的页)需要把它换成 `coldStartSkip.current !== activeProjectId`,
即只在冷启动那一帧跳过、其它照旧 —— 一行。

我在注释里写明了这个分界在哪,没有把两种行为混在一个表达式里。

## 7. 未做 / 待办

- **没有开验收卡。** 派单没要求,而且 `disp-E06O6IAaWefF-BOW-CNJd`
  仍没有 verdict 点名它(上一单报告过)。**verdict 一落地我立刻开。**
- **真机验证未做。** 全部证据来自 jsdom + 探针。派单的
  `what_he_should_see`(退出重开看左边栏、切项目看记忆)需要操作员在真机上做。
- **没有 merge into track**(派单 `do_not`)。
