# 补覆盖缺口 F1(`disp-B4Y5oAohMQwtXm5Vva1fw`)— M6 那条路径现在有测试了

| Field | Value |
|---|---|
| dispatch | `disp-B4Y5oAohMQwtXm5Vva1fw` |
| packet | `AEGIS-2026-10-27-mira-cover-flux-finding-f1-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-27 |
| 结论 | **F1 已补**。M6 指 `InitDialog` 的 `onInitialize`,那条路径原来零测试。新测试 2 条,贴回旧代码 **2 条全 FAIL**,改回来 **PASS**。**源码零改动**(实现是对的,只加测试)。另外顺手把套件既有的 14 个 unhandled error 降到了 0 |

## 一、M6 是哪条路径(去账本读了原文,不是转述)

读了 `ev-mAztaqFFzPSAQ-Fm3asBD`(Flux 重验,verdict=pass)。**先纠一处转述偏差**:
派单说 M6 在 `mutation_table` 字段里 —— **不在**。`mutation_table` 只有 5 条,
全是别的内容(restore gate / write gate / launch default / 等价变异)。M6 在
`finding_F1` 里,原文:

> **M6:恢复那条确切的旧行 → 7 个测试全过。**
> Reverting InitDialog's `setActiveProject('p-1')` -> `hydrateFromBackend()`
> left all 7 tests green.

所以「那条确切的旧行」= `onInitialize={() => { setActiveProject('p-1'); setIsInitOpen(false); }}`
(我用 `git show 0e71598~1:ui/src/App.tsx` 对过原文,逐字一致)。

**为什么 7 个测试都碰不到**:`InitDialog` 的打开条件是
`restoredPosition && !activeProjectId`,而这只在**后端一个项目都没有**时成立。
既有 7 个测试全部用有项目的 `CANNED` 挂载,对话框从未出现过。
提交信息声称修了三处同根因的缺陷,前两处有测试,第三处(`InitDialog`)没有。

## 二、加了什么

`src/App.restartRestore.test.tsx` 追加一个 describe(2 条),驱动**真实对话框的
真实按钮、真实点击**,用本文件已有的埋点(落盘写入日志 + 调用顺序日志)取证:

1. **后端始终没有项目**:点按钮 → 不造项目(`activeProjectId` 仍是 `null`)、
   不落盘(`readStoredAppStore()` 是 `null`)、**确实又读了一次后端**
   (`cmd_list_projects` 出现 2 次以上)、对话框关闭。
2. **启动后后端出现了项目**(操作者或别的席位建了一个):点按钮 →
   落在真实项目 `seatloom` 上,且该 id 是后端返回过的,且又读了后端。

第 2 条是防**反方向退化**的:一个只关对话框不重读的实现,会把操作者永久困在
「没有项目」上 —— 那是这半边的失效形态。

### 一条关键的额外断言(我第一版漏了,被变异抓出来)

第二版我加了一条「任何一次 scoped 读都不带种子 id」:

```ts
expect(readProjectIds()).not.toContain(OLD_LAUNCH_DEFAULT);
```

起因是变异 **M-D**:`hydrateFromBackend()` 之后**再** `setActiveProject('p-1')`。
这条变异第一版测试**全绿** —— 因为 `hydrateFromBackend` 会 `await` 后端列表,
之后 `nextActive` 逻辑发现 `'p-1'` 不在返回的项目里,自动修正到 `projects[0]`,
**最终值看起来是对的**。

插桩看到的实际痕迹证明它确实短暂落在 `p-1` 上:

```
DEBUG order: [..., "cmd_list_seats {\"projectId\":\"p-1\"}", ...,
              "cmd_list_seats {\"projectId\":\"seatloom\"}", ...]
```

这正是 Flux 10-21 抓过的同型问题:**最终值正确,窗口里先写错**。
对 `InitDialog` 来说窗口期的代价是:那次 scoped 读会去查一个不存在的项目。
所以我改成断言**痕迹**(后端被要求读过哪些 projectId),不断言最终值。

## 三、证明它会红(派单要求的第 3 条)

### 贴回 M6 旧代码 → FAIL

```
$ # onInitialize={() => { setActiveProject('p-1'); setIsInitOpen(false); }}
  × with no backend projects at all, clicking invents nothing and records nothing
  × when projects have appeared since launch, clicking lands on a real one
     Tests  2 failed | 7 passed (9)
```

失败断言逐字:

```
AssertionError: expected 'p-1' to be null
AssertionError: expected 'p-1' to be 'seatloom' // Object.is equality
```

注意 `7 passed` —— 既有的 7 条仍然全绿,**复现了 Flux 的观察**,证明我补的
正是那条无人看守的路径,而不是把别的测试改红了。

### 改回来 → PASS

```
$ pnpm vitest run src/App.restartRestore.test.tsx
     Tests  9 passed (9)
```

### 完整变异表(7 个,全被杀死)

| # | 变异 | 结果 |
|---|---|---|
| M6 | 贴回旧代码 `setActiveProject('p-1')` | **RED 2** |
| M-A | 只关对话框,不 re-read | RED 1(场景 2) |
| M-B | `setActiveProjectId(null)`,不读后端 | RED 1(场景 2) |
| M-C | `setActiveProject('p-2')`(另一个种子 id) | RED 2 |
| M-D | re-read 之后再 `setActiveProject('p-1')` | 第一版**全绿(漏!)** → 加「痕迹」断言后 RED 1 |
| V1 | restore effect 丢 `backendHydrated` 门(既有守卫复跑) | RED 6 —— 没被我改弱 |
| V2 | `useDataStore` 启动默认值回 `'p-1'`(既有守卫复跑) | RED 1 —— 没被我改弱 |

V1/V2 是我特意复跑的:新增断言和改动 mock 有可能把既有 7 条改弱,必须确认没有。

## 四、顺手拆掉一个既有的坑(要说清楚,因为它超出了「加测试」)

第一版跑完,套件 `Errors` 从 14 涨到 **18**。查下来不是我的测试写错,是
**同一个文件里两处 mock 不同源**:

- `vi.mock('@tauri-apps/api/core')` 用 CANNED 表回答;
- `window.__TAURI_INTERNALS__.invoke` 对**所有**命令一律抛
  `unmocked Tauri invoke`。

而 `App.tsx:160/166` 的 `onSupervisorDetached/onSupervisorReembedded` 订阅
**没有 `.catch()`**,`isTauri()` 走的是 internals 那条路 —— 于是每次挂载
产生 2 个 unhandled rejection。既有的 14 个 = 7 个测试 × 2,我的 2 个新测试各挂载
一次,所以 +4。

**这不是「反正本来就有」就能放过的**:一个只会增长的 error 数会让下一个人
分不清是不是自己弄坏的。我把两处 mock 统一到同一个 `answerCommand`
(文件注释里本来就写着 "one source of truth for the backend",但实现没做到),
并补上 event plugin 的 internals。结果:

```
改动后: Test Files 23 passed (23)   Tests 280 passed (280)   Errors 0
基线  : Test Files 23 passed (23)   Tests 278 passed (278)   Errors 14 errors
```

**测试多了 2 条,error 少了 14 个。** 这个改动只碰 mock,没碰任何产品代码,
也没改任何既有断言 —— V1/V2 复跑证明既有守卫照旧咬得住。

## 五、明确不做的,我没碰

- **没有改任何实现代码**(`git status` 只有一个测试文件)。派单明令
  「不许为了让 M6 变红而改实现 —— 实现是对的,Flux 验过了」。
- 没有加永远绿的测试:每条都对应一个变异,且 M-D 证明我会先漏再补。
- 没有改判据,没有合并,没有自己开验收卡。
- 没有为了让 error 数好看而加 `.catch()` 到产品代码 —— 那是另一个问题,
  要改也该单独派单(见第六节)。

## 六、需要上报的

1. **`App.tsx:160/166` 两个订阅没有 `.catch()`**。我是在测试侧补的 mock,
  产品代码里的承诺仍然悬空:如果 Tauri 的 `listen()` 真的失败(不是测试环境),
  那是个 unhandled rejection,而不是界面上一句话。
   同文件 `api.supervisorWindowStatus()` 有 `.catch(() => {})`(154 行),
   这两个漏了。**要不要修请派单,我没动。**
2. **「最终值断言抓不到窗口期错误」这个模式又出现了一次**(M-D)。
   这次是 `hydrateFromBackend` 的自动修正掩盖了短暂的错误 id。
   我在同一份交付说明里第二次记同型问题(前一次是 10-22 的 `loadedUIProjectId` 门)。
   已记为记忆。

## 复现命令

```bash
cd ui
pnpm vitest run src/App.restartRestore.test.tsx    # 9 passed
pnpm test                                          # 280 passed, 0 errors(基线 278 / 14 errors)
pnpm typecheck                                     # 干净
pnpm build                                         # 通过

# 证明 M6 会红:把 ui/src/App.tsx 的 onInitialize 改回
#   setActiveProject('p-1');
# 再跑第一行 → 2 failed / 7 passed(既有 7 条仍绿,即 Flux 的观察)
# 改回 void hydrateFromBackend(); 再跑 → 9 passed

# 证明既有守卫没被改弱:
#   restore effect 去掉 `if (!backendHydrated) return;`  → RED 6
#   useDataStore.ts:1614 activeProjectId 改回 'p-1'      → RED 1

# 与基线对比 error 数:
git stash -u && pnpm test 2>&1 | grep -E '^ +(Test Files|Tests|Errors) '
git stash pop
```

## Flux verdict 里我要确认的两件事(都确认了)

| 事项 | 结果 |
|---|---|
| verdict=pass,点名 `disp-DFeH6Ut_aNfzIbuuR21V5` | ✓ `ev-mAztaqFFzPSAQ-Fm3asBD` |
| 决定性对照:0e71598~1 的 App.tsx + useDataStore 原样跑新套件 | 原文记 7 of 7 FAILED,失败断言 `expected seatloom to be ql_delivery` |
| M6 位置 | 不在 `mutation_table`(那只有 5 条),在 `finding_F1`。派单转述把位置说错了,我按原文走的 |
| 修复在跑着的二进制里,不需重建 | 原文说他解压 blob 看过,`activeProjectId:null`、`p-1` 不在。我没重跑这一步(超出本单,且他没说需要) |
