# 实时席位镜像交叉串线 —— 根因已定位到字节级(派 Nimbus 修)

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | fix |
| id | AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1 |
| status | issued |
| author | aegis |
| date | 2026-10-27 |
| version | v1 |
| to | nimbus |
| depends_on | NIMBUS-2026-10-27-dual-render-seat-stream-v1 |
| tags | tmux,pipe-pane,live-mirror,cross-wiring,verification |

> 日期:2026-10-27 18:00–18:55。症状由张先生报告:"我在实时席位里跟 Aegis 说的话,
> 显示在了 Nimbus 的实时席里面。" Aegis 一开始不相信(第 4 次犯了
> 「不要怀疑操作者」的错),后来逐条量到了字节级证据。
>
> **本文档只记录根因和修复要求,不改任何账本行、不跑迁移、不注册席位。**

## 1. 症状(操作者原话)

> 实时席位的 live view 是 showing [my messages]。用户发 MESSAGE。So they're seeing
> Aegis's content in Nimbus's slot.

两个 pane 本身没串:Nimbus 的 tty 是 ttys005、Aegis 的是 ttys001,cwd 也不同。
串的是 **UI 上那一格显示的内容**。

## 2. 根因(字节级证明,不是推测)

### 2.1 采集链路

```
tmux pane  ──pipe-pane──▶  /tmp/seatloom-mirror/<sessionId>.fifo
                                              │
              Rust reader 线程 tail-read ─────┤
                                              ▼
                          broadcast::channel  ──▶  session:output  ──▶  xterm
```

**tmux 每个 pane 只有一个 pipe。** `pipe-pane` 不是引用计数,是"这个 pane 的输出
复制到哪"。谁后写谁赢,前一个 `cat` 当场死掉。这正是 `PtySession::kill()` 文档里
写明的已知限制("the pipe is per-pane, not per-client")。

### 2.2 被串的那个 FIFO 里发生了什么

`/tmp/seatloom-mirror/ses-18e24db955e0ad38.fifo`(下称 **FIFO-X**),它的转录文件
`.seatloom/transcripts/ses-18e24db955e0ad38.raw.log` 现在 **180,986,013 字节**,
已经冻住(mtime 18:37:21)。

按字节偏移切开:

| 区间 | 字节数 | 里面是谁的输出 |
|---|---|---|
| `0 → 6,819,895` | 6.8 MB | **Nimbus 的 pane**(`seatloom-worktrees/nimbus` / `packet/nimbus/project-scoped-reads`) |
| `6,819,895 → 180,986,013` | 174.2 MB | **Aegis 的 pane**(`seatloom (track/infra-foundation)`) |

即 **FIFO-X 前 3.8% 是 Nimbus,后 96.2% 是 Aegis**。一次交接,没有重叠,没有交错。

两侧各自独有的指纹(只有对应 pane 才可能打印):

| 指纹 | 交接前 | 交接后 |
|---|---|---|
| `~/Documents/GitHub/seatloom (track/infra-foundation)`(Aegis 的提示行) | 0 | 254 |
| `seatloom-worktrees/nimbus (packet/`(Nimbus 的提示行) | 22 | 1327 |
| `AEGIS-HEARTBEAT` / `fleet-status.sh`(只有 Aegis 跑) | 0 | 227 / 32 |
| `turn_stream.rs`(只有 Nimbus 在改) | — | 1051(他的旧输出仍在文件里) |

### 2.3 谁干的 —— 在 Nimbus 自己的 pane 滚动里

`Nimbus-TechArchi-seatloom:0` 的 scrollback 第 3294 行,Nimbus 自己写的交付说明:

> 2. **我误停过 Aegis 面板的 pipe,已恢复。** 定位"哪个 FIFO 属于哪个面板"时,
>    我对着 `Aegis-Supervisor` 执行了 `tmux pipe-pane`(不带参数 = 停止),
>    把 app 正在用的管断了。已挂回 app 自己的 FIFO,并用转录文件增长验证
>    (5 秒 +71939 字节)。**教训:`tmux pipe-pane` 不带参数是停止,不是查询** ——
>    查询要用 `display-message -p '#{pane_pipe}'`。我把它当成查询用了。

同一份文档 `NIMBUS-2026-10-27-dual-render-seat-stream-v1.md` 第 93 行也承认:

> (过程中我误停过 Aegis 面板的 pipe,已按原样恢复并用转录文件增长验证过)

**他的"恢复"挂错了对象。** 他把 Aegis pane 的输出接到了 **FIFO-X**(那个
app 以为属于 Nimbus 的管道),而 app 的 reader 仍然只从 FIFO-X 读 ——
于是 Nimbus 那一格开始显示 Aegis 的全部输出,包括张先生跟 Aegis 说的每句话。

他的验证方式("转录文件重新增长")**测不出挂错**:文件当然会增长,因为那是他自己
刚接上的那一端。**增长 ≠ 接对了。** 这是一个看起来完全正常的检查,正是最坏的那种。

### 2.4 为什么现在自愈了

Aegis 为了定位,对自己的 pane 执行了一次 `tmux pipe-pane`(不带参数)来关管验证 ——
**同一个动作,又踩了一次**。关管后:

- `cat 32264`(写 FIFO-X 的那个)和它的父 `sh 32262` 一起消失;
- app 的 reader 命中 EOF,把 FIFO-X 从自己的 fd 表里丢掉(现在只剩 7 个 FIFO);
- `Aegis pane_pipe=0`、`Nimbus pane_pipe=0`,两边都没有输出复制。

**所以症状现在不可见,不是因为修好了,是因为两个 pipe 都断了。** Nimbus 那一格现在是
死的。这是"症状消失但病因还在"的典型形态,不能当成已修复汇报。

### 2.5 附带发现:48 个 FIFO 与 6.3 GB 转录

- `/tmp/seatloom-mirror/` 有 **48 个 FIFO**,其中 9 个有活着的 `cat` 在写;
  其余 39 个是孤儿(detach 时 fifo 被 unlink 过、或 attach 失败后没清干净)。
  实际上 detach 路径**有** `remove_file`(见 `pty/mod.rs` `kill()`),所以这 39 个
  更可能来自 **attach 成功但 reader 早退** 的路径 —— 需要 Nimbus 一并核。
- `.seatloom/transcripts/` **6.3 GB / 422 个文件**。单个 FIFO-X 的转录就 181 MB,
  一次交叉串线让它同时吞下了两个高流量 pane 三小时的输出。**没有任何上限。**

## 3. 为什么这件事值得单独修,而不是"重启一下就好"

`kill()` 的文档已经写明 per-pane pipe 是 tmux 的性质,不是本函数的 bug。但这次不是
两个 SeatLoom 客户端互相踩 —— **是另一个进程(一个 agent 在排障)单方面抢走了 pipe**。
产品对此零防御:任何会敲 `tmux pipe-pane` 的人或 agent,都能让实时席位显示任意 pane
的内容,而且 app 不会报错、不会变灰、不会说"这个流不是我以为的那条"。

**修复要求(按优先级,都可独立验收):**

### R1(必做)让每个 pane 的 pipe 只能被 app 自己拥有,并持续自证

- attach 之后,周期性(建议 ≤ 10s)重读一次 pane 的 pipe 状态与目标 FIFO 的一致性;
  发现不一致就把该 session 标成 `mirror_lost` 并让 UI 显示"镜像已断开,点此重接",
  **而不是继续把陌生 pane 的字节显示成这个席位**。
- 判据必须是"能报错"的:必须存在一条路径,在 pipe 被抢走后 UI **不会**继续显示内容。
- 不接受用日志代替:日志里写了而 UI 照常显示,等于没修。

### R2(必做)给 FIFO 与 pane 建立可验证的对应关系,并把它显示出来

现在 UI 上没有任何东西能回答"这一格的字节来自哪个 pane"。建议在 attach 时把
`#{pane_pid}` / `#{pane_tty}` 一起存进 `LiveSessionDto`,并在终端头部显示。
这样交叉串线在发生的那一刻就能被人眼发现,而不是三小时后靠读 181 MB 日志。

### R3(强烈建议)转录文件加上限与轮转

- 单个 transcript 设上限(建议 64 MB),到顶轮转而不是无限追加;
- attach 失败/reader 早退时清理 FIFO,把 48 → 应收敛到"活着的 session 数"。
- 判据:跑一轮 attach/detach 之后,`ls /tmp/seatloom-mirror | wc -l` 必须等于活着的
  session 数,**多一个就是没修**。

### R4(建议)把"`tmux pipe-pane` 不带参数是停止"写进工程规范

这次是 Nimbus 踩的,但任何人都会踩。`docs/coordination/ENGINEERING_STANDARDS.md`
加一条:**排障时查询 pipe 状态用 `display-message -p '#{pane_pipe}'`;
`pipe-pane` 不带参数是停止操作,永远不要用它来"看一下"**。

## 4. 已验证的事实清单(可复现)

| # | 事实 | 怎么验 |
|---|---|---|
| 1 | FIFO-X 里前后是两个不同 pane 的输出 | 按字节偏移 6,819,895 切开,分别 grep 两侧独有的提示行 |
| 2 | 写 FIFO-X 的 `cat` 已被 Aegis 关管时一起杀掉 | `ps -p 32264` / `ps -p 32262` 现在都是空 |
| 3 | app 已丢掉 FIFO-X 的 reader | `lsof -p 70338 \| grep FIFO` 从 8 条降到 7 条 |
| 4 | Nimbus 的 pane 现在没有 pipe | `tmux display-message -p -t Nimbus-TechArchi-seatloom:0 '#{pane_pipe}'` → `0` |
| 5 | Nimbus 自己承认误停并"恢复" | `Nimbus-TechArchi-seatloom:0` scrollback 第 3294 行;`NIMBUS-2026-10-27-dual-render-seat-stream-v1.md` 第 93 行 |
| 6 | 48 个 FIFO / 6.3 GB 转录 | `ls /tmp/seatloom-mirror \| wc -l`;`du -sh .seatloom/transcripts` |

## 5. 需要上报给张先生的

**修复后必须重启 SeatLoom 才生效** —— 席位(包括各 pane 里的 agent)的 MCP 子进程是
它启动那一刻的二进制镜像。这条已经被 `process_running_does_not_mean_it_runs_the_new_code.md`
记过一次,今天这是第二次。
