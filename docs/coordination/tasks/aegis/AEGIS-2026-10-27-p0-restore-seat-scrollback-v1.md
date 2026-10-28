# P0:实时席位翻页 —— terminal.reset() 把 3000 行历史扔了,张先生要求最优先修

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-p0-restore-seat-scrollback-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |
| 优先级 | **P0** —— 张先生明确要求最优先 |

## 现象(张先生 2026-10-27)

「除了 ql 这个 seat 实时席位可以上下翻,**其他所有 seat 都不能上下翻**
(在 SeatLoom UI 中)」

## Aegis 定位到的真因(有源码行号)

`ui/src/app-v2/panel/SessionTerminal.tsx:202-203`:

```js
if (!clearedStaleSnapshotRef.current) {
  clearedStaleSnapshotRef.current = true;
  // ... that layering is what made pi's input box read as garbled.
  terminal.clear();
  terminal.reset();      // ← attach 时写进去的 3000 行历史,在这里被扔
}
```

**它在第一次 fit 成功后就清掉 scrollback。**

我读了 xterm 5.5 的产物和 d.ts,确认**两个调用都清历史**:

- `clear()`: d.ts 原文 "Clear the entire buffer"
- `reset()`: d.ts 原文 "Perform a full reset (RIS, aka `\x1bc`)";
  产物里 `BufferService.reset()` 是 `this._normal=new Buffer(...)` —— **直接新建 Buffer 对象**

**所以没有"换成 clear 就保住历史"这条路,两个都清。**

## 为什么当时必须清(你自己的交付里写了)

`MIRA-2026-10-08-um0-mirror-fidelity-delivery-v1.md` 第 3 条:

> This was chosen over re-feeding the snapshot because the snapshot is captured
> at the pane's launch geometry and is **stale by construction** after any resize.

**fit → 推新尺寸给 tmux → SIGWINCH → saih 用新尺寸从头重绘。**
旧快照是旧尺寸的绝对定位帧,叠在新重绘上,输入框错位花掉。
清掉是当时选出的修法。

**所以这是两个真 bug 撞在一起:一个修法是另一个的成因。**

## 实测对照(2026-10-27 16:40,tmux 侧完全一致)

| 会话 | tmux 可取 | UI 能滚 |
|---|---|---|
| Nimbus-TechArchi-seatloom | **3301 行** | ❌ |
| Mira-UX/UED-seatloom | **3104 行** | ❌ |
| Flux-Quality&Ops-seatloom | **3038 行** | ❌ |
| Onyx-data-seatloom | **3000 行** | ❌ |
| QL-seatloom | 1871 行 | ✅(只因张先生上一秒正在看它) |
| Stepaicloud-seatloom | 66 行 | ❌ |

**所有会话 `pane_pipe=1`、`session_attached=0`、`alternate_on=1`,完全一致。
差别全在 UI 侧。**

**QL 能翻不是它特殊,是它是当前活跃的那个** —— 切走就停,切回来接不上。

## 我要你做的

**保住 scrollback,同时不让输入框错位。** 方向由你定,Aegis 只提供已查到的:

1. **`scrollToBottom()` 存在**(d.ts:1195,"Scrolls the display of the terminal to the bottom")——
   不 reset 也能把视口推回底部。清历史的必要性可能不在"回底部"这一步
2. **`BufferService.reset()` 是新建 Buffer** —— 所以任何走 reset 的路都保不住
3. **另一个原始选项是"重新喂快照"**,你当时否了它(理由:快照按旧尺寸 captured,
   resize 后 stale by construction)。**但那个理由今天可能已经不全成立** ——
   因为现在 backfill 取的是 `capture-pane -e -p -S -5000`,那是**当前尺寸**的历史
4. **最小改动思路**(仅供参考,不是指令):只清视口内那一屏、不清 scrollback
   (比如只 `clearViewport()` 如果有,或手动写 `\x1b[2J\x1b[H`),
   然后 `scrollToBottom()`。但**会不会错位要你实测**

## 判据(可重跑,你要给实际输出)

1. **attach 一个历史很长的 seat**(Nimbus 3301 行),**不 reset 的那一版**
   在 UI 里能往上滚过 1000 行
2. **输入框不错位、不花** —— 这是原来那个 bug,不能回来
   (你自己写过的判据,你自己知道怎么测)
3. **切到别的 seat 再切回来**,历史还在(不是重新攒)
4. 现有测试全绿:`pnpm vitest run src/App.supervisorSync.test.tsx`(20 passed)、
   `pnpm tsc --noEmit`、`pnpm test`(300 passed)
5. **真机验**:build + 装 + 在装好的 app 里滚一个历史长的 seat

## 明确不做

- **不许把 `reset()` 换成 `clear()` 然后说修好了** —— 两个都清(Aegis 已读源码确认)
- **不许为了让历史回来就把 fit/几何同步整个关掉** —— 那会退回 UM0 之前的 bug
- **不许改后端** —— 这是纯 UI 的事,`pty/mod.rs` 那条链没问题(tmux 侧 3301 行都在)
- **不许改 saih**

## 关联

沙盒 MVP 已验证:交互式 saih + extension 能同时吐结构化事件
(33 条事件,含 11 个 `text_delta`)。那是"会话窗口"的方案,
**这一单不等它** —— 张先生要最优先解决 scroll,先把 xterm 这条腿修好。
