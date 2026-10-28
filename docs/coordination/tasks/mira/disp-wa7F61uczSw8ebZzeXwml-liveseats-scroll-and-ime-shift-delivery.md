# `disp-wa7F61uczSw8ebZzeXwml` — LiveSeats 滚动 + 中文 IME shift+key

## 一句话

**滚动：真 bug，已修，已测。IME：不是 SeatLoom 前端的缺陷，证据链完整附在下面 —— 我没有改任何代码。**

---

## Issue 1 — LiveSeats 不能往上滚看历史(已修)

### 根因(逐条排除,不是猜的)

按顺序量掉四个显疑点:

| 怀疑对象 | 实测 | 结论 |
|---|---|---|
| scrollback 太小 | `SessionTerminal.tsx:109` 已是 `scrollback: 10000`;写入 200 行真的进了 buffer | **不是** |
| 有人强制回底 | 全文件无 `scrollToBottom` 调用 | **不是** |
| 新输出把人拽回底部 | 视口停在 174,写新输出后仍是 174 | **不是** |
| 视口本身卡住 | `scrollLines()` 调用正常移动视口 | **不是** |

**真因:wheel 监听器从来没被注册。**

xterm 把 `wheel` 绑到自己的 element 上是有条件的 —— 绑定代码是 `16 & protocolMask`,而 `protocolMask` 只在 pane 里的程序发出 DECSET 1000/1006/1015 之后才非零。实测:

- 无任何鼠标协议时,element 的 class 里**没有** `enable-mouse-events`
- 写入 `ESC[?1000h` 之后,该 class 出现

tmux 的 `mouse on` 是 tmux 自己消费的,**不会代我们发这些序列**;而且 UI 里这个 xterm 是和 pane 相互独立的显示面。所以 mask 一直是 0,`addEventListener('wheel', ...)` 从未执行,滚轮事件无处可去。

这也解释了为什么它"时好时坏":取决于 pane 里当前跑的程序有没有启用鼠标上报。

### 修法

在 `SessionTerminal.tsx` 里自己绑 wheel,调 xterm 公开的 `scrollLines()` —— 和它内部 handler 本来要做的调用是同一个。`preventDefault` 防止面板底下的页面跟着滚。

**刻意没做的事**:没有自己去启用鼠标上报协议。那会让 pane 里的程序开始收到它没要过的鼠标事件,改变用户会话的行为 —— 远超"让我看历史"这个诉求。

### 测试(7 条,全部变异验证过)

新文件 `ui/src/app-v2/panel/SessionTerminal.wheel.test.tsx`,驱动**真实 xterm**(真实 buffer、真实解析器、真实视口),因为缺陷的本质就是"哪些 DOM 事件能到 xterm",mock 只会断言我们的意图。

变异结果:

| 变异 | 结果 |
|---|---|
| M1 删掉 wheel listener(回到 bug 态) | **红**,杀 3 条 |
| M2 滚动方向搞反 | **红**,杀 3 条 |
| M3 去掉 `preventDefault` | **绿** —— 见下方诚实说明 |
| M4 不清理 listener | **绿** —— 见下方诚实说明 |

**M3 为什么绿(如实说)**:实测 xterm 自己的 wheel handler 也调 `preventDefault`,所以当前版本下我们这个调用是冗余的。测试保留了它,但测试的注释明确写了"这个断言在删掉我们的调用时依然会过 —— 实测过,不是假设",它钉住的是**结果**(页面不滚),不是**归属**。代码注释同样说明:留着是因为我们的 listener 直接绑在 element 上,不该依赖 xterm 内部那个 listener 恰好注册了 —— 那个依赖正是滚动坏掉的那个假设本身。

**M4 为什么绿(如实说)**:我原本的卸载测试断言"卸载后视口不动",但 xterm 被 dispose 后也一样不动,所以那条测试什么都没证明。我重写了它:改为断言 element 已从文档断开、且组件确实持有过一个 Terminal,并在注释里写明"这一条证明的是组件没有把 listener 留在比它活得久的 element 上,不是证明 listener 被摘掉了 —— jsdom 里没有能观察这一点的钩子"。**清理代码本身是正确的**(`removeEventListener` 在 cleanup 里),只是 jsdom 无法为它建一条有鉴别力的断言。

### jsdom 的边界(派单要求说明替代物及其差异)

- jsdom **不做布局**,`scrollHeight`/`clientHeight` 全为 0,所以**无法合成真实浏览器滚轮手势**
- 这里真实的是**事件路径**(我们的 listener → `scrollLines` → 视口真的动了)和**解析器**
- "页面是否也滚了"的像素级问题,留给真机确认
- 为了能跑,xterm 的 `matchMedia` 和 canvas 2d context 打了桩。**canvas 桩意味着这些测试不断言任何渲染相关内容,只断言状态** —— 已写在测试文件头部注释里

---

## Issue 2 — 中文 IME 下 shift+key 打不出符号(未改代码:不是前端缺陷)

### 先说一个我自己的假阴性,因为它差点让我报错结论

第一版测试派发 `keydown{key:',', shiftKey:true}` 后,`ptyWriteBytes` **一次都没被调用**。看着就像"字符被吞了"。

实际上是我测试没给 `keyCode`。xterm 的 `evaluateKeyboardEvent` 里,单字符分支的完整条件是:

```
e.key && !e.ctrlKey && !e.altKey && !e.metaKey && e.keyCode >= 48 && 1 === e.key.length
```

**`keyCode >= 48` 是硬门槛**,`,` 的 keyCode 是 188。不设就是 0,直接被筛掉。补上 `keyCode: 188` 重测:

```
shift + ,  →  ptyWriteBytes([[44]])     // 44 = ASCII ','
plain ,    →  ptyWriteBytes([[44]])
```

**字符确实到达 PTY。** 之前那次 0 是测试缺陷,不是产品缺陷。

### 逐层排查过,没有 SeatLoom 的拦截者

| 位置 | 只拦截什么 | 会影响 shift+,? |
|---|---|---|
| `useGlobalShortcuts.ts` | `?`+shift、F1 | 否 |
| `App.tsx:192` | Cmd+K | 否 |
| `SupervisorPanel.tsx:251` | Escape | 否 |
| `SupervisorCommandBar.tsx:80` | Escape/Enter(且 isOpen 门控) | 否 |
| `LiveSeatsView` / `SessionTerminal` / `TmuxMirrorWorkspace` | 无 keydown 拦截 | 否 |

真实 hook + 真实 window listener,派发 `shift+,`,四种 target 场景(help / terminal / cmdBar / body)下 **没有任何快捷键触发**,`defaultPrevented` 全程 `false`。

### 真因在 xterm 5.5.0 的 CompositionHelper

```js
keydown(e) {
  if (this._isComposing || this._isSendingComposition) {
    if (229 === e.keyCode) return false;
    if (16 === e.keyCode || 17 === e.keyCode || 18 === e.keyCode) return false;  // Shift/Ctrl/Alt 放过
    this._finalizeComposition(false);   // 其它键:强行终止合成
  }
  ...
}
```

中文 IME 处于合成态时按 `shift+,`,`keyCode = 188`,不在 16/17/18 白名单里 → 走 `_finalizeComposition(false)` → **合成被强行终止**,该字符丢失。

这和 2026-10-26 那条 `Shift+/` 是**不同的故障**:那次是我们自己的快捷键吃输入,靠 `isTypingContext()` 守卫解决;这次前端根本没人拦,是 xterm 与中文 IME 的冲突。

### 我查过上游,没有可用的升级解法

下载了 xterm 6.0.0 对比,`CompositionHelper.keydown` 的这段逻辑**一字不差**。所以"升级 xterm"不解决这个问题,而且大版本升级是行为面很大的变更,不属于本派单范围,我不该单方面做。

### 我没能测到的部分(派单明确要求说明)

**真实中文 IME 下的 `event.isComposing` / `event.target.tagName` / `event.key` 实测值,我没有拿到。** jsdom 无法驱动真实 IME 合成循环。上面的结论来自:xterm 源码的判定逻辑 + jsdom 里可复现的 `keyCode` 硬门槛 + 前端全层无拦截者的事实。**这是读码得出的因果,不是真机量出来的。**

要闭环,需要在真机上:开中文 IME、终端聚焦、按 `shift+,`,同时抓 `keydown` 的三个字段。这个我做不到。

---

## 需要上报 Aegis 的

1. **IME 这条应当转给控制台/终端选型的负责人**,不是继续压在 UI 实现层。可选方向(我未评估,也未实施):给 xterm 打一个 `compositionstart` 期放宽 keyCode 白名单的补丁,或换终端组件。**我不建议在无派单的情况下自己动 xterm 内部。**
2. **滚动的真机确认仍未做**。跑着的 `/Applications/SeatLoom.app` 二进制不含本修复(这条 Flux 已经提过五次)。需要重新构建+重启才能在真机上看到滚轮可用。
3. `dto-to-v1.ts:159` 的 `isPinned: true` 硬编码 —— 上一条派单已报,仍未修。

## 门禁

- `pnpm test`:321 passed / 26 files(baseline 314,新增 7)
- `pnpm typecheck`:干净
- `pnpm exec eslint src`:143 problems(133 errors + 10 warnings)—— **与 baseline 完全一致**,我这两个文件 0 问题
- `prettier --check`:我改的两个文件干净
- `pnpm build`:通过

## 改动文件

- `ui/src/app-v2/panel/SessionTerminal.tsx` — 自绑 wheel + `scrollLines`,含 cleanup
- `ui/src/app-v2/panel/SessionTerminal.wheel.test.tsx` — 新增,7 条
