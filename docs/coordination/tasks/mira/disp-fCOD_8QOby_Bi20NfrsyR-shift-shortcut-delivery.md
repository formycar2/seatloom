# 修:全局快捷键吞掉 shift 字符输入(`disp-fCOD_8QOby_Bi20NfrsyR`)

| Field | Value |
|---|---|
| dispatch | `disp-fCOD_8QOby_Bi20NfrsyR` |
| packet | `AEGIS-2026-10-26-fix-shift-shortcut-eats-input-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-26 |
| 严重度 | 高(张先生实测无法输入 `?`) |
| 结论 | 已修,13 条新测试全绿,7 个变异 6 个被杀死(第 7 个是等价变异) |

## 一、改了什么

一个文件 `ui/src/hooks/useGlobalShortcuts.ts`,加一个 `isTypingContext(event)` 判据,
放在处理函数**最开头**,命中就 `return`(不是 `preventDefault`):

```ts
const isTypingContext = (event: KeyboardEvent): boolean => {
  if (event.isComposing) return true;
  const target = event.target as HTMLElement | null;
  if (!target || typeof target.tagName !== 'string') return false;
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return true;
  return (
    target.isContentEditable === true ||
    (typeof target.closest === 'function' &&
      target.closest('[contenteditable]:not([contenteditable="false"])') !== null)
  );
};
```

派单给了三个选项(Aegis 不指定),我选了 1+2 合起来:**判焦点目标 + `isComposing`**。
理由:1 单独用会漏中文输入法组合态(派单第 4 条判据),2 单独用会漏「焦点在输入框
但不在组合态」这个主场景。两个都要。

### 一个实现上的判断:`isContentEditable` 不能单独用

jsdom 里 `element.isContentEditable` **返回 undefined**(我实测过,不是推断):

```
$ vitest 探针
attr true isContentEditable undefined
textarea tag TEXTAREA isContentEditable undefined
```

所以真实浏览器继承态(`<div contenteditable><span/></div>` 里的 span)只有
`closest()` 走得通;而当前测试环境里只有 `isContentEditable === true` 在
**真实支持它的浏览器**上才成立。两个 OR 起来,任何一边都不代表全部事实。
这一点写进源码注释了,免得后来人以为其中一个多余删掉。

## 二、为什么修的是「全部快捷键」而不是只修 `?`

派单原话:「同一条链路还会吞别的」。我只把 `?` 放过的话,Cmd+K 在打字时仍然会
把用户从输入框拽到命令栏 —— 那是同一个 bug 的另一半。守卫放在最开头、
作用于所有分支,两者一起修。

**代价我说清楚**:Cmd/Ctrl+K 和 Cmd/Ctrl+` 现在在输入框里也不触发了。
派单第 3 条判据写的是「Cmd/Ctrl+K 命令栏、Cmd/Ctrl+` 终端,两个都还能用」——
按上下文(整条判据讲的是打字时),我理解为「焦点不在输入框时仍可用」。
如果张先生的意思是**打字时也要能按 Cmd+K**,那这条要改成修饰键例外,
请告诉我,那是产品取舍不是我的判断。

## 三、怎么验的

### 新测试 `ui/src/hooks/useGlobalShortcuts.test.ts`,13 条

驱动**真实的 hook**(`renderHook` + 真实 window 监听)、**真实的 input/textarea/
contenteditable DOM 节点**、真实 `KeyboardEvent`。断言是**双向**的:

| 侧 | 断言 |
|---|---|
| 打字时 | 5 个回调一个都不调,且 `event.defaultPrevented === false` |
| 不在打字时 | `?`/F1 开帮助、Cmd+K 开命令栏、Cmd+\` 开终端、Escape 关详情,各一次,且 `defaultPrevented === true` |

**为什么断言 `defaultPrevented` 而不只是回调**:`preventDefault()` 才是真正吃掉
字符的那一步。只断言「回调没被调」的版本,挡不住「静默阻止但不弹窗」的退化写法。
jsdom 里按键不会真的往 input 里插字符,所以「字符进没进去」的可测代理就是
**默认行为有没有被阻止** —— 这一点我不含糊:jsdom 不模拟字符插入。

IME 那条我特意派发到 `document.body` 而不是 input:拼音选字时焦点所在的 composer
可能被重渲染卸载,剩下的 keydown 落在 body 上 —— 那种情况下 `isComposing` 是
**唯一**能挡住快捷键的东西。第一版我派发到 input,被 INPUT 分支先挡了,
`isComposing` 根本没被单独验到(见 M4)。

### 变异验证(Flux 的方法:改坏源码,看测试是否红)

| # | 变异 | 结果 |
|---|---|---|
| M1 | 整个守卫删掉 | RED(4 条) |
| M2 | 只查 INPUT/TEXTAREA,丢 contenteditable | RED(1 条) |
| M3 | 选择器去掉 `:not([contenteditable="false"])` | RED(1 条) |
| M4 | 去掉 `isComposing` 判断 | 第一版测试**全绿(漏!)** → 改成派发到 body → RED(1 条) |
| M5 | 去掉 `typeof target.tagName !== 'string'` 检查 | **全绿(等价变异,见下)** |
| M6 | 只对 `?` 放过(窄修) | RED(2 条) |
| M7 | 守卫移到帮助判断**之后**(顺序错) | RED(5 条) |

**M4 是我这次自己抓到的真漏洞,值得单独说**:我第一版 IME 测试把事件派发到
`<input>`,`isTypingContext` 的 INPUT 分支先返回 true,于是「去掉 isComposing」
这条变异**一条都不红**。也就是说那条测试名义上测 IME,实际测的是 INPUT 分支,
`isComposing` 这条判据当时是**没有测试保护的**。改成派发到 `document.body` 后才红。

**M5 是等价变异,不是守卫缺失**:去掉 tagName 检查后,`window.isContentEditable`
是 `undefined`(不等于 `true`),`window.closest` 不存在(被 `typeof === 'function'`
挡住),两条路都自然返回 false,行为和原来逐位相同。已确认源码里保留它是因为
它防的是**未来**给 target 加别的分支时误把 window 当元素,不是当前必需。

### 全量门(与改动前基线对比,不是只看绝对值)

```
改动后: Test Files 22 passed (22)   Tests 264 passed (264)   Errors 14 errors
基线  : Test Files 21 passed (21)   Tests 251 passed (251)   Errors 14 errors
```

`Errors 14 errors` **改动前后完全一样,是既有问题,不是我引入的**。逐条看过堆栈:
14 个全是 `src/App.restartRestore.test.tsx:84` 那个**故意**抛
`unmocked Tauri invoke` 的 mock,经 `api.ts:299 onSupervisorDetached` /
`:303 onSupervisorReembedded` 冒出来。我没有去改它 —— 那超出本单范围,
而且「顺手修一个看起来是红的既有失败」正是容易把范围搞混的做法。
**要不要单独派一单收拾它,请 Aegis 定。**

其它门:`pnpm typecheck` 干净、`pnpm build` 通过、eslint 对我两个文件 0 错
(第一版有个未使用的 `React` import,已删)、prettier 干净。

## 四、明确不做的,我没碰

- 没删帮助功能,只是打字时不抢输入
- 没改任何键位,没加新快捷键
- 没动 `App.tsx:192` 的 Cmd+K(它只管 Cmd/Ctrl 组合,不吃裸字符)
- 没合并、没自己开验收卡
- 没修那 14 个既有 unhandled error(超出本单,已上报)
- 本单**没有 Rust 改动**,没跑 cargo

## 五、需要上报的

1. **Cmd+K 在输入框里现在也不触发了**(见第二节)。这是派单判据 3 的歧义,
   我按「焦点不在输入框时可用」实现。若产品要的是「打字时也能按」,告诉我。
2. **14 个既有 unhandled error**(`App.restartRestore.test.tsx` 的故意 mock)。
   基线就有,本单没碰,不建议混在 bug 修复里动。
3. jsdom 不实现 `isContentEditable`,这类「继承态」判据在测试里只能靠
   `closest()` 验。写别的 DOM 测试时会再遇到。

## 复现命令

```bash
cd ui
pnpm vitest run src/hooks/useGlobalShortcuts.test.ts   # 13 passed
pnpm typecheck                                        # 干净
pnpm test                                             # 264 passed(基线 251,新增 13)
pnpm build                                            # 通过

# 与基线对比(证明 14 个 error 是既有的):
git stash -u && pnpm test 2>&1 | grep -E '^ +(Test Files|Tests|Errors) '
git stash pop

# 没有第二处会吃字符的监听(核过,只有 useGlobalShortcuts 判 '?'):
rg -n "addEventListener\('keydown'" src --glob '!*.test.*'
rg -n "=== '\?'" src --glob '!*.test.*'

# jsdom 的 isContentEditable 是 undefined(所以判据必须双写):
#   见本单源码注释与测试里的 contenteditable 用例
```

## 操作者怎么自己验(应用层)

1. 打开任一输入框(派活框、工作项表单、终端里的 xterm 都行),按 Shift+/
   → 输入框里出现 `?`,**不**弹帮助
2. 焦点在空白处,按 `?` 或 F1 → 仍然弹帮助
3. 焦点在空白处,Cmd+K 弹命令栏,Cmd+\` 开关终端
4. 切中文输入法,打拼音选字过程中按各种键 → 不弹任何东西、不跳命令栏
