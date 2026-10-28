---
id: reset_clears_scrollback_to_fix_one_bug_and_causes_another
source_kind: canonical_event
source_ref: AEGIS-2026-10-27-liveseats-scroll-and-ime-shift-v1
session_scope: seatloom
title: terminal.reset() 为了修输入框错位而清掉 scrollback,于是每个 seat 的历史都只剩切过去之后的
domain: failure
modality: failure
status: active
curation: authored
confidence: inferred
owner: aegis
created_by: aegis
created_at: 2026-10-27
evidence_refs:
  - ui/src/app-v2/panel/SessionTerminal.tsx:202-203 (clear + reset)
  - ui/src/views/LiveSeatsView.tsx:605-640 (所有 Terminal 同时挂载,visibility 隐藏非活跃)
---

## 张先生的观察(2026-10-27)

「除了 ql 这个 seat 实时席位可以上下翻,其他所有 seat 都不能上下翻(在 SeatLoom UI 中)」

## 我一开始的判断错了两回

第一回:我说是 alternate screen 擦掉历史。**错** —— QL 也在 alternate screen,
照样有 1871 行。
第二回:我说是内容少一屏装得下。**错** —— 那一轮折行 109 行 > 66 行。

**两回都错在我没有直接测"那一轮的开头在不在 pane 里"。**

## 真因(代码自己写了为什么)

`SessionTerminal.tsx:202-203`:

```js
if (!clearedStaleSnapshotRef.current) {
  clearedStaleSnapshotRef.current = true;
  // The snapshot was painted at the pane's launch geometry. Once the
  // pane follows the viewport the TUI repaints from scratch (SIGWINCH),
  // so drop the stale absolutely-positioned frame instead of leaving it
  // underneath the repaint — that layering is what made pi's input box
  // read as garbled.
  terminal.clear();
  terminal.reset();      // ← scrollback 在这里被清空
}
```

**`terminal.reset()` 清 scrollback。** attach 时写进去的 3000 行历史,
在第一次 fit 成功后就被扔掉。之后只剩 tmux 重绘出来的那一屏。

## 为什么当时必须这么做

fit 成功 → UI 把新尺寸推给 tmux → tmux 发 SIGWINCH → saih 用新尺寸**从头重绘**。
旧快照是按旧尺寸画的绝对定位帧,叠在新重绘上会让输入框错位、花掉。
**清掉是修那个 bug 的唯一办法。**

## 于是每个 seat 的行为

| | 表现 |
|---|---|
| 切到某个 seat | 它的 xterm 活着,持续收新字节,scrollback 从 0 开始攒 |
| 切走 | 停止积累 |
| 切回来 | 接不上之前的,又是从"重绘那一屏"开始 |

**QL 能翻不是它特殊,是张先生上一秒正在看它。** 我实测过:它的 1871 行
是切换之后新攒的,不是 attach 时那 1805 行。

## 实测数据(2026-10-27 16:40)

| 会话 | tmux 可取行数 | UI 里能滚的 |
|---|---|---|
| Nimbus | 3301 | 只有切过去之后的 |
| Mira | 3104 | 同上 |
| Flux | 3038 | 同上 |
| QL | 1871 | 同上(但它刚被看过,所以现在有) |
| Stepaicloud | 66 | 同上 |

**tmux 侧全都 pipe=1、attached=0、alternate_on=1,完全一致。差别全在 UI 侧。**

## 两个修法冲突,要说清而不是挑一个

| 目标 | 做法 | 代价 |
|---|---|---|
| 输入框不错位 | fit 后 `reset()` | **丢 scrollback** |
| 保住历史 | 不 reset | 旧尺寸帧叠在新重绘上,输入框花 |

**要同时做到,得让 saih 重绘之后把历史补回来** —— 那不是 xterm 能单独解决的。
这正是 extension 方案(沙盒 MVP 已验证)能给的:结构化事件自己存历史,
不依赖 xterm 的 scrollback。

## 教训

**一个 bug 的修法可能是另一个 bug 的成因,而两者在代码里相隔很远。**
Mira 今天修"页面不滚"时看到了 fit no-op 那层,
没看到 `reset()` 这层 —— 因为注释把 reset 的理由写得很充分(它确实是必要的),
读的人会认为"这里没问题"。

**判断这类问题要看"这个修复扔掉了什么",不只是"这个修复解决了什么"。**
