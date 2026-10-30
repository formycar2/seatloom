# disp-EVFMbz9OxT2gyoFlYfzBE 交付说明 —— 按裁定返工:持久化从操作员输入来

**派单**:`disp-EVFMbz9OxT2gyoFlYfzBE`(Lyra 发,带裁定)
**门禁**:372 tests / 31 files,tsc 净,eslint 135 errors = baseline,prettier 净,build 2.17s

---

## 1. 裁定落地:写盘 effect 已删除,不是收窄

按 `done_definition` 第 3 条:**REMOVED, not narrowed**。

原来那个 effect(`if (!restoredPosition || loadedUIProjectId !== activeProjectId) return;`
+ `updateProjectUIState` + `setActiveProjectId`)**整块删掉**。

新增一个 wrapper,`chooseTab(tab)`:

```ts
const chooseTab = useCallback((tab: Tab) => {
  setActiveTab(tab);
  if (activeProjectId && activeProjectId !== 'all-projects' && tab !== 'all-projects') {
    updateProjectUIState(activeProjectId, { activeTab: tab, ... });
    setActiveProjectId(activeProjectId);
  }
}, [activeProjectId, selectedObject, updateProjectUIState, setActiveProjectId]);
```

**8 处 `setActiveTab` 里,7 处操作员驱动 → 全部改走 `chooseTab`;**
**restore 那一处(:489)保留裸 `setActiveTab`,按裁定它必须不持久化。**

没有把 `updateProjectUIState` 摊到 8 个调用点 —— 裁定说得很清楚,那是 8 次
机会忘一个,忘一个就是静默丢位置。

## 2. 我保留了一样裁定没要我保留的东西

原来那个 effect 里还有一句 `setActiveProjectId(activeProjectId)` ——
**记住在哪个项目**。它和"记住哪一页"不是一件事:

- 页 → 只该由操作员输入触发(裁定)
- 项目 → 切项目也必须记住,而切项目不走 tab 变化

所以我把它拆成一个独立的小 effect,只写项目 id,不碰 tab。删掉它会让
`does not restore a project the backend did not return` 那条测试红
(它断言"记录被修复成真项目")。

**裁定管的是 tab 持久化,不是"别再记住项目"。** 我把这两件事分开,并在注释里
写明为什么分开。

## 3. 变异测试:4 个,3 个红,1 个我判定为无效变异

| # | 变异 | 结果 |
|---|---|---|
| 1 | `chooseTab` 不持久化(**派单点名要的那个**) | **1 红** —— 新测试 `an operator-driven tab change IS persisted` 抓住 |
| 2 | `skipColdStart = false`(冷启动门失效) | **4 红** |
| 3 | 删掉消耗 skip 那行(Lyra 的变异 2) | **1 红** —— `switching AWAY and then BACK` 抓住 |
| 4a | restore 改走 `chooseTab` | **存活** |
| 4b | restore 直接调 `updateProjectUIState` | **存活** |
| 4c | restore 在冷启动帧把默认页写进记忆(第 6 轮污染) | **5 红** |

### 关于 4a / 4b:我判定它们是**无效变异**,不是测试缺口

4a 和 4b 都存活。我查了为什么 —— 用探针实测:

```
MUT4a/4b 下: appStoreWrites.total = 1(只有基线那次)
             projectUIStates[RECORDED].activeTab = 'artifacts'(未被改)
```

**restore 那一行在冷启动帧根本不执行** —— `skipColdStart` 为 true,进不到
`if (!skipColdStart)` 分支。4a 改的是一行死代码。

4b 在**切项目**帧会执行,但它写的是 `savedState.activeTab` —— **等于该项目
已有的记忆值**。磁盘前后一致,不可观测。

**这不是"测试没盯住",是变异本身是同义反复。** 我把 4b 换成语义正确的版本 4c
(把当前 `activeTab` 写进记忆,即第 6 轮污染的真身),它被 5 条测试抓住。

**我在这一条上多绕了两轮** —— 先做了 4a,发现是死代码;再做 4b,发现是同义反复。
应该在改第一处之前先确认那一行在什么帧执行。记下来。

## 4. 新测试:`an operator-driven tab change IS persisted — the wrapper is not decorative`

派单原话:"If nothing goes red, the wrapper is decorative and the position loss
is silent again."

**它当时确实什么都不红** —— 我第一版测试只断言了"冷启动不写盘",没有断言
"操作员点了会写盘"。补上后,变异 1 立刻红。

测试走 nav rail 的真实按钮(`getByTitle('工作项')` —— rail 在 jsdom 的
compact 断点下折叠,label 只在 `title` 上),然后:

1. 冷启动后有基线写盘,但 restore 没往里加(`slice(writes)` 后为空)
2. 点"工作项" → `activeTab` 到 workitems 页
3. **磁盘上出现了 `workitems`**,且是**新**的写盘(`values.length` 增长)
4. `relaunch()` 之后仍读回 `workitems` —— 记忆跨重启存活

### 我还改了一条我上一单写错的断言方式

`writtenTabsFor` 报的是**每次写盘时该项目记忆的快照**。`beforeEach` 的
`setState` 本身也触发 persist,所以"冷启动没写盘"不能断言成"数组为空"。

改成**基线相对**:先记录 `appStoreWrites.values.length`,再断言
`slice(那个数)` 之后没有新增。这是同一个事实,但断言方式不再依赖
"测试自己的写入不算写入"这个假前提。

## 5. done_definition 逐条对

| # | 要求 | 实测 |
|---|---|---|
| 1 | 两条新契约测试在未变异代码上绿 | ✅ `12 passed (12)` |
| 2 | 8 条既有测试保持绿(含我重写的 4 条) | ✅ 同一次运行 |
| 3 | 写盘 effect **已移除**,不是收窄 | ✅ 整块删除,无 effect 驱动的 tab 写盘 |
| 4 | `chooseTab` 存在,每处操作员驱动都走它;restore 不走 | ✅ 7 处改走,restore 保留裸 `setActiveTab`(:489) |
| 5 | 一条测试证明 restore 不持久化 | ✅ `tabsWrittenSince === []` + 污染变异 4c 被 5 条抓住 |
| 6 | 全套绿 / typecheck 净 / eslint 不劣于 baseline | ✅ 372/31,tsc 净,135 errors,prettier 净,build 2.17s |
| 7 | 派单点名要的变异(`chooseTab` 不持久化) | ✅ **1 红**,新测试抓住 |

## 6. 我保留了什么(裁定明确要求)

**冷启动门(`skipColdStart`)原样保留。** 裁定说它是上一半、比我原处方的
`loadedUIProjectId === null` 更好。我没动它。

## 7. 仍未闭环

- **没开验收卡** —— `disp-E06O6IAaWefF-BOW-CNJd` 仍无 verdict 点名它。
- **真机验证仍未做**(裁定 `also_still_owed`)。
- **两个远端都不通** —— gitlab SSH 22 超时,origin 无权限。提交在本地。
