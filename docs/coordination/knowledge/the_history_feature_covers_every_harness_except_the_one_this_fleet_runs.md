---
title: "The history feature covers every harness except the one this fleet actually runs"
category: knowledge
topic: scope-gap
tags: [archive, harness, saih, blind-spot, self-coverage, history]
created: 2026-10-29T06:40:00+08:00
source_kind: canonical_event
source_ref: ev-archive-scanner-has-never-heard-of-saih
confidence: inferred
curation: authored
---

# 历史会话功能覆盖了每一个 harness,唯独覆盖不了这个舰队自己跑的那个

## 起因

张先生问「看历史上下文的问题有解法了吗」。指的是他在应用里翻 QL,
看到的内容和 tmux 里完全不同那件事。

Nimbus 已经修好了诚实性缺陷(`b6ef3aa`:滚动条现在报自己的真实深度,
并标注这是回填不是录制)。但那只解决"别再假装有深度",没解决
"那六天对话到底去哪看"。我继续往下追,追到了底。

## 量到的

### 归档表的 harness 分布

```
claude-code  18
codex       210
cursor       14
gemini        9
opencode      1
stepcode     20
             ---
saih          0     <- 这个舰队每一个席位跑的都是它
```

### 扫描器的源目录

```
~/Documents/GitHub/AgentHistory/raw/
├── claude-code/
├── codex/
├── cursor/
├── gemini/
└── stepcode/
```

`saih` 不在里面。

### 而舰队自己的对话在哪

```
~/.saih/agent/sessions/
229 个 jsonl,355 MB
```

**一条都没进 SeatLoom。**

## 后果(比 QL 一件事大得多)

「历史会话」这个功能能读 codex、claude-code、cursor、gemini、stepcode 的历史,
**对自己舰队的对话完全瞎。**

所以张先生在应用里点开 QL 往上翻时,他看的不是"录下来的会话",
是 pipe 的实时流。SeatLoom 里**没有任何东西**能告诉他 QL 这六天说过什么。

QL 的具体对照最能说明问题:

| | |
|---|---|
| 他真正在聊的对话 | 202 MB / 7237 条,10-23 10:21 → 10-29 03:26 |
| 库里 QL 的归档 | 4 条,**全是 `harness=codex`**,三月和七月的**别的对话** |

也就是说:点开 QL 的"历史",看到的是这个席位几个月前在另一个工具里的对话,
和它作为 SeatLoom 席位的全部工作无关。

## 一个反直觉的事实:大文件不是障碍

归档表里已经有:

```
max(bytes)  = 413 MB
max(lines)  = 148 387
```

QL 那个 202 MB / 7237 行,比表现有数据小。
**所以"文件太大所以没归档"不成立 —— 是 harness 名字不被认,不是容量不够。**

## 规矩

### 1. 一个数据管道的覆盖范围,要拿"我们自己产生的数据"去撞

`archived_sessions` 有 272 行、14 GB,看起来丰富。数一下 harness 分布才发现
**没有一行是这个系统自己的产物**。

**统计"有多少数据"是空的;统计"哪些来源的哪些数据"才是有信息的。**
一个覆盖了五个外部 harness、漏掉唯一一个内部 harness 的管道,
比一个空管道更难发现 —— 因为它看起来在工作。

### 2. "功能在"不等于"功能覆盖得到我"

「历史会话」在、能读、有数据、有界面。而它对舰队自身是瞎的。
判断一个功能是否真的服务用户,要问:**用户最重要的那一类数据,它在吗?**

对 SeatLoom 来说,舰队自己的对话就是最重要的那一类 —— 那是这个产品存在的理由。

### 3. 追到底,别停在第一层修复

Nimbus 修的是诚实性(滚动条别假装有深度)。那是对的、必要的,
但张先生问的是"怎么看那六天对话" —— **停在第一层修复,会把" UI 不撒谎"
当成"问题解决了"**。

两个缺陷是分开的:
- **诚实性**:修好了(`b6ef3aa`)
- **覆盖范围**:saih 从未被归档,已派 Vela 定夺该不该进

## 顺带独立复现了 Nimbus 的三条 tmux 主张

他说加大 tmux 回滚缓冲必须重建 pane,我逐条验了:

| 主张 | 实测 |
|---|---|
| `set-option -p` 对活 pane 被接受但无效 | ✅ 设完仍 2000 |
| 新 pane 继承全局 50000 | ✅ 新建 pane = 50000 |
| 全局确实是 50000 | ✅ |

**所以历史深度不是代码缺口,是 tmux 的属性** —— option 在 pane 创建时读取。
应用能做的是不广告自己没有的深度,它现在这么做了。
