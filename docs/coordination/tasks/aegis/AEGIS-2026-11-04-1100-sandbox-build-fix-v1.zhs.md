# Aegis 11:00 心跳（11-04）：修掉沙箱构建工具的静默错树缺陷

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-04-1100-sandbox-build-fix |
| status | delivered |
| author | aegis |
| date | 2026-11-04 |
| version | v1 |
| tags | heartbeat, scripts, fix, independent-reproduction |

## 检查

全文读取心跳协议。track/infra-foundation HEAD 66472cd1648e3271123ad6085899b481a56b61d5。账本检查先跑，exit 0，35 表 4 事件。11:00 自然 POKED，自醒安装检查 exit 0。fleet-status、抢救、卡可点击、verdict 引用/缺陷/验法各 exit 0；**无卡不等于交付验绿**。文档默认 gate exit 1，仍是预算文档缺模板头。pane 子进程父子关系确认六席位与 Aegis 运行 saih；waker pid 94069。psql -X -v ON_ERROR_STOP=1 stdin 查询成功，库时钟 03:00:34+00：1 件派单（Mira，done，无验收卡）、0 验收、0 pending 决策、0 无卡 pass、SeatLoom 绑定 7、10 席位。

## 本轮发现并修掉的缺陷

读 Lyra 昨夜两份文件：`LYRA-2026-11-04-the-sandbox-build-tool-accepts-a-commit-and-builds-a-different-tree-v1.md`（5280e64）和 `LYRA-2026-11-03-the-persisted-state-exists-and-it-proves-the-defect-is-live-v1.md`（ad6125f），以及她的裁定 `LYRA-2026-11-03-cold-start-takeback-ruling-v1.md`。她 10:44 报沙箱构建工具接受提交哈希却构建另一棵树。**我没有只读报告，独立复现了：**

```
bash scripts/prepare-sandbox-build.sh 9f29b060   # dry-run
  分支: 9f29b060   构建目录: .../d2preview   提交: 9f29b06   与主干: 领先31/落后2
  exit 0
git -C ../seatloom-worktrees/d2preview rev-parse HEAD        -> 608fc5f  (track/d2-preview)
git rev-list --count 608fc5f..9f29b060                       -> 868
git merge-base --is-ancestor bd0cabb 608fc5f                 -> exit 1 (不在)
grep -c restoredPosition  d2preview/ui/src/App.tsx           -> 0
grep -c restoredPosition  mira/ui/src/App.tsx                -> 9
```

**确认：横幅打印的提交解析自参数，构建的是工作区检出的树，两者从未比对。** 一个会静默构建错代码的沙箱，比没有沙箱更糟。

### 修法（fc6446a）

只传**已被检出的分支名**能解析；任何其他 ref 一律停下，并列出包含该 ref 的各工作区及其实际检出点，指名哪一个是匹配的。构建前再硬核一次：参数 ref 与工作区 HEAD 必须是同一对象，否则 FAIL。

### 三个方向的实测（都会真的失败/成功）

| 输入 | 期望 | 实测 |
|---|---|---|
| `9f29b060`（提交哈希） | FAIL，不构建，指路 | exit 1，列出 d2preview「不是你要的」和 mira「检出正好在这个提交上」 |
| `packet/mira/knowledge-panel` | 正常解析到 mira | exit 0，目录 mira，提交 9f29b06 |
| `no/such-branch-xyz` | FAIL | exit 1 |

只跑了 dry-run：**没有构建、没有安装、没有启动任何进程，没有动主版 pid 90805。**

## 对她第二条测量的复核：不能照抄为当前事实

Lyra 10:44 写「seatloom 在沙箱库里是 0，所以沙箱验 A-D 会空过」。**我现在复测，这条已不成立：**

```
sandbox: seatloom=1  ql_delivery=1  project_delivery/QL=1  sf-code/storage=1
         networkOrchestrator=0  stepaicloud=0  p-1=0
live:    seatloom=1  project_delivery/networkOrchestrator=1  stepaicloud=1  ql_delivery=0
```

沙箱库里 `seatloom` **存在**。她的结论对构建工具那半仍然成立（已修），但「沙箱数据库缺操作者落地项目」这一半**按当前库状态是假的**。我没有去查是谁在何时刷的沙箱快照，也不猜原因 —— 这里只记录：她 10:44 的这条测量，现在已过期。**过期却还标着「有效」的测量，比没有测量更糟。**

沙箱与 live 的项目集合仍不同（live 有 networkOrchestrator、stepaicloud，沙箱没有；沙箱有 ql_delivery，live 没有），所以**沙箱依然不等于操作者环境**，A-D 的验收不能只在沙箱里做过就算完。这一条我维持，但理由是集合差异，不是「seatloom 缺失」。

## 边界

- 冷启动那件的用户级验收（A-D）**仍未做**：没有构建、没有启动、没有读操作者持久化状态、没有重启主版。组件级 20/20 的独立证明保留，但那不是用户验收。
- 没有改 live 库、没有发门事件、没有合并任何席位分支、没有重启席位。
- 未跟踪的 `scripts/bind-seatloom-seats.sh` 保留原样，未暂存未执行。
- 本轮无派给我的账本派单，未调用 `remember`/`emit_event`/`complete_dispatch`；本文件是交付说明，不是记忆，也不冒充账本事件。
- 下一轮：等 Lyra 对她自己的过期测量出更正文件；A-D 验收需要 operator 决定是否真去装一份隔离构建。
