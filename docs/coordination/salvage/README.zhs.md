# 未提交工作的抢救存档

| Field | Value |
|---|---|
| template | T7 |
| subtype | salvage |
| id | salvage-README |
| status | recorded |
| author | aegis |
| date | 2026-10-31 |
| version | v1 |
| tags | salvage, uncommitted, integrity |

**2026-10-31 15:00 建。** live 库被清空之后,git 提交成了这个项目唯一耐久的记录,
而**未提交的工作是第二脆弱的** —— 它不在任何提交里,`git bundle` 也不包含它
(bundle 记的是 commit,未提交的改动按定义不是 commit)。

## 这里放的是什么

每个 `.patch` 是某个席位工作区里**未提交**改动的完整快照,
用 `git diff > x.patch` 生成。它让人能在不动那个工作区的前提下,
看见、检查、必要时恢复那份工作。

## 已知的漏洞,以及我为什么还是放在这里

**这些补丁会过期。** 它们对应某个时刻的工作区状态;那个席位一旦提交、
变基或丢弃改动,补丁就再也贴不回去了。

所以每个补丁都要配一句:
**怎么用它**(`git apply --reverse` 可以反向验证它还对应那份工作区状态)。
本次两个补丁在 15:00 都用 `git apply --check --reverse` 验过,两个都还贴得上。

**不要把这些补丁当成备份。** 备份是 `backups/*.bundle`
(`scripts/backup-unpushed-branches.sh` 生成),那个验过能完整恢复。
补丁是**线索**,不是副本。

## 为什么从家目录搬进仓库

第一版我写在 `~/seatloom-uncommitted-salvage/`,仓库外。
Lyra 去找,找不到,于是她在自己的交付里写:
「这个项目第二脆弱的东西,被一个我找不到的产物保护着」。

**她说得对,而且这正是她一直在要求别人的标准。**
一件保护性产物,如果从别人的位置看不见、验不了,那它就不算存在 ——
它只在我自己的叙述里存在。这和「往账本谎称某事已发生」是同一类错误,
只是规模小得多:我说"我存了",而没有任何人能确认。

搬进仓库之后:它跟着提交走、能被 `git log` 追、任何人 `ls` 都能看见。

## 清单

| 文件 | 来自 | 内容 | 状态 |
|---|---|---|---|
| `nimbus-session-work-readmodel-20261031-1302.superseded.patch` | `packet/nimbus/session-work-readmodel` | 124 行,记忆去重前置检查。**是半个没打完的补丁**:在原来的 `let mut client` / `let tx` 之后又插了一对,第一个事务被遮蔽、从不提交也不回滚,`cargo check` 报 `unused variable: tx`。修法是删掉原来那两行,不是把补丁补完 —— 但那是 Nimbus 的决定。 | **已失效,别拿它恢复。** 2026-11-07 08:00 实测:它的内容已经作为提交落在同一个分支上,而且正是按上面那句话的方向修的 —— `git apply --check --reverse` 贴不回去(内容不在工作区里),`ee2d03b` 带着 `MemoryRejection::Duplicate`,`57b7737` 带着 `memory_dedup_key` 并且**删掉了那对死事务**。所以这不是工作丢失,是补丁完成了它的使命。 |
| `nimbus-candidate-final-20261108-1030.patch` | `packet/nimbus/candidate-final` | 82 行插入 / 2 行删除,`scripts/test-build-failure.sh`。**这是 33909f1 之后 Nimbus 手上正在做的活**:fixture 里那个假二进制原来印的是 `git rev-parse --short=12 HEAD` —— fixture 自己那个一次性仓库的 HEAD,那个提交在主仓库里**不存在**。Lyra 量到 `/Applications` 里那份假二进制带着 `7d577df02462`,`git cat-file -t` 在主仓库里失败,于是她无法判断那是真的还是编出来的。改成一眼就知道是 fixture 产物的字符串。 | **11-08 10:30 生成,双向验过**:`git apply --check --reverse` 贴回同一棵树通过,`git apply --check` 贴到一份干净的 `HEAD` 副本上通过。所以它是未提交工作的忠实拷贝。 |
| `nimbus-candidate-final-20261107-2100.superseded.patch` | `packet/nimbus/candidate-final` | 131 行插入 / 65 行删除,受保护路径改成两个独立条件的中间态。 | **已失效,别拿它恢复。** 2026-11-08 10:40 实测:它的 133 行新增里 **121 行逐字存在于 HEAD**(`33909f1` 之后),而那 12 行不在的,正是这次修替换掉的写死字面量(`REAL_SANDBOX_PATH="/Applications/..."` 等)—— 也就是说这份补丁的活已经被后面的提交做完并取代。不是工作丢失。 |
| `nimbus-candidate-final-20261107-1600.superseded.patch` | `packet/nimbus/candidate-final` | 190 行插入,`scripts/test-build-failure.sh` 的 `--apply` MUST_HAVE 测试。 | **已失效,别拿它恢复。** 2026-11-07 21:00 实测:它的结果状态 `f38f5d2` 已经作为提交 `0a7f1a7` 落在同一个分支上(`git log --find-object=f38f5d2` 命中 `0a7f1a7` 和 `48a216f`),基线 `f6ee787` 也在 `0a7f1a7` 上。`git apply --check --reverse` 贴不回去。所以这不是工作丢失,是那份活已经提交了。 |
| `lyra-milestone-summary-20261102-1102.patch` | `packet/lyra/milestone-summary` | 2 个文件的文档改动(都是既有文件,无新增)。 | **11-02 11:02 生成并验过贴得上** |
| `lyra-milestone-summary-20261031-1302.superseded.patch` | 同上 | **已被上面那份取代,留着只为历史。** 它含那次 `version v2→v3` 的一行改动 —— 那一行 11-01 04:00 被她提交了(`61fcc11`),所以它不再是「未提交的工作」,而是**已经在版本库里的一个待裁决事项**。 | 已失效,别拿它恢复 |

## 11-02 11:02 补:补丁是会过期的,而我五个小时没验

我 15:00 写下「补丁会过期」,然后**五个小时没有回来验**。
11:00 这一轮去验,Lyra 那份已经 stale 了 —— 不是她的工作丢了,
是**我对它的描述过期了**:那一行 v2→v3 已经被她提交,
所以那份补丁里有一块不再属于「未提交」。

所以这条要改成流程,不能靠记得:
**每次心跳都 `git diff` 一次、和 salvage 里的对一次,不一致就重存并把旧的标成 superseded。**
一个过期却还在清单里、还标着「有效」的补丁,比没有补丁更糟 ——
它会让人以为那份工作被保护着,而恢复出来的是一份旧状态。

## 11-07 08:00 补:第二次抓到期盼失效,而这次那份工作已经落地

`check-salvage-fresh.sh` 报 `nimbus-session-work-readmodel-20261031-1302.patch` 贴不回去。
我按协议改了名、更新了清单,但**没有停在「补丁过期」这个结论上** ——
过期有两种,而它们的含义相反:

1. 工作被提交了(补丁该退役);
2. 工作被丢弃/覆盖了(补丁是最后的副本)。

量出来的是第一种,证据三条:反向应用贴不回去(内容不在工作区);
`ee2d03b` 带着 `MemoryRejection::Duplicate`;`57b7737` 带着 `memory_dedup_key`
**并且删掉了 README 里点名的那对死事务**。也就是说 Nimbus 做了 README
里留给他的那个决定,而且做的是「删掉原来那两行」这一侧。

**规矩:补丁过期不等于工作消失。改名前先量它是哪一种,不然我会把一次成功
记成一次损失,或者把一次损失记成一次成功。**

## 一件仍然等你定的事

`lyra-milestone-summary-20261031-1302.superseded.patch` 里那行
**`version v2→v3`** 现在是提交里的既成事实(她的 `61fcc11`)。
那份合同文档该是 v2 还是 v3,**只有你能定**。她自己拒绝 revert,
理由是她不知道 v3 是谁写的、是不是有意的。

## 一条规矩

**声称做了保护,就要让保护可被检验。**
写在仓库外的产物、只在自己脑子里的一次操作、「我存了」这三个说法,
对被保护的东西来说是同一件事:什么也没发生。
