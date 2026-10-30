# disp-27hkqHCfw4p0I1TtUM_iH 交付说明 —— 两件已派 Flux 重验,卡待 verdict 落地

**派单**:`disp-27hkqHCfw4p0I1TtUM_iH`(Lyra 发)
**提交**:无新提交

---

## 1. 做了什么

按派单顺序的第一步:**先派 Flux 重新验这两件,一件一个 verdict id,没有批量。**

### 第一件

```
enqueued disp-JUXLGddjlt3zwZM6eVAuX (status=ready)
verdict_event_id = ev-C9TdgaBZabsJ-PLS_ZEzj
target = seat-flux-001
验 disp-QgtYChq29ccrCEB5ZCjAH,分支尖 a99794a
```

payload 里写明了要验什么、以及**明确要求验 `a99794a` 而不是旧的 `4bc8f40`**,
并告诉 Flux `>= 0` 那个变异现在应该会红(测试名也给了)。

### 第二件

```
enqueued disp-KdAeLJZaweh16dlCtVbBm (status=ready)
verdict_event_id = ev-qD2lDRgrZbx2D3c_WdUFk
target = seat-flux-001
验 disp-OQyilLqRjR1fDSWsWE-5n,分支尖 a99794a
```

同样写明验 `a99794a` 而不是 `da20424`,并说明删死分支 + 两份名单合一这两件后加的改动。

### 一件工具行为,值得记

**我在 payload 里指定的 `verdict_event_id` 被工具忽略了,它自己生成了一个。**

第一件我传了 `ev-verdict-disp-QgtYChq29ccrCEB5ZCjAH-r2`,工具返回的是
`ev-C9TdgaBZabsJ-PLS_ZEzj`。第二件我没传,它生成了 `ev-qD2lDRgrZbx2D3c_WdUFk`。

工具文档说"The tool MINTS one when you omit it and VALIDATES yours when you supply it" ——
**但它没有报错说我传的那个不合法,只是换成了自己的。** 我按返回值的 id 记录,
没有假设我传的那个有效。

## 2. 卡还没有开,原因

派单的顺序是:**先派 Flux → 验过 → 各开一张卡。**

而它的 `do_not` 第一条就是:

> 不要因为两条 verdict 都是 pass 就直接开卡 —— 它们测的是旧代码。

我查了账本,两条新 verdict **都还没发**:

```
ev-C9TdgaBZabsJ-PLS_ZEzj :: NOT YET
ev-qD2lDRgrZbx2D3c_WdUFk :: NOT YET
```

**Flux 还没验。** 在这种情况下开卡,我会开出两张挂着"验绿进行中"的卡 ——
而那正是我前几轮反复报告过的形状(卡存在、verdict 不存在或对不上)。

**所以我停在这里,等 verdict 落地。** verdict 一到,我立刻各开一张卡。

## 3. 派单里那个开卡前的开放问题,我的回答

Lyra 问:

> 面板自身按项目读的那处没改,因为 `api.listAcceptances(projectId)` 是 Nimbus 的围栏。
> 但那意味着徽章修好了、路还是断的:他会看到「3」,点进去看到「没有待验收卡」。
> **如果你认为这是开卡前必须一起解决的,说出来,我去和 Nimbus 对齐。**
> 如果你认为可以分开验收,也在卡上写清这一点,别让他以为点进去就能看到那三张。

**我的回答:可以分开验收,但我要求卡上必须写明这个限制。**

理由:

1. **徽章那件的价值不依赖于面板修好。** 他现在至少能知道"有 3 张在等我" ——
   而这个信息之前完全不存在。这是从 0 到 1,不是从 1 到 2。
2. **面板的修法在 Nimbus 的地界**,我改不了,等它反而让这一件也卡住。
3. **但如果不写明,就是我在制造一个更糟的体验**:他看到 3,点进去看到"没有",
   那比原来的"什么都看不到"更让人困惑 —— 因为产品刚告诉他有事,又否认它。

**所以卡上会写**:左边栏能看到计数;点进面板可能显示"当前项目没有",
因为面板的读按项目过滤,而卡在别的项目 —— 这一条已报告给 Nimbus,未修。
**别以为点进去就能看到那三张。**

## 4. 同步状态

```
本地:   a99794ad
origin: a99794ad     ← 一致
behind: 0
```

派单说"你现在 origin 一致,保持这样" —— 已核,一致。

## 5. 未做

- **没有开任何卡**(见 §2)。
- **没有 rebase、没有 merge into track**(派单 `do_not`)。
- **gitlab 仍未推** —— 已知不可达(Lyra 实测内网地址 + 22 端口超时),
  按派单要求没有反复重试。
