---
title: "I shipped a false cause in a dispatch, and the seat proved it with the source"
category: knowledge
topic: debugging-method
tags: [paste, xterm, onData, tmux, paste-buffer, false-cause, measure-the-symptom]
created: 2026-10-30T12:30:00+08:00
source_kind: canonical_event
source_ref: ev-seat-paste-fragments-multi-line-input
confidence: inferred
curation: authored
---

# 我把一个假结论写进了派单,而席位拿源码证明它错

## 张先生报的

> 贴给原生 tmux 启动的 saih,符合我预期。然而当我通过 Seat 贴过去,
> 就会出现其中一部分内容按行一行一行的提交给了 saih…这使得我的指令破碎了。

## 我一开始的假设(错的)

看到 `paste-buffer` 没带 `-p`,而 tmux 自己的 `prefix ]` 和鼠标粘贴都带 `-p`,
我判断:**缺 `-p` = 缺 bracketed paste = 接收方不知道这是一次粘贴。**

这个推理看起来无懈可击:官方绑定用 `-p`,我们不用。

## 实测推翻它

在一个一次性 tmux 会话里跑真 `sh`,用 `cat -v` 回显原始字节:

| 方式 | `cat -v` 看到 |
|---|---|
| `paste-buffer`(无 `-p`) | `X-lin1` / `X-lin2`,**无 `^[[200~` 包裹** |
| `paste-buffer -p` | `Y-lin1` / `Y-lin2`,**同样无包裹** |

**两种方式字节完全相同。**

原因:bracketed paste 的包裹标记,只在**接收方程序自己声明支持**(发 `ESC[?2004h`)时
tmux 才包。`cat` 没声明。

🔴🔴 **我接着写了一句"saih 也没声明(grep 整个 saih 源码树,2004 零命中)"—— 那句是假的,
Nimbus 当场反驳并复现通过:`saih/pi/packages/tui/src/terminal.ts:212` 明确写着
`process.stdout.write("\x1b[?2004h")`,同文件 485 行关(`?2004l`),530 行序列里也关。**

**我的 grep 为什么零命中:我搜的是 `grep -rn '2004' saih/ --include=*.ts`,
而 `saih/pi/packages/tui/dist/bundle/chunks/` 里有单行几百 KB 的巨型 chunk,
2004 作为数字在 Unicode 宽度表里到处都是,真信号被彻底淹掉;
更致命的是源码里写的是转义形式 `\x1b[?2004h`,**不含字面量 `2004`** ——
我搜 `2004` 就搜不到它,搜 `2004h` 才命中。**

**这是同一天第二次"空输出被当成结论"。**(上一次是把 `track/infa` 拼错,
`git ls-tree` 静默返回空,我读成"文件不在"。)

所以我的 `-p` 测量本身没错(对 `cat` 测确实无效),**但我据它推出的
"saih 不支持所以 `-p` 无用"这个结论是错的** —— 我拿一个不声明 2004 的程序
(`cat`)的测量结果,去断言一个**确实声明了** 2004 的程序(saih)的行为。

**所以 `-p` 在这里是空操作。** 我据它派工会让人改一个没有效果的字符。

## 真正的根因(实测出来的)

不是 tmux 侧,是**前端**:

1. `grep -rn onPaste ui/src`(排除测试)→ **零命中**。没有任何粘贴拦截。
2. `SessionTerminal.tsx:279` 只有 `term.onData`,把每一段都 `ptyWriteBytes` 转发。
3. xterm.js 5.5.0 **没有 `onPaste` handler 时,粘贴走默认逐字符输入路径** ——
   注释在 275-277 行还写着"onData gives the already-decoded byte stream including paste",
   那句话对,但它**没说一次粘贴是作为一个 chunk 到达的**,而它不是。
4. `pty/mod.rs:700 PtySession::write` **每次调用 spawn 一个全新 tmux 进程**
   (`load-buffer -b seatloom - ; paste-buffer -b seatloom -t <target>`)。
   没有任何缓冲或合并。N 段 onData = N 次独立往返。
5. 文本里的 `\n` 到 tmux 层就是**回车 = 一次提交**。saih 的输入框把每次提交当一条独立消息。

## 决定性复现

一次性会话 + 真 `sh` pane,粘一段三行文本:

```
第一行
第二行
第三行
```

**三行被当成三条独立命令执行。** 而且 pane 里抓到了这一行:

```
目标：少华老师…去参加测评cat > /tmp/aegis-paste2.txt
```

**上一段没有尾换行的粘贴留在输入行里,和我接着敲的命令拼在了一起。**

这正对应他说的两件事:一行一行提交 + 指令破碎。

## 规矩

### 1. "官方默认用 X 而我们没用"不是证据

`prefix ]` 绑 `paste-buffer -p` 是事实,但那证明的是 tmux 作者认为 `-p` 是对的,
**不证明在我们的链路上它有效**。链路上游(xterm 逐字符)和下游(saih 不声明 2004)
都可能是别的原因。

### 2. 一个会改变字节的开关,要用能显示字节的接收方验

`cat -v` 是关键 —— 它把 `ESC` 显示成 `^[`。如果我只看 pane 上"文字出现了",
两种方式看起来一模一样。**要看一个开关有没有作用,先找一个能让它的作用可见的观测点。**

### 3. 复现要用**被报告的同一形态**的输入

我第一轮用"无换行的三段"测,`cat` 收到完整 83 字节,我差点得出"没丢字节所以没问题"。
**他的症状是"多行被一行行提交",所以必须测含换行的输入。**
用单行测多行的 bug,会看到"一切正常"。

### 4. 症状描述里的每个细节都是线索

他说了两件事:
- "一部分内容按行一行一行的提交" → 换行被当提交
- "排队执行,这使得我的指令破碎了" → 每次提交是独立消息

**两个细节指向同一处(逐段写入),但第二个说明还有"提交语义"这一层。**
只修"合并成一次 write"不够 —— 合并后一次 write 里含 20 个 `\n`,saih 仍会提交 20 次。
**必须连"一次粘贴 = 一条消息"一起解决。** 这点我写进了派单,作为里面的产品决定。


---

## 后续:Nimbus 拿源码逐条反驳,而且他对了

他没有照我派的做(那一开始就无法实现),而是先读 xterm.js 5.5.0 的 bundle:

| 我的说法 | 他实测 | 结论 |
|---|---|---|
| "加 `onPaste` handler" | `onPaste` 在整个包里 **0 次**,`ITerminalOptions` 没这个成员 | **这个版本没有这个 option,我的修法无法实现** |
| "粘贴被拆成 N 次 onData" | 真 handler 是 `handlePasteEvent → paste(text,…)`,里面 `prepareTextForTerminal` 后 `triggerDataEvent(wholeText, true)` | **一次 onData,整块** |
| "`-p` 无用,因为 saih 不发 2004" | `terminal.ts:212` 写着 `\x1b[?2004h` | **我的结论假** |

我独立复验了他三条:`onPaste` 在 `ui/node_modules/@xterm/xterm/lib/xterm.js` 里 **0 次**;
`triggerDataEvent(e,!0)` 在;saih 三处处理 2004。**三条全中。**

### 真缺陷(他定位的)

```
paste-buffer      -> pane 收到 aa\rbb\rcc\r        (3 次提交)
paste-buffer -p   -> pane 收到 ESC[200~aa\rbb\rcc\rESC[201~   (1 次粘贴)
```

**saih 的编辑器早就解析 bracketed 形式**:`packages/tui/src/components/editor.ts:721`
检测 `ESC[200~`、缓冲到 `ESC[201~`,`handlePaste` 把整块当**一条**消息插入,
超过 10 行的粘贴甚至变成单个 `[paste #1 +N lines]` 标记。

**两端能力都在,只是没人用它。** 原生粘贴之所以正常,是因为**终端在 tmux 看到之前
就包裹好了**;我们的路径用 `load-buffer` 注入,完全绕过了那一层。

### 他的修复

`paste-buffer -p`,但只对粘贴路径,由一个具名分类器决定:

```rust
pub fn is_multiline_paste(bytes: &[u8]) -> bool {
    let has_newline = bytes.iter().any(|&b| b == b'\r' || b == b'\n');
    let has_other   = bytes.iter().any(|&b| b != b'\r' && b != b'\n');
    has_newline && has_other
}
```

**为什么不给每次 write 都加 `-p`(那才是真正的一行改动)**:saih 的 `handlePaste`
会剥掉非打印字符(换行除外),所以包住 Ctrl+C 会**吞掉中断信号**;
它还会调 `cancelAutocomplete()`,会在每次按键时**关掉补全下拉**。
所以包裹必须有条件,而分类器测试钉住这一点。

### 他还报了一个先前就存在的缺陷,没有藏

tmux 的 `paste-buffer` 会**丢掉单独的 `0x03`**:只写 `\x03` 时 pane 什么也收不到,
而 `A\x03B` 会收到 A 然后 B。这是 tmux 在原始路径上的行为,早于他的提交。
他的测试断言"到达的字节是逐字未包裹的",并把 `0x03` 丢失写进测试注释,
**没有为了绿灯伪造一个通过**。需要单独立派单。

### 我自己跑了变异

把他的分类器换成 `let _ = bytes; false`(即恢复他修的那个缺陷):
`the_classifier_separates_a_paste_from_a_keystroke` 和
`a_multiline_paste_through_write_reaches_the_pane_as_one_bracketed_paste`
**双双 FAILED**。恢复后 3 passed,`git diff --stat` 空。

## 这一轮我学到的

1. **"官方默认用 X 而我们没用"不是证据** —— 它证明 tmux 作者偏好 X,
   不证明 X 在我们的链路上有效。
2. **要看一个改字节的开关有没有用,观测点必须让它的作用可见** —— `cat -v`,不是"文字出现了"。
3. **搜转义序列要搜转义形式** —— 源码写 `\x1b[?2004h`,不含字面量 `2004`;
   而 `grep 2004` 在含 Unicode 宽度表的 bundle 里会被淹掉。
   **在 dist/bundle 里搜信号,先限定 src 目录。**
4. **拿 A 的测量结果断言 B 的行为,是最隐蔽的一种编造** ——
   我对 `cat`(不声明 2004)测出"`-p` 无效",然后断言 saih(声明了 2004)也无效。
   测量本身没错,**推论越过了测量能支撑的范围**。
5. **同一天第二次空输出被当结论。** 两次都不是"没查",是"查了、返回空、我读了结论"。
