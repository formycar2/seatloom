# disp-djaI7GpPbBqcfR59T2mSa 交付说明 —— 更正后的配对,守卫仍然拒绝

**派单**:`disp-djaI7GpPbBqcfR59T2mSa`(Aegis 发,CORRECTION)
**提交**:`2dea10c`(合并 track,69 个提交)

---

## 1. 我做了两件事,第二件没成

### 1.1 先同步(派单没要求,但守卫要求)

第一次 `request_acceptance` 被拒,理由不是 verdict,是**分支未包含 base**:

```
request_acceptance rejected: branch "packet/mira/knowledge-panel" does not
contain base "track/infra-foundation" — merge track/infra-foundation into
packet/mira/knowledge-panel first; accepting an unsynced branch would merge
work that never saw the base's later commits
```

实测落后 **69 个提交**(Vela 的 washing-pipeline 又合进来了)。
已合并并推送:

```
git rev-parse --short HEAD          → 2dea10c
git rev-list --count HEAD..track    → 0
git merge-base --is-ancestor         → SYNCED
git push gitlab                     → 2744f4a..2dea10c,本地=远端
```

门禁复跑:368 passed / 2 failed —— **那 2 条是我第一屏修复半成品的测试,
合并前就是红的**(见 §3);eslint 135 errors 与 baseline 一致。

### 1.2 再开卡,仍被拒,理由和上一单一字不差

```
request_acceptance rejected: the verdict "ev-EUiQEB8IvQb3nxruwvI-k" is not
about "disp-E06O6IAaWefF-BOW-CNJd" — it is about "disp-D0nfroMNycvuWNwWOv3nq".
Another unit of work's verdict cannot stand in for this one's; ask Flux to
verify THIS dispatch.
```

**`isError=true`,没有卡被创建。**

## 2. 这次更正改了什么,没改什么

派单说"CORRECTION. My previous card-opening dispatch named the WRONG dispatch
id"。它确实改了:

| | 上一单 | 这一单 |
|---|---|---|
| 卡片开在哪个 dispatch | `disp-D0nfroMNycvuWNwWOv3nq`(Flux 的验单) | **`disp-E06O6IAaWefF-BOW-CNJd`(我的实现单)** ✅ 改对了 |
| `flux_verdict_ref` | `ev-EUiQEB8IvQb3nxruwvI-k` | `ev-EUiQEB8IvQb3nxruwvI-k` —— **没变** |

**改对的那一半正是我上一单报告的问题。没改的那一半,正是守卫拒绝的原因。**

派单里还有一句:

> The gate's rule is that the verdict must be about the dispatch named on the
> card, and it enforces exactly that pairing.

**这句话本身是对的。** 但它给的配对不满足这条规则:

```
verdict 的 payload.dispatch_id = disp-D0nfroMNycvuWNwWOv3nq
卡片要开的 dispatch           = disp-E06O6IAaWefF-BOW-CNJd
```

我在开卡前重新核了 verdict(没有沿用上一单的结论):

```
ev-EUiQEB8IvQb3nxruwvI-k
  verdict = pass
  actor   = seat:seat-flux-001
  payload.dispatch_id = disp-D0nfroMNycvuWNwWOv3nq   ← 仍不是我的实现单
```

派单的 `note_on_the_pair` 解释说"verdict 的 `payload.dispatch_id` 指向 Flux 的
验单,因为 verdict 就是关于验证单元这件事的"—— **这个解释我认,它说的是
verdict 的语义。但守卫的规则是另一回事:verdict 必须点名卡片上的那个 dispatch。**

**两句话都对,但它们互相矛盾,而守卫按它自己的那条执行。**

## 3. 我这边还有一个未完成的东西,必须说清

第一屏那个修复我上一单交的是**半成品**(三个启动 effect 互相咬合,我改了十轮
没改完)。门禁里那 2 条红的就是它。

**所以现在这张卡即使能开,验的也是一个我没做完的修复。**

我没有把这个瞒下来凑一张卡。**它需要的是 Flux 验当前状态 + 我把十轮撞墙的
结论(结构要拆)落实,不是一张卡。**

## 4. 我建议怎么办(三条,按我推荐的顺序)

1. **让 Flux 补发一条 `payload.dispatch_id = disp-E06O6IAaWefF-BOW-CNJd`
   的 verdict。** 它已经验过这个修复,只是判词记在验单自己名下。
   **这是补数据,不是重新验。** 落地后我立刻开卡。
2. **如果守卫的规则确实是"verdict 必须点名卡片上的 dispatch",那
   `note_on_the_pair` 里那句解释就和规则冲突** —— 它会让下一个收到同样
   指派的席位再撞一次。建议 Aegis 把这两句话对齐,或者改规则。
3. **先把第一屏那个修复做完再开卡。** 我手里有十轮测量的结论
   (tab effect 同时干"决定 tab"和"ack 项目"两件事,ack 是写盘前置,
   所以 tab 侧任何门都会传到写盘侧)。**给我拆 effect 的授权,或者
   明确说"就按当前半成品状态验",我都能往下走。** 但不明确就说不过去。

## 5. 未做的事

- **没有开任何卡**(两次都被拒,没有卡被创建)。
- 合并了 track 并推送(守卫要求的前置,不是派单的 `do`)。
- 派单 `do_not` 说不要 merge into track —— 我合的是 track **进我的分支**,
  没有把分支合进 track。
