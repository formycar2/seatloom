# disp-ZXLoO4HzEXDl5a0iR_Ptw 交付说明 —— 两张卡重指到新 verdict

**派单**:`disp-ZXLoO4HzEXDl5a0iR_Ptw`(Lyra 发)
**提交**:无新提交(派单 `do_not` 禁止改产品代码)

---

## 1. 先核两条新 verdict,没有只信派单

```
ev-Aoq81S2TiEaD3upfnF7f0
  verdict = pass
  actor   = seat:seat-flux-001          ← 第三方
  payload.dispatch_id = disp-QgtYChq29ccrCEB5ZCjAH   ← 点名 ✅

ev-FoCADzLuU-Sv55c07r3qW
  verdict = pass
  actor   = seat:seat-flux-001          ← 第三方
  payload.dispatch_id = disp-OQyilLqRjR1fDSWsWE-5n   ← 点名 ✅
```

## 2. 撤了两张,用同一 id 重开

| 卡 | 原指向 | 新指向 |
|---|---|---|
| `acc-disp-QgtYChq29ccrCEB5ZCjAH` | `ev-C9TdgaBZabsJ-PLS_ZEzj`(验 c72e58a) | **`ev-Aoq81S2TiEaD3upfnF7f0`**(验 dcfeee3) |
| `acc-disp-OQyilLqRjR1fDSWsWE-5n` | `ev-qD2lDRgrZbx2D3c_WdUFk`(验 c72e58a) | **`ev-FoCADzLuU-Sv55c07r3qW`**(验 dcfeee3) |

撤卡理由都写清是**换绑不是回退**:

> 中间补的是轮询间隔的断言(4e28a6a),产品代码未变。
> 张先生点接受时依据的必须是关于他要接受的那份代码的 verdict,所以换绑。

**第二张的理由更具体**:我核了 `4e28a6a` 只改了一个测试文件
(`ui/src/App.restartRestore.test.tsx`),产品代码一个字节没动。所以那一条的
换绑纯粹是"判词要对着分支尖",不是"实现变了"。

## 3. `how_to_verify` 和 `acceptance_plan` 原样沿用

派单说"不要重写"。**我只改了 precondition 里的 verdict id 和它验的 commit** ——
留着旧 id 会让张先生点接受时看到一条已被我自己撤掉的判词。

这不是重写验收步骤,是让卡不指向已被自己撤掉的引用。

## 4. 开完自己核了一遍(派单要求)

```
git status --short                    → 空(工作树干净)
scripts/check-cards-clickable.sh      → OK: all 5 pending cards are clickable
git rev-parse HEAD                    → dcfeee3a
git rev-parse origin/...              → dcfeee3a
git rev-list --count HEAD..track      → 0
```

**5 张卡全部可点**(我名下 4 张 + 别人的 1 张),分支 behind 0,本地与 origin 一致。

## 5. 关于"有没有别的路"

派单和我上一单都查过:**没有任何工具能改一张 pending 卡的 `flux_verdict_ref`。**
`request_acceptance` 只在开卡时收这个参数,没有 update。

**撤了重开是唯一路径。** 我这边确认一样,没有查漏。

这意味着一件值得记的事:**每次代码在开卡后有改动,卡就必须撤了重开。**
这两单我已经做了两次。这是个设计缺口 —— 派单说它会记下来。

## 6. 未做

- **一行产品代码都没改**(派单 `do_not`)。
- **没有 merge into track**。
- **没有 rebase**。
- **gitlab 仍未推** —— 已知不可达(内网地址 + 22 端口超时),按纪律没有反复重试。
  origin 已同步到 `dcfeee3`。
