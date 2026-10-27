# 修两个订阅没有 `.catch()`(`disp-cLore9rXage5HKVn6Hsbd`)— 选「说出来」,没选「吞掉」

| Field | Value |
|---|---|
| dispatch | `disp-cLore9rXage5HKVn6Hsbd` |
| packet | `AEGIS-2026-10-27-mira-fix-unhandled-rejection-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-27 |
| 结论 | 已修。两个订阅都有失败处理,**方式不是照抄上面那个 `{}`**。界面上多一条黄条,写清「状态停在启动那一刻、不会再变」和看得见的后果。8 条新测试,5 个变异全杀死 |

## 一、派单问的三个问题,我的答案

### Q1:订阅失败时,supervisor 分离状态应该显示成什么?

**显示成「不知道」,而不是 `false`。**

这是整个修法的核心。`supervisorDetached` 是个 boolean,`false` 的含义是
「面板在主窗口里」—— 那是个**肯定句**。而订阅断掉后真实情况是「我不知道了」。
如果让状态保持 `false`,界面就在用自信的语气说一件它已经不知道的事。

这个区分不是洁癖,它有看得见的后果:`supervisorDetached === false` 同时决定
**⌘K 的行为**和**内嵌面板是否渲染**。状态冻结在 `false` 时:

- 操作者把面板拖成独立窗口 → 界面还以为它在主窗口 → ⌘K 在主窗口里**再开一份**,
  两个窗口同时有一份面板;
- 操作者把面板合回主窗口 → 界面还以为它是分离的 → ⌘K 只去聚焦那个窗口,
  **面板不回来**。

所以我加的**不是**「把状态改成某个值」,而是**一条独立的状态**:`supervisorSyncLost`。
启动查询照旧回答状态(它有自己的 `.catch`,本单没动),新状态只记录
「这个答案已经停止维护了」。两者分开,因为一个是答案、一个是答案的可信度。

### Q2:cleanup 还要成立吗?

**成立,而且比原来多管一件事。**

原来的 cleanup 有两个漏洞,都补了:

1. `if (offDetached) offDetached()` 的守卫**保留** —— 订阅失败时 `offDetached`
   还是 `null`,不守卫就会 `TypeError`。这原来是隐式正确的,现在有测试钉住。
2. **加了 `cancelled` 标志**。原来没有:如果 `listen()` 还在 pending 时组件就卸载了,
   resolve 之后会把 unlisten 函数存进一个再也不会有人读的闭包 —— **监听器泄漏**,
   页面存活期间一直挂着。加了标志后,卸载后才落地的订阅会被当场退订。

### Q3:Tauri 不在 vs 事件总线坏了,要区别对待吗?

**要,而且代码结构天然做到了 —— 但这一点必须说清,不然会误判。**

effect 第一行是 `if (!isTauri()) return;`。所以能走到我的 `.catch` 里的,
**只有 Tauri 在、但事件总线没收下这个订阅**这一种情况。非 Tauri 环境
(浏览器 dev)根本不会尝试订阅,那不是错误,不该报。

所以我的处理只有一条消息,没有 if/else 分支 —— **不是漏了第二种情况,
是第二种情况到不了这里**。这一点写在代码注释里,否则下一个人会以为要补分支。

## 二、改了什么

### `ui/src/components/SupervisorSyncLost.tsx`(新文件)

黄条(不是红条:数据读失败是红的,这个没那么严重 —— 主数据照常在读),
`role="status"`,内容:

- 标题:监督面板的实时状态收不到了 —— 下面显示的是启动那一刻的答案
- 解释通道断了,所以状态**不会再变**
- **看得见的后果**:拖成独立窗口后这里可能还不知道(⌘K 会再开一份);
  合回主窗口后也可能还以为分离(⌘K 不把面板带回来)
- 后端原样返回的错误原文(实测,不是转述)
- 下一步:重启应用;重启前把错误原文给 Aegis

**没有重试按钮。** 数据库读失败那个有,因为连接抖动重读就好;
事件订阅失败不是抖动,对着坏掉的总线反复订阅只会产生噪音和不存在的状态。

### `ui/src/App.tsx`

```diff
+ const [supervisorSyncLost, setSupervisorSyncLost] = useState<string | null>(null);
...
+ const lostChannels = new Map<string, string>();
+ const reportSubscriptionFailure = (eventName: string) => (err: unknown) => {
+   if (cancelled) return;
+   lostChannels.set(eventName, err instanceof Error ? err.message : String(err));
+   const lines = [...lostChannels].map(([n, d]) => `订阅 ${n} 失败:${d}`);
+   if (lostChannels.size > 1) lines.push('(两条通道都断了)');
+   setSupervisorSyncLost(lines.join('; '));
+ };
  onSupervisorDetached(...)
    .then((fn) => { if (cancelled) { fn(); return; } offDetached = fn; })
+   .catch(reportSubscriptionFailure('supervisor:detached'));
  onSupervisorReembedded(...)
    .then((fn) => { if (cancelled) { fn(); return; } offReembed = fn; })
+   .catch(reportSubscriptionFailure('supervisor:reembedded'));
```

**strip 挂在 `showSupervisorIM && !supervisorDetached` 外面** —— 这是有意的:
状态冻结成 `false` 正是「以为面板在内嵌着」的那种情况,把警告藏在那个判断后面,
会让它在本最需要的时候看不见。

## 三、为什么不是「吞掉」

派单问的就是这个。三种处理的区别:

| 处理 | 结果 |
|---|---|
| `.catch(() => {})`(照抄上面那个) | unhandled rejection 消失,**问题也消失** —— 状态是假的,界面用自信的语气说一件不知道的事 |
| `console.error` | 控制台有记录,**界面照旧撒谎**。操作者不看 console |
| **说出来(选的这个)** | 界面明确说「这个状态停在启动那一刻,不会再变」+ 看得见的后果 + 错误原文 + 下一步 |

判断依据就一条:**这个状态是不是 load-bearing 的**。`supervisorDetached` 决定
⌘K 开一个还是两个面板、决定内嵌面板渲不渲染 —— 是。那它失效就不能静默。

## 四、一个我在写测试时发现并修掉的真缺陷

第一版实现是 `setSupervisorSyncLost(\`订阅 ${eventName} 失败:${detail}\`)` ——
**后一个失败覆盖前一个**。两个订阅是并行 reject 的,测试直接抓到了:

```
expected '...supervisor:reembedded 失败:x...' to contain 'event bus refused supervisor:detached'
```

也就是说:**只有一条通道断了,和两条都断了,界面上看起来完全一样。**
而这个区别是要紧的 —— 一条通道抖动 vs 事件总线根本不收订阅,是两回事。
改成 `Map` 各自记自己的原因,两条都断时追加一句「(两条通道都断了)」。
**这条缺陷是我派单里没预见到的,是测试逼出来的。**

## 五、怎么验的

### 测试 `src/App.supervisorSync.test.tsx`,8 条

驱动**真实 App**、真实 effect、真实 cleanup,mock 掉 `./lib/api` 直接注入
reject 的订阅(最窄的接缝:被测的是 App 的反应,不是 Tauri 的传输):

| 断言 | 覆盖 |
|---|---|
| detached 被拒 → 有 strip,含后端错误原文、含「不会再变」、含 ⌘K 后果;启动查询仍跑过 | Q1 |
| reembedded 被拒 → 也有 strip | 两个调用点都要覆盖 |
| 两个都被拒 → 两个通道都出现在消息里 | 上面那个真缺陷 |
| 两个都被拒 → `unhandledrejection` 事件数 0 | 原始缺陷的头号症状 |
| 订阅成功 → **没有 strip** | 不能变成噪音 |
| 卸载 → 两个 unlisten 都被调 | cleanup 仍成立 |
| 卸载后才落地的订阅 → 被退订而不是泄漏 | `cancelled` 守卫 |

### 变异验证(5 个,全杀死)

| # | 变异 | 结果 |
|---|---|---|
| M1 | **一律 `.catch(() => {})`**(派单明令禁止的修法) | RED 4 |
| M2 | 只给 detached 加 catch,reembedded 漏掉(第一版修复最可能的漏法) | RED 2 |
| M3 | 去掉 `cancelled` 守卫(卸载窗口泄漏) | RED 1 |
| M4 | 改成 `console.error` 而不上界面(静默失败变体) | RED 4 |
| M5 | strip 无条件显示(噪音化) | RED 2 |

M1 和 M4 是这一单的重点:它们证明测试咬住的不是「有没有 `.catch`」,
而是「失败了界面上说不说」。

### 全量门(与基线对比)

```
改动后: Test Files 24 passed (24)   Tests 288 passed (288)   Errors 0
基线  : Test Files 23 passed (23)   Tests 280 passed (280)   Errors 0
```

`pnpm typecheck` 干净、`pnpm build` 通过。
eslint 全仓 **143 problems(133 error + 10 warning),与基线逐字相同** ——
`App.tsx` 那 15 个 `any` error 改动前后一模一样(我单独对比过),我的两个新文件 0 error。
prettier 三个文件全干净。

## 六、我踩的两个自己的坑(如实记)

1. **`vi.mock` 路径写错,而且静默失效**。我写了 `vi.mock('../lib/api')`,
   而测试文件在 `src/` 下,`'../lib/api'` 解析到 `ui/lib/api`(不存在)——
   mock **一条都没应用**,测试跑的是真模块。表现出来的现象很有欺骗性:
   strip 照样出现(因为真 `listen()` 也失败了),但 `listenCalls` 是空的,
   我一度以为 React 渲染中断了。是用一个探针打印 `factory runs: 0` 才定位的。
   正确路径是 `'./lib/api'`。
2. **mock 的 unlisten 是个不记账的空函数**。延迟落地的订阅被退订了,
   但我的 `fn` 是 `() => {}`,不往 `unlistenCalls` 里记 ——
   于是「退订了」这个断言看起来像失败。第一版我是靠改断言绕过去的,
   那不对;改成 resolve 一个**会记账的** unlisten 才测到了真东西。

## 七、明确不做的,我没碰

- **没有一律 `.catch(() => {})`**(派单明令禁止)
- **没有顺手改 `supervisorWindowStatus()` 那个 `.catch(() => {})`** ——
  派单说不许顺手改别的。但我要指出:它那个吞掉**同样是静默失败**
  (状态查询失败 → `supervisorDetached` 保持初值 `false` → 界面照样说
  「面板在主窗口里」)。**要不要一起修请派单,我没动。**
- 没有改别的订阅/effect,没有改测试的注入方式来让测试过
- 没有合并、没有自己开验收卡

## 八、需要上报

1. **`supervisorWindowStatus()` 的 `.catch(() => {})` 是同一个病的另一处**:
   它失败时 `supervisorDetached` 停在初值 `false`,界面照样自信地显示
   「面板在主窗口里」,而真相是「没查到」。我按派单要求没动它,
   但这条和本单修的是同一类问题,建议单独派一单。
2. 我的新 strip 只在 `supervisorSyncLost !== null` 时出现,而它是
   `useState` 的**本地**状态 —— 刷新页面就没了。这是有意的(重启正是
   strip 自己给的下一步),但意味着它不是一个持久告警。

## 复现命令

```bash
cd ui
pnpm vitest run src/App.supervisorSync.test.tsx    # 8 passed
pnpm test                                          # 288 passed, 0 errors(基线 280 / 0)
pnpm typecheck                                     # 干净
pnpm build                                         # 通过

# 证明「吞掉」会红:把两处 .catch(reportSubscriptionFailure(...)) 改成 .catch(() => {})
#   → 4 failed
# 证明只修一个会红:只去掉 reembedded 那个 catch
#   → 2 failed
# 证明 console.error 不够:把 setSupervisorSyncLost(...) 换成 console.error(...)
#   → 4 failed

# 与基线对比 lint(证明没增加问题):
git stash -u && pnpm exec eslint src 2>&1 | grep '✖'   # 143 problems
git stash pop
```

## 操作者怎么自己验

1. 正常启动 → **不该**看到黄条(订阅成功时它是隐形的)
2. 黄条出现的条件需要事件总线故障,日常点不出来;可信的办法是看代码路径 ——
   或者等 Aegis 决定要不要连 `supervisorWindowStatus()` 那个一起修
3. ⌘K 开面板、↗ Detach 拖成独立窗口、合回来 —— 这三步在黄条**不出现**时
   必须照旧工作(我的测试钉住了卸载路径,但真机拖窗口我验不了,
   **屏幕在我这边是锁的**)
