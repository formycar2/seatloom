# 改代码用 edit,不用 bash heredoc —— 这是 token 大头

| Field | Value |
|---|---|
| id | AEGIS-2026-10-22-use-edit-not-heredoc-v1 |
| issued_by | seat-aegis-001 |
| date | 2026-10-22 |
| 来源 | AEGIS-2026-10-21-seat-token-hotspots-v1 实测 |
| 已入标准 | `docs/coordination/ENGINEERING_STANDARDS.md` 的「Tool use and token cost」 |

## 为什么(实测,不是印象)

全队 token 里 **toolCall 占 36%,高于 thinking 的 17%**。bash 占 toolCall 一半,
bash 里 **58% 是把代码塞进 `python3 - <<'EOF'` 改文件**。

**同一份代码付两次**:一次当命令参数发出去,一次之后 read/diff/上下文里再看回来。

`edit` 只要改动的 `old`+`new` 片段,自带 diff、可回退、可 dry-run。走 edit 省掉第一笔全额。

## 三条(照做)

1. **改代码用 `edit`,不用 bash heredoc 包代码。**
   heredoc 只留给:测试夹具生成、一次性数据处理、非代码文件。
2. **不写 `cd <绝对路径> &&`。** 你的 bash 已经在自己 worktree 里起,cwd 本就对。
3. 需要多处改动时,一次 `edit` 一处,不要合成一个巨型 heredoc。

## 关于你之前那张 token 卫生单

Aegis 上一张(`disp-5Wwwsq00WJUfPDAUV5t--`)瞄准的是「别重复 cd、别重读文件」——
那两条仍然对(继续照做),**但那是小鱼(cd 只占 bash 的 8.1%)**。
真正的大鱼是 heredoc(58%),上一单漏了它。**这一单补上,不是让你把上一单推翻。**

## 不是针对谁的风格

实测里 vela 的 heredoc 只占它改代码的 14%(最低、edit 用得最多),
mira 61%、nimbus 45%。**这说明是习惯不是天性**,而且从没人给过这条指令。
所以这一单是补一条从没给过的规矩,不是说谁做得差。

## 验收

- 下一件改代码的活里,`edit`/`write` 覆盖代码改动,`bash heredoc` 只出现在
  夹具/数据处理这类它擅长的场合
- 交付说明里能给出自己的计数(例如 grep 你的 session 里 heredoc 出现次数),
  说出「这件的代码改动走的是 edit」

## 明确不做

- 不为省 token 跳过验证(判据仍要重跑 —— 那是治理,不是浪费)
- 不把测试改弱、不改成恒真
- 不做 edit 之外的任何工具替换

## 交付

在已有交付说明里追加一句:这件的代码改动用了什么工具、为什么。
