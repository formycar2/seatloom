# disp-Q_lJSK1MycMin948ghJzI 交付说明 —— 开卡被守卫拦下,派单指错了对象

**派单**:`disp-Q_lJSK1MycMin948ghJzI`(Aegis 发)
**提交**:无代码改动(仅本文件)

---

## 1. 我没有开出卡,因为派单让我开的那个 dispatch 不是我的

派单 `do` 第一条:

> Open an acceptance card for **`disp-D0nfroMNycvuWNwWOv3nq`** with
> `flux_verdict_ref=ev-EUiQEB8IvQb3nxruwvI-k`.

我查了这个 dispatch:

```
id           = disp-D0nfroMNycvuWNwWOv3nq
status       = done
targetSeatId = seat-flux-001        ← Flux 的
issuedBy     = seat:seat-aegis-001
title        = 验:默认落地页被持久化覆盖 —— 那张卡的断言是假的(disp-E06O6IAaWefF-BOW-CNJd)
```

**它是 Flux 的验单**,验的是我那个修复。它的 verdict
(`ev-EUiQEB8IvQb3nxruwvI-k`)也确实点名它自己。

**我不能给 Flux 的派单开卡。** 开出来的卡会挂在别人的派单上。

## 2. 我的实现派单是另一个,而它没有 verdict 点名它

我的实现派单是 `disp-E06O6IAaWefF-BOW-CNJd`(我上一单做的第一屏默认页修复)。
逐条查 verdict:

```
disp-E06O6IAaWefF-BOW-CNJd  →  0 条 verdict 点名它
```

所以:

- 给 **Flux 的验单** 开卡 → 越界(别人的派单)
- 给 **我的实现派单** 开卡,verdict ref 用 `ev-EUiQEB8IvQb3nxruwvI-k` → 守卫拒绝(见 §3)

**两条路都不通,而这正是该报告的状态。**

## 3. 我试了第二条,守卫拦下了,而且拦得对

我还是实测了一次(没有只凭推理下结论):

```js
request_acceptance({
  dispatch_id: 'disp-E06O6IAaWefF-BOW-CNJd',
  flux_verdict_ref: 'ev-EUiQEB8IvQb3nxruwvI-k',
  ...
})
```

工具返回:

> request_acceptance rejected: the verdict "ev-EUiQEB8IvQb3nxruwvI-k" is not
> about "disp-E06O6IAaWefF-BOW-CNJd" — it is about "disp-D0nfroMNycvuWNwWOv3nq".
> **Another unit of work's verdict cannot stand in for this one's; ask Flux to
> verify THIS dispatch.**

`isError=true`,**没有任何卡被创建**。我不需要像前几轮那样撤卡。

**这一条比前几轮好。** 我前两轮正是这么错的(照着派单指定的 verdict 开卡,
开出坏卡,被 Aegis 的 `check-verdict-refs.sh` 事后抓到,再撤重开)。
现在守卫在**开卡之前**就拦住了,而且错误消息直接说了该怎么做。

## 4. 派单为什么指错

我猜的原因是:**Aegis 把"验单的 dispatch_id"当成了"被验的实现的 dispatch_id"。**

它在派单里写 `your_dispatch: disp-D0nfroMNycvuWNwWOv3nq`,而那一条的
`targetSeatId` 是 `seat-flux-001`。它自己的心跳刚补了"verdict 必须点名被接受的
派单"这一步,但**在派单里指定 dispatch id 时,它选的是验单而不是实现单**。

这不是挑剔:**它把我该开卡的对象指成了别人的派单。** 我如果照做,会开出一张
挂在 Flux 名下的卡。

## 5. 我名下真正"有 pass verdict 但无卡"的

逐条查了我名下近 10 个 done 派单:

| 派单 | 有 pass verdict 点名它? | 卡 |
|---|---|---|
| `disp-session-key-ui` | ✅ 1 条 | 已开(已 accepted) |
| `disp-first-screen-default-tab` | ✅ 1 条 | 已开(pending) |
| `disp-blocked-on-whom-ui` | ✅ 1 条 | 已开(pending) |
| `disp-E06O6IAaWefF-BOW-CNJd` | ❌ **0 条** | 无 —— **就是这一件** |
| `disp-provenance-badge-honest` | ❌ 0 条 | 无 |
| `disp-KFwsxEkoV9FetcHRW9Z3M` | ❌ 0 条 | 无 |
| `disp-8PGCP6KH0TMPzp6-0h0nR` | ❌ 0 条 | 无 |
| `disp-eXemxrJCG3rEzECW18YU-` | ❌ 0 条 | 无 |
| `disp-wL0t1amBBqYeqTv-bwelk` | ❌ 0 条 | 无 |
| `disp-cxCBOxrvKX0qcoON2eg_r` | ❌ 0 条 | 无 |

**`disp-E06O6IAaWefF-BOW-CNJd` 是唯一"我刚做完、有独立验绿(Flux 验过)、
但没有一条 verdict 点名它"的。** 其余几条是同步类/修复类派单,本就不该有卡。

## 6. 我给 Aegis 的两句话

1. **`disp-D0nfroMNycvuWNwWOv3nq` 是 Flux 的验单,不是我的实现单。**
   我的实现单是 `disp-E06O6IAaWefF-BOW-CNJd`。
2. **要开这张卡,需要 Flux 补发一条 `payload.dispatch_id = disp-E06O6IAaWefF-BOW-CNJd`
   的 verdict。** 它已经验过这个修复(它在验单里写了验的什么),只是判词记在
   验单自己名下。**这是补数据,不是重新验。**

verdict 一落地,我立刻开卡,`how_to_verify` 我会按派单要求的写:
操作员在运行的应用里做、每一步能因为我的改动而失败、有明确拒绝条件 ——
具体就是"完全退出重开,看左边栏高亮哪一页",以及"会话内切项目,确认那个项目
还记得自己的页"(后者是**不许坏**的那一条)。

## 7. 未做的事

- **没有开任何卡。**
- **没有 merge**(派单 `do_not`)。
- 零代码改动,未跑门禁。
