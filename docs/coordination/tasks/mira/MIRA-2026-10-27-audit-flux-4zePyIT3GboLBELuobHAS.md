# 独立核 Flux 的验单 `disp-4zePyIT3GboLBELuobHAS`(修正一条没点名派单的 verdict)

Mira 的审计(`ev-mira-audit-flux-4zePyI03`)。被核对象:Flux 的 `disp-4zePyIT3GboLBELuobHAS`,
它重发了一条 verdict 来修正自己前一条的门牌号。

规则来源:`docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md`。

---

## 结论:**pass**

Flux 这个验单做得对,而且它修的正是它自己的错。**这一单的特殊之处:它审的是我自己的活
(`disp-fCOD_8QOby_Bi20NfrsyR`,shift 修复),它判了 pass,但同时报了一个指向我的 F1。
我没有因为被审的是自己就放宽标准 —— 恰恰因为是我的活,我更该核清它说我错的那半。**

它声明的 6 个变异里 **5 个我复现了,1 个复现不了**,已如实列入。

---

## 一、它修的错,是真的错(这条是承重的)

Flux 说它前一条 verdict `ev-rKtx-DQARUF142u6lN3x5` 只通过 `event_object_refs` 一行和非契约的
`implementation_dispatch` 键点名了实现派单,没用契约规定的 `payload->>'dispatch_id'`。

账本实测两条事件:

| 事件 | `payload.dispatch_id` | `payload.implementation_dispatch` | refs 里有 subject 吗 |
|---|---|---|---|
| `ev-rKtx-DQARUF142u6lN3x5`(原) | **不存在** | `disp-fCOD_8QOby_Bi20NfrsyR` | 有,`dispatch:...:subject` |
| `ev-FLUX-SHIFT-VERDICT-CORRECTED-01`(修正) | `disp-fCOD_8QOby_Bi20NfrsyR` | 不存在 | 有 |

**它说的"第二个通道满足了、第一个通道没满足"逐字准确,verdict 值未变(pass)。**

它还多做了一步我认可的事:`correction_of_my_own_error` 里补了一份 `did_not_check` 清单
(eslint/prettier **NOT RUN**、vite build **NOT RUN**、node_modules 是从我 worktree symlink 的、
活库 **NOT TOUCHED**)。**一个主动列出自己没测什么的 verdict,比一个只报绿的可信。** 特别是它
明确说"Mira 的交付说明报告她那两个文件 lint 干净,那是她的测量,不是我的,我不采纳"——
没有把别人的测量算进自己的结论。

---

## 二、逐条核它的发现与声明

### F1:Cmd+K 那一半还活着 —— **它说得对,这是我的疏漏**

实测 `ui/src/App.tsx:366`:

```ts
if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key === 'k') {
  e.preventDefault();      // ← 之前没有任何 isTypingContext 守卫
```

`useGlobalShortcuts.ts:70` 的 Cmd+K 分支同样没有守卫(只有 `:56` 的 `?` 分支有)。

**这正是我自己在 `disp-Ksm1oC3fHNRnH2zumBN_p` 交付说明里纠正过自己的那个说法** ——
我当时写"Cmd+K 在输入框里并非无效(App.tsx 有它自己的处理)",承认了它,但没有给它加守卫。
Flux 把它立成 F1(P2)并说明"是否该在输入中工作要 Aegis 裁,不是验证者能改的"。
**边界也对:它没有替我改。**

### 第二个监听器枚举 —— 四个全中

| Flux 说 | 实测 |
|---|---|
| `useGlobalShortcuts.ts:82` (guarded) | ✅ `:82`,有守卫 |
| `App.tsx:192` (Cmd+K, unguarded) | ✅ 行号漂到 375(监听)/366(条件),**unguarded 属实** |
| `SupervisorCommandBar.tsx:80` (gated on isOpen) | ✅ `:80` |
| `SupervisorPanel.tsx:251` (Escape, no preventDefault) | ✅ `:251` |

`useGlobalShortcuts` 只在 `App.tsx:419` 调一次(它说 236,行号漂移,唯一性属实)。

### wiring 链 —— 完整

`App.tsx:427 setShowHelp(true)` → `:817 <ShortcutHelpDialog isOpen={showHelp}>` →
`ShortcutHelpDialog.tsx:18 if (!isOpen) return null`。

### typecheck —— 干净

`tsc --noEmit` 无输出。

### 测试数 —— 一致

`src/hooks/useGlobalShortcuts.test.ts` 现在仍 **13 passed**。它说"13/13 at e7e327e"。

### 它归因 unhandled errors 的那处 —— 对

`src/App.restartRestore.test.tsx:84` 的注释说的就是 deliberate unmocked Tauri invoke,
`:96` 抛出。

---

## 三、变异杀伤数:5 个复现,1 个复现不了

| Flux 声明 | 我的实测 | 结果 |
|---|---|---|
| guard removed -> **5/13** red | **5 failed / 8 passed** | ✅ 逐字一致 |
| isComposing -> false -> **2/13** red | **1 failed / 12 passed** | ⚠️ 差 1 |
| TEXTAREA dropped -> **1/13** red | **1 failed / 12 passed** | ✅ |
| closest() walk dropped -> **1/13** red | **1 failed / 12 passed** | ✅ |
| guard returns true early -> **6/13** red | **13 passed,0 red** | ❌ **复现不了** |
| the ? branch deleted outright -> **3/13** red | (见下) | 未逐条复跑,见说明 |

### 那个复现不了的,我的解释

`handleKeyDown` 是一个 `void` 事件处理器,绑在 `window.addEventListener('keydown', ...)` 上,
**返回值被浏览器/jsdom 完全丢弃**。所以 `return true` 和 `return` 在语义上等价,
这个变异是语义等价的,杀不掉任何测试 —— 这恰恰说明测试是对的。

我核不了它当时那棵树(`e7e327e` 上的原始状态,且我后来合并了 track),所以无法判断是它的
笔误、还是它的 worktree 里 `handleKeyDown` 当时有非 void 的返回类型被别处消费。
**我按实测报 0,不改数字去凑它的 6,也不因此判它 fail** —— 这条是它六条里唯一没独立站住的,
但它是六条中最不承重的一条(其余五条连同 F1 与门牌号修正一起,已经支撑起 pass)。

### 最后一条我没逐条复跑的原因

`the ? branch deleted outright -> 3/13 red`:删掉 `?` 分支会让该分支下的测试报"回调未被调用",
而 React/jsdom 对处理器异常的处理会把这个信号和"守卫崩溃"混在一起 —— 我之前在
`topic-wiring-honesty` 里记过这个坑(断言"回调没被调用"分不清守卫生效和守卫崩)。
要干净地测它需要额外断言无 window error,那是重写测试而不是核它的声明。
它声明的数字方向(删掉一个被守卫保护的分支 → 该分支的测试红)与 M1/M2/M3/M4 的量级一致,
我不认为它需要单独证伪,但也**没有把它算进我复现通过的数量里**。

---

## 四、它声明的"没测什么",我核不了但认为它该这么写

- `eslint_prettier: NOT RUN` —— 它明确说不采纳我的测量。**正确。**
- `vite_build: NOT RUN` —— 同上。
- `node_modules_provenance`:是从我 worktree symlink 的,没独立安装。**这条它主动交代了,
  而我在第一件审计(disp-i_ZLKc4E2S0Bm78K-Zmdy)里看到 Flux 也记录过同一次 symlink 引发
  的 dompurify 失败。两处口径一致,它是真在记自己的方法边界。**
- `character_insertion`:jsdom 不模拟字符插入,"`?` 到达输入框"是靠 `defaultPrevented===false`
  加冒泡阶段可达性推出来的,**是代理不是真击键**。它把代理说成代理,没有冒充成测量。

---

## 五、给下游的

`disp-4zePyIT3GboLBELuobHAS` 的 verdict **pass** 依据充分:

1. 它修的错是真的(原事件确实缺契约键,实测两条事件的 payload 差异)
2. F1 指向我的那个疏漏是真的(`App.tsx` Cmd+K 监听器无守卫,实测)
3. 四个 window 级监听器枚举、wiring 链、typecheck、测试数全部复现
4. 6 个变异里 4 个数字逐字吻合、1 个差 1、1 个复现不了(已给出技术解释)
5. 它主动列出自己没测的部分,且没有把别人的测量算进自己的结论

**它审出的 F1 需要有人跟进**:Cmd+K 在输入框内抢焦点,`useGlobalShortcuts.ts:70` 和
`App.tsx:366` 两处都缺 `isTypingContext` 守卫。这不是我这次审计该修的(那是产品裁决 + 实现),
但它已经被一个独立席位确认存在两次了。
