---
id: git_merge_ab_form_merged_unverified_work_into_track
kind: failure
confidence: inferred
source_kind: canonical_event
source_ref: ev-JYTZGEWl7FO55no_UVbap
seat_id: seat-aegis-001
curation: authored
date: 2026-10-16
---

# `git merge A B` 的写法把未验的活直接合进了主分支

## 发生了什么

我想造一条「两边都有」的集成分支( Mira 的界面修复 + 主分支上已接受的四个修复),
写的是:

```sh
git merge --no-edit "$B" track/infra-foundation
```

而当前分支**正是** `track/infra-foundation`。

`git merge A B` 是八爪鱼合并 —— 它把 A 和 B **一起合进当前分支**。所以她的 8 个提交
直接进了主分支:**没有 Flux 验绿、没有验收卡、没等张先生点。**

## 同一次里我还先冤枉了她

查她的活时,我去 grep **主分支**。而席位从不自己合并是规矩,她的活本来就不该在主分支上。
于是得出「她什么都没做」的错误结论,还准备据此处理她。

她的提交 `cc0e409` / `f316201` 一直在她分支上,一直都在。

## 怎么发现的

合并后我核 `git merge-base --is-ancestor f81e913 <集成分支>`,返回「不带着」,
回头才看到主分支头变成了 `80f381a Merge branch 'packet/mira/...'` ——
**合并目标不是我要的那个分支。**

## 修法

```sh
git branch -f track-backup-bad-merge-80f381a 80f381a   # 先存备份,不删任何东西
git reset --hard f81e913                               # 主分支退回四个修复那个合并
```

她的分支自始至终没被碰过,仍在 `cc0e409`。备份分支留着,她的 8 个提交一个没丢。

## 三条通用教训

1. **要造一条新分支时,先 `git switch` 过去,再 merge。**
   `git merge A B` 几乎从来不是你想要的形式。想要「A 合进 B」应该是
   `git switch B && git merge A`。

2. **任何动主分支头的命令,执行后第一件事是 `git log --oneline -1 <主分支>`,
   看头还在不在原地。** 这次我隔了好几步 —— 装完应用、查完二进制哈希 —— 才发现。

3. **判断某个席位做没做一件事,查它自己的分支/工作区,不是查主分支。**
   席位不自己合并是规矩,所以主分支上没有,是正常,**不是它没做**。
   这条和 [[aegis_check_the_right_place_before_concluding]] 是同一族:
   查错了地方就下结论,比不查更糟,因为它带着"我查过了"的可信外观。

## 相关

- 这次差点造成的伤害:把未验的界面改动当成已接受的推到张先生眼前,
  而且是在一次"为了让你看到修复"的善意动作里。善意不改变它的性质。
- `remember` 工具当时被库拒(话题不存在,建话题是写库、我被权限封顶),
  所以这条按项目规矩写在 `docs/coordination/knowledge/`,等写入路径打通可直接灌库。
