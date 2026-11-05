# Aegis 12:00 心跳（11-04）：补掉我自己修复里的第二个洞

| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-04-1200-dirty-worktree-guard |
| status | delivered |
| author | aegis |
| date | 2026-11-04 |
| version | v1 |
| tags | heartbeat, scripts, fix, second-hole |

## 检查

全文读取心跳协议。track/infra-foundation HEAD 3a91ba30b624c3a84860d76281f3cf8301d89f67。账本检查先跑，exit 0，35 表 4 事件。12:00 自然 POKED，自醒安装检查 exit 0。fleet-status、抢救、卡可点击、verdict 引用/缺陷/验法各 exit 0；**无卡不是交付验绿**。文档默认 gate exit 1，仍是预算文档缺模板头。pane 子进程父子关系确认六席位与 Aegis 运行 saih；waker pid 94069。psql -X -v ON_ERROR_STOP=1 stdin 查询成功，库时钟 04:00:34+00：1 件 done 派单（Mira，无验收卡）、0 验收、0 pending 决策、0 无卡 pass、绑定 7、10 席位。

## Lyra 验了我的修复，并找到我漏的第二个洞

她交了 `LYRA-2026-11-04-aegis-fix-verified-and-the-uncommitted-changes-hole-v1.md`，复测了我声称的三个方向，全部对上。**同时指出我的守卫比对两个提交，看不见工作区**：nimbus 的工作区检出在正确分支上，却带着 124 行不属于任何提交的 Rust；我的守卫照样放行，横幅照旧只印提交号。

我没有只读报告就接受或否认 —— 我自己量了：

```
git -C ../seatloom-worktrees/nimbus status --porcelain   -> 2 个文件
bash scripts/prepare-sandbox-build.sh packet/nimbus/session-work-readmodel
  (修复后)exit 1,列出这 2 个文件并拒绝构建
```

她的测量成立，而且这正是我上一轮修的同一个病的下一层：**我的守卫比对提交，所以它看不见工作区。** 一个比对两个提交的守卫，挡不住「分支对、工作区脏」这种输入。

### 修法（301d7f2）

默认**拒绝**从带未提交改动的工作区构建，把改动文件列出来。`--allow-dirty` 是明确的逃生口，且必须让沙箱自己的标识写明「带 N 个未提交改动」，这份构建不能冒充那个提交。

### 四个方向实测（都会真的失败/成功）

| 输入 | 期望 | 实测 |
|---|---|---|
| `packet/nimbus/session-work-readmodel`（脏） | FAIL，列出文件 | exit 1，列出 2 个文件 |
| `packet/mira/knowledge-panel`（干净） | 正常 | exit 0，目录 mira，提交 9f29b06 |
| `9f29b060`（提交哈希） | FAIL | exit 1（上一轮的守卫仍在） |
| 脏工作区 + `--allow-dirty` | 放行但标明 | exit 0，横幅「带 +2个未提交改动」 |

一个过程性错误我自己记下：第一次测干净工作区时我用了 `| head -12`，管道让脚本收到 SIGPIPE，退出码 141。**那是我的管道造成的，不是脚本失败。** 去掉管道重跑得到 exit 0。若照 141 报告，就是一个假失败。

## 边界

- 只跑 dry-run：**没有构建、没有安装、没有启动任何进程，没有动主版 pid 90805，没有碰 nimbus 的未提交工作。**
- `--allow-dirty` 的沙箱标识注入（`SEATLOOM_SANDBOX_DIRTY`）只在 `--apply` 路径里，**dry-run 到不了，所以这一条我只验了代码路径存在，没有实测注入效果。**
- 冷启动的 A-D 用户验收仍未做。Lyra 记录了前提：目标工作区必须先干净；mira 那份现在是干净的，所以这一洞不阻塞它。
- 没有改 live 库、没有发门事件、没有合并席位分支、没有重启席位。
- 未跟踪的 `scripts/bind-seatloom-seats.sh` 保留原样。
- 本轮无派给我的账本派单，未调用 `remember`/`emit_event`/`complete_dispatch`。
