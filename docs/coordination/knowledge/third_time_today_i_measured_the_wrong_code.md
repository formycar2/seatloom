---
title: "Third time today I measured the wrong code and nearly reported it"
category: knowledge
topic: measurement-discipline
tags: [wrong-branch, wrong-worktree, false-conclusion, measure-the-object, recovery]
created: 2026-10-29T23:20:00+08:00
source_kind: canonical_event
source_ref: ev-two-cards-pending-and-one-had-a-now-fixed-defect
confidence: inferred
curation: authored
---

# 今天第三次:我测了错的代码,然后差点据它汇报

## 三次,同一天,同一个形状

| # | 我测了什么 | 实际是什么 | 我怎么发现的 |
|---|---|---|---|
| 1 | Mira 的新契约测试,在主检出跑 → `2 failed \| 8 passed` | 她的分支上是 **10 passed** | 汇报前一刻想起该在她的 worktree 跑 |
| 2 | "有缺陷的修复被合并了" | 那张卡绑定的是**另一个** verdict,干净 | 新检查拿那张卡去撞,**没命中**,那一秒知道搞错了 |
| 3 | "Nimbus 的修复丢失了" | 我站在 `nimbus` worktree,它在 `packet/nimbus/session-work-readmodel`;**正确分支是 `nimbus-psr`** | `git worktree add` 报 "already used by worktree at nimbus-psr" |

第三次最危险:我已經跑了一次变异测试、得出"clamp 没了"的结论、准备报告。
**只差一步就又是一次假警报。**

## 共同点

**我在一个名字对、内容错的地方测量,然后把结果当成那个分支的真相。**

`seatloom-worktrees/nimbus` 和 `seatloom-worktrees/nimbus-psr` 名字都像 Nimbus 的目录,
而它们在**不同的分支**上。我看目录名就进去了。

## 为什么这个错误特别容易犯

- 分支名和目录名不一致:`nimbus` 里是 `session-work-readmodel`,`nimbus-psr` 里才是 `project-scoped-reads`
- 卡上写的分支是 `packet/nimbus/project-scoped-reads`,我**有**这个信息,却没拿它去对
- `cd` 成功了 —— 目录存在、是 git 仓库、有 `crates/`、能跑 `cargo test`
- 测试跑了、有输出、有数字 —— **一切看起来都像成功**

## 规矩

### 1. 测量前先打印"我在测哪个分支",并和权威来源对一次

权威来源是卡上/派单上写的分支名,不是我猜的目录。

```bash
cd <worktree>
git branch --show-current          # 必须等于你要测的分支
git rev-parse HEAD
```

**不等就停。** 这一行能挡掉今天全部三次。

### 2. 目录名不是分支名

`worktrees/<name>` 的 `<name>` 是给人看的,**不是** git 分支。
Nimbus 有两个 worktree,一个装 `session-work-readmodel`,一个装 `project-scoped-reads`。
`git worktree list` 才是权威。

### 3. `git worktree add` 报 "already used by" 是救命的错误

它直接告诉我有另一个 worktree 已经检出了那个分支。
**如果我一开始就跑这条而不是 `cd` 进猜的目录,第三次就不会发生。**

### 4. "测出坏结果"时,先问"我测的是不是那个东西"

今天三次,每一次都是坏结果(测试红、修复丢、卡坏了)。
**坏结果让"我测错了"变得不可见** —— 因为错的测量也给出坏结果,看起来完全合理。

所以:**越是坏消息,越要先确认测量对象。** 好消息测错了会自我矛盾,坏消息不会。

### 5. 恢复现场,并证明恢复了

第三次我在别人分支上改过文件做变异。恢复后验了两件事:

```bash
grep -c 'Some(limit) => wanted.min(limit)' crates/.../pty/mod.rs   # → 1
git diff --stat                                                     # → 空
cargo test -p seatloom-core --lib                                   # → 215 passed / 0 failed
```

**在别人分支上做变异,必须留下可证明干净的现场。**

## 顺带:Nimbus 的修复是对的,我验过了

在正确的分支上:

```rust
pub fn clamp_backfill_to_pane(wanted: usize, pane_limit: Option<usize>) -> usize {
    match pane_limit {
        Some(limit) => wanted.min(limit),
        None => wanted,
    }
}
```

`Some` 才 clamp,`None` 原样返回 —— **既不猜一个数,也不假装 0**。
这正是 Flux 要求的两件事(钉住内联 clamp、区分 `None` 与 `Some(0)`)。

变异实测:把 body 换成 `let _ = pane_limit; wanted`,
`the_backfill_never_asks_for_more_than_the_pane_retains` 和
`an_unknown_limit_is_not_the_same_as_a_limit_of_zero` **双双变红**。
测试现在真的钉住它了。

**所以那张卡可以点。** 但它的 verdict 是修复前发的,账本里仍带着那两个缺陷的名字 ——
一个 verdict 记录它发出那一刻的状态,不会因为后来改好了而自己更新。
这个区别我已告诉张先生,并给他"要一个干净 verdict 再决定"的选项。
