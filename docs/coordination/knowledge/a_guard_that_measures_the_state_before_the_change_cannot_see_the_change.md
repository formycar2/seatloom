---
id: a_guard_that_measures_the_state_before_the_change_cannot_see_the_change
title: 一个只测"改动之前"状态的守卫,看不见它要防的那个改动
curation: authored
confidence: inferred
status: active
created_at: 2026-10-28
source_kind: canonical_event
source_ref: AEGIS-2026-10-28-release-three-stalled-gates-v1
tags: [pre-commit-hook, guard, measurement, self-inflicted, acceptance-gate]
---

# 一个只测"改动之前"状态的守卫,看不见它要防的那个改动

同一天第三次。2026-10-28 12:33。

前两次我把"开卡后不推 track"写成了散文规则,靠记住执行,破了两次。
第三次我把它做成了 `.git/hooks/pre-commit`,觉得这回机械了。

钩子跑起来,输出:

```
pre-commit: pending acceptance cards exist. Checking that they stay clickable...
pre-commit: OK - pending cards remain clickable
```

**然后它放行的那个提交,把 12 张卡卡住了。**

## 为什么

钩子里写的是:

```bash
if bash scripts/check-cards-clickable.sh --quiet; then
  echo "pre-commit: OK - pending cards remain clickable"
```

`check-cards-clickable.sh` 测的是**track 当前的 tip**。跑钩子的那一刻,
三个分支都包含当前 tip,所以它说 OK。

**而这个提交本身就会成为新的 tip,而那个新 tip 没有任何分支包含。**

守卫测的是"改动之前"的世界,而它要防的恰好是"改动"本身。

## 这比没有守卫更糟

没有守卫,我不会收到任何信号,会一直疑心。
有了守卫,我收到一个**明确的 OK**,于是我确信了 —— 然后照犯。

一个在犯错当场打印"没问题"的检查,比没有检查更有害,
因为它把"我没查"变成了"我查过了"。

## 修法

钩子改用 `git commit-tree` 造出**那个即将存在的提交**
(不移动任何 ref、不 push、不动工作区),然后拿每个 pending 卡去比**那个 tip**:

```bash
FUTURE=$(git commit-tree "$TREE" -p HEAD -m "pre-commit probe")
git merge-base --is-ancestor "$FUTURE" "$br"   # 每张 pending 卡
```

验证过它现在会拒:拿一个探测提交去试,它拒了,而且拒的理由是对的。

## 更广的一条

**凡是"防某个改动"的检查,都必须拿改动之后的世界去测。**

- 测"提交前分支同步吗" → 没用,提交是产生不同步的原因
- 测"发布后 live 能用吗" → 没用,发布是产生不可用的原因
- 测"合并前测试过吗" → 没用,合并可能引入新的失败

判据一句话:**这个检查测量的是它要防的那件事发生之前,还是之后?**
如果是之前,它测的是一个和结论无关的世界。
