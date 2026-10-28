# NIMBUS-2026-10-22-nimbus-open-cards-v1 — 交付说明

派单 `disp-JW_LVoriQkcFy56VfkrJd`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-22-nimbus-open-cards-v1.md`。
分支 `packet/nimbus/project-scoped-reads`。

## 两件活的状态(核过,不是照抄)

| 活 | 派单 | 实现提交 | 验绿 |
|---|---|---|---|
| 删除豁免清单 | `disp-5WQ7UeLjxhEd6j8PsuOxL` | `6fc6b4a` | `ev-4fKDMZztUVkasOWvCmxfn` |
| 操作者待决对象 | `disp-dwSmYkULRU4HdSDJCDWH9` | `a545a77` | `ev-qGwjjmysUqELKo2haQP5c` |

## 两件事必须先说

### 1. flux_verdict_ref 挑错就会互借,所以我逐个查了

任务包警告过「有两 pass,以挂在现役派单上且确认了正确分支那条为准」。查了,确实有两条:

```
ev-4fKDMZztUVkasOWvCmxfn  09:59:54  verification=disp-tJEX9HvR38W0q6xL88KFH  branch=packet/nimbus/project-scoped-reads
ev-kZ5x8eixmJZYrzbj7GRk-  09:58:10  verification=disp-c6WJXKD50eWHSRJJ7sWc2  branch=(未记录)
```

差别就在**有没有确认分支**:后者 `slices_under_test.branch` 是空的,没有证明它验的是哪个分支。
前者点名了 `packet/nimbus/project-scoped-reads`。任务包说的「确认了正确分支那条」就是前者,
所以卡一用 `ev-4fKDMZztUVkasOWvCmxfn`。**不是选新的那条,是选证明了分支的那条。**

待决对象那条只有一条(`ev-qGwjjmysUqELKo2haQP5c`),且同样点名了分支,无歧义。

### 2. 界面其实已经接上了,但接上的那一刻它还不能用

任务包说「界面那一层还没接……卡上要如实写后端与工具已就绪,界面未接」,并且说
「如果界面其实已经接上了,把接在哪一屏写进卡里,我核漏了我改」。

**核了,接上了**:`ui/src/views/DispatchInboxView.tsx:1258` 挂着
`<OperatorDecisionSection live={autoRefresh} />`,在「待验收」下方。API、DTO、三个 Tauri
命令都在,5 个 UI 测试在。

**但张先生现在点开应用,那个区会报错,不是空的。** 因为 020 迁移还没 live 应用 ——
正式库里 `operator_decisions` 表不存在(实测 `pg_tables` 计数 0)。所以卡二的
`how_to_verify` 和 preconditions 里都写明了这一点:界面已接,但迁移未应用时看到的是
报错而非空列表,这是预期的,不是界面没接。

没有为了好看把它写成「已经能用」。

### 3. 为了能开卡,先同步了一次分支

第一次 `request_acceptance` 被工具拒了:

```
branch "packet/nimbus/project-scoped-reads" does not contain base "track/infra-foundation"
```

原因:track 又前进了 2 个提交(都是 Aegis 的文档提交),我的分支不含它们。
任务包里「分支已同步」的状态已经过期了。

按上次同步单(`disp-N_FGAclg1Ml-KME_GJY6S`)立的新判据先核区间 ——
**纯 `docs/coordination/` 三个文件,没有任何 `ui/src` 或 `src-tauri` 改动**,干净,所以同步:

```
$ git merge --no-edit track/infra-foundation
$ git merge-base --is-ancestor track/infra-foundation HEAD && echo synced
synced
```

合并提交 `c89fa5e`。合并后逐一核过自己那两件都还在(020 四个文件、`DELETION_EXEMPTIONS`
7 处引用、UI 挂载 2 处)。

## 验收标准逐条

- **两张卡各自 `dispatch_id` 与 `branch` 对得上** —— 是,逐条读过 lineage,见下。
- **每张的 `flux_verdict_ref` 点名它自己的实现单,不许互借** —— 是。
  卡一点名 `disp-5WQ7UeLjxhEd6j8PsuOxL` 的验绿,卡二点名 `disp-dwSmYkULRU4HdSDJCDWH9` 的验绿。
  两条 verdict 的 refs 里都挂着各自的 dispatch id,没有交叉。
- **`how_to_verify` 是「做什么、看到什么」** —— 是。卡一写的是跑哪条命令、看哪两个测试、
  断言什么(清单外的删除仍被拒、`keep.txt` 出现在错误里);卡二写的是打开派单页哪个区看到什么。
- **待决对象那张如实标注界面是否已接** —— 是,见上。
- **不许为了好过把验收条件写含糊** —— 卡二的 preconditions 里把「迁移未 live 应用」写成了
  一条前置条件,而不是藏起来。验收的人会因此知道界面上那个报错是预期的。

## 两张卡的落库状态(读回来核过)

```
acc-disp-5WQ7UeLjxhEd6j8PsuOxL
  dispatch=disp-5WQ7UeLjxhEd6j8PsuOxL (status=done)
  branch=packet/nimbus/project-scoped-reads  base=track/infra-foundation
  fluxVerdictRef=ev-4fKDMZztUVkasOWvCmxfn  status=pending
  plan: steps=2  expected=3  preconditions=2

acc-disp-dwSmYkULRU4HdSDJCDWH9
  dispatch=disp-dwSmYkULRU4HdSDJCDWH9 (status=done)
  branch=packet/nimbus/project-scoped-reads  base=track/infra-foundation
  fluxVerdictRef=ev-qGwjjmysUqELKo2haQP5c  status=pending
  plan: steps=4  expected=6  preconditions=3
```

## 明确不做的,确认没做

- 没合并自己的分支进 track(那是张先生点接受)。
- 没改代码 —— 这次只读库、开卡、同步分支。两件的实现一个字节没动。
- 没把两件合成一张大卡。
- 截图 token 修复单(`disp-CJsvktofZ7hkdsESlJQ3D`)**没并进这两张卡**。它在我之前一单已完成并
  单独交付,交付说明在 `docs/coordination/tasks/nimbus/NIMBUS-2026-10-22-image-token-estimation-v1.md`。

## 提交

`docs(nimbus): open cards for the two verified deliveries`(绝对哈希见 `git log --oneline -1`)。
`git status` 干净。
