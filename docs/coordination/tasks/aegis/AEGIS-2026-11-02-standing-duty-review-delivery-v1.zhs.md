# Aegis 心跳补充交付：MCP 读取恢复，五席位常规职责落地仍需修订

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-11-02-standing-duty-review-delivery |
| status | delivered |
| author | aegis |
| date | 2026-11-02 |
| version | v1 |
| tags | heartbeat, standing-duty, evidence, review |

## 变了什么

13:27 本轮从 `134789aebc4604e4579191d9fdb2c0dcb9a818f8` 开始；已全文读取 Aegis 心跳协议。
重复消息合并到当前轮，不重复安装、不启动第二个 Aegis。

SeatLoom MCP 的 `query_ledger(project_id=seatloom,limit=5)` 实际调用成功，isError=false，
返回事故记录 `ev-__d45RX5PFjTs33KTPuy6`，与此前直接查库的 1 条事件一致。
这证明读取通道恢复，不证明派单已成功入队/被领取，也不授权为探测制造 live 写入。

psql stdin + ON_ERROR_STOP + 常量证明行查询成功：DB 时间 `2026-11-02 05:27:21+00`，
35 张表、dispatches=0、acceptances=0；队列、近48小时完成无卡、验绿无卡、待裁定分别为空。
六个核心席位 pane 的子进程均为 saih（ps 父子关系核验），waker 为 pid 94069。
抢救补丁仍匹配两工作区；所有卡检查退出 0，但对象为空，不等于全部交付已验证。

自醒 --check 返回 0，原本/副本/目标一致。日志最后 POKED 仍为 13:01:50 安装验证，
尚不能声称新版已经历下一次自然整点唤醒。

## 新核到的真正未完事项

Lyra 新报告 `72a629c49be7611ddf9142bad8a3a8dc42a58e95` 指出 Aegis 自醒与舰队常规职责唤醒是两条机制。
读取该报告全文后，独立核验：

- `.seatloom/dispatch-seats.tsv` 映射六个核心席位，不含 Aegis；Aegis 明确由 cron 自醒。
- `waker.rs:902–906,990–1008` 依赖主检出相对路径 `<SEAT>-HEARTBEAT.md`。
- `lsof -a -p 94069 -d cwd -Fn` 确认运行 waker cwd 是主检出。
- 主检出 Lyra 有对应文件；Mira/Nimbus/Flux/Onyx/Vela 五个文件均不存在。
- Aegis 协议存在于新名称并经 cron 消费，不是“没有任何名字的职责文件”；不应把它加入舰队映射造成双重自醒。

所以**五席位常规职责消费链未落地**成立，但“Aegis 无职责协议/应恢复旧名”不成立。
上轮修复解决 Aegis cron 的真实错误，不宣称解决全部舰队常规职责。

## 原稿审查与暂不落地的原因

已从可验证的来源 `origin/packet/lyra/standing-duty-docs-only` 固定提交
`7e067388a282a5739703a87a4cbc0172f8c0cac1` 取出五份职责文档和 persist 裁定到 /tmp，全文读完。
没有改源分支、没有把这些文件写入主检出；不做全分支合并，不直接运行旧落地脚本。

五份文档共同问题：

1. Step 0 使用 `check-ledger-exists.sh --quiet || echo ...`，失败分支 echo 成功，整体退出 0。
   离线控制 `false || echo ...` 实测退出 0，故不能作为会返回失败的停止检查。
2. 错误解释为不存在的表也返回 0。只读事务中查询一个明确不存在的表，psql 实测报 relation does not exist，ON_ERROR_STOP 下退出 3；无数据写入。
3. 文档里的历史数字/具体缺陷不能当当前值；本轮数据库已空，不能把旧基线作为每轮必须精确匹配的现状。
4. 部分测量方法需修订：Nimbus 不可用二进制时间戳证明源码一致；Flux 的 verdict 主体需承认 payload 与 dispatch ref 两条有效通道；Onyx 列注释应使用列注释读取而非表对象注释。
5. Vela 文档存在角色文件“仍是角色”与“角色文件不存在”的自相矛盾，且派单 id 两处不同；引用须核原证据，不把 typo 当事实。

同一 SQL 错误也存在于我自己的 Aegis 协议。本轮仅精准改正自己的说明为：表不存在报错，查询成功后再解释计数，禁止忽略退出码/丢 stderr 后报空。不是替别人修文档。

旧 `land-standing-duty-docs.sh` 在 Lyra 工作区，全文读过。它会在确认之前写文件/暂存，
不是干净 dry-run；它对“路径限定纯新增”仍强制整个源分支包含当前 HEAD，
该约束不等于六个固定文档提取的实际必要条件；也要求自身不能被跟踪。
因此不能把旧脚本原样拿来向操作者承诺“一条可运行命令”。

## 下一步修订要求（未在账本派发）

Lyra 负责修订五份职责原稿，保持只读检查/上报职责，不新增无人审批修复或 live 写入权。

- 账本检查必须失败后停止，保留真实退出码与错误；移除错误 SQL 解释。
- 历史例子注明测量日期；当前数值每次从正确来源量，未知就写未知。
- 修正上述测量/主体关联/引用问题；不通过删弱判据让文档变绿。
- 提交一份固定 source commit + 六个准确文件路径的交付清单（五职责加 persist 裁定）。
- 落地脚本应默认只读，应用才取文件；按目标路径检查冲突，不合并整分支、不 rebase、不 glob 暂存，不修改无关脏文件。
- 无需修改 waker 就能消费正确路径；但生效必须以运行服务日志的各席位实际调用证明，不能只用复制的 Rust 函数检查文件存在。
- 由于旧文档可能不在席位自己的工作区，收到相对路径后能否读到仍要验；不能只证明主检出存在就停。

这是待修订/交付要求，不是已入队派单、不冒称对方已收到，也不委托被拒的 live 操作。
权限与原有人工验收流程保持不变。未声称常规职责已生效。

## 验证和记录

`check-salvage-fresh.sh`、自醒 --check、卡检查均返回0，注明空样本边界如上。
文档 gate 与 git diff --check 在提交前运行；只提交自有协议修正和本说明。
用户 `.gitignore` 与12:00派单原文不动。本轮没有记忆写入/ArtifactCreated/complete_dispatch：
没有该轮生命周期事件，不制造 canonical_event 来源，不以文件冒充记忆，也不补造旧派单历史。

## 需要张先生做什么

本轮无需重复跑注册或自醒安装。架构报告已交付；五份职责和落地脚本先按上述修订，
然后才交一条可审计命令。当前没有可靠依据让操作者先运行旧脚本。
