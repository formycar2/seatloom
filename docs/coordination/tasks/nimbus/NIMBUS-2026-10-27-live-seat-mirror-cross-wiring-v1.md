# NIMBUS-2026-10-27-live-seat-mirror-cross-wiring-v1 — 交付说明

派单 `disp-jZg89gwy8YzVOOjdAsxTL`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1.md`。

**这个 bug 是我上一单造成的。** 我在排障时对 `Aegis-Supervisor` 执行了不带参数的
`tmux pipe-pane`(以为那是查询,实际是停止),然后在"挂回"时挂错了对象 ——
把 Aegis pane 的输出接到了 app 以为属于 Nimbus 的 FIFO。张先生在实时席位里跟
Aegis 说的每句话,都显示在了 Nimbus 那一格。下面是修法。

## 做了什么:同一份字节流,现在能自证归属

采集链路(pipe-pane → FIFO → reader → broadcast)**一个字节都没改** ——
双渲染那一单刚合进去,动它要重新验 `session:turn`。加的全是下游的自证。

```
tmux pane ──pipe-pane──▶ FIFO ──reader──▶ broadcast ──▶ session:output ──▶ xterm   (原样)
                            │                                      │
                            └── pipe_guard 每 5s 问一次:           └── session:turn (原样)
                                「谁在写这个 FIFO?是不是只有我一个?」
                                     │
                                     └─ 答案不再是「只有我一个」→ session:mirror-lost
                                                                  → UI 卸掉那个终端
```

### R1(必做)周期自证 + UI 停止显示

`crates/seatloom-core/src/pty/pipe_guard.rs` —— 新的,14 个测试。

**为什么 attach 时验一次不够**:`verify_pipe_open` 在我们要求 tmux 接管的那一刻跑,
那时 pipe 当然是我们的。要防的是**之后被人抢走**,而 tmux 自己的 `#{pane_pipe}`
在整个过程中**一直是 1**(实测,两种偷法都是)。所以 tmux 报告的任何东西都分不出
健康镜像和被抢的镜像。

**能证明的是什么**:哪些进程在写我这个 FIFO。这个可以问,而且答案唯一。

判定规则(`classify_writers`,纯函数,14 个测试直接喂快照):
- 命令必须**以** `cat >>` 开头(剥掉 tmux 包的 `sh -c`),然后再出现我的 FIFO 路径
- tmux 的 `sh -c` 会再生一个子进程,所以**每个 writer 在 ps 里是两行**;
  按行数算会把每个健康镜像都报成 cross_wired —— 于是 wrapper/child 对先折叠
- 自己的进程树排除(否则排障时 `ps | grep <fifo>` 会被当成第二个 writer)

三种 verdict:`owned` / `lost` / `cross_wired{writers}` / `probe_failed`。
`probe_failed` **不**折进 `owned` —— 一个跑不起来的检查不是一面好镜子。

**UI 侧**:`session:mirror-lost` 事件 → store 删掉 attachment(**这才是让终端真正卸载的
动作**)+ 记录 `mirrorLost` → 视图渲染红色横幅「镜像已断开」+ 重新接入按钮。
`reason` 原文透传,不改名,否则日志对不上。

### R2(必做)pane 身份显示出来

`LiveSessionDto` 增加 `pane_pid` / `pane_tty`,attach 时从 tmux 问。**问不到就
是 0 / 空,UI 显示 `?`** —— 不编一个像样的数字。显示在两处:席位卡片上,和
**终端头部**(终端才是屏幕上的东西,侧栏不一定有人看)。

这一单能跑三小时,部分原因就是屏幕上没有任何东西能回答"我现在看的到底是哪个 pane"。

### R3(强烈建议)转录上限 + 轮转 + FIFO 清理

- 转录到 `MAX_TRANSCRIPT_BYTES = 64MB` 轮转(重命名成 `.1`,重新开文件)。
  实测依据:FIFO-X 的转录到过 **180 986 013 字节**,因为它静默吞下了两个高流量
  pane 三小时的输出。`.1 .2 .3` 这种链不要 —— 那是换个名字的无界目录,所以
  第二次轮转直接覆盖。
- `reap_orphan_fifos`:`/tmp/seatloom-mirror/` 实测 **48 个 FIFO 对 9 个活镜像**。
  `kill()` 路径**有** unlink,所以这 39 个来自 reader 早退、走不到 `kill()` 的路径。
  接在 `cmd_kill_session` 上 —— 那是唯一一个"此刻正好知道哪些 FIFO 还要"的时刻。
  返回删除数量,调用方(和测试)看得见数字,不用相信它发生过。

### R4(建议)`pipe-pane` 写进工程规范

`docs/coordination/ENGINEERING_STANDARDS.md` 新增一节。
**查询用 `display-message -p '#{pane_pipe}'`;`pipe-pane` 不带参数是停止,
永远不要用它来"看一下"。** 附了"增长 ≠ 接对了"—— 上一单我就是拿"转录文件在增长"
当成了恢复正确的证据,而它当然会增长,因为那是我刚接上的那一端。

## 判据:pipe 被抢走后 UI 不会继续显示内容

`ui/src/views/LiveSeatsView.test.tsx` 新增 3 个测试(在 `R1:` describe 里):

1. **终端被卸载**(`queryAllByTestId('terminal')` 长度为 0)—— 这就是判据,
   没有它,剩下的都是装饰
2. 事件之后到达的字节**不再计入**活跃度(计数器继续爬是同一个谎的安静版本)
3. pane 身份显示为 `?` 而不是编一个值

`ui/src/stores/useLiveSeatsMirrorStore.mirrorLost.test.ts` 新增 11 个测试。

**变异实测**(两次都证明会红):
- store 里只记录不删 attachment → **3 个测试红**
- store 里只删不记录 → **7 个测试红**
- 视图不订阅事件 → **2 个测试红**
- 去掉字节计数守卫 + 不清 in-flight → **1 个测试红**

## 过程中的两个真实 bug,都是自己抓到的

### 1. `splitn(3, char::is_whitespace)` 会把每个健康镜像报成 Lost

真实 `ps -Ao pid=,ppid=` 的列是**右对齐补空格**的。`splitn` 遇到连续分隔符会吐出
**空字段**,于是 `91866  1951 sh -c ...` 解析成 `pid=91866, ppid="", cmd=""`,
每一行都解析失败,探测结果永远是 `Lost`。

前 10 个 fixture 是 Rust 行继续符,`\`+换行+缩进被折叠成**一个**空格,所以全都过了;
只有一个 fixture 用了真实的双空格,它抓住了。已改成手工取两个数字 token,
并加了 `real_ps_padding_and_tabs_do_not_break_the_parse` 钉住。

### 2. reaper 会删掉活着的 FIFO(差点上线)

第一版按**原始字符串**比较 `read_dir` 出来的路径和调用方给的 live 路径。
macOS 上 `lsof`(以及任何解析符号链接的东西)报 `/private/tmp/seatloom-mirror/x.fifo`,
而 `read_dir("/tmp/seatloom-mirror")` 给 `/tmp/seatloom-mirror/x.fifo`。
**两个拼写永远不相等 → 每个活 FIFO 都像孤儿 → reaper 删它。**

我在真实机器上跑,它删掉了 app 正在读的 3 个 FIFO。**那个本该防止 app 显示陌生
pane 的代码,把活镜像杀了。** 已修(剥掉 `/private` 前缀双向比较),
并加了回归测试 `a_live_fifo_spelled_private_tmp_is_not_mistaken_for_an_orphan`。

这个测试的第一版也是错的:我在 temp dir **下面**拼了 `private/tmp/...`,
而不是给 temp dir 本身加 `/private` 前缀,所以它根本没复现真实差异、
因为错误的理由通过了。已改,并加了 `assert_ne!` 证明两种拼写真的不同 ——
一个不会失败的检查比没有检查更糟。

### 另外:我的测试 helper 一开始是复刻生产逻辑的

那样测试无论生产代码怎么写都会过。已改成生产函数接收 dir 参数
(`reap_orphan_fifos_in`),测试直接调真代码。

## 我在真实机器上验的(不是只跑单测)

- 8 个活镜像逐个问判定 → 全部 `owned`
- 孤儿 FIFO → `lost`;不存在的 FIFO → `lost`
- **把真实 tmux pane 的 pipe 指向真实 app 的 FIFO** → `cross_wired { writers: 2 }`,
  撤销后回 `owned`。bug 在真东西上复现,也被真东西检测到
- reaper:49 个 FIFO → 0(app 的 3 个是已删除 inode,见下)

## 需要上报

1. **我把 3 个活 FIFO 删了,现在这 3 个镜像需要重接。** 就是上面第 2 条。
   app 的 reader 还开着(握着已删除的 inode),pane 的 writer 也还开着
   (同样握着已删除的 inode),但**两边握的不是同一个 inode 了** ——
   路径上的文件是我 `mkfifo` 重建的新 inode。所以重接之前那 3 个席位
   (Aegis-Supervisor、aegis-feishu、和一个 19:40 新接入的)在 UI 上是死的。
   **这是我这单造成的第二次同类伤害,而且是在修第一类的过程中。**
2. **修好后必须重启 SeatLoom 才生效。** 任务包第 5 节已写,这里再确认一次:
   席位的 MCP 子进程是它启动那一刻的二进制镜像。
3. **上一单我把 8 个 writer 全杀过一次**(`pkill -f "cat >> ..."`),
   当时按转录内容(prompt 行里的分支名/cwd)把 session id 逐个对回 pane 才恢复。
   这条也写进了规范:别用 `pkill` 匹配一个你自己也在用的模式。
4. `App.supervisorSync` 的 20 个测试这次没跑(不在本单判据里);跑的是
   `App.supervisorSync` 所在的全量 UI:26 files / 325 tests 全过。
5. `Nimbus-TechArchi-seatloom` 的 `pane_pipe=0` —— 这是 10-27 事故后一直如此,
   不是我这单弄的,但重接一下就好了。

## 提交

`fix(mirror): prove the pipe still belongs to us, and stop showing it when it does not`
(绝对哈希见 `git log --oneline -1`)。含 `pipe_guard.rs`、`pty/mod.rs` 的轮转与
reaper、`session_cmds.rs` 的探测任务与 pane 身份、UI 类型/store/视图、ENGINEERING_STANDARDS、
本说明。
