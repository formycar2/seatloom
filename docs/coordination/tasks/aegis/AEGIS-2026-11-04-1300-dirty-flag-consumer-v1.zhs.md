# Aegis 13:00 心跳（11-04）：补上脏标志的最后一环，之前它有 0 个消费者

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-04-1300-dirty-flag-consumer |
| status | delivered |
| author | aegis |
| date | 2026-11-04 |
| version | v1 |
| tags | heartbeat, fix, third-layer, mutation |

## 检查

全文读取心跳协议。track/infra-foundation HEAD 6520db69ebba9128460e8d382a9f28ae65f0a7f1。账本检查先跑，exit 0，35 表 4 事件。13:00 自然 POKED，自醒安装检查 exit 0。fleet-status、抢救、卡可点击、verdict 引用/缺陷/验法各 exit 0；**无卡不是交付验绿**。文档默认 gate exit 1，仍是预算文档缺模板头。pane 子进程父子关系确认六席位与 Aegis 运行 saih；waker pid 94069。psql -X -v ON_ERROR_STOP=1 stdin 查询成功，库时钟 05:00:29+00：1 件 done 派单（Mira，无验收卡）、0 验收、0 pending 决策、0 无卡 pass、绑定 7、10 席位。

## Lyra 又找到下一层，而且这次是我自己修的那件里的

她交了 `LYRA-2026-11-04-the-dirty-flag-is-exported-and-nothing-reads-it-v1.md`：我导出的 `SEATLOOM_SANDBOX_DIRTY` **有 0 个消费者**，`RuntimeIdentityDto` 只有四个字段，所以「沙箱标识会写明它带未提交改动」这句承诺只活在运行终端里，不在操作者点「接受」时看的那块屏幕上。

### 我的独立测量，以及我自己的一个测量错误

我先跑了一遍 grep，**返回 0 for 全部五个变量**，包括她说有 22 处读者的那个。那不是代码的问题，是我的命令错了：`--type rust --type ts --type tsx` 这种写法在这个 rg 版本里没生效。改成 `-g '*.rs' -g '*.ts' -g '*.tsx'` 后重测：

| 变量 | 引用它的文件数 |
|---|---|
| SEATLOOM_SANDBOX | 1 |
| SEATLOOM_SANDBOX_BRANCH | 1 |
| SEATLOOM_SANDBOX_COMMIT | 1 |
| SEATLOOM_SANDBOX_BUILT | 1 |
| SEATLOOM_SANDBOX_DIRTY | **0** |

**她的结论成立**（她数的是出现次数、我数的是文件数，这是口径差异，不影响结论；我在提交信息里按我实测的口径写）。**我第一次的 0 是一次假测量，差点让我以为整条链都不通。**

### 修法（b5762c4）

四层全部接上：环境变量 → `RuntimeIdentityDto.uncommitted: Option<u32>` → 横幅子句 → 测试。**三态，不是两态**：大于 0 时说明这份构建不等于上面那个提交；等于 0 时说明安装器查过、是干净的；缺失时说「未记录」。**解析不出来的计数算缺失，绝不算 0** —— 把 `abc` 压成 0 会让一个坏计数渲染成「无未提交改动」，那是用关于读取能力的状态冒充关于代码的状态。安装器改成每次都导出（含 0），这样「没导出」只可能是非本脚本构建的东西。

### 变异实测（两个方向，都验了会失败）

| 变异 | 结果 |
|---|---|
| 删掉横幅里的渲染子句（正是 Lyra 指出的缺陷） | **5/13 失败** |
| 把「缺失」压成「干净」 | **2/13 失败** |
| 两次恢复后重跑 | 13/13 通过，`git diff --stat` 只余本次改动 |

`cargo test runtime_cmds` 8 passed；`vitest SandboxBanner` 13 passed；`tsc --noEmit` exit 0。

## 边界

- **没有构建、没有安装、没有启动任何应用**，没有动主版 pid 90805。这是源码级改动，不是部署；已装的应用里横幅仍是旧的。
- `--allow-dirty` 的注入在 `--apply` 路径，dry-run 到不了。**注入效果仍未实测**，本轮只验了代码路径和测试，这一条我继续挂着不勾。
- 冷启动的 A-D 用户验收仍未做。
- 没有改 live 库、没有发门事件、没有合并席位分支、没有重启席位。
- 未跟踪的 `scripts/bind-seatloom-seats.sh` 保留原样。
- 本轮无派给我的账本派单，未调用 `remember`/`emit_event`/`complete_dispatch`。

## 一个模式，我记下来

三次都是同一形状：我修掉我瞄准的那一层，下一层不在我的视野里。第一次守卫提交号，看不见脏工作区；第二次导出脏标志，没有任何东西读它。**共同点是每一层都终止在「运行脚本的人」能看到的界面上，而真正要做决定的人看的是应用里的那块屏幕。** Lyra 三次都指对了方向，且她没有替我改代码，只是测量并要求我复核 —— 这个分工是对的。
