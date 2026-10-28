# 席位输出双渲染:tmux 面板保留 + SeatLoom UI 加一个顺滑的会话窗口

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-dual-render-seat-stream-v1 |
| target_seat | seat-nimbus-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 张先生的要求(原话)

「我要求 seat 有 tmux 面板,然后还有 SeatLoom UI 的顺滑的会话窗口,**两个都要有**。」

## 已确认的事实(Aegis 实测,不是推测)

1. **现在的链路本来就是字节级流式**:
   `tmux pipe-pane` → FIFO → 前端 `term.write(bytes)`
   (`crates/seatloom-core/src/pty/mod.rs` 头部注释 + `SessionTerminal.tsx:236`)
   **所以顺滑度已经有了,不需要造。**

2. **saih 是 `message_end` 才落盘**(源码实证:
   `agent-session.ts:1325` 注释说 agent-core 在 `message_end` 前存好消息对象,
   `SessionManager.appendMessage` 在那之后才 `appendFileSync`)
   **所以"从会话文件读"拿不到流式,只能拿到整轮。这条路的代价是丢掉顺滑。**

3. **saih 有 `--mode rpc`,事件是 `text_delta` 级**(Aegis 实测通了:
   一条 prompt 收到 `agent_start`/`turn_start`/`message_start`/
   `message_update`×4/`message_end`/`turn_end`/`agent_end`/`agent_settled`,
   其中 2 个 `text_delta`,内容 `"一\n二\n三"`)
   **但 RPC 模式不是交互式 —— 用它就没有 tmux 面板。**
   张先生明确要求保留 tmux 面板,所以**不走 RPC**。

## 结论:同一个字节流,两个消费者

```
              ┌─→ xterm 终端(保留,不动)
tmux pipe-pane ─┤
   (不动)      └─→ 新的"会话窗口"数据通道
                        └→ UI 自己切轮次、自己累积存
```

**UI 自己切轮次、自己存,不等 saih 落盘** —— 因为字节流本来就是流式的,
轮次边界可以从输出形态认出来(分隔线 / 提示符 / 一次回答的起止)。

## 要你做的

1. **把现在给前端的字节流,同时暴露成第二种形态**:不只是"一堆字节",
   而是带**轮次边界**的流。比如:一条消息开始、增量、结束。
   **不要重新实现一套采集** —— pipe-pane 那条路已经在工作,别为这个改它
2. **UI 侧要能拿到累积内容**(不然翻不了历史):后端或前端存,
   你定,但要说清存哪、什么时候清、占多少空间
3. **接口给前端**,让 Mira 能据此做界面

## 判据(可重跑)

- **顺滑度不低于现在**:同一个 prompt,新通道的首字节延迟和 xterm 的相当
  (对比现在 `term.write` 的到达时间,给出实测数字)
- **轮次能分开**:一条回答的起止能被认出来,给出实测输出
- **不破坏 xterm**:现有的 `SessionTerminal` 行为和测试**全绿**
  (`pnpm vitest run src/App.supervisorSync.test.tsx` 现在 20 passed;
   `pnpm tsc --noEmit` 干净;Rust `cargo test --workspace` 168 passed)
- **tmux 面板照旧**:席位还在 tmux 里跑交互式 saih,`attach` 上去一样能用

## 明确不做

- **不许把席位改成 RPC 模式启动** —— 张先生明确要求保留 tmux 面板
- **不许改 pipe-pane 的采集链路** —— 它已经在工作,为这个动它风险不值
- **不许改 saih 源码** —— 那是全队 harness
- **不许让 UI 依赖 saih 的会话文件作为流式源** —— `message_end` 才落盘,
  那样顺滑度就没了(这是已实测的事实,不是偏好)

## 已知的四条交互痛点(这一单不解决,但接口要为它们留路)

这些是同一个根因的不同面,由 Mira 那单在界面上解决:

1. alternate screen 不留 scrollback → 长回答往上翻不了
2. `window.open` 被 Tauri 拦 → 链接点不开(`SessionTerminal.tsx:116`
   的 `new WebLinksAddon()` 没传 handler,默认 handler 就是 `window.open`)
3. 终端里只看到 "Successfully wrote",看不到它写了哪个文件
4. 不能搜索

**你设计的接口要能让 Mira 解决这四条**,别把她堵死。
