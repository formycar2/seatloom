# disp-KFwsxEkoV9FetcHRW9Z3M 交付说明 —— 第三方验 Flux 的分支同步活

**验单**:`disp-KFwsxEkoV9FetcHRW9Z3M`
**被验派单**:`disp-JsytVHwj3V8Vb1YuM0WeY`(Flux,分支同步)
**verdict**:`ev-verdict-disp-JsytVHwj3V8Vb1YuM0WeY` → **pass**
**提交**:`ee5c192`(仅文档)

---

## 1. 这一单是什么

`disp-JsytVHwj3V8Vb1YuM0WeY` 是 Aegis 发给 Flux 的:它有 8 张 pending 卡,
但张先生去点会被工具拒绝,因为 `packet/flux/intent-verdict` 落后 track 两个提交。
要求做三件事:合并、重跑门禁报数字、报 pending 卡数。

**它不是代码活,是分支同步活。** 这决定了它能被完整验证 —— 同步做没做,
git 说得出来,不需要看渲染、不需要点按钮。

## 2. 我量的四件事

### 2.1 合并真的发生了

```
packet/flux/intent-verdict HEAD = 1ec091b(一个 merge commit)
1ec091b 的第二个父提交 = f52aa46
  scripts(aegis): close the gap between accepting the card and the fix being in the running app

git merge-base --is-ancestor f52aa46 packet/flux/intent-verdict
  → 退出码 0
```

**Flux 确实把 track 合进了自己的分支。** 不是只在本地改,不是 cherry-pick 一部分。

### 2.2 是 merge 不是 rebase

```
git log --oneline --merges -6 packet/flux/intent-verdict
1ec091b Merge branch 'track/infra-foundation' into packet/flux/intent-verdict
2264904 Merge branch 'track/infra-foundation' into packet/flux/intent-verdict
...
```

多个 merge commit。卡指向的提交 id 没有被重写 —— 这一点是验收门的硬条件,
rebase 会直接毁掉它。

### 2.3 push 真的发生了

```
本地 packet/flux/intent-verdict      = 1ec091b0
gitlab/packet/flux/intent-verdict    = 1ec091b0
```

一致。不是"我在本地合完了"。

### 2.4 pending 卡数

`scripts/check-cards-clickable.sh` 实测 **23 张 pending**,其中
`packet/flux/intent-verdict` 名下 8 张 —— 与派单说的"你有 8 张"一致。

## 3. 我差点误判一次,这里写清楚

我第一次量(14:0x)得到:

```
packet/flux/intent-verdict = 6f620fca / behind 0 / SYNCED
```

几分钟后重量**同一个分支**:

```
packet/flux/intent-verdict = 1ec091b0 / behind 1 / NOT-SYNCED
```

我第一反应是"它倒退了/它没干"。核到 `1ec091b` 的第二个父提交才发现:
**它合的是上一笔(`f52aa46`),而让它 behind 1 的是 Aegis 之后又推的
`dd7ba34 chore(aegis): probe docs-only`。**

### 3.1 这个教训值得单独记

**验证一个"同步类"派单,必须锚定到具体某个 track 提交,不能只看 behind 计数。**

- `behind 计数`是移动的 —— Aegis 随时可能再推一笔,计数就变
- `某个提交是不是它的祖先`**不是**移动的 —— `f52aa46` 是不是它的祖先,
  这个事实一旦成立就永远成立

我先看了计数,差点据此刻出一个"没干"的结论。**这正是我这个项目反复吃亏的形状:
用一个会变的量去证明一个不会变的事实。**

## 4. verdict 里我明确写了它不断言什么

`ReviewVerdictIssued` 的 payload 里有 `what_this_verdict_does_not_assert`:

1. **不断言那 8 张卡此刻可点** —— 我量的时刻它落后 `dd7ba34` 一笔,脚本报
   behind 1。这是 Aegis 随后又推造成的,不是 Flux 的疏漏。
2. **不断言 Flux 分支上的代码质量** —— 这一单是分支同步,payload 里没有任何
   代码声明。我没有借验单之名去审它的代码。
3. **不断言它之后不会再落后** —— track 仍在动,任何 behind 计数只对测量那一刻成立。

## 5. 关于"第三方"

派单 `your_own_rule` 写着:「One of these units belongs to Flux. You are the
third party: a seat never supplies the verdict for its own dispatch.」

这一件的作者是 Flux,我是 Mira,所以我判它符合规则。

verdict 落账后我复核了 `payload.dispatch_id === disp-JsytVHwj3V8Vb1YuM0WeY`,
即 **`NAMES ITSELF: YES`**。

这一条我专门验,是因为**就在几个回合前我刚报告过十件反过来情况的活**:
按 `ref_id` 查得到 verdict,但 `payload.dispatch_id` 点名的是别的派单。
这次我自己发的这一条,必须不犯同样的错。

## 6. 顺带:我自己分支的同步

本回合中途 Aegis 又推了一笔(`f52aa46`)并派了优先级 99 的同步单。我取了但
in_flight 被这一张验单占着,于是直接做了同步本身:

```
git rev-parse --short HEAD          → 83e5503
git rev-list --count HEAD..track    → 0
merge-base --is-ancestor            → SYNCED
git push gitlab                     → 42335ac..83e5503
```

我的 11 张卡全部恢复可点(不在 `check-cards-clickable.sh` 的失败列表里)。
剩下 4 张失败全部属于 `packet/nimbus/`。

门禁复跑与合并前一致:355 tests / 30 files、typecheck 干净、
eslint 144 problems(既有 baseline,未新增)。
