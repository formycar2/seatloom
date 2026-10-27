---
id: a_path_limited_merge_is_not_a_whole_branch_merge
title: 路径限定合并 ≠ 整分支覆盖:我差点删掉自己刚写的两节
curation: authored
confidence: quoted
source_kind: canonical_event
source_ref: AEGIS-2026-10-27-live-seat-mirror-cross-wiring-v1
created_at: 2026-10-27 20:10:00
tags: [git, merge, path-limited, self-inflicted, verification]
---

# 路径限定合并 ≠ 整分支覆盖:我差点删掉自己刚写的两节

2026-10-27 20:00。Lyra 的验收门类裁决在她的分支上,她的分支 10-09 分叉,
**落后 track 373 个提交**。裁决本身是对的、文档也带齐了 universal header,
我只要把它弄进 track。

## 我做的事

`cp her/ENGINEERING_STANDARDS.md track/ENGINEERING_STANDARDS.md`

理由是"反正只有两份文档要动,覆盖最快"。

## 覆盖删掉了什么

| 被删的节 | 谁写的 |
|---|---|
| `A dispatch is not done when the work is done` | Aegis,10-26/27 刚写,5 个守门条件全在里面 |
| `Tool use and token cost` | Aegis,10-21 的 token 调查结论 |
| `tmux pipe-pane is a stop command, not a query` | Aegis,**当天 19:00 刚写**,就是为了修这次串线 |

**同一天写的东西,当天差点被自己删掉。**

## 怎么发现的

不是靠 diff 预览(我没做),是覆盖完之后逐节 grep,发现三节不在了。
`git diff` 当时会显示 `85 insertions / 2 deletions` —— 数字小得可疑,
但"删除的行数少"不等于"没删东西":一个 30 节的章节被整段换掉,diff 里就是
几行。

## 判据(以后照这个做)

要把另一个分支上的改动拿进 track:

1. **先量分叉**:`git merge-base --is-ancestor track HEAD` 和
   `git rev-list --count HEAD..track`。落后超过几十个提交,就**没有**
   "直接覆盖"这个选项。
2. **只取要的那几个文件**,而且取之前先 `git diff --stat base...branch -- <文件>`,
   看清楚这个文件在两边各自改了什么。
3. **同一文件两边都改过 → 手工合并,以 track 为基底**,把对方的增量摘出来。
   这次的增量就是:一份新文档 + 一个新章节 + header 三行。
4. **合并后逐节 grep**:`grep -c '^## '` 数节数,再拿一份节名清单逐一确认。
   **只看 diff 统计数字会漏掉整节删除。**

## 更一般的形式

"两个文件"听起来像小改动,所以大脑会跳过检查。但文件的数量和改动的大小
没有关系:一份 373 个提交没同步的 `ENGINEERING_STANDARDS.md`,和一个
拼错的单词一样,都是一个文件。
