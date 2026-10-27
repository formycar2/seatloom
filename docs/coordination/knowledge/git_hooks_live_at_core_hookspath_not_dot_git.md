---
memory_id: git_hooks_live_at_core_hookspath_not_dot_git
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: fact
confidence: inferred
status: active
tier: warm
source_kind: file
source_ref: saih/pi/.husky/pre-commit
l2_ref_kind: file
l2_file_path: saih/pi/.husky/pre-commit
l2_line_start: 1
l2_line_end: 15
curation: authored
l0_summary: git 的钩子不一定在 .git/hooks/。husky 用 core.hooksPath 指到 .husky/,只 ls .git/hooks 会得到"只有样例"的假阴性 —— 然后钩子照样拦提交。查钩子先跑 git config core.hooksPath。
---

## 发生了什么

在 saih/pi clone 提交 rebrand 前,用 `ls .git/hooks/ | grep -v '\.sample$'` 查钩子,
输出"只有样例",据此判断无钩子。随后 `git commit` 被 husky 的 pre-commit 拦了两次:
第一次拦 package-lock.json(要 `PI_ALLOW_LOCKFILE_CHANGE=1` 或先审查),
第二次拦 install-lock 过期(要重新生成)。钩子一直在,只是住在 `.husky/pre-commit`
—— `git config core.hooksPath` 的值是 `.husky/_`。

## 为什么会改变未来动作

- 查"这个仓库有没有钩子"用 `git config core.hooksPath`,没有配置才默认 `.git/hooks`。
- 被 hook 拦下不要条件反射 `--no-verify`:hook 的每个要求(审查 lockfile、重新生成
  install-lock、跑格式化)都是真检查,照做再提交。这次照做之后 tree 是干净的,
  而且 hook 自带的 `tsc --noEmit` 全绿还额外证明了 rebrand 后源码类型没问题。
- 和 AGENTS.md「每一项检查都必须能报错」同向:我的钩子检查报了假阴性,
  比没有检查更坏。

## 证据

- `git config core.hooksPath` → `.husky/_`;`.husky/pre-commit` 第 6 行
  `node scripts/check-lockfile-commit.mjs`,第 13 行 `npm run check`。
- `ls .git/hooks/ | grep -v '\.sample$'` → 空(假阴性)。
- 两次 commit 被拦的输出:lockfile 审查清单(85+ 依赖漂移)、install-lock out of date。
