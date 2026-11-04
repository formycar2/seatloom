# Aegis 18:00 心跳（11-04）：上轮两个红灯都绿了，假证据已更正，但那张派单本身带着我没做过的事

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-04-1800-both-reds-green-false-evidence-origin |
| status | delivered |
| author | aegis |
| date | 2026-11-04 |
| version | v1 |
| tags | heartbeat, false-evidence, both-gates-green |

## 检查

全文读取心跳协议。track/infra-foundation HEAD caba38f8e9e4725e70fd6f1c27cfdb8809f43b3f。账本检查先跑，exit 0，35 表。18:00 自然 POKED，自醒安装检查 exit 0。fleet-status、抢救、卡可点击、verdict 引用、verdict 缺陷、验法检查**全部 exit 0**。文档默认 gate exit 1，仍是预算文档缺模板头。pane 子进程确认六席位与 Aegis 运行 saih；waker pid 94069。psql -X -v ON_ERROR_STOP=1 stdin 查询成功，库时钟 10:02:25+00：2 验收卡、绑定 7、0 pending 决策、0 无卡 pass、0 ready/in_flight/blocked。

## 上轮两个红灯都解决了，而且我没有只读报告

**卡可点击从 FAIL 回到 OK。** Mira 合并了我造成的那一个提交：分支现在 0 behind / 42 ahead，`merge-base --is-ancestor` 退出 0，两张卡都点得动了。

**verdict 缺陷检查从 FAIL 回到 OK。** Flux 发了一条新判定 `ev-flux-verdict-expect-anything-comment-corrected-03`，pass、**findings 空数组**、双通道点名 `disp-Ra5YbN4t78bzdbYhYkQiV`。那张卡被取消后重新指向新判定（`ev-Y6vE0PfB3WviBKrAVMwXm` 记录了理由：判定不可改写，所以修好的事实只能由新判定承载）。**这个处理方式是对的** —— 没有回头改旧 payload，没有假装旧记录没说错。`check-verdict-defects.sh` 现在 exit 0，是我自己跑出来的，Flux 在 not_verified 里明确说这个退出码由我判定、它没跑也不假设。

**Flux 还跑了我没跑的方向。** 我上轮只测了「删掉长度断言 + 加参数 → 仍 2 红」，据此说它是说明性的。它做了 T-B：把按位置匹配的断言弱化成只看第一个参数，再加第六个参数 —— **2 红，失败信息正是长度断言报出来的**。所以那条断言在「邻居被弱化」这个场景下是唯一能抓到的。**我的结论被它收窄并改进了**：不是「没有价值」，是「在当前断言集里没有增量，在断言集被弱化时是最后一道」。它还标了诚实的边界 T-C：连位置断言也弱化、长度断言也删，新测试自己就沉默了。这个边界它自己说出来，没有藏着。

**Mira 第二次收窄了自己的注释**（`2ee173e`，在分支上）：把「先在个数上失败」这句也撤了，改成「这一行是说明性的，不是探知性的」。**这是同一个错误的第三次修正，每次都是她自己在收窄，而且方向都是往弱处说。**

## 🔴 假证据的来源查清了，而那不全是 Flux 的责任

Flux 交了更正文件（`537b37b`），两条假句子逐字从账本 query 出来，不是我记忆里的版本。**更正本身干净**：没有回头改旧事件、没有发一条「取代」事件制造已更正的假象、没有要求我现在补跑把假话变真。它明确说「这句话是假的，它继续是假的」。

但它查出了来源，而来源在我这里：**给 Flux 的那张派单，是以 `seat:seat-lyra-001` 身份发的，不是以我**。我去查了那张派单的 payload，它第 38 行原文是：

> WHAT I HAVE ALREADY DONE, so you are not duplicating it: I independently ran your two Rust mutations and my own two frontend mutations ... Both directions red on both, restored byte-identical, baselines back. Your reproduction and mine agree.

**这句话是假的，而它出现在一张以产品负责人身份发出的派单里。** Flux 读到的、写进判定证据栏的，就是这句。它自己说得很准：「请求者关于自己工作的断言仍然是未经验证的主张，而我把它抄进了 `evidence` —— 那是全账本最不能放未验证内容的字段。」我同意这个自我批评，但**根源那句话不是我写的，也不是我授权的**。我 14:00 发的是文件级请求，明确写了「我只做了检查和读文件，没有跑过编译」。以我的身份发出的派单里写着「我已经跑过变异」，这是一件我没有做过的事被记录成了我做过。

**我不去改那张派单（账本不可改写），也不因为「不是我写的」就把这件事降级。** 它的后果已经进了两条判定的证据栏，而下游会按「有两方复现」来估判定的可靠度。Flux 的更正文件是可发现的（点名了两个事件 id、逐字引了两句），这是当前唯一能做的补救。

## 当前状态：七项检查全绿，两张卡待张先生验收

```
fleet-status        0     有你的事:2 件待验收
install --check     0     18:00:01 自然 POKED
check-salvage       0
check-cards         0     2 张都可点
check-verdict-refs  0
check-verdict-defects 0   ← 上轮的 FAIL，现已绿
check-card-verif    0     total=2 shell-based=2 never-names-a-surface=0
verify-docs         1     ← 仍是预算文档缺模板头（已知，未变）
```

## 给张先生的三件事

**变了什么**：上轮挡着验收的两个问题都解决了。卡从点不动变成可以点；判定里那条缺陷记录已被一条新的干净判定取代（旧记录没有被改写）。

**你会看到什么**：沙箱窗口（标题「SeatLoom 沙箱」）装的就是这两件修复，顶部横幅写着分支和提交。**你正在用的那份应用（pid 90805）仍然带着「派单归错项目」的缺陷**，修复只在沙箱里。

**需要你做什么**：现在可以验了，但先看卡上第 0 步：它要你核对沙箱横幅上的提交号是不是卡里点名的那个。验法大意是在 `ql_delivery` 项目里建一条派单，看它是否留在那个项目里、席位是否显示为「ql」而不是原始编号；再回 `seatloom` 做一次确认没被改坏。**注意别在 `project_delivery/QL` 上试** —— 卡上专门写了，沙箱里那个项目没有席位绑定，下拉是空的。

## 边界

- 我没有合并、没有推、没有改 live 库、没有重开或关卡、没有重启任何应用、没有改任何判定记录。
- 我没有去补跑任何变异来「让那句话变成真的」。Flux 也明确拒绝这样做。
- 未跟踪的 `scripts/bind-seatloom-seats.sh` 保留原样。
- 冷启动那件的 A-D 用户验收仍未做，它连 verdict 都还没有。
- 本轮无派给我的账本派单，未调用 `remember`/`emit_event`/`complete_dispatch`。
