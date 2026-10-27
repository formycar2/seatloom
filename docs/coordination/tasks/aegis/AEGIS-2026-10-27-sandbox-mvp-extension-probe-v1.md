# 沙盒 MVP 验证结论:交互式 saih + extension = tmux 面板和结构化会话窗口可以并存

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-sandbox-mvp-extension-probe-v1 |
| target_seat | (记录,Aegis 自己验的) |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 为什么做这个沙盒

张先生要求:「找个沙盒 MVP 验证关键技术点和用户体验,**不要影响现在的 SeatLoom**」。
上一个方案(RPC 模式)被否,因为它会干掉 tmux 面板。

## 沙盒怎么隔离的

- tmux 会话名 **`sandbox-mvp`** —— **不以 `-seatloom` 结尾**,
  后端 `list_tmux_sessions` 只认那个后缀,所以 **SeatLoom UI 枚举不到它**
- 工作目录 `/tmp/seatloom-sandbox-mvp/`
- 只往 `/tmp` 写文件,**没碰账本、没碰任何席位、没碰 `/Applications`**

## 验的东西

一个 1682 字节的 extension(`/tmp/seatloom-sandbox-mvp/probe.ts`),用
`pi.on(...)` 订阅事件,把每条 append 进 `/tmp/.../events.jsonl`。
启动方式:**交互式 saih**(不是 RPC)+ `--extension` + `--no-session` + `--approve`。

## 实测结果:33 条事件,一条不漏

发一句「用 bash 跑一下 echo hello,然后告诉我说了几个字」,收到:

| 事件 | 条数 | 说明 |
|---|---|---|
| `turn_start` / `turn_end` | 2 / 2 | 一轮的边界 |
| `message_start` / `message_end` | 5 / 5 | 含 system / user / assistant 三种角色 |
| `text_delta` | **11** | **逐字**:`'跑'` `'完了，输出'` `'是：\n\n```\n'` ... |
| `assistant_event` | 6 | `toolcall_start` / `toolcall_delta`×2 / `toolcall_end` |
| `tool_execution_start` / `end` | 1 / 1 | 带 `toolName: "bash"` |

`text_delta` 拼起来正是最终回答:`跑完了，输出是：hello 一共 **5 个字**`。

**三条关键事实被证实:**

1. **tmux 面板还在**(pane 里跑的是 `node`,是交互式进程)
2. **结构化事件照样出来** —— `_emitExtensionEvent` 在交互式路径上就跑
   (`agent-session.ts:1367` 处理 `message_update`,交互式和 RPC 共用这条线)
3. **零侵入** —— 不改 saih 核心代码,只加一个 extension 文件

## 🔴 一个限制,必须如实记:`ctx.ui` 的 dialog 方法在交互式模式不外发

`extension_ui_request` 这个记录类型**只在 `src/modes/rpc/rpc-types.ts` 里定义**。
`docs/rpc-extension-ui.md` 通篇讲的是 RPC 模式。

交互式(tui)模式下,`ctx.ui.confirm()` 是**直接画在终端上的**
(`src/modes/interactive/interactive-mode.ts:2636` 的 `showExtensionConfirm`)。
所以 **extension 在交互式模式里想弹一个"要不要执行"的框,那个框出现在 tmux pane 里,
不会变成给 UI 的事件。**

`ToolExecutionStartEvent`(`types.ts:1094`)只带 `toolCallId`/`toolName`/`args`,
**没有"取消"这个返回值** —— 也就是说 extension 能**看到**工具要执行,
不能**拦下**它。

**这对"全功能"意味着什么**:批准按钮做不到"拦在执行前"。能做到的是
"执行前后都告诉你,并且执行结果可回看"。**要不要为此付 RPC 的代价(丢 tmux 面板),
是张先生的产品决定,不在技术方案里偷偷替他选。**

## 张先生已定的四个 UX 决策(2026-10-27)

1. **会话窗口形态**:靠拢 cursor / cowork / opencode desktop ——
   工具调用作为消息下的可折叠分区(不是纯气泡,也不是调试器式全展开)
2. **thinking_delta**:**折叠,默认收起**。他的理由:大部分情况没人看,
   只在 debug"为什么做得好/差"时看,查幻觉;而且**事后其他 agent/model 也会查**,
   **大概率是给机器看的**。所以默认收起、需要时展开,不给默认展开添吵
3. **实时席位(xterm)**:**留着,两个都在**。交互上**分开切换,不耦合在一个屏幕里**
4. **做到哪一步**:**全功能**

## 由此派生的技术边界(给 Nimbus 的任务书)

- 输入通道**现成**:`SessionTerminal.tsx:218` 的 `term.onData` → `ptyWriteBytes`
  (注释明说 round-trip every byte,sa行 看到的和直接敲键盘一样)
  → **会话窗口要能打字,复用这条,别新造**
- 工具调用的"可折叠分区"数据来自 `toolcall_start/delta/end` +
  `tool_execution_start/update/end`(沙盒实测有)
- thinking 折叠:数据是 `assistantMessageEvent.type === "thinking_delta"`,
  沙盒里没触发(那次回答没开思考),**实现时要单独验一次**
- **批准按钮**:受上面那个限制,先做成"事前告知 + 事后可查",
  不假装能拦。要真能拦,得换 RPC 模式,那是另一个决定
