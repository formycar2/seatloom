# Aegis 21:00 心跳（11-04）：两盏红灯，其中一盏是认真的（真的会静默删文件），另一盏是我第三次造成

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-04-2100-two-reds-one-serious |
| status | delivered |
| author | aegis |
| date | 2026-11-04 |
| version | v1 |
| tags | heartbeat, accept-path-guard-gap, third-breakage |

## 检查

全文读取心跳协议。track/infra-foundation HEAD 683422b54aad584685169f4d27c2e61c23f87749。账本检查先跑，exit 0，35 表。21:00 自然 POKED，自醒安装检查 exit 0。fleet-status、verdict 引用、verdict 缺陷、验法检查各 exit 0。**两个红灯**：`check-salvage-fresh.sh` exit 1、`check-cards-clickable.sh` exit 1。文档默认 gate exit 1，仍是预算文档缺模板头。pane 子进程确认六席位与 Aegis 运行 saih；waker pid 94069。psql 查询成功：2 张卡 pending、指向未变、0 pending 决策、1 件 `in_flight`（Nimbus，刚派）。

## 红灯一（这个是真的要紧）：点接受时会静默删文件，而门说「可以点」

账本新增两事件：Lyra 给 Nimbus 派了一件 `in_flight` 活（`disp-AbQ4m1TKQZGLMeZ4bhwjy`）。我去读了派单内容并**独立核了它的核心主张**，因为它不是关于某件功能，是关于**验收门本身会放过数据丢失**。

### 我自己核到的代码事实

```
crates/seatloom-core/src/git/ops.rs:124        merge_would_delete 定义
crates/seatloom-core/src/git/ops.rs:311,317    它的两个测试
crates/seatloom-mcp-emit/src/dispatch.rs:1822  唯一的调用点:request_acceptance(开卡时)
src-tauri/src/commands/acceptance_cmds.rs      0 次
```

**同一个仓库里有两套检查，强度不一样：**
- 开卡时（席位侧）会问「这次合并会不会删掉基准里的文件」。
- **点接受时（操作者侧）不会问。** `cmd_accept_delivery` 只重校验了 verdict（我读了 `acceptance_cmds.rs:52-87`，那段注释明确写着「Checked again at decision time, not only at request time, because a verdict can be re-issued」），然后直接调 `merge_branch_into_base`。

**所以 verdict 会在两个时刻之间被重新核对，分支不会。** 这是代码里自己写出来的不对称，不是我推想的。

我还核了豁免名单：`DELETION_EXEMPTIONS` 只有一条（`ui/src/app-v2/AppV2.tsx`，注明由谁裁定、为什么），而注释说这份名单「由编译器审计，不只是靠一个可能被删的测试」。**Lyra 要求「先减掉操作者裁定过的删除再决定拒绝」是对的** —— 不减就会把一次已被批准的删除当成事故拦下来。

### 这意味着什么（对张先生的直接影响）

派单里量了两个方向，我核了代码但没有复跑它的实验（它是在 scratch 仓库里做的）：
- 方向一：一个**安全**的合并（只加文件、零冲突、零删除）会被现在的可点击性检查拒绝 —— 这件事今天真实发生了三次，每次都因为我一个文档提交。
- 方向二：一个**会删掉基准里文件**的合并会被同一个检查放行，因为不冲突、且分支与基准齐平。

**方向二就是静默数据丢失，而系统的门会显示「可以点」。** 这不是理论风险，是「什么都不冲突所以什么都没拦住」。

**我没有自己去实现它** —— 那是 Nimbus 的活，已经 `in_flight`，而且他的工作区里已经有五个文件在改（见下）。

## 红灯二（这个不新鲜，但第三次了）：卡又点不动，还是我造成的

```
FAIL: 2 of 2 pending cards are UNCLICKABLE
packet/mira/knowledge-panel|acc-disp-Ra5YbN4t78bzdbYhYkQiV|behind 1
packet/mira/knowledge-panel|acc-disp-XpAdx0a9YLL83JVoPesTj|behind 1
git log --oneline 0d1cd1e..HEAD   →  只有 683422b（我 20:00 的交付说明）
```

**第三次，同一个原因，同一时段。** 我上一轮的说明提交又让她落后 1 个。这次我不再重复问甲/乙/丙怎么选 —— 那个问题 19:00 就报给你了，还没答复。我按原样（乙）请她再合并，但**我要把频率说清楚：只要卡在等验收，我每小时提交一次说明，她就要每小时合并一次。**

## 红灯一的伴生发现：Nimbus 的未提交工作没有补丁覆盖

`check-salvage-fresh.sh` 报 `nimbus-psr` 有 5 个未提交文件、没有任何补丁覆盖。**我查了时间，这不是被遗弃的工作，是正在进行的活**：

```
11-04 20:54  crates/seatloom-core/src/git/ops.rs
11-04 20:55  crates/seatloom-mcp-emit/src/dispatch.rs
11-04 20:58  crates/seatloom-core/tests/pipe_guard_real_steal.rs
11-04 21:00  crates/seatloom-core/tests/accept_deletion_guard.rs   ← 新建
11-04 21:01  src-tauri/src/commands/acceptance_cmds.rs
宿主机现在    11-04 21:01
```

**最后修改时间就是这一分钟。** Nimbus 的面板活着（pane 5964，子进程 saih）。**这正是抢救检查存在的理由：一个 seat 正在写的活，如果此刻掉电就没有任何副本。** 但我不去动它 —— 不 stash、不提交、不打补丁，那会干扰一个正在工作的席位。我只报告：**这 5 个文件目前没有任何补丁保护，风险窗口是真实的。**

## 给张先生的三件事

**变了什么**：发现验收门本身有一个洞 —— 点接受时不会检查「这次合并会不会删掉你仓库里的文件」，而开卡时会。Lyra 已派 Nimbus 去补（进行中，这一分钟还在改）。

**你会看到什么**：那两张卡又点不动了（我造成的，第三次）。沙箱仍是那份修复；你正在用的应用没变。

**需要你做什么**：
1. 那个结构冲突三个选项（甲/乙/丙）仍等你定 —— 现在已经第三次发生了，不是偶发。
2. Nimbus 正在改的东西**会改变验收门的实际行为**。等它落地、独立验过之后，你要验的就不只是「派单归对项目」，还包括「门真的会拦住删除合并」。这个我不会替你提前勾掉。
3. 如果你希望「有卡等验收时我不再每小时提交说明」（选项甲），一句话我就改。

## 边界

- 我没有碰 Nimbus 的 5 个未提交文件、没有 stash/提交/打补丁，没有实现那个 guard。
- 没有合并、没有推、没有改 live 库、没有重开或关卡、没有重启任何应用。
- 我没有复跑 Lyra 派单里的两个 git 实验（它在 scratch 仓库做的），我只核了代码里检查的存在与缺失。
- 未跟踪的 `scripts/bind-seatloom-seats.sh` 保留原样。
- 冷启动那件的 A-D 用户验收仍未做。
- 本轮无派给我的账本派单，未调用 `remember`/`emit_event`/`complete_dispatch`。
