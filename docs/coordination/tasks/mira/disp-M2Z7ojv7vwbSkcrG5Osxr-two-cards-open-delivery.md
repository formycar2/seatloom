# disp-M2Z7ojv7vwbSkcrG5Osxr 交付说明 —— 两件活各有一张卡了

**派单**:`disp-M2Z7ojv7vwbSkcrG5Osxr`(Aegis 发)
**提交**:仅本文件

---

## 1. 同步:已经是 behind 0,不用再合

派单要求先同步(落后 42)。**那 42 个提交我在上一单已经合过并推了**
(`87509f7`,两个远端都通)。实测:

```
git rev-parse --short HEAD        → 58961a86
git rev-list --count HEAD..track  → 0
工作树                              → 干净
```

所以这一条已经满足,没有重复合并。

## 2. 两件的 verdict,都核过

| 实现派单 | verdict | 判定 | 谁发的 | 点名自己? |
|---|---|---|---|---|
| `disp-faEWBt8kSGgv-6pJs_Rqw` | `ev-Dq2vTW8WjIU_hQufDDp2F` | pass | `seat:seat-flux-001` | ✅ |
| `disp-E06O6IAaWefF-BOW-CNJd` | `ev-JPEis2MT47OXUnHW1yE5b` | pass | `seat:seat-flux-001` | ✅ |

两条都是 **Flux 发的**(第三方),`payload.dispatch_id` 都点名各自的实现派单。
id 从账本读,没有手打。

## 3. 第一件的卡:已经有了,没有重开

派单说两件都"acceptances 表里 0 行"。**第一件不是 0 行** ——
我上一单(`disp-TXctB3UeAc52uosH_tCAv`)刚开了它:

```
acc-disp-faEWBt8kSGgv-6pJs_Rqw  →  pending operator decision
```

派单 `do_not` 明确说"不要重开已有的卡"。**所以我没动它。**

这里有个信息时差:Aegis 的心跳查询在我开卡之前跑,所以它看到 0 行。
不是它查错,是我这边晚一步。

## 4. 第二件的卡:新开

```
requested acceptance acc-disp-E06O6IAaWefF-BOW-CNJd
  for disp-E06O6IAaWefF-BOW-CNJd
  branch packet/mira/knowledge-panel, base track/infra-foundation
  flux verdict ev-JPEis2MT47OXUnHW1yE5b
  → pending operator decision
```

### `how_to_verify` 按派单要求写

派单特别强调:**"冷启动那一件,他要真的退出应用再打开,不是切页面。"**

我写成了 7 步,前 4 步专测冷启动:

```
1. 在应用里切到一个不是「派单」的页面
2. 完全退出应用进程,不是关窗口     ← 明确写了
3. 重新打开
4. 看左边栏哪一页高亮
```

通过条件:高亮「派单」。拒绝条件:落在别的页(尤其上次退出那一页)。

后 3 步测没被修坏:

```
5. 会话内切到另一个项目
6. 看落在哪一页(应是那个项目自己记住的)
7. 再切回第一个项目
```

**拒绝条件里我写明:切项目后所有项目落在同一页、或切走后记忆丢失,
比第一条更严重 —— 因为那是操作员明确要的功能。**

每一步都能因为我的改动而失败,没有"无论改动在不在都会通过"的步骤。

### precondition 里写明了真实范围

- 分支包含 base(behind 0,已核)
- **必须在真机上跑,且第 2 步要真的退出进程** —— 我的全部证据来自 jsdom 与探针
- Flux 已独立验过
- 已知的 M-4 缺口**已由 `d2fea84` 补上并验证变异会红**,不是拒绝理由

最后一条我特意写进去:M-4 是这个链上刚补的,不写就等于让操作员以为
那个缺口还在。

## 5. 两件活,两张卡,没有合并

| 实现派单 | 卡 |
|---|---|
| `disp-faEWBt8kSGgv-6pJs_Rqw` | `acc-disp-faEWBt8kSGgv-6pJs_Rqw`(上一单开的) |
| `disp-E06O6IAaWefF-BOW-CNJd` | `acc-disp-E06O6IAaWefF-BOW-CNJd`(本单开的) |

## 6. 未做

- **没有 merge into track**(派单 `do_not`)。
- **没有 rebase**。
- **没有撤任何卡**。
- 两件活指向同一分支上的同一套改动,所以两张卡的验收步骤有重叠 ——
  这是派单要求的"一件一卡"的自然结果,我在第二张卡里没有重复写第一张
  已经写过的代码范围细节,只写了它自己的行为断言。
