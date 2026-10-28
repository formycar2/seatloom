# 修:实时席位页面不能上下滚屏(一定几率出现)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-liveseats-scroll-unreliable-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |
| 严重度 | 中高 —— 张先生实测「一定几率」无法翻屏 |
| 来源 | 张先生报的 bug |

## 已核到的结构(你不用重新找)

渲染链:`LiveSeatsView.tsx:600` 的 `<div style={{flex:1, position:'relative', minHeight:0}}>`
→ 每个接入的席位一层 `position:'absolute'; inset:0`
(用 `display: a.seat.sessionName===activeName ? 'block':'none'` 切换显隐)
→ `<SessionTerminal>`(xterm.js,`scrollback: 10000`,`FitAddon`)。

QL 是**非标准席位**(不在 `seatloom-worktrees/` 下,工作区是
`project_delivery/QL`),所以它这条链路最可能暴露边界情况。

## 「一定几率」说明是竞态,不是 CSS 写错

Aegis 的可疑点(按可能性排,你逐条核,**核不掉的要如实说核不掉**,别硬编原因):

1. **切换席位时 display:none→block 的瞬间 fit()**:容器从 `display:none`
   变成 `block` 时宽度高度是 0,xterm 的 FitAddon 可能算出 rows=0,
   viewport 高度塌掉 → 看着在、滚不动。**这是最可能的。**
2. **xterm viewport 未获焦**:终端区没拿到焦点时,滚轮事件被外层 window 吃掉。
3. **多个 absolute 层叠在一起**:未激活的那些虽然 `display:none`,
   但 xterm 实例仍在(切来切去时旧实例的 scrollback 状态可能干扰)。

## 判据(可重跑,你要给出实际输出)

1. **反复切换席位 ≥10 次**,每次切到 QL 后立刻用滚轮/触控板上下翻,
   记录「翻不动」出现几次。目标:**0 次**。
2. 切到 QL 后,输出 xterm 的 `term.rows` / `term.cols` / viewport 高度,
   确认都不是 0(把实际数字写进交付说明)。
3. 滚轮在终端区里能翻 scrollback;**在页面其它区域**滚轮仍归页面(别把全局滚动也劫走)。
4. `pnpm test` 与 `pnpm typecheck` 仍全绿。

## 明确不做

- 不改 SeatLoom 只观察不拥有的定位(不给终端加输入框之类)
- 不加新的快捷键
- 不用「定时 refresh()」这种糊法(那会掩盖真因,下次还犯)
