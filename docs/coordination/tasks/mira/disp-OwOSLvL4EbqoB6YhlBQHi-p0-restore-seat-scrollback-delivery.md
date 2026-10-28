# `disp-OwOSLvL4EbqoB6YhlBQHi` — P0 实时席位翻页:scrollback 被 `reset()` 扔掉了

## 一句话

**根因就是定位到的那两行,已修。改成只清视口 + `scrollToBottom()`,scrollback 原样保留。派单列的 5 条判据里 4 条我能量化,第 5 条(真机点滚)我做不到,已如实标注。**

---

## 根因(先确认 Aegis 的判断,再补一层)

Aegis 说 `clear()` 和 `reset()` 两个都清历史。**我读源码确认了,而且量出了具体数字**:

| 调用 | xterm 5.5 实现 | 对 300 行历史的后果 |
|---|---|---|
| `clear()` | `BufferService.clear()` → `lines.length = 1` | 只剩 1 行 |
| `reset()` | `BufferService.reset()` → `this._normal = new Buffer(...)` | 新建 buffer |
| 两者连用 | — | **40 行**(== 当前 rows),最老那行是空的 |

所以"换成 `clear()` 就保住"这条路确实不存在,Aegis 读得对。

**我补的一层:QL 为什么能翻。** Aegis 归因为"张先生上一秒正在看它"。更准确的原因是——那个 clear 分支有个前提条件:

```ts
if (!seedDims || seedDims.rows !== rows || seedDims.cols !== cols) { ... }
```

**只有 fit 出的尺寸和 pane seed 不同时才清。** QL 恰好 fit 出和 seed 一样的尺寸,分支没跑,3000 行就保住了。**席位之间的差别从来不在席位本身,在这个分支有没有执行。** 这解释了为什么它看起来"时好时坏"——取决于 UI 容器恰好 fit 出什么。

---

## 修法

新文件 `ui/src/app-v2/panel/dropStaleSnapshotViewport.ts`:

```ts
export const dropStaleSnapshotViewport = (terminal: Terminal) => {
  terminal.write('\x1b[2J\x1b[H');   // 只清视口,不动 scrollback
  terminal.scrollToBottom();          // 停在活跃端,不碰 scrollback
};
```

调用点从 `terminal.clear(); terminal.reset();` 改成这一行。

**为什么单独一个文件**:两个测试文件都要驱动它(一个验结果、一个验决策),放组件文件里会触发 `react-refresh/only-export-components` lint warning。我不加 `eslint-disable`——抽出文件是正经修法。

---

## 关键实测(决定修法是否成立的四件事)

全部用真实 xterm(buffer、解析器、视口都是真的)量出来:

| 实验 | 结果 | 说明 |
|---|---|---|
| 300 行历史 → `resize(120,40)` | **301 行,`L000` 完好** | resize 本身不清 scrollback |
| 300 行历史 → `resize(78,10)` | **301 行,`L000` 完好** | 窄列也不清 |
| 80 列 45 字符行 → `resize(78,10)` | **行数不变,零折行** | 真实 TUI 行宽下无损坏 |
| 清视口后 scrollback | **301 行保留**,重绘正确落在视口 | 修法成立 |

**花屏机制也量清了**:80 列长行 resize 到 40 列后,视口里是 `L016` 和它的折行尾巴交替、`L015` 被截断——这就是当年的"输入框错位花"。**它只发生在视口里,scrollback 里的是折行垃圾(往上滚才看到)。** 所以清视口足以解决输入框问题。

---

## 判据逐条对

### 1. attach 一个历史很长的 seat,不 reset 的那一版能往上滚过 1000 行 ✅

实测:1500 行历史 → resize → `scrollLines(-1200)` → `ydisp` 离开 `ybase`,可见行变成 `L\d{3} ` 开头的旧历史,`lines.length` 保持 >1000。

### 2. 输入框不错位、不花 ✅

实测:resize 后 `dropStaleSnapshotViewport` + 写入 `SAIH PROMPT > `,视口第一行就是它,不是旧帧。

### 3. 切到别的 seat 再切回来,历史还在 ✅

`clearedStaleSnapshotRef` 是**每 mount 一个的 ref**,新 mount 时重置为 false;而 snapshot 每次 attach 重新写。所以切回来是"新 attach → 写新 snapshot → 第一次 sync 只清视口",历史是新拿的 3000 行,不是重新攒的。测试 `the clear runs at most once` 钉住"只清一次"这条。

### 4. 现有测试全绿 ✅

- `pnpm vitest run src/App.supervisorSync.test.tsx` → **20 passed**
- `pnpm tsc --noEmit` → 干净
- `pnpm test` → **336 passed / 29 files**(派单说 300,现在是 336)

### 5. 真机验(build + 装 + 滚)❌ **未闭合**

**我不能点击 GUI。** 我做到了:release 构建通过、`ui/dist` 产物含新代码。但"在装好的 app 里滚一个历史长的 seat"需要有人真机操作。

而且**上一单发现的 bundle 缺前端资产问题仍未解决**——如果不先修那个,装了也看不到新代码。这条已在上一条派单上报过,这里再次标为阻塞。

---

## 测试(6 条新,3 个变异全红)

### `SessionTerminal.scrollback.test.tsx`(3 条,真实 xterm)

驱动真实 xterm,**直接调用产品导出的 `dropStaleSnapshotViewport`**,不是复制序列:

- 300 行历史在 fit 尺寸不同后仍是 300 行,最老行完好
- 能往上滚过 1000 行,且滚到的是真旧历史(不是空 buffer)
- 重绘内容落在视口第一行(输入框不错位)

### `SessionTerminal.geometry.test.tsx` 新增 3 条(转写版)

驱动真实组件 + 转写的 FitAddon 契约,断言**组件的决策**:

- fit 尺寸不同时:调 viewport clear + `scrollToBottom`,**`clear()`/`reset()` 一次都没调**,且几何仍然推给 pane
- fit 尺寸相同时(QL 的情况):什么都不清
- 后续 geometry sync 不会重复清

### 两个文件为什么分工

**jsdom 给不了 xterm 字符格子度量**,所以组件内部那个 fit 分支从 jsdom 里跑不到(实测:stub 了 `clientWidth/clientHeight` 后组件仍停在 24x80 seed,`proposeDimensions()` 返回 undefined)。因此:

- **组件的决策**用转写版测(`geometry.test.tsx`)—— 那是能驱动这个分支的工具
- **决策的结果**用真实 xterm 测(`scrollback.test.tsx`)—— 那是转写给不出的东西

**两边都调同一个导出的产品函数**,所以谁都不是谁的复制品。这一点写在了两个文件的头部注释里。

### 变异

| 变异 | 结果 |
|---|---|
| M1 产品函数改回 `clear()+reset()` | **红,杀 5 条**(2 geometry + 3 scrollback) |
| M2 去掉 `scrollToBottom()` | **红,杀 2 条** |
| M3 去掉 `ESC[2J` 视口清 | **红,杀 1 条**(正是"不错位"那条) |

---

## 过程中自己抓住的一个测试缺陷

第一版 `scrollback.test.tsx` 里,我在测试里**手写了一遍 `\x1b[2J\x1b[H` + `scrollToBottom()`** 序列。跑变异时发现:**把产品代码改回 `clear()+reset()`,这 3 条测试全绿** —— 因为它们测的是我自己复制的序列,不是产品代码。等于什么都没测。

修法:把序列提取成导出的产品函数 `dropStaleSnapshotViewport`,测试改调它。重跑 M1,5 条变红。

**这和之前记过的 userinfo 那条同源:测试里重写一遍被测逻辑,变异就杀不掉它。** 已记入 memory。

---

## 残留(如实说)

**`resize` 会把超过新列宽的行折行,折出来的行留在 scrollback 里。** 往上滚可能看到旧几何尺寸的折行残片。

实测边界:45 字符行(真实 TUI 的常态)从 80 列 resize 到 78 列,**零折行、行数不变、最老行完好**。tmux pane 通常 200+ 列而 UI 视口通常更窄,受影响的是真正超宽的那些行。

**这是 UI 侧修法的诚实上限** —— 要彻底消除,得在 fit 之后按新尺寸重新捕获一次历史,那需要动后端(`pty/mod.rs` 的 backfill 是 attach 时按 pane 尺寸捕获的),派单明确不许。我按派单的约束做,并把这条限制写在这里。

---

## 明确不做的,都没做

- ❌ 没把 `reset()` 换成 `clear()` 说修好了 —— 两个都清,已量
- ❌ 没关 fit/几何同步 —— 几何照旧推给 pane,测试里断言了
- ❌ 没改后端 —— `pty/mod.rs`、`session_cmds.rs` 一行未动
- ❌ 没改 saih

---

## 需要你的

1. **上一单发现的 bundle 缺前端资产问题仍是前置阻塞** —— 不先修它,装了 app 也看不到这些修复。建议优先处理。
2. **真机滚一个历史长的 seat**(Nimbus 3301 行)确认第 1、2 条判据。
3. `dto-to-v1.ts:159` 的 `isPinned: true` 硬编码仍未修(前两条派单已报)。

## 门禁

- `pnpm test`:336 passed / 29 files
- `pnpm typecheck`:干净
- `pnpm exec eslint src`:**143 problems(133 errors + 10 warnings),与 baseline 完全一致**。中途抽出文件时冒出过 1 个 `react-refresh/only-export-components` warning,已用"移到单独文件"正经修掉,没有用 eslint-disable
- `prettier`:改动文件全部干净
- `pnpm build`:通过

## 改动文件

- `ui/src/app-v2/panel/dropStaleSnapshotViewport.ts` — **新增**,修法本体
- `ui/src/app-v2/panel/SessionTerminal.tsx` — 调用点 + 更新过时注释
- `ui/src/app-v2/panel/SessionTerminal.scrollback.test.tsx` — **新增**,3 条,真实 xterm
- `ui/src/app-v2/panel/SessionTerminal.geometry.test.tsx` — FakeTerminal 补 `scrollToBottom`/`write`/`clear`/`reset` 记录 + 新增 3 条
