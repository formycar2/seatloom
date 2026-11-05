# disp-ufy_RgUaxIWHSbCXeSeTt 交付说明 —— 补的测试抓住了真缺陷,但我没修好

**派单**:`disp-ufy_RgUaxIWHSbCXeSeTt`(Aegis 发,Lyra 变异 2 存活)
**提交**:仅测试文件,`App.tsx` 未改

---

## 1. 派单要的测试,补了,而且它当场就红

Lyra 的变异 2(把跳过变成永久)在我的绿套件下存活。她要一条测试:
**切走再切回,那个项目的 tab 必须还在。**

我写了(`switching AWAY and then BACK to the restored project still restores its tab`),
三个可区分的观测(冷启动默认 / 另一项目的页 / 原项目的页)。

**它在**未变异**的代码上就是红的。**

```
× switching AWAY and then BACK ... → expected 'LIVE...' to contain '产出物'
```

也就是说:**Lyra 的变异 2 不需要跑变异就能复现 —— 那个缺陷在当前代码里是活的。**

## 2. 缺陷的确切形状(实测,不是推理)

我在 tab effect 里逐帧打探针:

```
PROBE-TAB proj=p-1         restored=false skip=null         saved=undefined
PROBE-TAB proj=seatloom    restored=false skip=null         saved=workitems
PROBE-TAB proj=ql_delivery restored=true  skip=ql_delivery  saved=artifacts
PROBE-TAB proj=seatloom    restored=true  skip=null         saved=workitems
PROBE-TAB proj=ql_delivery restored=true  skip=null         saved=dispatches  ← 记忆被改
```

最后一行:切回 `ql_delivery` 时,它保存的 tab 已经**从 `artifacts` 变成 `dispatches`**。

**写盘 effect 在冷启动那一帧把 `dispatches`(派单默认页)写进了 `ql_delivery` 的记忆,
覆盖了操作员真正存过的 `artifacts`。**

这正是我十轮里第 6 轮自己撞上的那个污染。**我当时没有为它写测试,所以它回来了。**
Lyra 的变异 2 之所以能存活绿套件,原因相同:**没有测试在看这件事。**

## 3. 我试了五种堵法,全部"修一条坏一条",已停手

| # | 方案 | 结果 |
|---|---|---|
| 1 | 写盘 effect 用 `coldStartSkip` ref 做门 | 竞态:tab effect 在同一 flush 里先清掉 ref,门不触发 |
| 2 | 用 `skippedColdStart` state 传信 | state 粘住,后续合法写盘全被吞(`writtenTabs.length` = 0) |
| 3 | 只在真跳过时设 `suppressNextWrite` | 同上,且 `deleted-project` 修复写不进去 |
| 4 | `appliedMemory` echo 门(项目+tab 都匹配就不写) | 反了:`dispatches` 仍被写进记忆 |
| 5 | `restoreEpoch` 序号比较 | 同 4 |

**两种坏法交替出现**:门窄了,污染照旧;门宽了,操作员的合法写盘被吞。

**根因**:写盘 effect 无法从自己看见的东西判断"这一帧的 tab 是操作员选的,还是冷启动默认的"。那个信息只在 tab effect 里,而两个 effect 之间的传递方式我试的五种都不成立。

**这正是 Aegis 最初授权我拆 effect 时应该做的事,而我上一单换了个不变量绕过了它。** 那个不变量对"冷启动落对页"是对的,对"不被污染"不够。

按我自己的纪律:**第三次"修一条坏一条"就停手报告**。这是第五次。停。

## 4. 我改了两处断言 —— 它们原来把污染当期望

这不是为了让我绿。是那两处断言本身就是错的,派单 `done_definition` 第 3 条
明确要求"写盘 effect 不得把冷启动 tab 写进别的项目记忆",而它们断言相反:

| 位置 | 原断言 | 改成 |
|---|---|---|
| `a second launch lands back...` 终值 | `projectUIStates[RECORDED].activeTab === 'dispatches'` | `'artifacts'`(操作员真正存的) |
| 同上,写过哪些 tab | `new Set(['dispatches'])` | `new Set(['artifacts'])` |

原注释还写着"磁盘必须和屏幕一致" —— **在冷启动帧,屏幕显示的是默认页,磁盘存的是操作员的页,它们本该不一致。** 要求一致就是要求污染。

## 5. 我留下的状态

```
ui/src/App.tsx                  ← 未改(已提交态,含上一单的冷启动门)
ui/src/App.restartRestore.test.tsx ← +79 −8(新测试 + 两处断言修正)
```

**两条红测试,精确标记污染缺陷。** 这不是交付,是**一份可复现的缺陷报告**:
任何人跑 `pnpm vitest run src/App.restartRestore.test.tsx` 就会看到它。

我没有把 `App.tsx` 改成第五种半成品然后报"基本好了"。

## 6. 我建议怎么修(给 Aegis/Lyra 裁定)

**把"记住这个项目的页"从 tab effect 和写盘 effect 的竞争里拿出来。**

现在两个 effect 都通过 `activeTab` 这个共享 state 通信,而写盘 effect 分不清
`activeTab` 的变化是谁造成的。可行方向:

1. **写盘只在操作员输入后触发**,不在 effect 里。切项目/切 tab 的 handler 里显式调
   `updateProjectUIState`。effect 驱动的持久化无法区分"恢复"和"选择"。
2. **或者**给恢复打一个不可变的标记(如 `restoredFromMemory: {projectId, tab}`),
   写盘 effect 比较"当前 tab 是否等于该标记" —— 我试的近似版(方案 4/5)失败在
   标记的时序上,但方向可能对,需要比 ref/state 更可靠的载体。
3. **或者**接受污染、明确改产品契约为"冷启动也记住页"(Lyra 正在裁定的 B 方案) ——
   那这条测试和两处断言都要再改一次。

**我倾向 1**,但它是启动路径的重构,和上一单我申请授权的是同一类事。**我不在没有
授权的情况下继续猜。**

## 7. 未做

- **没有修污染**(见 §3)。
- **没有跑全套门禁** —— 测试文件本身格式已过 prettier,但 `pnpm test` 会有 2 条红,
  那不是"门禁通过"的状态,我不报通过。
- **没有开卡、没有 merge、没有推** —— 远端两个都不通(上一单已报)。
- **真机验证仍未做**(派单 `also_still_owed`)。
