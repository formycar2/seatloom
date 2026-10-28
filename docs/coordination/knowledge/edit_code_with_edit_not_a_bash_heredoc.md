---
memory_id: edit_code_with_edit_not_a_bash_heredoc
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: decision
confidence: quoted
status: active
tier: hot
source_kind: canonical_event
source_ref: disp-lVJKIg1QtRSIlhJ-FlDou
l2_ref_kind: file
l2_file_path: docs/coordination/ENGINEERING_STANDARDS.md
l2_line_start: 158
l2_line_end: 172
curation: authored
l0_summary: token 大头是席位把代码塞进 `python3 - <<'EOF'` 改文件:占 toolCall 的 58%,且同一份代码付两次(一次当命令参数,一次之后 read/diff 再看回来)。改代码走 `edit`(只要 old+new 片段、自带 diff、可回退)。toolCall 占全队 token 36%,高于 thinking 的 17%。已入 ENGINEERING_STANDARDS。
---

## 实测(全队 session 拆开数的)

| 项 | 占比 |
|---|---|
| toolCall | **36%**(高于 thinking 的 17%) |
| bash | 占 toolCall 的一半 |
| bash 里 heredoc 包代码 | **58%** |

heredoc 的形式是 `python3 - <<'EOF' ... EOF`,用来改文件。

**同一份代码付两次**:一次当命令参数发出去,一次之后 read/diff/上下文里再看回来。

`edit` 只要改动的 `old` + `new` 片段,自带 diff、可回退、可 dry-run,省掉第一笔全额。

## 这是习惯,不是天性

实测里 vela 的 heredoc 只占它改代码的 **14%**(最低、`edit` 用得最多),
mira 61%、nimbus 45%。**从没人给过这条指令** —— 所以这是补一条规矩,不是说谁做得差。

## 三条

1. **改代码用 `edit`,不用 bash heredoc 包代码。**
   heredoc 只留给:测试夹具生成、一次性数据处理、非代码文件。
2. **不写 `cd <绝对路径> &&`。** bash 工具本身就在自己 worktree 里起,cwd 本就对。
3. 多处改动时一次 `edit` 一处,不要合成一个巨型 heredoc。

## 和上一单的关系(不许互相推翻)

上一单 `disp-5Wwwsq00WJUfPDAUV5t--`(见
[seat_tool_calls_must_not_restate_cwd_or_reread_files](seat_tool_calls_must_not_restate_cwd_or_reread_files.md))
瞄准的是「别重复 cd、别重读同一文件」——**那两条仍然对**,但那是小鱼
(`cd` 只占 bash 的 8.1%)。真正的大鱼是 heredoc(58%)。**这一单是补上,不是推翻。**

## 不许碰的那条

**不为省 token 跳过验证。** 判据仍要重跑 —— 那是治理,不是浪费。
同理不许把测试改弱或改恒真。

## 一条从这次派生的教训(比规矩本身更该记住)

执行这一单时,我用一个 `python3 <<'PY'` heredoc 去解析冲突块,它的 `assert` 因为空白
不匹配失败了;而我用 `&&` 把它和 `git commit` 串在同一条命令里,**于是 commit 照跑,
把带冲突标记的文件提交了**。

两件事:

- 那天我在**修冲突标记**的动作本身就是一个 heredoc —— 正好犯着这条规矩要防的事,
  而且因此把一个静默失败变成了已提交产物。
- 更一般的教训:**解决步骤和提交步骤不要串成一条命令。** 前者静默失败就会变成后者
  的既成事实。提交前应当先核一次(这次是 grep 全仓冲突标记),而不是假定上一步成功了。
