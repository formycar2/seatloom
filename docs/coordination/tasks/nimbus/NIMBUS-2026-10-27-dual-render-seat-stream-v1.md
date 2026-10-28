# NIMBUS-2026-10-27-dual-render-seat-stream-v1 — 交付说明

派单 `disp-98QeI7oCe_posblRFa_nf`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-27-dual-render-seat-stream-v1.md`。
分支 `packet/nimbus/project-scoped-reads`,合并提交 `8e33ea7`(与 track 同步,24 个提交,
区间含 11 个 `ui/src` 文件,命中判据)。

## 做了什么:同一个字节流,两个消费者

任务包的方案图就是实现。**pipe-pane 那条路一个字节都没动** —— 改的是它下游:

```
              ┌─→ xterm 终端(`session:output`,原样不动)
tmux pipe-pane ─┤
   (不动)      └─→ TurnDetector → `session:turn`(新事件,带轮次边界)
```

四个文件:

| 文件 | 是什么 |
|---|---|
| `crates/seatloom-core/src/turn_stream.rs` | 轮次边界状态机(8 个测试) |
| `src-tauri/src/commands/session_cmds.rs` | 在 attach 循环里喂同一个 `rx`,`session:output` 之后追加发 `session:turn` |
| `ui/src/lib/types-dto.ts` + `api.ts` | `SessionTurnEvent` 类型 + `onSessionTurn()` |
| `ui/src/stores/useConversationStore.ts` | 前端累积器(11 个测试)—— Mira 做界面的地基 |

## 判据一:顺滑度不低于现在(实测)

新通道的首字节延迟 = `feed()` 的耗时。实测(**一次性 probe 打真实流,用完已删**):

```
--- one 4KB chunk ---
existing path (encode only):        84ns
with turn channel (encode + feed):  8.7µs
added latency per chunk:            8.6µs
added CPU at 100 chunks/sec:        0.086% of one core
added CPU at 1000 chunks/sec:       0.86% of one core
```

**8.6µs,不是毫秒级。** 席位每秒最多几十个 chunk,所以这是百分之一核以下。
用户感知不到,xterm 那条路的到达时间一个字节都没变。

**过程中我把它优化过一次,值得记**:第一版 `visible_text()` 给每行建一个 `String`,
实测占掉检测器 **419µs / 442µs = 95%** 的耗时。换成字节级扫描(`LineScan`,只看
 braille 块和 `─` 的 UTF-8 序列)之后,同样的答案,成本降到约五分之一。
这是诚实的修法,不是 `#[allow]` 掉那个分配。

## 判据二:轮次能分开(实测)

在**真实 saih 面板**里跑三个 prompt(`TURN_ALPHA` / `TURN_BETA` / `TURN_GAMMA`),
`tmux pipe-pane` 录下字节流喂给检测器:

```
TurnStarted: 3
TurnEnded:   3
events total: 18
turn #1 bytes=12288 markers_visible=["TURN_ALPHA"]
turn #2 bytes=12288 markers_visible=["TURN_ALPHA", "TURN_BETA"]
turn #3 bytes=8192  markers_visible=["TURN_ALPHA", "TURN_BETA", "TURN_GAMMA"]
```

**3 个 prompt → 3 个 TurnStarted + 3 个 TurnEnded,一个不多一个不少。**

### 但这里有一个必须说清的发现

第三轮里三个 marker 都在 —— **不是轮次没分开,是 tmux 每帧重绘整个面板**,
所以旧 prompt 的文字仍在字节流里。我一开始把这个误读成"分离失败",
查了才发现:轮次**边界**是对的(3/3),但对"某个 marker 属于哪一轮"这个问题,
字节流给不出答案,因为那一轮的红叉叉的是整个屏幕。

**这对 Mira 有直接影响**:她不能靠"这个字符串在哪一轮的字节里"来归属内容。
累积器因此只按边界切分、原样保留文字,由界面去解释。
我把这条写进了 `useConversationStore` 的注释,免得她重新踩一遍。

## 判据三:不破坏 xterm(全绿)

| 检查 | 结果 |
|---|---|
| `pnpm vitest run src/App.supervisorSync.test.tsx`(任务包点名) | **20 passed** —— 与任务包写的数字一致 |
| `cd ui && pnpm vitest run` | **25 files / 311 tests 全过** |
| `cd ui && pnpm tsc --noEmit` | 干净 |
| `cargo test --workspace` | **305 passed, 0 failed** |
| `cargo clippy --workspace --all-targets` | **0 warning** |
| `cargo fmt --all -- --check` | 干净 |

## 判据四:tmux 面板照旧

- 没有把任何席位改成 RPC 启动 —— 一个进程都没动。
- 没有改 pipe-pane 采集链路 —— `pty/mod.rs` 的 reader 一行没改。
- 没有改 saih 源码。
- 实测:9 个席位面板 `pane_pipe=1` 全部照旧,Aegis 的面板也是。

(过程中我误停过 Aegis 面板的 pipe,已按原样恢复并用转录文件增长验证过 —— 见「需要上报」。)

## 测试:两个方向都钉住,而且证明过会红

**Rust 8 个**(`turn_stream.rs`),含两个反例:
- 普通输出里的 braille 字符**不**开启幻影轮次(没这条,上面两条都没意义)
- 没有 spinner 的状态行**不**算轮次结束
- 未闭合的轮次保持开启,**不**被强行关闭(模块注释里承诺过)
- 跨 read 的标记仍能匹配
- chunk 原样返回(拼起来等于输入)

**UI 11 个**(`useConversationStore.test.ts`),含两个反例:
- 没有开启轮次的字节**被保留**,不丢(否则界面会靠遗漏来说谎)
- `turnEnded` 没有开启轮次时**不**凭空造一个
- 上限只在异常输出时生效;普通轮次永不被裁
- `whereIsItStored` 明说只在内存(防的是"以为有持久记录")

**两个变异实测**:把 `path: d.homePath ?? null` 那种手法用在这里 ——
我把"丢掉孤立字节"和"上限改成 10 字节"分别打回去,前者 1 个测试红、后者 3 个测试红。
(试过第三个变异"turnEnded 关闭所有轮次",测试没红 —— 查下来那个变异在当前语义下等价,
因为同时只能有一个轮次开启。**没有为了凑一个红测试而把代码改坏**。)

## 存哪、何时清、占多少空间(任务包要求说清)

- **存在哪**:内存,这个 app session。**不写库、不写盘。** 理由写在 `useConversationStore`
  的 `whereIsItStored` 常量旁边:写库等于给没人捕获过的东西造事件(和 session-work-bridge
  契约是同一条红线);写盘等于在 `pty/mod.rs` 已经在写的转录旁边再放一份可能不一致的副本。
- **何时清**:`reset()`(面板 detach / exit 时调),或 app 重启。
- **占多少**:单轮文字上限 256KB(超出从**头部**裁,`truncated` 由长度差可推),
  保留轮数上限 50。两个上限都只在异常输出时才真正生效,普通轮次一个字节都不裁。
- **后果,说清楚免得以后才发现**:关掉 app,会话窗口是空的。持久的记录是终端自己的历史
  和 saih 的会话文件。要持久的会话视图是另一个决定,有它自己的代价。

## 给 Mira 的路:四条痛点都没有被堵死

接口按"让内容可拿"设计,四条都能在它上面解:

1. **alternate screen 不留 scrollback** → 累积器自己存文字,不依赖终端回滚
2. **`window.open` 被 Tauri 拦** → URL 在文字里,可以从文字提
3. **只看到 "Successfully wrote" 看不到路径** → 路径在文字里,没被终端改写
4. **不能搜索** → 文字是可搜索的

**这四条我一个都没解** —— 任务包说它们在 Mira 那单。但我把第 3 条的来源查了:
`write.ts:86` 的 `Successfully wrote to ${path}`,路径**确实**在输出里,
只是被终端的行宽裁掉了。这对她有用。

## 明确不做的,确认没做

- 没把席位改 RPC 启动(9 个面板一个没动)
- 没改 pipe-pane 采集链路(`pty/mod.rs` 零改动)
- 没改 saih 源码
- 没让 UI 依赖 saih 会话文件当流式源(那是 `message_end` 才落盘的,顺滑度会没)
- 没为了让判据过而改判据

## 需要上报

1. **`~/networkorchestrator` 那一轮的检测是启发式,不是事实。** 它认的是"工作指示器出现 →
   被状态行取代"这个形状。saih 改画法,边界就会失效,而**字节流仍然是对的**。
   我把这条写进了模块头、事件类型注释和 store 注释三处,因为这是最容易被忘掉的一条:
   一个错的边界会让界面把两次回答并成一次,而终端里看着完全正常。
2. **我误停过 Aegis 面板的 pipe,已恢复。** 定位"哪个 FIFO 属于哪个面板"时,
   我对着 `Aegis-Supervisor` 执行了 `tmux pipe-pane`(不带参数 = 停止),
   把 SeatLoom app 正在用的那个管断了。恢复方式:找到 app 的 FIFO
   (`ses-18e24db955e0ad38.fifo`,靠转录文件里 `CH99.7%` 这个 Aegis 独有的状态行认出来),
   `tmux pipe-pane -t Aegis-Supervisor "cat >> <那个 FIFO>"` 挂回去,
   并验证转录文件重新增长(5 秒 +71939 字节)。**教训:`tmux pipe-pane` 不带参数是停止,
   不是查询** —— 查询要用 `display-message -p '#{pane_pipe}'`。我把它当成查询用了。
3. **分支落后 track 24 个提交,已合并**(`8e33ea7`)。区间命中判据(11 个 `ui/src`),
   而且带回了任务包点名的那 20 个 `App.supervisorSync` 测试 —— 我合并前在 worktree 里
   根本找不到那个文件,差点以为任务包写错了路径。

## 提交

`feat(stream): a turn-aware second view of the tmux mirror — same bytes, boundaries added`
(绝对哈希见 `git log --oneline -1`)。含 `turn_stream.rs`、`session_cmds.rs` 的事件、
UI 类型与 `onSessionTurn`、`useConversationStore` 及其测试、本说明。
