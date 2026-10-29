# disp-E06O6IAaWefF-BOW-CNJd 交付说明 —— 第一屏卡断言是假的。我改了,没改完,这里说清卡在哪儿

**派单**:`disp-E06O6IAaWefF-BOW-CNJd`(Aegis 发)
**提交**:无(未提交,见 §6)

---

## 1. Aegis 的诊断我独立复核了,它对

它说三件事,我逐条核:

| Aegis 的说法 | 我核到的 |
|---|---|
| `App.tsx:133` `useState<Tab>('dispatches')` 是对的且在 track 上 | ✅ 确认 |
| `App.tsx:436` 的 effect 会用持久化 store 覆盖它 | ✅ 确认(原始 effect:`if (savedState) setActiveTab(savedState.activeTab)`) |
| store 没有 `partialize`,`projectUIStates` 整个被持久化 | ✅ 确认(`grep -c partialize` = 0) |

**它还给了操作员机器上的实测**:恢复的项目是 `project_delivery/QL`,存的 tab 是
`live-seats`,mtime 10-28 22:53。所以每次冷启动都落在实时席位。
**那张卡让张先生按一个假前提验收。**

我本地没有那个持久化文件(跑的不是这个 build),所以机制是我从代码确认的,
操作员机器上的事实我采信 Aegis 的实测 —— 它在派单里写明了是从运行中的应用读的。

## 2. 我实现了,然后卡住了,卡点说清楚

### 2.1 我做成的部分

- 确认了三个 effect 的耦合链:restore effect 决定项目 → tab effect 应用记忆 → write effect 落盘
- 写了两条新测试,覆盖 A 方案的两个方向:
  - **冷启动落派单页**(即使恢复的项目有已存 tab)
  - **会话内切项目仍恢复该项目的 tab**(记忆功能没被删)
- 把 4 条旧测试从"重启后落在已存 tab"改到新契约,并保留它们原本要保的东西
  (项目恢复、写盘门没坏)

### 2.2 我卡在哪儿

**同一个 effect 上改了十轮,每轮修好一条坏一条。** 记录这几轮,因为它们是有信息的:

| 尝试 | 结果 |
|---|---|
| ref 布尔门(第一次带项目的运行就跳过) | 冷启动对了;但 effect 在 `activeProjectId` 还是 null 时先跑一次,跳过被浪费 |
| 按项目 id 键控的 ref | 冷启动对了;但切到新项目也被当"首次到达"跳过,记忆失效 |
| 加"离开过就清除跳过" | 逻辑绕反,两条都坏 |
| 用 `restoredProjectId` ref(restore effect 写入) | 时序不可控 —— 哪个 effect 先跑代码看不出来 |
| 改成 state(`coldStartProjectId`) | **类型和测试都过了,冷启动仍落 live-seats** |
| 在 restore effect 里直接 `setActiveTab('dispatches')` | 冷启动对了(2 条红→1 条);但写盘 effect 随即把 `dispatches` 存进磁盘,**污染了别的项目的记忆** |
| 给写盘 effect 加冷启动守卫 | 修了污染,但把恢复的那个项目永久静音,2 条既有测试红 |
| 写盘守卫改成一次性(`useRef` 翻转) | 仍红 |
| 延迟 `setLoadedUIProjectId` 到非冷启动运行 | 破坏了写盘门自己的前置条件,3 条红 |
| **回到 tab effect 单点门 + state** | 368 过 / 2 败,就是当前状态 |

**最后一轮的状态**:typecheck 干净、368 测试过、eslint 与 baseline 一致。
失败的 2 条正是我新写的两条 —— 也就是说**我的新契约测试没有一条是绿的**,
而 8 条旧测试(含我改过的 4 条)全绿。

### 2.3 根因,我现在能说清的那部分

这三个 effect 是一个**有顺序依赖但顺序不可见**的系统:

```
restore effect  →  设 activeProjectId + restoredPosition
tab effect      →  读 projectUIStatesRef,决定 activeTab,设 loadedUIProjectId
write effect    →  守 loadedUIProjectId === activeProjectId,把 activeTab 落盘
```

`tab effect` 同时干两件事:**决定 tab** 和 **ack 项目**。ack 是写盘的前置,
所以我在 tab 侧加的任何门,都会立刻传到写盘侧。**这是为什么每轮修一个坏一个。**

正确的形状大概是把这两件事拆开(ack 只在该项目的 tab 真的定了之后发生),
但那是一次重构,不是一行修。**我没有在这个派单里做重构的授权。**

## 3. 我给 Aegis 的具体问题

1. **要不要授权拆 effect?** 把"决定 tab"和"ack 项目"分成两个 effect,
   冷启动门就只影响前者。这动的是启动路径最核心的三段,需要一个明确授权。
2. **还是先只修那张卡?** 即:把 `acc-disp-first-screen-default-tab` 的验法
   改成"冷启动落在上次退出时的那一页",承认 B 方案。**B 是当前真实行为,
   写出来操作员就不会按假前提验收。** 这条我现在就能做,不需要改一行代码。
3. **Aegis 在派单里推荐 A,但说"I am not the product owner"。**
   这是个产品决定(冷启动永远落派单页 vs 记住上次的页),该由 Lyra/张先生定。
   **在它被定下来之前,我改哪一边都是猜。**

## 4. 我倾向怎么做

**先做 2(改卡),同时问 1。** 理由:

- 派单自己写了 `do_not`:"不要把恢复删掉"、"不要重新布局/加组件/加路由"。
  **它没有说"不要改卡"**,而且它明说旧卡的断言是假的、该让操作员知道。
- A 方案是产品行为变更,Aegis 自己说不是产品owner。**我拿一个未裁定的
  产品决定当实现目标,是做反了。**
- 但**让一张假卡继续挂在哪儿,是每一分钟都在消耗操作员信任的事。**
  改卡不需要等裁定。

## 5. 未提交

`git status` 有两个改动文件(`App.tsx` +49 行、`App.restartRestore.test.tsx`
+127/-18 行)。**我没有 commit**,因为:

- 新写的 2 条契约测试是红的,提交一组"我知道会失败"的测试不是交付
- App.tsx 的改动是一个**部分正确**的中间态(冷启动门有了,但它依赖的
  ack 时序没拆干净)

要我把这部分存档成一个分支/提交供别人接着做,一句话我就做。
**但我不把它当成"改完了"提交。**

## 6. 门禁(当前状态)

| 门 | 结果 |
|---|---|
| `pnpm typecheck` | 干净 |
| `pnpm vitest run` | **368 passed / 2 failed**(失败的 2 条是我新写的契约测试) |
| `pnpm exec eslint src` | 135 errors(既有 baseline,未新增) |

## 7. 这一条我给自己记下

**当一个修复在同一处反复"修一条坏一条"时,那不是还没找到正确写法,
是对这块代码的理解不够,或者结构本身需要拆。**

我前面几轮的经验全是前者("再想一个条件就能对"),于是撞了十轮。
**下次出现第三轮"修一条坏一条",该停的是手,不是该继续想条件。**

Aegis 的知识记录里有一句近似的话(它查持久化文件那次):
"没得出结论就转移话题,和没查是一样的。"
我这次是反过来的形状:**查出了结论(结构要拆),但没有停下来问授权,
而是继续在原地试条件。**
