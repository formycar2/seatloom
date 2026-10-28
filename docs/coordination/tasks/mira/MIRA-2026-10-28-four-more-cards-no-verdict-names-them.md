# 关于「四件已完成的活开验收卡」的第二轮 —— 这次是另外四件,结论相同

派单 `disp-owGEYGYCUYcJu-Ls9Qy0V` 说:

> open one acceptance card per dispatch, with `flux_verdict_ref` naming the verdict
> event that exists **for it**

**我逐件核了。四件的 verdict 都没有点名它们自己。我一张卡都没开。**

---

## 一、这次是另外四件(与上一轮不同,我核对了)

| dispatch | 完成时间 | verdict 数 |
|---|---|---|
| `disp-Gf6YwpXeUtdvJizzxtXvo` | 10-27 13:12 | 2 |
| `disp-kF-8yuqK1LDq750ahdhIB` | 10-27 13:17 | 1 |
| `disp-Dp6MAZwDxczKu1VcX_Zeg` | 10-27 13:21 | 1 |
| `disp-XWyYtCk_zY7NLGJQV3ukK` | 10-27 13:41 | 1 |

派单 `what_i_measured_on_10_28_1330` 里的数字我复现了:verdict 数一条不差。

## 二、但没有一条 verdict 点名它自己

派单 `do` 第一条的关键词是 **"for it"**。逐件对 `payload.dispatch_id`:

| 要开卡的派单 | verdict 事件 | verdict 里的 `dispatch_id` | 点名它自己? |
|---|---|---|---|
| `disp-Gf6YwpXeUtdvJizzxtXvo` | `ev-mira-audit-flux-i_ZLKc01` | `disp-i_ZLKc4E2S0Bm78K-Zmdy` | **NO** |
| `disp-Gf6YwpXeUtdvJizzxtXvo` | `ev-xWVqiso1z-r967Io2EXJ8` | `disp-i_ZLKc4E2S0Bm78K-Zmdy` | **NO** |
| `disp-kF-8yuqK1LDq750ahdhIB` | `ev-mira-audit-flux-VH6l1H02` | `disp-VH6l1HixlUVYuy4m0NEp-` | **NO** |
| `disp-Dp6MAZwDxczKu1VcX_Zeg` | `ev-mira-audit-flux-4zePyI03` | `disp-4zePyIT3GboLBELuobHAS` | **NO** |
| `disp-XWyYtCk_zY7NLGJQV3ukK` | `ev-mirror-pipe-guard-flux-verdict-01` | `disp-jZg89gwy8YzVOOjdAsxTL` | **NO** |

**5 条 verdict,0 条点名要开卡的那个派单。**

## 三、而那些被点名的派单,卡都已经开过了

| verdict 指向 | 状态 | 已有卡 |
|---|---|---|
| `disp-i_ZLKc4E2S0Bm78K-Zmdy`(独立验 Nimbus 两个修复) | done | **1** |
| `disp-VH6l1HixlUVYuy4m0NEp-`(三门审计) | done | **1** |
| `disp-4zePyIT3GboLBELuobHAS`(补实现单号) | done | **1** |
| `disp-jZg89gwy8YzVOOjdAsxTL`(mirror pipe 被抢) | done | **1** |

所以这四件开卡,只能是:

1. **重复卡** —— 把 `flux_verdict_ref` 填成一条已经挂在别人卡上的 verdict
2. **张冠李戴卡** —— 声称某条 verdict 为这件活背书,而它审的是另一件
3. **自证卡** —— 前三条 verdict 的 actor 是 `seat-mira-001`(我自己),用它们给
   我自己的审计活开卡,正是 10-27 裁决关掉的那个洞

派单 `do_not` 第一条自己写着:「不要给没有 verdict 事件的派单开卡 —— 一张没有
verdict ref 的卡就是 10-27 裁决关掉的那个完整性漏洞」。**这四件属于同一类的
镜像:有 verdict 事件,但那个 verdict 不为它背书。**

## 四、为什么这个形状会重复出现

因为两个查询方式给出不同答案,而**看起来更自然的那个是错的**:

```
按 ref_type=dispatch + ref_id=<这个派单> 查 ReviewVerdictIssued
   → 每件都"有 verdict"  ← 派单和我第一轮用的都是这个

按同一条事件的 payload.dispatch_id 对
   → 全部对不上          ← 这个才回答"是不是给它判的"
```

`ref_id` 记的是"这条 verdict 挂在谁的上下文里",我作为审计的执行者,verdict
的 ref 自然落在我头上。但 `dispatch_id` 说的是"判的是哪件活"——那是**被审的活**。

**"挂着"和"判的是"是两件事。** 上一轮 6 件、这一轮 4 件,一共 10 件,全是这一个原因。

## 五、我建议怎么真正解决

这四件(加上上一轮那 6 件)都是**审计类派单**:产出是一份判定,不是代码。
按 `docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md`
的门类规则,它们可能本来就不该走交付卡 —— 一条验单的"验收"是什么?验收我
 Audit 得对不对?

如果要卡,需要的是一条**点名它们自己的** verdict,验的是"这个审计动作本身
有没有看错、有没有漏"。那需要有人去审我的审计。

**我建议先定这一类要不要卡,再谈怎么开。** 否则会开出一批不该存在的卡,
或者逼出一批为了凑数而发的 verdict。

## 六、Nimbus 那五件

派单说它们 done 且**零 verdict 事件**,开不出来,并且明说没有解决、不假装解决。

这一点我认:零 verdict 比"有 verdict 但不点名"更清楚,至少不会误导。
我这十件的危险恰在于**看起来有**。

## 七、我做了什么

- 逐件核了 verdict 的 `payload.dispatch_id`(不是只数条数)
- 核了四个被点名派单的状态和已有卡数
- **没有开任何一张卡**
- 没有 merge(派单 `do_not` 第二条)

`request_acceptance` 一次都没调。

## 八、给 Aegis 的下一步

1. **先裁定审计类派单走不走卡**(依据是 10-27 那份门类规则决定)。这是根子。
2. 若裁定要卡:为这四件各派一条验单,验审计动作本身,而不是被审的活。
3. 在根子定下来之前,**再派同样的开卡派单给我,我还会给出同样的答复** ——
   这不是我拒不执行,是这条命令按它自己写的条件(`for it`)无法执行。
