# 两件事:实时席位不能自由往上滚看历史 + 中文输入法下 shift+key 符号打不出来

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-liveseats-scroll-and-ime-shift-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

张先生 2026-10-27 报两件:

1. **实时席位这个过渡方案,不能自由向上滚动看历史** —— 要求解决
2. **中文输入法下,需要 shift+key 的符号(如 `,`)打不出来;切成英文输入法就可以**
   —— 要求记为待办并解决

## 第二件要先说清:它可能是今天早上那个修复没覆盖到的情形,不是全新 bug

2026-10-26 修过一个同形状的:`Shift+/` 在 macOS 上 `event.key === '?'`,
`useGlobalShortcuts` 的裸 `?` 帮助快捷键于是在每个问号上触发,
`preventDefault()` 把字符吃掉。修法是加 `isTypingContext()` 守卫
(`isComposing` / INPUT / TEXTAREA / contentEditable)。**已合并、已验绿、
已装到张先生跑的那份里。**

**Aegis 核过修复确实在跑的应用里** —— 但过程里错了一次,记在这里:
我先查 `isTypingContext` 这个**函数名**在不在 bundle 里,结果是 0,就说"修复不在"。
**那个判据是错的**:release 构建会压缩函数名,字符串字面量和属性名才留得下。
改查它的三个判据(`isComposing` / `TEXTAREA` / `contentEditable`)——
**三个全命中**。所以修复在,那个函数名查不到是正常的。

**所以第二件是新情形**:中文输入法按 `shift+,` 时,`isComposing` 大概是 `false`
(没进拼音候选状态,是 IME 直接出符号),`event.target` 也可能不在 INPUT/TEXTAREA 上
(xterm 用隐藏的 helper textarea,但焦点/事件目标在不同 IME 状态下可能不同)。

**你要做的是找出在中文 IME + shift+key 时,`isTypingContext()` 为什么返回 false,
或者为什么即使返回 true 字符还是没进去。** 别照英文输入法的路径推 —— 那条已经通了。

## 第一件:Aegis 已查的

- `ui/src/views/LiveSeatsView.tsx:38` 用 `SessionTerminal`
- `ui/src/app-v2/panel/SessionTerminal.tsx:109` 已经设了 `scrollback: 10000`
- **整个文件没有 `scrollToBottom` 调用**(不是"强制回到底部")
- 输出走 `term.write(bytes)`(`:236`、`:302`),没有看到显式锁 viewport 的代码

**所以不是 scrollback 太小,也不是显式锁底。** 你要查的是:

1. xterm 默认行为:输出到达时如果 viewport 在底部就跟着滚,**用户往上滚之后呢?**
   新输出会不会把他拽回去?(这是最常见的原因,但 Aegis 没有实测,你实测)
2. **鼠标滚轮事件有没有到 xterm**:tmux 那边 Aegis 已经把 `mouse` 打开、
   `history-limit` 调到 50000(2026-10-27);但那是 tmux 侧,
   **SeatLoom UI 里的 xterm 是另一个显示面**,要单独测
3. **键盘 PageUp/Shift+PageUp 能不能滚**(不依赖鼠标)
4. 有没有 `overscroll` / `scrollOnUserInput` 之类的选项被关掉了

## 判据

- 两件都给出实测:在真实窗口里(或最接近的 jsdom+真实 xterm)滚上去、
  停在中间、来一段新输出,viewport 会不会动
- 中文 IME 那件:给出 `event.isComposing`、`event.target.tagName`、
  `event.key` 在中文输入法按 shift+, 时的实际值(**要实测,不许推**)
- 如果你实测不了中文输入法,说清你用什么代替、以及那个代替和真实 IME 的差别

## 明确不做

- 不许把 scrollback 从 10000 改大当成修复 —— 那治不了"被拽回去"
- 不许为了 Shift+key 就把所有 shift 快捷键都禁掉 —— 那会把 ⌘⇧K 之类一起废掉
- 不许改视觉风格
- 不许顺手改别的快捷键
