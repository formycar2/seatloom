# 修:实时席位页面不能上下滚屏(`disp-yhtJXKP7_krz7NBwz6v12`)

| Field | Value |
|---|---|
| dispatch | `disp-yhtJXKP7_krz7NBwz6v12` |
| packet | `AEGIS-2026-10-26-liveseats-scroll-unreliable-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-26 |
| 结论 | **根因不是 Aegis 排第一的那个假设**。真因是另一条:`display:none` 让 xterm 量不到字符格,FitAddon.fit() **静默不做事**,viewport 停在 attach-time 种子尺寸。已修,两个文件,18 条测试,7 个变异全被杀死 |

## 一、先纠正派单里的一个判断

派单按可能性排,第一嫌疑是「切换时 fit() 算出 rows=0,viewport 高度塌掉」。
**这个假设不成立**,我去读了装着的 FitAddon 源码(`node_modules/@xterm/addon-fit/src/FitAddon.ts`):

```js
const geometry = {
  cols: Math.max(MINIMUM_COLS, Math.floor(availableWidth / dims.css.cell.width)),
  rows: Math.max(MINIMUM_ROWS, Math.floor(availableHeight / dims.css.cell.height))
};
```

`MINIMUM_ROWS = 1`、`MINIMUM_COLS = 2`。**rows 永远不可能是 0**,所以「算出 rows=0 塌掉」这条路不存在。

但读源码时发现了真正的洞,在同一文件上面几行:

```js
public fit(): void {
  const dims = this.proposeDimensions();
  if (!dims || !this._terminal || isNaN(dims.cols) || isNaN(dims.rows)) {
    return;                       // ← 静默返回,不 resize、不报错、不打日志
  }
  ...
}
public proposeDimensions() {
  const dims = ..._renderService.dimensions;
  if (dims.css.cell.width === 0 || dims.css.cell.height === 0) {
    return undefined;             // ← 量不到字符格就返回 undefined
  }
  ...
}
```

**`fit()` 在量不到字符格时是个静默 no-op。** 调用方无法区分「fit 成功了」和
「fit 什么都没做」—— 除非自己先问一次 `proposeDimensions()`。

## 二、根因链(逐环都有出处)

1. `LiveSeatsView.tsx:600` 每个接入席位一层 `position:absolute; inset:0`,
   用 `display: a.seat.sessionName===activeName ? 'block':'none'` 切换显隐。
2. **一个 `display:none` 的祖先把元素从布局里摘掉了**,盒子不存在。
   xterm 的字符尺寸测量读的是 measure 元素的 `offsetWidth/offsetHeight`
   (`CharSizeService` → `Number(this._measureElement.offsetWidth)/32`)——
   没有盒子 → 宽高 0 → `cell.width === 0`。
3. `FitAddon.proposeDimensions()` 返回 `undefined` → `fit()` **静默返回**。
4. 于是 viewport 停在**构造时的种子尺寸**:`SessionTerminal` 故意用 pane 的
   attach-time rows/cols 当种子(文件头的 geometry contract,为了让首帧不被
   re-wrap)。那个种子**通常比这个容器高**(QL 实测 54 行)。
5. xterm 的滚动区高度 = `bufferLength * rowHeight`,viewport 高度 = 容器高度。
   当 pane 内容行数 **少于种子行数**时,滚动区高度 ≤ viewport 高度 →
   **没有可滚的东西**。页面明明在,滚不动。

**为什么「一定几率」**:能不能 fit 成功取决于 arm 那一刻字符尺寸量没量到。
display:none 的层在 React 挂载后是否已经量到尺寸,是个时序问题 ——
同一个操作有时成功有时不成功。这也解释了为什么 QL 最常暴露:它是非标准席位。

xterm 自己是有 IntersectionObserver 的(元素变可见时会重测字符尺寸),
但**它只在尺寸无效时重测,重测完没有任何人重跑 fit** —— 它重渲染用的是
*当前* rows,而 rows 还是种子值。缺的就是这一步。

## 三、改了什么

### 1. `ui/src/views/LiveSeatsView.tsx` —— 隐藏方式从 `display:none` 改成 `visibility:hidden`

```diff
-  display: a.seat.sessionName === activeName ? 'block' : 'none',
+  visibility: a.seat.sessionName === activeName ? 'visible' : 'hidden',
+  pointerEvents: a.seat.sessionName === activeName ? 'auto' : 'none',
```

- `visibility:hidden` **保留布局盒子**,所以非激活层的终端照样量得到字符格,
  一旦 arm 就能 fit 对。这些层本来就已经是 `position:absolute`(脱离文档流),
  所以改成 visibility 不会引入第二个可见层。
- 顺手保留 `display:none` 原有的两个性质:不进无障碍树、不可聚焦。
- `pointerEvents:none` 是兜底:非激活层绝不能吃掉滚轮或点击。

### 2. `ui/src/app-v2/panel/SessionTerminal.tsx` —— 主动发现「fit 其实没做」并重试

```ts
const proposed = fitAddon.proposeDimensions();
if (!proposed) {
  geometryFitted = false;   // fit() 会是静默 no-op,记下「还没 fit 上」
  return;
}
fitAddon.fit();
geometryFitted = true;
```

外加两条重试通道(原来只有 container ResizeObserver + window resize):

```ts
const retryIfUnfitted = () => { if (!geometryFitted) syncGeometry(); };
const elementResizeObserver = term.element ? new ResizeObserver(retryIfUnfitted) : null;
if (term.element && elementResizeObserver) elementResizeObserver.observe(term.element);
const resizeSubscription = term.onResize(retryIfUnfitted);
```

为什么是这两条:xterm 自己会在「变可见 / dpr 变化」时重测字符尺寸,而尺寸变化
会让它按**当前 rows** 重渲染,节点高度随之改变 —— 所以 `term.element` 的盒子
变化,就是「之前 fit 不了,现在可能能了」的可观测信号。
在此之前,只有 resize window 才会重新 fit,这正是「一定几率」的来源:
**要等一次无关的窗口 resize 才会好**。

清理里补了 `elementResizeObserver?.disconnect()` 和 `resizeSubscription.dispose()`。

## 四、怎么验的

### 测试:21 条(全部新增:几何 10 + DOM 5 + 原有 LiveSeatsView 8 条不动)

**`src/app-v2/panel/SessionTerminal.geometry.test.tsx`(10 条,新文件)** ——
驱动**真实组件**,对着 FitAddon 契约的**转写**(不是记忆,是从装着的源码抄的):

| 断言 | 覆盖 |
|---|---|
| 量不到字符格 → `resizeCalls` 空、后端 0 次 push、rows/cols 还是种子 54×219 | 缺陷的稳态 |
| 量得到 → fit + push(140×30) | 正常路径 |
| 先量不到、后量得到 → 自动重试 fit(不靠 window resize) | **修复本身** |
| xterm 自己的 `onResize` 也能触发重试 | 第二条通道不是死订阅 |
| `term.element` 被 ResizeObserver 盯着 + 触发它就能 fit | 第一条通道不是死订阅 |
| fit 成功后丢度量、再拿回来(不同尺寸)→ 再 fit 一次 | 标志是**活答案**不是单向锁 |
| 同一几何重复 resize → 后端只 push 一次 | 不刷屏 |
| 容器无面积 → 不 fit | 旧守卫的第二道防线没丢 |
| 卸载 → dispose + 监听清空 | 不泄漏 |

**`src/views/LiveSeatsView.test.tsx`(追加 5 条)** —— 真实视图 + mock 终端:
激活层 visible/auto、非激活层 hidden 但 `display !== none` 且 `pointerEvents:none`、
切席位两个终端都不卸载且可见性正确翻转。

**我在这个文件里明确写了它不能断言什么**:jsdom 完全没有布局(每个元素
`clientWidth/clientHeight` 都是 0),所以「隐藏层仍有真盒子」是**结构性断言**
(`display` 不是 `none`、`visibility` 是 `hidden`),不是测量。
可测的那一半在 geometry 测试里。

### 变异验证(7 个,全部被杀死)

| # | 变异 | 结果 |
|---|---|---|
| M1 | 去掉 `proposeDimensions()` 预检,回到无条件 fit | RED 6 |
| M2' | 两个观察点都留着,但 `retryIfUnfitted` 空转 | RED 2 |
| M3 | 只删 `term.onResize` 订阅(会抛错,不算精确变异) | 弃用,改用 M2' |
| M4 | `geometryFitted` 变单向锁(永不回 false) | RED 1 |
| M5 | `visibility` 改回 `display:none` | RED 2 |
| M6 | 删掉 `pointerEvents` | RED 2 |
| M7 | 激活层写成 `hidden`(写反) | RED 3 |

**两个我第一版测试没覆盖到、被变异抓出来的真缺口:**

- **M2' 一开始只杀死 1 条**:我的 `StubResizeObserver` 有 `fire()` 却没在测试里
  用过 —— `term.element` 那条观察器**当时没有任何测试保护**。补了那条断言后才
  杀死 2 条。
- **M4 一开始全绿**:我的测试用 `window.dispatchEvent` 触发重试,而 window 路径
  是**无条件直接调 `syncGeometry()`** 的,绕过了 `geometryFitted` 标志。
  改成经 element observer 触发(唯一会看标志的通道)后才红。
  这两个都是「测试用的触发方式和被测代码的判定条件不一致」—— 测了,但没测到。

### 全量门(与基线对比,不是只看绝对值)

```
改动后: Test Files 23 passed (23)   Tests 277 passed (277)   Errors 14 errors
基线  : Test Files 22 passed (22)   Tests 264 passed (264)   Errors 14 errors
```

`Errors 14 errors` 两边一致,**既有问题**(`App.restartRestore.test.tsx:84`
故意抛的 Tauri mock),已 stash 对比确认,本单没碰。

`pnpm typecheck` 干净、`pnpm build` 通过。
eslint 全仓 **143 problems(133 error + 10 warning),与基线逐字相同**;
我改的 4 个文件 **0 error**,2 个 warning 是既有的(`seats` conditional)。
第一版我跑过 `prettier --write`,它顺手重排了 `LiveSeatsView.tsx` 两处
**与本单无关的既有代码**,已回退 —— 那会让 diff 变脏、掩盖真改动。
现在这两个文件 prettier 的告警与基线相同(都是既有那两行)。

## 五、明确不做的,我没碰

- **没用定时 `refresh()` 糊**(派单明令禁止):重试挂在真实信号上
  (`term.element` 的盒子变化、xterm 的 `onResize`),不是轮询。
- 没改 SeatLoom「只观察不拥有」的定位,没给终端加输入框。
- 没加任何快捷键。
- 没改 `useGlobalShortcuts`、没碰 Rust、没合并、没自己开验收卡。
- 没顺手修那 14 个既有 unhandled error(范围外,已三次上报)。

## 六、派单三条可疑点的核验结果(核不掉的就实说)

| 派单假设 | 结论 | 怎么核的 |
|---|---|---|
| 1. fit 算出 rows=0,viewport 塌掉 | **不成立** | 读 FitAddon 源码:`MINIMUM_ROWS=1`、`MINIMUM_COLS=2`,rows 不可能是 0。但同一处发现真洞:`cell.width===0` 时 `proposeDimensions()` 返回 undefined、`fit()` 静默返回 |
| 2. xterm viewport 未获焦,滚轮被外层 window 吃掉 | **不是本 bug 的成因** | xterm 的 viewport CSS 是 `overflow-y:scroll; position:absolute; inset:0`,滚动区高度由 buffer 行数决定。内容比种子短时滚动区≤viewport,**没有可滚距离**,与焦点无关。焦点问题我**没有独立复现过**,不声称它不存在 |
| 3. 多个 absolute 层叠,旧实例 scrollback 干扰 | **不是成因** | 层叠层互不重叠(都 `inset:0` 且只有一个可见),且旧实例的 scrollback 不影响另一个实例的 viewport 高度。**这条我没有实测到干扰**,如实说核掉了 |

## 七、需要上报

1. **「一定几率」的机制值得记住**:`display:none` 的祖先让依赖布局的第三方组件
   (xterm 字符测量)量不到东西,而它的失败是**静默**的。同类模式在别处可能还有 ——
   任何 `display:none` 包着需要测量的组件都值得看一眼。我没全仓扫,
   **扫全仓超出本单范围**,要的话请派单。
2. **派单的第一嫌疑是错的**(rows=0)。Aegis 读源码时值得连 `MINIMUM_ROWS`
   这类下限一起看 —— 我一开始也准备照这个假设修,是读源码才发现方向不对。
3. 14 个既有 unhandled error 仍未处理(第 N 次上报)。

## 复现命令

```bash
cd ui
pnpm test                                    # 277 passed (基线 264,新增 13)
pnpm vitest run src/app-v2/panel/SessionTerminal.geometry.test.tsx   # 10 passed
pnpm vitest run src/views/LiveSeatsView.test.tsx                      # 13 passed(8 原有 + 5 新)
pnpm typecheck                               # 干净
pnpm build                                   # 通过

# 与基线对比(证明 14 errors 与 143 lint 问题都是既有的):
git stash -u && pnpm test 2>&1 | grep -E '^ +(Test Files|Tests|Errors) '
git stash -u && pnpm exec eslint src 2>&1 | grep '✖'
git stash pop

# FitAddon 的真身(判据出处,不是我编的):
sed -n '30,60p'  node_modules/.pnpm/@xterm+addon-fit@0.10.0*/node_modules/@xterm/addon-fit/src/FitAddon.ts

# 修复后没有第二处 display:none 包着终端:
rg -n "display:.*none" src/views/LiveSeatsView.tsx
```

## 操作者怎么自己验

1. 接入 2 个以上席位,**反复切 ≥10 次**,每次切到 QL 后**立刻**滚轮/触控板上下翻
   → 每次都滚得动(目标:翻不动 0 次)
2. 切到 QL 后按 ⌘⌥I 看 `.xterm-viewport` 的 `scrollHeight` 与
   `.xterm-screen` 的高度,或看终端行数是否等于容器高度(不再是 54 行)
3. 滚轮在终端区里翻 scrollback;在侧栏/标题栏滚,页面滚动照旧归页面
   (非激活层已 `pointerEvents:none`,不会劫走)
