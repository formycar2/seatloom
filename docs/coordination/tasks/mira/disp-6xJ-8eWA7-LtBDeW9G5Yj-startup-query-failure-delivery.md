# 修 Flux 的 F1:启动查询被拒还是被吞掉(`disp-6xJ-8eWA7-LtBDeW9G5Yj`)

| Field | Value |
|---|---|
| dispatch | `disp-6xJ-8eWA7-LtBDeW9G5Yj` |
| packet | `AEGIS-2026-10-27-mira-fix-flux-finding-f1-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-27 |
| 结论 | 已修。启动查询失败不再有静默的 `false`,改用**另一条**黄条形状(红 + 重试按钮),和订阅那条(黄 + 无重试)分开。4 条新测试,4 个变异全杀死 |

## 一、先核 Flux 的 F1(不凭转述)

读了 `ev-taJ9qsGBaNLKuGQQ6Ahtm`(他对上一个单的 verdict=pass)。F1 的原文和
派单转述一致,而且他是**实测的**,不是推断:

> `shownWhenQueryFails=false, shownWhenSubFails=true`

方法:注入一个 reject 的 `supervisorWindowStatus()`,断言它的理由到不了界面;
对订阅做同样的事,断言能到。**他说得对,我认。** 而且他在 verdict 里确认了我
自己上报的这条(`mira_flagged_it_herself`),并指出我的自报是准确的 ——
一个自己上报的缺口通常是假的,这次不是。

## 二、派单三个问题的答案

### Q1:启动查询失败时该显示成什么?

**「不知道」,而且比订阅那条更彻底。**

订阅失败时,状态**曾经被回答对过**,只是停止维护;启动查询失败时,
**状态从来没被回答过**。`supervisorDetached` 的 `useState` 初值 `false`
在这种情况下不是"旧的正确答案",是**一个从未被验证过的猜测**。

所以修法不是把 `false` 换个值,而是让界面说出"没问到"。

### Q2:用同一条黄条还是另一条?

**另一条。** 三条理由,按重要性排:

1. **可采取的行动不同。** 启动查询是**一次读**,偶发的连接抖动重读就好 ——
   和数据库读失败给重试按钮是同一个理由。订阅是**持续通道**,对着坏掉的
   事件总线反复订阅只产生噪音和不存在的状态。所以一条该有按钮,一条不该。
   合成一条就得在按钮上做选择:要么给不能用的按钮,要么把能用的收起来。
2. **严重程度不同。** 没答过的那条更糟(Flux 的原话:"strictly worse because
   it is the FIRST read")。用同一条同色的条,会把两种严重程度显示成一样。
3. **文案必须不同。** "答案停止维护"这句话在没答过的场景下是**假的** ——
   没有什么在维护。我的测试里有一条专门断言它**不**出现
   (`not.toContain('启动时查到的那一次')`)。

### Q3:文案要能区分吗?

**要,而且区分点就是 Q2 第 1 条那个可采取的行动。** 对操作者来说光是
"我不知道面板在哪"不够 —— 他需要知道**现在有没有事可做**。所以:

| 形状 | 颜色 | 标题 | 按钮 |
|---|---|---|---|
| `never-answered` | 红 | 读不到监督面板现在在哪 —— 这个界面不知道,也不猜 | **再问一次** + 重启 |
| `stale` | 黄 | 监督面板的实时状态收不到了 —— 下面显示的是启动那一刻的答案 | 重启 |

颜色差是最快的"有没有东西可以依靠"的读数:红=第一次读就坏了,没有答案;
黄=曾经有答案,只是不更新了。

## 三、改了什么

### `ui/src/components/SupervisorSyncLost.tsx`

加 `kind` prop(`'never-answered' | 'stale'`)和可选 `onRetry`。
文案按 kind 分两支,颜色按 kind 分两支,`data-kind` 属性供测试断言。
**只有 `never-answered` 渲染重试按钮。**

### `ui/src/App.tsx`

```diff
-  const [supervisorSyncLost, setSupervisorSyncLost] = useState<string | null>(null);
+  const [supervisorSyncLost, setSupervisorSyncLost] = useState<{
+    kind: 'never-answered' | 'stale';
+    reason: string;
+  } | null>(null);

+  // 提到组件作用域:重试按钮要能调它,而按钮在 effect 外面
+  const askSupervisorWindowStatus = useCallback(() => {
+    api.supervisorWindowStatus()
+      .then((s) => { setSupervisorDetached(s.detached); setSupervisorSyncLost(null); })
+      .catch((err) => { setSupervisorSyncLost({ kind: 'never-answered', reason: ... }); });
+  }, []);
+
   useEffect(() => {
     ...
-    api.supervisorWindowStatus().then(...).catch(() => {});
+    askSupervisorWindowStatus();
     ...
+    // 订阅失败改成 kind:'stale'
   }, [askSupervisorWindowStatus]);
```

三个细节:

1. **查询提成 `useCallback` 而不是留在 effect 里** —— 重试按钮在 JSX 上,
   调不了 effect 内部的函数。提出来还能让"成功就清条"对重试也成立。
2. **成功时 `setSupervisorSyncLost(null)`** —— 重试按钮只有"成功了条真的消失"
   才算诚实,否则它是个带点击处理的谎话。有测试钉这条(M2)。
3. **effect 依赖数组写 `[askSupervisorWindowStatus]` 而不是 `[]`** ——
   我第一版写了 `[]`,eslint 报了一个新 warning(`missing dependency`)。
   **没有用 `eslint-disable` 掩盖**:`useCallback` 的空依赖列表让它引用恒定,
   所以写进去效果不变(还是只跑一次),但规则是对的、警告是真的。
   写进依赖数组后 warning 消失,全仓 lint 回到与基线一致的 143。

## 四、怎么验的

### 测试 `src/App.supervisorSync.test.tsx`,12 条(原 8 + 新 4)

新四条:

| 断言 | 覆盖 |
|---|---|
| 启动查询被拒 → `data-kind="never-answered"`、含后端错误原文、含「没问到」「不知道」、含 ⌘K 后果、**不含**「启动时查到的那一次」 | Q1/Q2/Q3 |
| 启动查询被拒 → `unhandledrejection` 事件数 0 | 原始症状 |
| 点「再问一次」→ 第二次查询、成功后条消失 | 重试按钮是诚实的 |
| 订阅失败那条 → `data-kind="stale"` 且**没有**重试按钮 | 另一侧的区别也被钉住 |

最后一条是从**反方向**钉这个设计的:只测"never-answered 有按钮"不够,
还要测"stale 没有按钮",否则将来有人给 stale 也加一个,没有任何测试会红。

### 变异验证(4 个,全杀死)

| # | 变异 | 结果 |
|---|---|---|
| M1 | **贴回 `.catch(() => {})`**(F1 本身) | RED 3 |
| M2 | 成功后不清 strip(重试按钮变谎话) | RED 1 |
| M3 | 启动查询失败用 `stale` 形状(两种失败混为一谈) | RED 2 |
| M4 | 给 stale 也加重试按钮(不能用的按钮) | RED 1 |

M1 是派单明确要求的证明;M3/M4 证明两种形状**不是装饰**,是有断言保护的。

## 五、我自己的两个操作失误(如实记,都发生了)

1. **`cp /tmp/app4.bak src/App.tsx` 把我的 lint 修复覆盖掉了。**
   那个备份是加 lint 修复**之前**存的。跑完 M1 后我照惯例还原,结果回到了
   有 warning 的版本 —— 而我以为还原完了。是靠 `grep` 依赖数组才发现,
   重新打了一遍。**教训:备份要在"所有修复都打完"之后取,或者还原后必须
   重新跑一遍门,不能只跑测试。** 我这次只跑了测试就以为完了,
   lint 是后来单独查的。
2. **`git checkout -- src/components/SupervisorSyncLost.tsx` 把我新写的组件
   整个回退了**(89 行旧版)。当时只想撤销 M4 那一行变异,但那个文件整个是
   未提交的新内容,checkout 直接回到 HEAD。只能整个重写一遍。
   **教训:对未提交的新文件做变异,还原要用备份而不是 git checkout。**

两次都是"以为还原了,其实没有",而且都不会被测试发现(M1 之后测试确实全绿)。
最后是靠逐项跑 lint/prettier/typecheck/build 才确认真的干净。

## 六、明确不做的,我没碰

- **没有又把 `.catch(() => {})` 写上**(派单明令禁止,那正是 F1 本身)
- **没有改 ⌘K 或内嵌面板的渲染逻辑**(那是读侧,不是这个问题)
- 没有顺手改别的 `.catch` —— 全仓还有别的,但派单说不许
- 没有合并、没有自己开验收卡

## 七、需要上报

1. **Flux 说这个修复还没到操作者眼前**(他 verdict 的 `deployment` 字段):
   内容在 track 的 `879ff89`,但跑着的 `/Applications/SeatLoom.app` 是 00:03
   的二进制,strip 文字不在它的 blob 里。**这是 Aegis 在 `eb47ec8` 记过的
   第三次复发**:进程在跑不等于跑的是新代码。本单不涉及(我不重建二进制),
   但操作者现在仍然看不到黄条 —— 要看到需要重建+重启。
2. 同类模式还有:全仓有其他 `.catch(() => {})`,是否都要按"状态是否
   load-bearing"过一遍,请派单定,我没顺手改。

## 复现命令

```bash
cd ui
pnpm vitest run src/App.supervisorSync.test.tsx   # 12 passed
pnpm test                                         # 292 passed(基线 288)
pnpm typecheck                                    # 干净
pnpm exec eslint src 2>&1 | grep '✖'              # 143 problems,与基线一致
pnpm build                                        # 通过

# 证明 F1 修法会红:把 askSupervisorWindowStatus 里那个 .catch 贴回 .catch(() => {})
#   → 3 failed(其中一条是"没有 unhandled rejection"那条,即原始症状)
# 证明两种形状不是装饰:把 kind:'never-answered' 改成 kind:'stale'
#   → 2 failed
# 证明重试诚实:删掉 .then 里的 setSupervisorSyncLost(null)
#   → 1 failed

# 与基线对比:
git stash -u && pnpm test 2>&1 | grep -E '^ +(Test Files|Tests|Errors) '
git stash -u && pnpm exec eslint src 2>&1 | grep '✖'
git stash pop
```

## 操作者怎么自己验

1. 正常启动 → **不该**看到任何条
2. 两种失败日常点不出来(需要后端拒绝);可信的办法是看代码路径,
   或者等重建后由 Aegis 决定怎么造这个场景
3. 黄条出现时:红的那条有「再问一次」按钮,黄的没有 —— 这是有意的,
   因为只有一次读失败是重读能修好的
