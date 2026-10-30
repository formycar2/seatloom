---
title: "I nearly reported bracketed paste as the fix; measuring showed it changes nothing"
category: knowledge
topic: debugging-method
tags: [paste, xterm, onData, tmux, paste-buffer, false-cause, measure-the-symptom]
created: 2026-10-30T12:30:00+08:00
source_kind: canonical_event
source_ref: ev-seat-paste-fragments-multi-line-input
confidence: inferred
curation: authored
---

# 我差点把 bracketed paste 当成修法,实测证明它毫无作用

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
tmux 才包。`cat` 没声明,saih 也没声明(grep 整个 saih 源码树,2004 零命中)。

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
