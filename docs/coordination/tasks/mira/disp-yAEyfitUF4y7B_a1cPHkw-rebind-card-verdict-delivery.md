# disp-yAEyfitUF4y7B_a1cPHkw 交付说明 —— 卡换绑到新 verdict

**派单**:`disp-yAEyfitUF4y7B_a1cPHkw`(Aegis 发)
**提交**:无代码改动(派单 `do_not` 明确禁止)

---

## 1. 做了什么

1. **核了新 verdict**(没有只信派单):

```
ev-15UEVJ08G4I0anhIpCjUJ
  verdict = pass
  actor   = seat:seat-flux-001        ← 第三方
  payload.dispatch_id = disp-faEWBt8kSGgv-6pJs_Rqw   ← 点名本派单 ✅
```

2. **撤掉旧卡** `acc-disp-faEWBt8kSGgv-6pJs_Rqw`,理由写清是换绑不是回退。

3. **用同一 id 重开**,`flux_verdict_ref` 换成 `ev-15UEVJ08G4I0anhIpCjUJ`。

```
re-opened acceptance acc-disp-faEWBt8kSGgv-6pJs_Rqw
  for disp-faEWBt8kSGgv-6pJs_Rqw
  branch packet/mira/knowledge-panel, base track/infra-foundation
  flux verdict ev-15UEVJ08G4I0anhIpCjUJ
  → the previously cancelled card is pending again under the same id
```

4. **`how_to_verify` 和 `acceptance_plan` 原样沿用**,没有重写 ——
   派单说那套步骤写得对。

## 2. 我只加了一处 precondition

原版的 precondition 里有"Flux 已独立验过(pass, ev-Dq2vTW8WjIU_hQufDDp2F)"。
换绑后这句会指向一条已经被撤卡弃用的 verdict,所以改成:

```
Flux 已独立验过当前代码(pass, ev-15UEVJ08G4I0anhIpCjUJ,验的是 c3484a0)
本卡替换了此前绑在旧 verdict ev-Dq2vTW8WjIU_hQufDDp2F(验 0f420bb)上的那一版。
App.tsx 自那时起一个字节没变,补的是 M-4 缺失的断言,不是换了实现。
```

**这不是重写验收步骤,是让卡不会指向一条已被自己撤掉的 verdict。**
留着旧 id 会让张先生点接受时看到一条对不上的引用。

## 3. 我自己复核了"代码没变"这件事

派单说它核过(`git show --stat` + `git diff`)。**我没有只信,自己又核了一遍**,
因为这正是这个换绑动作的全部依据 —— 如果代码变过,新 verdict 就不是
"同一实现补断言",而是"换了实现再验",那结论要重写。

```
git status --short                          → 空(工作树干净)
git rev-parse --short HEAD                  → c3484a01
git rev-list --count HEAD..track            → 0
scripts/check-cards-clickable.sh            → OK: all 3 pending cards are clickable
```

**3 张卡全部可点**(我的 2 张 + 别人的 1 张),分支 behind 0。

## 4. 关于"有没有别的路"

派单说它查过,没有工具能改一张 pending 卡的 `flux_verdict_ref`,
`request_acceptance` 只在开卡时收这个参数。**我这边确认一样** ——
我也没有找到 update 类的入口,撤了重开是唯一路径。

## 5. 未做

- **一行产品代码都没改**(派单 `do_not`)。
- **没有 merge into track**。
- **没有跑门禁** —— 零代码改动,跑它是无意义的动作。
