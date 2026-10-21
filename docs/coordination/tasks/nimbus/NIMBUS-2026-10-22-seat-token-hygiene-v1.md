# NIMBUS-2026-10-22-seat-token-hygiene-v1 — 交付说明

派单 `disp-5Wwwsq00WJUfPDAUV5t--`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-22-seat-token-hygiene-v1.md`。
分支 `packet/nimbus/project-scoped-reads`。

## 这一单没有代码工件,所以先说交付物是什么

习惯这种东西只活在会话上下文里,会话一结束就没了 —— 那正是张先生定过的
「记忆写在仓库里,不写那个 agent memory 目录」要防的事。所以交付物是一条
**落库的知识条目 + 三条判据**,不是一段我提醒自己「以后注意」。

新增 `docs/coordination/knowledge/seat_tool_calls_must_not_restate_cwd_or_reread_files.md`
(并登记进 `INDEX.md`),frontmatter 对齐 `memories` 表的列,`l2_file_path` +
行号锚到任务包上,`confidence: quoted`(张先生的原话在里面),`kind: failure`。

## 一、Aegis 的「之前」数,我复测了,不是照抄

任务包给的是 582 次 / 21 回合。我在一份更早的 session 文件上复测,只会更大:

```
session: ~/.pi/agent/sessions/*nimbus*/2026-10-16T04-59-29-536Z_*.jsonl
bash                        1242
cd <绝对路径> && 前缀        1209
最常读的那个文件               6 次
```

说明:那份 session 跨了 54 个派单(`get_next_dispatch` 54 次),所以不能直接和
「一件事 582 次」比。它能证明的是量级和形状 —— `cd` 前缀的数量级和 bash 调用
同数量级,也就是**几乎每条 bash 都带一次零信息量的前缀**。

## 二、这一单我自己做了什么,以及计数

任务包要求「给出你自己的计数」。这一单从 `get_next_dispatch` 到写这条说明:

| 工具 | 次数 |
|---|---|
| codemode(get_next_dispatch / describeTool / complete_dispatch) | 4 |
| read(任务包) | 1 |
| bash | 12 |
| write(知识条目 + 交付说明) | 2 |

**合计约 19 次调用**,远低于 300 的上限。

三条判据逐条自查:

1. **同一文件 read ≤ 2 次** —— 满足。整个派单只 read 了任务包一次。
2. **一条命令里 `cd <绝对路径> &&` ≤ 1 次** —— 满足,且多数 bash 根本没用 `cd`
   (bash 工具本身就在 worktree 起,这是任务包第一句就指出的事)。
3. **一件事调用数 ≤ 300** —— 满足,19 次。

有一处我要如实说:开头定位 session 文件时我连发了 7 条 bash
(`ls ~/.pi` → `ls ~/.saih` → `find` → 两次 `ls -t` → 又一次 `ls ~/.pi` → `find -newermt`
→ 一次 for 循环),其中第 5 条和第 3 条在问同一件事。**这正是不该犯的那个毛病**,
我当场收住了,后面一次 bash 把统计做完。写在这里而不是省掉,是因为一条
「我注意过了」的记录没有价值,一条记录了自己在哪一刻犯了、然后改什么的才有。

## 三、哪些验证保留了(这条最重要)

**判据全部保留,一条没省。** 上面三条判据是**实测**的,不是估计的:

- 判据 1 和 3 的数来自逐条点数我自己这一单的调用
- 判据 2 检查了每一条 bash 命令的 `cd` 前缀
- 「Aegis 的 582 次我没有照抄」—— 复测了一份 session 文件,并且说明了为什么
  那个数不能和 582 直接比(跨了 54 个派单)。能比的是量级和形状。

**没有为了让数字好看而:**
- 跳过任何验证(判据全跑)
- 把测试改弱或改恒真(这一单没有测试可改,更没有借口)
- 把「read 一次任务包」说成「read 零次」

## 四、知识条目里钉死的东西

除了三条判据,条目里写进了两条不许碰的红线,因为它们和「省 token」最容易被混为一谈:

1. **不为省调用而跳过验证。** 省调用的正确方向是去掉重复劳动(重述 cwd、
   重读文件、逐条查询),不是去掉检查。
2. **不许因为省 token 把测试改弱或改恒真。** 一个永远不会失败的检查比没有检查
   更有害 —— 它把「验过了」变成一句无法证伪的话。

## 五、需要上报

1. **这个习惯的复发风险是结构性的。** 它靠会话内的自觉,而会话会结束、会压缩、
  会换模型。知识条目能降低复发,但不能保证。如果 Aegis 想让它真正可核,
   可行的方向是在派单交付说明里固定要求报这三个数(任务包这次就是这么要求的)——
   那等于把判据变成流程的一部分,而不是一条自律。**这是 Aegis 的决定,不是我的。**
2. **派单长度本身是省 token 的另一半。** 任务包承认了(Aegis 改成一件事一单、
   派单只写三段)。这一单我读到的那份派单本身约 2KB,已经比它自述的 3-5KB 短,
   方向是对的。

## 提交

`docs(nimbus): seat token hygiene — measured, with the criterion kept intact`
(绝对哈希见 `git log --oneline -1`)。`git status` 干净。

---

## 追加(派单 `disp-lVJKIg1QtRSIlhJ-FlDou` —— 改用 edit,不用 heredoc)

任务包要求「在已有交付说明里追加一句:这件的代码改动用了什么工具、为什么」。
诚实说:**这一单我在开头犯了一次,之后全程走 `edit`。**

### 计数(这一单,自己数的)

| 工具 | 次数 | 用途 |
|---|---|---|
| `edit` | **5** | 全部代码/知识文件改动(冲突解决、两条 INDEX 行、知识条目) |
| `write` | **2** | 新建两个知识条目文件(无 `old` 片段可给,`write` 是正确工具) |
| bash | 约 10 | 全部是**查询**(git 状态、判据核验、测试、grep 冲突标记) |
| codemode | 3 | 派单循环 |
| read | 1 | 任务包 |
| **heredoc** | **2** | 一次是下面那个错误(包 python 改代码);一次是往本说明追加这一节 —— 后者是 markdown 文件,正是规则允许 heredoc 的场合(夹具/数据处理/非代码文件) |

### 那次错误,如实记

我用一个 `python3 <<'PY'` heredoc 去解析冲突块。它的 `assert` 因为空白不匹配失败了,
而我用 `&&` 把它和 `git commit` 串在同一条命令里 —— **commit 照跑,把带冲突标记的
`INDEX.md` 提交了**。

也就是说:我在**修冲突标记**这个动作本身上犯了这个 dispatch 要防的那条规矩,
而且把一个静默失败变成了已提交产物。随后用 `edit` 修好,提交前 grep 全仓确认无标记
(`docs/ scripts/ ui/src/ crates/ src-tauri/ infra/` 全部为空)。

### 为什么这条比「注意一点」值钱

一般化的教训:**解决步骤和提交步骤不要串成一条命令。** 前者静默失败就会变成后者
的既成事实。已写进知识条目 `edit_code_with_edit_not_a_bash_heredoc`。

### 验证没省

- 判据全跑:`cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit` → **89 passed; 0 failed**
- 两个被接回来的件都在合并后的树里核过(`decisions.rs`、`020_operator_decisions.sql`、
  `DELETION_EXEMPTIONS` 7 处引用)
- 提交前 grep 全仓冲突标记,而不是假定上一步成功 —— 这正是这次学到的那条
